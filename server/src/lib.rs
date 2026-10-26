//! Rhystic Cloud — Opt-in Anonymized Match Ingestion & Meta Analytics API.
//!
//! Endpoints:
//! - `GET  /health` -> Service status & aggregate match/contributor counts
//! - `POST /api/v1/matches` -> Single anonymized match payload ingestion
//! - `POST /api/v1/matches/batch` -> Batch anonymized match ingestion (`{ matches: [...] }`)
//! - `GET  /api/v1/meta/formats` -> Format-by-format meta breakdown (turns, play/draw WR, Salt Index)
//! - `GET  /api/v1/meta/commanders` -> Commander & Brawl tier list (`?format=Commander&min_matches=1`)
//! - `GET  /api/v1/meta/cards` -> Card seen win rates & micro-combat stats (`?format=...&commander_grp_id=...`)
//! - `GET  /api/v1/meta/matchups` -> Head-to-head Commander matchup matrix (`?commander_grp_id=...&format=...`)

pub mod db;
pub mod models;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use db::AnalyticalDb;
use models::{
    AnonymizedMatchPayload, BatchIngestRequest, CardPerformanceEntry, CommanderMatchupEntry,
    CommanderTierEntry, FormatMetaSummary, HealthResponse, IngestResponse,
};
use serde::Deserialize;
use tower_http::cors::CorsLayer;

pub fn build_router(db: AnalyticalDb) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/v1/ingest", post(ingest_batch_handler))
        .route("/api/v1/matches", post(ingest_single_handler))
        .route("/api/v1/matches/batch", post(ingest_batch_handler))
        .route("/api/v1/meta/formats", get(meta_formats_handler))
        .route("/api/v1/meta/commanders", get(meta_commanders_handler))
        .route("/api/v1/meta/cards", get(meta_cards_handler))
        .route("/api/v1/meta/matchups", get(meta_matchups_handler))
        .layer(CorsLayer::permissive())
        .with_state(db)
}

async fn health_handler(
    State(db): State<AnalyticalDb>,
) -> Result<Json<HealthResponse>, (StatusCode, String)> {
    let (
        total_matches,
        unique_contributors,
        constructed_matches,
        limited_matches,
        commander_matches,
    ) = db
        .get_health_stats()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        service: "rhystic-cloud-server".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        total_matches,
        unique_contributors,
        constructed_matches,
        limited_matches,
        commander_matches,
    }))
}

#[derive(Debug, Deserialize, Default)]
pub struct FormatsQuery {
    pub category: Option<String>,
    pub active_only: Option<bool>,
}

async fn ingest_single_handler(
    State(db): State<AnalyticalDb>,
    Json(payload): Json<AnonymizedMatchPayload>,
) -> Result<Json<IngestResponse>, (StatusCode, String)> {
    let res = db
        .ingest_batch(&[payload])
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(res))
}

async fn ingest_batch_handler(
    State(db): State<AnalyticalDb>,
    Json(req): Json<BatchIngestRequest>,
) -> Result<Json<IngestResponse>, (StatusCode, String)> {
    if req.matches.len() > 1000 {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            "Batch size exceeds maximum of 1000 matches per request".to_string(),
        ));
    }
    let res = db
        .ingest_batch(&req.matches)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(res))
}

async fn meta_formats_handler(
    State(db): State<AnalyticalDb>,
    Query(q): Query<FormatsQuery>,
) -> Result<Json<Vec<FormatMetaSummary>>, (StatusCode, String)> {
    let summary = db
        .get_format_meta(q.category.as_deref(), q.active_only.unwrap_or(false))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(summary))
}

#[derive(Debug, Deserialize)]
pub struct CommandersQuery {
    pub format: Option<String>,
    pub min_matches: Option<i64>,
}

async fn meta_commanders_handler(
    State(db): State<AnalyticalDb>,
    Query(q): Query<CommandersQuery>,
) -> Result<Json<Vec<CommanderTierEntry>>, (StatusCode, String)> {
    let list = db
        .get_commander_tier_list(q.format.as_deref(), q.min_matches.unwrap_or(1))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(list))
}

#[derive(Debug, Deserialize)]
pub struct CardsQuery {
    pub format: Option<String>,
    pub commander_grp_id: Option<u32>,
    pub limit: Option<i64>,
}

async fn meta_cards_handler(
    State(db): State<AnalyticalDb>,
    Query(q): Query<CardsQuery>,
) -> Result<Json<Vec<CardPerformanceEntry>>, (StatusCode, String)> {
    let list = db
        .get_card_performance(
            q.format.as_deref(),
            q.commander_grp_id,
            q.limit.unwrap_or(100),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(list))
}

#[derive(Debug, Deserialize)]
pub struct MatchupsQuery {
    pub commander_grp_id: u32,
    pub format: Option<String>,
}

async fn meta_matchups_handler(
    State(db): State<AnalyticalDb>,
    Query(q): Query<MatchupsQuery>,
) -> Result<Json<Vec<CommanderMatchupEntry>>, (StatusCode, String)> {
    let list = db
        .get_commander_matchups(q.commander_grp_id, q.format.as_deref())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(list))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SyncCardEntry, SyncImpactfulEntry};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn sample_commander_match(
        hash: &str,
        contributor: &str,
        result: &str,
        reason: Option<&str>,
        going_first: bool,
        turns: u32,
        commander: u32,
        opp_commander: u32,
    ) -> AnonymizedMatchPayload {
        AnonymizedMatchPayload {
            schema_version: 1,
            contributor_id: contributor.to_string(),
            match_hash: hash.to_string(),
            timestamp: "2026-10-26T18:00:00Z".to_string(),
            format: "Commander".to_string(),
            result: result.to_string(),
            result_reason: reason.map(|s| s.to_string()),
            duration_seconds: 420,
            turns,
            going_first,
            hero_starting_life: Some(40),
            hero_life_end: Some(if result == "Win" { 34 } else { 0 }),
            hero_min_life: Some(if result == "Win" { 28 } else { 0 }),
            opponent_life_end: Some(if result == "Win" { 0 } else { 22 }),
            hero_mulligans: 0,
            opponent_mulligans: 0,
            hero_commander_id: Some(commander),
            companion_grp_id: None,
            opponent_commander_id: Some(opp_commander),
            opponent_companion_grp_id: None,
            cards_seen: vec![
                SyncCardEntry {
                    grp_id: 90001, // Rhystic Study
                    is_opponent: false,
                    count: 1,
                },
                SyncCardEntry {
                    grp_id: 90002, // Craterhoof Behemoth
                    is_opponent: false,
                    count: 1,
                },
            ],
            impactful_cards: vec![
                SyncImpactfulEntry {
                    grp_id: 90001,
                    total_damage: 0,
                    max_hit: 0,
                    damage_combat: 0,
                    damage_spell: 0,
                    cards_drawn: 7,
                    counters_added: 0,
                    titles: vec!["Rhystic Tracker (Gold)".to_string()],
                },
                SyncImpactfulEntry {
                    grp_id: 90002,
                    total_damage: 42,
                    max_hit: 24,
                    damage_combat: 42,
                    damage_spell: 0,
                    cards_drawn: 0,
                    counters_added: 0,
                    titles: vec![
                        "Over Killer (Gold)".to_string(),
                        "Scoop Inducer (Silver)".to_string(),
                    ],
                },
            ],
        }
    }

    #[tokio::test]
    async fn batch_ingest_is_idempotent_and_rejects_invalid_payloads() {
        let db = AnalyticalDb::connect(":memory:").await.unwrap();
        let app = build_router(db);

        let valid1 = sample_commander_match(
            "rh_1111111111111111",
            "contrib-a",
            "Win",
            Some("ResultReason_Concede"),
            true,
            9,
            77701,
            88801,
        );
        let duplicate1 = valid1.clone();
        let mut invalid = valid1.clone();
        invalid.match_hash = "rh_bad".to_string();
        invalid.turns = 0; // invalid: 0 turns

        let req_body = serde_json::to_string(&BatchIngestRequest {
            matches: vec![valid1, duplicate1, invalid],
        })
        .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/matches/batch")
                    .header("content-type", "application/json")
                    .body(Body::from(req_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let report: IngestResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(report.accepted, 1);
        assert_eq!(report.duplicates, 1);
        assert_eq!(report.rejected, 1);
        assert_eq!(report.accepted_hashes.len(), 2);
    }

    #[tokio::test]
    async fn commander_tier_list_cards_and_matchups_aggregate_accurately() {
        let db = AnalyticalDb::connect(":memory:").await.unwrap();
        let app = build_router(db.clone());

        // Commander 77701 (Atraxa) plays 4 games vs 88801 (Ragavan): wins 3 (2 by scoop), loses 1
        let m1 = sample_commander_match(
            "rh_01",
            "u1",
            "Win",
            Some("ResultReason_Concede"),
            true,
            8,
            77701,
            88801,
        );
        let m2 = sample_commander_match(
            "rh_02",
            "u2",
            "Win",
            Some("ResultReason_Concede"),
            false,
            10,
            77701,
            88801,
        );
        let m3 = sample_commander_match("rh_03", "u1", "Win", Some("Normal"), true, 12, 77701, 88801);
        let m4 = sample_commander_match("rh_04", "u3", "Loss", Some("Normal"), false, 14, 77701, 88801);

        db.ingest_batch(&[m1, m2, m3, m4]).await.unwrap();

        // 1. Check /health
        let resp = app
            .clone()
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let health: HealthResponse =
            serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
        assert_eq!(health.total_matches, 4);
        assert_eq!(health.unique_contributors, 3);
        assert_eq!(health.commander_matches, 4);

        // 2. Check /api/v1/meta/commanders?format=Commander
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/meta/commanders?format=Commander&min_matches=1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let commanders: Vec<CommanderTierEntry> =
            serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
        assert_eq!(commanders.len(), 1);
        let c = &commanders[0];
        assert_eq!(c.commander_grp_id, 77701);
        assert_eq!(c.matches, 4);
        assert_eq!(c.wins, 3);
        assert!((c.win_rate - 0.75).abs() < 1e-4);
        assert!((c.play_win_rate - 1.0).abs() < 1e-4);
        assert!((c.draw_win_rate - 0.5).abs() < 1e-4);
        assert!((c.salt_index - 0.5).abs() < 1e-4); // 2 scoops in 4 matches
        assert_eq!(c.tier, "S");

        // 3. Check /api/v1/meta/cards?format=Commander&commander_grp_id=77701
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/meta/cards?format=Commander&commander_grp_id=77701")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let cards: Vec<CardPerformanceEntry> =
            serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
        assert_eq!(cards.len(), 2);
        let study = cards.iter().find(|x| x.grp_id == 90001).unwrap();
        assert_eq!(study.matches_seen, 4);
        assert_eq!(study.wins_when_seen, 3);
        assert_eq!(study.total_cards_drawn, 28); // 7 cards * 4 matches
        assert_eq!(study.top_titles[0].title, "Rhystic Tracker (Gold)");

        let hoof = cards.iter().find(|x| x.grp_id == 90002).unwrap();
        assert_eq!(hoof.total_damage, 168);
        assert_eq!(hoof.max_single_hit, 24);

        // 4. Check /api/v1/meta/matchups?commander_grp_id=77701&format=Commander
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/meta/matchups?commander_grp_id=77701&format=Commander")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let matchups: Vec<CommanderMatchupEntry> =
            serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
        assert_eq!(matchups.len(), 1);
        assert_eq!(matchups[0].opponent_commander_grp_id, 88801);
        assert_eq!(matchups[0].matches, 4);
        assert_eq!(matchups[0].wins, 3);
    }

    #[tokio::test]
    async fn formats_endpoint_lists_all_constructed_and_limited_except_alchemy() {
        let db = AnalyticalDb::connect(":memory:").await.unwrap();
        let app = build_router(db.clone());

        // Even with 0 matches, /api/v1/meta/formats returns all supported MTGA Constructed & Limited formats
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/meta/formats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let formats: Vec<FormatMetaSummary> =
            serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
        assert!(formats.iter().any(|f| f.format == "Commander" && f.category == "Commander & Brawl"));
        assert!(formats.iter().any(|f| f.format == "Standard Ranked" && f.category == "Constructed"));
        assert!(formats.iter().any(|f| f.format == "Historic Ranked" && f.category == "Constructed"));
        assert!(formats.iter().any(|f| f.format == "Timeless Ranked" && f.category == "Constructed"));
        assert!(formats.iter().any(|f| f.format == "Pioneer Ranked" && f.category == "Constructed"));
        assert!(formats.iter().any(|f| f.format == "Draft" && f.category == "Limited"));
        assert!(formats.iter().any(|f| f.format == "Sealed" && f.category == "Limited"));
        assert!(!formats.iter().any(|f| f.format.to_lowercase().contains("alchemy")));

        // Verify Alchemy payload is rejected at ingestion while Draft and Standard Ranked are accepted
        let mut draft_match = sample_commander_match(
            "rh_draft_1",
            "u1",
            "win",
            Some("ResultReason_Game"),
            true,
            14,
            0,
            0,
        );
        draft_match.format = "Draft".to_string();
        draft_match.hero_commander_id = None;
        draft_match.opponent_commander_id = None;

        let mut std_match = draft_match.clone();
        std_match.match_hash = "rh_std_1".to_string();
        std_match.format = "Standard Ranked".to_string();

        let mut alchemy_match = draft_match.clone();
        alchemy_match.match_hash = "rh_alch_1".to_string();
        alchemy_match.format = "Alchemy Ranked".to_string();

        let report = db
            .ingest_batch(&[draft_match, std_match, alchemy_match])
            .await
            .unwrap();
        assert_eq!(report.accepted, 2);
        assert_eq!(report.rejected, 1);
    }
}
