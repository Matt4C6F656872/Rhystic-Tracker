//! Analytical storage layer for Rhystic Cloud.
//!
//! Uses SQLite (WAL mode) by default for lightweight self-hosting and instant
//! zero-config testing, while keeping all SQL standard and portable to PostgreSQL.

use crate::models::{
    AnonymizedMatchPayload, CardPerformanceEntry, CommanderMatchupEntry, CommanderTierEntry,
    FormatMetaSummary, IngestResponse, TitleFrequency,
};
use chrono::Utc;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    Pool, Row, Sqlite,
};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Clone)]
pub struct AnalyticalDb {
    pool: Pool<Sqlite>,
}

impl AnalyticalDb {
    pub async fn connect(db_url: &str) -> Result<Self, sqlx::Error> {
        let pool = if db_url == ":memory:" || db_url.starts_with("sqlite::memory:") {
            SqlitePoolOptions::new()
                .max_connections(1)
                .connect("sqlite::memory:")
                .await?
        } else {
            let clean = db_url.strip_prefix("sqlite://").unwrap_or(db_url);
            let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}", clean))?
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                .synchronous(SqliteSynchronous::Normal)
                .busy_timeout(std::time::Duration::from_secs(10));
            SqlitePoolOptions::new()
                .max_connections(8)
                .connect_with(opts)
                .await?
        };

        let db = Self { pool };
        db.init_schema().await?;
        Ok(db)
    }

    async fn init_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ingested_matches (
                match_hash TEXT PRIMARY KEY,
                schema_version INTEGER NOT NULL,
                contributor_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                format TEXT NOT NULL,
                result TEXT NOT NULL,
                result_reason TEXT,
                duration_seconds INTEGER NOT NULL,
                turns INTEGER NOT NULL,
                going_first INTEGER NOT NULL,
                hero_starting_life INTEGER NOT NULL,
                hero_end_life INTEGER,
                hero_min_life INTEGER,
                commander_grp_id INTEGER,
                companion_grp_id INTEGER,
                opponent_commander_grp_id INTEGER,
                opponent_companion_grp_id INTEGER,
                ingested_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_ingested_matches_format
                ON ingested_matches(format);
            CREATE INDEX IF NOT EXISTS idx_ingested_matches_commander
                ON ingested_matches(commander_grp_id, format);
            CREATE INDEX IF NOT EXISTS idx_ingested_matches_opp_commander
                ON ingested_matches(opponent_commander_grp_id, format);

            CREATE TABLE IF NOT EXISTS ingested_match_cards (
                match_hash TEXT NOT NULL,
                grp_id INTEGER NOT NULL,
                is_opponent INTEGER NOT NULL,
                count INTEGER NOT NULL,
                PRIMARY KEY (match_hash, grp_id, is_opponent)
            );

            CREATE INDEX IF NOT EXISTS idx_ingested_match_cards_grp
                ON ingested_match_cards(grp_id, is_opponent);

            CREATE TABLE IF NOT EXISTS ingested_impactful_cards (
                match_hash TEXT NOT NULL,
                grp_id INTEGER NOT NULL,
                total_damage INTEGER NOT NULL,
                max_hit INTEGER NOT NULL,
                damage_combat INTEGER NOT NULL,
                damage_spell INTEGER NOT NULL,
                cards_drawn INTEGER NOT NULL,
                counters_added INTEGER NOT NULL,
                titles_json TEXT NOT NULL,
                PRIMARY KEY (match_hash, grp_id)
            );

            CREATE INDEX IF NOT EXISTS idx_ingested_impactful_grp
                ON ingested_impactful_cards(grp_id);
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Validates a payload to ensure basic integrity and privacy invariants.
    /// Explicitly excludes Alchemy, Bot Matches, Color Challenge, and Precon events.
    pub fn validate_payload(p: &AnonymizedMatchPayload) -> bool {
        if p.match_hash.trim().is_empty() || p.match_hash.len() > 128 {
            return false;
        }
        if p.contributor_id.trim().is_empty() || p.contributor_id.len() > 128 {
            return false;
        }
        let fmt_trimmed = p.format.trim();
        if fmt_trimmed.is_empty() || fmt_trimmed.len() > 64 {
            return false;
        }
        let fmt_lower = fmt_trimmed.to_ascii_lowercase();
        if fmt_lower.contains("alchemy")
            || fmt_lower.contains("bot match")
            || fmt_lower.contains("color challenge")
            || fmt_lower.contains("dualcolorprecon")
        {
            return false;
        }
        let res_lower = p.result.trim().to_ascii_lowercase();
        if res_lower != "win" && res_lower != "loss" {
            return false;
        }
        if p.turns == 0 {
            return false;
        }
        true
    }

    fn normalize_result(result: &str) -> &'static str {
        if result.trim().eq_ignore_ascii_case("win") {
            "Win"
        } else {
            "Loss"
        }
    }

    fn default_starting_life(format: &str) -> i32 {
        let lower = format.to_ascii_lowercase();
        if lower.contains("commander") && !lower.contains("brawl") {
            40
        } else if lower.contains("brawl") {
            25
        } else {
            20
        }
    }

    /// Idempotently ingests a batch of anonymized match payloads inside a transaction.
    pub async fn ingest_batch(
        &self,
        payloads: &[AnonymizedMatchPayload],
    ) -> Result<IngestResponse, sqlx::Error> {
        let mut accepted = 0usize;
        let mut duplicates = 0usize;
        let mut rejected = 0usize;
        let mut accepted_hashes = Vec::new();
        let now = Utc::now().to_rfc3339();

        let mut tx = self.pool.begin().await?;

        for p in payloads {
            if !Self::validate_payload(p) {
                rejected += 1;
                continue;
            }

            let normalized_result = Self::normalize_result(&p.result);
            let starting_life = p
                .hero_starting_life
                .unwrap_or_else(|| Self::default_starting_life(&p.format));

            let res = sqlx::query(
                r#"
                INSERT INTO ingested_matches (
                    match_hash, schema_version, contributor_id, timestamp, format,
                    result, result_reason, duration_seconds, turns, going_first,
                    hero_starting_life, hero_end_life, hero_min_life,
                    commander_grp_id, companion_grp_id,
                    opponent_commander_grp_id, opponent_companion_grp_id,
                    ingested_at
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(match_hash) DO NOTHING
                "#,
            )
            .bind(&p.match_hash)
            .bind(p.schema_version as i64)
            .bind(&p.contributor_id)
            .bind(&p.timestamp)
            .bind(&p.format)
            .bind(normalized_result)
            .bind(&p.result_reason)
            .bind(p.duration_seconds as i64)
            .bind(p.turns as i64)
            .bind(if p.going_first { 1i64 } else { 0i64 })
            .bind(starting_life as i64)
            .bind(p.hero_life_end.map(|v| v as i64))
            .bind(p.hero_min_life.map(|v| v as i64))
            .bind(p.hero_commander_id.filter(|&id| id > 0).map(|v| v as i64))
            .bind(p.companion_grp_id.filter(|&id| id > 0).map(|v| v as i64))
            .bind(
                p.opponent_commander_id
                    .filter(|&id| id > 0)
                    .map(|v| v as i64),
            )
            .bind(
                p.opponent_companion_grp_id
                    .filter(|&id| id > 0)
                    .map(|v| v as i64),
            )
            .bind(&now)
            .execute(&mut *tx)
            .await?;

            if res.rows_affected() == 0 {
                duplicates += 1;
                accepted_hashes.push(p.match_hash.clone());
                continue;
            }

            for c in &p.cards_seen {
                if c.grp_id == 0 {
                    continue;
                }
                sqlx::query(
                    r#"
                    INSERT INTO ingested_match_cards (match_hash, grp_id, is_opponent, count)
                    VALUES (?, ?, ?, ?)
                    ON CONFLICT(match_hash, grp_id, is_opponent) DO UPDATE SET
                        count = MAX(ingested_match_cards.count, excluded.count)
                    "#,
                )
                .bind(&p.match_hash)
                .bind(c.grp_id as i64)
                .bind(if c.is_opponent { 1i64 } else { 0i64 })
                .bind(c.count as i64)
                .execute(&mut *tx)
                .await?;
            }

            for imp in &p.impactful_cards {
                if imp.grp_id == 0 {
                    continue;
                }
                let titles_json =
                    serde_json::to_string(&imp.titles).unwrap_or_else(|_| "[]".to_string());
                sqlx::query(
                    r#"
                    INSERT INTO ingested_impactful_cards (
                        match_hash, grp_id, total_damage, max_hit,
                        damage_combat, damage_spell, cards_drawn, counters_added, titles_json
                    )
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ON CONFLICT(match_hash, grp_id) DO NOTHING
                    "#,
                )
                .bind(&p.match_hash)
                .bind(imp.grp_id as i64)
                .bind(imp.total_damage as i64)
                .bind(imp.max_hit as i64)
                .bind(imp.damage_combat as i64)
                .bind(imp.damage_spell as i64)
                .bind(imp.cards_drawn)
                .bind(imp.counters_added)
                .bind(titles_json)
                .execute(&mut *tx)
                .await?;
            }

            accepted += 1;
            accepted_hashes.push(p.match_hash.clone());
        }

        tx.commit().await?;

        Ok(IngestResponse {
            accepted,
            duplicates,
            rejected,
            accepted_hashes,
        })
    }

    pub async fn get_health_stats(&self) -> Result<(i64, i64, i64, i64, i64), sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) AS total_matches,
                COUNT(DISTINCT contributor_id) AS unique_contributors,
                SUM(CASE WHEN commander_grp_id IS NOT NULL OR LOWER(format) LIKE '%commander%' OR LOWER(format) LIKE '%brawl%' OR LOWER(format) LIKE '%gladiator%' THEN 1 ELSE 0 END) AS commander_matches,
                SUM(CASE WHEN LOWER(format) LIKE '%draft%' OR LOWER(format) LIKE '%sealed%' OR LOWER(format) LIKE '%limited%' THEN 1 ELSE 0 END) AS limited_matches
            FROM ingested_matches
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let total_matches: i64 = row.try_get("total_matches").unwrap_or(0);
        let unique_contributors: i64 = row.try_get("unique_contributors").unwrap_or(0);
        let commander_matches: i64 = row
            .try_get::<Option<i64>, _>("commander_matches")
            .ok()
            .flatten()
            .unwrap_or(0);
        let limited_matches: i64 = row
            .try_get::<Option<i64>, _>("limited_matches")
            .ok()
            .flatten()
            .unwrap_or(0);
        let constructed_matches = (total_matches - limited_matches).max(0);

        Ok((
            total_matches,
            unique_contributors,
            constructed_matches,
            limited_matches,
            commander_matches,
        ))
    }

    pub async fn get_format_meta(
        &self,
        category_filter: Option<&str>,
        active_only: bool,
    ) -> Result<Vec<FormatMetaSummary>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                format,
                COUNT(*) AS total_matches,
                AVG(CAST(turns AS REAL)) AS avg_turns,
                AVG(CAST(duration_seconds AS REAL)) AS avg_duration,
                SUM(CASE WHEN going_first = 1 THEN 1 ELSE 0 END) AS play_games,
                SUM(CASE WHEN going_first = 1 AND result = 'Win' THEN 1 ELSE 0 END) AS play_wins,
                SUM(CASE WHEN going_first = 0 THEN 1 ELSE 0 END) AS draw_games,
                SUM(CASE WHEN going_first = 0 AND result = 'Win' THEN 1 ELSE 0 END) AS draw_wins,
                SUM(CASE WHEN result = 'Win' THEN 1 ELSE 0 END) AS total_wins,
                SUM(CASE WHEN result = 'Win' AND LOWER(COALESCE(result_reason, '')) LIKE '%concede%' THEN 1 ELSE 0 END) AS concede_wins,
                COUNT(DISTINCT commander_grp_id) AS unique_commanders
            FROM ingested_matches
            WHERE LOWER(format) NOT LIKE '%alchemy%'
            GROUP BY format
            ORDER BY total_matches DESC, format ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut by_lower: HashMap<String, FormatMetaSummary> = HashMap::new();
        for r in rows {
            let fmt: String = r.get("format");
            let total_matches: i64 = r.get("total_matches");
            let play_games: i64 = r.try_get("play_games").unwrap_or(0);
            let play_wins: i64 = r.try_get("play_wins").unwrap_or(0);
            let draw_games: i64 = r.try_get("draw_games").unwrap_or(0);
            let draw_wins: i64 = r.try_get("draw_wins").unwrap_or(0);
            let total_wins: i64 = r.try_get("total_wins").unwrap_or(0);
            let concede_wins: i64 = r.try_get("concede_wins").unwrap_or(0);

            by_lower.insert(
                fmt.to_ascii_lowercase(),
                FormatMetaSummary {
                    category: classify_format_category(&fmt).to_string(),
                    format: fmt,
                    total_matches,
                    wins: total_wins,
                    win_rate: if total_matches > 0 {
                        round4(total_wins as f64 / total_matches as f64)
                    } else {
                        0.0
                    },
                    avg_turns: round2(r.try_get("avg_turns").unwrap_or(0.0)),
                    avg_duration_seconds: round2(r.try_get("avg_duration").unwrap_or(0.0)),
                    play_win_rate: if play_games > 0 {
                        round4(play_wins as f64 / play_games as f64)
                    } else {
                        0.0
                    },
                    draw_win_rate: if draw_games > 0 {
                        round4(draw_wins as f64 / draw_games as f64)
                    } else {
                        0.0
                    },
                    concede_rate: if total_wins > 0 {
                        round4(concede_wins as f64 / total_wins as f64)
                    } else {
                        0.0
                    },
                    unique_commanders: r.try_get("unique_commanders").unwrap_or(0),
                },
            );
        }

        // Seed canonical MTGA Constructed & Limited formats (excluding Alchemy)
        for &(canonical_fmt, category) in SUPPORTED_MTGA_FORMATS {
            let key = canonical_fmt.to_ascii_lowercase();
            by_lower.entry(key).or_insert_with(|| FormatMetaSummary {
                format: canonical_fmt.to_string(),
                category: category.to_string(),
                total_matches: 0,
                wins: 0,
                win_rate: 0.0,
                avg_turns: 0.0,
                avg_duration_seconds: 0.0,
                play_win_rate: 0.0,
                draw_win_rate: 0.0,
                concede_rate: 0.0,
                unique_commanders: 0,
            });
        }

        let cat_filter = category_filter
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty());

        let mut out: Vec<FormatMetaSummary> = by_lower
            .into_values()
            .filter(|entry| {
                if active_only && entry.total_matches == 0 {
                    return false;
                }
                if let Some(ref cf) = cat_filter {
                    let entry_cat = entry.category.to_ascii_lowercase();
                    if cf == "constructed" {
                        // Constructed includes both 60-card Constructed and Commander & Brawl
                        entry_cat.contains("constructed") || entry_cat.contains("commander")
                    } else {
                        entry_cat.contains(cf.as_str())
                    }
                } else {
                    true
                }
            })
            .collect();

        out.sort_by(|a, b| {
            b.total_matches
                .cmp(&a.total_matches)
                .then_with(|| format_sort_rank(&a.format).cmp(&format_sort_rank(&b.format)))
                .then_with(|| a.format.cmp(&b.format))
        });

        Ok(out)
    }

    pub async fn get_commander_tier_list(
        &self,
        format_filter: Option<&str>,
        min_matches: i64,
    ) -> Result<Vec<CommanderTierEntry>, sqlx::Error> {
        let min_m = min_matches.max(1);
        let rows = if let Some(fmt) = format_filter.filter(|s| !s.trim().is_empty()) {
            sqlx::query(
                r#"
                SELECT
                    commander_grp_id,
                    format,
                    COUNT(*) AS matches,
                    SUM(CASE WHEN result = 'Win' THEN 1 ELSE 0 END) AS wins,
                    SUM(CASE WHEN going_first = 1 THEN 1 ELSE 0 END) AS play_games,
                    SUM(CASE WHEN going_first = 1 AND result = 'Win' THEN 1 ELSE 0 END) AS play_wins,
                    SUM(CASE WHEN going_first = 0 THEN 1 ELSE 0 END) AS draw_games,
                    SUM(CASE WHEN going_first = 0 AND result = 'Win' THEN 1 ELSE 0 END) AS draw_wins,
                    AVG(CAST(turns AS REAL)) AS avg_turns,
                    SUM(CASE WHEN result = 'Win' AND LOWER(COALESCE(result_reason, '')) LIKE '%concede%' THEN 1 ELSE 0 END) AS opp_scoops
                FROM ingested_matches
                WHERE commander_grp_id IS NOT NULL AND commander_grp_id > 0
                  AND LOWER(format) = LOWER(?)
                GROUP BY commander_grp_id, format
                HAVING COUNT(*) >= ?
                ORDER BY (CAST(SUM(CASE WHEN result = 'Win' THEN 1 ELSE 0 END) AS REAL) / COUNT(*)) DESC, matches DESC
                "#,
            )
            .bind(fmt)
            .bind(min_m)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT
                    commander_grp_id,
                    format,
                    COUNT(*) AS matches,
                    SUM(CASE WHEN result = 'Win' THEN 1 ELSE 0 END) AS wins,
                    SUM(CASE WHEN going_first = 1 THEN 1 ELSE 0 END) AS play_games,
                    SUM(CASE WHEN going_first = 1 AND result = 'Win' THEN 1 ELSE 0 END) AS play_wins,
                    SUM(CASE WHEN going_first = 0 THEN 1 ELSE 0 END) AS draw_games,
                    SUM(CASE WHEN going_first = 0 AND result = 'Win' THEN 1 ELSE 0 END) AS draw_wins,
                    AVG(CAST(turns AS REAL)) AS avg_turns,
                    SUM(CASE WHEN result = 'Win' AND LOWER(COALESCE(result_reason, '')) LIKE '%concede%' THEN 1 ELSE 0 END) AS opp_scoops
                FROM ingested_matches
                WHERE commander_grp_id IS NOT NULL AND commander_grp_id > 0
                GROUP BY commander_grp_id, format
                HAVING COUNT(*) >= ?
                ORDER BY (CAST(SUM(CASE WHEN result = 'Win' THEN 1 ELSE 0 END) AS REAL) / COUNT(*)) DESC, matches DESC
                "#,
            )
            .bind(min_m)
            .fetch_all(&self.pool)
            .await?
        };

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let matches: i64 = r.get("matches");
            let wins: i64 = r.get("wins");
            let play_games: i64 = r.try_get("play_games").unwrap_or(0);
            let play_wins: i64 = r.try_get("play_wins").unwrap_or(0);
            let draw_games: i64 = r.try_get("draw_games").unwrap_or(0);
            let draw_wins: i64 = r.try_get("draw_wins").unwrap_or(0);
            let opp_scoops: i64 = r.try_get("opp_scoops").unwrap_or(0);

            let win_rate = if matches > 0 {
                round4(wins as f64 / matches as f64)
            } else {
                0.0
            };

            let tier = classify_commander_tier(win_rate, matches);

            out.push(CommanderTierEntry {
                commander_grp_id: r.get::<i64, _>("commander_grp_id") as u32,
                format: r.get("format"),
                matches,
                wins,
                win_rate,
                play_win_rate: if play_games > 0 {
                    round4(play_wins as f64 / play_games as f64)
                } else {
                    0.0
                },
                draw_win_rate: if draw_games > 0 {
                    round4(draw_wins as f64 / draw_games as f64)
                } else {
                    0.0
                },
                avg_turns: round2(r.try_get("avg_turns").unwrap_or(0.0)),
                salt_index: if matches > 0 {
                    round4(opp_scoops as f64 / matches as f64)
                } else {
                    0.0
                },
                tier: tier.to_string(),
            });
        }

        Ok(out)
    }

    pub async fn get_card_performance(
        &self,
        format_filter: Option<&str>,
        commander_grp_id: Option<u32>,
        limit: i64,
    ) -> Result<Vec<CardPerformanceEntry>, sqlx::Error> {
        let max_rows = limit.clamp(1, 500);
        let fmt_opt = format_filter.filter(|s| !s.trim().is_empty()).unwrap_or("");
        let cmd_opt = commander_grp_id.unwrap_or(0) as i64;

        let rows = sqlx::query(
            r#"
            SELECT
                c.grp_id AS grp_id,
                COUNT(DISTINCT c.match_hash) AS matches_seen,
                SUM(CASE WHEN m.result = 'Win' THEN 1 ELSE 0 END) AS wins_when_seen,
                COALESCE(SUM(ic.total_damage), 0) AS total_damage,
                COALESCE(MAX(ic.max_hit), 0) AS max_single_hit,
                COALESCE(SUM(ic.damage_combat), 0) AS combat_damage,
                COALESCE(SUM(ic.damage_spell), 0) AS spell_damage,
                COALESCE(SUM(ic.cards_drawn), 0) AS total_cards_drawn
            FROM ingested_match_cards c
            INNER JOIN ingested_matches m ON m.match_hash = c.match_hash
            LEFT JOIN ingested_impactful_cards ic
                ON ic.match_hash = c.match_hash AND ic.grp_id = c.grp_id
            WHERE c.is_opponent = 0
              AND (? = '' OR LOWER(m.format) = LOWER(?))
              AND (? = 0 OR m.commander_grp_id = ?)
            GROUP BY c.grp_id
            ORDER BY matches_seen DESC, wins_when_seen DESC
            LIMIT ?
            "#,
        )
        .bind(fmt_opt)
        .bind(fmt_opt)
        .bind(cmd_opt)
        .bind(cmd_opt)
        .bind(max_rows)
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let grp_id = r.get::<i64, _>("grp_id") as u32;
            let matches_seen: i64 = r.get("matches_seen");
            let wins_when_seen: i64 = r.get("wins_when_seen");

            let title_rows = sqlx::query(
                r#"
                SELECT ic.titles_json
                FROM ingested_impactful_cards ic
                INNER JOIN ingested_matches m ON m.match_hash = ic.match_hash
                WHERE ic.grp_id = ?
                  AND (? = '' OR LOWER(m.format) = LOWER(?))
                  AND (? = 0 OR m.commander_grp_id = ?)
                "#,
            )
            .bind(grp_id as i64)
            .bind(fmt_opt)
            .bind(fmt_opt)
            .bind(cmd_opt)
            .bind(cmd_opt)
            .fetch_all(&self.pool)
            .await?;

            let mut title_counts: HashMap<String, u64> = HashMap::new();
            for tr in title_rows {
                let raw: String = tr.try_get("titles_json").unwrap_or_default();
                if let Ok(titles) = serde_json::from_str::<Vec<String>>(&raw) {
                    for t in titles {
                        if !t.trim().is_empty() {
                            *title_counts.entry(t).or_insert(0) += 1;
                        }
                    }
                }
            }
            let mut top_titles: Vec<TitleFrequency> = title_counts
                .into_iter()
                .map(|(title, count)| TitleFrequency { title, count })
                .collect();
            top_titles.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.title.cmp(&b.title)));
            top_titles.truncate(5);

            out.push(CardPerformanceEntry {
                grp_id,
                matches_seen,
                wins_when_seen,
                seen_win_rate: if matches_seen > 0 {
                    round4(wins_when_seen as f64 / matches_seen as f64)
                } else {
                    0.0
                },
                total_damage: r.try_get("total_damage").unwrap_or(0),
                max_single_hit: r.try_get::<i64, _>("max_single_hit").unwrap_or(0) as i32,
                combat_damage: r.try_get("combat_damage").unwrap_or(0),
                spell_damage: r.try_get("spell_damage").unwrap_or(0),
                total_cards_drawn: r.try_get("total_cards_drawn").unwrap_or(0),
                top_titles,
            });
        }

        Ok(out)
    }

    pub async fn get_commander_matchups(
        &self,
        commander_grp_id: u32,
        format_filter: Option<&str>,
    ) -> Result<Vec<CommanderMatchupEntry>, sqlx::Error> {
        let fmt_opt = format_filter.filter(|s| !s.trim().is_empty()).unwrap_or("");
        let rows = sqlx::query(
            r#"
            SELECT
                commander_grp_id,
                opponent_commander_grp_id,
                COUNT(*) AS matches,
                SUM(CASE WHEN result = 'Win' THEN 1 ELSE 0 END) AS wins,
                AVG(CAST(turns AS REAL)) AS avg_turns
            FROM ingested_matches
            WHERE commander_grp_id = ?
              AND opponent_commander_grp_id IS NOT NULL
              AND opponent_commander_grp_id > 0
              AND (? = '' OR LOWER(format) = LOWER(?))
            GROUP BY commander_grp_id, opponent_commander_grp_id
            ORDER BY matches DESC, wins DESC
            "#,
        )
        .bind(commander_grp_id as i64)
        .bind(fmt_opt)
        .bind(fmt_opt)
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let matches: i64 = r.get("matches");
            let wins: i64 = r.get("wins");
            out.push(CommanderMatchupEntry {
                commander_grp_id: r.get::<i64, _>("commander_grp_id") as u32,
                opponent_commander_grp_id: r.get::<i64, _>("opponent_commander_grp_id") as u32,
                matches,
                wins,
                win_rate: if matches > 0 {
                    round4(wins as f64 / matches as f64)
                } else {
                    0.0
                },
                avg_turns: round2(r.try_get("avg_turns").unwrap_or(0.0)),
            });
        }

        Ok(out)
    }
}

/// Canonical MTGA Constructed & Limited formats supported by Rhystic Cloud (Alchemy excluded).
pub const SUPPORTED_MTGA_FORMATS: &[(&str, &str)] = &[
    // Commander & Brawl
    ("Commander", "Commander & Brawl"),
    ("Commander - Competitive", "Commander & Brawl"),
    ("Brawl", "Commander & Brawl"),
    ("Brawl - Competitive", "Commander & Brawl"),
    ("Standard Brawl", "Commander & Brawl"),
    ("Gladiator", "Commander & Brawl"),
    // Constructed (60-Card)
    ("Standard", "Constructed"),
    ("Standard Ranked", "Constructed"),
    ("Pioneer", "Constructed"),
    ("Pioneer Ranked", "Constructed"),
    ("Explorer", "Constructed"),
    ("Explorer Ranked", "Constructed"),
    ("Historic", "Constructed"),
    ("Historic Ranked", "Constructed"),
    ("Timeless", "Constructed"),
    ("Timeless Ranked", "Constructed"),
    ("Midweek Magic", "Constructed"),
    ("Direct Challenge", "Constructed"),
    // Limited
    ("Draft", "Limited"),
    ("Premier Draft", "Limited"),
    ("Quick Draft", "Limited"),
    ("Traditional Draft", "Limited"),
    ("Sealed", "Limited"),
    ("Traditional Sealed", "Limited"),
];

pub fn classify_format_category(format: &str) -> &'static str {
    let lower = format.trim().to_ascii_lowercase();
    if lower.contains("commander") || lower.contains("brawl") || lower.contains("gladiator") {
        "Commander & Brawl"
    } else if lower.contains("draft") || lower.contains("sealed") || lower.contains("limited") {
        "Limited"
    } else {
        "Constructed"
    }
}

fn format_sort_rank(format: &str) -> usize {
    SUPPORTED_MTGA_FORMATS
        .iter()
        .position(|(name, _)| name.eq_ignore_ascii_case(format))
        .unwrap_or(999)
}

fn classify_commander_tier(win_rate: f64, _matches: i64) -> &'static str {
    if win_rate >= 0.60 {
        "S"
    } else if win_rate >= 0.54 {
        "A"
    } else if win_rate >= 0.48 {
        "B"
    } else if win_rate >= 0.42 {
        "C"
    } else {
        "D"
    }
}

fn round2(val: f64) -> f64 {
    (val * 100.0).round() / 100.0
}

fn round4(val: f64) -> f64 {
    (val * 10000.0).round() / 10000.0
}
