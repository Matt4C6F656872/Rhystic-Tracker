import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalPosition } from '@tauri-apps/api/window';
import { X } from 'lucide-react';
import { LiveDeckTrackerPanel, DeckTrackerData } from './LiveDeckTrackerPanel';
import { ErrorBoundary } from '../ErrorBoundary';
import logoImg from '../../assets/RhysticTrackerLogo.svg';
import symbolIcon from '../../assets/RhysticTrackerICON.svg';

type ResizeDirection =
  | 'East'
  | 'North'
  | 'NorthEast'
  | 'NorthWest'
  | 'South'
  | 'SouthEast'
  | 'SouthWest'
  | 'West';

export const DeckTrackerOverlayView: React.FC = () => {
  const [trackerData, setTrackerData] = useState<DeckTrackerData | null>(null);
  const [turn, setTurn] = useState<number>(1);
  const [deckName, setDeckName] = useState<string>('Active Deck');
  const [matchFormat, setMatchFormat] = useState<string>('');
  const [isMatchActive, setIsMatchActive] = useState<boolean>(true);
  const [matchResult, setMatchResult] = useState<{ result: string; reasonLabel?: string } | null>(null);
  const lastPolledJsonRef = useRef<string>('');

  // Ensure solid root overlay classes on mount for WebKitGTK surface painting
  useEffect(() => {
    document.documentElement.classList.add('overlay-window');
    document.body.classList.add('overlay-window');
    return () => {
      document.documentElement.classList.remove('overlay-window');
      document.body.classList.remove('overlay-window');
    };
  }, []);

  // Restore saved screen position on mount if enabled
  useEffect(() => {
    const rememberPos = localStorage.getItem('rememberDeckTrackerPosition') !== 'false';
    if (!rememberPos) return;

    try {
      const savedPosRaw = localStorage.getItem('deckTrackerSavedPosition');
      if (savedPosRaw) {
        const { x, y } = JSON.parse(savedPosRaw);
        if (typeof x === 'number' && typeof y === 'number') {
          getCurrentWindow().setPosition(new LogicalPosition(x, y)).catch(console.error);
        }
      }
    } catch (e) {
      console.error('Failed to restore saved deck tracker position:', e);
    }
  }, []);

  // Track and save window position when moved
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let saveTimeout: any = null;

    getCurrentWindow().onMoved(() => {
      const rememberPos = localStorage.getItem('rememberDeckTrackerPosition') !== 'false';
      if (!rememberPos) return;

      if (saveTimeout) clearTimeout(saveTimeout);
      saveTimeout = setTimeout(async () => {
        try {
          const win = getCurrentWindow();
          const factor = await win.scaleFactor();
          const physPos = await win.outerPosition();
          const logicalPos = physPos.toLogical(factor);
          localStorage.setItem(
            'deckTrackerSavedPosition',
            JSON.stringify({ x: logicalPos.x, y: logicalPos.y })
          );
          // Also persist directly into backend settings so Tauri window builder creates it at this coordinate
          invoke('save_deck_tracker_position', { x: logicalPos.x, y: logicalPos.y }).catch((err) => {
            console.error('Failed to invoke save_deck_tracker_position:', err);
          });
        } catch (err) {
          console.error('Failed to save deck tracker position:', err);
        }
      }, 250);
    }).then((unsub) => {
      unlisten = unsub;
    }).catch(console.error);

    return () => {
      if (unlisten) unlisten();
      if (saveTimeout) clearTimeout(saveTimeout);
    };
  }, []);

  useEffect(() => {
    let cancelled = false;
    let endDelayTimeout: any = null;

    const poll = async () => {
      try {
        const liveState = await invoke<any>('get_live_match_state');
        if (cancelled) return;
        const nextJson = JSON.stringify(liveState ?? null);
        if (nextJson === lastPolledJsonRef.current) return;
        lastPolledJsonRef.current = nextJson;

        if (liveState && liveState.is_active && liveState.deck_tracker) {
          setIsMatchActive(true);
          setMatchResult(null);
          setTrackerData(liveState.deck_tracker);
          setTurn(liveState.turn || 1);
          setDeckName(liveState.player_deck_name || 'Active Deck');
          setMatchFormat(liveState.format || '');
        } else if (liveState && liveState.is_active) {
          setIsMatchActive(true);
          setMatchResult(null);
          setTurn(liveState.turn || 1);
          setDeckName(liveState.player_deck_name || 'Active Deck');
          setMatchFormat(liveState.format || '');
        } else if (liveState && (liveState.just_completed || liveState.result)) {
          // A match has just concluded! Keep the live deck tracker for a brief moment,
          // then show victory/defeat before transitioning to standby.
          setMatchResult({
            result: liveState.result || 'defeat',
            reasonLabel: liveState.reason_label || 'Match Concluded',
          });
          if (!endDelayTimeout) {
            endDelayTimeout = setTimeout(() => {
              setIsMatchActive(false);
            }, 3000);
          }
        } else {
          // Idle standby
          setIsMatchActive(false);
          setMatchResult(null);
        }
      } catch (err) {
        console.error('Failed to poll live state in overlay:', err);
      }
    };

    poll();
    const interval = setInterval(poll, 800);
    return () => {
      cancelled = true;
      clearInterval(interval);
      if (endDelayTimeout) clearTimeout(endDelayTimeout);
    };
  }, []);

  const handleClose = () => {
    invoke('close_deck_tracker_window').catch(console.error);
  };

  const handleResizeMouseDown = (dir: ResizeDirection) => (e: React.MouseEvent) => {
    if (e.button !== 0) return;
    e.stopPropagation();
    getCurrentWindow().startResizeDragging(dir).catch(console.error);
  };

  const handleStartDragging = (e: React.MouseEvent) => {
    // Only drag on left-click and not on buttons/inputs/resize handles
    if (
      e.button === 0 &&
      !(e.target as HTMLElement).closest('button, input, [data-resize-handle]')
    ) {
      getCurrentWindow().startDragging().catch(console.error);
    }
  };

  return (
    <ErrorBoundary fallbackTitle="Overlay Deck Tracker Error">
      <div
        onMouseDown={handleStartDragging}
        style={{ backfaceVisibility: 'hidden', transform: 'translateZ(0)' }}
        className="w-full h-full min-h-[200px] min-w-[250px] bg-slate-900 text-white border border-slate-700 rounded-lg shadow-2xl overflow-hidden select-none flex flex-col relative"
      >
        {/* Edge & Corner Resize Handles for Frameless Window */}
        <div data-resize-handle onMouseDown={handleResizeMouseDown('North')} className="absolute top-0 left-2 right-2 h-1.5 z-50 cursor-n-resize" />
        <div data-resize-handle onMouseDown={handleResizeMouseDown('South')} className="absolute bottom-0 left-2 right-2 h-1.5 z-50 cursor-s-resize" />
        <div data-resize-handle onMouseDown={handleResizeMouseDown('West')} className="absolute left-0 top-2 bottom-2 w-1.5 z-50 cursor-w-resize" />
        <div data-resize-handle onMouseDown={handleResizeMouseDown('East')} className="absolute right-0 top-2 bottom-2 w-1.5 z-50 cursor-e-resize" />
        <div data-resize-handle onMouseDown={handleResizeMouseDown('NorthWest')} className="absolute top-0 left-0 w-2.5 h-2.5 z-50 cursor-nw-resize" />
        <div data-resize-handle onMouseDown={handleResizeMouseDown('NorthEast')} className="absolute top-0 right-0 w-2.5 h-2.5 z-50 cursor-ne-resize" />
        <div data-resize-handle onMouseDown={handleResizeMouseDown('SouthWest')} className="absolute bottom-0 left-0 w-2.5 h-2.5 z-50 cursor-sw-resize" />
        <div data-resize-handle onMouseDown={handleResizeMouseDown('SouthEast')} className="absolute bottom-0 right-0 w-2.5 h-2.5 z-50 cursor-se-resize" />

        {isMatchActive ? (
          <LiveDeckTrackerPanel
            trackerData={trackerData}
            turn={turn}
            deckName={deckName}
            formatName={matchFormat}
            isOverlay={true}
            defaultExpanded={true}
            onDockBack={handleClose}
          />
        ) : (
          /* Standby / Waiting for Match Screen */
          <div className="w-full h-full flex flex-col justify-between p-4 relative select-none bg-slate-900 text-white">
            {/* Top Bar with Clear Close 'X' Button */}
            <div className="flex items-center justify-between w-full border-b border-slate-700 pb-2 z-10 pointer-events-auto">
              <div className="flex items-center gap-1.5">
                <img src={symbolIcon} alt="Rhystic Tracker" className="w-4 h-4 object-contain" />
                <span className="text-[11px] font-sans font-bold uppercase tracking-wider text-neutral-300">
                  Live Deck Tracker
                </span>
              </div>
              <button
                onClick={handleClose}
                className="p-1 text-neutral-300 bg-slate-800 border border-slate-700 rounded-none cursor-pointer"
                title="Close window"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {/* Match Concluded Victory / Defeat Badge or Centered Standby */}
            <div className="flex flex-col items-center justify-center my-auto space-y-4 px-4 text-center z-10">
              {matchResult ? (
                <div
                  className={`w-full py-6 px-4 border flex flex-col items-center justify-center space-y-2.5 shadow-2xl ${
                    matchResult.result === 'win'
                      ? 'bg-[#0a120c] border-[#4ADE80]'
                      : 'bg-[#220d11] border-[#F87171]'
                  }`}
                >
                  <span
                    className={`text-2xl font-display font-bold tracking-[0.16em] uppercase ${
                      matchResult.result === 'win' ? 'text-[#4ADE80]' : 'text-[#F87171]'
                    }`}
                  >
                    {matchResult.result === 'win' ? 'VICTORY' : 'DEFEAT'}
                  </span>
                  <span className="text-[10px] font-mono uppercase tracking-widest text-neutral-300 bg-slate-950 px-2.5 py-0.5 border border-slate-700">
                    {matchResult.reasonLabel || 'Match Concluded'}
                  </span>
                </div>
              ) : (
                <>
                  {/* Centered Rhystic Tracker Icon */}
                  <div className="w-16 h-16 flex items-center justify-center p-2.5 bg-slate-950 border border-slate-700 shadow-xl">
                    <img
                      src={symbolIcon}
                      alt="Rhystic Tracker"
                      className="w-full h-full object-contain"
                    />
                  </div>

                  {/* Rhystic Tracker Brand Wordmark */}
                  <img
                    src={logoImg}
                    alt="Rhystic Tracker"
                    className="w-40 object-contain"
                  />

                  {/* Waiting for Match Header & Message */}
                  <div className="flex flex-col items-center space-y-1.5 pt-1">
                    <div className="flex items-center gap-2">
                      <span className="w-2 h-2 rounded-full bg-emerald-500" />
                      <span className="text-xs font-sans font-bold tracking-[0.14em] uppercase text-white">
                        Waiting for Match
                      </span>
                    </div>
                    <p className="text-[10px] font-sans text-neutral-400 leading-relaxed max-w-[210px]">
                      Launch an MTGA match to activate real-time card and draw tracking.
                    </p>
                  </div>
                </>
              )}
            </div>

            {/* Empty space filler for layout symmetry */}
            <div className="h-2 w-full" />
          </div>
        )}
      </div>
    </ErrorBoundary>
  );
};
