# Changelog

All notable changes to Rhystic Tracker are documented in this file.

## [2.0.0] - 2026-10-26 — 🌐 Rhystic Cloud Meta Platform, Live Deck Overlays, Bento Dashboard & Core Engine Overhaul

### 🚀 Major New Features

- **Community Meta Server & Glassmorphic Web Portal (`server/`)**:
  - Added a standalone, self-hostable Rust (`axum` + `sqlx`) ingestion and analytical server (`server/src/lib.rs`, `server/src/db.rs`, `server/src/portal.rs`).
  - **4-Layer Archetype & Deck Clustering Engine**: Groups submitted lists by exact `deck_hash` (collapsing basic land printings), Commander/Brawl pairings, TF-IDF Weighted Jaccard 60-card archetype similarity ($\ge 0.65$), and 17Lands-style Limited color pairs.
  - **Unified Core vs. Flex-Slot Matrix**: Side-by-side breakdown of **Core Cards ($\ge 70\%$ inclusion)** vs. **Flex Slots ($< 70\%$ inclusion)** with inclusion win-rate delta bars and an in-place **Variant Switcher**.
  - **Scryfall Art Headers, Hover Previews & 1-Click MTGA Export**: Zero-blocking Scryfall `art_crop` blurred headers for Commanders and Archetypes, interactive floating `normal` card hover previews, real `mana-font` WUBRG pips, Keyrune icons, and 1-click MTGA clipboard deck export.
  - **Global Claimed Player & Hall of Fame Leaderboards**: Public REST endpoints (`/api/v1/meta/*`, `/api/v1/leaderboards`) and interactive portal views for top commanders, salt index, deadliest finishers, and player win rates.

- **Opt-In 3-Tier Cloud Sync & First-Seen Handle Pinning (`src-tauri/src/sync.rs`, `server/src/db.rs`)**:
  - Renamed **MTGA Connection** to **Connections & Cloud Sync** (`SettingsView.tsx`, `ConnectionTab.tsx`) with a 3-tier opt-in privacy architecture:
    - *Tier 0 (Offline Default)*: Zero outbound network requests.
    - *Tier 1 (Anonymous Meta Contributor)*: Submits match outcomes, turn counts, format, and card telemetry using a local `contributor_id` UUID.
    - *Tier 2 (Full Decklist Contributor)*: Includes maindeck card quantities for Core/Flex archetype clustering.
    - *Tier 3 (Log-Detected / Claimed Leaderboard Profile)*: Attributes records to the user's log-detected MTG Arena handle (`AuthenticateResponse`).
  - **Server-Side First-Seen Handle Pinning (`claimed_handles`)**: Binds each normalized Arena handle and `player_account_hash` to `SHA-256("rhystic-handle-claim-v1:" || contributor_id)` on first sync. Spoofed handle attributions from `Player.log` tampering are automatically stripped while preserving valid anonymous telemetry.
  - **Cross-Device Contributor Token Recovery**: Added 1-click copy and restore controls in **Settings → Connections & Cloud Sync** (`get_sync_contributor_token` / `restore_sync_contributor_token`) so users can migrate their handle ownership key across PCs or OS reinstalls.

- **Live Match HUD & Live Deck Tracker Overlays (`LiveHUDView.tsx`, `LiveDeckView.tsx`)**:
  - **Always-on-Top Live Match HUD**: Real-time life bars scaled dynamically by format (`40` for Commander, `25` for Brawl, `20` for Constructed/Limited, expanding dynamically on lifegain), Commander zone tracking, turn-by-turn combat/spell damage feed, and embedded split-panel Deck Tracker toggle (`liveHudDeckSplit`).
  - **Live Deck Tracker**: Real-time hypergeometric draw probabilities ($1+$, land, and per-card odds), live library/hand/battlefield/graveyard/exile zone tracking, and configurable match-start auto-switching (`Off` | `Live Match HUD` | `Live Deck Tracker`).

- **21 Custom SVG Card Trophies (`ImpactfulCardTracker`) & Activated Deck Honors**:
  - **Card Trophies**: 21 custom multi-tier SVG achievement emblems (Iron, Bronze, Silver, Gold, Titanium/Platinum, Legendary) across 7 combat categories (*Executioner*, *Overkiller*, *Board Wiper*, *Card Draw Engine*, *Battlefield Stalwart*, *Negator*, *Scoop Inducer*, and more).
  - **Deck Honors (`deck_achievements.rs`)**: Activated and backfilled deck-level honors (*On a Roll*, *Comeback Kid*, *Gotta Go Fast*, *Iron Fortress*, *Marathon Master*, *Deck Dominance*) with starting-life-relative scaling for 20-life, 25-life Brawl, and 40-life Commander games.

- **Customizable Multi-Page Bento Dashboard Widgets (`src/components/dashboard/`)**:
  - Modular drag, resize, and multi-page dashboard layout with sticky navigation and persistent page memory.
  - Dedicated widgets for Time-Series Win Rates, Format Distribution, Commander Spotlight, Mana Curve & Speed Clocks, Play vs. Draw Splits, Mulligan Analytics, Nemesis & Victim Tracker, and Lucky/Cursed Cards.

- **Economy, Mastery Pass, Daily Quests & Collection Analytics**:
  - **Currencies, Wildcards & Vault**: Physical fanned foil booster pack visuals (including Golden Packs), rotary combination-safe Vault progress, and `WildcardHistoryModal` with timeline craft/acquisition markers.
  - **Daily Quests & Reroll Engine**: 3-slot MTGA quest layout with `✔ COMPLETED` stamps, automated daily quest reroll detection (`player_quest_rerolls`), and 500g $\to$ 750g upgrade conversion analytics.
  - **Collection & Set Completion**: Visual 4×3 card library, set completion progress bars, 4-diamond ownership toggles, and persistent per-card custom set/printing preferences.

---

### 🛡️ Core Engine, Reliability & Security Fixes

- **Single-Flight SQLite Initialization (`src-tauri/src/db.rs`)**:
  - Refactored `DatabaseManager::init()` so the entire SQLite pool creation, schema migration, and backfill sequence runs inside `DB_SINGLETON.get_or_try_init(...)`, preventing concurrent startup race conditions between the background log tailer and Tauri IPC commands.
- **Removal of Destructive Account-Specific Startup Queries (`src-tauri/src/db.rs`)**:
  - Removed hardcoded startup `DELETE` statements and account-specific card/deck heuristics that could corrupt or delete legitimate user matches and decks on startup.
- **Best-of-3 Multi-Game State Isolation & Turn Accumulation (`src-tauri/src/match_assembler.rs`)**:
  - Ensured `MatchAssembler::start_match` completely resets all per-match and per-game tracking maps (`instance_previous_zone_map`, `turn_damage_taken_by_instance`, `last_hero_damage_hit`, `processed_msg_ids`, etc.) when starting a new `match_id`.
  - Preserved match-wide accumulators (`match_start_time`, `player_cards_seen`, `turn_events`, `impactful_cards`, and `completed_games_turns`) across duplicate room-state events and Best-of-3 Game 2 / Game 3 transitions while resetting per-game battlefield state and life totals.
- **String-Aware JSON Log Accumulator & Rotation Reset (`src-tauri/src/main.rs`)**:
  - Replaced naive `{`/`}` character counting in `process_tailer_events` with a string-aware JSON scanner that tracks `in_string` and `escape_next` states across lines, ignores braces inside string literals (such as player names or localization strings), enforces a 2 MB safety cap, and clears partial buffers on `TailerEvent::Rotated`.
- **Out-of-Order `MatchCompleted` ID Guard (`src-tauri/src/match_assembler.rs`, `src-tauri/src/main.rs`)**:
  - Added `MatchAssembler::complete_match_for_id` to verify incoming `ParsedEvent::MatchCompleted` `match_id` values against `active_match.match_id`, ignoring stale/out-of-order completion events and populating placeholder IDs when needed.
- **Elimination of N+1 SQLite Queries & Scoped Cloud Sync Lookups (`src-tauri/src/commands/matches.rs`, `src-tauri/src/sync.rs`, `src/App.tsx`)**:
  - Replaced per-card `get_card_metadata` loops in `get_impactful_cards` and `get_match_turn_events` with single batched `WHERE grp_id IN (...)` queries.
  - Scoped `enqueue_match_for_sync` in `sync.rs` to query only the `grp_id`s present in the match instead of loading the entire `cards_cache` table into memory.
  - Added a `cancelled` cleanup guard and `prevSelectedMatchIdRef` check in `App.tsx` to eliminate stale IPC races and UI flicker during background match polling.
- **Zero-Warning Rust & Vite Production Build**:
  - Cleaned up unused legacy helpers and standardized static `@tauri-apps/api/core` imports across the frontend.

---

### 🙏 Attribution & License

Rhystic Tracker v2.0.0 is an extended fork of [**Balthazzahr/Rhystic-Tracker**](https://github.com/Balthazzahr/Rhystic-Tracker), originally created by [**Balthazzahr**](https://github.com/Balthazzahr) under the [MIT License](LICENSE). Special thanks to **Balthazzahr** for building the foundational 1.x local-first MTG Arena tracker architecture,Belere/Plantin visual identity, and GRE combat parser upon which v2.0.0 is built.

- **Copyright (c) 2026 [Balthazzahr](https://github.com/Balthazzahr)**
- **Copyright (c) 2026 [Matt4C6F656872](https://github.com/Matt4C6F656872)** *(Fork modifications and v2.0.0 additions)*

For historical 1.x release notes, see [`docs/CHANGELOG.md`](docs/CHANGELOG.md).

