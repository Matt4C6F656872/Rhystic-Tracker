//! Data models for the Rhystic Cloud ingestion API and analytical endpoints.
//!
//! Designed to match the client's `AnonymizedMatchPayload` schema (v1) in
//! `src-tauri/src/sync.rs` with zero personal identifiers by default, and
//! optional log-verified Arena handle & full decklist clustering when opted in.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCardEntry {
    pub grp_id: u32,
    pub is_opponent: bool,
    pub count: u32,
    #[serde(default)]
    pub card_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDeckCardEntry {
    pub grp_id: u32,
    pub count: u32,
    #[serde(default)]
    pub card_name: Option<String>,
    #[serde(default)]
    pub is_basic_land: bool,
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
    #[serde(default)]
    pub card_name: Option<String>,
}

/// Match telemetry payload sent by opted-in clients.
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
    pub hero_commander_name: Option<String>,
    #[serde(default)]
    pub companion_grp_id: Option<u32>,
    #[serde(default, alias = "opponent_commander_grp_id")]
    pub opponent_commander_id: Option<u32>,
    #[serde(default)]
    pub opponent_commander_name: Option<String>,
    #[serde(default)]
    pub opponent_companion_grp_id: Option<u32>,
    #[serde(default)]
    pub player_handle: Option<String>,
    #[serde(default)]
    pub player_account_hash: Option<String>,
    #[serde(default)]
    pub player_rank: Option<String>,
    #[serde(default)]
    pub deck_hash: Option<String>,
    #[serde(default)]
    pub archetype_hash: Option<String>,
    #[serde(default)]
    pub color_identity: Option<String>,
    #[serde(default)]
    pub deck_cards: Vec<SyncDeckCardEntry>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commander_name: Option<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card_name: Option<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commander_name: Option<String>,
    pub opponent_commander_grp_id: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opponent_commander_name: Option<String>,
    pub matches: i64,
    pub wins: i64,
    pub win_rate: f64,
    pub avg_turns: f64,
}

/// Card inclusion and win-rate delta inside an archetype cluster (`GET /api/v1/meta/decks`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchetypeCardStat {
    pub grp_id: u32,
    pub card_name: Option<String>,
    pub is_basic_land: bool,
    /// Fraction of matches in this archetype that included this card (0.0 - 1.0).
    pub inclusion_rate: f64,
    pub avg_copies: f64,
    /// Win rate when this card is in the decklist (0.0 - 1.0).
    pub win_rate_when_included: f64,
    /// Win rate difference vs. the archetype's baseline win rate (e.g., +0.032 = +3.2%).
    pub win_rate_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckVariantEntry {
    pub deck_hash: String,
    pub matches: i64,
    pub wins: i64,
    pub win_rate: f64,
    pub cards: Vec<SyncDeckCardEntry>,
}

/// Archetype cluster & Core/Flex-Slot breakdown returned by `GET /api/v1/meta/decks`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchetypeMetaEntry {
    pub archetype_hash: String,
    pub archetype_name: String,
    pub format: String,
    pub color_identity: String,
    pub matches: i64,
    pub wins: i64,
    pub win_rate: f64,
    /// 95% Wilson lower confidence bound so high-sample decks outrank 1-0 flukes.
    pub wilson_lower_bound: f64,
    pub play_win_rate: f64,
    pub draw_win_rate: f64,
    pub avg_turns: f64,
    pub tier: String,
    /// Cards appearing in >= 70% of lists in this archetype (plus basic lands).
    pub core_cards: Vec<ArchetypeCardStat>,
    /// Non-basic cards appearing in < 70% of lists in this archetype, with win_rate_delta.
    pub flex_cards: Vec<ArchetypeCardStat>,
    /// Exact 60/100-card variant lists (`deck_hash`) within this archetype.
    pub variants: Vec<DeckVariantEntry>,
}

/// Log-verified player leaderboard entry returned by `GET /api/v1/leaderboards`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardPlayerEntry {
    pub player_handle: String,
    pub player_account_hash: String,
    pub latest_rank: Option<String>,
    pub matches: i64,
    pub wins: i64,
    pub win_rate: f64,
    pub wilson_lower_bound: f64,
    pub favorite_format: String,
}

/// Global combat Hall of Fame record returned by `GET /api/v1/leaderboards`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatRecordEntry {
    pub grp_id: u32,
    pub card_name: Option<String>,
    pub max_single_hit: i32,
    pub total_damage: i64,
    pub cards_drawn: i64,
    pub format: String,
    pub player_handle: Option<String>,
    pub titles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalLeaderboardsResponse {
    pub players: Vec<LeaderboardPlayerEntry>,
    pub combat_records: Vec<CombatRecordEntry>,
}
