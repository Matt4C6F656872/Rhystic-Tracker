//! Analytical storage layer for Rhystic Cloud.
//!
//! Uses SQLite (WAL mode) by default for lightweight self-hosting and instant
//! zero-config testing, while keeping all SQL standard and portable to PostgreSQL.

use crate::models::{
    AnonymizedMatchPayload, ArchetypeCardStat, ArchetypeMetaEntry, CardPerformanceEntry,
    CombatRecordEntry, CommanderMatchupEntry, CommanderTierEntry, DeckVariantEntry,
    FormatMetaSummary, GlobalLeaderboardsResponse, IngestResponse, LeaderboardPlayerEntry,
    SyncDeckCardEntry, TitleFrequency,
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
                player_handle TEXT,
                player_account_hash TEXT,
                player_rank TEXT,
                deck_hash TEXT,
                archetype_hash TEXT,
                color_identity TEXT,
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

            CREATE TABLE IF NOT EXISTS ingested_card_names (
                grp_id INTEGER PRIMARY KEY,
                card_name TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS ingested_deck_cards (
                match_hash TEXT NOT NULL,
                grp_id INTEGER NOT NULL,
                count INTEGER NOT NULL,
                is_basic_land INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (match_hash, grp_id)
            );

            CREATE INDEX IF NOT EXISTS idx_ingested_deck_cards_grp
                ON ingested_deck_cards(grp_id);

            CREATE TABLE IF NOT EXISTS claimed_handles (
                normalized_handle TEXT PRIMARY KEY,
                display_handle TEXT NOT NULL,
                player_account_hash TEXT NOT NULL,
                owner_token_hash TEXT NOT NULL,
                claimed_at TEXT NOT NULL,
                last_seen_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_claimed_handles_account_hash
                ON claimed_handles(player_account_hash);
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Idempotent column migrations for existing Phase 4 SQLite databases
        let alter_cols = [
            "ALTER TABLE ingested_matches ADD COLUMN player_handle TEXT",
            "ALTER TABLE ingested_matches ADD COLUMN player_account_hash TEXT",
            "ALTER TABLE ingested_matches ADD COLUMN player_rank TEXT",
            "ALTER TABLE ingested_matches ADD COLUMN deck_hash TEXT",
            "ALTER TABLE ingested_matches ADD COLUMN archetype_hash TEXT",
            "ALTER TABLE ingested_matches ADD COLUMN color_identity TEXT",
        ];
        for stmt in alter_cols {
            let _ = sqlx::query(stmt).execute(&self.pool).await;
        }

        let _ = sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_ingested_matches_archetype
                ON ingested_matches(archetype_hash, format);
            CREATE INDEX IF NOT EXISTS idx_ingested_matches_player
                ON ingested_matches(player_account_hash);
            "#,
        )
        .execute(&self.pool)
        .await;

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

            // Record any card names provided in the payload
            if let (Some(cid), Some(cname)) = (
                p.hero_commander_id.filter(|&id| id > 0),
                p.hero_commander_name.as_deref().filter(|s| !s.trim().is_empty()),
            ) {
                let _ = sqlx::query(
                    "INSERT INTO ingested_card_names (grp_id, card_name) VALUES (?, ?) ON CONFLICT(grp_id) DO UPDATE SET card_name = excluded.card_name",
                )
                .bind(cid as i64)
                .bind(cname.trim())
                .execute(&mut *tx)
                .await;
            }
            if let (Some(cid), Some(cname)) = (
                p.opponent_commander_id.filter(|&id| id > 0),
                p.opponent_commander_name.as_deref().filter(|s| !s.trim().is_empty()),
            ) {
                let _ = sqlx::query(
                    "INSERT INTO ingested_card_names (grp_id, card_name) VALUES (?, ?) ON CONFLICT(grp_id) DO UPDATE SET card_name = excluded.card_name",
                )
                .bind(cid as i64)
                .bind(cname.trim())
                .execute(&mut *tx)
                .await;
            }
            for sc in &p.cards_seen {
                if sc.grp_id > 0 {
                    if let Some(cname) = sc.card_name.as_deref().filter(|s| !s.trim().is_empty()) {
                        let _ = sqlx::query(
                            "INSERT INTO ingested_card_names (grp_id, card_name) VALUES (?, ?) ON CONFLICT(grp_id) DO UPDATE SET card_name = excluded.card_name",
                        )
                        .bind(sc.grp_id as i64)
                        .bind(cname.trim())
                        .execute(&mut *tx)
                        .await;
                    }
                }
            }
            for ic in &p.impactful_cards {
                if ic.grp_id > 0 {
                    if let Some(cname) = ic.card_name.as_deref().filter(|s| !s.trim().is_empty()) {
                        let _ = sqlx::query(
                            "INSERT INTO ingested_card_names (grp_id, card_name) VALUES (?, ?) ON CONFLICT(grp_id) DO UPDATE SET card_name = excluded.card_name",
                        )
                        .bind(ic.grp_id as i64)
                        .bind(cname.trim())
                        .execute(&mut *tx)
                        .await;
                    }
                }
            }
            for dc in &p.deck_cards {
                if dc.grp_id > 0 {
                    if let Some(cname) = dc.card_name.as_deref().filter(|s| !s.trim().is_empty()) {
                        let _ = sqlx::query(
                            "INSERT INTO ingested_card_names (grp_id, card_name) VALUES (?, ?) ON CONFLICT(grp_id) DO UPDATE SET card_name = excluded.card_name",
                        )
                        .bind(dc.grp_id as i64)
                        .bind(cname.trim())
                        .execute(&mut *tx)
                        .await;
                    }
                    if dc.count > 0 {
                        let _ = sqlx::query(
                            r#"
                            INSERT INTO ingested_deck_cards (match_hash, grp_id, count, is_basic_land)
                            VALUES (?, ?, ?, ?)
                            ON CONFLICT(match_hash, grp_id) DO UPDATE SET
                                count = excluded.count,
                                is_basic_land = excluded.is_basic_land
                            "#,
                        )
                        .bind(&p.match_hash)
                        .bind(dc.grp_id as i64)
                        .bind(dc.count as i64)
                        .bind(if dc.is_basic_land { 1i64 } else { 0i64 })
                        .execute(&mut *tx)
                        .await;
                    }
                }
            }

            let normalized_result = Self::normalize_result(&p.result);
            let starting_life = p
                .hero_starting_life
                .unwrap_or_else(|| Self::default_starting_life(&p.format));

            // Verify or pin handle ownership via claimed_handles (First-Seen Token Pinning)
            let raw_handle = p.player_handle.as_deref().map(str::trim).filter(|s| !s.is_empty());
            let raw_acc_hash = p
                .player_account_hash
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty());
            let raw_rank = p.player_rank.as_deref().map(str::trim).filter(|s| !s.is_empty());

            let (verified_handle, verified_acc_hash, verified_rank): (
                Option<&str>,
                Option<&str>,
                Option<&str>,
            ) = if let (Some(h), Some(acc)) = (raw_handle, raw_acc_hash) {
                let norm_handle = h.to_ascii_lowercase();
                let token_hash = hash_contributor_token(&p.contributor_id);

                let existing_claims = sqlx::query(
                    "SELECT owner_token_hash FROM claimed_handles WHERE normalized_handle = ? OR player_account_hash = ?",
                )
                .bind(&norm_handle)
                .bind(acc)
                .fetch_all(&mut *tx)
                .await?;

                if existing_claims.is_empty() {
                    let _ = sqlx::query(
                        r#"
                        INSERT INTO claimed_handles (
                            normalized_handle, display_handle, player_account_hash,
                            owner_token_hash, claimed_at, last_seen_at
                        )
                        VALUES (?, ?, ?, ?, ?, ?)
                        "#,
                    )
                    .bind(&norm_handle)
                    .bind(h)
                    .bind(acc)
                    .bind(&token_hash)
                    .bind(&now)
                    .bind(&now)
                    .execute(&mut *tx)
                    .await?;
                    (Some(h), Some(acc), raw_rank)
                } else if existing_claims
                    .iter()
                    .all(|r| r.get::<String, _>("owner_token_hash") == token_hash)
                {
                    let _ = sqlx::query(
                        "UPDATE claimed_handles SET last_seen_at = ?, display_handle = ? WHERE normalized_handle = ? AND owner_token_hash = ?",
                    )
                    .bind(&now)
                    .bind(h)
                    .bind(&norm_handle)
                    .bind(&token_hash)
                    .execute(&mut *tx)
                    .await;
                    (Some(h), Some(acc), raw_rank)
                } else {
                    // Spoof attempt detected: handle or account_hash is already pinned to another contributor token.
                    // Strip identity attribution while preserving anonymous match telemetry.
                    (None, None, None)
                }
            } else {
                (None, None, None)
            };

            let res = sqlx::query(
                r#"
                INSERT INTO ingested_matches (
                    match_hash, schema_version, contributor_id, timestamp, format,
                    result, result_reason, duration_seconds, turns, going_first,
                    hero_starting_life, hero_end_life, hero_min_life,
                    commander_grp_id, companion_grp_id,
                    opponent_commander_grp_id, opponent_companion_grp_id,
                    player_handle, player_account_hash, player_rank,
                    deck_hash, archetype_hash, color_identity,
                    ingested_at
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
            .bind(verified_handle)
            .bind(verified_acc_hash)
            .bind(verified_rank)
            .bind(p.deck_hash.as_deref().filter(|s| !s.trim().is_empty()))
            .bind(p.archetype_hash.as_deref().filter(|s| !s.trim().is_empty()))
            .bind(p.color_identity.as_deref().filter(|s| !s.trim().is_empty()))
            .bind(&now)
            .execute(&mut *tx)
            .await?;

            if res.rows_affected() == 0 {
                // Enrich existing row if new Phase 4.5 fields were provided by the same contributor
                let _ = sqlx::query(
                    r#"
                    UPDATE ingested_matches
                    SET player_handle = COALESCE(?, player_handle),
                        player_account_hash = COALESCE(?, player_account_hash),
                        player_rank = COALESCE(?, player_rank),
                        deck_hash = COALESCE(?, deck_hash),
                        archetype_hash = COALESCE(?, archetype_hash),
                        color_identity = COALESCE(?, color_identity)
                    WHERE match_hash = ? AND contributor_id = ?
                    "#,
                )
                .bind(verified_handle)
                .bind(verified_acc_hash)
                .bind(verified_rank)
                .bind(p.deck_hash.as_deref().filter(|s| !s.trim().is_empty()))
                .bind(p.archetype_hash.as_deref().filter(|s| !s.trim().is_empty()))
                .bind(p.color_identity.as_deref().filter(|s| !s.trim().is_empty()))
                .bind(&p.match_hash)
                .bind(&p.contributor_id)
                .execute(&mut *tx)
                .await;

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
                    m.commander_grp_id AS commander_grp_id,
                    cn.card_name AS commander_name,
                    m.format AS format,
                    COUNT(*) AS matches,
                    SUM(CASE WHEN m.result = 'Win' THEN 1 ELSE 0 END) AS wins,
                    SUM(CASE WHEN m.going_first = 1 THEN 1 ELSE 0 END) AS play_games,
                    SUM(CASE WHEN m.going_first = 1 AND m.result = 'Win' THEN 1 ELSE 0 END) AS play_wins,
                    SUM(CASE WHEN m.going_first = 0 THEN 1 ELSE 0 END) AS draw_games,
                    SUM(CASE WHEN m.going_first = 0 AND m.result = 'Win' THEN 1 ELSE 0 END) AS draw_wins,
                    AVG(CAST(m.turns AS REAL)) AS avg_turns,
                    SUM(CASE WHEN m.result = 'Win' AND LOWER(COALESCE(m.result_reason, '')) LIKE '%concede%' THEN 1 ELSE 0 END) AS opp_scoops
                FROM ingested_matches m
                LEFT JOIN ingested_card_names cn ON cn.grp_id = m.commander_grp_id
                WHERE m.commander_grp_id IS NOT NULL AND m.commander_grp_id > 0
                  AND LOWER(m.format) = LOWER(?)
                GROUP BY m.commander_grp_id, m.format
                HAVING COUNT(*) >= ?
                ORDER BY (CAST(SUM(CASE WHEN m.result = 'Win' THEN 1 ELSE 0 END) AS REAL) / COUNT(*)) DESC, matches DESC
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
                    m.commander_grp_id AS commander_grp_id,
                    cn.card_name AS commander_name,
                    m.format AS format,
                    COUNT(*) AS matches,
                    SUM(CASE WHEN m.result = 'Win' THEN 1 ELSE 0 END) AS wins,
                    SUM(CASE WHEN m.going_first = 1 THEN 1 ELSE 0 END) AS play_games,
                    SUM(CASE WHEN m.going_first = 1 AND m.result = 'Win' THEN 1 ELSE 0 END) AS play_wins,
                    SUM(CASE WHEN m.going_first = 0 THEN 1 ELSE 0 END) AS draw_games,
                    SUM(CASE WHEN m.going_first = 0 AND m.result = 'Win' THEN 1 ELSE 0 END) AS draw_wins,
                    AVG(CAST(m.turns AS REAL)) AS avg_turns,
                    SUM(CASE WHEN m.result = 'Win' AND LOWER(COALESCE(m.result_reason, '')) LIKE '%concede%' THEN 1 ELSE 0 END) AS opp_scoops
                FROM ingested_matches m
                LEFT JOIN ingested_card_names cn ON cn.grp_id = m.commander_grp_id
                WHERE m.commander_grp_id IS NOT NULL AND m.commander_grp_id > 0
                GROUP BY m.commander_grp_id, m.format
                HAVING COUNT(*) >= ?
                ORDER BY (CAST(SUM(CASE WHEN m.result = 'Win' THEN 1 ELSE 0 END) AS REAL) / COUNT(*)) DESC, matches DESC
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
                commander_name: r.try_get("commander_name").ok().flatten(),
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
                cn.card_name AS card_name,
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
            LEFT JOIN ingested_card_names cn ON cn.grp_id = c.grp_id
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
                card_name: r.try_get("card_name").ok().flatten(),
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
                m.commander_grp_id AS commander_grp_id,
                cn1.card_name AS commander_name,
                m.opponent_commander_grp_id AS opponent_commander_grp_id,
                cn2.card_name AS opponent_commander_name,
                COUNT(*) AS matches,
                SUM(CASE WHEN m.result = 'Win' THEN 1 ELSE 0 END) AS wins,
                AVG(CAST(m.turns AS REAL)) AS avg_turns
            FROM ingested_matches m
            LEFT JOIN ingested_card_names cn1 ON cn1.grp_id = m.commander_grp_id
            LEFT JOIN ingested_card_names cn2 ON cn2.grp_id = m.opponent_commander_grp_id
            WHERE m.commander_grp_id = ?
              AND m.opponent_commander_grp_id IS NOT NULL
              AND m.opponent_commander_grp_id > 0
              AND (? = '' OR LOWER(m.format) = LOWER(?))
            GROUP BY m.commander_grp_id, m.opponent_commander_grp_id
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
                commander_name: r.try_get("commander_name").ok().flatten(),
                opponent_commander_grp_id: r.get::<i64, _>("opponent_commander_grp_id") as u32,
                opponent_commander_name: r.try_get("opponent_commander_name").ok().flatten(),
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

    /// Returns Archetype Clusters + Core/Flex-Slot Win-Rate Deltas + Exact Deck Variants (`GET /api/v1/meta/decks`).
    pub async fn get_archetype_meta(
        &self,
        format_filter: Option<&str>,
        min_matches: i64,
    ) -> Result<Vec<ArchetypeMetaEntry>, sqlx::Error> {
        let min_m = min_matches.max(1);
        let fmt_opt = format_filter.filter(|s| !s.trim().is_empty()).unwrap_or("");

        // Load card name dictionary once
        let cname_rows = sqlx::query("SELECT grp_id, card_name FROM ingested_card_names")
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();
        let mut card_names: HashMap<u32, String> = HashMap::with_capacity(cname_rows.len());
        for r in cname_rows {
            let gid = r.get::<i64, _>("grp_id") as u32;
            let name: String = r.get("card_name");
            card_names.insert(gid, name);
        }

        // Group matches by COALESCE(archetype_hash, 'fmt_' || LOWER(format))
        let cluster_rows = sqlx::query(
            r#"
            SELECT
                COALESCE(NULLIF(archetype_hash, ''), 'fmt_' || LOWER(REPLACE(format, ' ', '_'))) AS arch_key,
                format,
                MAX(COALESCE(color_identity, 'C')) AS color_identity,
                MAX(COALESCE(commander_grp_id, 0)) AS commander_grp_id,
                COUNT(*) AS matches,
                SUM(CASE WHEN result = 'Win' THEN 1 ELSE 0 END) AS wins,
                SUM(CASE WHEN going_first = 1 THEN 1 ELSE 0 END) AS play_games,
                SUM(CASE WHEN going_first = 1 AND result = 'Win' THEN 1 ELSE 0 END) AS play_wins,
                SUM(CASE WHEN going_first = 0 THEN 1 ELSE 0 END) AS draw_games,
                SUM(CASE WHEN going_first = 0 AND result = 'Win' THEN 1 ELSE 0 END) AS draw_wins,
                AVG(CAST(turns AS REAL)) AS avg_turns
            FROM ingested_matches
            WHERE (? = '' OR LOWER(format) = LOWER(?))
              AND LOWER(format) NOT LIKE '%alchemy%'
            GROUP BY arch_key, format
            HAVING COUNT(*) >= ?
            "#,
        )
        .bind(fmt_opt)
        .bind(fmt_opt)
        .bind(min_m)
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::with_capacity(cluster_rows.len());

        for cr in cluster_rows {
            let arch_key: String = cr.get("arch_key");
            let format: String = cr.get("format");
            let color_identity: String = cr.try_get("color_identity").unwrap_or_else(|_| "C".to_string());
            let commander_grp_id = cr.try_get::<i64, _>("commander_grp_id").unwrap_or(0) as u32;
            let matches: i64 = cr.get("matches");
            let wins: i64 = cr.get("wins");
            let play_games: i64 = cr.try_get("play_games").unwrap_or(0);
            let play_wins: i64 = cr.try_get("play_wins").unwrap_or(0);
            let draw_games: i64 = cr.try_get("draw_games").unwrap_or(0);
            let draw_wins: i64 = cr.try_get("draw_wins").unwrap_or(0);

            let win_rate = if matches > 0 {
                round4(wins as f64 / matches as f64)
            } else {
                0.0
            };
            let wlb = round4(wilson_lower_bound(wins, matches));

            // Fetch all matches in this cluster
            let m_rows = sqlx::query(
                r#"
                SELECT match_hash, result, COALESCE(NULLIF(deck_hash, ''), 'dh_observed_' || SUBSTR(match_hash, 1, 8)) AS d_hash
                FROM ingested_matches
                WHERE COALESCE(NULLIF(archetype_hash, ''), 'fmt_' || LOWER(REPLACE(format, ' ', '_'))) = ?
                  AND LOWER(format) = LOWER(?)
                "#,
            )
            .bind(&arch_key)
            .bind(&format)
            .fetch_all(&self.pool)
            .await?;

            // Map match_hash -> (is_win, deck_hash, Vec<(grp_id, count, is_basic)>)
            let mut card_agg: HashMap<u32, (i64, i64, i64, bool)> = HashMap::new(); // grp_id -> (matches_with, wins_with, total_copies, is_basic)
            let mut variant_agg: HashMap<String, (i64, i64, Vec<SyncDeckCardEntry>)> = HashMap::new();

            for mr in &m_rows {
                let m_hash: String = mr.get("match_hash");
                let res_str: String = mr.get("result");
                let is_win = res_str == "Win";
                let d_hash: String = mr.get("d_hash");

                // Prefer ingested_deck_cards; fall back to ingested_match_cards (is_opponent=0)
                let dc_rows = sqlx::query(
                    "SELECT grp_id, count, is_basic_land FROM ingested_deck_cards WHERE match_hash = ? ORDER BY count DESC, grp_id ASC",
                )
                .bind(&m_hash)
                .fetch_all(&self.pool)
                .await?;

                let cards_for_match: Vec<(u32, u32, bool)> = if !dc_rows.is_empty() {
                    dc_rows
                        .into_iter()
                        .map(|r| {
                            (
                                r.get::<i64, _>("grp_id") as u32,
                                r.get::<i64, _>("count") as u32,
                                r.get::<i64, _>("is_basic_land") != 0,
                            )
                        })
                        .collect()
                } else {
                    let mc_rows = sqlx::query(
                        "SELECT grp_id, count FROM ingested_match_cards WHERE match_hash = ? AND is_opponent = 0 ORDER BY count DESC, grp_id ASC",
                    )
                    .bind(&m_hash)
                    .fetch_all(&self.pool)
                    .await?;
                    mc_rows
                        .into_iter()
                        .map(|r| {
                            let gid = r.get::<i64, _>("grp_id") as u32;
                            let cnt = r.get::<i64, _>("count") as u32;
                            let is_b = card_names
                                .get(&gid)
                                .map(|n| is_basic_land_name(n))
                                .unwrap_or(false);
                            (gid, cnt, is_b)
                        })
                        .collect()
                };

                for &(gid, cnt, is_b) in &cards_for_match {
                    let entry = card_agg.entry(gid).or_insert((0, 0, 0, is_b));
                    entry.0 += 1;
                    if is_win {
                        entry.1 += 1;
                    }
                    entry.2 += cnt as i64;
                    if is_b {
                        entry.3 = true;
                    }
                }

                let v_entry = variant_agg.entry(d_hash).or_insert_with(|| {
                    let v_cards = cards_for_match
                        .iter()
                        .map(|&(gid, cnt, is_b)| SyncDeckCardEntry {
                            grp_id: gid,
                            count: cnt,
                            card_name: card_names.get(&gid).cloned(),
                            is_basic_land: is_b,
                        })
                        .collect();
                    (0, 0, v_cards)
                });
                v_entry.0 += 1;
                if is_win {
                    v_entry.1 += 1;
                }
            }

            let mut core_cards = Vec::new();
            let mut flex_cards = Vec::new();

            for (gid, (m_with, w_with, tot_copies, is_basic)) in card_agg {
                let inc_rate = round4(m_with as f64 / matches.max(1) as f64);
                let avg_c = round2(tot_copies as f64 / m_with.max(1) as f64);
                let wr_inc = round4(w_with as f64 / m_with.max(1) as f64);
                let delta = round4(wr_inc - win_rate);

                let stat = ArchetypeCardStat {
                    grp_id: gid,
                    card_name: card_names.get(&gid).cloned(),
                    is_basic_land: is_basic,
                    inclusion_rate: inc_rate,
                    avg_copies: avg_c,
                    win_rate_when_included: wr_inc,
                    win_rate_delta: delta,
                };

                if is_basic || inc_rate >= 0.70 {
                    core_cards.push(stat);
                } else {
                    flex_cards.push(stat);
                }
            }

            core_cards.sort_by(|a, b| {
                a.is_basic_land
                    .cmp(&b.is_basic_land)
                    .then_with(|| b.inclusion_rate.partial_cmp(&a.inclusion_rate).unwrap_or(std::cmp::Ordering::Equal))
                    .then_with(|| b.avg_copies.partial_cmp(&a.avg_copies).unwrap_or(std::cmp::Ordering::Equal))
                    .then_with(|| a.grp_id.cmp(&b.grp_id))
            });

            flex_cards.sort_by(|a, b| {
                b.win_rate_delta
                    .partial_cmp(&a.win_rate_delta)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| b.inclusion_rate.partial_cmp(&a.inclusion_rate).unwrap_or(std::cmp::Ordering::Equal))
                    .then_with(|| a.grp_id.cmp(&b.grp_id))
            });

            let mut variants: Vec<DeckVariantEntry> = variant_agg
                .into_iter()
                .map(|(deck_hash, (v_m, v_w, cards))| DeckVariantEntry {
                    deck_hash,
                    matches: v_m,
                    wins: v_w,
                    win_rate: round4(v_w as f64 / v_m.max(1) as f64),
                    cards,
                })
                .collect();
            variants.sort_by(|a, b| {
                b.matches
                    .cmp(&a.matches)
                    .then_with(|| b.win_rate.partial_cmp(&a.win_rate).unwrap_or(std::cmp::Ordering::Equal))
            });

            // Build human-readable archetype name
            let guild = color_guild_label(&color_identity);
            let archetype_name = if commander_grp_id > 0 {
                if let Some(cname) = card_names.get(&commander_grp_id) {
                    format!("{} ({})", cname, guild)
                } else {
                    format!("Commander #{} ({})", commander_grp_id, guild)
                }
            } else if arch_key.starts_with("lim_") {
                format!("{} {}", guild, format)
            } else {
                let top_named: Vec<&str> = core_cards
                    .iter()
                    .chain(flex_cards.iter())
                    .filter(|c| !c.is_basic_land)
                    .filter_map(|c| c.card_name.as_deref())
                    .take(2)
                    .collect();
                if !top_named.is_empty() {
                    format!("{} — {}", guild, top_named.join(" / "))
                } else {
                    format!("{} {}", guild, format)
                }
            };

            out.push(ArchetypeMetaEntry {
                archetype_hash: arch_key,
                archetype_name,
                format,
                color_identity,
                matches,
                wins,
                win_rate,
                wilson_lower_bound: wlb,
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
                avg_turns: round2(cr.try_get("avg_turns").unwrap_or(0.0)),
                tier: classify_commander_tier(win_rate, matches).to_string(),
                core_cards,
                flex_cards,
                variants,
            });
        }

        out.sort_by(|a, b| {
            b.wilson_lower_bound
                .partial_cmp(&a.wilson_lower_bound)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.matches.cmp(&a.matches))
        });

        Ok(out)
    }

    /// Returns Global Verified Player Leaderboards & Combat Hall of Fame (`GET /api/v1/leaderboards`).
    pub async fn get_global_leaderboards(
        &self,
        format_filter: Option<&str>,
        limit: i64,
    ) -> Result<GlobalLeaderboardsResponse, sqlx::Error> {
        let max_rows = limit.clamp(1, 200);
        let fmt_opt = format_filter.filter(|s| !s.trim().is_empty()).unwrap_or("");

        let p_rows = sqlx::query(
            r#"
            SELECT
                player_account_hash,
                MAX(player_handle) AS player_handle,
                MAX(player_rank) AS latest_rank,
                COUNT(*) AS matches,
                SUM(CASE WHEN result = 'Win' THEN 1 ELSE 0 END) AS wins,
                MAX(format) AS sample_format
            FROM ingested_matches
            WHERE player_handle IS NOT NULL AND player_handle != ''
              AND player_account_hash IS NOT NULL AND player_account_hash != ''
              AND (? = '' OR LOWER(format) = LOWER(?))
              AND LOWER(format) NOT LIKE '%alchemy%'
            GROUP BY player_account_hash
            ORDER BY wins DESC, matches DESC
            LIMIT ?
            "#,
        )
        .bind(fmt_opt)
        .bind(fmt_opt)
        .bind(max_rows)
        .fetch_all(&self.pool)
        .await?;

        let mut players = Vec::with_capacity(p_rows.len());
        for r in p_rows {
            let matches: i64 = r.get("matches");
            let wins: i64 = r.get("wins");
            let wr = if matches > 0 {
                round4(wins as f64 / matches as f64)
            } else {
                0.0
            };
            let wlb = round4(wilson_lower_bound(wins, matches));
            players.push(LeaderboardPlayerEntry {
                player_handle: r.get("player_handle"),
                player_account_hash: r.get("player_account_hash"),
                latest_rank: r.try_get("latest_rank").ok().flatten(),
                matches,
                wins,
                win_rate: wr,
                wilson_lower_bound: wlb,
                favorite_format: r.try_get("sample_format").unwrap_or_else(|_| "Constructed".to_string()),
            });
        }
        players.sort_by(|a, b| {
            b.wilson_lower_bound
                .partial_cmp(&a.wilson_lower_bound)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.wins.cmp(&a.wins))
                .then_with(|| b.matches.cmp(&a.matches))
        });

        let c_rows = sqlx::query(
            r#"
            SELECT
                ic.grp_id AS grp_id,
                cn.card_name AS card_name,
                ic.max_hit AS max_single_hit,
                ic.total_damage AS total_damage,
                ic.cards_drawn AS cards_drawn,
                m.format AS format,
                m.player_handle AS player_handle,
                ic.titles_json AS titles_json
            FROM ingested_impactful_cards ic
            INNER JOIN ingested_matches m ON m.match_hash = ic.match_hash
            LEFT JOIN ingested_card_names cn ON cn.grp_id = ic.grp_id
            WHERE (? = '' OR LOWER(m.format) = LOWER(?))
              AND (ic.max_hit > 0 OR ic.total_damage > 0 OR ic.cards_drawn > 0)
            ORDER BY ic.max_hit DESC, ic.total_damage DESC, ic.cards_drawn DESC
            LIMIT ?
            "#,
        )
        .bind(fmt_opt)
        .bind(fmt_opt)
        .bind(max_rows)
        .fetch_all(&self.pool)
        .await?;

        let mut combat_records = Vec::with_capacity(c_rows.len());
        for cr in c_rows {
            let titles_raw: String = cr.try_get("titles_json").unwrap_or_default();
            let titles: Vec<String> = serde_json::from_str(&titles_raw).unwrap_or_default();
            combat_records.push(CombatRecordEntry {
                grp_id: cr.get::<i64, _>("grp_id") as u32,
                card_name: cr.try_get("card_name").ok().flatten(),
                max_single_hit: cr.get::<i64, _>("max_single_hit") as i32,
                total_damage: cr.get("total_damage"),
                cards_drawn: cr.get("cards_drawn"),
                format: cr.get("format"),
                player_handle: cr.try_get("player_handle").ok().flatten(),
                titles,
            });
        }

        Ok(GlobalLeaderboardsResponse {
            players,
            combat_records,
        })
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

pub fn wilson_lower_bound(wins: i64, matches: i64) -> f64 {
    if matches <= 0 {
        return 0.0;
    }
    let n = matches as f64;
    let p_hat = (wins.max(0) as f64) / n;
    let z = 1.96_f64;
    let z2 = z * z;
    let num = p_hat + z2 / (2.0 * n) - z * ((p_hat * (1.0 - p_hat) + z2 / (4.0 * n)) / n).sqrt();
    let den = 1.0 + z2 / n;
    (num / den).clamp(0.0, 1.0)
}

fn is_basic_land_name(name: &str) -> bool {
    let trimmed = name.trim();
    let base = trimmed.strip_prefix("Snow-Covered ").unwrap_or(trimmed);
    matches!(
        base,
        "Plains" | "Island" | "Swamp" | "Mountain" | "Forest" | "Wastes"
    )
}

pub fn color_guild_label(ci: &str) -> &'static str {
    match ci.trim() {
        "W" => "Mono-White (W)",
        "U" => "Mono-Blue (U)",
        "B" => "Mono-Black (B)",
        "R" => "Mono-Red (R)",
        "G" => "Mono-Green (G)",
        "WU" => "Azorius (WU)",
        "UB" => "Dimir (UB)",
        "BR" => "Rakdos (BR)",
        "RG" => "Gruul (RG)",
        "WG" | "GW" => "Selesnya (WG)",
        "WB" => "Orzhov (WB)",
        "UR" => "Izzet (UR)",
        "BG" => "Golgari (BG)",
        "WR" | "RW" => "Boros (WR)",
        "UG" | "GU" => "Simic (UG)",
        "WUG" => "Bant (WUG)",
        "WUB" => "Esper (WUB)",
        "UBR" => "Grixis (UBR)",
        "BRG" => "Jund (BRG)",
        "WRG" => "Naya (WRG)",
        "WBG" => "Abzan (WBG)",
        "WUR" => "Jeskai (WUR)",
        "UBG" => "Sultai (UBG)",
        "WBR" => "Mardu (WBR)",
        "URG" => "Temur (URG)",
        "WUBRG" => "Five-Color (WUBRG)",
        _ => "Colorless / Mixed",
    }
}

fn round2(val: f64) -> f64 {
    (val * 100.0).round() / 100.0
}

fn round4(val: f64) -> f64 {
    (val * 10000.0).round() / 10000.0
}

/// Computes the domain-separated SHA-256 hex digest of a secret `contributor_id` token
/// so the server can pin handle ownership in `claimed_handles` without storing the raw secret.
pub fn hash_contributor_token(contributor_id: &str) -> String {
    let payload = format!("rhystic-handle-claim-v1:{}", contributor_id.trim());
    sha256_hex(payload.as_bytes())
}

/// Pure-Rust FIPS 180-4 / RFC 6234 SHA-256 implementation returning a 64-char lowercase hex string.
pub fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    let bit_len = (data.len() as u64) * 8;
    let mut padded = Vec::with_capacity(((data.len() + 9 + 63) / 64) * 64);
    padded.extend_from_slice(data);
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0x00);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    format!(
        "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]
    )
}
