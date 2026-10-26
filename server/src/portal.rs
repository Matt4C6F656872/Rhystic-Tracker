//! Phase 5: Rhystic Community Meta & Companion Web Portal (`GET /`).
//!
//! Styled to match the local Rhystic Tracker's dark obsidian glassmorphic aesthetic.
//! Includes:
//! 1. Format Matrix & Meta Overview (All MTGA Constructed, Commander/Brawl, and Limited formats; Alchemy excluded)
//! 2. Interactive Archetype & Core + Flex-Slot Explorer (side-by-side Core >=70% vs. Flex <70% Win-Rate Delta inspector + Variant Switcher)
//! 3. Commander & Brawl Tier List + Salt Index + Head-to-Head Matchup Inspector
//! 4. Card Performance & Micro-Combat Telemetry with Scryfall hover previews
//! 5. Global Log-Verified Player Leaderboards & Combat Hall of Fame

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
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500;600;700&display=swap" rel="stylesheet">
  <script>
    tailwind.config = {
      darkMode: 'class',
      theme: {
        extend: {
          fontFamily: {
            sans: ['Inter', 'system-ui', 'sans-serif'],
            mono: ['JetBrains Mono', 'monospace'],
          },
        }
      }
    }
  </script>
  <style>
    body {
      background-color: #090a0d;
      background-image:
        radial-gradient(circle at 15% 10%, rgba(16, 185, 129, 0.08), transparent 35%),
        radial-gradient(circle at 85% 15%, rgba(56, 189, 248, 0.07), transparent 35%),
        radial-gradient(circle at 50% 90%, rgba(245, 158, 11, 0.05), transparent 40%);
      color: #e5e7eb;
      min-height: 100vh;
    }
    .glass-panel {
      background: rgba(255, 255, 255, 0.025);
      border: 1px solid rgba(255, 255, 255, 0.09);
      backdrop-filter: blur(12px);
    }
    .glass-panel-hover:hover {
      background: rgba(255, 255, 255, 0.045);
      border-color: rgba(255, 255, 255, 0.18);
    }
    .mana-pip {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 18px;
      height: 18px;
      border-radius: 9999px;
      font-family: 'JetBrains Mono', monospace;
      font-size: 10px;
      font-weight: 700;
      border: 1px solid rgba(255,255,255,0.25);
    }
    .pip-W { background: #fef3c7; color: #78350f; }
    .pip-U { background: #bae6fd; color: #0c4a6e; }
    .pip-B { background: #cbd5e1; color: #0f172a; }
    .pip-R { background: #fecaca; color: #7f1d1d; }
    .pip-G { background: #bbf7d0; color: #14532d; }
    .pip-C { background: #e5e7eb; color: #374151; }
    ::-webkit-scrollbar { width: 6px; height: 6px; }
    ::-webkit-scrollbar-track { background: rgba(0,0,0,0.2); }
    ::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.15); }
  </style>
</head>
<body class="font-sans antialiased selection:bg-emerald-500/30 selection:text-emerald-200">
  <!-- Floating Scryfall Card Preview Tooltip -->
  <div id="card-preview-tooltip" class="fixed z-50 pointer-events-none hidden transition-opacity duration-150">
    <div class="p-1.5 bg-black/90 border border-white/20 shadow-2xl">
      <img id="card-preview-img" src="" alt="Card Preview" class="w-56 h-auto block" />
    </div>
  </div>

  <!-- Top Navigation Header -->
  <header class="sticky top-0 z-40 border-b border-white/10 bg-[#090a0d]/85 backdrop-blur-xl">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 py-3 flex flex-wrap items-center justify-between gap-4">
      <div class="flex items-center gap-3">
        <div class="w-9 h-9 border border-emerald-500/40 bg-emerald-500/10 flex items-center justify-center font-mono font-bold text-emerald-400 text-lg shadow-inner">
          R
        </div>
        <div>
          <div class="flex items-center gap-2">
            <h1 class="text-sm font-bold uppercase tracking-wider text-white">Rhystic Tracker</h1>
            <span class="px-2 py-0.5 text-[10px] font-mono font-bold uppercase tracking-wider bg-emerald-500/15 border border-emerald-500/40 text-emerald-300">
              Community Meta Portal
            </span>
          </div>
          <p class="text-[11px] font-mono text-neutral-400">
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
          <select id="global-format-select" class="px-3 py-1.5 bg-white/[0.04] border border-white/15 text-xs font-mono text-white focus:outline-none focus:border-emerald-400/60">
            <option value="">All Formats (Global)</option>
          </select>
        </div>
        <button id="refresh-all-btn" class="px-3 py-1.5 border border-white/15 bg-white/[0.04] hover:bg-white/[0.09] text-[11px] font-mono font-bold uppercase tracking-wider text-white transition cursor-pointer">
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
      <button data-tab="archetypes" class="tab-btn px-4 py-2.5 text-xs font-mono font-bold uppercase tracking-wider border-b-2 border-emerald-400 text-emerald-300 bg-white/[0.03] cursor-pointer whitespace-nowrap">
        ⚔️ Archetypes &amp; Flex-Slot Explorer
      </button>
      <button data-tab="formats" class="tab-btn px-4 py-2.5 text-xs font-mono font-bold uppercase tracking-wider border-b-2 border-transparent text-neutral-400 hover:text-white cursor-pointer whitespace-nowrap">
        📊 Format Matrix (Constructed &amp; Limited)
      </button>
      <button data-tab="commanders" class="tab-btn px-4 py-2.5 text-xs font-mono font-bold uppercase tracking-wider border-b-2 border-transparent text-neutral-400 hover:text-white cursor-pointer whitespace-nowrap">
        👑 Commander &amp; Brawl Tier List
      </button>
      <button data-tab="cards" class="tab-btn px-4 py-2.5 text-xs font-mono font-bold uppercase tracking-wider border-b-2 border-transparent text-neutral-400 hover:text-white cursor-pointer whitespace-nowrap">
        🃏 Card Performance &amp; Combat
      </button>
      <button data-tab="leaderboards" class="tab-btn px-4 py-2.5 text-xs font-mono font-bold uppercase tracking-wider border-b-2 border-transparent text-neutral-400 hover:text-white cursor-pointer whitespace-nowrap">
        🏆 Claimed Leaderboards &amp; Hall of Fame
      </button>
    </div>
  </header>

  <main class="max-w-7xl mx-auto px-4 sm:px-6 py-6 space-y-6">
    <!-- Top KPI Telemetry Strip -->
    <section class="grid grid-cols-2 sm:grid-cols-5 gap-3">
      <div class="glass-panel p-3.5">
        <div class="text-[10px] font-mono uppercase tracking-wider text-neutral-400">Total Synced Matches</div>
        <div id="kpi-total-matches" class="text-xl font-mono font-bold text-white mt-1">—</div>
      </div>
      <div class="glass-panel p-3.5">
        <div class="text-[10px] font-mono uppercase tracking-wider text-neutral-400">Constructed Matches</div>
        <div id="kpi-constructed-matches" class="text-xl font-mono font-bold text-cyan-300 mt-1">—</div>
      </div>
      <div class="glass-panel p-3.5">
        <div class="text-[10px] font-mono uppercase tracking-wider text-neutral-400">Limited (Draft/Sealed)</div>
        <div id="kpi-limited-matches" class="text-xl font-mono font-bold text-amber-300 mt-1">—</div>
      </div>
      <div class="glass-panel p-3.5">
        <div class="text-[10px] font-mono uppercase tracking-wider text-neutral-400">Commander &amp; Brawl</div>
        <div id="kpi-commander-matches" class="text-xl font-mono font-bold text-emerald-300 mt-1">—</div>
      </div>
      <div class="glass-panel p-3.5 col-span-2 sm:col-span-1">
        <div class="text-[10px] font-mono uppercase tracking-wider text-neutral-400">Unique Contributors</div>
        <div id="kpi-contributors" class="text-xl font-mono font-bold text-purple-300 mt-1">—</div>
      </div>
    </section>

    <!-- TAB 1: ARCHETYPES & INTERACTIVE CORE + FLEX-SLOT EXPLORER -->
    <section id="tab-archetypes" class="tab-panel space-y-4">
      <div class="glass-panel p-4 flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 class="text-xs font-mono font-bold uppercase tracking-wider text-emerald-300">
            Unified Archetype &amp; Flex-Slot Explorer
          </h2>
          <p class="text-xs text-neutral-400 mt-0.5">
            Select any archetype on the left to inspect its <strong class="text-white">Fixed Core (≥70% inclusion)</strong> side-by-side with its <strong class="text-emerald-300">Flex-Slot Win-Rate Delta (&lt;70% inclusion)</strong> and switch between exact deck variants in a single view—no multi-tab hunting required.
          </p>
        </div>
        <div class="flex items-center gap-2">
          <span class="px-2.5 py-1 border border-white/10 bg-black/40 text-[10px] font-mono text-neutral-300">
            Ranked by 95% Wilson Confidence Bound
          </span>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-12 gap-4 items-start">
        <!-- Left Column: Archetype Tier List (5 cols) -->
        <div class="lg:col-span-5 space-y-2" id="archetype-list-container">
          <div class="glass-panel p-6 text-center text-xs font-mono text-neutral-400">
            Loading archetype clusters…
          </div>
        </div>

        <!-- Right Column: Unified Core + Flex-Slot Inspector (7 cols) -->
        <div class="lg:col-span-7 glass-panel p-5 space-y-5" id="archetype-inspector">
          <div class="text-center py-12 text-xs font-mono text-neutral-400">
            Select an archetype cluster on the left to inspect its Core Staples, Flex-Slot Win-Rate Deltas, and Exact Variants.
          </div>
        </div>
      </div>
    </section>

    <!-- TAB 2: FORMAT MATRIX -->
    <section id="tab-formats" class="tab-panel hidden space-y-4">
      <div class="glass-panel p-4 flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 class="text-xs font-mono font-bold uppercase tracking-wider text-cyan-300">
            MTG Arena Format Matrix (All Constructed, Commander &amp; Limited)
          </h2>
          <p class="text-xs text-neutral-400 mt-0.5">
            Compare average game length, Play vs. Draw win-rate split, and Concession ("Salt Index") across every official Arena format (Alchemy excluded).
          </p>
        </div>
        <div class="flex flex-wrap items-center gap-1.5">
          <button data-cat="" class="fmt-cat-btn px-3 py-1 text-[10px] font-mono font-bold uppercase border border-emerald-400/50 bg-emerald-500/15 text-emerald-300 cursor-pointer">
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
            <input type="checkbox" id="active-formats-only" class="accent-emerald-500" />
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
              <th class="py-3 px-3 text-right">Play WR</th>
              <th class="py-3 px-3 text-right">Draw WR</th>
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
          <h2 class="text-xs font-mono font-bold uppercase tracking-wider text-amber-300">
            Commander &amp; Brawl Meta Tier List + Salt Index
          </h2>
          <p class="text-xs text-neutral-400 mt-0.5">
            Command-zone win rates, Play/Draw advantage, average turn clock, and Opponent Concession Rate ("Salt Index"). Click any Commander to inspect head-to-head matchups.
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

        <div class="lg:col-span-4 glass-panel p-4 space-y-3" id="commander-matchups-panel">
          <h3 class="text-xs font-mono font-bold uppercase tracking-wider text-white border-b border-white/10 pb-2">
            Head-to-Head Matchup Matrix
          </h3>
          <p class="text-xs font-mono text-neutral-400">
            Click a Commander row to view its head-to-head record against opposing Commanders.
          </p>
        </div>
      </div>
    </section>

    <!-- TAB 4: CARD PERFORMANCE & COMBAT TELEMETRY -->
    <section id="tab-cards" class="tab-panel hidden space-y-4">
      <div class="glass-panel p-4 flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 class="text-xs font-mono font-bold uppercase tracking-wider text-purple-300">
            Card Performance &amp; Micro-Combat Analytics
          </h2>
          <p class="text-xs text-neutral-400 mt-0.5">
            Game-In-Hand / Seen Win Rate, Total Damage Dealt, Biggest Single Hit, Cards Drawn, and Earned Combat Titles. Hover any named card for live Scryfall art.
          </p>
        </div>
        <div>
          <input id="card-search-input" type="text" placeholder="Filter by card name or grpId…" class="px-3 py-1.5 bg-white/[0.04] border border-white/15 text-xs font-mono text-white placeholder:text-neutral-500 focus:outline-none focus:border-purple-400" />
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
          <h2 class="text-xs font-mono font-bold uppercase tracking-wider text-amber-300">
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
            <span class="text-xs font-mono font-bold uppercase tracking-wider text-white">
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
            <span class="text-xs font-mono font-bold uppercase tracking-wider text-amber-300">
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
      .replace(/"/g, '&quot;');

    function renderColorPips(ci) {
      const raw = (ci || 'C').toUpperCase();
      return raw.split('').map(ch => `<span class="mana-pip pip-${esc(ch)}">${esc(ch)}</span>`).join(' ');
    }

    function tierBadgeClass(tier) {
      switch (tier) {
        case 'S': return 'bg-amber-500/20 border-amber-400/50 text-amber-300';
        case 'A': return 'bg-emerald-500/20 border-emerald-400/50 text-emerald-300';
        case 'B': return 'bg-cyan-500/20 border-cyan-400/50 text-cyan-300';
        case 'C': return 'bg-purple-500/20 border-purple-400/50 text-purple-300';
        default:  return 'bg-white/5 border-white/15 text-neutral-400';
      }
    }

    // Scryfall hover preview
    const tooltip = document.getElementById('card-preview-tooltip');
    const tooltipImg = document.getElementById('card-preview-img');
    let currentPreviewName = '';

    document.addEventListener('mouseover', (e) => {
      const target = e.target.closest('[data-card-name]');
      if (!target) return;
      const cname = target.getAttribute('data-card-name');
      if (!cname || cname.startsWith('Card #') || cname.startsWith('Commander #')) return;
      if (currentPreviewName !== cname) {
        currentPreviewName = cname;
        tooltipImg.src = `https://api.scryfall.com/cards/named?exact=${encodeURIComponent(cname)}&format=image&version=normal`;
      }
      tooltip.classList.remove('hidden');
    });

    document.addEventListener('mousemove', (e) => {
      if (tooltip.classList.contains('hidden')) return;
      const x = Math.min(window.innerWidth - 250, e.clientX + 20);
      const y = Math.min(window.innerHeight - 340, Math.max(12, e.clientY - 120));
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
      if (sel.options.length > 1) return; // already populated
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
            <div class="text-xs font-mono text-neutral-300 font-bold uppercase">No Archetype Clusters Yet</div>
            <p class="text-xs text-neutral-400">
              Enable <strong>Community Meta &amp; Cloud Sync</strong> in the desktop Rhystic Tracker Settings → Connection tab and click <strong>Queue Match History for Sync</strong>.
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
        return `
          <div data-arch-idx="${idx}" class="arch-card glass-panel glass-panel-hover p-3.5 cursor-pointer transition border ${active ? 'border-emerald-400/70 bg-emerald-500/[0.06]' : 'border-white/10'}">
            <div class="flex items-center justify-between gap-2">
              <div class="flex items-center gap-2 min-w-0">
                <span class="px-2 py-0.5 text-[10px] font-mono font-bold border ${tierBadgeClass(a.tier)}">${esc(a.tier)}</span>
                <span class="text-xs font-bold text-white truncate">${esc(a.archetype_name)}</span>
              </div>
              <div class="flex items-center gap-1 shrink-0">${renderColorPips(a.color_identity)}</div>
            </div>
            <div class="mt-2 flex flex-wrap items-center justify-between gap-2 text-[11px] font-mono text-neutral-400">
              <span>Format: <strong class="text-neutral-200">${esc(a.format)}</strong></span>
              <span>Matches: <strong class="text-white">${a.matches}</strong> (${a.wins}W-${a.matches - a.wins}L)</span>
              <span>Win Rate: <strong class="${a.win_rate >= 0.5 ? 'text-emerald-300' : 'text-amber-300'}">${pct(a.win_rate)}</strong></span>
            </div>
            <div class="mt-1.5 flex items-center justify-between text-[10px] font-mono text-neutral-500">
              <span>Core Cards: ${a.core_cards.length} • Flex Options: ${a.flex_cards.length}</span>
              <span>Variants: ${a.variants.length} • Avg Turns: ${a.avg_turns}</span>
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

    function renderArchetypeInspector(arch) {
      const inspEl = document.getElementById('archetype-inspector');
      if (!arch) return;

      const variantPills = [
        `<button data-var-idx="-1" class="var-pill px-2.5 py-1 text-[10px] font-mono font-bold uppercase border cursor-pointer ${state.selectedVariantIdx === -1 ? 'border-emerald-400 bg-emerald-500/20 text-emerald-300' : 'border-white/15 bg-white/[0.03] text-neutral-300'}">
          ⚡ Unified Core + Flex Delta (${arch.matches} matches)
        </button>`
      ].concat((arch.variants || []).map((v, i) => `
        <button data-var-idx="${i}" class="var-pill px-2.5 py-1 text-[10px] font-mono font-bold border cursor-pointer ${state.selectedVariantIdx === i ? 'border-cyan-400 bg-cyan-500/20 text-cyan-300' : 'border-white/15 bg-white/[0.03] text-neutral-300'}">
          Variant #${i + 1} (${esc(v.deck_hash.slice(0, 10))} • ${pct(v.win_rate)} in ${v.matches}m)
        </button>
      `)).join('');

      const activeVariant = state.selectedVariantIdx >= 0 ? arch.variants[state.selectedVariantIdx] : null;
      const variantCardSet = new Set(activeVariant ? activeVariant.cards.map(c => c.grp_id) : []);

      const coreRows = (arch.core_cards || []).map(c => {
        const displayName = c.card_name || `Card #${c.grp_id}`;
        const dimClass = activeVariant && !variantCardSet.has(c.grp_id) ? 'opacity-35' : '';
        return `
          <tr class="border-b border-white/5 hover:bg-white/[0.03] ${dimClass}">
            <td class="py-1.5 px-2.5 font-sans text-xs text-white cursor-pointer" data-card-name="${esc(displayName)}">
              <span class="font-mono text-emerald-300 font-bold mr-1.5">${c.avg_copies}x</span>
              ${esc(displayName)}
              ${c.is_basic_land ? '<span class="ml-1 text-[9px] font-mono uppercase text-neutral-500">[Basic]</span>' : ''}
            </td>
            <td class="py-1.5 px-2 text-right font-mono text-[11px] text-neutral-300">${pct(c.inclusion_rate)}</td>
            <td class="py-1.5 px-2.5 text-right font-mono text-[11px] ${c.win_rate_when_included >= 0.5 ? 'text-emerald-300' : 'text-neutral-300'}">${pct(c.win_rate_when_included)}</td>
          </tr>
        `;
      }).join('');

      const flexRows = (arch.flex_cards || []).map(c => {
        const displayName = c.card_name || `Card #${c.grp_id}`;
        const inActiveVar = activeVariant && variantCardSet.has(c.grp_id);
        const highlightClass = inActiveVar
          ? 'bg-cyan-500/10 border-l-2 border-l-cyan-400'
          : activeVariant
          ? 'opacity-40'
          : '';
        const deltaColor = c.win_rate_delta > 0.005
          ? 'text-emerald-300 font-bold'
          : c.win_rate_delta < -0.005
          ? 'text-rose-400'
          : 'text-neutral-400';
        return `
          <tr class="border-b border-white/5 hover:bg-white/[0.03] ${highlightClass}">
            <td class="py-1.5 px-2.5 font-sans text-xs text-white cursor-pointer" data-card-name="${esc(displayName)}">
              <span class="font-mono text-cyan-300 font-bold mr-1.5">${c.avg_copies}x</span>
              ${esc(displayName)}
            </td>
            <td class="py-1.5 px-2 text-right font-mono text-[11px] text-neutral-300">${pct(c.inclusion_rate)}</td>
            <td class="py-1.5 px-2 text-right font-mono text-[11px] text-white">${pct(c.win_rate_when_included)}</td>
            <td class="py-1.5 px-2.5 text-right font-mono text-[11px] ${deltaColor}">${signedPct(c.win_rate_delta)}</td>
          </tr>
        `;
      }).join('');

      inspEl.innerHTML = `
        <div class="flex flex-wrap items-start justify-between gap-3 border-b border-white/10 pb-3">
          <div>
            <div class="flex items-center gap-2">
              <span class="px-2 py-0.5 text-[10px] font-mono font-bold border ${tierBadgeClass(arch.tier)}">Tier ${esc(arch.tier)}</span>
              <h3 class="text-sm font-bold text-white">${esc(arch.archetype_name)}</h3>
              <span>${renderColorPips(arch.color_identity)}</span>
            </div>
            <p class="text-[11px] font-mono text-neutral-400 mt-1">
              Format: <strong class="text-white">${esc(arch.format)}</strong> •
              Win Rate: <strong class="text-emerald-300">${pct(arch.win_rate)}</strong> (${arch.wins}W-${arch.matches - arch.wins}L) •
              Play: <strong class="text-cyan-300">${pct(arch.play_win_rate)}</strong> / Draw: <strong class="text-amber-300">${pct(arch.draw_win_rate)}</strong> •
              Wilson Bound: <strong>${pct(arch.wilson_lower_bound)}</strong>
            </p>
          </div>
          <button id="copy-deck-btn" class="px-3 py-1.5 border border-emerald-400/40 bg-emerald-500/10 hover:bg-emerald-500/20 text-[10px] font-mono font-bold uppercase tracking-wider text-emerald-300 cursor-pointer">
            📋 Copy Decklist for MTGA
          </button>
        </div>

        <!-- Instant Variant Switcher Bar -->
        <div class="space-y-1.5">
          <div class="text-[10px] font-mono uppercase tracking-wider text-neutral-400 font-bold">
            Instant Variant Switcher (Click to highlight exact 60/100-card configuration in-place):
          </div>
          <div class="flex flex-wrap gap-1.5">${variantPills}</div>
        </div>

        <!-- Side-by-Side Fixed Core vs. Flex-Slot Win-Rate Delta -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div class="border border-white/10 bg-black/30">
            <div class="p-2.5 border-b border-white/10 flex items-center justify-between bg-white/[0.02]">
              <span class="text-[11px] font-mono font-bold uppercase text-emerald-300">
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

          <div class="border border-white/10 bg-black/30">
            <div class="p-2.5 border-b border-white/10 flex items-center justify-between bg-white/[0.02]">
              <span class="text-[11px] font-mono font-bold uppercase text-cyan-300">
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
          const cardsToExport = activeVariant
            ? activeVariant.cards
            : arch.core_cards.concat(arch.flex_cards.slice(0, 6)).map(c => ({
                count: Math.max(1, Math.round(c.avg_copies || 1)),
                card_name: c.card_name || `Card #${c.grp_id}`,
              }));
          const text = ['Deck']
            .concat(cardsToExport.map(c => `${c.count} ${c.card_name || ('Card #' + c.grp_id)}`))
            .join('\n');
          navigator.clipboard.writeText(text);
          copyBtn.textContent = '✓ Copied to Clipboard!';
          setTimeout(() => { copyBtn.textContent = '📋 Copy Decklist for MTGA'; }, 1800);
        });
      }
    }

    function renderFormatsTable() {
      const tbody = document.getElementById('formats-table-body');
      tbody.innerHTML = state.formats.map(f => `
        <tr class="hover:bg-white/[0.03]">
          <td class="py-2.5 px-4 font-bold text-white">${esc(f.format)}</td>
          <td class="py-2.5 px-3 text-neutral-400">${esc(f.category)}</td>
          <td class="py-2.5 px-3 text-right font-bold ${f.total_matches > 0 ? 'text-white' : 'text-neutral-600'}">${f.total_matches}</td>
          <td class="py-2.5 px-3 text-right ${f.total_matches > 0 ? 'text-emerald-300' : 'text-neutral-600'}">${f.total_matches > 0 ? pct(f.win_rate) : '—'}</td>
          <td class="py-2.5 px-3 text-right text-cyan-300">${f.total_matches > 0 ? pct(f.play_win_rate) : '—'}</td>
          <td class="py-2.5 px-3 text-right text-amber-300">${f.total_matches > 0 ? pct(f.draw_win_rate) : '—'}</td>
          <td class="py-2.5 px-3 text-right text-neutral-300">${f.total_matches > 0 ? f.avg_turns : '—'}</td>
          <td class="py-2.5 px-4 text-right text-rose-300">${f.total_matches > 0 ? pct(f.concede_rate) : '—'}</td>
        </tr>
      `).join('');
    }

    function renderCommandersTable() {
      const tbody = document.getElementById('commanders-table-body');
      if (!state.commanders.length) {
        tbody.innerHTML = `<tr><td colspan="8" class="py-8 text-center text-neutral-500">No Commander or Brawl matches recorded for this filter yet.</td></tr>`;
        return;
      }
      tbody.innerHTML = state.commanders.map(c => {
        const cname = c.commander_name || `Commander #${c.commander_grp_id}`;
        return `
          <tr data-cmd-id="${c.commander_grp_id}" data-cmd-name="${esc(cname)}" class="cmd-row hover:bg-white/[0.04] cursor-pointer">
            <td class="py-2.5 px-4"><span class="px-2 py-0.5 text-[10px] font-bold border ${tierBadgeClass(c.tier)}">${esc(c.tier)}</span></td>
            <td class="py-2.5 px-3 font-sans font-bold text-white" data-card-name="${esc(cname)}">${esc(cname)}</td>
            <td class="py-2.5 px-3 text-neutral-400">${esc(c.format)}</td>
            <td class="py-2.5 px-3 text-right text-white">${c.matches}</td>
            <td class="py-2.5 px-3 text-right font-bold text-emerald-300">${pct(c.win_rate)}</td>
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
          const panel = document.getElementById('commander-matchups-panel');
          panel.innerHTML = `<div class="text-xs font-mono text-neutral-400">Loading matchups for ${esc(cname)}…</div>`;
          const res = await fetch(`/api/v1/meta/matchups?commander_grp_id=${encodeURIComponent(cid)}`).then(r => r.json());
          if (!res.length) {
            panel.innerHTML = `
              <h3 class="text-xs font-mono font-bold uppercase tracking-wider text-white border-b border-white/10 pb-2">${esc(cname)} — Matchups</h3>
              <p class="text-xs font-mono text-neutral-400">No opposing Commander head-to-head records yet.</p>`;
            return;
          }
          panel.innerHTML = `
            <h3 class="text-xs font-mono font-bold uppercase tracking-wider text-white border-b border-white/10 pb-2">${esc(cname)} — Head-to-Head</h3>
            <div class="space-y-2">
              ${res.map(m => {
                const oppName = m.opponent_commander_name || `Commander #${m.opponent_commander_grp_id}`;
                return `
                  <div class="p-2.5 border border-white/10 bg-black/30 flex items-center justify-between text-xs font-mono">
                    <span class="text-white font-sans font-semibold" data-card-name="${esc(oppName)}">vs ${esc(oppName)}</span>
                    <span class="${m.win_rate >= 0.5 ? 'text-emerald-300' : 'text-rose-300'} font-bold">${pct(m.win_rate)} (${m.wins}W-${m.matches - m.wins}L)</span>
                  </div>
                `;
              }).join('')}
            </div>
          `;
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
          <tr class="hover:bg-white/[0.03]">
            <td class="py-2.5 px-4 font-sans font-semibold text-white cursor-pointer" data-card-name="${esc(cname)}">
              ${esc(cname)}
              <span class="text-[10px] font-mono text-neutral-500 ml-1">#${c.grp_id}</span>
            </td>
            <td class="py-2.5 px-3 text-right text-white">${c.matches_seen}</td>
            <td class="py-2.5 px-3 text-right font-bold ${c.seen_win_rate >= 0.5 ? 'text-emerald-300' : 'text-amber-300'}">${pct(c.seen_win_rate)}</td>
            <td class="py-2.5 px-3 text-right text-rose-300">${c.total_damage}</td>
            <td class="py-2.5 px-3 text-right font-bold text-amber-300">${c.max_single_hit}</td>
            <td class="py-2.5 px-3 text-right text-neutral-400">${c.combat_damage} / ${c.spell_damage}</td>
            <td class="py-2.5 px-3 text-right text-cyan-300">${c.total_cards_drawn}</td>
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
          <tr class="hover:bg-white/[0.03]">
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
            <td class="py-2.5 px-3 text-right font-bold text-emerald-300">${pct(p.win_rate)}</td>
            <td class="py-2.5 px-4 text-right text-cyan-300">${pct(p.wilson_lower_bound)}</td>
          </tr>
        `).join('');
      }

      if (!records.length) {
        cBody.innerHTML = `<div class="py-6 text-center text-xs font-mono text-neutral-500">No single-match combat records yet.</div>`;
      } else {
        cBody.innerHTML = records.slice(0, 12).map((r, idx) => {
          const cname = r.card_name || `Card #${r.grp_id}`;
          return `
            <div class="p-2.5 border border-white/10 bg-black/30 flex items-center justify-between gap-2">
              <div class="min-w-0">
                <div class="flex items-center gap-2">
                  <span class="text-[10px] font-mono font-bold text-amber-300">#${idx + 1}</span>
                  <span class="text-xs font-bold text-white truncate cursor-pointer" data-card-name="${esc(cname)}">${esc(cname)}</span>
                  <span class="text-[10px] font-mono text-neutral-400">(${esc(r.format)})</span>
                </div>
                <div class="text-[10px] font-mono text-neutral-400 mt-0.5">
                  Pilot: <strong class="text-emerald-300">${esc(r.player_handle || 'Anonymous Contributor')}</strong>
                  ${(r.titles && r.titles.length) ? ` • ${esc(r.titles.join(', '))}` : ''}
                </div>
              </div>
              <div class="text-right font-mono shrink-0">
                <div class="text-xs font-bold text-rose-300">${r.max_single_hit} Max Hit</div>
                <div class="text-[10px] text-neutral-400">${r.total_damage} Dmg • ${r.cards_drawn} Drawn</div>
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
          b.className = `tab-btn px-4 py-2.5 text-xs font-mono font-bold uppercase tracking-wider border-b-2 cursor-pointer whitespace-nowrap ${
            active ? 'border-emerald-400 text-emerald-300 bg-white/[0.03]' : 'border-transparent text-neutral-400 hover:text-white'
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
            active ? 'border-emerald-400/50 bg-emerald-500/15 text-emerald-300' : 'border-white/15 bg-white/[0.03] text-neutral-300'
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

