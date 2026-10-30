# Rhystic Tracker & Cloud: Master Program Inventory & Architecture Reference

> **Document Status:** Complete & Factual Master Architecture Reference  
> **Applies to:** `Rhystic-Tracker-Web` (`main`) & Upstream Core Engine (`Balthazzahr/Rhystic-Tracker`)  
> **Date:** October 2026

This document is the definitive, code-verified inventory of all desktop UI views, modals, dashboard widgets, settings, log parsers, state machines, achievements, leaderboards, and SQLite schemas across `src/`, `src-tauri/`, and `server/`.

---

## Table of Contents
1. [Desktop UI Views & Components (`src/components/`)](#1-desktop-ui-views--components)
   - [Sidebar Navigation Views](#11-sidebar-navigation-views)
   - [Modals & Wizards](#12-modals--wizards)
   - [Overlay Windows & Floating Panels](#13-overlay-windows--floating-panels)
   - [Bento Dashboard 2.0 Widgets](#14-bento-dashboard-20-widgets)
2. [Settings Inventory](#2-settings-inventory)
   - [Backend File Settings (`settings.json`)](#21-backend-file-settings-settingsjson)
   - [Cloud Sync Settings (`sync_settings.json`)](#22-cloud-sync-settings-sync_settingsjson)
   - [Frontend LocalStorage Settings](#23-frontend-localstorage-settings)
   - [Database-Backed Settings](#24-database-backed-settings)
3. [`Player.log` Parser & State Machine Inventory](#3-playerlog-parser--state-machine-inventory)
   - [Log Tailer & Path Discovery (`tailer.rs`)](#31-log-tailer--path-discovery)
   - [Parser Event Registry (`parser.rs`)](#32-parser-event-registry)
   - [Match Assembler State Machine (`match_assembler.rs`)](#33-match-assembler-state-machine)
   - [Unparsed & Ignored MTGA Log Events](#34-unparsed--ignored-mtga-log-events)
4. [Achievements & Leaderboards Inventory](#4-achievements--leaderboards-inventory)
   - [Card Trophies Registry (26 Badges)](#41-card-trophies-registry-26-badges)
   - [Deck Honors Registry (6 Badges)](#42-deck-honors-registry-6-badges)
   - [Local Hall of Fame Queries (`analytics.rs`)](#43-local-hall-of-fame-queries)
   - [Global Server Leaderboards & Handle Claiming (`server/`)](#44-global-server-leaderboards--handle-claiming)
5. [SQLite Database Schemas](#5-sqlite-database-schemas)
   - [Client Database Schema (`rhystic.db`)](#51-client-database-schema-rhysticdb)
   - [Server Database Schema (`rhystic_cloud.db`)](#52-server-database-schema-rhystic_clouddb)
6. [Client vs. Server Feature Matrix](#6-client-vs-server-feature-matrix)

---

## 1. Desktop UI Views & Components

### 1.1 Sidebar Navigation Views

The desktop shell (`src/App.tsx`) hosts 9 core views in the primary workspace:

| View ID | Primary Component File | Displayed Information | Tauri IPC Commands Invoked |
| :--- | :--- | :--- | :--- |
| `dashboard` | `DashboardView.tsx` (1.0 Classic)<br>`dashboard/Dashboard2View.tsx` (2.0 Bento) | Win rates (overall, today, rolling trend), current streak, format distribution, recent matches, active deck spotlight, telemetry fun facts, recent trophies, economy & quest widgets. | `get_recent_matches`<br>`get_deck_overview`<br>`get_matches_count`<br>`get_dashboard_layout`<br>`save_dashboard_layout`<br>`get_player_economy`<br>`get_active_quests`<br>`get_reward_tracks_status`<br>`get_player_rank_status`<br>`get_mastery_pass_status`<br>`get_global_achievements`<br>`get_global_leaderboards` |
| `matches` | `MatchHistoryView.tsx` | Virtualized, searchable match list. Format, Hero Deck, Opponent Deck/Commander, Result, Turns, Duration, Mulligans, Final Life margin, Action buttons (Full Info, H2H, Delete match). | `get_recent_matches`<br>`get_match_cards`<br>`delete_match`<br>`get_opponent_h2h_stats` |
| `collection` | `CollectionView.tsx` | Draw-discovered cards and imported collection. Grid/List view, owned card quantities (1–4x playset), mana cost filters, card types, set symbols/rarity, Scryfall art previews. | `get_collection`<br>`get_set_metadata`<br>`refresh_set_metadata`<br>`update_collection_card_count`<br>`sync_card_database`<br>`get_raw_card_db_status`<br>`set_raw_path` |
| `decks` | `DeckLibraryView.tsx`<br>`DeckDetailView.tsx` | Deck Library view displays deck cards with 3D visual flair, win rates, commander badges, deck honors. Selecting a deck opens `DeckDetailView` showing full 60/100 card list, mainboard/sideboard, mana curve, custom art overrides, deck achievements, and match history. | `get_deck_overview`<br>`get_deck_detail`<br>`get_deck_cards`<br>`save_deck_list`<br>`delete_deck`<br>`export_decklist`<br>`set_deck_custom_art`<br>`reset_deck_custom_art`<br>`set_deck_custom_bg_art`<br>`reset_deck_custom_bg_art`<br>`get_deck_owned_stats`<br>`get_deck_achievements`<br>`get_all_deck_achievements` |
| `achievements`| `AchievementsView.tsx` | Card Trophy case (26 badge types across 6 tiers) and Deck Honors (6 badge types across 3 tiers). Total honor count, category filters, earned vs. unearned toggle, modal details. | `get_global_achievements`<br>`get_all_deck_achievements` |
| `leaderboards`| `LeaderboardsView.tsx` | Personal Hall of Fame across 9 categories (Combat Single Hit, Match Damage, Total Damage; Non-Combat Single Hit, Match Damage, Total Damage; Most Decorated, Draw Engines, Battlefield Stalwarts). | `get_global_leaderboards`<br>`get_recent_matches` |
| `live` | `LiveHUDView.tsx` | Real-time live match state (`IN_MATCH`, `COMPLETED`, `IDLE`). Hero & Opponent life totals, turn counter, cards played per turn, card combat log, active commander cards, Bo3 sideboard segregated drawer. | `get_live_match_state`<br>`toggle_deck_overlay`<br>`set_deck_overlay_bounds`<br>`open_deck_tracker_window`<br>`close_deck_tracker_window` |
| `live-deck` | `LiveDeckTrackerView.tsx`<br>`deck_tracker/LiveDeckTrackerPanel.tsx` | Detached or embedded live deck tracker. Starting library vs. cards remaining, cards in hand/board/graveyard/exile, dynamic hypergeometric draw odds for lands and specific cards. | `get_live_match_state`<br>`open_deck_tracker_window` |
| `settings` | `SettingsView.tsx` | Master configuration console: General (behavior & desktop lifecycle), Appearance (themes, mana pips, opacity), Connection (log path & cloud sync), Storage (DB maintenance, image cache, Scryfall sets), About. | `get_log_path`<br>`set_log_path`<br>`get_minimize_to_tray`<br>`set_minimize_to_tray`<br>`get_cache_stats`<br>`clear_image_cache`<br>`get_avatar_cache_stats`<br>`clear_avatar_cache`<br>`get_database_stats`<br>`export_database_backup`<br>`sync_card_database`<br>`refresh_set_metadata`<br>`get_sync_settings_status`<br>`update_sync_settings`<br>`enqueue_historical_sync`<br>`flush_sync_outbox`<br>`check_for_app_update`<br>`install_app_update`<br>`restart_app` |

---

### 1.2 Modals & Wizards

| Modal Component | File Location | Purpose & Trigger | Tauri IPC Commands |
| :--- | :--- | :--- | :--- |
| `FullMatchInfoModal` | `src/components/FullMatchInfoModal.tsx` | Displays deep match details: full turn timeline, damage breakdown, card lists for both players, impactful card awards. | `get_match_cards`<br>`get_match_turn_events`<br>`get_impactful_cards`<br>`get_card_info` |
| `OpponentH2HModal` | `src/components/OpponentH2HModal.tsx` | Shows lifetime head-to-head match history, win rate, and commander encounters against a specific opponent. | `get_opponent_h2h_stats`<br>`get_opponent_matches` |
| `CardInspectorModal`| `src/components/CardInspectorModal.tsx` | High-resolution Scryfall card inspector, alternative printings selector, oracle text, ruling, and pricing. | `get_card_printings`<br>`set_preferred_print`<br>`clear_preferred_print` |
| `CardTrophyCaseModal`| `src/components/CardTrophyCaseModal.tsx`| Displays all lifetime trophies and titles awarded to a specific card across all games. | `get_impactful_cards` |
| `AchievementDetailModal` | `src/components/AchievementDetailModal.tsx` | Shows criteria, tier progression, flavor text, and all cards that earned a specific Card Trophy. | None (receives metadata & achievement entries via props) |
| `DeckAchievementDetailModal` | `src/components/DeckAchievementDetailModal.tsx` | Shows criteria, tier progression, flavor text, and matches where a specific Deck Honor was earned. | None (receives metadata via props) |
| `DeckAchievementsModal` | `src/components/DeckAchievementsModal.tsx` | Overview of all 6 deck honors earned by a specific deck. | `get_deck_achievements` |
| `ColumnCustomizerModal` | `src/components/ColumnCustomizerModal.tsx` | Reorders and toggles visibility of columns in the Match History table. | None (persists to `localStorage`) |
| `FirstTimeSetupWizard` | `src/components/FirstTimeSetupWizard.tsx` | Initial onboarding wizard to auto-detect MTGA `Player.log`, verify detailed logging, and sync Scryfall universe. | `get_setup_status`<br>`complete_setup`<br>`set_log_path`<br>`sync_card_database` |
| `AvatarOnboardingModal`| `src/components/AvatarOnboardingModal.tsx` | One-time prompt introducing the MTGA client avatar asset extraction feature. | None (sets `localStorage` dismissed flag) |
| `NewSetReminderModal` | `src/components/NewSetReminderModal.tsx` | Prompts player to update the card database when a newly released MTGA set is detected. | `sync_card_database` |
| `NewAvatarReminderModal`| `src/components/NewAvatarReminderModal.tsx`| Prompts player when new avatar assets are detected in the MTGA local game installation bundle. | `extract_avatars_from_mtga_client` |
| `WhatsNewModal` | `src/components/WhatsNewModal.tsx` | Highlights new release features (e.g., Live Deck Tracker, cloud sync outbox). | None (sets `localStorage` seen flag) |
| `DashboardColorPickerModal` | `src/components/dashboard/DashboardColorPickerModal.tsx` | Customizes individual Bento widget accent colors. | None (persists to `localStorage`) |
| `MasteryLevelModal` | `src/components/dashboard/widgets/MasteryLevelModal.tsx` | Displays mastery pass reward track progression, level XP, orbs, and premium rewards. | `get_mastery_pass_status` |
| `QuestStatisticsModal` | `src/components/dashboard/widgets/QuestStatisticsModal.tsx` | Lifetime quest completion statistics, gold/XP earned, completion speeds, and swap upgrade rates. | `get_quest_statistics` |
| `RankHistoryModal` | `src/components/dashboard/widgets/RankHistoryModal.tsx` | Historical season-by-season ladder ranking progression (Constructed & Limited). | `get_player_rank_history` |
| `WildcardHistoryModal` | `src/components/dashboard/widgets/WildcardHistoryModal.tsx` | History of wildcard acquisition, crafting usage, vault progress, and pack opening deltas. | `get_economy_history` |
| `LeaderboardDetailModal` | `src/components/leaderboards/LeaderboardDetailModal.tsx` | Deep view of a specific Hall of Fame leaderboard category with full card rankings. | None (receives category list via props) |

---

### 1.3 Overlay Windows & Floating Panels

| Overlay / Panel Component | File Location | Window / Render Context | Purpose & Features |
| :--- | :--- | :--- | :--- |
| `LiveDeckOverlayWindow` | `src/components/LiveDeckOverlayWindow.tsx` | Standalone secondary Tauri window (`deck-tracker-overlay`) | Detached floating live deck tracker window with transparency, borderless drag bar, minimize/close controls, and always-on-top pinning. |
| `DeckTrackerOverlayView` | `src/components/deck_tracker/DeckTrackerOverlayView.tsx` | Root component inside `LiveDeckOverlayWindow` | Automatically tracks screen position coordinates, restores window geometry on startup, and renders `LiveDeckTrackerPanel`. |
| `LiveDeckTrackerPanel` | `src/components/deck_tracker/LiveDeckTrackerPanel.tsx` | Rendered both in `DeckTrackerOverlayView` and `LiveDeckTrackerView` | Displays remaining cards in library with exact draw odds, cards drawn/outside library (hand, battlefield, graveyard, exile), and sorting modes (mana cost, card name, draw probability). |
| `MemoryStatsPanel` | `src/components/MemoryStatsPanel.tsx` | Test/Dev overlay in main window (`App.tsx`) | Displays real-time memory usage (RSS) of the main process and WebKit subprocesses on Linux. |

---

### 1.4 Bento Dashboard 2.0 Widgets

The 28 customizable widgets registered in `src/components/dashboard/widgetRegistry.tsx`:

| Widget ID (`kind`) | Title | Subtitle | Data Displayed | Direct IPC Commands |
| :--- | :--- | :--- | :--- | :--- |
| `win_rate_summary` | All-Time Win Rate | Lifetime record | Overall win percentage, total matches, total wins, and total losses. | None (reads `matches` prop) |
| `today` | Today Win Rate | Current session | Matches played today, today's win percentage, and session win/loss delta. | None (reads `matches` prop) |
| `current_streak` | Current Streak | Active streak | Current consecutive win or loss streak with fire/ice styling. | None (reads `matches` prop) |
| `win_rate_trend` | Trending Win Rate | Rolling performance | 10-match and 30-match rolling win-rate trendline chart. | None (reads `matches` prop) |
| `deck_spotlight` | Deck Spotlight | Featured deck | Most played active deck, win rate, games played, and hero commander art. | None (reads `deckOverview` prop) |
| `recent_matches` | Recent Matches | Latest activity | Compact list of last 5 matches with format, deck, result, and time ago. | None (reads `matches` prop) |
| `format_distribution` | Format Distribution | Format share | Donut chart of match distribution across Commander, Brawl, Standard, Draft, etc. | None (reads `matches` prop) |
| `fun_facts` | Fun Facts | Play telemetry | Longest match, highest turn count, shortest game, highest health margin. | `get_match_cards` |
| `recent_achievements` | Recent Achievements | Latest trophies | Last 3 Card Trophies unlocked with card art and tier badge. | `get_global_achievements` |
| `featured_leaderboard` | Featured Leaderboard | Hall of fame | Rotating podium card highlighting top local record holder in combat damage or spell impact. | `get_global_leaderboards` |
| `frequent_opponents` | Frequent Opponents | Most faced rivals | Top 5 opponents encountered most frequently with head-to-head records. | None (reads `matches` prop) |
| `play_vs_draw` | Play vs. Draw | Turn advantage | Win rate comparison when going first (on the play) vs. going second (on the draw). | None (reads `matches` prop) |
| `victory_breakdown` | Victory Breakdown | Win conditions | Breakdown of wins/losses by opponent concession, lethal damage, or deck-out. | None (reads `matches` prop) |
| `life_closeness` | End-Game Life Totals | Health margins | Average ending life total and distribution of razor-thin victories (≤ 5 life). | None (reads `matches` prop) |
| `activity_heatmap` | Activity by Day | Volume & trends | 7-day activity bar chart with daily match counts and win rates. | None (reads `matches` prop) |
| `lucky_cursed_cards` | Lucky & Cursed Cards | Card impact | Cards with highest positive win-rate delta when drawn vs. lowest win-rate delta. | `get_card_win_correlations` |
| `guild_mastery` | Guild & Clan Mastery | Color pairs | Win rates across two-color guild and three-color clan identity archetypes. | None (reads `matches` prop) |
| `mulligan_meter` | Mulligan Resilience | Hand win drop-off | Win rates based on opening hand size (keep 7 vs. mulligan to 6, 5, or 4). | None (reads `matches` prop) |
| `archetype_clash` | Archetype Pace Matrix | Aggro/Mid/Control | Win rates categorized by game pace (Aggro: ≤5 turns, Midrange: 6–9 turns, Control: 10+ turns). | None (reads `matches` prop) |
| `currencies_vault` | Currencies & Inventory | Player assets | Current Gold, Gems, Vault progress %, Draft Tokens, and Jump In Tokens. | `get_player_economy` |
| `wildcards_golden_pack` | Wildcards & Vault | Crafting assets | Common, Uncommon, Rare, and Mythic Wildcards count, plus Golden Pack progress (0/10). | `get_player_economy` |
| `economy_overview` | Economy Overview | Complete treasury | Full player treasury combining currencies, wildcards, and booster inventory. | `get_player_economy` |
| `economy_trends` | Economy Trends | Balance history | Historical line chart tracking Gold, Gems, and Vault progress over time. | `get_economy_history` |
| `active_quests` | Active Quests | Daily objectives | Up to 3 active daily quests with progress bars, rewards (500/750g), and reroll status. | `get_active_quests` |
| `reward_tracks` | Win Reward Tracks | Daily & weekly wins | Daily win track (up to 15 wins) and weekly win track (up to 15 wins) with reset countdowns. | `get_reward_tracks_status` |
| `quests_rewards_overview`| Quests & Win Tracks | Unified progression | Combined panel showing both daily quests and daily/weekly win track progress. | `get_active_quests`<br>`get_reward_tracks_status` |
| `ranked_ladder` | Ranked Ladder | Seasonal rank | Constructed and Limited seasonal rank tiers (Bronze to Mythic) with step pips and season end timer. | `get_player_rank_status` |
| `mastery_pass` | Mastery Pass | Set pass progress | Current mastery pass level, XP toward next level, available orbs, and pass track name. | `get_mastery_pass_status` |

---

## 2. Settings Inventory

Settings in Rhystic Tracker are partitioned across three tiers: OS-level JSON files in `~/.config/rhystic-tracker/`, frontend `localStorage`, and SQLite tables.

### 2.1 Backend File Settings (`settings.json`)

Managed by `src-tauri/src/settings.rs` in `~/.config/rhystic-tracker/settings.json`:

| Setting Key | Rust Type | Default Value | Purpose & Effect |
| :--- | :--- | :--- | :--- |
| `mtga_log_path` | `Option<String>` | `None` (auto-detect) | Absolute path to MTGA `Player.log`. If set, overrides auto-discovery. |
| `mtga_raw_dir` | `Option<String>` | `None` (auto-detect) | Absolute directory containing raw MTGA client databases (`Raw_CardDatabase_*.bytes`). |
| `minimize_to_tray` | `bool` | `true` | If true, closing the main window hides it to the system tray rather than exiting. |
| `setup_completed` | `bool` | `false` | Tracks whether the initial First Time Setup Wizard was completed. |
| `last_avatar_sync_bundle_mtime` | `u64` | `0` | Modification timestamp of MTGA asset bundle when avatars were last extracted. |
| `deck_tracker_position` | `Option<(f64, f64)>`| `None` | Persisted `(x, y)` screen coordinates for the floating Live Deck Tracker popup. |

---

### 2.2 Cloud Sync Settings (`sync_settings.json`)

Managed by `src-tauri/src/sync.rs` in `~/.config/rhystic-tracker/sync_settings.json`:

| Setting Key | Rust Type | Default Value | Privacy Tier & Effect |
| :--- | :--- | :--- | :--- |
| `enabled` | `bool` | `false` | **Strictly Opt-In.** Master switch enabling community match synchronization. |
| `contributor_id` | `String` | Random UUIDv4 | Anonymous local client identifier. Never linked to real identity. |
| `server_url` | `String` | `"https://api.rhystic.gg"` | Ingestion server endpoint (`POST /v1/ingest` or `/api/v1/matches/batch`). |
| `include_deck_cards` | `bool` | `true` | **Tier 1:** Includes seen card `grp_id`s, damage dealt, and draw counts in sync payload. |
| `share_full_decklist` | `bool` | `true` | **Tier 2:** Includes submitted 60/100 card list to power archetype core/flex clustering. |
| `share_player_identity`| `bool` | `false` | **Tier 3:** Includes log-detected Arena handle & seasonal rank for public leaderboards. |
| `verified_arena_handle`| `Option<String>` | `None` (auto-read) | Read-only Arena screen name detected from MTGA log `AuthenticateResponse`. |
| `arena_account_hash` | `Option<String>` | `None` (computed) | SHA-256 hash of `client_id + screen_name` ensuring handles cannot be spoofed. |
| `sync_key` | `Option<String>` | `None` | Optional token for future cross-device synchronization (e.g., iPad companion). |

---

### 2.3 Frontend LocalStorage Settings

Stored in the browser/WebKit local storage (`~/.local/share/com.rhystic.tracker/`):

| LocalStorage Key | Type | Default | Controlled In / Effect |
| :--- | :--- | :--- | :--- |
| `activeTab` | `string` | `'dashboard'` | Remembers last selected sidebar navigation tab. |
| `defaultStartupTab` | `string` | `'dashboard'` | Settings → General: Tab automatically displayed upon app launch. |
| `sidebarCollapsed` | `'true' \| 'false'` | `'false'` | Sidebar collapse toggle state. |
| `sidebarIcons` | `JSON string` | `{}` | Custom MTG mana pip icons selected for sidebar tabs. |
| `activeThemeId` | `string` | `'mana_blue'` | Active mana theme (`mana_white`, `mana_blue`, `mana_black`, `mana_red`, `mana_green`, `mana_gold`, `mana_colorless`). |
| `glassOpacity` | `string` | `'standard'` | Settings → Appearance: Glassmorphism transparency level (`crisp`, `standard`, `obsidian`). |
| `manaPipStyle` | `string` | `'graphic'` | Settings → Appearance: Mana pip rendering mode (`graphic` SVG vs. `font` vector glyphs). |
| `autoSwitchLiveView` | `'off' \| 'live' \| 'live-deck'` | `'live'` | Settings → General: View auto-navigated to when a match starts. |
| `liveHudAlwaysOnTop` | `'true' \| 'false'` | `'false'` | Settings → General: Keeps main window pinned above MTG Arena. |
| `autoOpenDeckTracker` | `'true' \| 'false'` | `'false'` | Settings → General: Automatically opens the detached live deck tracker window on match start. |
| `rememberDeckTrackerPosition` | `'true' \| 'false'` | `'true'` | Settings → General: Restores screen coordinates for the deck tracker window. |
| `excludeSparkyMatches` | `'true' \| 'false'` | `'true'` | Settings → General: Filters out Sparky practice challenges and bot matches from statistics. |
| `enableAudioCues` | `'true' \| 'false'` | `'true'` | Settings → General: Plays audio notifications on match start, win, loss, and achievement unlock. |
| `confirmDeckDelete` | `'true' \| 'false'` | `'true'` | Settings → General: Prompts for confirmation before deleting a deck from Deck Library. |
| `allowMatchDeletion` | `'true' \| 'false'` | `'false'` | Settings → General: Enables delete trash icon in Match History table. |
| `autoExportMatches` | `'true' \| 'false'` | `'false'` | Settings → General: Automatically dumps match records to disk upon completion. |
| `autoExportFormat` | `'csv' \| 'json'` | `'json'` | Settings → General: Export file format for completed matches. |
| `persistDashboardPage` | `'true' \| 'false'` | `'true'` | Settings → General: Remembers active Bento Dashboard page across restarts. |
| `rhystic_dashboard_mode`| `'1.0' \| '2.0'` | `'2.0'` | Toggles between classic 1.0 dashboard and 2.0 Bento Grid dashboard. |
| `rhystic_dashboard_layout`| `JSON string` | Presets | Stores custom Bento Grid layout, active widgets, and dimensions. |
| `rhystic_dashboard_colors`| `JSON string` | `{}` | Stores custom accent colors per Bento widget. |
| `autoCheckUpdates` | `'true' \| 'false'` | `'true'` | Settings → General: Automatically checks GitHub releases for app updates on startup. |
| `bo3SideboardTracking` | `'true' \| 'false'` | `'true'` | Settings → Connection: Segregates sideboarded cards in Bo3 match games 2 & 3. |
| `autoBackupEnabled` | `'true' \| 'false'` | `'true'` | Settings → Storage: Prompts for database backup intervals. |
| `imageCacheQuota` | `string` | `'1gb'` | Settings → Storage: Disk quota cap for cached Scryfall card illustrations. |
| `deckBoxFlair` | `'true' \| 'false'` | `'true'` | Settings → Appearance: Renders 3D perspective box flair and holographic commander foil. |
| `compactCardsMode` | `'true' \| 'false'` | `'false'` | Settings → Appearance: Renders compact dense rows instead of visual card tiles. |
| `defaultCollectionSort`| `string` | `'name'` | Settings → Appearance: Default sorting criteria for Card Library (`name`, `cmc`, `rarity`, `set`). |
| `collectionOwnedFilter`| `string` | `'all'` | Card Library view filter (`all`, `owned`, `unowned`). |
| `collectionView` | `'grid' \| 'table'` | `'grid'` | Card Library layout mode. |
| `collectionCardSize` | `'sm' \| 'md' \| 'lg'` | `'md'` | Card Library card preview thumbnail size. |
| `rhystic_deck_sort` | `string` | `'games'` | Deck Library sorting criteria (`games`, `winrate`, `name`, `updated`). |
| `rhystic_deck_view` | `'grid' \| 'table'` | `'grid'` | Deck Library display mode. |
| `rhystic_achievements_view`| `'all' \| 'card' \| 'deck'` | `'all'` | Achievements view active tab filter. |
| `rhystic_achievements_show_unearned` | `'true' \| 'false'` | `'true'` | Achievements view toggle to display locked badges. |

---

### 2.4 Database-Backed Settings

Stored inside tables in `rhystic.db`:
- `dashboard_layouts`: Layout schema version and JSON widget positions.
- `card_preferred_prints`: User's preferred card printing (art, set, collector number) per card name.
- `deck_art_overrides`: Custom hero commander card art chosen for a specific deck.
- `deck_bg_art_overrides`: Custom background wallpaper card art chosen for a specific deck.

---

## 3. `Player.log` Parser & State Machine Inventory

### 3.1 Log Tailer & Path Discovery (`tailer.rs`)

The tailer runs as an asynchronous background worker (`tokio::task`) monitoring MTGA's active log:

1. **Path Discovery Hierarchy:**
   - Explicit environment variable: `RHYSTIC_MTGA_LOG`
   - Stored user override: `mtga_log_path` in `settings.json`
   - Platform automatic discovery:
     - **Linux (Proton / Steam):** Scans `~/.local/share/Steam`, `~/.steam/steam`, Flatpak Steam paths (`~/.var/app/com.valvesoftware.Steam`), Lutris Wine prefixes (`Games/magic-the-gathering-arena`), Bottles, and Heroic prefixes for compatdata AppID `2141910` (`AppData/LocalLow/Wizards Of The Coast/MTGA/Player.log`).
     - **Windows:** `%APPDATA%\..\LocalLow\Wizards Of The Coast\MTGA\Player.log`.
     - **macOS:** `~/Library/Logs/Wizards Of The Coast/MTGA/Player.log`.
2. **Streaming & Rotation Lifecycle:**
   - Reads lines incrementally via `BufReader`.
   - Checks file size and inode every 250ms.
   - Detects file rotation (MTGA relaunch) or truncation and resets file pointer.
   - Emits `TailerEvent::Line(String)`, `TailerEvent::Rotated`, and `TailerEvent::InitialCatchupComplete`.

---

### 3.2 Parser Event Registry (`parser.rs`)

The parser inspects raw log lines using substring matchers and parses JSON payloads into `ParsedEvent`:

| Log Event Signature / Trigger | Extracted Fields & Data | Resulting `ParsedEvent` |
| :--- | :--- | :--- |
| `authenticateResponse` | Player `screenName` (e.g. `Hero#12345`), `clientId` | `ParsedEvent::Auth` |
| `matchGameRoomStateChangedEvent` / `Connecting to matchId` | `matchId`, `gameRoomConfig.reservedPlayers`, `gameRoomConfig.matchId`, format name, `assignedDeckEvent` | `ParsedEvent::MatchCreated` |
| `EventGetCourses` / `CourseDeckSummary` | Active event course decks: `DeckId`, deck name, `MainDeck` (card IDs & quantities), `CommandZone` | `ParsedEvent::DeckCatalogBatch` |
| `EventSetDeck` / `DeckUpsertDeck` / `deckSubmit` | Submitted deck name, card quantities, commander `grp_id`, deck ID | `ParsedEvent::DeckSubmitted` |
| `GREMessageType_ConnectResp` | Authoritative starting deck card list (`deckCards`), commanders (`commanderCards`), local player `systemSeatId` | `ParsedEvent::ConnectResp` |
| `GREMessageType_GameStateMessage` / `GREMessageType_PromptReq` | Complete game turn steps: turn number, active phase, zone transfers, game objects, instance IDs, card damage events, life totals | `ParsedEvent::GameStateUpdates` |
| `ClientMessageType_MulliganResp` | `seatId`, `mulliganCount`, whether hand was kept or mulliganed | `ParsedEvent::MulliganEvent` |
| `matchGameRoomStateChangedEvent` (`MatchCompleted`) | Winning team ID, match result reason (`Concede`, `Lethal`, etc.), duration | `ParsedEvent::MatchCompleted` |
| `InventoryInfo` | Player currency balances: Gold, Gems, Vault progress, Wildcards (Common, Uncommon, Rare, Mythic), Draft Tokens, Jump-In Tokens, Boosters | `ParsedEvent::InventoryUpdate` |
| `OpenBooster` / `openBooster` / `CardsAdded` | Pack opening records: `CollationId`, set code, added card IDs, wildcard track advances, vault progression delta | `ParsedEvent::BoosterOpened` |
| `"quests"` / `QuestGetQuests` | Active quests list: `questId`, title, description, category, colors, goal, current progress, rewards (Gold, XP), can-swap flag | `ParsedEvent::QuestUpdate` |
| `constructedSeasonOrdinal` / `RankGetCombinedRankInfo` | Ranked ladder progression: season ordinal, Constructed tier/step/wins/losses, Limited tier/step/wins/losses | `ParsedEvent::RankUpdate` & `SeasonUpdate` |
| `LevelTrack_Level_` / `BattlePass_` | Mastery pass tracking: current level, current XP, premium track unlocked, available mastery orbs | `ParsedEvent::MasteryPassUpdate` |

---

### 3.3 Match Assembler State Machine (`match_assembler.rs`)

`MatchAssembler` maintains the authoritative live match state in memory:

```
                  +--------------------------------+
                  |              IDLE              |
                  +--------------------------------+
                                  |
                                  | `MatchCreated`
                                  v
                  +--------------------------------+
                  |         MATCH_STARTING         |
                  | (Hero seat unresolved,         |
                  |  format normalized)            |
                  +--------------------------------+
                                  |
                                  | `ConnectResp`
                                  v
                  +--------------------------------+
                  |            IN_MATCH            |
                  | - Hero seat ID bound           |
                  | - Starting deck locked         |
                  | - Opening hands tracked        |
                  +--------------------------------+
                                  |
                                  | `GameStateUpdates` (Turn loop)
                                  | -> Zone transfers (Library -> Hand)
                                  | -> Combat & Spell damage accumulators
                                  | -> Impactful Card metrics computed
                                  v
                  +--------------------------------+
                  |        MATCH_COMPLETED         |
                  | - Winner resolved              |
                  | - Card Trophies calculated     |
                  | - Deck Honors evaluated        |
                  | - Committed to SQLite          |
                  | - Queued to Sync Outbox        |
                  +--------------------------------+
```

1. **Card Instance Tracking:** Maps MTGA ephemeral `instanceId` to canonical card `grp_id`, `ownerSeatId`, `controllerSeatId`, and `currentZoneId`.
2. **Collection Draw Feeding:** Every card moving from the player's library to their hand in a legitimate match is added to `collection_draws`, incrementally unlocking cards in `collection_cards` up to playset cap (4x).
3. **Telemetry & Combat Accumulation:** Tracks per-card combat damage, spell damage, cards drawn, permanents eliminated, counters added, life drained, and spells copied.

---

### 3.4 Unparsed & Ignored MTGA Log Events

The following event types appear in `Player.log` but are currently **ignored / unparsed** by the application:

1. **Draft Pick Telemetry:**
   - `Draft.MakePick`, `Event.Draft`, `DraftPack`, and `BotDraftPack` events are unparsed. Draft card choices, pick orders, pack contents, and signals are not recorded. Only final submitted 40-card decks are captured via `ConnectResp`.
2. **Event Run Progression:**
   - `Event.Join`, match wins/losses inside an event run (e.g. 7-win Traditional Draft run, 3-loss elimination), drop events, and event reward claiming are unparsed.
3. **Direct Challenge & Social:**
   - Friend challenges, matchmaking lobbies, chat messages, and social requests.
4. **Cosmetics & Customization:**
   - Sleeve choices, companion selection, and emote usage.
5. **Store Transactions:**
   - Gold/Gem purchases, bundle buys, and cosmetic unlocks (only pack opening `CardsAdded` is tracked).
6. **Pre-Game Sideboarding:**
   - Cards swapped in/out during tournament sideboarding phases before Game 2/3 (currently only cards actually cast/seen in Game 2/3 are recorded).

---

## 4. Achievements & Leaderboards Inventory

### 4.1 Card Trophies Registry (26 Badges)

Evaluated per card instance during match assembly in `src-tauri/src/match_assembler.rs` and registered in `src/utils/achievementBadges.ts`:

| # | Achievement ID | Title | Category | Trigger / Evaluation Rule | Thresholds (Bronze / Silver / Gold / Plat / Leg) |
| :- | :--- | :--- | :--- | :--- | :--- |
| 1 | `scoop_inducer` | Scoop Inducer | Closer | Opponent concedes after player casts high-CMC card while opponent has high health. | Br: ≤T7, ≥20hp \| Sil: ≤T6, ≥25hp \| Gld: ≤T5, ≥25hp \| Plt: ≤T4, ≥26hp \| Leg: ≤T3, ≥30hp |
| 2 | `executioner` | Executioner | Closer | Deals the killing blow reducing opponent's life to ≤ 0. | Br: hit on opp ≤7hp \| Sil: ≤9hp \| Gld: ≤14hp \| Plt: ≤20hp \| Leg: ≤25hp |
| 3 | `over_killer` | Over-Killer | Combat | Excess lethal damage dealt beyond opponent's remaining life. | Br: ≥10 overkill \| Sil: ≥15 \| Gld: ≥25 \| Plt: ≥40 \| Leg: ≥60 |
| 4 | `haymaker` | Haymaker | Combat | Highest single-hit combat damage dealt to an opponent. | Br: ≥10 dmg \| Sil: ≥15 \| Gld: ≥25 \| Plt: ≥40 \| Leg: ≥60 |
| 5 | `juggernaut` | Juggernaut | Combat | Total cumulative combat damage dealt by one card in a match. | Br: ≥20 total \| Sil: ≥35 \| Gld: ≥50 \| Plt: ≥80 \| Leg: ≥120 |
| 6 | `hardened` | Hardened | Combat | Maximum toughness achieved by a creature through counters or pumps. | Br: ≥10 toughness \| Sil: ≥15 \| Gld: ≥25 \| Plt: ≥40 \| Leg: ≥60 |
| 7 | `ozolithic` | Ozolithic! | Synergy | Total counters added to a permanent across the match. | Br: ≥6 counters \| Sil: ≥10 \| Gld: ≥18 \| Plt: ≥30 \| Leg: ≥50 |
| 8 | `vampiric` | Vampiric | Life | Cumulative life drained or gained from damage dealt. | Br: ≥8 life \| Sil: ≥15 \| Gld: ≥25 \| Plt: ≥40 \| Leg: ≥60 |
| 9 | `negator` | Negator | Control | Counterspells cast targeting opponent spells. | Br: 2 counters \| Sil: 3 \| Gld: 4 \| Plt: 5 \| Leg: 6+ |
| 10 | `sweeper` | Sweeper | Removal | Destroys or removes multiple opponent permanents in a single action. | Br: ≥3 wiped \| Sil: ≥4 \| Gld: ≥6 \| Plt: ≥8 \| Leg: ≥10 |
| 11 | `cataclysm` | Cataclysm | Removal | Total board wipe destroying permanents on both sides. | Br: ≥6 wiped \| Sil: ≥8 \| Gld: ≥12 \| Plt: ≥16 \| Leg: ≥20 |
| 12 | `royal_assassin` | Royal Assassin | Removal | Single-target spot removal of enemy threats. | Br: 2 kills \| Sil: 3 \| Gld: 4 \| Plt: 5 \| Leg: 6+ |
| 13 | `ironclad` | Ironclad | Defense | Damage absorbed while blocking attacker creatures. | Br: ≥8 absorbed \| Sil: ≥15 \| Gld: ≥25 \| Plt: ≥40 \| Leg: ≥60 |
| 14 | `tax_collector` | Tax Collector | Stax | Additional mana taxes paid by opponent to cast spells or attack. | Br: ≥4 mana \| Sil: ≥7 \| Gld: ≥12 \| Plt: ≥18 \| Leg: ≥25 |
| 15 | `cat_burglar` | Cat Burglar | Theft | Cards stolen from opponent library, graveyard, or hand. | Br: 2 cards \| Sil: 3 \| Gld: 5 \| Plt: 7 \| Leg: 10+ |
| 16 | `grand_larceny` | Grand Larceny | Theft | Permanents stolen from opponent's battlefield. | Br: 1 permanent \| Sil: 2 \| Gld: 3 \| Plt: 4 \| Leg: 5+ |
| 17 | `blinkmaster` | Blinkmaster | Synergy | Times a permanent is flickered/exiled and returned to battlefield. | Br: 2 flickers \| Sil: 3 \| Gld: 5 \| Plt: 7 \| Leg: 10+ |
| 18 | `immortal` | Immortal | Survival | Times a card dies and returns to hand or battlefield. | Br: 2 recurrences \| Sil: 3 \| Gld: 4 \| Plt: 5 \| Leg: 6+ |
| 19 | `swarmer` | Swarmer | Tokens | Creature tokens spawned by a single card in a match. | Br: ≥6 tokens \| Sil: ≥10 \| Gld: ≥18 \| Plt: ≥30 \| Leg: ≥50 |
| 20 | `rhystic_tracker` | Rhystic Tracker | Value | Extra cards drawn from a single engine card. | Br: ≥4 cards \| Sil: ≥7 \| Gld: ≥12 \| Plt: ≥18 \| Leg: ≥25 |
| 21 | `mana_dynamo` | Mana Dynamo | Ramp | Total mana produced by a card in a single turn. | Br: ≥6 mana \| Sil: ≥10 \| Gld: ≥16 \| Plt: ≥24 \| Leg: ≥35 |
| 22 | `brain_freeze` | Brain Freeze | Mill | Cards milled from opponent's library. | Br: ≥10 cards \| Sil: ≥18 \| Gld: ≥30 \| Plt: ≥45 \| Leg: ≥60 |
| 23 | `bouncer` | Bouncer | Tempo | Permanents returned to owner's hand (bounced). | Br: 2 bounces \| Sil: 3 \| Gld: 5 \| Plt: 7 \| Leg: 10+ |
| 24 | `thought_seizer` | Thought Seizer | Discard | Cards forced discarded from opponent's hand. | Br: 2 discards \| Sil: 3 \| Gld: 5 \| Plt: 7 \| Leg: 10+ |
| 25 | `reanimator` | Reanimator | Graveyard | Permanents returned from graveyard directly to the battlefield. | Br: 2 returns \| Sil: 3 \| Gld: 4 \| Plt: 5 \| Leg: 6+ |
| 26 | `rider_of_the_storm`| Rider of the Storm | Spellslinger | Instant/Sorcery spells copied in a match. | Br: 2 copies \| Sil: 3 \| Gld: 5 \| Plt: 7 \| Leg: 10+ |

---

### 4.2 Deck Honors Registry (6 Badges)

Evaluated per deck upon match victory in `src-tauri/src/deck_achievements.rs`:

| Honor ID | Title | Concept & Eligibility Rules | Thresholds (Bronze / Silver / Gold) |
| :--- | :--- | :--- | :--- |
| `on_a_roll` | On a Roll | Consecutive match win streak with the exact same deck. | Bronze: **3 wins**<br>Silver: **5 wins**<br>Gold: **7+ wins** |
| `comeback_kid` | Comeback Kid | Victory achieved after dropping to near-death life total at any point during the game. | Bronze: **≤ 5 life**<br>Silver: **≤ 2 life**<br>Gold: **1 life** |
| `gotta_go_fast` | Gotta Go Fast | Fast win (by player's turn count). Excludes opponent concessions. | Bronze: **Turn 4**<br>Silver: **Turn 3**<br>Gold: **Turn 2 or earlier** |
| `iron_fortress` | Iron Fortress | Winning a complete match (≥ 4 turns) while sustaining almost no damage relative to format starting life (20 constructed, 25 Brawl, 40 Commander). | Bronze: **Lost ≤ 5 life**<br>Silver: **Lost ≤ 2 life**<br>Gold: **Lost 0 life** |
| `marathon` | Marathon Master | Winning an endurance match reaching high global turn counts. | Bronze: **Turn 15**<br>Silver: **Turn 20**<br>Gold: **Turn 25+** |
| `dominance` | Dominance | Ending the match with life total substantially higher than starting life. | Bronze: **+10 above starting life**<br>Silver: **+20 above starting life**<br>Gold: **+30+ above starting life** |

*Eligibility Filter:* Excludes Bot matches, Sparky challenges, Starter Deck Duels, Jump In, and preconstructed event decks (`preset_deck_reason`).

---

### 4.3 Local Hall of Fame Queries

In `src-tauri/src/commands/analytics.rs`, `get_global_leaderboards` queries the local `match_impactful_cards` table for 9 podium records:
1. `combat_single_hit`: `MAX(CASE WHEN max_hit_combat > 0 THEN max_hit_combat ELSE max_hit END)`
2. `combat_match_damage`: `MAX(damage_combat)`
3. `combat_total_damage`: `SUM(damage_combat)`
4. `spell_single_hit`: `MAX(max_hit_spell)`
5. `spell_match_damage`: `MAX(damage_spell)`
6. `spell_total_damage`: `SUM(damage_spell)`
7. `most_decorated`: `SUM(json_array_length(titles))`
8. `draw_engines`: `SUM(cards_drawn)`
9. `battlefield_stalwarts`: `COUNT(match_id)` where non-land

---

### 4.4 Global Server Leaderboards & Handle Claiming

In `server/src/db.rs` and `server/src/portal.rs`:
- Server exposes `GET /api/v1/leaderboards?format=<fmt>&limit=50`.
- **Claimed Handles Pinning (`claimed_handles` table):**
  - When a player syncs with Tier 3 (`share_player_identity: true`), their Arena handle is normalized (e.g. `Hero#12345` -> `Hero`).
  - On initial sync, the server binds `normalized_handle` to `SHA256(contributor_id)`.
  - Future matches submitted under that handle with a different token hash are rejected from public leaderboards.
- Aggregates seasonal win rates, matches played, and highest combat hits across all participating players.

---

## 5. SQLite Database Schemas

### 5.1 Client Database Schema (`rhystic.db`)

Created and migrated in `src-tauri/src/db.rs`:

```sql
-- 1. Soft-deleted matches log (prevents re-import on log catchup)
CREATE TABLE IF NOT EXISTS deleted_matches (
    match_id TEXT PRIMARY KEY,
    deleted_at TEXT NOT NULL
);

-- 2. Core match history records
CREATE TABLE IF NOT EXISTS matches (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    date_str TEXT NOT NULL,
    format TEXT NOT NULL,
    result TEXT NOT NULL,
    duration_seconds INTEGER NOT NULL,
    turns INTEGER NOT NULL,
    going_first BOOLEAN NOT NULL,
    hero_seat_id INTEGER NOT NULL DEFAULT 1,
    hero_deck_name TEXT,
    hero_commander_id INTEGER,
    hero_life_end INTEGER,
    hero_mulligans INTEGER DEFAULT 0,
    hero_platform TEXT,
    hero_avatar TEXT,
    opponent_name TEXT,
    opponent_colors TEXT,
    opponent_commander_id INTEGER,
    opponent_mulligans INTEGER,
    opponent_life_end INTEGER,
    opponent_platform TEXT,
    opponent_avatar TEXT,
    result_reason TEXT,
    hero_min_life INTEGER,
    hero_starting_life INTEGER,
    raw_payload TEXT
);
CREATE INDEX IF NOT EXISTS idx_matches_timestamp ON matches(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_matches_hero_deck_name ON matches(hero_deck_name);
CREATE INDEX IF NOT EXISTS idx_matches_opponent_name ON matches(opponent_name);

-- 3. Cards observed during match play
CREATE TABLE IF NOT EXISTS match_cards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    match_id TEXT NOT NULL,
    grp_id INTEGER NOT NULL,
    is_opponent BOOLEAN NOT NULL,
    count INTEGER NOT NULL,
    FOREIGN KEY (match_id) REFERENCES matches(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_match_cards_match_id ON match_cards(match_id);

-- 4. Scryfall card metadata cache
CREATE TABLE IF NOT EXISTS cards_cache (
    grp_id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    mana_cost TEXT,
    cmc INTEGER NOT NULL DEFAULT 0,
    colors TEXT,
    color_identity TEXT,
    set_code TEXT,
    rarity INTEGER NOT NULL,
    collector_number TEXT,
    card_type TEXT,
    last_updated TEXT NOT NULL
);

-- 5. Turn-by-turn play event replay log
CREATE TABLE IF NOT EXISTS match_turn_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    match_id TEXT NOT NULL,
    turn_number INTEGER NOT NULL,
    seat_id INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    grp_id INTEGER NOT NULL,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (match_id) REFERENCES matches(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_match_turn_events_match_id ON match_turn_events(match_id);
CREATE INDEX IF NOT EXISTS idx_match_turn_events_seat_type ON match_turn_events(seat_id, event_type, grp_id);

-- 6. Micro-combat metrics and earned Card Trophies
CREATE TABLE IF NOT EXISTS match_impactful_cards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    match_id TEXT NOT NULL,
    grp_id INTEGER NOT NULL,
    seat_id INTEGER NOT NULL,
    total_damage INTEGER NOT NULL DEFAULT 0,
    max_hit INTEGER NOT NULL DEFAULT 0,
    max_hit_combat INTEGER NOT NULL DEFAULT 0,
    max_hit_spell INTEGER NOT NULL DEFAULT 0,
    damage_to_player INTEGER NOT NULL DEFAULT 0,
    damage_to_permanents INTEGER NOT NULL DEFAULT 0,
    damage_combat INTEGER NOT NULL DEFAULT 0,
    damage_spell INTEGER NOT NULL DEFAULT 0,
    titles TEXT DEFAULT '[]',
    cards_drawn INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (match_id) REFERENCES matches(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_match_impactful_cards_match_id ON match_impactful_cards(match_id);
CREATE INDEX IF NOT EXISTS idx_match_impactful_cards_hero ON match_impactful_cards(seat_id, grp_id);

-- 7. Submitted / imported full decklists
CREATE TABLE IF NOT EXISTS deck_lists (
    deck_name TEXT PRIMARY KEY,
    cards_json TEXT NOT NULL,
    sideboard_json TEXT,
    commander_grp_id INTEGER,
    source TEXT DEFAULT 'export',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deck_id TEXT
);

-- 8. Custom deck thumbnail overrides
CREATE TABLE IF NOT EXISTS deck_art_overrides (
    deck_name TEXT PRIMARY KEY,
    card_name TEXT NOT NULL,
    grp_id INTEGER,
    updated_at TEXT NOT NULL
);

-- 9. Custom deck wallpaper art overrides
CREATE TABLE IF NOT EXISTS deck_bg_art_overrides (
    deck_name TEXT PRIMARY KEY,
    card_name TEXT NOT NULL,
    grp_id INTEGER,
    updated_at TEXT NOT NULL
);

-- 10. Draw-discovered card collection
CREATE TABLE IF NOT EXISTS collection_cards (
    grp_id INTEGER PRIMARY KEY,
    owned_count INTEGER NOT NULL DEFAULT 0,
    provenance TEXT NOT NULL DEFAULT '',
    first_seen_at TEXT,
    last_updated_at TEXT,
    draw_seen INTEGER NOT NULL DEFAULT 0
);

-- 11. Match deck audit log (filters preset starter decks)
CREATE TABLE IF NOT EXISTS match_decks (
    match_id TEXT PRIMARY KEY,
    deck_name TEXT,
    deck_id TEXT,
    preset_deck BOOLEAN NOT NULL DEFAULT 0,
    exclusion_reason TEXT,
    submitted_at TEXT,
    FOREIGN KEY (match_id) REFERENCES matches(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_match_decks_deck_id ON match_decks(deck_id);

-- 12. MTGA Set metadata catalog
CREATE TABLE IF NOT EXISTS sets_metadata (
    set_code TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    released_at TEXT,
    icon_svg_uri TEXT,
    updated_at TEXT NOT NULL
);

-- 13. Bento Dashboard layout persistence
CREATE TABLE IF NOT EXISTS dashboard_layouts (
    id TEXT PRIMARY KEY,
    schema_version INTEGER NOT NULL,
    layout_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 14. Preferred card printing selections
CREATE TABLE IF NOT EXISTS card_preferred_prints (
    card_name TEXT PRIMARY KEY,
    set_code TEXT NOT NULL,
    collector_number TEXT NOT NULL,
    grp_id INTEGER,
    updated_at TEXT NOT NULL
);

-- 15. Deck-level achievements
CREATE TABLE IF NOT EXISTS deck_achievements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    deck_name TEXT NOT NULL,
    achievement_id TEXT NOT NULL,
    tier TEXT NOT NULL,
    achieved_at TEXT NOT NULL,
    match_id TEXT,
    UNIQUE(deck_name, achievement_id, tier)
);
CREATE INDEX IF NOT EXISTS idx_deck_achievements_deck ON deck_achievements(deck_name);

-- 16. Outbox queue for opt-in community telemetry sync
CREATE TABLE IF NOT EXISTS sync_outbox (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    match_id TEXT NOT NULL UNIQUE,
    payload_json TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    created_at TEXT NOT NULL,
    synced_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_sync_outbox_status ON sync_outbox(status);

-- 17. Player economy snapshots (currencies & inventory)
CREATE TABLE IF NOT EXISTS player_economy_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    gold INTEGER NOT NULL,
    gems INTEGER NOT NULL,
    vault_progress_tenths INTEGER NOT NULL,
    wc_track_pos INTEGER NOT NULL,
    wc_common INTEGER NOT NULL,
    wc_uncommon INTEGER NOT NULL,
    wc_rare INTEGER NOT NULL,
    wc_mythic INTEGER NOT NULL,
    draft_tokens INTEGER NOT NULL DEFAULT 0,
    jump_in_tokens INTEGER NOT NULL DEFAULT 0,
    golden_pack_progress INTEGER NOT NULL DEFAULT 0,
    boosters_json TEXT NOT NULL DEFAULT '[]'
);
CREATE INDEX IF NOT EXISTS idx_player_economy_snapshots_timestamp ON player_economy_snapshots(timestamp);

-- 18. Booster pack opening log
CREATE TABLE IF NOT EXISTS player_booster_openings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    pack_id TEXT,
    cards_json TEXT NOT NULL,
    wildcards_json TEXT,
    vault_delta REAL
);
CREATE INDEX IF NOT EXISTS idx_player_booster_openings_timestamp ON player_booster_openings(timestamp);

-- 19. Daily quests lifecycle
CREATE TABLE IF NOT EXISTS player_quests (
    quest_id TEXT PRIMARY KEY,
    loc_key TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    category TEXT NOT NULL,
    colors TEXT NOT NULL,
    goal INTEGER NOT NULL,
    current_progress INTEGER NOT NULL,
    starting_progress INTEGER NOT NULL,
    reward_gold INTEGER NOT NULL,
    reward_xp INTEGER NOT NULL,
    can_swap BOOLEAN NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    completed_at TEXT,
    duration_seconds INTEGER,
    matches_played_during INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_player_quests_status ON player_quests(status);
CREATE INDEX IF NOT EXISTS idx_player_quests_category ON player_quests(category);
CREATE INDEX IF NOT EXISTS idx_player_quests_first_seen ON player_quests(first_seen_at DESC);

-- 20. Daily quest swap / reroll audit log
CREATE TABLE IF NOT EXISTS player_quest_rerolls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rerolled_at TEXT NOT NULL,
    old_quest_id TEXT NOT NULL,
    old_title TEXT NOT NULL,
    old_reward_gold INTEGER NOT NULL,
    old_reward_xp INTEGER NOT NULL,
    old_category TEXT NOT NULL,
    new_quest_id TEXT NOT NULL,
    new_title TEXT NOT NULL,
    new_reward_gold INTEGER NOT NULL,
    new_reward_xp INTEGER NOT NULL,
    new_category TEXT NOT NULL,
    gold_diff INTEGER NOT NULL,
    is_upgrade BOOLEAN NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_player_quest_rerolls_old_id ON player_quest_rerolls(old_quest_id);
CREATE INDEX IF NOT EXISTS idx_player_quest_rerolls_at ON player_quest_rerolls(rerolled_at DESC);

-- 21. Periodic win tracks (daily & weekly win resets)
CREATE TABLE IF NOT EXISTS player_reward_tracks (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    daily_reset_timestamp TEXT NOT NULL,
    weekly_reset_timestamp TEXT NOT NULL,
    daily_wins INTEGER NOT NULL DEFAULT 0,
    weekly_wins INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);

-- 22. Ranked ladder progression snapshots
CREATE TABLE IF NOT EXISTS player_rank_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    season_ordinal INTEGER NOT NULL,
    constructed_tier TEXT NOT NULL,
    constructed_level INTEGER NOT NULL,
    constructed_step INTEGER NOT NULL,
    constructed_wins INTEGER NOT NULL,
    constructed_losses INTEGER NOT NULL,
    limited_tier TEXT NOT NULL,
    limited_level INTEGER NOT NULL,
    limited_step INTEGER NOT NULL,
    limited_wins INTEGER NOT NULL,
    limited_losses INTEGER NOT NULL,
    season_end_time TEXT
);
CREATE INDEX IF NOT EXISTS idx_player_rank_snapshots_timestamp ON player_rank_snapshots(timestamp);
CREATE INDEX IF NOT EXISTS idx_player_rank_snapshots_season ON player_rank_snapshots(season_ordinal);

-- 23. Player seasonal schedule metadata
CREATE TABLE IF NOT EXISTS player_season_info (
    season_ordinal INTEGER PRIMARY KEY,
    season_start_time TEXT,
    season_end_time TEXT,
    updated_at TEXT NOT NULL
);

-- 24. Mastery Pass progression and rewards
CREATE TABLE IF NOT EXISTS player_mastery_pass (
    pass_id TEXT PRIMARY KEY,
    set_code TEXT NOT NULL,
    pass_name TEXT NOT NULL,
    current_level INTEGER NOT NULL,
    current_xp INTEGER NOT NULL,
    xp_per_level INTEGER NOT NULL DEFAULT 1000,
    is_premium BOOLEAN NOT NULL DEFAULT 0,
    orbs INTEGER NOT NULL DEFAULT 0,
    max_level INTEGER NOT NULL DEFAULT 40,
    claimed_levels_json TEXT NOT NULL DEFAULT '[]',
    updated_at TEXT NOT NULL
);
```

---

### 5.2 Server Database Schema (`rhystic_cloud.db`)

Created and migrated in `server/src/db.rs`:

```sql
-- 1. Ingested anonymized match headers
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
CREATE INDEX IF NOT EXISTS idx_ingested_matches_format ON ingested_matches(format);
CREATE INDEX IF NOT EXISTS idx_ingested_matches_commander ON ingested_matches(commander_grp_id, format);
CREATE INDEX IF NOT EXISTS idx_ingested_matches_opp_commander ON ingested_matches(opponent_commander_grp_id, format);
CREATE INDEX IF NOT EXISTS idx_ingested_matches_archetype ON ingested_matches(archetype_hash, format);
CREATE INDEX IF NOT EXISTS idx_ingested_matches_player ON ingested_matches(player_account_hash);

-- 2. Ingested seen card identifiers
CREATE TABLE IF NOT EXISTS ingested_match_cards (
    match_hash TEXT NOT NULL,
    grp_id INTEGER NOT NULL,
    is_opponent INTEGER NOT NULL,
    count INTEGER NOT NULL,
    PRIMARY KEY (match_hash, grp_id, is_opponent)
);
CREATE INDEX IF NOT EXISTS idx_ingested_match_cards_grp ON ingested_match_cards(grp_id, is_opponent);

-- 3. Ingested combat impact and titles
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
CREATE INDEX IF NOT EXISTS idx_ingested_impactful_grp ON ingested_impactful_cards(grp_id);

-- 4. Cached card names
CREATE TABLE IF NOT EXISTS ingested_card_names (
    grp_id INTEGER PRIMARY KEY,
    card_name TEXT NOT NULL
);

-- 5. Full submitted decklist cards (powers archetype clustering)
CREATE TABLE IF NOT EXISTS ingested_deck_cards (
    match_hash TEXT NOT NULL,
    grp_id INTEGER NOT NULL,
    count INTEGER NOT NULL,
    is_basic_land INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (match_hash, grp_id)
);
CREATE INDEX IF NOT EXISTS idx_ingested_deck_cards_grp ON ingested_deck_cards(grp_id);

-- 6. Claimed Arena handles bound to contributor tokens
CREATE TABLE IF NOT EXISTS claimed_handles (
    normalized_handle TEXT PRIMARY KEY,
    display_handle TEXT NOT NULL,
    player_account_hash TEXT NOT NULL,
    owner_token_hash TEXT NOT NULL,
    claimed_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_claimed_handles_account_hash ON claimed_handles(player_account_hash);
```

---

## 6. Client vs. Server Feature Matrix

| Feature / Metric | Client Offline (`Rhystic Desktop`) | Central Server (`server/`) | Requirement & Implementation Details |
| :--- | :---: | :---: | :--- |
| **Real-time Log Tailing** | **Yes** | No | 100% local. Reads local MTGA `Player.log` incrementally. |
| **Live Match HUD** | **Yes** | No | Real-time board state, hero/opp life, turn counter, and Bo3 sideboard. |
| **Floating Live Deck Tracker** | **Yes** | No | Hypergeometric probability engine calculating draw odds for lands/cards. |
| **Match History & Replay** | **Yes** | No | Full match history, duration, turns, mulligan count, turn timeline. |
| **Local Deck Library** | **Yes** | No | Mana curves, custom art overrides, deck owned card count, local stats. |
| **Collection & Economy** | **Yes** | No | Draw-based collection, booster openings, quest tracking, currency history. |
| **Local Card Trophies** | **Yes** | No | 26 badges evaluated locally per card based on game combat events. |
| **Local Deck Honors** | **Yes** | No | 6 badges evaluated locally based on match streaks, comebacks, and speed. |
| **Personal Hall of Fame** | **Yes** | No | Top 9 personal combat records stored in `match_impactful_cards`. |
| **Global Format Tier Lists** | No | **Yes** | Aggregates win rates & play rates across multiple players per format. |
| **Commander Meta & Matchup Matrix** | No | **Yes** | Aggregates commander win rates and head-to-head records across contributors. |
| **Archetype Core vs. Flex Slots** | No | **Yes** | Clusters canonical `archetype_hash` decklists into core cards vs. flex substitutes with win-rate correlations. |
| **Card Win-Rate Engine (GIH WR)** | No | **Yes** | Aggregates win rates when cards are drawn/seen across all players. |
| **Public Player Leaderboard** | No | **Yes** | Ranks opted-in players (Tier 3) by seasonal rank and match win rate. |
| **Handle Claim Pinning** | No | **Yes** | Binds Arena screen names to secret contributor tokens to prevent spoofing. |
| **Global Combat Hall of Fame** | No | **Yes** | Surfaces the highest single-hit damage and total damage strikes community-wide. |
| **Web Companion Portal** | No | **Yes** | Interactive web analytics dashboard (`GET /` on `server/`). |
