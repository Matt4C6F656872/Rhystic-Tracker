# Rhystic Cloud & Platform Vision & Roadmap

This document outlines the architectural strategy, vision, and roadmap for evolving Rhystic Tracker from a local-first desktop tracker into an open, community-driven analytics platform with an optional opt-in central server (inspired by platforms like Untapped.gg and 17lands).

---

## 1. Legal, Licensing & Values

- **License:** Licensed under the [MIT License](file:///home/matt/Code/Rhystic-Tracker-1.7.8/LICENSE). Anyone may fork, modify, rename, distribute, and build on the codebase provided the original MIT copyright notice is preserved.
- **WotC Fan Content Policy:** Strictly compliant. All tools and analytics are free forever. Optional donations / patronage cover server hosting and infrastructure costs with zero mandatory feature paywalls.
- **Privacy-First & Zero Telemetry by Default:** The application remains 100% operational offline. Communication with central servers is strictly opt-in.

---

## 2. Product Vision & Opportunities

### A. The "17lands for Brawl & Commander" (Rhystic's Niche)
While Untapped and 17lands focus heavily on 60-card Standard or Limited draft picks, Rhystic Tracker already houses the deepest Brawl and Commander combat engine in existence:
- **Commander Tier Lists & Salt Index:** Win rates, average turn counts, scoop frequency, and play rates for every commander on Arena.
- **Card Performance Metrics:** Game-in-hand win rate (GIH WR), cast frequency, and combat efficiency.
- **Micro-Combat Telemetry:**
  - *Deadliest Finishers:* Cards dealing the most lethal combat damage.
  - *Counterspell Impact:* Win-rate correlations when countering high-CMC spells.
  - *Titles:* Community stats for badges like "Scoop Inducer", "Royal Assassin", and "Negator".

### B. Global Meta Analytics & Matchup Matrix
- Aggregated archetype win rates across Standard, Historic, Timeless, Brawl, and Draft.
- Head-to-head matchup matrices (e.g. Dimir Midrange vs. Mono-Red on the play vs. on the draw).
- Deck building wildcard cost calculations integrated with user's uploaded collection.

### C. Web Companion & Social Sharing
- **Mobile/iPad Companion:** View match history, decks, and real-time live matches from Safari on an iPad or phone on your desk.
- **Shareable Match Timelines:** Direct web links displaying the visual turn-by-turn combat replay.

---

## 3. Core Architecture Rethink

### A. Live Match HUD vs. Live Deck Tracker UX
- **The Problem:** Two separate sidebar tabs created competition over auto-switching during match start and split the player's focus during games.
- **The Fix:**
  - Introduce an **On Match Start Auto-Switch** selector: `Off` | `Live Match HUD` | `Live Deck Tracker` | `HUD + Deck Split`.
  - Provide an embedded split-view mode (or toggle) in the Live HUD so deck remaining probabilities and live game logs coexist naturally.

### B. Leaderboards: Local vs. Global Community
- **Local Scope (Offline):** Personal Hall of Fame for single-hit damage, lifetime damage, and spell records.
- **Global Scope (Server):** Global Commander tier lists, community highest combat strikes, and format trends.

### C. Achievements: Card & Deck Synergy + Global Rarity
- Activate the shelved Deck Achievements engine (`On a Roll`, `Comeback Kid`, `Gotta Go Fast`, `Iron Fortress`, `Marathon Master`, `Deck Dominance`).
- Calculate Steam-style global rarity percentages on achievement badges without leaking player identities.

### D. Multi-Tier & Granular Privacy Architecture
- **Tier 0 (Offline Default):** Zero network traffic to central servers.
- **Tier 1 (Anonymous Meta Contributor):** One-click opt-in using a locally generated UUID (`contributor_id`). Submits match outcomes, turn counts, format, and card performance anonymously (no screen names).
- **Tier 2 (Linked Companion Profile):** Optional private `sync_key` for cross-device synchronization (Desktop PC, Linux/Steam Deck, Web/iPad companion) and private collection wildcard calculations.
- **Tier 3 (Public Leaderboard & Hall of Fame Profile):** Opt-in public player handle (`display_name`, with `#12345` Arena discriminator stripped) and seasonal Constructed/Limited rank. Unlocks placement on **Global Player Leaderboards**, Community Hall of Fame records, and attributed top decklists.
- **Granular Per-Signal Toggles (User-Controlled Checkboxes):**
  1. `share_match_outcomes`: Format, Win/Loss, Turn count, Play/Draw, Duration.
  2. `share_card_telemetry`: Seen card `grp_id`s, combat/spell damage, cards drawn, and earned combat titles.
  3. `share_full_decklist`: Full submitted maindeck/sideboard `(grp_id, count)` list for archetype clustering and deck win rates.
  4. `share_player_identity`: Public display name and rank tier for Global Leaderboards & Hall of Fame attribution.
  5. `share_collection_private`: Private collection counts tied only to the user's `sync_key` for web/iPad "Missing Wildcards" calculations.

### E. Deck & Meta Archetype Clustering Engine (How Deck Winrates Work)
Because players tweak 3–10 flex slots even in established meta decks, Rhystic Cloud uses a 4-layer aggregation pipeline (combining the best of Untapped.gg and 17Lands):
1. **Layer 1 — Exact Deck Fingerprint (`deck_hash`):**
   - Normalizes cosmetic reprints to canonical card names/oracle IDs and collapses basic land printings (`Plains`, `Island`, `Swamp`, `Mountain`, `Forest`, `Wastes`), then hashes sorted `(card_id, quantity)` pairs. Identical 60-card lists share the exact same `deck_hash`.
2. **Layer 2 — Commander & Brawl Grouping (`commander_grp_id` + Core/Flex Split):**
   - Singleton 100-card decks group primarily by `commander_grp_id` (+ `companion_grp_id`). Within each Commander, cards appearing in $\ge 70\%$ of lists form the **Core Package**, while flex slots are ranked by **Inclusion Win-Rate Delta** and **Seen Win Rate**.
3. **Layer 3 — 60-Card Constructed Archetype Clustering (TF-IDF Weighted Jaccard):**
   - Strips basic lands, computes color identity (`WUBRG`), and weights non-land cards by specificity (synergy build-arounds carry higher weight than ubiquitous removal).
   - Decklists with weighted Jaccard similarity $\ge 0.65$ (~85% shared non-land core) cluster into a single **Meta Archetype** (e.g. *Izzet Spellementals*, *Dimir Reanimator*), reporting both the **Overall Archetype Win Rate** and the **Highest-Winrate 60-Card Variant (`deck_hash`)**.
4. **Layer 4 — Limited Analytics (`Draft` & `Sealed` — 17Lands Methodology):**
   - Groups Limited decks by **Format + Color Pair (`WU`, `BR`, `GWr`)** and evaluates cards via **GP WR** (Maindeck Win Rate), **Seen WR** (Game-in-Hand / Cast Win Rate), and **IWD** (Improvement When Drawn).

## 4. Repository Strategy: Upstream Tracking & `+web` Edition Architecture

To avoid duplicating desktop UI work while maintaining a full-featured telemetry and self-hostable server platform, development follows a two-repository model:

1. **Upstream Sync & PR Repository (`Matt4C6F656872/Rhystic-Tracker`):**
   - Tracks `Balthazzahr/Rhystic-Tracker` (`upstream/main`) directly.
   - Used to package and submit core engine bug fixes, SQLite optimizations, Bo3 state fixes, and Linux/WebKitGTK rendering improvements as clean Pull Requests to upstream.
2. **Telemetry & Server Distribution (`Matt4C6F656872/Rhystic-Tracker-Web`):**
   - Releases follow the **`v<upstream_version>+web.<revision>`** naming scheme (e.g., `v1.8.0+web.1`).
   - Merges every official upstream release tag and layers the standalone `server/`, `src-tauri/src/sync.rs` outbox, and client telemetry views on top via minimal hook points.

---

## 5. Implementation Roadmap

### Phases 1 – 5: Core Telemetry, Sync Outbox, Handle Pinning & Web Portal (Completed)
- [x] **Phase 1–2:** Auto-Switch on Match Start, Commander/Brawl format promotion, and scaled Deck Honors activation (`deck_achievements.rs`).
- [x] **Phase 3:** Opt-in SQLite `sync_outbox` queue and `AnonymizedMatchPayload` dispatcher (`src-tauri/src/sync.rs`).
- [x] **Phase 4–4.5:** Rust `axum` ingestion server (`server/`), 3-tier privacy controls (`ConnectionTab.tsx`), First-Seen Handle Pinning (`claimed_handles`), and 4-layer Archetype + Core/Flex clustering.
- [x] **Phase 5:** Responsive Web Companion & Community Portal (`server/src/portal.rs`).

---

### Phase 6: Upstream `v1.8.0` Reconciliation & Core Bugfix PRs (Next Up)
- [ ] **Reconcile Upstream `v1.8.0` into `Rhystic-Tracker-Web` (`v1.8.0+web.1`):**
  - Merge upstream `v1.8.0` (`/overlay/deck-tracker`, 3-way Land/Creature/Spell draw odds, Brawl Command Zone pin, and `autoOpenDeckTracker` setting).
  - Port our Linux `WEBKIT_DISABLE_COMPOSITING_MODE=1` solid-backing fix, hover transition stripping, and `48×48` floating symbol collapse mode onto the unified overlay.
- [ ] **Submit Core Engine & Linux Hardening PRs to Upstream (`Balthazzahr/Rhystic-Tracker`):**
  - **PR 1 (Engine & SQLite Stability):** `DB_SINGLETON` race condition fix (`db.rs`), string-aware `Player.log` JSON brace scanner + 2 MB buffer cap (`main.rs`), Bo3 per-game state isolation, and out-of-order `MatchCompleted` ID verification (`match_assembler.rs`).
  - **PR 2 (Linux WebKitGTK & Query Performance):** Batched SQLite card metadata lookups (`matches.rs`), `React.memo` timeline turn blocks + `CardImage` state guard (`MatchTimeline.tsx`, `RoundTurnGroup.tsx`, `CardImage.tsx`), and `scripts/install-linux.sh`.
- [ ] **Full Program & Settings Inventory (`docs/PROGRAM_INVENTORY.md`):**
  - Generate a complete reference audit of all current UI tabs, Bento widgets, SQL queries, achievements, leaderboards, settings keys, and `Player.log` parsers.

---

### Phase 7: Limited / Draft Analytics & Economy Grind Calculator (Client + Server)
- [ ] **`Player.log` Event & Draft Course Parser (`parser.rs` & `match_assembler.rs`):**
  - Parse `Course` / `Courses` blocks (`CourseId`, `InternalEventName`, `WinLossGate`, `CurrentWins`, `CurrentLosses`, `MaxWins`, `MaxLosses`, `ProcessedMatchIds`, `CourseDeck`).
  - Automatically extract **Event Type** (`PremierDraft`, `QuickDraft`, `TradDraft`, `Sealed`, `Constructed_Event`) and **3-Letter Set Code** (e.g., `FRA`, `FDN`, `DSK`).
- [ ] **Trophy & Run Tracking (`event_runs` Table + Historical Backfill):**
  - Flag runs reaching `CurrentWins >= MaxWins` (`7-x` Bo1 or `3-0` Bo3) as **Trophies (`is_trophy = true`)**.
  - Reconstruct historical draft runs and trophies on startup from existing Limited matches in `rhystic.db`.
  - Display live event record badges (`5-1` / `7-2 Trophy`) in the Live HUD and Deck Overlay.
- [ ] **Draft Economy Sustainability & "Days to Grind" Calculator:**
  - Calculate net Gem/Gold cost per draft by comparing event entry fees (`1,500 Gems` / `10,000 Gold` Premier; `750 Gems` / `5,000 Gold` Quick) against MTGA's exact win-payout table for the player's average win rate per set/type.
  - Combine with the player's logged daily Gold income (Daily Quests + daily win rewards, or a `1,150 Gold/day` baseline) to display **"Days of Grinding per Draft"** when net Gem/Gold delta is negative.

---

### Phase 8: Achievements, Personal Records & Community Leaderboards Rethink
- [ ] **Client vs. Server Scope Separation:**
  - **Local Client ("Personal Hall of Fame"):** Reframe local card leaderboards into personal milestones (Personal Best Single-Hit Damage, Most Reliable Finisher, Format Win-Streak Records, and Draft Trophy Cabinet) that are meaningful for a single offline player.
  - **Community Server ("Global Meta & Player Leaderboards"):** Host multi-player analytics that require community sample sizes—Global Archetype Tier Lists, Core/Flex Win-Rate Deltas, Limited Card GIH WR / IWD, Global Salt Index, Player Win-Rate/Trophy Leaderboards, and Global Achievement Rarity percentages.
- [ ] **Relational Best-of-3 Per-Game Tracking (`match_games`):**
  - Split Bo3 series into per-game rows (`game_number`, `on_the_play`, `turns`, `sideboard_in_json`, `sideboard_out_json`) so turn-based achievements (`Gotta Go Fast`, `Marathon Master`) and Play/Draw win rates are evaluated per game rather than summed across a 3-game series.

---

### Phase 9: In-App Updater, Standalone Docker Server & Multi-OS Releases
- [ ] **In-App Update Checker & One-Click Update Button:**
  - Add a version checker in the sidebar/Settings that queries GitHub Releases for newer `vX.Y.Z+web.N` builds, displays release notes, and triggers Tauri's updater or opens the release asset.
- [ ] **Standalone Dockerized Server (`Dockerfile` + `docker-compose.yml`):**
  - Package `server/` (`rhystic-cloud-server`) into a self-contained Docker container with persistent SQLite volume mounting, health checks, and a 1-command `docker compose up -d` setup for self-hosters.
- [ ] **Automated `+web` Release Pipeline (`.github/workflows/release.yml`):**
  - Build Linux, Windows, macOS, and standalone `rhystic-cloud-server` binaries automatically whenever a `v*+web.*` tag is pushed.