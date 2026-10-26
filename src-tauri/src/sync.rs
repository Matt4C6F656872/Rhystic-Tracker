//! Privacy-first, strictly opt-in Community Meta & Cloud Sync engine (Phase 3).
//!
//! Responsibilities:
//! 1. Manage user sync preferences (`SyncConfig`), defaulting to `enabled = false`
//!    (Tier 0: 100% offline).
//! 2. Generate a random local anonymous contributor UUID (Tier 1) that is never
//!    derived from the player's MTG Arena screen name or account ID.
//! 3. Transform completed legitimate matches into an [`AnonymizedMatchPayload`]
//!    that strips all screen names and personal identifiers while preserving
//!    format, Commander, turn count, card performance, and micro-combat metrics.
//! 4. Enqueue payloads into the local SQLite `sync_outbox` table with idempotent
//!    deduplication and support one-click historical backfill.

use crate::db::DatabaseManager;
use crate::deck_achievements;
use crate::match_assembler::{MatchCardRecord, MatchImpactfulRecord, MatchRecord};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::path::PathBuf;

pub const DEFAULT_SERVER_URL: &str = "https://api.rhystic.gg";
pub const PAYLOAD_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Strictly opt-in: false by default (Tier 0 offline mode).
    #[serde(default)]
    pub enabled: bool,
    /// Random local anonymous UUID (Tier 1). Never linked to Arena screen name.
    #[serde(default = "generate_anonymous_uuid")]
    pub contributor_id: String,
    /// Target ingestion server URL.
    #[serde(default = "default_server_url")]
    pub server_url: String,
    /// Whether to include the hero's seen card grp_ids for archetype/Commander clustering.
    #[serde(default = "default_true")]
    pub include_deck_cards: bool,
    /// Optional Tier 2 cross-device sync key for web/iPad companion linking.
    #[serde(default)]
    pub sync_key: Option<String>,
}

fn default_server_url() -> String {
    DEFAULT_SERVER_URL.to_string()
}

fn default_true() -> bool {
    true
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            contributor_id: generate_anonymous_uuid(),
            server_url: default_server_url(),
            include_deck_cards: true,
            sync_key: None,
        }
    }
}

/// Generate a random v4-formatted UUID using OS entropy (`/dev/urandom` on Unix)
/// with a timestamp/address entropy mixer fallback.
pub fn generate_anonymous_uuid() -> String {
    let mut bytes = [0u8; 16];
    let mut filled = false;

    #[cfg(unix)]
    {
        use std::io::Read;
        if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
            if f.read_exact(&mut bytes).is_ok() {
                filled = true;
            }
        }
    }

    if !filled {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0x9e3779b97f4a7c15);
        let addr = (&bytes as *const _ as usize) as u128;
        let mixed = nanos ^ addr.wrapping_mul(0xbf58476d1ce4e5b9);
        bytes = mixed.to_le_bytes();
    }

    // RFC 4122 version 4 & variant 1 bits
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

/// Deterministic 128-bit FNV-1a hash of `(contributor_id, match_id)` so the server
/// can deduplicate match submissions without learning the raw MTGA match UUID.
pub fn anonymize_match_id(contributor_id: &str, raw_match_id: &str) -> String {
    const OFFSET: u128 = 0x6c62272e07bb014262b821756295c58d;
    const PRIME: u128 = 0x0000000001000000000000000000013b;

    let mut hash = OFFSET;
    for b in contributor_id.as_bytes().iter().chain(b":").chain(raw_match_id.as_bytes()) {
        hash ^= *b as u128;
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{:032x}", hash)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCardEntry {
    pub grp_id: u32,
    pub is_opponent: bool,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncImpactfulEntry {
    pub grp_id: u32,
    pub total_damage: i32,
    pub max_hit: i32,
    pub damage_combat: i32,
    pub damage_spell: i32,
    pub cards_drawn: i64,
    pub counters_added: i64,
    pub titles: Vec<String>,
}

/// Anonymized match telemetry payload queued into `sync_outbox`.
/// Contains ZERO player names, opponent screen names, or raw account IDs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizedMatchPayload {
    pub schema_version: u32,
    pub contributor_id: String,
    pub match_hash: String,
    pub timestamp: String,
    pub format: String,
    pub result: String,
    pub result_reason: Option<String>,
    pub duration_seconds: u32,
    pub turns: u32,
    pub going_first: bool,
    pub hero_commander_id: Option<u32>,
    pub opponent_commander_id: Option<u32>,
    pub hero_life_end: Option<i32>,
    pub hero_min_life: Option<i32>,
    pub opponent_life_end: Option<i32>,
    pub hero_mulligans: u32,
    pub opponent_mulligans: u32,
    pub cards_seen: Vec<SyncCardEntry>,
    pub impactful_cards: Vec<SyncImpactfulEntry>,
}

/// Whether a match is eligible for community telemetry synchronization.
/// Supports all MTGA Constructed (60-card + Commander/Brawl) and Limited (Draft/Sealed)
/// formats, while explicitly excluding Alchemy, Bot Matches, Color Challenge, and Precon events.
pub fn is_sync_eligible(format: &str, deck_name: &str, preset_or_assigned: bool) -> bool {
    if preset_or_assigned {
        return false;
    }
    let f = format.trim().to_ascii_lowercase();
    if f.is_empty()
        || f.contains("alchemy")
        || f.contains("bot match")
        || f == "bot"
        || f.contains("color challenge")
        || f.contains("dualcolorprecon")
        || f.contains("jump in")
    {
        return false;
    }

    // Limited formats (Draft, Premier Draft, Quick Draft, Traditional Draft, Sealed, Limited)
    // are always eligible even when deck_name is "Draft Deck", "Selected Deck", or empty.
    if f.contains("draft") || f.contains("sealed") || f.contains("limited") {
        return true;
    }

    deck_achievements::is_eligible(format, deck_name, preset_or_assigned)
}

/// Build an [`AnonymizedMatchPayload`] if and only if the match is eligible
/// (not Alchemy, not a Bot Match, not an assigned/preset deck, and result is win or loss).
pub fn build_anonymized_payload(
    config: &SyncConfig,
    record: &MatchRecord,
    cards: &[MatchCardRecord],
    impactful: &[MatchImpactfulRecord],
    preset_or_assigned: bool,
) -> Option<AnonymizedMatchPayload> {
    if !is_sync_eligible(&record.format_name, &record.player_deck_name, preset_or_assigned) {
        return None;
    }
    let res_lower = record.result.to_lowercase();
    if res_lower != "win" && res_lower != "loss" {
        return None;
    }

    let cards_seen = if config.include_deck_cards {
        cards
            .iter()
            .filter(|c| c.grp_id > 0)
            .map(|c| SyncCardEntry {
                grp_id: c.grp_id,
                is_opponent: c.is_opponent,
                count: c.count,
            })
            .collect()
    } else {
        cards
            .iter()
            .filter(|c| c.grp_id > 0 && c.is_opponent)
            .map(|c| SyncCardEntry {
                grp_id: c.grp_id,
                is_opponent: true,
                count: c.count,
            })
            .collect()
    };

    let impactful_cards = impactful
        .iter()
        .filter(|i| i.seat_id == record.hero_seat_id && i.grp_id > 0)
        .map(|i| SyncImpactfulEntry {
            grp_id: i.grp_id,
            total_damage: i.total_damage,
            max_hit: i.max_hit,
            damage_combat: i.damage_combat,
            damage_spell: i.damage_spell,
            cards_drawn: i.cards_drawn,
            counters_added: i.counters_added,
            titles: i.titles.clone(),
        })
        .collect();

    Some(AnonymizedMatchPayload {
        schema_version: PAYLOAD_SCHEMA_VERSION,
        contributor_id: config.contributor_id.clone(),
        match_hash: anonymize_match_id(&config.contributor_id, &record.match_id),
        timestamp: record.timestamp.to_rfc3339(),
        format: record.format_name.clone(),
        result: res_lower,
        result_reason: record.result_reason.clone(),
        duration_seconds: record.duration_seconds,
        turns: record.turns,
        going_first: record.going_first,
        hero_commander_id: record.player_commander_id,
        opponent_commander_id: record.opponent_commander_id,
        hero_life_end: record.player_life_end,
        hero_min_life: record.min_player_life,
        opponent_life_end: record.opponent_life_end,
        hero_mulligans: record.player_mulligans.unwrap_or(0),
        opponent_mulligans: record.opponent_mulligans.unwrap_or(0),
        cards_seen,
        impactful_cards,
    })
}

fn sync_config_path() -> Option<PathBuf> {
    #[cfg(test)]
    {
        return None;
    }
    #[cfg(not(test))]
    {
        let mut dir = dirs::config_dir()?;
        dir.push("rhystic-tracker");
        let env_mode = DatabaseManager::resolve_env();
        if env_mode.eq_ignore_ascii_case("development") {
            dir.push("sync_settings_dev.json");
        } else {
            dir.push("sync_settings.json");
        }
        Some(dir)
    }
}

pub fn load_sync_config() -> SyncConfig {
    let Some(path) = sync_config_path() else {
        return SyncConfig::default();
    };
    if let Ok(raw) = std::fs::read_to_string(&path) {
        if let Ok(mut cfg) = serde_json::from_str::<SyncConfig>(&raw) {
            if cfg.contributor_id.trim().is_empty() {
                cfg.contributor_id = generate_anonymous_uuid();
            }
            if cfg.server_url.trim().is_empty() {
                cfg.server_url = default_server_url();
            }
            return cfg;
        }
    }
    let cfg = SyncConfig::default();
    let _ = save_sync_config(&cfg);
    cfg
}

pub fn save_sync_config(config: &SyncConfig) -> Result<(), String> {
    let Some(path) = sync_config_path() else {
        return Ok(());
    };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutboxStats {
    pub pending_count: i64,
    pub synced_count: i64,
    pub failed_count: i64,
    pub last_synced_at: Option<String>,
}

pub async fn get_outbox_stats(db: &DatabaseManager) -> OutboxStats {
    let pool = db.pool();
    let pending_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sync_outbox WHERE status = 'pending'")
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    let synced_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sync_outbox WHERE status = 'synced'")
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    let failed_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sync_outbox WHERE status = 'failed'")
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    let last_synced_at: Option<String> = sqlx::query_scalar(
        "SELECT MAX(synced_at) FROM sync_outbox WHERE status = 'synced'"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(None);

    OutboxStats {
        pending_count,
        synced_count,
        failed_count,
        last_synced_at,
    }
}

/// Enqueue a completed match into `sync_outbox` if `config.enabled` is true
/// and the match is eligible. Returns `Ok(true)` if a new row was queued.
pub async fn enqueue_match(
    db: &DatabaseManager,
    config: &SyncConfig,
    record: &MatchRecord,
    cards: &[MatchCardRecord],
    impactful: &[MatchImpactfulRecord],
    preset_or_assigned: bool,
) -> Result<bool, String> {
    if !config.enabled {
        return Ok(false);
    }
    let Some(payload) = build_anonymized_payload(config, record, cards, impactful, preset_or_assigned) else {
        return Ok(false);
    };
    let payload_json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();

    let res = sqlx::query(
        r#"
        INSERT INTO sync_outbox (match_id, payload_json, status, attempts, created_at)
        VALUES (?, ?, 'pending', 0, ?)
        ON CONFLICT(match_id) DO NOTHING
        "#,
    )
    .bind(&record.match_id)
    .bind(&payload_json)
    .bind(&now)
    .execute(db.pool())
    .await
    .map_err(|e| e.to_string())?;

    Ok(res.rows_affected() > 0)
}

/// Backfill historical eligible matches from local SQLite into `sync_outbox`
/// when a user opts in and clicks "Queue Match History for Sync".
pub async fn enqueue_historical_matches(
    db: &DatabaseManager,
    config: &SyncConfig,
) -> Result<usize, String> {
    let pool = db.pool();

    // Remove any previously queued non-synced rows from retired/excluded formats (Alchemy, DualColorPrecons, Bot Match)
    let _ = sqlx::query(
        r#"
        DELETE FROM sync_outbox
        WHERE status != 'synced'
          AND (
              LOWER(payload_json) LIKE '%"format":"alchemy%'
              OR LOWER(payload_json) LIKE '%"format":"dualcolorprecon%'
              OR LOWER(payload_json) LIKE '%"format":"bot match"%'
          )
        "#,
    )
    .execute(pool)
    .await;

    let rows = sqlx::query(
        r#"
        SELECT m.id, m.timestamp, m.date_str, m.format, m.result, m.duration_seconds,
               m.turns, m.going_first, m.hero_seat_id, m.hero_deck_name,
               m.hero_commander_id, m.hero_life_end, m.hero_min_life, m.hero_mulligans,
               m.opponent_commander_id, m.opponent_mulligans, m.opponent_life_end,
               m.result_reason, COALESCE(d.preset_deck, 0) as preset_deck
        FROM matches m
        LEFT JOIN match_decks d ON d.match_id = m.id
        WHERE m.format != 'Bot Match'
          AND LOWER(m.format) NOT LIKE '%alchemy%'
          AND LOWER(m.format) NOT LIKE '%dualcolorprecon%'
          AND COALESCE(d.preset_deck, 0) = 0
          AND m.id NOT IN (SELECT match_id FROM sync_outbox)
        ORDER BY m.timestamp ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut queued = 0usize;
    for r in rows {
        let match_id: String = r.get("id");
        let ts_raw: String = r.get("timestamp");
        let parsed_ts = chrono::DateTime::parse_from_rfc3339(&ts_raw)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let record = MatchRecord {
            match_id: match_id.clone(),
            timestamp: parsed_ts,
            date_str: r.get("date_str"),
            format_name: r.get("format"),
            result: r.get("result"),
            duration_seconds: r.get::<i64, _>("duration_seconds").max(0) as u32,
            turns: r.get::<i64, _>("turns").max(0) as u32,
            going_first: r.get("going_first"),
            hero_seat_id: r.try_get::<i64, _>("hero_seat_id").unwrap_or(1) as u32,
            player_deck_name: r.try_get::<Option<String>, _>("hero_deck_name").ok().flatten().unwrap_or_default(),
            player_commander_id: r.try_get::<Option<i64>, _>("hero_commander_id").ok().flatten().map(|v| v as u32),
            player_commander_name: None,
            player_life_end: r.try_get("hero_life_end").ok(),
            player_mulligans: r.try_get::<Option<i64>, _>("hero_mulligans").ok().flatten().map(|v| v as u32),
            hero_platform: None,
            hero_avatar: None,
            opponent_name: None,
            opponent_commander_id: r.try_get::<Option<i64>, _>("opponent_commander_id").ok().flatten().map(|v| v as u32),
            opponent_commander_name: None,
            opponent_mulligans: r.try_get::<Option<i64>, _>("opponent_mulligans").ok().flatten().map(|v| v as u32),
            opponent_life_end: r.try_get("opponent_life_end").ok(),
            opponent_platform: None,
            opponent_avatar: None,
            result_reason: r.try_get("result_reason").ok(),
            min_player_life: r.try_get("hero_min_life").ok(),
        };

        let preset: i64 = r.try_get("preset_deck").unwrap_or(0);

        let card_rows = sqlx::query(
            "SELECT grp_id, is_opponent, count FROM match_cards WHERE match_id = ?"
        )
        .bind(&match_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let cards: Vec<MatchCardRecord> = card_rows
            .into_iter()
            .map(|cr| MatchCardRecord {
                grp_id: cr.get::<i64, _>("grp_id") as u32,
                is_opponent: cr.get("is_opponent"),
                count: cr.get::<i64, _>("count") as u32,
            })
            .collect();

        let imp_rows = sqlx::query(
            r#"
            SELECT grp_id, seat_id, total_damage, max_hit, max_hit_combat, max_hit_spell,
                   damage_to_player, damage_to_permanents, damage_combat, damage_spell,
                   titles, COALESCE(cards_drawn, 0) as cards_drawn
            FROM match_impactful_cards
            WHERE match_id = ?
            "#,
        )
        .bind(&match_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let impactful: Vec<MatchImpactfulRecord> = imp_rows
            .into_iter()
            .map(|ir| {
                let titles_raw: Option<String> = ir.try_get("titles").ok();
                let titles: Vec<String> = titles_raw
                    .as_deref()
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or_default();
                MatchImpactfulRecord {
                    grp_id: ir.get::<i64, _>("grp_id") as u32,
                    seat_id: ir.get::<i64, _>("seat_id") as u32,
                    total_damage: ir.get::<i64, _>("total_damage") as i32,
                    max_hit: ir.get::<i64, _>("max_hit") as i32,
                    max_hit_combat: ir.get::<i64, _>("max_hit_combat") as i32,
                    max_hit_spell: ir.get::<i64, _>("max_hit_spell") as i32,
                    damage_to_player: ir.get::<i64, _>("damage_to_player") as i32,
                    damage_to_permanents: ir.get::<i64, _>("damage_to_permanents") as i32,
                    damage_combat: ir.get::<i64, _>("damage_combat") as i32,
                    damage_spell: ir.get::<i64, _>("damage_spell") as i32,
                    titles,
                    cards_drawn: ir.get::<i64, _>("cards_drawn"),
                    counters_added: 0,
                }
            })
            .collect();

        let mut forced_cfg = config.clone();
        forced_cfg.enabled = true;
        if enqueue_match(db, &forced_cfg, &record, &cards, &impactful, preset != 0).await? {
            queued += 1;
        }
    }

    Ok(queued)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFlushReport {
    pub flushed_count: usize,
    pub remaining_pending: i64,
    pub last_error: Option<String>,
}

#[derive(Debug, Serialize)]
struct BatchIngestWire<'a> {
    matches: &'a [AnonymizedMatchPayload],
}

pub fn resolve_batch_ingest_url(raw_url: &str) -> String {
    let trimmed = raw_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return "http://127.0.0.1:8787/api/v1/matches/batch".to_string();
    }
    if trimmed.ends_with("/api/v1/matches/batch") || trimmed.ends_with("/v1/ingest") {
        trimmed.to_string()
    } else if let Some(prefix) = trimmed.strip_suffix("/api/v1/matches") {
        format!("{}/api/v1/matches/batch", prefix)
    } else {
        format!("{}/api/v1/matches/batch", trimmed)
    }
}

/// Flushes up to `batch_size` pending rows from `sync_outbox` to the configured
/// Rhystic Cloud ingestion endpoint (`POST /api/v1/matches/batch`).
/// On network/HTTP failure, increments `attempts` and records `last_error` for retry backoff.
pub async fn flush_outbox_batch(
    db: &DatabaseManager,
    config: &SyncConfig,
    batch_size: i64,
) -> Result<SyncFlushReport, String> {
    if !config.enabled {
        let stats = get_outbox_stats(db).await;
        return Ok(SyncFlushReport {
            flushed_count: 0,
            remaining_pending: stats.pending_count,
            last_error: Some("Sync is currently disabled (Tier 0 Offline Mode).".to_string()),
        });
    }

    let pool = db.pool();
    let limit = batch_size.clamp(1, 500);
    let rows = sqlx::query(
        r#"
        SELECT id, match_id, payload_json
        FROM sync_outbox
        WHERE status IN ('pending', 'failed') AND attempts < 10
        ORDER BY id ASC
        LIMIT ?
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    if rows.is_empty() {
        let stats = get_outbox_stats(db).await;
        return Ok(SyncFlushReport {
            flushed_count: 0,
            remaining_pending: stats.pending_count,
            last_error: None,
        });
    }

    let mut row_ids: Vec<i64> = Vec::with_capacity(rows.len());
    let mut payloads: Vec<AnonymizedMatchPayload> = Vec::with_capacity(rows.len());

    for r in rows {
        let id: i64 = r.get("id");
        let raw_json: String = r.get("payload_json");
        match serde_json::from_str::<AnonymizedMatchPayload>(&raw_json) {
            Ok(p) => {
                row_ids.push(id);
                payloads.push(p);
            }
            Err(e) => {
                let _ = sqlx::query(
                    "UPDATE sync_outbox SET status = 'failed', attempts = 10, last_error = ? WHERE id = ?",
                )
                .bind(format!("Malformed payload JSON: {}", e))
                .bind(id)
                .execute(pool)
                .await;
            }
        }
    }

    if payloads.is_empty() {
        let stats = get_outbox_stats(db).await;
        return Ok(SyncFlushReport {
            flushed_count: 0,
            remaining_pending: stats.pending_count,
            last_error: None,
        });
    }

    let target_url = resolve_batch_ingest_url(&config.server_url);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let send_res = client
        .post(&target_url)
        .json(&BatchIngestWire { matches: &payloads })
        .send()
        .await;

    match send_res {
        Ok(resp) if resp.status().is_success() => {
            let now = Utc::now().to_rfc3339();
            for id in &row_ids {
                let _ = sqlx::query(
                    r#"
                    UPDATE sync_outbox
                    SET status = 'synced', synced_at = ?, last_error = NULL
                    WHERE id = ?
                    "#,
                )
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await;
            }
            let stats = get_outbox_stats(db).await;
            Ok(SyncFlushReport {
                flushed_count: row_ids.len(),
                remaining_pending: stats.pending_count,
                last_error: None,
            })
        }
        Ok(resp) => {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            let err_msg = format!("HTTP {}: {}", status, body_text);
            for id in &row_ids {
                let _ = sqlx::query(
                    r#"
                    UPDATE sync_outbox
                    SET attempts = attempts + 1,
                        status = CASE WHEN attempts + 1 >= 5 THEN 'failed' ELSE 'pending' END,
                        last_error = ?
                    WHERE id = ?
                    "#,
                )
                .bind(&err_msg)
                .bind(id)
                .execute(pool)
                .await;
            }
            let stats = get_outbox_stats(db).await;
            Ok(SyncFlushReport {
                flushed_count: 0,
                remaining_pending: stats.pending_count,
                last_error: Some(err_msg),
            })
        }
        Err(net_err) => {
            let err_msg = format!("Connection error ({}): {}", target_url, net_err);
            for id in &row_ids {
                let _ = sqlx::query(
                    r#"
                    UPDATE sync_outbox
                    SET attempts = attempts + 1,
                        status = CASE WHEN attempts + 1 >= 5 THEN 'failed' ELSE 'pending' END,
                        last_error = ?
                    WHERE id = ?
                    "#,
                )
                .bind(&err_msg)
                .bind(id)
                .execute(pool)
                .await;
            }
            let stats = get_outbox_stats(db).await;
            Ok(SyncFlushReport {
                flushed_count: 0,
                remaining_pending: stats.pending_count,
                last_error: Some(err_msg),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_record() -> MatchRecord {
        MatchRecord {
            match_id: "raw-arena-match-uuid-999".to_string(),
            timestamp: Utc::now(),
            date_str: "2026-10-26".to_string(),
            format_name: "Commander".to_string(),
            result: "win".to_string(),
            duration_seconds: 540,
            turns: 11,
            going_first: true,
            hero_seat_id: 1,
            player_deck_name: "Secret Personal Deck Name".to_string(),
            player_commander_id: Some(77777),
            player_commander_name: Some("Atraxa, Praetors' Voice".to_string()),
            player_life_end: Some(36),
            player_mulligans: Some(1),
            hero_platform: Some("Steam".to_string()),
            hero_avatar: Some("Avatar_Urza".to_string()),
            opponent_name: Some("PrivateOpponent#12345".to_string()),
            opponent_commander_id: Some(88888),
            opponent_commander_name: Some("Etali, Primal Conqueror".to_string()),
            opponent_mulligans: Some(0),
            opponent_life_end: Some(0),
            opponent_platform: Some("Windows".to_string()),
            opponent_avatar: None,
            result_reason: Some("ResultReason_Game".to_string()),
            min_player_life: Some(28),
        }
    }

    #[test]
    fn uuid_v4_format_is_valid_and_unique() {
        let u1 = generate_anonymous_uuid();
        let u2 = generate_anonymous_uuid();
        assert_eq!(u1.len(), 36);
        assert_eq!(&u1[14..15], "4", "UUID version nibble must be 4");
        assert_ne!(u1, u2, "Generated UUIDs should be distinct");
    }

    #[test]
    fn payload_strips_all_personal_names_and_hashes_match_id() {
        let mut cfg = SyncConfig::default();
        cfg.enabled = true;
        let rec = sample_record();
        let cards = vec![
            MatchCardRecord { grp_id: 1001, is_opponent: false, count: 1 },
            MatchCardRecord { grp_id: 2002, is_opponent: true, count: 1 },
        ];
        let impactful = vec![MatchImpactfulRecord {
            grp_id: 1001,
            seat_id: 1,
            total_damage: 18,
            max_hit: 9,
            max_hit_combat: 9,
            max_hit_spell: 0,
            damage_to_player: 18,
            damage_to_permanents: 0,
            damage_combat: 18,
            damage_spell: 0,
            titles: vec!["Haymaker (Bronze)".to_string()],
            cards_drawn: 2,
            counters_added: 0,
        }];

        let payload = build_anonymized_payload(&cfg, &rec, &cards, &impactful, false)
            .expect("Commander match should produce payload");
        let json = serde_json::to_string(&payload).unwrap();

        assert!(!json.contains("PrivateOpponent#12345"), "Opponent screen name must never be in payload");
        assert!(!json.contains("Secret Personal Deck Name"), "Personal deck name must never be in payload");
        assert!(!json.contains("raw-arena-match-uuid-999"), "Raw match_id must be hashed");
        assert_eq!(payload.format, "Commander");
        assert_eq!(payload.hero_commander_id, Some(77777));
        assert_eq!(payload.opponent_commander_id, Some(88888));
        assert_eq!(payload.cards_seen.len(), 2);
        assert_eq!(payload.impactful_cards.len(), 1);
    }

    #[test]
    fn bot_matches_and_preset_decks_are_excluded_from_sync() {
        let mut cfg = SyncConfig::default();
        cfg.enabled = true;
        let mut rec = sample_record();
        rec.format_name = "Bot Match".to_string();
        assert!(build_anonymized_payload(&cfg, &rec, &[], &[], false).is_none());

        let rec2 = sample_record();
        assert!(build_anonymized_payload(&cfg, &rec2, &[], &[], true).is_none());

        // Alchemy and DualColorPrecons are excluded
        let mut rec_alch = sample_record();
        rec_alch.format_name = "Alchemy Ranked".to_string();
        assert!(build_anonymized_payload(&cfg, &rec_alch, &[], &[], false).is_none());

        let mut rec_precon = sample_record();
        rec_precon.format_name = "DualColorPrecons".to_string();
        assert!(build_anonymized_payload(&cfg, &rec_precon, &[], &[], false).is_none());

        // Limited formats (Draft, Sealed) are accepted even with generic/empty deck names
        let mut rec_draft = sample_record();
        rec_draft.format_name = "Draft".to_string();
        rec_draft.player_deck_name = "Draft Deck".to_string();
        assert!(build_anonymized_payload(&cfg, &rec_draft, &[], &[], false).is_some());

        let mut rec_sealed = sample_record();
        rec_sealed.format_name = "Sealed".to_string();
        rec_sealed.player_deck_name = "".to_string();
        assert!(build_anonymized_payload(&cfg, &rec_sealed, &[], &[], false).is_some());
    }

    #[tokio::test]
    async fn outbox_enqueue_and_historical_backfill_are_idempotent() {
        let db = DatabaseManager::init().await.expect("test db");
        let mut cfg = SyncConfig::default();
        cfg.enabled = true;

        let rec = sample_record();
        let cards = vec![MatchCardRecord { grp_id: 1001, is_opponent: false, count: 1 }];
        db.upsert_match(&rec, &cards, &[], &[]).await.expect("upsert match");

        // Enqueue once -> true; enqueue same match again -> false (idempotent)
        let first = enqueue_match(&db, &cfg, &rec, &cards, &[], false).await.unwrap();
        let second = enqueue_match(&db, &cfg, &rec, &cards, &[], false).await.unwrap();
        assert!(first);
        assert!(!second);

        let stats = get_outbox_stats(&db).await;
        assert_eq!(stats.pending_count, 1);

        // Historical backfill sees the match is already in sync_outbox -> 0 newly added
        let backfilled = enqueue_historical_matches(&db, &cfg).await.unwrap();
        assert_eq!(backfilled, 0);
    }
}
