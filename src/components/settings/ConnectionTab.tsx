import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Search, FolderOpen, Check, Radio, ShieldCheck, RefreshCw, UploadCloud } from 'lucide-react';
import { MTG_COLORS } from './types';

export interface ConnectionTabProps {
  palette: any;
  isSearching: boolean;

  // Search matches
  matchLogPath: boolean;
  matchBo3Sideboard: boolean;
  matchInstallLocations: boolean;

  // Log Path State & Handlers
  logPath: string;
  setLogPath: (path: string) => void;
  loadingPath: boolean;
  browseSuccess: boolean;
  savedSuccess: boolean;
  onBrowse: () => void;
  onSaveConfig: () => void;

  // Bo3 Sideboard
  bo3SideboardTracking: boolean;
  onToggleBo3Sideboard: (val: boolean) => void;
}

export const ConnectionTab: React.FC<ConnectionTabProps> = ({
  isSearching,
  matchLogPath,
  matchBo3Sideboard,
  matchInstallLocations,
  logPath,
  setLogPath,
  loadingPath,
  browseSuccess,
  savedSuccess,
  onBrowse,
  onSaveConfig,
  bo3SideboardTracking,
  onToggleBo3Sideboard,
}) => {
  const [syncStatus, setSyncStatus] = useState<{
    enabled: boolean;
    contributor_id: string;
    server_url: string;
    include_deck_cards: boolean;
    sync_key?: string | null;
    pending_count: number;
    synced_count: number;
    failed_count: number;
    last_synced_at?: string | null;
  } | null>(null);
  const [serverUrlInput, setServerUrlInput] = useState('https://api.rhystic.gg');
  const [syncSaved, setSyncSaved] = useState(false);
  const [backfillBusy, setBackfillBusy] = useState(false);
  const [backfillMsg, setBackfillMsg] = useState<string | null>(null);

  useEffect(() => {
    invoke<any>('get_sync_settings_status')
      .then((res) => {
        if (res) {
          setSyncStatus(res);
          setServerUrlInput(res.server_url || 'https://api.rhystic.gg');
        }
      })
      .catch((err) => console.error('Failed to load sync settings:', err));
  }, []);

  const applySyncUpdate = async (patch: {
    enabled?: boolean;
    server_url?: string;
    include_deck_cards?: boolean;
    rotate_contributor_id?: boolean;
  }) => {
    const nextEnabled = patch.enabled ?? syncStatus?.enabled ?? false;
    try {
      const updated = await invoke<any>('update_sync_settings', {
        enabled: nextEnabled,
        serverUrl: patch.server_url ?? serverUrlInput,
        includeDeckCards: patch.include_deck_cards ?? syncStatus?.include_deck_cards ?? true,
        syncKey: syncStatus?.sync_key ?? null,
        rotateContributorId: patch.rotate_contributor_id ?? false,
      });
      if (updated) {
        setSyncStatus(updated);
        setServerUrlInput(updated.server_url || 'https://api.rhystic.gg');
        setSyncSaved(true);
        setTimeout(() => setSyncSaved(false), 1800);
      }
    } catch (err) {
      console.error('Failed to update sync settings:', err);
    }
  };

  const handleQueueHistorical = async () => {
    setBackfillBusy(true);
    setBackfillMsg(null);
    try {
      const res = await invoke<any>('enqueue_historical_sync');
      if (res) {
        setSyncStatus((prev) =>
          prev
            ? {
                ...prev,
                pending_count: res.pending_count ?? prev.pending_count,
                synced_count: res.synced_count ?? prev.synced_count,
                failed_count: res.failed_count ?? prev.failed_count,
              }
            : prev
        );
        setBackfillMsg(`Queued ${res.queued ?? 0} match(es) into local outbox`);
      }
    } catch (err) {
      console.error('Failed to enqueue historical matches:', err);
    } finally {
      setBackfillBusy(false);
    }
  };

  return (
    <div className="space-y-6">
      {(matchLogPath || !isSearching) && (
        <div className="space-y-3">
          <div className="border-b border-white/10 pb-2 flex items-center justify-between">
            <div>
              <span className="text-[11px] font-semibold uppercase tracking-wider text-neutral-300 font-sans">
                MTGA Active Log Path Configuration
              </span>
              <p className="text-xs font-sans text-neutral-400 mt-0.5">
                Rhystic Tracker reads MTG Arena's active <code className="font-mono text-white font-bold" style={{ color: MTG_COLORS.green.text }}>Player.log</code> in real time with high-performance incremental tailing.
              </p>
            </div>
            <span 
              style={{ backgroundColor: MTG_COLORS.green.bg, borderColor: MTG_COLORS.green.border, color: '#FFFFFF' }}
              className="inline-flex items-center gap-1.5 px-2.5 py-0.5 border text-[10px] font-mono font-bold uppercase tracking-wider"
            >
              <Radio className="w-3 h-3 animate-pulse" style={{ color: MTG_COLORS.green.text }} /> Live Tailer Active
            </span>
          </div>

          <div className="space-y-2 pt-1">
            <label className="text-[10px] font-mono uppercase text-neutral-400 font-bold">
              Active Player.log Path
            </label>
            <div className="flex gap-2">
              <div className="flex-1 relative">
                <Search className="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-neutral-500" />
                <input
                  type="text"
                  value={logPath}
                  onChange={(e) => setLogPath(e.target.value)}
                  placeholder={loadingPath ? 'Loading…' : 'No log detected — use Browse to select'}
                  className="w-full pl-9 pr-3 py-2 border border-white/10 bg-white/[0.03] text-xs font-mono text-white placeholder:text-neutral-600 focus:outline-none focus:border-white/30"
                />
              </div>
              <button
                onClick={onBrowse}
                className="px-4 py-2 border border-white/15 bg-white/[0.04] hover:bg-white/[0.08] active:scale-95 text-xs font-mono font-bold uppercase tracking-wider text-white transition-all flex items-center gap-1.5 cursor-pointer shrink-0"
              >
                {browseSuccess ? <Check className="w-3.5 h-3.5" style={{ color: MTG_COLORS.green.text }} /> : <FolderOpen className="w-3.5 h-3.5" />}
                {browseSuccess ? 'Applied' : 'Browse…'}
              </button>
              <button
                onClick={onSaveConfig}
                style={{ backgroundColor: MTG_COLORS.green.bg, borderColor: MTG_COLORS.green.border }}
                className="px-4 py-2 border hover:brightness-125 active:scale-95 text-xs font-mono font-bold uppercase tracking-wider text-white transition-all flex items-center gap-1.5 cursor-pointer shrink-0"
              >
                {savedSuccess ? <Check className="w-3.5 h-3.5" style={{ color: MTG_COLORS.green.text }} /> : null}
                {savedSuccess ? 'Saved' : 'Save Config'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Best-of-Three Tracking */}
      {(matchBo3Sideboard || !isSearching) && (
        <div className="space-y-3 pt-2">
          <div className="border-b border-white/10 pb-2 flex items-center justify-between">
            <span className="text-[11px] font-semibold uppercase tracking-wider text-neutral-300 font-sans">
              Competitive Match Tracking
            </span>
            <span className="text-[10px] font-mono text-neutral-500 uppercase tracking-wider">Tournament Rules</span>
          </div>

          <div className="flex items-center justify-between py-2 border border-white/10 bg-white/[0.02] p-3">
            <div className="space-y-0.5 pr-4">
              <p className="text-xs font-bold text-white uppercase tracking-wide font-sans">
                Best-of-Three Sideboard Segregation
              </p>
              <p className="text-xs font-sans text-neutral-400">
                In Bo3 matches, track sideboarded cards brought in for games 2 & 3 in a dedicated Live HUD panel.
              </p>
            </div>
            <button
              type="button"
              role="switch"
              aria-checked={bo3SideboardTracking}
              onClick={() => onToggleBo3Sideboard(!bo3SideboardTracking)}
              style={bo3SideboardTracking ? { backgroundColor: MTG_COLORS.green.bg, borderColor: MTG_COLORS.green.border } : undefined}
              className={`relative inline-flex items-center h-5 w-10 shrink-0 cursor-pointer border transition-colors ${
                bo3SideboardTracking ? '' : 'bg-white/[0.04] border-white/15'
              }`}
            >
              <span
                style={bo3SideboardTracking ? { backgroundColor: MTG_COLORS.green.base } : undefined}
                className={`inline-block h-3.5 w-3.5 transform transition-transform ${
                  bo3SideboardTracking ? 'translate-x-5' : 'translate-x-0.5 bg-neutral-500'
                }`}
              />
            </button>
          </div>
        </div>
      )}

      {/* Where to find Player.log assistant guide */}
      {(matchInstallLocations || !isSearching) && (
        <div className="space-y-3 pt-2">
          <div className="border border-white/10 bg-white/[0.02] p-4 space-y-2.5">
            <p className="text-[10px] font-mono font-bold uppercase tracking-wider" style={{ color: MTG_COLORS.gold.text }}>
              Common MTGA Install Locations
            </p>
            <div className="text-[11px] font-mono space-y-1.5 text-neutral-300">
              <div>
                <span className="text-neutral-500">Steam (Proton):</span>{' '}
                <span className="text-neutral-200 break-all">&lt;SteamLibrary&gt;/steamapps/compatdata/2141910/pfx/drive_c/users/steamuser/AppData/LocalLow/Wizards Of The Coast/MTGA/Player.log</span>
              </div>
              <div>
                <span className="text-neutral-500">Lutris / Wine:</span>{' '}
                <span className="text-neutral-200 break-all">&lt;wine-prefix&gt;/drive_c/users/&lt;user&gt;/AppData/LocalLow/Wizards Of The Coast/MTGA/Player.log</span>
              </div>
              <div>
                <span className="text-neutral-500">Native Linux:</span>{' '}
                <span className="text-neutral-200 break-all">&lt;SteamLibrary&gt;/steamapps/common/MTGA/MTGA_Data/Downloads/Player.log</span>
              </div>
            </div>
            <p className="text-[10px] font-sans italic text-neutral-500 pt-2 border-t border-white/5">
              Note: Detailed logging must be enabled in MTGA (Options → Account → Detailed Logs Plugin Support).
            </p>
          </div>
        </div>
      )}

      {/* Community Meta & Cloud Sync (Opt-In, Phase 3) */}
      {!isSearching && (
        <div className="space-y-3 pt-2">
          <div className="border-b border-white/10 pb-2 flex items-center justify-between">
            <div>
              <span className="text-[11px] font-semibold uppercase tracking-wider text-neutral-300 font-sans">
                Community Meta &amp; Cloud Sync (Strictly Opt-In)
              </span>
              <p className="text-xs font-sans text-neutral-400 mt-0.5">
                Contribute 100% anonymized match &amp; Commander telemetry (zero screen names, zero account IDs) to power global format tier lists and achievement rarity.
              </p>
            </div>
            <span
              className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 border text-[10px] font-mono font-bold uppercase tracking-wider shrink-0 ${
                syncStatus?.enabled
                  ? 'bg-emerald-500/15 border-emerald-500/40 text-emerald-300'
                  : 'bg-white/[0.04] border-white/15 text-neutral-400'
              }`}
            >
              <ShieldCheck className="w-3 h-3" />
              {syncStatus?.enabled ? 'Tier 1: Opted In' : 'Tier 0: 100% Offline'}
            </span>
          </div>

          <div className="space-y-2.5 border border-white/10 bg-white/[0.02] p-3.5">
            <div className="flex items-center justify-between pb-2.5 border-b border-white/5">
              <div className="space-y-0.5 pr-4">
                <p className="text-xs font-bold text-white uppercase tracking-wide font-sans">
                  Enable Anonymized Match Telemetry Outbox
                </p>
                <p className="text-xs font-sans text-neutral-400">
                  Queue completed legitimate matches (excluding Bot Matches &amp; precons) into the local sync outbox.
                </p>
              </div>
              <button
                type="button"
                role="switch"
                aria-checked={!!syncStatus?.enabled}
                onClick={() => applySyncUpdate({ enabled: !syncStatus?.enabled })}
                style={syncStatus?.enabled ? { backgroundColor: MTG_COLORS.green.bg, borderColor: MTG_COLORS.green.border } : undefined}
                className={`relative inline-flex items-center h-5 w-10 shrink-0 cursor-pointer border transition-colors ${
                  syncStatus?.enabled ? '' : 'bg-white/[0.04] border-white/15'
                }`}
              >
                <span
                  style={syncStatus?.enabled ? { backgroundColor: MTG_COLORS.green.base } : undefined}
                  className={`inline-block h-3.5 w-3.5 transform transition-transform ${
                    syncStatus?.enabled ? 'translate-x-5' : 'translate-x-0.5 bg-neutral-500'
                  }`}
                />
              </button>
            </div>

            <div className="flex items-center justify-between py-1.5 border-b border-white/5">
              <div className="space-y-0.5 pr-4">
                <p className="text-xs font-bold text-white uppercase tracking-wide font-sans">
                  Include Seen Card IDs for Commander &amp; Archetype Analytics
                </p>
                <p className="text-xs font-sans text-neutral-400">
                  Shares numeric card <code className="font-mono text-neutral-200">grpId</code>s seen during the match (never your personal deck name).
                </p>
              </div>
              <button
                type="button"
                role="switch"
                aria-checked={syncStatus?.include_deck_cards ?? true}
                onClick={() => applySyncUpdate({ include_deck_cards: !(syncStatus?.include_deck_cards ?? true) })}
                style={(syncStatus?.include_deck_cards ?? true) ? { backgroundColor: MTG_COLORS.green.bg, borderColor: MTG_COLORS.green.border } : undefined}
                className={`relative inline-flex items-center h-5 w-10 shrink-0 cursor-pointer border transition-colors ${
                  (syncStatus?.include_deck_cards ?? true) ? '' : 'bg-white/[0.04] border-white/15'
                }`}
              >
                <span
                  style={(syncStatus?.include_deck_cards ?? true) ? { backgroundColor: MTG_COLORS.green.base } : undefined}
                  className={`inline-block h-3.5 w-3.5 transform transition-transform ${
                    (syncStatus?.include_deck_cards ?? true) ? 'translate-x-5' : 'translate-x-0.5 bg-neutral-500'
                  }`}
                />
              </button>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-3 pt-1">
              <div className="space-y-1">
                <label className="text-[10px] font-mono uppercase text-neutral-400 font-bold">
                  Anonymous Contributor UUID
                </label>
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    readOnly
                    value={syncStatus?.contributor_id || '—'}
                    className="flex-1 px-2.5 py-1.5 border border-white/10 bg-black/40 text-[11px] font-mono text-neutral-300 select-all"
                  />
                  <button
                    type="button"
                    onClick={() => applySyncUpdate({ rotate_contributor_id: true })}
                    title="Generate a fresh anonymous UUID"
                    className="px-2.5 py-1.5 border border-white/15 bg-white/[0.04] hover:bg-white/[0.08] text-[10px] font-mono uppercase tracking-wider text-neutral-200 flex items-center gap-1 cursor-pointer shrink-0"
                  >
                    <RefreshCw className="w-3 h-3" /> Rotate
                  </button>
                </div>
              </div>

              <div className="space-y-1">
                <label className="text-[10px] font-mono uppercase text-neutral-400 font-bold">
                  Ingestion Server Endpoint
                </label>
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    value={serverUrlInput}
                    onChange={(e) => setServerUrlInput(e.target.value)}
                    className="flex-1 px-2.5 py-1.5 border border-white/10 bg-white/[0.03] text-[11px] font-mono text-white focus:outline-none focus:border-white/30"
                  />
                  <button
                    type="button"
                    onClick={() => applySyncUpdate({ server_url: serverUrlInput })}
                    className="px-3 py-1.5 border border-white/15 bg-white/[0.06] hover:bg-white/[0.12] text-[10px] font-mono font-bold uppercase tracking-wider text-white flex items-center gap-1 cursor-pointer shrink-0"
                  >
                    {syncSaved ? <Check className="w-3 h-3 text-emerald-400" /> : null}
                    {syncSaved ? 'Saved' : 'Save'}
                  </button>
                </div>
              </div>
            </div>

            <div className="flex flex-wrap items-center justify-between gap-3 pt-2 border-t border-white/10 text-[11px] font-mono">
              <div className="flex items-center gap-4 text-neutral-400">
                <span>
                  Outbox Pending: <strong className="text-amber-300">{syncStatus?.pending_count ?? 0}</strong>
                </span>
                <span>
                  Synced: <strong className="text-emerald-300">{syncStatus?.synced_count ?? 0}</strong>
                </span>
                {backfillMsg && <span className="text-cyan-300">{backfillMsg}</span>}
              </div>
              <button
                type="button"
                disabled={backfillBusy}
                onClick={handleQueueHistorical}
                className="px-3 py-1 border border-white/15 bg-white/[0.04] hover:bg-white/[0.09] disabled:opacity-40 text-[10px] font-mono font-bold uppercase tracking-wider text-white flex items-center gap-1.5 cursor-pointer"
              >
                <UploadCloud className="w-3.5 h-3.5" />
                {backfillBusy ? 'Queueing…' : 'Queue Match History for Sync'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
