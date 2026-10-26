//! Data models for the Rhystic Cloud ingestion API and analytical endpoints.
//!
//! Designed to match the client's `AnonymizedMatchPayload` schema (v1) in
//! `src-tauri/src/sync.rs` with zero personal identifiers.

use serde::{Deserialize, Serialize};

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
    #[serde(default)]
    pub titles: Vec<String>,
}

/// Anonymized match telemetry payload sent by opted-in clients.
/// Supports both `hero_commander_id` (client wire format) and `commander_grp_id` aliases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizedMatchPayload {
    pub schema_version: u32,
    pub contributor_id: String,
    pub match_hash: String,
    pub timestamp: String,
    pub format: String,
    pub result: String,
    #[serde(default)]
    pub result_reason: Option<String>,
    pub duration_seconds: u32,
    pub turns: u32,
    pub going_first: bool,
    #[serde(default)]
    pub hero_starting_life: Option<i32>,
    #[serde(default, alias = "hero_end_life")]
    pub hero_life_end: Option<i32>,
    #[serde(default)]
    pub hero_min_life: Option<i32>,
    #[serde(default)]
    pub opponent_life_end: Option<i32>,
    #[serde(default)]
    pub hero_mulligans: u32,
    #[serde(default)]
    pub opponent_mulligans: u32,
    #[serde(default, alias = "commander_grp_id")]
    pub hero_commander_id: Option<u32>,
    #[serde(default)]
    pub companion_grp_id: Option<u32>,
    #[serde(default, alias = "opponent_commander_grp_id")]
    pub opponent_commander_id: Option<u32>,
    #[serde(default)]
    pub opponent_companion_grp_id: Option<u32>,
    #[serde(default)]
    pub cards_seen: Vec<SyncCardEntry>,
    #[serde(default)]
    pub impactful_cards: Vec<SyncImpactfulEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchIngestRequest {
    pub matches: Vec<AnonymizedMatchPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IngestResponse {
    pub accepted: usize,
    pub duplicates: usize,
    pub rejected: usize,
    pub accepted_hashes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub total_matches: i64,
    pub unique_contributors: i64,
    pub constructed_matches: i64,
    pub limited_matches: i64,
    pub commander_matches: i64,
}

/// Format-level summary metrics returned by `GET /api/v1/meta/formats`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatMetaSummary {
    pub format: String,
    /// Category: `"Commander & Brawl"`, `"Constructed"`, or `"Limited"`.
    pub category: String,
    pub total_matches: i64,
    pub wins: i64,
    pub win_rate: f64,
    pub avg_turns: f64,
    pub avg_duration_seconds: f64,
    pub play_win_rate: f64,
    pub draw_win_rate: f64,
    /// Percentage of wins in this format caused by opponent concession ("Salt Index").
    pub concede_rate: f64,
    pub unique_commanders: i64,
}

/// Commander tier-list row returned by `GET /api/v1/meta/commanders`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommanderTierEntry {
    pub commander_grp_id: u32,
    pub format: String,
    pub matches: i64,
    pub wins: i64,
    pub win_rate: f64,
    pub play_win_rate: f64,
    pub draw_win_rate: f64,
    pub avg_turns: f64,
    /// Percentage of matches against this commander that end in an opponent concession.
    pub salt_index: f64,
    /// Calculated tier ("S", "A", "B", "C", "D") based on win rate and sample size.
    pub tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleFrequency {
    pub title: String,
    pub count: u64,
}

/// Card performance & micro-combat telemetry row returned by `GET /api/v1/meta/cards`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardPerformanceEntry {
    pub grp_id: u32,
    pub matches_seen: i64,
    pub wins_when_seen: i64,
    /// Win rate in games where the hero cast or revealed this card (0.0 - 1.0).
    pub seen_win_rate: f64,
    pub total_damage: i64,
    pub max_single_hit: i32,
    pub combat_damage: i64,
    pub spell_damage: i64,
    pub total_cards_drawn: i64,
    pub top_titles: Vec<TitleFrequency>,
}

/// Head-to-head Commander matchup row returned by `GET /api/v1/meta/matchups`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommanderMatchupEntry {
    pub commander_grp_id: u32,
    pub opponent_commander_grp_id: u32,
    pub matches: i64,
    pub wins: i64,
    pub win_rate: f64,
    pub avg_turns: f64,
}
