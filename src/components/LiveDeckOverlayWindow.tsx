import React, { useState, useEffect, useMemo, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Minimize2, X, Eye, EyeOff, Sliders } from 'lucide-react';
import { RenderManaCost } from '../utils/manaUtils';
import { ManaPip } from './ManaPip';
import symbolIcon from '../assets/RhysticTrackerICON.svg';
import { WindowResizeHandles } from './WindowResizeHandles';
import type { LiveDeckCardEntry, LiveDeckSnapshot } from './LiveDeckTrackerView';

const STORAGE_POS_KEY = 'rhystic_overlay_position';
const STORAGE_SIZE_KEY = 'rhystic_overlay_expanded_size';
const STORAGE_OPACITY_KEY = 'rhystic_overlay_opacity';
const STORAGE_SORT_KEY = 'rhystic_overlay_sort_mode';
const STORAGE_HIDE_ZERO_KEY = 'rhystic_overlay_hide_zero';

const DEFAULT_EXPANDED_WIDTH = 260;
const DEFAULT_EXPANDED_HEIGHT = 440;
const SYMBOL_SIZE = 48;

export type OverlaySortMode = 'prob' | 'cmc' | 'name';

interface OverlayBounds {
  width: number;
  height: number;
}

interface OverlayPosition {
  x: number;
  y: number;
}

function loadSavedExpandedSize(): OverlayBounds {
  try {
    const raw = localStorage.getItem(STORAGE_SIZE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (
        typeof parsed?.width === 'number' &&
        typeof parsed?.height === 'number' &&
        parsed.width >= 250 &&
        parsed.height >= 200
      ) {
        return { width: Math.round(parsed.width), height: Math.round(parsed.height) };
      }
    }
  } catch {
    // Ignore parse errors
  }
  return { width: DEFAULT_EXPANDED_WIDTH, height: DEFAULT_EXPANDED_HEIGHT };
}

function loadSavedPosition(): OverlayPosition | null {
  try {
    const raw = localStorage.getItem(STORAGE_POS_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (typeof parsed?.x === 'number' && typeof parsed?.y === 'number') {
        return { x: Math.round(parsed.x), y: Math.round(parsed.y) };
      }
    }
  } catch {
    // Ignore parse errors
  }
  return null;
}

export async function triggerToggleDeckOverlay(): Promise<boolean> {
  const savedSize = loadSavedExpandedSize();
  const savedPos = loadSavedPosition();
  try {
    return await invoke<boolean>('toggle_deck_overlay', {
      x: savedPos ? savedPos.x : null,
      y: savedPos ? savedPos.y : null,
      width: savedSize.width,
      height: savedSize.height,
    });
  } catch (err) {
    console.error('Failed to toggle deck overlay:', err);
    return false;
  }
}

export const LiveDeckOverlayWindow: React.FC = () => {
  const [liveState, setLiveState] = useState<any>(null);
  const [collapsed, setCollapsed] = useState<boolean>(false);
  const [showOpacitySlider, setShowOpacitySlider] = useState<boolean>(false);

  const [opacity, setOpacity] = useState<number>(() => {
    const saved = parseFloat(localStorage.getItem(STORAGE_OPACITY_KEY) || '0.84');
    return Number.isFinite(saved) ? Math.min(0.95, Math.max(0.4, saved)) : 0.84;
  });

  const [sortMode, setSortMode] = useState<OverlaySortMode>(() => {
    const saved = localStorage.getItem(STORAGE_SORT_KEY);
    if (saved === 'prob' || saved === 'cmc' || saved === 'name') return saved;
    return 'prob';
  });

  const [hideDepleted, setHideDepleted] = useState<boolean>(() => {
    return localStorage.getItem(STORAGE_HIDE_ZERO_KEY) === 'true';
  });

  const expandedSizeRef = useRef<OverlayBounds>(loadSavedExpandedSize());
  const collapsedRef = useRef<boolean>(false);
  collapsedRef.current = collapsed;
  const lastStateJsonRef = useRef<string>('');

  // Symbol mode drag-vs-click state
  const pointerStateRef = useRef<{
    isDown: boolean;
    startX: number;
    startY: number;
    dragged: boolean;
  }>({
    isDown: false,
    startX: 0,
    startY: 0,
    dragged: false,
  });

  // Ensure transparent root classes on mount
  useEffect(() => {
    document.documentElement.classList.add('overlay-window');
    document.body.classList.add('overlay-window');
    return () => {
      document.documentElement.classList.remove('overlay-window');
      document.body.classList.remove('overlay-window');
    };
  }, []);

  // Hydrate live match state immediately + listen to `deck-tracker-updated` + fallback poll
  const fetchLiveState = useCallback(async () => {
    try {
      const state: any = await invoke('get_live_match_state');
      const nextJson = JSON.stringify(state ?? null);
      if (nextJson === lastStateJsonRef.current) return;
      lastStateJsonRef.current = nextJson;

      if (state && (state.is_active || state.just_completed)) {
        setLiveState({
          ...state,
          status: state.just_completed ? 'COMPLETED' : 'IN_MATCH',
        });
      } else {
        setLiveState(null);
      }
    } catch {
      // Ignore transient IPC errors
    }
  }, []);

  useEffect(() => {
    fetchLiveState();

    let unlistenFn: (() => void) | null = null;
    listen('deck-tracker-updated', () => {
      fetchLiveState();
    })
      .then((unlisten) => {
        unlistenFn = unlisten;
      })
      .catch(() => {});

    const interval = setInterval(fetchLiveState, 1000);
    return () => {
      clearInterval(interval);
      if (unlistenFn) unlistenFn();
    };
  }, [fetchLiveState]);

  // Persist window movement & custom expanded size via Tauri window listeners
  useEffect(() => {
    const appWindow = getCurrentWindow();
    let unlistenMove: (() => void) | null = null;
    let unlistenResize: (() => void) | null = null;

    appWindow
      .onMoved(async ({ payload: physPos }) => {
        try {
          const scale = await appWindow.scaleFactor();
          const lx = Math.round(physPos.x / scale);
          const ly = Math.round(physPos.y / scale);
          localStorage.setItem(STORAGE_POS_KEY, JSON.stringify({ x: lx, y: ly }));
        } catch {
          // Ignore
        }
      })
      .then((u) => {
        unlistenMove = u;
      })
      .catch(() => {});

    appWindow
      .onResized(async ({ payload: physSize }) => {
        if (collapsedRef.current) return;
        try {
          const scale = await appWindow.scaleFactor();
          const lw = Math.round(physSize.width / scale);
          const lh = Math.round(physSize.height / scale);
          if (lw >= 250 && lh >= 200) {
            expandedSizeRef.current = { width: lw, height: lh };
            localStorage.setItem(STORAGE_SIZE_KEY, JSON.stringify({ width: lw, height: lh }));
          }
        } catch {
          // Ignore
        }
      })
      .then((u) => {
        unlistenResize = u;
      })
      .catch(() => {});

    return () => {
      if (unlistenMove) unlistenMove();
      if (unlistenResize) unlistenResize();
    };
  }, []);

  // Capture current logical position and custom expanded size before collapsing to 48x48
  const handleCollapseToSymbol = useCallback(async () => {
    const appWindow = getCurrentWindow();
    let lx: number | null = null;
    let ly: number | null = null;

    try {
      const [physSize, physPos, scale] = await Promise.all([
        appWindow.innerSize(),
        appWindow.outerPosition(),
        appWindow.scaleFactor(),
      ]);
      const lw = Math.round(physSize.width / scale);
      const lh = Math.round(physSize.height / scale);
      if (lw >= 250 && lh >= 200) {
        expandedSizeRef.current = { width: lw, height: lh };
        localStorage.setItem(STORAGE_SIZE_KEY, JSON.stringify({ width: lw, height: lh }));
      }
      lx = Math.round(physPos.x / scale);
      ly = Math.round(physPos.y / scale);
      localStorage.setItem(STORAGE_POS_KEY, JSON.stringify({ x: lx, y: ly }));
    } catch {
      const savedPos = loadSavedPosition();
      if (savedPos) {
        lx = savedPos.x;
        ly = savedPos.y;
      }
    }

    setShowOpacitySlider(false);
    setCollapsed(true);

    try {
      await invoke('set_deck_overlay_bounds', {
        width: SYMBOL_SIZE,
        height: SYMBOL_SIZE,
        resizable: false,
        x: lx,
        y: ly,
      });
    } catch (err) {
      console.error('Failed to collapse deck overlay:', err);
    }
  }, []);

  // Restore user's custom expanded size and keep position anchored when expanding
  const handleExpandFromSymbol = useCallback(async () => {
    const appWindow = getCurrentWindow();
    let lx: number | null = null;
    let ly: number | null = null;

    try {
      const [physPos, scale] = await Promise.all([
        appWindow.outerPosition(),
        appWindow.scaleFactor(),
      ]);
      lx = Math.round(physPos.x / scale);
      ly = Math.round(physPos.y / scale);
      localStorage.setItem(STORAGE_POS_KEY, JSON.stringify({ x: lx, y: ly }));
    } catch {
      const savedPos = loadSavedPosition();
      if (savedPos) {
        lx = savedPos.x;
        ly = savedPos.y;
      }
    }

    const targetSize = expandedSizeRef.current || loadSavedExpandedSize();
    setCollapsed(false);

    try {
      await invoke('set_deck_overlay_bounds', {
        width: targetSize.width,
        height: targetSize.height,
        resizable: true,
        x: lx,
        y: ly,
      });
    } catch (err) {
      console.error('Failed to expand deck overlay:', err);
    }
  }, []);

  const handleCloseOverlay = useCallback(async () => {
    try {
      await getCurrentWindow().hide();
    } catch {
      // Fallback
    }
  }, []);

  const handleHeaderDragStart = (e: React.PointerEvent<HTMLDivElement>) => {
    if (e.button !== 0) return;
    getCurrentWindow().startDragging().catch(() => {});
  };

  // Symbol Mode: 4px threshold distinguishing drag from click-to-expand
  const handleSymbolPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (e.button !== 0) return;
    pointerStateRef.current = {
      isDown: true,
      startX: e.screenX,
      startY: e.screenY,
      dragged: false,
    };
  };

  const handleSymbolPointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    const st = pointerStateRef.current;
    if (!st.isDown || st.dragged) return;
    const dist = Math.hypot(e.screenX - st.startX, e.screenY - st.startY);
    if (dist >= 4) {
      st.dragged = true;
      st.isDown = false;
      getCurrentWindow().startDragging().catch(() => {});
    }
  };

  const handleSymbolPointerUp = () => {
    const st = pointerStateRef.current;
    if (!st.isDown) return;
    const wasDragged = st.dragged;
    st.isDown = false;
    st.dragged = false;
    if (!wasDragged) {
      handleExpandFromSymbol();
    }
  };

  const handleOpacityChange = (val: number) => {
    const clamped = Math.min(0.95, Math.max(0.4, val));
    setOpacity(clamped);
    localStorage.setItem(STORAGE_OPACITY_KEY, String(clamped));
  };

  const handleSortModeChange = (mode: OverlaySortMode) => {
    setSortMode(mode);
    localStorage.setItem(STORAGE_SORT_KEY, mode);
  };

  const toggleHideDepleted = () => {
    setHideDepleted((prev) => {
      const next = !prev;
      localStorage.setItem(STORAGE_HIDE_ZERO_KEY, String(next));
      return next;
    });
  };

  const deckTracker: LiveDeckSnapshot | null = liveState?.deck_tracker || null;
  const isMatchActive = !!liveState && (liveState.status === 'IN_MATCH' || liveState.status === 'COMPLETED');
  const landProb = deckTracker?.land_draw_probability ?? 0;
  const nonLandProb = deckTracker?.non_land_draw_probability ?? 0;

  // Deterministic sorting with required secondary and tertiary tie-breakers
  const sortedCards = useMemo(() => {
    const raw: LiveDeckCardEntry[] = deckTracker?.cards || [];
    const filtered = hideDepleted ? raw.filter((c) => c.remaining_count > 0) : raw;

    return [...filtered].sort((a, b) => {
      // Keep 0x drawn-out cards at the bottom when dimmed rather than hidden
      const aDepleted = a.remaining_count === 0 ? 1 : 0;
      const bDepleted = b.remaining_count === 0 ? 1 : 0;
      if (aDepleted !== bDepleted) return aDepleted - bDepleted;

      if (sortMode === 'prob') {
        // Primary: Draw % (Highest to Lowest)
        if (Math.abs(b.draw_probability - a.draw_probability) > 1e-6) {
          return b.draw_probability - a.draw_probability;
        }
        // Secondary tie-breaker: CMC (Lowest to Highest)
        if (a.cmc !== b.cmc) {
          return a.cmc - b.cmc;
        }
        // Tertiary tie-breaker: Card Name (A to Z)
        return a.name.localeCompare(b.name);
      }

      if (sortMode === 'cmc') {
        // Primary: CMC (Lowest to Highest)
        if (a.cmc !== b.cmc) {
          return a.cmc - b.cmc;
        }
        // Secondary tie-breaker: Card Name (A to Z)
        return a.name.localeCompare(b.name);
      }

      // sortMode === 'name': Card Name (A to Z)
      return a.name.localeCompare(b.name);
    });
  }, [deckTracker?.cards, hideDepleted, sortMode]);

  const renderCompactCostOrPips = (card: LiveDeckCardEntry) => {
    if (card.mana_cost && card.mana_cost.trim().length > 0) {
      return <RenderManaCost costStr={card.mana_cost} size={11} />;
    }
    const pips = card.color_identity?.length
      ? card.color_identity
      : card.colors?.length
      ? card.colors
      : [];
    if (pips.length > 0) {
      return (
        <div className="flex items-center gap-0.5 shrink-0">
          {pips.map((sym, i) => (
            <ManaPip key={i} symbol={sym} size={11} />
          ))}
        </div>
      );
    }
    if (card.is_land) {
      return <span className="ms ms-land text-amber-400/80 text-[10px] shrink-0" title="Land" />;
    }
    return null;
  };

  // 1. COLLAPSED FLOATING SYMBOL MODE (48x48 px)
  if (collapsed) {
    return (
      <div
        onPointerDown={handleSymbolPointerDown}
        onPointerMove={handleSymbolPointerMove}
        onPointerUp={handleSymbolPointerUp}
        title="Click to expand Live Deck Overlay • Drag to reposition"
        className={`w-[48px] h-[48px] rounded-full flex items-center justify-center cursor-pointer select-none relative overflow-hidden bg-slate-900/70 backdrop-blur-md shadow-2xl ${
          isMatchActive
            ? 'border-2 border-emerald-400'
            : 'border border-sky-400'
        }`}
      >
        <img
          src={symbolIcon}
          alt="Rhystic Overlay"
          draggable={false}
          className="w-7 h-7 object-contain pointer-events-none"
        />
        <span
          className={`absolute bottom-1 right-1 w-2 h-2 rounded-full pointer-events-none ${
            isMatchActive ? 'bg-emerald-400' : 'bg-sky-400'
          }`}
        />
      </div>
    );
  }

  // 2. EXPANDED OVERLAY MODE (~260x440 px default, user-resizable)
  return (
    <div
      style={{ opacity }}
      className="w-full h-full min-h-[200px] min-w-[250px] flex flex-col overflow-hidden select-none bg-slate-900/60 backdrop-blur-md text-white border border-white/10 rounded-lg shadow-2xl relative"
    >
      {/* Edge & Corner Resize Handles for Frameless Window */}
      <WindowResizeHandles />

      {/* DRAGGABLE HEADER */}
      <div
        data-tauri-drag-region
        onPointerDown={handleHeaderDragStart}
        className="flex items-center justify-between gap-1.5 px-2.5 py-1.5 border-b border-white/10 bg-slate-950/40 backdrop-blur-sm cursor-move shrink-0"
      >
        <div className="flex items-center gap-1.5 min-w-0 pointer-events-none">
          <img src={symbolIcon} alt="" className="w-4 h-4 object-contain shrink-0" />
          <span
            className={`w-1.5 h-1.5 rounded-full shrink-0 ${
              isMatchActive ? 'bg-emerald-400' : 'bg-neutral-500'
            }`}
          />
          <span className="text-[11px] font-display font-bold tracking-wider uppercase text-white truncate">
            {liveState?.player_deck_name || 'RHYSTIC DECK'}
          </span>
        </div>

        {/* Header Action Controls */}
        <div
          className="flex items-center gap-1 shrink-0"
          onPointerDown={(e) => e.stopPropagation()}
        >
          <button
            onClick={() => setShowOpacitySlider((prev) => !prev)}
            className={`p-1 rounded cursor-pointer transition-colors ${
              showOpacitySlider
                ? 'bg-sky-500/20 text-sky-300'
                : 'text-neutral-400 hover:text-white'
            }`}
            title={`Overlay Opacity (${Math.round(opacity * 100)}%)`}
          >
            <Sliders className="w-3 h-3" />
          </button>

          <button
            onClick={handleCollapseToSymbol}
            className="p-1 rounded text-neutral-400 hover:text-white cursor-pointer transition-colors"
            title="Collapse to floating Rhystic symbol (48×48)"
          >
            <Minimize2 className="w-3 h-3" />
          </button>

          <button
            onClick={handleCloseOverlay}
            className="p-1 rounded text-neutral-400 hover:text-rose-400 cursor-pointer transition-colors"
            title="Close Pop-Out Overlay"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* OPTIONAL COMPACT OPACITY SLIDER BAR */}
      {showOpacitySlider && (
        <div
          onPointerDown={(e) => e.stopPropagation()}
          className="flex items-center justify-between gap-2 px-2.5 py-1 bg-slate-950/60 border-b border-white/10 shrink-0 text-[10px] font-mono"
        >
          <span className="text-neutral-400 uppercase">Opacity</span>
          <input
            type="range"
            min={0.4}
            max={0.95}
            step={0.05}
            value={opacity}
            onChange={(e) => handleOpacityChange(parseFloat(e.target.value))}
            className="flex-1 h-1 accent-sky-400 cursor-pointer"
          />
          <span className="text-sky-300 font-bold w-8 text-right tabular-nums">
            {Math.round(opacity * 100)}%
          </span>
        </div>
      )}

      {/* TOP SUMMARY BAR: Land Draw % and Non-Land Draw % (Omit total library counter) */}
      <div className="grid grid-cols-2 gap-1.5 px-2 py-1.5 border-b border-white/10 bg-slate-950/30 shrink-0">
        {/* Land Draw % Pill */}
        <div className="border border-amber-500/25 bg-black/40 px-2 py-1 flex flex-col justify-between">
          <div className="flex items-center justify-between gap-1">
            <span className="text-[9px] font-sans font-bold uppercase tracking-wider text-amber-300/90 flex items-center gap-1">
              <span className="ms ms-land text-[10px] text-amber-400" />
              Land
            </span>
            <span className="text-xs font-mono font-black tabular-nums text-amber-300">
              {landProb.toFixed(1)}%
            </span>
          </div>
          <div className="w-full h-1 bg-white/10 mt-1 overflow-hidden">
            <div
              className="h-full bg-amber-400"
              style={{ width: `${Math.min(100, Math.max(0, landProb))}%` }}
            />
          </div>
        </div>

        {/* Non-Land Draw % Pill */}
        <div className="border border-emerald-500/25 bg-black/40 px-2 py-1 flex flex-col justify-between">
          <div className="flex items-center justify-between gap-1">
            <span className="text-[9px] font-sans font-bold uppercase tracking-wider text-emerald-300/90 flex items-center gap-1">
              <span className="ms ms-instant text-[10px] text-emerald-400" />
              Non-Land
            </span>
            <span className="text-xs font-mono font-black tabular-nums text-emerald-300">
              {nonLandProb.toFixed(1)}%
            </span>
          </div>
          <div className="w-full h-1 bg-white/10 mt-1 overflow-hidden">
            <div
              className="h-full bg-emerald-400"
              style={{ width: `${Math.min(100, Math.max(0, nonLandProb))}%` }}
            />
          </div>
        </div>
      </div>

      {/* SORT SELECTOR & 0x TOGGLE BAR */}
      <div className="flex items-center justify-between gap-1 px-2 py-1 bg-slate-950/40 border-b border-white/10 shrink-0 text-[9.5px] font-mono uppercase">
        <div className="flex items-center gap-1">
          <button
            onClick={() => handleSortModeChange('prob')}
            className={`px-1.5 py-0.5 border cursor-pointer font-bold ${
              sortMode === 'prob'
                ? 'bg-sky-500/25 text-sky-300 border-sky-400/40'
                : 'bg-slate-900/40 text-neutral-400 border-white/10 hover:bg-slate-800/60 hover:text-white'
            }`}
            title="Sort by Next-Draw % (tie-broken by CMC, then Name)"
          >
            Draw %
          </button>
          <button
            onClick={() => handleSortModeChange('cmc')}
            className={`px-1.5 py-0.5 border cursor-pointer font-bold ${
              sortMode === 'cmc'
                ? 'bg-sky-500/25 text-sky-300 border-sky-400/40'
                : 'bg-slate-900/40 text-neutral-400 border-white/10 hover:bg-slate-800/60 hover:text-white'
            }`}
            title="Sort by Mana Value / CMC (tie-broken by Name)"
          >
            CMC
          </button>
          <button
            onClick={() => handleSortModeChange('name')}
            className={`px-1.5 py-0.5 border cursor-pointer font-bold ${
              sortMode === 'name'
                ? 'bg-sky-500/25 text-sky-300 border-sky-400/40'
                : 'bg-slate-900/40 text-neutral-400 border-white/10 hover:bg-slate-800/60 hover:text-white'
            }`}
            title="Sort Alphabetically (A–Z)"
          >
            Name
          </button>
        </div>

        <button
          onClick={toggleHideDepleted}
          className={`flex items-center gap-1 px-1.5 py-0.5 border cursor-pointer font-bold ${
            hideDepleted
              ? 'bg-amber-500/20 text-amber-300 border-amber-500/40'
              : 'bg-white/[0.04] text-neutral-400 border-white/10 hover:bg-white/[0.08]'
          }`}
          title={hideDepleted ? 'Showing remaining cards only (click to dim 0x cards)' : 'Dimming 0x cards (click to hide 0x cards)'}
        >
          {hideDepleted ? <EyeOff className="w-2.5 h-2.5" /> : <Eye className="w-2.5 h-2.5" />}
          <span>{hideDepleted ? 'Hide 0x' : 'Dim 0x'}</span>
        </button>
      </div>

      {/* REMAINING CARD LIST (Single-line per card) */}
      <div
        style={{
          backfaceVisibility: 'hidden',
          WebkitBackfaceVisibility: 'hidden',
          transform: 'translateZ(0)',
          WebkitTransform: 'translateZ(0)',
        }}
        className="flex-1 min-h-0 overflow-y-auto divide-y divide-white/[0.05] bg-slate-950/30"
      >
        {!deckTracker?.has_deck ? (
          <div className="h-full flex flex-col items-center justify-center px-4 text-center">
            <img
              src={symbolIcon}
              alt=""
              className="w-8 h-8 object-contain opacity-25 mb-2 pointer-events-none"
            />
            <span className="text-[11px] font-sans font-bold uppercase tracking-wider text-neutral-300">
              Waiting for Match Deck
            </span>
            <span className="text-[10px] font-sans text-neutral-400 mt-1">
              Enter a match in MTG Arena to see live draw odds.
            </span>
          </div>
        ) : sortedCards.length === 0 ? (
          <div className="py-8 text-center text-[11px] font-sans italic text-neutral-400">
            No cards remaining in library.
          </div>
        ) : (
          sortedCards.map((card) => {
            const isDepleted = card.remaining_count === 0;
            const barWidth = Math.min(100, Math.max(0, card.draw_probability * 2.5));

            return (
              <div
                key={card.grp_id}
                style={{
                  backfaceVisibility: 'hidden',
                  WebkitBackfaceVisibility: 'hidden',
                  transform: 'translateZ(0)',
                  WebkitTransform: 'translateZ(0)',
                }}
                className={`relative flex items-center justify-between gap-1.5 px-2 h-6 overflow-hidden transition-colors ${
                  isDepleted ? 'opacity-35 grayscale bg-black/30' : 'bg-slate-900/40 hover:bg-white/[0.08]'
                }`}
              >
                {/* Subtle probability fill bar */}
                {!isDepleted && card.draw_probability > 0 && (
                  <div
                    className={`absolute inset-y-0 left-0 pointer-events-none ${
                      card.is_land ? 'bg-amber-500/[0.09]' : 'bg-sky-500/[0.09]'
                    }`}
                    style={{ width: `${barWidth}%` }}
                  />
                )}

                {/* Left: Mana Cost / Color Pip + Card Name */}
                <div className="relative z-10 flex items-center gap-1.5 min-w-0 flex-1">
                  <div className="flex items-center justify-start shrink-0 min-w-[26px]">
                    {renderCompactCostOrPips(card)}
                  </div>
                  <span
                    className={`font-sans text-[11px] font-semibold truncate ${
                      isDepleted ? 'line-through text-neutral-400' : 'text-neutral-100'
                    }`}
                    title={card.name}
                  >
                    {card.name}
                  </span>
                </div>

                {/* Right: Remaining Copies (x1, x2) + Next-Draw % */}
                <div className="relative z-10 flex items-center gap-2 shrink-0 font-mono tabular-nums">
                  <span
                    className={`text-[10.5px] font-bold ${
                      isDepleted ? 'text-neutral-500' : 'text-neutral-200'
                    }`}
                  >
                    x{card.remaining_count}
                  </span>
                  <span
                    className={`w-11 text-right text-[10.5px] font-black ${
                      isDepleted
                        ? 'text-neutral-500'
                        : card.draw_probability >= 10.0
                        ? 'text-emerald-300'
                        : card.draw_probability >= 5.0
                        ? 'text-sky-300'
                        : 'text-neutral-300'
                    }`}
                  >
                    {card.draw_probability.toFixed(1)}%
                  </span>
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};

export default LiveDeckOverlayWindow;

