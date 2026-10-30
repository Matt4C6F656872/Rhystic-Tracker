import React from 'react';
import { Sliders, RefreshCw, ArrowUpCircle, ExternalLink, Check, Download, AlertCircle, RotateCcw } from 'lucide-react';
import { CustomDropdown } from '../CustomDropdown';
import { MTG_COLORS } from './types';
import { useAppUpdate, checkForAppUpdate, installAppUpdate, restartApp, setAutoCheckUpdates } from '../../utils/appUpdate';
import { APP_VERSION } from '../../version';

export interface GeneralTabProps {
  palette: any;
  isSearching: boolean;
  // Search flags
  matchMinimize: boolean;
  matchAutoSwitch: boolean;
  matchAlwaysOnTop: boolean;
  matchAutoOpenDeckTracker?: boolean;
  matchAudioCues: boolean;
  matchExcludeSparky: boolean;
  matchConfirmDelete: boolean;
  matchAllowDelete: boolean;
  matchAutoExport: boolean;
  matchStartupTab: boolean;
  matchPersistDashboardPage: boolean;
  matchSetupWizard: boolean;
  matchUpdates?: boolean;

  // State values
  minimizeToTray: boolean;
  autoSwitchLiveView: 'off' | 'live' | 'live-deck';
  liveHudAlwaysOnTop: boolean;
  autoOpenDeckTracker?: boolean;
  rememberDeckTrackerPosition?: boolean;
  excludeSparkyMatches: boolean;
  enableAudioCues: boolean;
  confirmDeckDelete: boolean;
  allowMatchDeletion: boolean;
  autoExportMatches: boolean;
  autoExportFormat: string;
  defaultStartupTab: string;
  persistDashboardPage: boolean;

  // Options
  autoExportFormatOptions: { value: string; label: string }[];
  startupTabOptions: { value: string; label: string }[];

  // Action Handlers
  onToggleMinimizeToTray: (val: boolean) => void;
  onChangeAutoSwitchLiveView: (val: 'off' | 'live' | 'live-deck') => void;
  onToggleAlwaysOnTop: (val: boolean) => void;
  onToggleAutoOpenDeckTracker?: (val: boolean) => void;
  onToggleRememberDeckTrackerPosition?: (val: boolean) => void;
  onToggleExcludeSparky: (val: boolean) => void;
  onToggleAudioCues: (val: boolean) => void;
  onToggleConfirmDeckDelete: (val: boolean) => void;
  onToggleAllowMatchDeletion: (val: boolean) => void;
  onToggleAutoExport: (val: boolean) => void;
  onChangeAutoExportFormat: (val: string) => void;
  onChangeDefaultStartupTab: (val: string) => void;
  onTogglePersistDashboardPage: (val: boolean) => void;
  onOpenResetWizardModal: () => void;
}

export const GeneralTab: React.FC<GeneralTabProps> = ({
  palette,
  isSearching,
  matchMinimize,
  matchAutoSwitch,
  matchAlwaysOnTop,
  matchAutoOpenDeckTracker,
  matchAudioCues,
  matchExcludeSparky,
  matchConfirmDelete,
  matchAllowDelete,
  matchAutoExport,
  matchStartupTab,
  matchPersistDashboardPage,
  matchSetupWizard,
  matchUpdates,
  minimizeToTray,
  autoSwitchLiveView,
  liveHudAlwaysOnTop,
  autoOpenDeckTracker,
  rememberDeckTrackerPosition,
  excludeSparkyMatches,
  enableAudioCues,
  confirmDeckDelete,
  allowMatchDeletion,
  autoExportMatches,
  autoExportFormat,
  defaultStartupTab,
  persistDashboardPage,
  autoExportFormatOptions,
  startupTabOptions,
  onToggleMinimizeToTray,
  onChangeAutoSwitchLiveView,
  onToggleAlwaysOnTop,
  onToggleAutoOpenDeckTracker,
  onToggleRememberDeckTrackerPosition,
  onToggleExcludeSparky,
  onToggleAudioCues,
  onToggleConfirmDeckDelete,
  onToggleAllowMatchDeletion,
  onToggleAutoExport,
  onChangeAutoExportFormat,
  onChangeDefaultStartupTab,
  onTogglePersistDashboardPage,
  onOpenResetWizardModal,
}) => {
  const appUpdate = useAppUpdate();
  return (
    <div className="space-y-6">
      {/* Application Behavior Sub-section */}
      {(matchMinimize || matchAutoSwitch || matchAlwaysOnTop || matchAudioCues || matchExcludeSparky || matchConfirmDelete || matchAllowDelete || !isSearching) && (
        <div className="space-y-3">
          <div className="border-b border-white/10 pb-2 flex items-center justify-between">
            <span className="text-[11px] font-semibold uppercase tracking-wider text-neutral-300 font-sans">
              Application Behavior
            </span>
            <span className="text-[10px] font-mono text-neutral-500 uppercase tracking-wider">
              Desktop Lifecycle
            </span>
          </div>

          <div className="divide-y divide-white/5">
            {/* Minimize to Tray */}
            {(matchMinimize || !isSearching) && (
              <div className="flex items-center justify-between py-3">
                <div className="space-y-0.5 pr-4">
                  <p className="text-xs font-bold text-white uppercase tracking-wide font-sans">
                    Minimize to System Tray on Close
                  </p>
                  <p className="text-xs font-sans text-neutral-400">
                    Keep Rhystic Tracker actively tracking matches in the background when the main window is closed.
                  </p>
                </div>
                <button
                  type="button"
                  role="switch"
                  aria-checked={minimizeToTray}
                  onClick={() => onToggleMinimizeToTray(!minimizeToTray)}
                  style={minimizeToTray ? { backgroundColor: MTG_COLORS.green.bg, borderColor: MTG_COLORS.green.border } : undefined}
                  className={`relative inline-flex items-center h-5 w-10 shrink-0 cursor-pointer border transition-colors ${
                    minimizeToTray ? '' : 'bg-white/[0.04] border-white/15'
                  }`}
                >
                  <span
                    style={minimizeToTray ? { backgroundColor: MTG_COLORS.green.base } : undefined}
                    className={`inline-block h-3.5 w-3.5 transform transition-transform ${
                      minimizeToTray ? 'translate-x-5' : 'translate-x-0.5 bg-neutral-500'
                    }`}
                  />
                </button>
              </div>
            )}

            {/* Auto Switch on Match Start */}
            {(matchAutoSwitch || !isSearching) && (
              <div className="flex flex-col sm:flex-row sm:items-center justify-between py-3 gap-3 border-b border-white/5">
                <div className="space-y-0.5 pr-4">
                  <p className="text-xs font-bold text-white uppercase tracking-wide font-sans">
                    On Match Start: Auto-Switch View
                  </p>
                  <p className="text-xs font-sans text-neutral-400">
                    Automatically navigate to your preferred view whenever a match begins in MTGA.
                  </p>
                </div>
                <div className="inline-flex rounded border border-white/10 bg-black/40 p-0.5 shrink-0 self-start sm:self-auto">
                  <button
                    type="button"
                    onClick={() => onChangeAutoSwitchLiveView('off')}
                    className={`px-2.5 py-1 text-[11px] font-mono uppercase tracking-wider rounded transition-colors ${
                      autoSwitchLiveView === 'off'
                        ? 'bg-white/20 text-white font-bold'
                        : 'text-neutral-400 hover:text-white'
                    }`}
                  >
                    Off
                  </button>
                  <button
                    type="button"
                    onClick={() => onChangeAutoSwitchLiveView('live')}
                    className={`px-2.5 py-1 text-[11px] font-mono uppercase tracking-wider rounded transition-colors ${
                      autoSwitchLiveView === 'live'
                        ? 'bg-amber-500/25 text-amber-300 font-bold border border-amber-500/40'
                        : 'text-neutral-400 hover:text-white'
                    }`}
                  >
                    Live HUD
                  </button>
                  <button
                    type="button"
                    onClick={() => onChangeAutoSwitchLiveView('live-deck')}
                    className={`px-2.5 py-1 text-[11px] font-mono uppercase tracking-wider rounded transition-colors ${
                      autoSwitchLiveView === 'live-deck'
                        ? 'bg-cyan-500/25 text-cyan-300 font-bold border border-cyan-500/40'
                        : 'text-neutral-400 hover:text-white'
                    }`}
                  >
                    Deck Tracker
                  </button>
                </div>
              </div>
            )}

            {/* Live HUD Always on Top */}
            {(matchAlwaysOnTop || !isSearching) && (
              <div className="flex items-center justify-between py-3">
                <div className="space-y-0.5 pr-4">
                  <p className="text-xs font-bold text-white uppercase tracking-wide font-sans">
                    Pin Window Always on Top (In-Game Overlay)
                  </p>
                  <p className="text-xs font-sans text-neutral-400">
                    Keep Rhystic Tracker floating on top of MTG Arena when playing in windowed or borderless mode.
                  </p>
                </div>
                <button
                  type="button"
                  role="switch"
                  aria-checked={liveHudAlwaysOnTop}
                  onClick={() => onToggleAlwaysOnTop(!liveHudAlwaysOnTop)}
                  style={liveHudAlwaysOnTop ? { backgroundColor: MTG_COLORS.green.bg, borderColor: MTG_COLORS.green.border } : undefined}
                  className={`relative inline-flex items-center h-5 w-10 shrink-0 cursor-pointer border transition-colors ${
                    liveHudAlwaysOnTop ? '' : 'bg-white/[0.04] border-white/15'
                  }`}
                >
                  <span
                    style={liveHudAlwaysOnTop ? { backgroundColor: MTG_COLORS.green.base } : undefined}
                    className={`inline-block h-3.5 w-3.5 transform transition-transform ${
                      liveHudAlwaysOnTop ? 'translate-x-5' : 'translate-x-0.5 bg-neutral-500'
                    }`}
                  />
                </button>
              </div>
            )}

            {/* Auto Open Live Deck Tracker Popup */}
            {(matchAutoOpenDeckTracker || !isSearching) && (
              <div className="flex items-center justify-between py-3">
                <div className="space-y-0.5 pr-4">
                  <p className="text-xs font-bold text-white uppercase tracking-wide font-sans">
                    Auto-Open Live Deck Tracker Overlay
                  </p>
                  <p className="text-xs font-sans text-neutral-400">
                    Automatically launch the floating, detached live deck tracker pop-up window when a new match begins.
                  </p>
                </div>
                <button
                  type="button"
                  role="switch"
                  aria-checked={autoOpenDeckTracker}
                  onClick={() => onToggleAutoOpenDeckTracker?.(!autoOpenDeckTracker)}
                  style={autoOpenDeckTracker ? { backgroundColor: MTG_COLORS.green.bg, borderColor: MTG_COLORS.green.border } : undefined}
                  className={`relative inline-flex items-center h-5 w-10 shrink-0 cursor-pointer border transition-colors ${
                    autoOpenDeckTracker ? '' : 'bg-white/[0.04] border-white/15'
                  }`}
                >
                  <span
                    style={autoOpenDeckTracker ? { backgroundColor: MTG_COLORS.green.base } : undefined}
                    className={`inline-block h-3.5 w-3.5 transform transition-transform ${
                      autoOpenDeckTracker ? 'translate-x-5' : 'translate-x-0.5 bg-neutral-500'
                    }`}
                  />
                </button>
              </div>
            )}

            {/* Remember Deck Tracker Window Position */}
            {(matchAutoOpenDeckTracker || !isSearching) && (
              <div className="flex items-center justify-between py-3">
                <div className="space-y-0.5 pr-4">
                  <p className="text-xs font-bold text-white uppercase tracking-wide font-sans">
                    Remember Deck Tracker Screen Position
                  </p>
                  <p className="text-xs font-sans text-neutral-400">
                    Preserve the exact screen coordinates where you last dragged the deck tracker pop-up on this machine.
                  </p>
                </div>
                <button
                  type="button"
                  role="switch"
                  aria-checked={rememberDeckTrackerPosition}
                  onClick={() => onToggleRememberDeckTrackerPosition?.(!rememberDeckTrackerPosition)}
                  style={rememberDeckTrackerPosition ? { backgroundColor: MTG_COLORS.green.bg, borderColor: MTG_COLORS.green.border } : undefined}
                  className={`relative inline-flex items-center h-5 w-10 shrink-0 cursor-pointer border transition-colors ${
                    rememberDeckTrackerPosition ? '' : 'bg-white/[0.04] border-white/15'
                  }`}
                >
                  <span
                    style={rememberDeckTrackerPosition ? { backgroundColor: MTG_COLORS.green.base } : undefined}
                    className={`inline-block h-3.5 w-3.5 transform transition-transform ${
                      rememberDeckTrackerPosition ? 'translate-x-5' : 'translate-x-0.5 bg-neutral-500'
                    }`}
                  />
                </button>
              </div>
            )}

            {/* Exclude Sparky / Bot Matches */}
            {(matchExcludeSparky || !isSearching) && (
              <div className="flex items-center justify-between py-3">
                <div className="space-y-0.5 pr-4">
                  <p className="text-xs font-bold text-white uppercase tracking-wide font-sans">
                    Exclude Sparky & Tutorial Matches
                  </p>
                  <p className="text-xs font-sans text-neutral-400">
                    Ignore practice matches against Sparky or tutorial challenges in match history and win-rate statistics.
                  </p>
                </div>
                <button
                  type="button"
                  role="switch"
                  aria-checked={excludeSparkyMatches}
                  onClick={() => onToggleExcludeSparky(!excludeSparkyMatches)}
                  style={excludeSparkyMatches ? { backgroundColor: MTG_COLORS.green.bg, borderColor: MTG_COLORS.green.border } : undefined}
                  className={`relative inline-flex items-center h-5 w-10 shrink-0 cursor-pointer border transition-colors ${
                    excludeSparkyMatches ? '' : 'bg-white/[0.04] border-white/15'
                  }`}
                >
                  <span
                    style={excludeSparkyMatches ? { backgroundColor: MTG_COLORS.green.base } : undefined}
                    className={`inline-block h-3.5 w-3.5 transform transition-transform ${
                      excludeSparkyMatches ? 'translate-x-5' : 'translate-x-0.5 bg-neutral-500'
                    }`}
                  />
                </button>
              </div>
            )}

            {/* Audio Cues */}
            {(matchAudioCues || !isSearching) && (
              <div className="flex items-center justify-between py-3">
                <div className="space-y-0.5 pr-4">
                  <p className="text-xs font-bold text-white uppercase tracking-wide font-sans">
                    Sound Effects & Audio Cues
                  </p>
                  <p className="text-xs font-sans text-neutral-400">
                    Play subtle audio notifications when a match starts, when a victory/defeat is recorded, and on achievement unlock.
                  </p>
                </div>
                <button
                  type="button"
                  role="switch"
                  aria-checked={enableAudioCues}
                  onClick={() => onToggleAudioCues(!enableAudioCues)}
                  style={enableAudioCues ? { backgroundColor: MTG_COLORS.green.bg, borderColor: MTG_COLORS.green.border } : undefined}
                  className={`relative inline-flex items-center h-5 w-10 shrink-0 cursor-pointer border transition-colors ${
                    enableAudioCues ? '' : 'bg-white/[0.04] border-white/15'
                  }`}
                >
                  <span
                    style={enableAudioCues ? { backgroundColor: MTG_COLORS.green.base } : undefined}
                    className={`inline-block h-3.5 w-3.5 transform transition-transform ${
                      enableAudioCues ? 'translate-x-5' : 'translate-x-0.5 bg-neutral-500'
                    }`}
                  />
                </button>
              </div>
            )}

            {/* Confirm Deck Delete */}
            {(matchConfirmDelete || !isSearching) && (
              <div className="flex items-center justify-between py-3">
                <div className="space-y-0.5 pr-4">
                  <p className="text-xs font-bold text-white uppercase tracking-wide font-sans">
                    Confirm Before Deleting Decks
                  </p>
                  <p className="text-xs font-sans text-neutral-400">
                    Prompt with a confirmation dialog when deleting a deck from the Deck Library.
                  </p>
                </div>
                <button
                  type="button"
                  role="switch"
                  aria-checked={confirmDeckDelete}
                  onClick={() => onToggleConfirmDeckDelete(!confirmDeckDelete)}
                  style={confirmDeckDelete ? { backgroundColor: MTG_COLORS.green.bg, borderColor: MTG_COLORS.green.border } : undefined}
                  className={`relative inline-flex items-center h-5 w-10 shrink-0 cursor-pointer border transition-colors ${
                    confirmDeckDelete ? '' : 'bg-white/[0.04] border-white/15'
                  }`}
                >
                  <span
                    style={confirmDeckDelete ? { backgroundColor: MTG_COLORS.green.base } : undefined}
                    className={`inline-block h-3.5 w-3.5 transform transition-transform ${
                      confirmDeckDelete ? 'translate-x-5' : 'translate-x-0.5 bg-neutral-500'
                    }`}
                  />
                </button>
              </div>
            )}

            {/* Allow Match Deletion in History */}
            {(matchAllowDelete || !isSearching) && (
              <div className="flex items-center justify-between py-3">
                <div className="space-y-0.5 pr-4">
                  <p className="text-xs font-bold text-white uppercase tracking-wide font-sans">
                    Enable Match Deletion in History
                  </p>
                  <p className="text-xs font-sans text-neutral-400">
                    Add a Delete column with a trash icon to Match History table view, allowing permanent removal of specific matches from the database.
                  </p>
                </div>
                <button
                  type="button"
                  role="switch"
                  aria-checked={allowMatchDeletion}
                  onClick={() => onToggleAllowMatchDeletion(!allowMatchDeletion)}
                  style={allowMatchDeletion ? { backgroundColor: MTG_COLORS.green.bg, borderColor: MTG_COLORS.green.border } : undefined}
                  className={`relative inline-flex items-center h-5 w-10 shrink-0 cursor-pointer border transition-colors ${
                    allowMatchDeletion ? '' : 'bg-white/[0.04] border-white/15'
                  }`}
                >
                  <span
                    style={allowMatchDeletion ? { backgroundColor: MTG_COLORS.green.base } : undefined}
                    className={`inline-block h-3.5 w-3.5 transform transition-transform ${
                      allowMatchDeletion ? 'translate-x-5' : 'translate-x-0.5 bg-neutral-500'
                    }`}
                  />
                </button>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Data Export Sub-section */}
      {(matchAutoExport || !isSearching) && (
        <div className="space-y-3 pt-2">
          <div className="border-b border-white/10 pb-2 flex items-center justify-between">
            <span className="text-[11px] font-semibold uppercase tracking-wider text-neutral-300 font-sans">
              Automated Data Export
            </span>
            <span className="text-[10px] font-mono text-neutral-500 uppercase tracking-wider">
              Match Archives
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-1">
            <div className="flex items-center justify-between p-3 border border-white/10 bg-white/[0.02]">
              <div className="space-y-0.5 pr-2">
                <p className="text-xs font-bold text-white font-sans uppercase">Auto-Export Completed Matches</p>
                <p className="text-[11px] font-sans text-neutral-400">Save structured record files upon match end.</p>
              </div>
              <button
                type="button"
                role="switch"
                aria-checked={autoExportMatches}
                onClick={() => onToggleAutoExport(!autoExportMatches)}
                style={autoExportMatches ? { backgroundColor: MTG_COLORS.green.bg, borderColor: MTG_COLORS.green.border } : undefined}
                className={`relative inline-flex items-center h-5 w-9 shrink-0 cursor-pointer border transition-colors ${
                  autoExportMatches ? '' : 'bg-white/[0.04] border-white/15'
                }`}
              >
                <span
                  style={autoExportMatches ? { backgroundColor: MTG_COLORS.green.base } : undefined}
                  className={`inline-block h-3 w-3 transform transition-transform ${
                    autoExportMatches ? 'translate-x-5' : 'translate-x-1 bg-neutral-500'
                  }`}
                />
              </button>
            </div>

            <div className="space-y-1.5">
              <label className="text-[10px] font-mono uppercase text-neutral-400 font-bold">
                Export File Format
              </label>
              <CustomDropdown
                options={autoExportFormatOptions}
                value={autoExportFormat}
                onChange={onChangeAutoExportFormat}
                palette={palette}
              />
            </div>
          </div>
        </div>
      )}

      {/* Startup & Navigation Sub-section */}
      {(matchStartupTab || matchPersistDashboardPage || matchSetupWizard || !isSearching) && (
        <div className="space-y-3 pt-2">
          <div className="border-b border-white/10 pb-2 flex items-center justify-between">
            <span className="text-[11px] font-semibold uppercase tracking-wider text-neutral-300 font-sans">
              Startup & Navigation
            </span>
            <span className="text-[10px] font-mono text-neutral-500 uppercase tracking-wider">
              Workspaces
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-5 pt-1">
            {(matchStartupTab || !isSearching) && (
              <div className="space-y-2">
                <label className="text-[10px] font-mono uppercase text-neutral-400 font-bold">
                  Default Startup Tab
                </label>
                <CustomDropdown
                  options={startupTabOptions}
                  value={defaultStartupTab}
                  onChange={onChangeDefaultStartupTab}
                  palette={palette}
                />
                <p className="text-[11px] font-sans text-neutral-500">
                  View loaded automatically when Rhystic Tracker starts.
                </p>
              </div>
            )}

            {(matchPersistDashboardPage || !isSearching) && (
              <div className="flex items-center justify-between p-3 border border-white/10 bg-white/[0.02]">
                <div className="space-y-0.5 pr-2">
                  <p className="text-xs font-bold text-white font-sans uppercase">Remember Dashboard Page</p>
                  <p className="text-[11px] font-sans text-neutral-400">
                    {persistDashboardPage
                      ? "Keep last viewed dashboard page across tab switches and restarts."
                      : "Always reset to Dashboard Page 1 when navigating to dashboard."}
                  </p>
                </div>
                <button
                  type="button"
                  role="switch"
                  aria-checked={persistDashboardPage}
                  onClick={() => onTogglePersistDashboardPage(!persistDashboardPage)}
                  style={persistDashboardPage ? { backgroundColor: MTG_COLORS.blue.bg, borderColor: MTG_COLORS.blue.border } : undefined}
                  className={`relative inline-flex items-center h-5 w-9 shrink-0 cursor-pointer border transition-colors ${
                    persistDashboardPage ? '' : 'bg-white/[0.04] border-white/15'
                  }`}
                >
                  <span
                    style={persistDashboardPage ? { backgroundColor: MTG_COLORS.blue.base } : undefined}
                    className={`inline-block h-3 w-3 transform transition-transform ${
                      persistDashboardPage ? 'translate-x-5' : 'translate-x-1 bg-neutral-500'
                    }`}
                  />
                </button>
              </div>
            )}

            {(matchSetupWizard || !isSearching) && (
              <div className="space-y-2 md:col-span-2">
                <label className="text-[10px] font-mono uppercase text-neutral-400 font-bold">
                  First-Time Setup Assistant
                </label>
                <div>
                  <button
                    onClick={onOpenResetWizardModal}
                    className="w-full sm:w-auto px-4 py-2 border border-white/15 bg-white/[0.04] hover:bg-white/[0.08] active:scale-95 text-xs font-mono font-bold uppercase tracking-wider text-white transition-all flex items-center justify-center gap-2 cursor-pointer"
                  >
                    <Sliders className="w-3.5 h-3.5" style={{ color: MTG_COLORS.blue.text }} /> Re-run Setup Wizard
                  </button>
                </div>
                <p className="text-[11px] font-sans text-neutral-500">
                  Re-opens the wizard to re-scan log paths and card databases.
                </p>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Application Updates Sub-section */}
      {(matchUpdates || !isSearching) && (
        <div className="space-y-3 pt-2">
          <div className="border-b border-white/10 pb-2 flex items-center justify-between">
            <div>
              <span className="text-[11px] font-semibold uppercase tracking-wider text-neutral-300 font-sans">
                Application Updates
              </span>
              <p className="text-xs font-sans text-neutral-400 mt-0.5">
                Check GitHub releases for updates and install new builds seamlessly.
              </p>
            </div>
            <span className="text-[10px] font-mono text-neutral-500 uppercase tracking-wider">
              GitHub Releases
            </span>
          </div>

          <div className="border border-white/10 bg-white/[0.02] p-4 space-y-4">
            {/* Version Status and Target Repo Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-1">
                <label className="text-[10px] font-mono uppercase text-neutral-400 font-bold">
                  Current Version
                </label>
                <div className="flex items-center gap-2">
                  <span className="px-2.5 py-1 bg-black/40 border border-white/10 text-xs font-mono text-white">
                    v{APP_VERSION}
                  </span>
                  {appUpdate.info?.update_available ? (
                    <span className="inline-flex items-center gap-1 px-2 py-0.5 border border-emerald-500/40 bg-emerald-500/10 text-emerald-300 text-[10px] font-mono font-bold uppercase">
                      <ArrowUpCircle className="w-3 h-3" /> Update Available: v{appUpdate.info.latest_version}
                    </span>
                  ) : appUpdate.info ? (
                    <span className="inline-flex items-center gap-1 px-2 py-0.5 border border-white/10 bg-white/[0.04] text-neutral-400 text-[10px] font-mono font-bold uppercase">
                      <Check className="w-3 h-3 text-emerald-400" /> Up to Date
                    </span>
                  ) : null}
                </div>
              </div>

              <div className="space-y-1">
                <label className="text-[10px] font-mono uppercase text-neutral-400 font-bold">
                  Target Repository
                </label>
                <div className="flex items-center gap-2">
                  <span className="px-2.5 py-1 bg-black/40 border border-white/10 text-xs font-mono text-neutral-300 truncate">
                    {appUpdate.info?.repo_slug || 'Matt4C6F656872/Rhystic-Tracker-Web'}
                  </span>
                  {appUpdate.info?.release_url && (
                    <a
                      href={appUpdate.info.release_url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="p-1 border border-white/10 bg-white/[0.04] hover:bg-white/[0.08] text-neutral-300 text-xs cursor-pointer"
                      title="View release on GitHub"
                    >
                      <ExternalLink className="w-3.5 h-3.5" />
                    </a>
                  )}
                </div>
              </div>
            </div>

            {/* If an update is available: details banner */}
            {appUpdate.info?.update_available && (
              <div className="p-3 border border-emerald-500/30 bg-emerald-500/[0.06] space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-xs font-bold text-white uppercase font-sans flex items-center gap-1.5">
                    <ArrowUpCircle className="w-4 h-4 text-emerald-400" />
                    New Release Available: v{appUpdate.info.latest_version}
                  </span>
                  {appUpdate.info.asset_name && (
                    <span className="text-[10px] font-mono text-neutral-400">
                      Asset: {appUpdate.info.asset_name}
                    </span>
                  )}
                </div>
                {appUpdate.info.release_name && (
                  <p className="text-xs font-sans text-neutral-200 font-medium">
                    {appUpdate.info.release_name}
                  </p>
                )}
                {appUpdate.info.release_notes && (
                  <div className="text-[11px] font-sans text-neutral-400 max-h-24 overflow-y-auto border-t border-white/5 pt-1.5 whitespace-pre-wrap">
                    {appUpdate.info.release_notes}
                  </div>
                )}
                <div className="text-[10px] font-mono text-neutral-400 pt-1 border-t border-white/5 flex items-center gap-1.5">
                  <span>Installation Strategy:</span>
                  <span className="text-neutral-200 font-bold">
                    {appUpdate.info.install_mode === 'self_replace'
                      ? 'In-Place Binary Replacement (Restart Required)'
                      : appUpdate.info.install_mode === 'installer'
                      ? 'Platform Package / Installer Download'
                      : `Manual Download (${appUpdate.info.install_mode_reason || 'Release Page'})`}
                  </span>
                </div>
              </div>
            )}

            {/* Success message / Restart prompt */}
            {appUpdate.installResult && (
              <div className="p-3 border border-emerald-500/40 bg-emerald-500/10 space-y-2">
                <p className="text-xs font-mono text-emerald-300">
                  {appUpdate.installResult.message}
                </p>
                {appUpdate.installResult.restart_required && (
                  <button
                    type="button"
                    onClick={() => restartApp()}
                    className="px-3 py-1.5 border border-emerald-400/50 bg-emerald-500/20 hover:bg-emerald-500/30 text-xs font-mono font-bold uppercase tracking-wider text-emerald-200 flex items-center gap-1.5 cursor-pointer"
                  >
                    <RotateCcw className="w-3.5 h-3.5" /> Restart Rhystic Tracker Now
                  </button>
                )}
              </div>
            )}

            {/* Errors */}
            {(appUpdate.error || appUpdate.installError) && (
              <div className="p-3 border border-red-500/30 bg-red-500/10 text-xs font-mono text-red-300 flex items-center gap-2">
                <AlertCircle className="w-4 h-4 shrink-0" />
                <span>{appUpdate.installError || appUpdate.error}</span>
              </div>
            )}

            {/* Action Bar */}
            <div className="flex flex-wrap items-center justify-between gap-3 pt-2 border-t border-white/10">
              <div className="flex items-center gap-2">
                {/* Check for updates button */}
                <button
                  type="button"
                  disabled={appUpdate.checking || appUpdate.installing}
                  onClick={() => checkForAppUpdate({ force: true })}
                  className="px-3.5 py-1.5 border border-white/15 bg-white/[0.04] hover:bg-white/[0.08] active:scale-95 disabled:opacity-40 text-xs font-mono font-bold uppercase tracking-wider text-white transition-all flex items-center gap-1.5 cursor-pointer"
                >
                  <RefreshCw className={`w-3.5 h-3.5 ${appUpdate.checking ? 'animate-spin' : ''}`} />
                  {appUpdate.checking ? 'Checking…' : 'Check for Updates'}
                </button>

                {/* One-click Update trigger */}
                {appUpdate.info?.update_available && !appUpdate.installResult?.restart_required && (
                  <button
                    type="button"
                    disabled={appUpdate.installing}
                    onClick={() => installAppUpdate()}
                    style={{ backgroundColor: MTG_COLORS.green.bg, borderColor: MTG_COLORS.green.border }}
                    className="px-4 py-1.5 border hover:brightness-125 active:scale-95 disabled:opacity-40 text-xs font-mono font-bold uppercase tracking-wider text-white transition-all flex items-center gap-1.5 cursor-pointer"
                  >
                    <Download className={`w-3.5 h-3.5 ${appUpdate.installing ? 'animate-bounce' : ''}`} />
                    {appUpdate.installing ? 'Updating…' : `Update to v${appUpdate.info.latest_version}`}
                  </button>
                )}
              </div>

              {/* Startup auto-check toggle */}
              <div className="flex items-center gap-2">
                <span className="text-[11px] font-sans text-neutral-400">
                  Automatically check for updates on startup
                </span>
                <button
                  type="button"
                  role="switch"
                  aria-checked={appUpdate.autoCheck}
                  onClick={() => setAutoCheckUpdates(!appUpdate.autoCheck)}
                  style={appUpdate.autoCheck ? { backgroundColor: MTG_COLORS.green.bg, borderColor: MTG_COLORS.green.border } : undefined}
                  className={`relative inline-flex items-center h-5 w-9 shrink-0 cursor-pointer border transition-colors ${
                    appUpdate.autoCheck ? '' : 'bg-white/[0.04] border-white/15'
                  }`}
                >
                  <span
                    style={appUpdate.autoCheck ? { backgroundColor: MTG_COLORS.green.base } : undefined}
                    className={`inline-block h-3 w-3 transform transition-transform ${
                      appUpdate.autoCheck ? 'translate-x-5' : 'translate-x-1 bg-neutral-500'
                    }`}
                  />
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
