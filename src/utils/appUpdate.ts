import { useSyncExternalStore } from 'react';
import { invoke } from '@tauri-apps/api/core';

/**
 * Shared in-app update state.
 *
 * The sidebar indicator and the Settings → General "Application Updates"
 * section both read from this single store so GitHub's (unauthenticated,
 * 60 req/h) releases API is queried at most once per check, no matter how many
 * components are mounted.
 */

/** Mirrors `AppUpdateInfo` in src-tauri/src/commands/system.rs. */
export interface AppUpdateInfo {
  update_available: boolean;
  current_version: string;
  latest_version: string;
  repo_slug: string;
  release_url: string;
  release_notes: string | null;
  download_url: string | null;
  release_name: string | null;
  published_at: string | null;
  asset_name: string | null;
  /** How `install_app_update` will apply the update on this platform/install. */
  install_mode: 'self_replace' | 'installer' | 'release_page';
  /** Human-readable reason when install_mode falls back to `release_page`. */
  install_mode_reason: string | null;
}

/** Mirrors `AppUpdateInstallResult` in src-tauri/src/commands/system.rs. */
export interface AppUpdateInstallResult {
  action: 'binary_replaced' | 'installer_launched' | 'opened_release_page' | 'up_to_date';
  message: string;
  latest_version: string;
  restart_required: boolean;
  will_exit: boolean;
  checksum_verified: boolean;
  path: string | null;
}

export interface AppUpdateState {
  checking: boolean;
  info: AppUpdateInfo | null;
  error: string | null;
  lastChecked: number | null;
  installing: boolean;
  installResult: AppUpdateInstallResult | null;
  installError: string | null;
  autoCheck: boolean;
}

const AUTO_CHECK_KEY = 'autoCheckUpdates';
/** Re-use a successful result for this long unless the user forces a re-check. */
const CHECK_CACHE_MS = 6 * 60 * 60 * 1000;

const readAutoCheck = (): boolean => {
  try {
    return localStorage.getItem(AUTO_CHECK_KEY) !== 'false';
  } catch {
    return true;
  }
};

let state: AppUpdateState = {
  checking: false,
  info: null,
  error: null,
  lastChecked: null,
  installing: false,
  installResult: null,
  installError: null,
  autoCheck: readAutoCheck(),
};

const listeners = new Set<() => void>();
let inFlight: Promise<AppUpdateInfo | null> | null = null;

const setState = (patch: Partial<AppUpdateState>) => {
  state = { ...state, ...patch };
  listeners.forEach((l) => l());
};

const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};

const getSnapshot = () => state;

const errorMessage = (err: unknown, fallback: string) =>
  typeof err === 'string' ? err : err instanceof Error ? err.message : fallback;

/**
 * Query the configured GitHub repository for the latest release.
 * Concurrent callers share one request; recent results are cached unless `force`.
 */
export function checkForAppUpdate(options: { force?: boolean } = {}): Promise<AppUpdateInfo | null> {
  if (inFlight) return inFlight;
  if (
    !options.force &&
    state.info &&
    state.lastChecked !== null &&
    Date.now() - state.lastChecked < CHECK_CACHE_MS
  ) {
    return Promise.resolve(state.info);
  }

  setState({ checking: true, error: null });
  inFlight = invoke<AppUpdateInfo>('check_for_app_update')
    .then((info) => {
      setState({ info, checking: false, lastChecked: Date.now() });
      return info;
    })
    .catch((err) => {
      setState({
        checking: false,
        error: errorMessage(err, 'Failed to check for updates'),
        lastChecked: Date.now(),
      });
      return null;
    })
    .finally(() => {
      inFlight = null;
    });
  return inFlight;
}

/**
 * Apply the latest release using the platform strategy reported by the backend,
 * or just open the GitHub release page when `preferReleasePage` is set.
 */
export async function installAppUpdate(preferReleasePage = false): Promise<AppUpdateInstallResult | null> {
  if (state.installing) return null;
  setState({ installing: true, installError: null, installResult: null });
  try {
    const result = await invoke<AppUpdateInstallResult>('install_app_update', { preferReleasePage });
    setState({ installing: false, installResult: result });
    return result;
  } catch (err) {
    setState({ installing: false, installError: errorMessage(err, 'Update failed') });
    return null;
  }
}

/** Relaunch the app (used after the Linux binary was replaced in place). */
export async function restartApp(): Promise<void> {
  try {
    await invoke('restart_app');
  } catch (err) {
    setState({ installError: errorMessage(err, 'Failed to restart Rhystic Tracker') });
  }
}

export function setAutoCheckUpdates(enabled: boolean) {
  try {
    localStorage.setItem(AUTO_CHECK_KEY, String(enabled));
  } catch {
    /* storage unavailable — keep in-memory value only */
  }
  setState({ autoCheck: enabled });
}

export function useAppUpdate(): AppUpdateState {
  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}

