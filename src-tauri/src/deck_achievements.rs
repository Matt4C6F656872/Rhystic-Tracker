//! Deck-level achievements ("On a Roll", "Comeback Kid", "Gotta Go Fast",
//! "Iron Fortress", "Marathon", "Dominance").
//!
//! The evaluation rules are kept in one pure function ([`evaluate`]) so they can
//! be unit-tested and reused by both the live match pipeline and the history
//! backfill. Thresholds MUST stay in sync with `DECK_ACHIEVEMENTS_REGISTRY` in
//! `src/utils/achievementBadges.ts` (the user-facing copy).
//!
//! Life-based rules are measured relative to the player's *starting* life so
//! they are fair across formats (20-life constructed, 25-life Brawl, and a
//! potential 40-life Commander queue).

use crate::db::DatabaseManager;
use crate::deck_legitimacy::preset_deck_reason;
use crate::match_assembler::PRESET_EVENT_DECK_NAME;
use std::collections::HashMap;

pub const ON_A_ROLL: &str = "on_a_roll";
pub const COMEBACK_KID: &str = "comeback_kid";
pub const GOTTA_GO_FAST: &str = "gotta_go_fast";
#[allow(dead_code)]
pub const BLITZKRIEG: &str = GOTTA_GO_FAST;
pub const IRON_FORTRESS: &str = "iron_fortress";
pub const MARATHON: &str = "marathon";
pub const DOMINANCE: &str = "dominance";

/// Facts about one finished match, as needed by the deck achievement rules.
#[derive(Debug, Clone)]
pub struct DeckMatchFacts<'a> {
    pub format: &'a str,
    pub is_win: bool,
    /// Global game turn counter (both players' turns), as shown in Match History.
    pub turns: u32,
    pub result_reason: Option<&'a str>,
    pub end_life: Option<i32>,
    /// Lowest life total the player reached during the match (None for matches
    /// recorded before this was tracked).
    pub min_life: Option<i32>,
    /// Observed starting life (None if unknown; falls back to the format default).
    pub starting_life: Option<i32>,
    /// Consecutive wins with this deck, including this match.
    pub win_streak: u32,
}

/// Starting life assumed when the real value was not observed.
pub fn default_starting_life(format: &str) -> i32 {
    let f = format.to_lowercase();
    if f.contains("commander") && !f.contains("brawl") {
        40
    } else if f.contains("brawl") {
        25
    } else {
        20
    }
}

/// Number of turns the *player* took when winning on their own turn.
fn player_turns(turns: u32) -> u32 {
    (turns + 1) / 2
}

fn is_concede(reason: Option<&str>) -> bool {
    reason
        .map(|r| {
            let r = r.to_lowercase();
            r.contains("concede") || r.contains("scoop")
        })
        .unwrap_or(false)
}

/// Whether a match can count toward deck achievements at all.
pub fn is_eligible(format: &str, deck_name: &str, preset_or_assigned: bool) -> bool {
    if preset_or_assigned {
        return false;
    }
    if format.eq_ignore_ascii_case("Bot Match") {
        return false;
    }
    let name = deck_name.trim();
    if name.is_empty() || name == "Selected Deck" || name == PRESET_EVENT_DECK_NAME {
        return false;
    }
    if name.to_lowercase().starts_with("jump in") {
        return false;
    }
    preset_deck_reason(name).is_none()
}

/// Evaluate all deck achievements for one match. Returns `(achievement_id, tier)`
/// pairs, at most one (the highest qualifying) tier per achievement.
pub fn evaluate(f: &DeckMatchFacts) -> Vec<(&'static str, &'static str)> {
    let mut out = Vec::new();
    if !f.is_win {
        return out;
    }
    let concede = is_concede(f.result_reason);
    let start = f.starting_life.unwrap_or_else(|| default_starting_life(f.format));
    let your_turns = player_turns(f.turns);

    // On a Roll: consecutive wins with this deck.
    let tier = match f.win_streak {
        s if s >= 7 => Some("gold"),
        s if s >= 5 => Some("silver"),
        s if s >= 3 => Some("bronze"),
        _ => None,
    };
    if let Some(t) = tier {
        out.push((ON_A_ROLL, t));
    }

    // Comeback Kid: won after dropping to near-death (absolute life, any format).
    if let Some(min) = f.min_life {
        let tier = match min {
            m if m <= 1 => Some("gold"),
            m if m <= 2 => Some("silver"),
            m if m <= 5 => Some("bronze"),
            _ => None,
        };
        if let Some(t) = tier {
            out.push((COMEBACK_KID, t));
        }
    }

    // Gotta Go Fast: won by your Nth turn (excludes opponent concessions).
    if !concede && f.turns > 0 {
        let tier = match your_turns {
            t if t <= 2 => Some("gold"),
            t if t <= 3 => Some("silver"),
            t if t <= 4 => Some("bronze"),
            _ => None,
        };
        if let Some(t) = tier {
            out.push((GOTTA_GO_FAST, t));
        }
    }

    // Iron Fortress: little to no life lost over a real game (>= 4 of your turns,
    // no early concession).
    if let Some(min) = f.min_life {
        if !concede && your_turns >= 4 {
            let lost = (start - min).max(0);
            let tier = match lost {
                0 => Some("gold"),
                l if l <= 2 => Some("silver"),
                l if l <= 5 => Some("bronze"),
                _ => None,
            };
            if let Some(t) = tier {
                out.push((IRON_FORTRESS, t));
            }
        }
    }

    // Marathon: game reached turn 15/20/25 (global turn counter).
    let tier = match f.turns {
        t if t >= 25 => Some("gold"),
        t if t >= 20 => Some("silver"),
        t if t >= 15 => Some("bronze"),
        _ => None,
    };
    if let Some(t) = tier {
        out.push((MARATHON, t));
    }

    // Dominance: finished far above starting life.
    if let Some(end) = f.end_life {
        let gain = end - start;
        let tier = match gain {
            g if g >= 30 => Some("gold"),
            g if g >= 20 => Some("silver"),
            g if g >= 10 => Some("bronze"),
            _ => None,
        };
        if let Some(t) = tier {
            out.push((DOMINANCE, t));
        }
    }

    out
}

/// Consecutive-win streak for a deck, newest match first, counting only
/// eligible (non-bot, non-preset) matches already stored in the database.
pub async fn current_win_streak(db: &DatabaseManager, deck_name: &str) -> u32 {
    let results: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT m.result
        FROM matches m
        LEFT JOIN match_decks d ON d.match_id = m.id
        WHERE m.hero_deck_name = ?
          AND m.format != 'Bot Match'
          AND COALESCE(d.preset_deck, 0) = 0
        ORDER BY m.timestamp DESC
        LIMIT 50
        "#,
    )
    .bind(deck_name)
    .fetch_all(db.pool())
    .await
    .unwrap_or_default();

    results.iter().take_while(|r| r.eq_ignore_ascii_case("win")).count() as u32
}

/// Evaluate and persist deck achievements for a just-finished (already stored)
/// match. Returns the newly earned `(achievement_id, tier)` pairs.
pub async fn award_for_match(
    db: &DatabaseManager,
    match_id: &str,
    deck_name: &str,
    facts_without_streak: DeckMatchFacts<'_>,
) -> Vec<(&'static str, &'static str)> {
    let mut facts = facts_without_streak;
    facts.win_streak = if facts.is_win { current_win_streak(db, deck_name).await.max(1) } else { 0 };

    let mut newly = Vec::new();
    for (id, tier) in evaluate(&facts) {
        if let Ok(true) = db.record_deck_achievement(deck_name, id, tier, Some(match_id), None).await {
            newly.push((id, tier));
        }
    }
    newly
}

/// Recompute deck achievements from the full match history. Idempotent: rows
/// that already exist are kept (`ON CONFLICT DO NOTHING`), and `achieved_at` is
/// set to the original match time. Returns the number of newly inserted rows.
pub async fn backfill(db: &DatabaseManager) -> usize {
    let rows = sqlx::query_as::<_, (String, String, String, String, i64, Option<String>, Option<String>, Option<i64>, Option<i64>, Option<i64>, i64)>(
        r#"
        SELECT m.id, m.timestamp, m.format, m.result, m.turns, m.result_reason,
               m.hero_deck_name, m.hero_life_end, m.hero_min_life, m.hero_starting_life,
               COALESCE(d.preset_deck, 0)
        FROM matches m
        LEFT JOIN match_decks d ON d.match_id = m.id
        ORDER BY m.timestamp ASC
        "#,
    )
    .fetch_all(db.pool())
    .await;

    let rows = match rows {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[DECK ACHIEVEMENTS] Backfill query failed: {}", e);
            return 0;
        }
    };

    let mut streaks: HashMap<String, u32> = HashMap::new();
    let mut inserted = 0usize;

    for (id, ts, format, result, turns, reason, deck, end_life, min_life, start_life, preset) in rows {
        let deck = match deck {
            Some(d) => d,
            None => continue,
        };
        if !is_eligible(&format, &deck, preset != 0) {
            continue;
        }
        let is_win = result.eq_ignore_ascii_case("win");
        let streak = streaks.entry(deck.clone()).or_insert(0);
        *streak = if is_win { *streak + 1 } else { 0 };
        if !is_win {
            continue;
        }

        let facts = DeckMatchFacts {
            format: &format,
            is_win,
            turns: turns.max(0) as u32,
            result_reason: reason.as_deref(),
            end_life: end_life.map(|v| v as i32),
            min_life: min_life.map(|v| v as i32),
            starting_life: start_life.map(|v| v as i32),
            win_streak: *streak,
        };
        for (aid, tier) in evaluate(&facts) {
            if let Ok(true) = db.record_deck_achievement(&deck, aid, tier, Some(&id), Some(&ts)).await {
                inserted += 1;
            }
        }
    }
    inserted
}

#[cfg(test)]
mod tests {
    use super::*;

    fn win() -> DeckMatchFacts<'static> {
        DeckMatchFacts {
            format: "Standard",
            is_win: true,
            turns: 12,
            result_reason: Some("ResultReason_Game"),
            end_life: Some(14),
            min_life: Some(10),
            starting_life: Some(20),
            win_streak: 1,
        }
    }

    fn tier_of(out: &[(&str, &str)], id: &str) -> Option<String> {
        out.iter().find(|(a, _)| *a == id).map(|(_, t)| t.to_string())
    }

    #[test]
    fn losses_never_award() {
        let mut f = win();
        f.is_win = false;
        f.win_streak = 9;
        f.min_life = Some(1);
        assert!(evaluate(&f).is_empty());
    }

    #[test]
    fn plain_win_awards_nothing() {
        assert!(evaluate(&win()).is_empty());
    }

    #[test]
    fn on_a_roll_tiers() {
        let mut f = win();
        f.win_streak = 2;
        assert_eq!(tier_of(&evaluate(&f), ON_A_ROLL), None);
        f.win_streak = 3;
        assert_eq!(tier_of(&evaluate(&f), ON_A_ROLL).as_deref(), Some("bronze"));
        f.win_streak = 5;
        assert_eq!(tier_of(&evaluate(&f), ON_A_ROLL).as_deref(), Some("silver"));
        f.win_streak = 7;
        assert_eq!(tier_of(&evaluate(&f), ON_A_ROLL).as_deref(), Some("gold"));
    }

    #[test]
    fn comeback_kid_tiers_and_unknown_min_life() {
        let mut f = win();
        f.min_life = Some(5);
        assert_eq!(tier_of(&evaluate(&f), COMEBACK_KID).as_deref(), Some("bronze"));
        f.min_life = Some(2);
        assert_eq!(tier_of(&evaluate(&f), COMEBACK_KID).as_deref(), Some("silver"));
        f.min_life = Some(1);
        assert_eq!(tier_of(&evaluate(&f), COMEBACK_KID).as_deref(), Some("gold"));
        f.min_life = None;
        assert_eq!(tier_of(&evaluate(&f), COMEBACK_KID), None);
    }

    #[test]
    fn gotta_go_fast_uses_player_turns_and_ignores_concedes() {
        let mut f = win();
        f.turns = 7; // your 4th turn on the play
        assert_eq!(tier_of(&evaluate(&f), GOTTA_GO_FAST).as_deref(), Some("bronze"));
        f.turns = 8; // your 4th turn on the draw
        assert_eq!(tier_of(&evaluate(&f), GOTTA_GO_FAST).as_deref(), Some("bronze"));
        f.turns = 9;
        assert_eq!(tier_of(&evaluate(&f), GOTTA_GO_FAST), None);
        f.turns = 5;
        assert_eq!(tier_of(&evaluate(&f), GOTTA_GO_FAST).as_deref(), Some("silver"));
        f.turns = 3;
        assert_eq!(tier_of(&evaluate(&f), GOTTA_GO_FAST).as_deref(), Some("gold"));
        f.result_reason = Some("ResultReason_Concede");
        assert_eq!(tier_of(&evaluate(&f), GOTTA_GO_FAST), None);
    }

    #[test]
    fn iron_fortress_is_relative_to_starting_life() {
        let mut f = win();
        f.turns = 10;
        f.starting_life = Some(20);
        f.min_life = Some(20);
        assert_eq!(tier_of(&evaluate(&f), IRON_FORTRESS).as_deref(), Some("gold"));
        f.min_life = Some(18);
        assert_eq!(tier_of(&evaluate(&f), IRON_FORTRESS).as_deref(), Some("silver"));
        f.min_life = Some(15);
        assert_eq!(tier_of(&evaluate(&f), IRON_FORTRESS).as_deref(), Some("bronze"));
        f.min_life = Some(14);
        assert_eq!(tier_of(&evaluate(&f), IRON_FORTRESS), None);

        // Brawl (25 life): dropping to 20 is a 5-life loss -> bronze, not "never below 15".
        f.format = "Standard Brawl";
        f.starting_life = Some(25);
        f.min_life = Some(20);
        assert_eq!(tier_of(&evaluate(&f), IRON_FORTRESS).as_deref(), Some("bronze"));

        // 40-life Commander falls back correctly when the start is observed.
        f.format = "Commander";
        f.starting_life = Some(40);
        f.min_life = Some(40);
        assert_eq!(tier_of(&evaluate(&f), IRON_FORTRESS).as_deref(), Some("gold"));
    }

    #[test]
    fn iron_fortress_requires_a_real_game() {
        let mut f = win();
        f.min_life = Some(20);
        f.turns = 5; // only 3 of your turns
        assert_eq!(tier_of(&evaluate(&f), IRON_FORTRESS), None);
        f.turns = 10;
        f.result_reason = Some("ResultReason_Concede");
        assert_eq!(tier_of(&evaluate(&f), IRON_FORTRESS), None);
    }

    #[test]
    fn marathon_tiers() {
        let mut f = win();
        f.turns = 14;
        assert_eq!(tier_of(&evaluate(&f), MARATHON), None);
        f.turns = 15;
        assert_eq!(tier_of(&evaluate(&f), MARATHON).as_deref(), Some("bronze"));
        f.turns = 20;
        assert_eq!(tier_of(&evaluate(&f), MARATHON).as_deref(), Some("silver"));
        f.turns = 25;
        assert_eq!(tier_of(&evaluate(&f), MARATHON).as_deref(), Some("gold"));
    }

    #[test]
    fn dominance_is_relative_to_starting_life() {
        let mut f = win();
        f.end_life = Some(30);
        assert_eq!(tier_of(&evaluate(&f), DOMINANCE).as_deref(), Some("bronze"));
        f.end_life = Some(40);
        assert_eq!(tier_of(&evaluate(&f), DOMINANCE).as_deref(), Some("silver"));
        f.end_life = Some(50);
        assert_eq!(tier_of(&evaluate(&f), DOMINANCE).as_deref(), Some("gold"));

        // Brawl needs 35 for bronze; unknown starting life uses the format default.
        f.format = "Brawl";
        f.starting_life = None;
        f.end_life = Some(34);
        assert_eq!(tier_of(&evaluate(&f), DOMINANCE), None);
        f.end_life = Some(35);
        assert_eq!(tier_of(&evaluate(&f), DOMINANCE).as_deref(), Some("bronze"));
    }

    #[test]
    fn eligibility_rules() {
        assert!(is_eligible("Standard", "Mono Red Aggro", false));
        assert!(!is_eligible("Bot Match", "Mono Red Aggro", false));
        assert!(!is_eligible("Standard", "Mono Red Aggro", true));
        assert!(!is_eligible("Standard", "", false));
        assert!(!is_eligible("Standard", "Selected Deck", false));
        assert!(!is_eligible("Standard", PRESET_EVENT_DECK_NAME, false));
        assert!(!is_eligible("Jump In", "Jump In: Dragons", false));
        assert!(!is_eligible("Standard", "?=?Loc/Decks/Precon/Something", false));
    }
}

