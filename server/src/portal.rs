//! Phase 5: Rhystic Community Meta & Companion Web Portal (`GET /`).
//!
//! Styled to match the local Rhystic Tracker desktop application's dark obsidian-slate
//! glassmorphic aesthetic (`backdrop-filter: blur(16px)`), display serif typography,
//! authentic `mana-font` WUBRG cost pips, `keyrune` set/format glyphs, lazy Scryfall
//! `art_crop` header banners, visual win-rate/inclusion bars, and 1-click MTGA deck export.

use axum::response::Html;

pub async fn portal_index_handler() -> Html<&'static str> {
    Html(PORTAL_HTML)
}

pub const PORTAL_HTML: &str = r##"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Rhystic Tracker — Community Meta &amp; Companion Portal</title>
  <script src="https://cdn.tailwindcss.com"></script>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Cinzel:wght@600;700;800&family=Inter:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500;600;700&display=swap" rel="stylesheet">
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/mana-font@latest/css/mana.min.css" />
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/keyrune@latest/css/keyrune.min.css" />
  <script>
    tailwind.config = {
      darkMode: 'class',
      theme: {
        extend: {
          fontFamily: {
            display: ['Beleren', 'Cinzel', 'Georgia', 'serif'],
            sans: ['Inter', 'system-ui', 'sans-serif'],
            mono: ['JetBrains Mono', 'monospace'],
          },
          colors: {
            rt: {
              base: '#0E0E10',
              mantle: '#151518',
              surface: '#1D1D22',
              border: '#2A2A32',
              gold: '#D4A237',
              indigo: '#4A7FA3',
              green: '#3FA66C',
              red: '#C24E42',
            }
          }
        }
      }
    }
  </script>
  <style>
    :root {
      --rt-base: #0E0E10;
      --rt-mantle: #151518;
      --rt-surface: #1D1D22;
      --rt-border: #2A2A32;
      --rt-text: #F4F4F6;
      --rt-subtext: #92929D;
      --rt-accent: #4A7FA3;
      --rt-gold: #D4A237;
      --rt-green: #3FA66C;
      --rt-red: #C24E42;
    }
    body {
      background-color: var(--rt-base);
      background-image:
        radial-gradient(circle at 12% 8%, rgba(74, 127, 163, 0.14), transparent 38%),
        radial-gradient(circle at 88% 12%, rgba(212, 162, 55, 0.10), transparent 36%),
        radial-gradient(circle at 50% 92%, rgba(63, 166, 108, 0.08), transparent 42%);
      color: var(--rt-text);
      min-height: 100vh;
      font-variant-numeric: tabular-nums lining-nums;
    }
    .glass-panel {
      background: rgba(21, 21, 24, 0.72);
      border: 1px solid rgba(255, 255, 255, 0.09);
      backdrop-filter: blur(16px);
      -webkit-backdrop-filter: blur(16px);
      box-shadow: 0 12px 32px -8px rgba(0, 0, 0, 0.55);
    }
    .glass-panel-hover {
      transition: all 0.18s cubic-bezier(0.22, 1, 0.36, 1);
    }
    .glass-panel-hover:hover {
      background: rgba(29, 29, 34, 0.85);
      border-color: rgba(212, 162, 55, 0.35);
      transform: translateY(-1px);
    }
    .rt-gold-glow {
      box-shadow: 0 0 24px -6px rgba(212, 162, 55, 0.35);
    }
    .art-crop-banner {
      position: relative;
      overflow: hidden;
      background: linear-gradient(135deg, #151518 0%, #1D1D22 50%, #0E0E10 100%);
    }
    .art-crop-img {
      position: absolute;
      inset: 0;
      width: 100%;
      height: 100%;
      object-fit: cover;
      object-position: center 22%;
      filter: blur(3px) saturate(1.15);
      transform: scale(1.06);
      opacity: 0.34;
      pointer-events: none;
      transition: opacity 0.3s ease, transform 0.4s ease;
    }
    .art-crop-banner:hover .art-crop-img {
      opacity: 0.44;
      transform: scale(1.09);
    }
    .art-crop-overlay {
      position: absolute;
      inset: 0;
      background: linear-gradient(90deg, rgba(14, 14, 16, 0.94) 0%, rgba(14, 14, 16, 0.76) 55%, rgba(14, 14, 16, 0.88) 100%);
      pointer-events: none;
    }
    .ms-cost.ms-shadow {
      text-shadow: 0 1px 2px rgba(0,0,0,0.65);
    }
    .wr-bar-track {
      height: 5px;
      background: rgba(255, 255, 255, 0.08);
      overflow: hidden;
    }
    .wr-bar-fill {
      height: 100%;
      transition: width 0.35s cubic-bezier(0.22, 1, 0.36, 1);
    }
    ::-webkit-scrollbar { width: 8px; height: 8px; }
    ::-webkit-scrollbar-track { background: #0E0E10; }
    ::-webkit-scrollbar-thumb { background: #2A2A32; border-radius: 4px; }
    ::-webkit-scrollbar-thumb:hover { background: #4A7FA3; }
  </style>
</head>
<body class="font-sans antialiased selection:bg-amber-500/30 selection:text-amber-200">
  <!-- Floating Scryfall Card Preview Tooltip -->
  <div id="card-preview-tooltip" class="fixed z-50 pointer-events-none hidden transition-opacity duration-150">
    <div class="p-1.5 bg-[#0E0E10]/95 border border-amber-400/40 shadow-2xl rt-gold-glow">
      <img id="card-preview-img" src="" alt="Card Preview" class="w-60 h-auto block bg-[#151518]" />
      <div id="card-preview-caption" class="px-2 py-1 text-[10px] font-mono text-amber-300 text-center truncate max-w-[240px]"></div>
    </div>
  </div>

  <!-- Top Navigation Header -->
  <header class="sticky top-0 z-40 border-b border-white/10 bg-[#0E0E10]/90 backdrop-blur-xl">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 py-3.5 flex flex-wrap items-center justify-between gap-4">
      <div class="flex items-center gap-3.5">
        <div class="w-10 h-10 border border-amber-400/40 bg-gradient-to-br from-amber-500/20 via-sky-500/10 to-emerald-500/15 flex items-center justify-center font-display font-bold text-amber-300 text-xl rt-gold-glow">
          <i class="ss ss-cmd text-amber-300 text-lg"></i>
        </div>
        <div>
          <div class="flex items-center gap-2.5">
            <h1 class="text-lg font-display font-bold tracking-[0.10em] uppercase text-white leading-none">
              RHYSTIC TRACKER
            </h1>
            <span class="px-2 py-0.5 text-[10px] font-mono font-bold uppercase tracking-wider bg-amber-500/15 border border-amber-400/40 text-amber-300">
              Community Meta &amp; Cloud
            </span>
          </div>
          <p class="text-[11px] font-mono text-neutral-400 mt-1">
            Privacy-First MTG Arena Analytics • Constructed, Commander &amp; Limited (Alchemy-Free)
          </p>
        </div>
      </div>

      <!-- Global Format Filter & Live Server Status -->
      <div class="flex flex-wrap items-center gap-3">
        <div class="flex items-center gap-2">
          <label for="global-format-select" class="text-[10px] font-mono uppercase tracking-wider text-neutral-400 font-bold">
            Active Format:
          </label>
          <select id="global-format-select" class="px-3 py-1.5 bg-[#151518] border border-white/15 text-xs font-mono text-white focus:outline-none focus:border-amber-400/60">
            <option value="">All Formats (Global)</option>
          </select>
        </div>
        <button id="refresh-all-btn" class="px-3 py-1.5 border border-white/15 bg-white/[0.04] hover:bg-white/[0.09] hover:border-amber-400/40 text-[11px] font-mono font-bold uppercase tracking-wider text-white transition cursor-pointer">
          ↻ Refresh Telemetry
        </button>
        <div id="server-status-pill" class="px-2.5 py-1 border border-emerald-500/30 bg-emerald-500/10 text-[10px] font-mono text-emerald-300 flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
          <span id="server-status-text">Connecting…</span>
        </div>
      </div>
    </div>

    <!-- Primary Tab Bar -->
    <div class="max-w-7xl mx-auto px-4 sm:px-6 flex items-center gap-1 overflow-x-auto border-t border-white/5 pt-1">
      <button data-tab="archetypes" class="tab-btn px-4 py-2.5 text-xs font-mono font-bold uppercase tracking-wider border-b-2 border-amber-400 text-amber-300 bg-white/[0.04] cursor-pointer whitespace-nowrap flex items-center gap-2">
        <i class="ss ss-dfn text-sm"></i>
        <span>Archetypes &amp; Flex-Slot Explorer</span>
      </button>
      <button data-tab="formats" class="tab-btn px-4 py-2.5 text-xs font-mono font-bold uppercase tracking-wider border-b-2 border-transparent text-neutral-400 hover:text-white cursor-pointer whitespace-nowrap flex items-center gap-2">
        <i class="ss ss-lea text-sm"></i>
        <span>Format Matrix (Constructed &amp; Limited)</span>
      </button>
      <button data-tab="commanders" class="tab-btn px-4 py-2.5 text-xs font-mono font-bold uppercase tracking-wider border-b-2 border-transparent text-neutral-400 hover:text-white cursor-pointer whitespace-nowrap flex items-center gap-2">
        <i class="ss ss-cmd text-sm"></i>
        <span>Commander &amp; Brawl Tier List</span>
      </button>
      <button data-tab="cards" class="tab-btn px-4 py-2.5 text-xs font-mono font-bold uppercase tracking-wider border-b-2 border-transparent text-neutral-400 hover:text-white cursor-pointer whitespace-nowrap flex items-center gap-2">
        <i class="ms ms-planeswalker text-sm"></i>
        <span>Card Performance &amp; Combat</span>
      </button>
      <button data-tab="leaderboards" class="tab-btn px-4 py-2.5 text-xs font-mono font-bold uppercase tracking-wider border-b-2 border-transparent text-neutral-400 hover:text-white cursor-pointer whitespace-nowrap flex items-center gap-2">
        <i class="ss ss-vma text-sm"></i>
        <span>Claimed Leaderboards &amp; Hall of Fame</span>
      </button>
    </div>
  </header>

  <main class="max-w-7xl mx-auto px-4 sm:px-6 py-6 space-y-6">
    <!-- Top KPI Telemetry Strip -->
    <section class="grid grid-cols-2 sm:grid-cols-5 gap-3">
      <div class="glass-panel p-4 border-l-2 border-l-amber-400/70">
        <div class="text-[10px] font-mono uppercase tracking-wider text-neutral-400">Total Synced Matches</div>
        <div id="kpi-total-matches" class="text-2xl font-display font-bold text-white mt-1">—</div>
      </div>
      <div class="glass-panel p-4 border-l-2 border-l-sky-400/70">
        <div class="text-[10px] font-mono uppercase tracking-wider text-neutral-400">Constructed Matches</div>
        <div id="kpi-constructed-matches" class="text-2xl font-display font-bold text-sky-300 mt-1">—</div>
      </div>
      <div class="glass-panel p-4 border-l-2 border-l-amber-500/70">
        <div class="text-[10px] font-mono uppercase tracking-wider text-neutral-400">Limited (Draft/Sealed)</div>
        <div id="kpi-limited-matches" class="text-2xl font-display font-bold text-amber-300 mt-1">—</div>
      </div>
      <div class="glass-panel p-4 border-l-2 border-l-emerald-400/70">
        <div class="text-[10px] font-mono uppercase tracking-wider text-neutral-400">Commander &amp; Brawl</div>
        <div id="kpi-commander-matches" class="text-2xl font-display font-bold text-emerald-300 mt-1">—</div>
      </div>
      <div class="glass-panel p-4 col-span-2 sm:col-span-1 border-l-2 border-l-purple-400/70">
        <div class="text-[10px] font-mono uppercase tracking-wider text-neutral-400">Unique Contributors</div>
        <div id="kpi-contributors" class="text-2xl font-display font-bold text-purple-300 mt-1">—</div>
      </div>
    </section>

    <!-- TAB 1: ARCHETYPES & INTERACTIVE CORE + FLEX-SLOT EXPLORER -->
    <section id="tab-archetypes" class="tab-panel space-y-4">
      <div class="glass-panel p-4 flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 class="text-sm font-display font-bold uppercase tracking-wider text-amber-300">
            Unified Archetype &amp; Flex-Slot Explorer
          </h2>
          <p class="text-xs text-neutral-400 mt-0.5">
            Select any archetype on the left to inspect its <strong class="text-white">Fixed Core (≥70% inclusion)</strong> side-by-side with its <strong class="text-emerald-300">Flex-Slot Win-Rate Delta (&lt;70% inclusion)</strong>, hover any card for live Scryfall art, and export directly to MTG Arena.
          </p>
        </div>
        <div class="flex items-center gap-2">
          <span class="px-2.5 py-1 border border-amber-400/30 bg-amber-500/10 text-[10px] font-mono text-amber-200">
            Ranked by 95% Wilson Confidence Bound
          </span>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-12 gap-4 items-start">
        <!-- Left Column: Archetype Tier List (5 cols) -->
        <div class="lg:col-span-5 space-y-2.5" id="archetype-list-container">
          <div class="glass-panel p-6 text-center text-xs font-mono text-neutral-400">
            Loading archetype clusters…
          </div>
        </div>

        <!-- Right Column: Unified Core + Flex-Slot Inspector (7 cols) -->
        <div class="lg:col-span-7 glass-panel overflow-hidden" id="archetype-inspector">
          <div class="text-center py-12 px-4 text-xs font-mono text-neutral-400">
            Select an archetype cluster on the left to inspect its Core Staples, Flex-Slot Win-Rate Deltas, and Exact Variants.
          </div>
        </div>
      </div>
    </section>

    <!-- TAB 2: FORMAT MATRIX -->
    <section id="tab-formats" class="tab-panel hidden space-y-4">
      <div class="glass-panel p-4 flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 class="text-sm font-display font-bold uppercase tracking-wider text-sky-300">
            MTG Arena Format Matrix (All Constructed, Commander &amp; Limited)
          </h2>
          <p class="text-xs text-neutral-400 mt-0.5">
            Compare average game length, Play vs. Draw win-rate split, and Concession ("Salt Index") across every official Arena format (Alchemy excluded).
          </p>
        </div>
        <div class="flex flex-wrap items-center gap-1.5">
          <button data-cat="" class="fmt-cat-btn px-3 py-1 text-[10px] font-mono font-bold uppercase border border-amber-400/50 bg-amber-500/15 text-amber-300 cursor-pointer">
            All Formats
          </button>
          <button data-cat="Commander" class="fmt-cat-btn px-3 py-1 text-[10px] font-mono font-bold uppercase border border-white/15 bg-white/[0.03] text-neutral-300 cursor-pointer">
            Commander &amp; Brawl
          </button>
          <button data-cat="Constructed" class="fmt-cat-btn px-3 py-1 text-[10px] font-mono font-bold uppercase border border-white/15 bg-white/[0.03] text-neutral-300 cursor-pointer">
            Constructed
          </button>
          <button data-cat="Limited" class="fmt-cat-btn px-3 py-1 text-[10px] font-mono font-bold uppercase border border-white/15 bg-white/[0.03] text-neutral-300 cursor-pointer">
            Limited (Draft &amp; Sealed)
          </button>
          <label class="ml-2 inline-flex items-center gap-1.5 text-[11px] font-mono text-neutral-300 cursor-pointer">
            <input type="checkbox" id="active-formats-only" class="accent-amber-500" />
            Active matches only
          </label>
        </div>
      </div>

      <div class="glass-panel overflow-x-auto">
        <table class="w-full text-left border-collapse">
          <thead>
            <tr class="border-b border-white/10 text-[10px] font-mono uppercase tracking-wider text-neutral-400 bg-black/40">
              <th class="py-3 px-4">Format</th>
              <th class="py-3 px-3">Category</th>
              <th class="py-3 px-3 text-right">Matches</th>
              <th class="py-3 px-3 text-right">Win Rate</th>
              <th class="py-3 px-3 text-right">Play / Draw Split</th>
              <th class="py-3 px-3 text-right">Avg Turns</th>
              <th class="py-3 px-4 text-right">Salt Index (Concede %)</th>
            </tr>
          </thead>
          <tbody id="formats-table-body" class="divide-y divide-white/5 text-xs font-mono"></tbody>
        </table>
      </div>
    </section>

    <!-- TAB 3: COMMANDER & BRAWL TIER LIST -->
    <section id="tab-commanders" class="tab-panel hidden space-y-4">
      <div class="glass-panel p-4 flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 class="text-sm font-display font-bold uppercase tracking-wider text-amber-300">
            Commander &amp; Brawl Meta Tier List + Salt Index
          </h2>
          <p class="text-xs text-neutral-400 mt-0.5">
            Command-zone win rates, Play/Draw advantage, average turn clock, and Opponent Concession Rate ("Salt Index"). Click any Commander to inspect head-to-head matchups with Scryfall art.
          </p>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-12 gap-4">
        <div class="lg:col-span-8 glass-panel overflow-x-auto">
          <table class="w-full text-left border-collapse">
            <thead>
              <tr class="border-b border-white/10 text-[10px] font-mono uppercase tracking-wider text-neutral-400 bg-black/40">
                <th class="py-3 px-4">Tier</th>
                <th class="py-3 px-3">Commander</th>
                <th class="py-3 px-3">Format</th>
                <th class="py-3 px-3 text-right">Matches</th>
                <th class="py-3 px-3 text-right">Win Rate</th>
                <th class="py-3 px-3 text-right">Play / Draw</th>
                <th class="py-3 px-3 text-right">Avg Turns</th>
                <th class="py-3 px-4 text-right">Salt Index</th>
              </tr>
            </thead>
            <tbody id="commanders-table-body" class="divide-y divide-white/5 text-xs font-mono"></tbody>
          </table>
        </div>

        <div class="lg:col-span-4 glass-panel overflow-hidden" id="commander-matchups-panel">
          <div class="p-4 space-y-3">
            <h3 class="text-xs font-display font-bold uppercase tracking-wider text-white border-b border-white/10 pb-2">
              Head-to-Head Matchup Matrix
            </h3>
            <p class="text-xs font-mono text-neutral-400">
              Click a Commander row on the left to inspect its head-to-head combat record against opposing Commanders.
            </p>
          </div>
        </div>
      </div>
    </section>

    <!-- TAB 4: CARD PERFORMANCE & COMBAT TELEMETRY -->
    <section id="tab-cards" class="tab-panel hidden space-y-4">
      <div class="glass-panel p-4 flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 class="text-sm font-display font-bold uppercase tracking-wider text-purple-300">
            Card Performance &amp; Micro-Combat Analytics
          </h2>
          <p class="text-xs text-neutral-400 mt-0.5">
            Game-In-Hand / Seen Win Rate, Total Damage Dealt, Biggest Single Hit, Cards Drawn, and Earned Combat Titles. Hover any named card for live Scryfall art.
          </p>
        </div>
        <div>
          <input id="card-search-input" type="text" placeholder="Filter by card name or grpId…" class="px-3 py-1.5 bg-[#151518] border border-white/15 text-xs font-mono text-white placeholder:text-neutral-500 focus:outline-none focus:border-amber-400" />
        </div>
      </div>

      <div class="glass-panel overflow-x-auto">
        <table class="w-full text-left border-collapse">
          <thead>
            <tr class="border-b border-white/10 text-[10px] font-mono uppercase tracking-wider text-neutral-400 bg-black/40">
              <th class="py-3 px-4">Card</th>
              <th class="py-3 px-3 text-right">Games Seen</th>
              <th class="py-3 px-3 text-right">Seen Win Rate</th>
              <th class="py-3 px-3 text-right">Total Damage</th>
              <th class="py-3 px-3 text-right">Max Single Hit</th>
              <th class="py-3 px-3 text-right">Combat / Spell</th>
              <th class="py-3 px-3 text-right">Cards Drawn</th>
              <th class="py-3 px-4">Top Combat Titles</th>
            </tr>
          </thead>
          <tbody id="cards-table-body" class="divide-y divide-white/5 text-xs font-mono"></tbody>
        </table>
      </div>
    </section>

    <!-- TAB 5: GLOBAL CLAIMED LEADERBOARDS & HALL OF FAME -->
    <section id="tab-leaderboards" class="tab-panel hidden space-y-4">
      <div class="glass-panel p-4 flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 class="text-sm font-display font-bold uppercase tracking-wider text-amber-300">
            Global Log-Detected Player Leaderboards &amp; Combat Hall of Fame
          </h2>
          <p class="text-xs text-neutral-400 mt-0.5">
            Every player handle below is detected from MTG Arena's <code class="font-mono text-neutral-200">AuthenticateResponse</code> in <code class="font-mono text-neutral-200">Player.log</code> (stripped of <code class="font-mono text-neutral-200">#12345</code>) and pinned on first sync to the owner's secret Contributor Token (<code class="font-mono text-neutral-200">claimed_handles</code>)—preventing third-party handle spoofing.
          </p>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-12 gap-4">
        <!-- Claimed Player Leaderboard (7 cols) -->
        <div class="lg:col-span-7 glass-panel overflow-x-auto">
          <div class="p-3.5 border-b border-white/10 flex items-center justify-between">
            <span class="text-xs font-display font-bold uppercase tracking-wider text-white">
              🏆 Claimed Player Standings (Tier 3 Opt-In)
            </span>
            <span class="text-[10px] font-mono text-neutral-400">Sorted by Wilson Rating</span>
          </div>
          <table class="w-full text-left border-collapse">
            <thead>
              <tr class="border-b border-white/10 text-[10px] font-mono uppercase tracking-wider text-neutral-400 bg-black/40">
                <th class="py-2.5 px-4">#</th>
                <th class="py-2.5 px-3">Claimed Arena Handle</th>
                <th class="py-2.5 px-3">Rank Tier</th>
                <th class="py-2.5 px-3">Primary Format</th>
                <th class="py-2.5 px-3 text-right">Record</th>
                <th class="py-2.5 px-3 text-right">Win Rate</th>
                <th class="py-2.5 px-4 text-right">Rating</th>
              </tr>
            </thead>
            <tbody id="leaderboard-players-body" class="divide-y divide-white/5 text-xs font-mono"></tbody>
          </table>
        </div>

        <!-- Combat Hall of Fame (5 cols) -->
        <div class="lg:col-span-5 glass-panel overflow-x-auto">
          <div class="p-3.5 border-b border-white/10 flex items-center justify-between">
            <span class="text-xs font-display font-bold uppercase tracking-wider text-amber-300">
              ⚡ Combat Hall of Fame (Single-Match Records)
            </span>
          </div>
          <div id="leaderboard-combat-body" class="divide-y divide-white/5 p-3 space-y-2.5"></div>
        </div>
      </div>
    </section>
  </main>

  <script>
    const state = {
      activeTab: 'archetypes',
      selectedFormat: '',
      formatCategory: '',
      activeFormatsOnly: false,
      formats: [],
      archetypes: [],
      selectedArchetypeIdx: 0,
      selectedVariantIdx: -1, // -1 = Core + Flex overview, >=0 = specific variant
      commanders: [],
      cards: [],
      leaderboards: { players: [], combat_records: [] },
    };

    const pct = (v) => `${((v || 0) * 100).toFixed(1)}%`;
    const signedPct = (v) => {
      const n = (v || 0) * 100;
      return `${n >= 0 ? '+' : ''}${n.toFixed(1)}%`;
    };
    const esc = (s) => String(s ?? '')
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');

    // Authentic mana-font WUBRG pip renderer
    function renderColorPips(ci) {
      const raw = String(ci || 'C').toUpperCase().replace(/[^WUBRGC]/g, '') || 'C';
      return raw.split('').map(ch => {
        const lower = ch.toLowerCase();
        return `<i class="ms ms-${esc(lower)} ms-cost ms-shadow text-[13px] mx-[1px]" title="Mana Identity: ${esc(ch)}"></i>`;
      }).join('');
    }

    // Keyrune icon helper for format categories
    function formatKeyruneIcon(format, category) {
      const f = String(format || '').toLowerCase();
      const c = String(category || '').toLowerCase();
      if (f.includes('commander') || f.includes('brawl') || c.includes('commander')) return 'ss-cmd text-amber-300';
      if (f.includes('draft') || f.includes('sealed') || c.includes('limited')) return 'ss-ody text-amber-200';
      if (f.includes('timeless') || f.includes('historic')) return 'ss-vma text-purple-300';
      if (f.includes('pioneer') || f.includes('explorer')) return 'ss-rtr text-sky-300';
      return 'ss-dfn text-emerald-300';
    }

    function isRealCardName(name) {
      if (!name) return false;
      const s = String(name).trim();
      return s.length > 0 && !s.startsWith('Card #') && !s.startsWith('Commander #') && !s.startsWith('Unknown ');
    }

    // Clean card name for Scryfall exact lookup (strips guild suffixes like "Atraxa, Grand Unifier (Four-Color)")
    function extractPrimaryCardName(label, arch) {
      if (arch) {
        const namedCore = (arch.core_cards || []).find(c => !c.is_basic_land && isRealCardName(c.card_name));
        const namedFlex = (arch.flex_cards || []).find(c => !c.is_basic_land && isRealCardName(c.card_name));
        if (arch.archetype_name && arch.archetype_name.includes(' (')) {
          const prefix = arch.archetype_name.split(' (')[0].trim();
          if (isRealCardName(prefix) && !prefix.includes('—')) return prefix;
        }
        if (namedCore) return namedCore.card_name;
        if (namedFlex) return namedFlex.card_name;
      }
      if (!label) return '';
      let s = String(label).trim();
      if (s.includes(' — ')) {
        s = s.split(' — ')[1].split(' / ')[0].trim();
      } else if (s.includes(' (')) {
        s = s.split(' (')[0].trim();
      }
      return isRealCardName(s) ? s : '';
    }

    function scryfallArtCropUrl(cardName) {
      if (!isRealCardName(cardName)) return '';
      const face = String(cardName).split(' // ')[0].trim();
      return `https://api.scryfall.com/cards/named?exact=${encodeURIComponent(face)}&format=image&version=art_crop`;
    }

    function scryfallNormalUrl(cardName) {
      if (!isRealCardName(cardName)) return '';
      const face = String(cardName).split(' // ')[0].trim();
      return `https://api.scryfall.com/cards/named?exact=${encodeURIComponent(face)}&format=image&version=normal`;
    }

    function renderWinRateBar(winRate, widthClass = 'w-20') {
      const pctNum = Math.max(0, Math.min(100, (winRate || 0) * 100));
      const barColor = pctNum >= 55 ? 'bg-emerald-400' : pctNum >= 48 ? 'bg-amber-400' : 'bg-rose-400';
      return `
        <div class="inline-flex flex-col items-end gap-1">
          <span>${pct(winRate)}</span>
          <div class="wr-bar-track ${widthClass} rounded-full">
            <div class="wr-bar-fill ${barColor}" style="width: ${pctNum.toFixed(1)}%"></div>
          </div>
        </div>
      `;
    }

    function tierBadgeClass(tier) {
      switch (tier) {
        case 'S': return 'bg-amber-500/20 border-amber-400/60 text-amber-300 rt-gold-glow';
        case 'A': return 'bg-emerald-500/20 border-emerald-400/50 text-emerald-300';
        case 'B': return 'bg-sky-500/20 border-sky-400/50 text-sky-300';
        case 'C': return 'bg-purple-500/20 border-purple-400/50 text-purple-300';
        default:  return 'bg-white/5 border-white/15 text-neutral-400';
      }
    }

    // Floating Scryfall Card Preview Tooltip
    const tooltip = document.getElementById('card-preview-tooltip');
    const tooltipImg = document.getElementById('card-preview-img');
    const tooltipCaption = document.getElementById('card-preview-caption');
    let currentPreviewName = '';

    tooltipImg.addEventListener('error', () => {
      tooltip.classList.add('hidden');
    });

    document.addEventListener('mouseover', (e) => {
      const target = e.target.closest('[data-card-name]');
      if (!target) return;
      const cname = target.getAttribute('data-card-name');
      if (!isRealCardName(cname)) return;
      if (currentPreviewName !== cname) {
        currentPreviewName = cname;
        tooltipImg.src = scryfallNormalUrl(cname);
        tooltipCaption.textContent = cname;
      }
      tooltip.classList.remove('hidden');
    });

    document.addEventListener('mousemove', (e) => {
      if (tooltip.classList.contains('hidden')) return;
      const x = Math.min(window.innerWidth - 260, e.clientX + 22);
      const y = Math.min(window.innerHeight - 360, Math.max(12, e.clientY - 130));
      tooltip.style.left = `${x}px`;
      tooltip.style.top = `${y}px`;
    });

    document.addEventListener('mouseout', (e) => {
      const target = e.target.closest('[data-card-name]');
      if (target) {
        tooltip.classList.add('hidden');
      }
    });

    async function fetchAllTelemetry() {
      try {
        const fmtParam = state.selectedFormat ? `format=${encodeURIComponent(state.selectedFormat)}` : '';
        const [healthRes, formatsRes, archRes, cmdRes, cardsRes, lbRes] = await Promise.all([
          fetch('/health').then(r => r.json()),
          fetch(`/api/v1/meta/formats?category=${encodeURIComponent(state.formatCategory)}&active_only=${state.activeFormatsOnly}`).then(r => r.json()),
          fetch(`/api/v1/meta/decks?${fmtParam}&min_matches=1`).then(r => r.json()),
          fetch(`/api/v1/meta/commanders?${fmtParam}&min_matches=1`).then(r => r.json()),
          fetch(`/api/v1/meta/cards?${fmtParam}&limit=150`).then(r => r.json()),
          fetch(`/api/v1/leaderboards?${fmtParam}&limit=50`).then(r => r.json()),
        ]);

        // Update KPI strip
        document.getElementById('kpi-total-matches').textContent = healthRes.total_matches ?? 0;
        document.getElementById('kpi-constructed-matches').textContent = healthRes.constructed_matches ?? 0;
        document.getElementById('kpi-limited-matches').textContent = healthRes.limited_matches ?? 0;
        document.getElementById('kpi-commander-matches').textContent = healthRes.commander_matches ?? 0;
        document.getElementById('kpi-contributors').textContent = healthRes.unique_contributors ?? 0;
        document.getElementById('server-status-text').textContent = `Online • v${healthRes.version || '0.1.0'}`;

        state.formats = formatsRes || [];
        state.archetypes = archRes || [];
        state.commanders = cmdRes || [];
        state.cards = cardsRes || [];
        state.leaderboards = lbRes || { players: [], combat_records: [] };

        populateFormatDropdown(state.formats);
        renderArchetypes();
        renderFormatsTable();
        renderCommandersTable();
        renderCardsTable();
        renderLeaderboards();
      } catch (err) {
        console.error('Failed to fetch telemetry:', err);
        document.getElementById('server-status-text').textContent = 'Offline / Error';
      }
    }

    function populateFormatDropdown(formats) {
      const sel = document.getElementById('global-format-select');
      if (sel.options.length > 1) return;
      formats.forEach(f => {
        const opt = document.createElement('option');
        opt.value = f.format;
        opt.textContent = f.total_matches > 0 ? `${f.format} (${f.total_matches})` : f.format;
        sel.appendChild(opt);
      });
    }

    function renderArchetypes() {
      const listEl = document.getElementById('archetype-list-container');
      const inspEl = document.getElementById('archetype-inspector');

      if (!state.archetypes.length) {
        listEl.innerHTML = `
          <div class="glass-panel p-6 text-center space-y-2">
            <div class="text-xs font-display text-amber-300 font-bold uppercase tracking-wider">No Archetype Clusters Yet</div>
            <p class="text-xs text-neutral-400">
              Enable <strong>Community Meta &amp; Cloud Sync</strong> in the desktop Rhystic Tracker Settings → Connections &amp; Cloud Sync tab and click <strong>Queue Match History for Sync</strong>.
            </p>
          </div>`;
        inspEl.innerHTML = `<div class="text-center py-12 text-xs font-mono text-neutral-500">No archetype selected.</div>`;
        return;
      }

      if (state.selectedArchetypeIdx >= state.archetypes.length) {
        state.selectedArchetypeIdx = 0;
      }

      listEl.innerHTML = state.archetypes.map((a, idx) => {
        const active = idx === state.selectedArchetypeIdx;
        const featuredCard = extractPrimaryCardName(a.archetype_name, a);
        const artUrl = scryfallArtCropUrl(featuredCard);
        const wrPct = Math.max(0, Math.min(100, (a.win_rate || 0) * 100));
        return `
          <div data-arch-idx="${idx}" class="arch-card art-crop-banner glass-panel glass-panel-hover p-3.5 cursor-pointer border ${active ? 'border-amber-400/80 rt-gold-glow' : 'border-white/10'}">
            ${artUrl ? `<img src="${esc(artUrl)}" alt="" loading="lazy" class="art-crop-img" onerror="this.style.display='none'" />` : ''}
            <div class="art-crop-overlay"></div>
            <div class="relative z-10">
              <div class="flex items-center justify-between gap-2">
                <div class="flex items-center gap-2 min-w-0">
                  <span class="px-2 py-0.5 text-[10px] font-mono font-bold border ${tierBadgeClass(a.tier)}">${esc(a.tier)}</span>
                  <span class="text-xs font-display font-bold text-white truncate" ${featuredCard ? `data-card-name="${esc(featuredCard)}"` : ''}>${esc(a.archetype_name)}</span>
                </div>
                <div class="flex items-center gap-0.5 shrink-0">${renderColorPips(a.color_identity)}</div>
              </div>
              <div class="mt-2.5 flex flex-wrap items-center justify-between gap-2 text-[11px] font-mono text-neutral-300">
                <span><i class="ss ${formatKeyruneIcon(a.format, '')} mr-1"></i><strong class="text-white">${esc(a.format)}</strong></span>
                <span>Matches: <strong class="text-white">${a.matches}</strong> (${a.wins}W-${a.matches - a.wins}L)</span>
                <span>WR: <strong class="${a.win_rate >= 0.5 ? 'text-emerald-300' : 'text-amber-300'}">${pct(a.win_rate)}</strong></span>
              </div>
              <div class="mt-2 wr-bar-track w-full rounded-full">
                <div class="wr-bar-fill ${a.win_rate >= 0.5 ? 'bg-emerald-400' : 'bg-amber-400'}" style="width: ${wrPct.toFixed(1)}%"></div>
              </div>
              <div class="mt-1.5 flex items-center justify-between text-[10px] font-mono text-neutral-400">
                <span>Core: ${a.core_cards.length} • Flex: ${a.flex_cards.length}</span>
                <span>Variants: ${a.variants.length} • Avg Turns: ${a.avg_turns}</span>
              </div>
            </div>
          </div>
        `;
      }).join('');

      listEl.querySelectorAll('.arch-card').forEach(el => {
        el.addEventListener('click', () => {
          state.selectedArchetypeIdx = parseInt(el.getAttribute('data-arch-idx'), 10) || 0;
          state.selectedVariantIdx = -1;
          renderArchetypes();
        });
      });

      renderArchetypeInspector(state.archetypes[state.selectedArchetypeIdx]);
    }

    function formatMtgaDeckClipboard(arch, activeVariant) {
      const fmtLower = String(arch.format || '').toLowerCase();
      const isCmdOrBrawl = fmtLower.includes('commander') || fmtLower.includes('brawl');
      let commanderLine = '';
      if (isCmdOrBrawl && arch.archetype_name && arch.archetype_name.includes(' (')) {
        const maybeCmd = arch.archetype_name.split(' (')[0].trim();
        if (isRealCardName(maybeCmd)) {
          commanderLine = maybeCmd;
        }
      }

      const rawCards = activeVariant
        ? activeVariant.cards
        : arch.core_cards.concat(arch.flex_cards.slice(0, 8)).map(c => ({
            count: Math.max(1, Math.round(c.avg_copies || 1)),
            card_name: c.card_name || `Card #${c.grp_id}`,
          }));

      const deckLines = rawCards
        .filter(c => {
          const nm = c.card_name || `Card #${c.grp_id}`;
          return !commanderLine || nm.toLowerCase() !== commanderLine.toLowerCase();
        })
        .map(c => `${c.count} ${c.card_name || ('Card #' + c.grp_id)}`);

      if (commanderLine) {
        return ['Commander', `1 ${commanderLine}`, '', 'Deck'].concat(deckLines).join('\n');
      }
      return ['Deck'].concat(deckLines).join('\n');
    }

    function renderArchetypeInspector(arch) {
      const inspEl = document.getElementById('archetype-inspector');
      if (!arch) return;

      const featuredCard = extractPrimaryCardName(arch.archetype_name, arch);
      const artUrl = scryfallArtCropUrl(featuredCard);

      const variantPills = [
        `<button data-var-idx="-1" class="var-pill px-2.5 py-1 text-[10px] font-mono font-bold uppercase border cursor-pointer transition ${state.selectedVariantIdx === -1 ? 'border-amber-400 bg-amber-500/20 text-amber-300' : 'border-white/15 bg-white/[0.03] text-neutral-300 hover:border-white/30'}">
          ⚡ Unified Core + Flex Delta (${arch.matches} matches)
        </button>`
      ].concat((arch.variants || []).map((v, i) => `
        <button data-var-idx="${i}" class="var-pill px-2.5 py-1 text-[10px] font-mono font-bold border cursor-pointer transition ${state.selectedVariantIdx === i ? 'border-sky-400 bg-sky-500/20 text-sky-300' : 'border-white/15 bg-white/[0.03] text-neutral-300 hover:border-white/30'}">
          Variant #${i + 1} (${esc(v.deck_hash.slice(0, 8))} • ${pct(v.win_rate)} in ${v.matches}m)
        </button>
      `)).join('');

      const activeVariant = state.selectedVariantIdx >= 0 ? arch.variants[state.selectedVariantIdx] : null;
      const variantCardSet = new Set(activeVariant ? activeVariant.cards.map(c => c.grp_id) : []);

      const coreRows = (arch.core_cards || []).map(c => {
        const displayName = c.card_name || `Card #${c.grp_id}`;
        const dimClass = activeVariant && !variantCardSet.has(c.grp_id) ? 'opacity-35' : '';
        const incPct = Math.max(0, Math.min(100, (c.inclusion_rate || 0) * 100));
        return `
          <tr class="border-b border-white/5 hover:bg-white/[0.04] ${dimClass}">
            <td class="py-2 px-2.5 font-sans text-xs text-white cursor-pointer" data-card-name="${esc(displayName)}">
              <span class="font-mono text-emerald-300 font-bold mr-1.5">${c.avg_copies}x</span>
              <span class="hover:text-amber-300 transition-colors">${esc(displayName)}</span>
              ${c.is_basic_land ? '<span class="ml-1 text-[9px] font-mono uppercase text-neutral-500">[Basic]</span>' : ''}
            </td>
            <td class="py-2 px-2 text-right font-mono text-[11px] text-neutral-300">
              <div>${pct(c.inclusion_rate)}</div>
              <div class="wr-bar-track w-12 ml-auto mt-0.5 rounded-full">
                <div class="wr-bar-fill bg-emerald-400/80" style="width: ${incPct.toFixed(0)}%"></div>
              </div>
            </td>
            <td class="py-2 px-2.5 text-right font-mono text-[11px] ${c.win_rate_when_included >= 0.5 ? 'text-emerald-300 font-bold' : 'text-neutral-300'}">${pct(c.win_rate_when_included)}</td>
          </tr>
        `;
      }).join('');

      const flexRows = (arch.flex_cards || []).map(c => {
        const displayName = c.card_name || `Card #${c.grp_id}`;
        const inActiveVar = activeVariant && variantCardSet.has(c.grp_id);
        const highlightClass = inActiveVar
          ? 'bg-sky-500/10 border-l-2 border-l-sky-400'
          : activeVariant
          ? 'opacity-40'
          : '';
        const deltaColor = c.win_rate_delta > 0.005
          ? 'text-emerald-300 font-bold'
          : c.win_rate_delta < -0.005
          ? 'text-rose-400'
          : 'text-neutral-400';
        const incPct = Math.max(0, Math.min(100, (c.inclusion_rate || 0) * 100));
        return `
          <tr class="border-b border-white/5 hover:bg-white/[0.04] ${highlightClass}">
            <td class="py-2 px-2.5 font-sans text-xs text-white cursor-pointer" data-card-name="${esc(displayName)}">
              <span class="font-mono text-sky-300 font-bold mr-1.5">${c.avg_copies}x</span>
              <span class="hover:text-amber-300 transition-colors">${esc(displayName)}</span>
            </td>
            <td class="py-2 px-2 text-right font-mono text-[11px] text-neutral-300">
              <div>${pct(c.inclusion_rate)}</div>
              <div class="wr-bar-track w-12 ml-auto mt-0.5 rounded-full">
                <div class="wr-bar-fill bg-sky-400/80" style="width: ${incPct.toFixed(0)}%"></div>
              </div>
            </td>
            <td class="py-2 px-2 text-right font-mono text-[11px] text-white">${pct(c.win_rate_when_included)}</td>
            <td class="py-2 px-2.5 text-right font-mono text-[11px] ${deltaColor}">${signedPct(c.win_rate_delta)}</td>
          </tr>
        `;
      }).join('');

      inspEl.innerHTML = `
        <!-- Scryfall Art Crop Hero Banner -->
        <div class="art-crop-banner p-5 border-b border-white/10">
          ${artUrl ? `<img src="${esc(artUrl)}" alt="" loading="lazy" class="art-crop-img" onerror="this.style.display='none'" />` : ''}
          <div class="art-crop-overlay"></div>
          <div class="relative z-10 flex flex-wrap items-start justify-between gap-4">
            <div class="flex items-center gap-3.5">
              ${artUrl ? `
                <div class="w-16 h-12 border border-amber-400/40 bg-black/60 overflow-hidden shrink-0 shadow-lg cursor-pointer" data-card-name="${esc(featuredCard)}">
                  <img src="${esc(artUrl)}" alt="${esc(featuredCard)}" class="w-full h-full object-cover" onerror="this.parentElement.style.display='none'" />
                </div>
              ` : ''}
              <div>
                <div class="flex flex-wrap items-center gap-2">
                  <span class="px-2 py-0.5 text-[10px] font-mono font-bold border ${tierBadgeClass(arch.tier)}">Tier ${esc(arch.tier)}</span>
                  <h3 class="text-base font-display font-bold text-white tracking-wide">${esc(arch.archetype_name)}</h3>
                  <span class="inline-flex items-center">${renderColorPips(arch.color_identity)}</span>
                </div>
                <p class="text-[11px] font-mono text-neutral-300 mt-1.5">
                  Format: <strong class="text-white">${esc(arch.format)}</strong> •
                  Win Rate: <strong class="text-emerald-300">${pct(arch.win_rate)}</strong> (${arch.wins}W-${arch.matches - arch.wins}L) •
                  Play: <strong class="text-sky-300">${pct(arch.play_win_rate)}</strong> / Draw: <strong class="text-amber-300">${pct(arch.draw_win_rate)}</strong> •
                  Wilson Rating: <strong class="text-amber-200">${pct(arch.wilson_lower_bound)}</strong>
                </p>
              </div>
            </div>
            <button id="copy-deck-btn" class="px-3.5 py-2 border border-amber-400/50 bg-amber-500/15 hover:bg-amber-500/25 text-[11px] font-mono font-bold uppercase tracking-wider text-amber-200 cursor-pointer transition shadow-md">
              📋 Copy Deck to MTGA
            </button>
          </div>
        </div>

        <div class="p-5 space-y-5">
          <!-- Instant Variant Switcher Bar -->
          <div class="space-y-1.5">
            <div class="text-[10px] font-mono uppercase tracking-wider text-neutral-400 font-bold">
              Instant Variant Switcher (Click to highlight exact 60/100-card configuration in-place):
            </div>
            <div class="flex flex-wrap gap-1.5">${variantPills}</div>
          </div>

          <!-- Side-by-Side Fixed Core vs. Flex-Slot Win-Rate Delta -->
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div class="border border-white/10 bg-black/35">
              <div class="p-2.5 border-b border-white/10 flex items-center justify-between bg-white/[0.02]">
                <span class="text-[11px] font-display font-bold uppercase tracking-wider text-emerald-300">
                  🔒 Fixed Archetype Core (≥70% Inclusion)
                </span>
                <span class="text-[10px] font-mono text-neutral-400">${arch.core_cards.length} cards</span>
              </div>
              <div class="max-h-96 overflow-y-auto">
                <table class="w-full text-left border-collapse">
                  <thead>
                    <tr class="border-b border-white/10 text-[9px] font-mono uppercase text-neutral-500">
                      <th class="py-1.5 px-2.5">Card</th>
                      <th class="py-1.5 px-2 text-right">In %</th>
                      <th class="py-1.5 px-2.5 text-right">Win %</th>
                    </tr>
                  </thead>
                  <tbody>
                    ${coreRows || '<tr><td colspan="3" class="py-6 text-center text-xs font-mono text-neutral-500">All cards in this cluster are currently in Flex (&lt;70% inclusion).</td></tr>'}
                  </tbody>
                </table>
              </div>
            </div>

            <div class="border border-white/10 bg-black/35">
              <div class="p-2.5 border-b border-white/10 flex items-center justify-between bg-white/[0.02]">
                <span class="text-[11px] font-display font-bold uppercase tracking-wider text-sky-300">
                  🧪 Flex-Slot Explorer (&lt;70% Inclusion)
                </span>
                <span class="text-[10px] font-mono text-neutral-400">Sorted by WR Delta</span>
              </div>
              <div class="max-h-96 overflow-y-auto">
                <table class="w-full text-left border-collapse">
                  <thead>
                    <tr class="border-b border-white/10 text-[9px] font-mono uppercase text-neutral-500">
                      <th class="py-1.5 px-2.5">Flex Card</th>
                      <th class="py-1.5 px-2 text-right">In %</th>
                      <th class="py-1.5 px-2 text-right">Win %</th>
                      <th class="py-1.5 px-2.5 text-right">Δ vs Base</th>
                    </tr>
                  </thead>
                  <tbody>
                    ${flexRows || '<tr><td colspan="4" class="py-6 text-center text-xs font-mono text-neutral-500">No flex-slot divergence yet (100% unified list).</td></tr>'}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        </div>
      `;

      inspEl.querySelectorAll('.var-pill').forEach(btn => {
        btn.addEventListener('click', () => {
          state.selectedVariantIdx = parseInt(btn.getAttribute('data-var-idx'), 10);
          renderArchetypeInspector(arch);
        });
      });

      const copyBtn = document.getElementById('copy-deck-btn');
      if (copyBtn) {
        copyBtn.addEventListener('click', () => {
          const text = formatMtgaDeckClipboard(arch, activeVariant);
          navigator.clipboard.writeText(text);
          copyBtn.textContent = '✓ Copied to MTGA Clipboard!';
          setTimeout(() => { copyBtn.textContent = '📋 Copy Deck to MTGA'; }, 1800);
        });
      }
    }

    function renderFormatsTable() {
      const tbody = document.getElementById('formats-table-body');
      tbody.innerHTML = state.formats.map(f => {
        const iconCls = formatKeyruneIcon(f.format, f.category);
        return `
          <tr class="hover:bg-white/[0.04]">
            <td class="py-2.5 px-4 font-bold text-white">
              <i class="ss ${iconCls} mr-2"></i>${esc(f.format)}
            </td>
            <td class="py-2.5 px-3 text-neutral-400">${esc(f.category)}</td>
            <td class="py-2.5 px-3 text-right font-bold ${f.total_matches > 0 ? 'text-white' : 'text-neutral-600'}">${f.total_matches}</td>
            <td class="py-2.5 px-3 text-right ${f.total_matches > 0 ? 'text-emerald-300 font-bold' : 'text-neutral-600'}">
              ${f.total_matches > 0 ? renderWinRateBar(f.win_rate, 'w-16') : '—'}
            </td>
            <td class="py-2.5 px-3 text-right">
              ${f.total_matches > 0 ? `<span class="text-sky-300">${pct(f.play_win_rate)}</span> <span class="text-neutral-500">/</span> <span class="text-amber-300">${pct(f.draw_win_rate)}</span>` : '—'}
            </td>
            <td class="py-2.5 px-3 text-right text-neutral-300">${f.total_matches > 0 ? f.avg_turns : '—'}</td>
            <td class="py-2.5 px-4 text-right text-rose-300">${f.total_matches > 0 ? pct(f.concede_rate) : '—'}</td>
          </tr>
        `;
      }).join('');
    }

    function renderCommandersTable() {
      const tbody = document.getElementById('commanders-table-body');
      if (!state.commanders.length) {
        tbody.innerHTML = `<tr><td colspan="8" class="py-8 text-center text-neutral-500">No Commander or Brawl matches recorded for this filter yet.</td></tr>`;
        return;
      }
      tbody.innerHTML = state.commanders.map(c => {
        const cname = c.commander_name || `Commander #${c.commander_grp_id}`;
        const artUrl = scryfallArtCropUrl(cname);
        return `
          <tr data-cmd-id="${c.commander_grp_id}" data-cmd-name="${esc(cname)}" class="cmd-row hover:bg-white/[0.04] cursor-pointer transition">
            <td class="py-2.5 px-4"><span class="px-2 py-0.5 text-[10px] font-bold border ${tierBadgeClass(c.tier)}">${esc(c.tier)}</span></td>
            <td class="py-2.5 px-3 font-sans font-bold text-white" data-card-name="${esc(cname)}">
              <div class="flex items-center gap-2.5">
                ${artUrl ? `<img src="${esc(artUrl)}" alt="" loading="lazy" class="w-9 h-7 object-cover border border-white/15 shrink-0" onerror="this.style.display='none'" />` : ''}
                <span class="hover:text-amber-300 transition-colors">${esc(cname)}</span>
              </div>
            </td>
            <td class="py-2.5 px-3 text-neutral-400">${esc(c.format)}</td>
            <td class="py-2.5 px-3 text-right text-white">${c.matches}</td>
            <td class="py-2.5 px-3 text-right font-bold text-emerald-300">${renderWinRateBar(c.win_rate, 'w-16')}</td>
            <td class="py-2.5 px-3 text-right text-neutral-300">${pct(c.play_win_rate)} / ${pct(c.draw_win_rate)}</td>
            <td class="py-2.5 px-3 text-right text-neutral-300">${c.avg_turns}</td>
            <td class="py-2.5 px-4 text-right text-rose-300">${pct(c.salt_index)}</td>
          </tr>
        `;
      }).join('');

      tbody.querySelectorAll('.cmd-row').forEach(row => {
        row.addEventListener('click', async () => {
          const cid = row.getAttribute('data-cmd-id');
          const cname = row.getAttribute('data-cmd-name');
          const artUrl = scryfallArtCropUrl(cname);
          const panel = document.getElementById('commander-matchups-panel');
          panel.innerHTML = `<div class="p-4 text-xs font-mono text-neutral-400">Loading matchups for ${esc(cname)}…</div>`;
          const res = await fetch(`/api/v1/meta/matchups?commander_grp_id=${encodeURIComponent(cid)}`).then(r => r.json());
          if (!res.length) {
            panel.innerHTML = `
              <div class="art-crop-banner p-4 border-b border-white/10">
                ${artUrl ? `<img src="${esc(artUrl)}" alt="" class="art-crop-img" onerror="this.style.display='none'" />` : ''}
                <div class="art-crop-overlay"></div>
                <h3 class="relative z-10 text-xs font-display font-bold uppercase tracking-wider text-white">${esc(cname)} — Matchups</h3>
              </div>
              <div class="p-4 text-xs font-mono text-neutral-400">No opposing Commander head-to-head records yet.</div>`;
            return;
          }
          panel.innerHTML = `
            <div class="art-crop-banner p-4 border-b border-white/10">
              ${artUrl ? `<img src="${esc(artUrl)}" alt="" class="art-crop-img" onerror="this.style.display='none'" />` : ''}
              <div class="art-crop-overlay"></div>
              <div class="relative z-10 flex items-center justify-between gap-2">
                <h3 class="text-xs font-display font-bold uppercase tracking-wider text-amber-300">${esc(cname)} — Head-to-Head</h3>
                <button id="copy-cmd-btn" data-cmd="${esc(cname)}" class="px-2 py-1 border border-amber-400/40 bg-amber-500/15 text-[9px] font-mono font-bold uppercase text-amber-200 cursor-pointer">
                  📋 Copy Commander
                </button>
              </div>
            </div>
            <div class="p-4 space-y-2">
              ${res.map(m => {
                const oppName = m.opponent_commander_name || `Commander #${m.opponent_commander_grp_id}`;
                const oppArt = scryfallArtCropUrl(oppName);
                return `
                  <div class="p-2.5 border border-white/10 bg-black/40 flex items-center justify-between gap-2 text-xs font-mono">
                    <div class="flex items-center gap-2 min-w-0">
                      ${oppArt ? `<img src="${esc(oppArt)}" alt="" loading="lazy" class="w-7 h-6 object-cover border border-white/15 shrink-0" onerror="this.style.display='none'" />` : ''}
                      <span class="text-white font-sans font-semibold truncate cursor-pointer hover:text-amber-300" data-card-name="${esc(oppName)}">vs ${esc(oppName)}</span>
                    </div>
                    <span class="${m.win_rate >= 0.5 ? 'text-emerald-300' : 'text-rose-300'} font-bold shrink-0">${pct(m.win_rate)} (${m.wins}W-${m.matches - m.wins}L)</span>
                  </div>
                `;
              }).join('')}
            </div>
          `;
          const copyCmdBtn = document.getElementById('copy-cmd-btn');
          if (copyCmdBtn) {
            copyCmdBtn.addEventListener('click', () => {
              const cmdName = copyCmdBtn.getAttribute('data-cmd') || cname;
              navigator.clipboard.writeText(`Commander\n1 ${cmdName}\n`);
              copyCmdBtn.textContent = '✓ Copied!';
              setTimeout(() => { copyCmdBtn.textContent = '📋 Copy Commander'; }, 1600);
            });
          }
        });
      });
    }

    function renderCardsTable() {
      const tbody = document.getElementById('cards-table-body');
      const q = (document.getElementById('card-search-input')?.value || '').trim().toLowerCase();
      const filtered = state.cards.filter(c => {
        if (!q) return true;
        const name = (c.card_name || '').toLowerCase();
        return name.includes(q) || String(c.grp_id).includes(q);
      });

      if (!filtered.length) {
        tbody.innerHTML = `<tr><td colspan="8" class="py-8 text-center text-neutral-500">No matching card telemetry found.</td></tr>`;
        return;
      }

      tbody.innerHTML = filtered.map(c => {
        const cname = c.card_name || `Card #${c.grp_id}`;
        const titles = (c.top_titles || []).map(t => `<span class="px-1.5 py-0.5 text-[10px] border border-amber-400/30 bg-amber-500/10 text-amber-300">${esc(t.title)} ×${t.count}</span>`).join(' ');
        return `
          <tr class="hover:bg-white/[0.04]">
            <td class="py-2.5 px-4 font-sans font-semibold text-white cursor-pointer hover:text-amber-300" data-card-name="${esc(cname)}">
              ${esc(cname)}
              <span class="text-[10px] font-mono text-neutral-500 ml-1">#${c.grp_id}</span>
            </td>
            <td class="py-2.5 px-3 text-right text-white">${c.matches_seen}</td>
            <td class="py-2.5 px-3 text-right font-bold ${c.seen_win_rate >= 0.5 ? 'text-emerald-300' : 'text-amber-300'}">${renderWinRateBar(c.seen_win_rate, 'w-16')}</td>
            <td class="py-2.5 px-3 text-right text-rose-300">${c.total_damage}</td>
            <td class="py-2.5 px-3 text-right font-bold text-amber-300">${c.max_single_hit}</td>
            <td class="py-2.5 px-3 text-right text-neutral-400">${c.combat_damage} / ${c.spell_damage}</td>
            <td class="py-2.5 px-3 text-right text-sky-300">${c.total_cards_drawn}</td>
            <td class="py-2.5 px-4 flex flex-wrap gap-1">${titles || '<span class="text-neutral-600">—</span>'}</td>
          </tr>
        `;
      }).join('');
    }

    function renderLeaderboards() {
      const pBody = document.getElementById('leaderboard-players-body');
      const cBody = document.getElementById('leaderboard-combat-body');
      const players = state.leaderboards.players || [];
      const records = state.leaderboards.combat_records || [];

      if (!players.length) {
        pBody.innerHTML = `
          <tr>
            <td colspan="7" class="py-8 px-4 text-center text-neutral-400">
              No players have opted into Tier 3 Public Leaderboard Identity yet. Enable <strong>Public Leaderboard Identity (Log-Detected Arena Handle)</strong> in Settings → Connections &amp; Cloud Sync and re-sync!
            </td>
          </tr>`;
      } else {
        pBody.innerHTML = players.map((p, idx) => `
          <tr class="hover:bg-white/[0.04]">
            <td class="py-2.5 px-4 font-bold text-amber-300">#${idx + 1}</td>
            <td class="py-2.5 px-3">
              <div class="flex items-center gap-2">
                <span class="font-sans font-bold text-white">${esc(p.player_handle)}</span>
                <span title="Pinned to owner Contributor Token on first sync" class="px-1.5 py-0.5 text-[9px] border border-emerald-400/40 bg-emerald-500/10 text-emerald-300">
                  ✓ Claimed (${esc(p.player_account_hash.slice(0, 8))})
                </span>
              </div>
            </td>
            <td class="py-2.5 px-3 text-amber-200">${esc(p.latest_rank || 'Claimed Contender')}</td>
            <td class="py-2.5 px-3 text-neutral-300">${esc(p.favorite_format)}</td>
            <td class="py-2.5 px-3 text-right text-white">${p.wins}W - ${p.matches - p.wins}L</td>
            <td class="py-2.5 px-3 text-right font-bold text-emerald-300">${renderWinRateBar(p.win_rate, 'w-14')}</td>
            <td class="py-2.5 px-4 text-right text-sky-300">${pct(p.wilson_lower_bound)}</td>
          </tr>
        `).join('');
      }

      if (!records.length) {
        cBody.innerHTML = `<div class="py-6 text-center text-xs font-mono text-neutral-500">No single-match combat records yet.</div>`;
      } else {
        cBody.innerHTML = records.slice(0, 12).map((r, idx) => {
          const cname = r.card_name || `Card #${r.grp_id}`;
          const artUrl = scryfallArtCropUrl(cname);
          return `
            <div class="art-crop-banner p-3 border border-white/10 bg-black/40 flex items-center justify-between gap-2">
              ${artUrl ? `<img src="${esc(artUrl)}" alt="" loading="lazy" class="art-crop-img" onerror="this.style.display='none'" />` : ''}
              <div class="art-crop-overlay"></div>
              <div class="relative z-10 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="text-[10px] font-mono font-bold text-amber-300">#${idx + 1}</span>
                  <span class="text-xs font-display font-bold text-white truncate cursor-pointer hover:text-amber-300" data-card-name="${esc(cname)}">${esc(cname)}</span>
                  <span class="text-[10px] font-mono text-neutral-400">(${esc(r.format)})</span>
                </div>
                <div class="text-[10px] font-mono text-neutral-300 mt-0.5">
                  Pilot: <strong class="text-emerald-300">${esc(r.player_handle || 'Anonymous Contributor')}</strong>
                  ${(r.titles && r.titles.length) ? ` • ${esc(r.titles.join(', '))}` : ''}
                </div>
              </div>
              <div class="relative z-10 text-right font-mono shrink-0">
                <div class="text-xs font-bold text-rose-300">${r.max_single_hit} Max Hit</div>
                <div class="text-[10px] text-neutral-300">${r.total_damage} Dmg • ${r.cards_drawn} Drawn</div>
              </div>
            </div>
          `;
        }).join('');
      }
    }

    // Wire Tab Switching
    document.querySelectorAll('.tab-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const tab = btn.getAttribute('data-tab');
        state.activeTab = tab;
        document.querySelectorAll('.tab-btn').forEach(b => {
          const active = b.getAttribute('data-tab') === tab;
          b.className = `tab-btn px-4 py-2.5 text-xs font-mono font-bold uppercase tracking-wider border-b-2 cursor-pointer whitespace-nowrap flex items-center gap-2 ${
            active ? 'border-amber-400 text-amber-300 bg-white/[0.04]' : 'border-transparent text-neutral-400 hover:text-white'
          }`;
        });
        document.querySelectorAll('.tab-panel').forEach(p => {
          p.classList.toggle('hidden', p.id !== `tab-${tab}`);
        });
      });
    });

    // Wire Format Filter & Category Buttons
    document.getElementById('global-format-select').addEventListener('change', (e) => {
      state.selectedFormat = e.target.value;
      state.selectedArchetypeIdx = 0;
      state.selectedVariantIdx = -1;
      fetchAllTelemetry();
    });

    document.getElementById('refresh-all-btn').addEventListener('click', () => {
      fetchAllTelemetry();
    });

    document.querySelectorAll('.fmt-cat-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        state.formatCategory = btn.getAttribute('data-cat') || '';
        document.querySelectorAll('.fmt-cat-btn').forEach(b => {
          const active = (b.getAttribute('data-cat') || '') === state.formatCategory;
          b.className = `fmt-cat-btn px-3 py-1 text-[10px] font-mono font-bold uppercase border cursor-pointer ${
            active ? 'border-amber-400/50 bg-amber-500/15 text-amber-300' : 'border-white/15 bg-white/[0.03] text-neutral-300'
          }`;
        });
        fetchAllTelemetry();
      });
    });

    document.getElementById('active-formats-only').addEventListener('change', (e) => {
      state.activeFormatsOnly = e.target.checked;
      fetchAllTelemetry();
    });

    document.getElementById('card-search-input').addEventListener('input', () => {
      renderCardsTable();
    });

    // Initial load
    fetchAllTelemetry();
  </script>
</body>
</html>
"##;
