#!/usr/bin/env bash
# Build, install and launch Rhystic Tracker for the current user (no root needed).
#
#   ./scripts/install-linux.sh            build + install + launch
#   ./scripts/install-linux.sh --no-build install the existing release binary + launch
#   ./scripts/install-linux.sh --no-run   build + install, don't launch
#
# On immutable distros (Bazzite / Silverblue) the GTK/WebKit dev libraries are not
# on the host, so the build step is automatically run inside a distrobox
# (default: fedora-dev, override with RHYSTIC_DISTROBOX=<name>).
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BOX="${RHYSTIC_DISTROBOX:-fedora-dev}"
BIN_SRC="$PROJECT_DIR/src-tauri/target/release/rhystic-tracker"
BIN_DST="$HOME/.local/bin/rhystic-tracker"
DESKTOP_DST="$HOME/.local/share/applications/rhystic-tracker.desktop"
ICON_ROOT="$HOME/.local/share/icons/hicolor"
LOG_FILE="${XDG_STATE_HOME:-$HOME/.local/state}/rhystic-tracker/rhystic-tracker.log"

DO_BUILD=1
DO_RUN=1
BUILD_ONLY=0
for arg in "$@"; do
  case "$arg" in
    --no-build) DO_BUILD=0 ;;
    --no-run) DO_RUN=0 ;;
    --build-only) BUILD_ONLY=1 ;;
    *) echo "Unknown option: $arg" >&2; exit 2 ;;
  esac
done

has_build_deps() {
  command -v pkg-config >/dev/null 2>&1 && pkg-config --exists webkit2gtk-4.1 gtk+-3.0
}

build() {
  cd "$PROJECT_DIR"
  [ -s "$HOME/.nvm/nvm.sh" ] && . "$HOME/.nvm/nvm.sh" >/dev/null
  [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
  [ -d node_modules ] || npm install
  npm run build:app
}

# ---- 1. Build ---------------------------------------------------------------
if [ "$BUILD_ONLY" = 1 ]; then
  build
  exit 0
fi

if [ "$DO_BUILD" = 1 ]; then
  if has_build_deps; then
    echo "==> Building on this system"
    build
  elif command -v distrobox >/dev/null 2>&1; then
    echo "==> Building inside distrobox '$BOX' (host lacks GTK/WebKit dev libraries)"
    distrobox enter "$BOX" -- bash "$PROJECT_DIR/scripts/install-linux.sh" --build-only
  else
    echo "ERROR: webkit2gtk-4.1 / gtk3 development packages not found and distrobox unavailable." >&2
    exit 1
  fi
fi

[ -x "$BIN_SRC" ] || { echo "ERROR: $BIN_SRC not found - build first." >&2; exit 1; }

# ---- 2. Stop any running instance (single-instance would hand off to it) -----
# SIGTERM first so the app can flush SQLite/WAL and sync state, then escalate
# only if it hasn't exited after ~5s.
if pgrep -x rhystic-tracker >/dev/null 2>&1; then
  echo "==> Stopping running Rhystic Tracker"
  pkill -x rhystic-tracker || true
  for _ in $(seq 1 20); do pgrep -x rhystic-tracker >/dev/null || break; sleep 0.25; done
  pkill -9 -x rhystic-tracker 2>/dev/null || true
fi

# ---- 3. Clean uninstall of the previous version -------------------------------
# Removes ONLY launcher artefacts (binary, .desktop entries, hicolor icons).
# User data is never touched:
#   ~/.config/rhystic-tracker/         rhystic.db, settings.json, sync_settings.json
#   ~/.config/com.rhystic.tracker/     avatar / card-image caches
#   ~/.local/share/com.rhystic.tracker/ WebView localStorage (UI preferences)
#   ~/.local/share/rhystic-tracker/    avatar-extractor Python venv
PROTECTED_DIRS=(
  "${XDG_CONFIG_HOME:-$HOME/.config}/rhystic-tracker"
  "${XDG_CONFIG_HOME:-$HOME/.config}/com.rhystic.tracker"
  "${XDG_DATA_HOME:-$HOME/.local/share}/com.rhystic.tracker"
  "${XDG_DATA_HOME:-$HOME/.local/share}/rhystic-tracker"
)

# Delete a single regular file / symlink, refusing directories and anything
# that resolves inside a protected user-data directory.
safe_rm() {
  local target="$1" resolved protected
  [ -e "$target" ] || [ -L "$target" ] || return 0
  if [ -d "$target" ] && [ ! -L "$target" ]; then
    echo "    ! refusing to remove directory: $target" >&2
    return 0
  fi
  resolved="$(readlink -f "$target" 2>/dev/null || echo "$target")"
  for protected in "${PROTECTED_DIRS[@]}"; do
    case "$resolved" in
      "$protected"|"$protected"/*)
        echo "    ! refusing to remove protected user data: $target" >&2
        return 0 ;;
    esac
  done
  case "$(basename "$resolved")" in
    *.db|*.db-wal|*.db-shm|settings*.json|sync_settings*.json)
      echo "    ! refusing to remove user data file: $target" >&2
      return 0 ;;
  esac
  rm -f -- "$target" && echo "    - removed $target"
}

echo "==> Removing previous installation (user data is preserved)"
# Binary plus leftovers from interrupted installs / in-app updates.
safe_rm "$BIN_DST"
safe_rm "$BIN_DST.new"
safe_rm "$BIN_DST.update-new"
# Launchers written by this script, install.sh, or older releases.
APPS_DIR="$(dirname "$DESKTOP_DST")"
safe_rm "$DESKTOP_DST"
safe_rm "$APPS_DIR/com.rhystic.tracker.desktop"
safe_rm "$APPS_DIR/Rhystic Tracker.desktop"
# Icons at every hicolor size we (or install.sh) have ever shipped.
for size in 16 24 32 48 64 128 256 512 1024; do
  safe_rm "$ICON_ROOT/${size}x${size}/apps/rhystic-tracker.png"
done
safe_rm "$ICON_ROOT/scalable/apps/rhystic-tracker.svg"
safe_rm "$HOME/.local/share/pixmaps/rhystic-tracker.png"

# ---- 4. Install -------------------------------------------------------------
echo "==> Installing to $BIN_DST"
mkdir -p "$(dirname "$BIN_DST")" "$(dirname "$DESKTOP_DST")" "$(dirname "$LOG_FILE")"
install -m 755 "$BIN_SRC" "$BIN_DST.new"
mv -f "$BIN_DST.new" "$BIN_DST"

ICONS="$PROJECT_DIR/src-tauri/icons"
for size in 32 64 128; do
  if [ -f "$ICONS/${size}x${size}.png" ]; then
    install -Dm644 "$ICONS/${size}x${size}.png" "$ICON_ROOT/${size}x${size}/apps/rhystic-tracker.png"
  fi
done
[ -f "$ICONS/128x128@2x.png" ] && install -Dm644 "$ICONS/128x128@2x.png" "$ICON_ROOT/256x256/apps/rhystic-tracker.png"

cat > "$DESKTOP_DST" <<EOF
[Desktop Entry]
Name=Rhystic Tracker
GenericName=MTG Arena Tracker
Comment=Real-time MTGA match tracking and deck analysis companion
Exec=env GDK_BACKEND=x11 WEBKIT_DISABLE_COMPOSITING_MODE=1 RHYSTIC_ENV=production $BIN_DST
Icon=rhystic-tracker
Terminal=false
Type=Application
Categories=Game;Utility;
StartupNotify=false
Keywords=mtg;magic;arena;tracker;rhystic;
EOF
# ---- 5. Refresh desktop + icon databases ---------------------------------------
# Each step is optional and best-effort: missing tools must never fail the install.
# -t lets gtk-update-icon-cache work on a per-user hicolor dir without index.theme.
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$(dirname "$DESKTOP_DST")" 2>/dev/null || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -f -t -q "$ICON_ROOT" 2>/dev/null || true
command -v xdg-desktop-menu >/dev/null 2>&1 && xdg-desktop-menu forceupdate 2>/dev/null || true
for kbuild in kbuildsycoca6 kbuildsycoca5; do
  if command -v "$kbuild" >/dev/null 2>&1; then "$kbuild" >/dev/null 2>&1 || true; break; fi
done

echo "==> Installed $(stat -c '%y' "$BIN_DST" | cut -d. -f1)"

# ---- 6. Launch --------------------------------------------------------------
if [ "$DO_RUN" = 1 ]; then
  echo "==> Launching (log: $LOG_FILE)"
  # Detach fully and send stdout/stderr to a file: the backend prints a lot, and a
  # closed terminal would otherwise break its log processing.
  env GDK_BACKEND=x11 WEBKIT_DISABLE_COMPOSITING_MODE=1 RHYSTIC_ENV=production \
    setsid nohup "$BIN_DST" >"$LOG_FILE" 2>&1 < /dev/null &
  disown || true
fi
echo "Done. You can also start it from your app menu: 'Rhystic Tracker'."

