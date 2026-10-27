import React, { useState, useMemo } from 'react';
import { Search, X, Swords, ExternalLink } from 'lucide-react';
import { RenderManaCost } from '../utils/manaUtils';
import { ManaPip } from './ManaPip';
import { CardNameTooltip } from './CardNameTooltip';
import CardImage from './CardImage';
import { triggerToggleDeckOverlay } from './LiveDeckOverlayWindow';
import logoImg from '../assets/RhysticTrackerLogo.svg';
import symbolIcon from '../assets/RhysticTrackerICON.svg';

export interface LiveDeckCardEntry {
  grp_id: number;
  name: string;
  mana_cost: string;
  cmc: number;
  type_line: string;
  is_land: boolean;
  colors: string[];
  color_identity: string[];
  rarity: string;
  set_code: string;
  initial_count: number;
  remaining_count: number;
  outside_library_count: number;
  draw_probability: number;
}

export interface LiveDeckSnapshot {
  has_deck: boolean;
  deck_source: string;
  total_library_cards: number;
  initial_library_cards: number;
  total_remaining_lands: number;
  total_remaining_non_lands: number;
  land_draw_probability: number;
  non_land_draw_probability: number;
  cards: LiveDeckCardEntry[];
}

interface LiveDeckTrackerPanelProps {
  deckTracker?: LiveDeckSnapshot | null;
  deckName?: string;
  format?: string;
  palette?: any;
  compact?: boolean;
  searchQuery?: string;
  onShowCard?: (card: { name: string; grp_id?: number }, isCommander?: boolean) => void;
}

export const LiveDeckTrackerPanel: React.FC<LiveDeckTrackerPanelProps> = ({
  deckTracker,
  deckName,
  compact = false,
  searchQuery = '',
  onShowCard,
}) => {
  const [typeFilter, setTypeFilter] = useState<'all' | 'nonland' | 'land'>('all');

  const cards = useMemo(() => {
    const raw = deckTracker?.cards || [];
    const q = searchQuery.trim().toLowerCase();

    const filtered = raw.filter((c) => {
      if (typeFilter === 'land' && !c.is_land) return false;
      if (typeFilter === 'nonland' && c.is_land) return false;
      if (q) {
        return (
          c.name.toLowerCase().includes(q) ||
          (c.type_line && c.type_line.toLowerCase().includes(q))
        );
      }
      return true;
    });

    // Ensure depleted cards (0 remaining) always sit at the bottom, sorted by draw_probability DESC, cmc ASC, name ASC
    return [...filtered].sort((a, b) => {
      const aDepleted = a.remaining_count === 0 ? 1 : 0;
      const bDepleted = b.remaining_count === 0 ? 1 : 0;
      if (aDepleted !== bDepleted) return aDepleted - bDepleted;
      if (Math.abs(b.draw_probability - a.draw_probability) > 1e-6) {
        return b.draw_probability - a.draw_probability;
      }
      if (a.cmc !== b.cmc) return a.cmc - b.cmc;
      return a.name.localeCompare(b.name);
    });
  }, [deckTracker?.cards, searchQuery, typeFilter]);

  const totalLib = deckTracker?.total_library_cards ?? 0;
  const initialLib = deckTracker?.initial_library_cards ?? 0;
  const landProb = deckTracker?.land_draw_probability ?? 0;
  const nonLandProb = deckTracker?.non_land_draw_probability ?? 0;
  const remainingLands = deckTracker?.total_remaining_lands ?? 0;
  const remainingNonLands = deckTracker?.total_remaining_non_lands ?? 0;

  const renderCardCostOrPips = (card: LiveDeckCardEntry) => {
    if (card.mana_cost && card.mana_cost.trim().length > 0) {
      return <RenderManaCost costStr={card.mana_cost} size={compact ? 13 : 15} />;
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
            <ManaPip key={i} symbol={sym} size={compact ? 13 : 15} />
          ))}
        </div>
      );
    }
    if (card.is_land) {
      return (
        <span
          className="ms ms-land text-amber-400/80 text-xs shrink-0"
          title="Land"
        />
      );
    }
    return null;
  };

  return (
    <div className="flex-1 min-h-0 flex flex-col space-y-2.5 overflow-hidden select-none">
      {/* 1. TOP SUMMARY BAR */}
      <div className="grid grid-cols-3 gap-2.5 shrink-0">
        {/* Total Cards in Library */}
        <div className="border border-white/15 bg-neutral-950/70 backdrop-blur-md px-3.5 py-2.5 flex flex-col justify-between relative overflow-hidden">
          <div className="flex items-center justify-between gap-1.5">
            <span className="text-[10px] font-sans font-bold uppercase tracking-wider text-neutral-400">
              Library Remaining
            </span>
            <span className="ms ms-library text-xs text-sky-400" />
          </div>
          <div className="flex items-baseline gap-1.5 mt-1">
            <span className="text-xl font-mono font-black tabular-nums text-white">
              {totalLib} / {initialLib}
            </span>
            <span className="text-[10px] font-mono text-neutral-400 uppercase">
              cards
            </span>
          </div>
          <div className="w-full h-1 bg-white/10 mt-1.5 overflow-hidden">
            <div
              className="h-full bg-sky-400 transition-all duration-300"
              style={{
                width: `${initialLib > 0 ? Math.min(100, (totalLib / initialLib) * 100) : 0}%`,
              }}
            />
          </div>
        </div>

        {/* Land Draw % */}
        <div className="border border-amber-500/25 bg-neutral-950/70 backdrop-blur-md px-3.5 py-2.5 flex flex-col justify-between relative overflow-hidden">
          <div className="flex items-center justify-between gap-1.5">
            <span className="text-[10px] font-sans font-bold uppercase tracking-wider text-amber-300/90">
              Land Draw %
            </span>
            <span className="ms ms-land text-xs text-amber-400" />
          </div>
          <div className="flex items-baseline justify-between gap-1.5 mt-1">
            <span className="text-xl font-mono font-black tabular-nums text-amber-300">
              {landProb.toFixed(1)}%
            </span>
            <span className="text-[10px] font-mono tabular-nums text-neutral-400">
              {remainingLands} left
            </span>
          </div>
          <div className="w-full h-1 bg-white/10 mt-1.5 overflow-hidden">
            <div
              className="h-full bg-amber-400 transition-all duration-300"
              style={{ width: `${Math.min(100, Math.max(0, landProb))}%` }}
            />
          </div>
        </div>

        {/* Non-Land Draw % */}
        <div className="border border-emerald-500/25 bg-neutral-950/70 backdrop-blur-md px-3.5 py-2.5 flex flex-col justify-between relative overflow-hidden">
          <div className="flex items-center justify-between gap-1.5">
            <span className="text-[10px] font-sans font-bold uppercase tracking-wider text-emerald-300/90">
              Non-Land Draw %
            </span>
            <span className="ms ms-instant text-xs text-emerald-400" />
          </div>
          <div className="flex items-baseline justify-between gap-1.5 mt-1">
            <span className="text-xl font-mono font-black tabular-nums text-emerald-300">
              {nonLandProb.toFixed(1)}%
            </span>
            <span className="text-[10px] font-mono tabular-nums text-neutral-400">
              {remainingNonLands} left
            </span>
          </div>
          <div className="w-full h-1 bg-white/10 mt-1.5 overflow-hidden">
            <div
              className="h-full bg-emerald-400 transition-all duration-300"
              style={{ width: `${Math.min(100, Math.max(0, nonLandProb))}%` }}
            />
          </div>
        </div>
      </div>

      {/* Filter Pills + Column Headers */}
      <div className="flex items-center justify-between px-3 py-1.5 bg-neutral-950/60 border border-white/10 shrink-0 text-[10px] font-mono uppercase tracking-wider text-neutral-400">
        <div className="flex items-center gap-1.5">
          <button
            onClick={() => setTypeFilter('all')}
            className={`px-2 py-0.5 border transition-colors cursor-pointer font-bold ${
              typeFilter === 'all'
                ? 'bg-white/15 text-white border-white/30'
                : 'bg-transparent text-neutral-400 border-transparent hover:text-white'
            }`}
          >
            All ({deckTracker?.cards?.length || 0})
          </button>
          <button
            onClick={() => setTypeFilter('nonland')}
            className={`px-2 py-0.5 border transition-colors cursor-pointer font-bold ${
              typeFilter === 'nonland'
                ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/40'
                : 'bg-transparent text-neutral-400 border-transparent hover:text-emerald-300'
            }`}
          >
            Spells ({remainingNonLands})
          </button>
          <button
            onClick={() => setTypeFilter('land')}
            className={`px-2 py-0.5 border transition-colors cursor-pointer font-bold ${
              typeFilter === 'land'
                ? 'bg-amber-500/20 text-amber-300 border-amber-500/40'
                : 'bg-transparent text-neutral-400 border-transparent hover:text-amber-300'
            }`}
          >
            Lands ({remainingLands})
          </button>
        </div>

        <div className="flex items-center gap-4">
          {deckName && !compact && (
            <span className="text-neutral-300 font-sans font-bold truncate max-w-[200px]">
              {deckName}
            </span>
          )}
          <span className="w-14 text-right">Copies</span>
          <span className="w-16 text-right">Next Draw</span>
        </div>
      </div>

      {/* 2. SORTED DECK LIST */}
      <div className="flex-1 min-h-0 border border-white/10 bg-neutral-950/55 backdrop-blur-md overflow-y-auto custom-scrollbar divide-y divide-white/[0.05]">
        {cards.length === 0 ? (
          <div className="py-12 text-center text-xs font-sans italic text-neutral-400">
            {deckTracker?.has_deck
              ? 'No cards match the current filter.'
              : 'Waiting for match deck list from GRE connection...'}
          </div>
        ) : (
          cards.map((card) => {
            const isDepleted = card.remaining_count === 0;
            const probFormatted = `${card.draw_probability.toFixed(1)}%`;
            const barWidth = Math.min(100, Math.max(0, card.draw_probability * 2.5));

            return (
              <div
                key={card.grp_id}
                onClick={() => onShowCard?.({ name: card.name, grp_id: card.grp_id }, false)}
                className={`relative flex items-center justify-between gap-2.5 px-3 py-1.5 transition-all cursor-pointer group overflow-hidden ${
                  isDepleted
                    ? 'opacity-40 grayscale bg-black/40 hover:opacity-60'
                    : 'hover:bg-white/[0.06]'
                }`}
              >
                {/* Subtle probability fill bar along the bottom/background */}
                {!isDepleted && card.draw_probability > 0 && (
                  <div
                    className={`absolute inset-y-0 left-0 pointer-events-none transition-all duration-300 ${
                      card.is_land ? 'bg-amber-500/[0.07]' : 'bg-sky-500/[0.07]'
                    }`}
                    style={{ width: `${barWidth}%` }}
                  />
                )}

                {/* Left: Art thumbnail + Mana Cost / Color Icons + Card Name */}
                <div className="relative z-10 flex items-center gap-2.5 min-w-0 flex-1">
                  {/* Mini art crop */}
                  <div className="w-7 h-5 shrink-0 border border-white/15 overflow-hidden bg-neutral-900">
                    <CardImage
                      name={card.name}
                      setCode={card.set_code}
                      version="art_crop"
                      alt=""
                      className="w-full h-full object-cover"
                    />
                  </div>

                  {/* Mana Cost / Color Icons */}
                  <div className="flex items-center justify-end min-w-[44px] shrink-0">
                    {renderCardCostOrPips(card)}
                  </div>

                  {/* Card Name */}
                  <CardNameTooltip
                    name={card.name}
                    setCode={card.set_code}
                    className={`font-sans text-xs font-semibold truncate transition-colors ${
                      isDepleted
                        ? 'line-through text-neutral-400'
                        : 'text-neutral-100 group-hover:text-white'
                    }`}
                  >
                    {card.name}
                  </CardNameTooltip>

                  {card.is_land && !compact && (
                    <span className="text-[9px] font-mono uppercase px-1.5 py-0.2 bg-amber-500/10 border border-amber-500/30 text-amber-300/90 shrink-0">
                      Land
                    </span>
                  )}
                </div>

                {/* Right: Copies Remaining / Initial Copies & Next Draw % */}
                <div className="relative z-10 flex items-center gap-4 shrink-0 font-mono tabular-nums">
                  {/* Copies Remaining / Initial Copies (e.g., 3 / 4) */}
                  <span
                    className={`w-14 text-right text-xs font-bold ${
                      isDepleted ? 'text-neutral-500' : 'text-neutral-200'
                    }`}
                    title={`${card.remaining_count} remaining in library of ${card.initial_count} initial copies`}
                  >
                    {card.remaining_count} / {card.initial_count}
                  </span>

                  {/* Next Draw Probability formatted to 1 decimal place (e.g., 11.1%) */}
                  <span
                    className={`w-16 text-right text-xs font-black px-1.5 py-0.5 border ${
                      isDepleted
                        ? 'text-neutral-500 border-white/5 bg-black/30'
                        : card.draw_probability >= 10.0
                        ? 'text-emerald-300 border-emerald-500/30 bg-emerald-950/40'
                        : card.draw_probability >= 5.0
                        ? 'text-sky-300 border-sky-500/30 bg-sky-950/40'
                        : 'text-neutral-200 border-white/10 bg-white/[0.03]'
                    }`}
                  >
                    {probFormatted}
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

interface LiveDeckTrackerViewProps {
  palette?: any;
  liveMatchState?: any;
  onShowCard?: (card: { name: string; grp_id?: number }, isCommander?: boolean) => void;
  formatChipColor?: (format?: string) => { text: string; bg: string; border: string };
}

export const LiveDeckTrackerView: React.FC<LiveDeckTrackerViewProps> = ({
  palette,
  liveMatchState,
  onShowCard,
  formatChipColor,
}) => {
  const accentColor = palette?.accent || '#38BDF8';
  const [search, setSearch] = useState<string>('');

  const isMatchActive =
    !!liveMatchState &&
    (liveMatchState.status === 'IN_MATCH' || liveMatchState.status === 'COMPLETED');

  const deckTracker: LiveDeckSnapshot | null = liveMatchState?.deck_tracker || null;
  const hasDeckData = isMatchActive && !!deckTracker?.has_deck;

  const formatBadge = liveMatchState?.format
    ? formatChipColor
      ? formatChipColor(liveMatchState.format)
      : { text: '#F3F4F6', bg: 'rgba(255,255,255,0.06)', border: 'rgba(255,255,255,0.15)' }
    : null;

  return (
    <div className="flex-1 min-h-0 flex flex-col space-y-3 px-8 py-4 overflow-hidden select-none">
      {/* 1. HEADER */}
      <div className="flex items-center justify-between pb-2 border-b border-white/10 shrink-0">
        <div className="flex items-center gap-2.5">
          <span className="ms ms-library text-2xl" style={{ color: accentColor }} />
          <h1 className="text-[26px] font-display font-bold tracking-[0.12em] uppercase text-white leading-none">
            LIVE DECK TRACKER
          </h1>
          <span className="text-xs text-neutral-400 font-sans ml-2">
            {isMatchActive
              ? '(Real-time library & next-draw probabilities)'
              : '(Constructed, Draft, Sealed & Direct Challenge)'}
          </span>
        </div>
      </div>

      {/* 2. TOOLBAR */}
      <div className="shrink-0 flex items-center gap-2.5 pb-1 flex-wrap">
        <div className="relative w-64 shrink-0 h-8 flex items-center">
          <Search className="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-neutral-400 pointer-events-none" />
          <input
            type="text"
            placeholder="Filter cards in library..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full pl-9 pr-8 py-1.5 text-xs rounded-none bg-white/[0.04] hover:bg-white/[0.07] focus:bg-white/[0.09] text-white placeholder:text-neutral-500 focus:outline-none transition-colors font-sans"
          />
          {search && (
            <button
              onClick={() => setSearch('')}
              className="absolute right-2.5 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-white cursor-pointer"
            >
              <X className="w-3 h-3" />
            </button>
          )}
        </div>

        <div className="flex-1" />

        <div className="flex items-center gap-2.5 flex-wrap">
          <div className="flex items-center gap-2 px-3 py-1 bg-white/[0.04] border border-white/10 h-8">
            <span
              className={`w-2 h-2 rounded-full ${
                isMatchActive
                  ? 'bg-emerald-500 shadow-[0_0_8px_rgba(34,197,94,0.8)] animate-pulse'
                  : 'bg-neutral-600'
              }`}
            />
            <span className="text-xs font-sans uppercase font-bold tracking-wider text-white">
              {isMatchActive ? 'Match in Progress' : 'Waiting for Match'}
            </span>
          </div>

          {liveMatchState?.format && formatBadge && (
            <div className="flex items-center h-8">
              <span
                className="text-[11px] font-mono font-bold px-2.5 py-1 border uppercase tracking-wider h-8 flex items-center"
                style={{
                  color: formatBadge.text,
                  backgroundColor: formatBadge.bg,
                  borderColor: formatBadge.border,
                }}
              >
                {liveMatchState.format}
              </span>
            </div>
          )}

          {isMatchActive && (
            <div className="flex items-center gap-2 px-3 py-1 bg-white/[0.04] border border-white/10 text-xs tabular-nums text-neutral-300 h-8 font-mono">
              <Swords className="w-3.5 h-3.5 text-amber-400" />
              <span className="text-neutral-400 text-[11px] font-sans uppercase">Turn:</span>
              <span className="font-bold text-white">{liveMatchState?.turn || 1}</span>
              <span className="text-neutral-500">|</span>
              <span className="text-neutral-400 text-[11px] font-sans uppercase">Round:</span>
              <span className="font-bold text-white">
                {liveMatchState?.round ?? Math.ceil((liveMatchState?.turn || 1) / 2)}
              </span>
            </div>
          )}

          <button
            onClick={() => triggerToggleDeckOverlay()}
            className="flex items-center gap-2 px-3 py-1 bg-sky-500/15 hover:bg-sky-500/25 border border-sky-400/40 text-sky-300 hover:text-white text-xs font-sans font-bold uppercase tracking-wider transition-colors h-8 cursor-pointer"
            title="Toggle frameless always-on-top In-Game Pop-Out Overlay window"
          >
            <ExternalLink className="w-3.5 h-3.5" />
            <span>Pop-Out Overlay</span>
          </button>
        </div>
      </div>

      {/* 3. MAIN CONTENT OR IDLE STATE */}
      {isMatchActive || hasDeckData ? (
        <LiveDeckTrackerPanel
          deckTracker={deckTracker}
          deckName={liveMatchState?.player_deck_name}
          format={liveMatchState?.format}
          palette={palette}
          searchQuery={search}
          onShowCard={onShowCard}
        />
      ) : (
        <div className="flex-1 relative flex flex-col items-center justify-center overflow-hidden select-none">
          <img
            src={logoImg}
            alt=""
            className="absolute top-[15%] left-1/2 -translate-x-1/2 w-[46%] object-contain opacity-10 grayscale pointer-events-none"
          />
          <div className="relative z-10 px-8 py-5 border border-white/20 bg-neutral-950/80 backdrop-blur-md shadow-2xl flex flex-col items-center space-y-2 text-center">
            <div className="flex items-center gap-2.5">
              <span className="w-2.5 h-2.5 rounded-full bg-neutral-500 animate-pulse" />
              <span className="text-base font-sans font-bold tracking-[0.14em] uppercase text-white">
                Idle · Waiting for Match Deck
              </span>
            </div>
            <p className="text-xs font-sans text-neutral-300 max-w-md">
              Start any Constructed, Draft, Sealed, or Direct Challenge match in MTG Arena to track remaining cards and real-time next-draw probabilities.
            </p>
          </div>
          <img
            src={symbolIcon}
            alt=""
            className="absolute bottom-[10%] left-1/2 -translate-x-1/2 w-[18%] object-contain opacity-5 grayscale pointer-events-none"
          />
        </div>
      )}
    </div>
  );
};

export default LiveDeckTrackerView;
