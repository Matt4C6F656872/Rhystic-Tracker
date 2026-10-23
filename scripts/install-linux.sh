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
if pgrep -x rhystic-tracker >/dev/null 2>&1; then
  echo "==> Stopping running Rhystic Tracker"
  pkill -x rhystic-tracker || true
  for _ in $(seq 1 20); do pgrep -x rhystic-tracker >/dev/null || break; sleep 0.25; done
  pkill -9 -x rhystic-tracker 2>/dev/null || true
fi

# ---- 3. Install -------------------------------------------------------------
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
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$(dirname "$DESKTOP_DST")" 2>/dev/null || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q "$ICON_ROOT" 2>/dev/null || true

echo "==> Installed $(stat -c '%y' "$BIN_DST" | cut -d. -f1)"

# ---- 4. Launch --------------------------------------------------------------
if [ "$DO_RUN" = 1 ]; then
  echo "==> Launching (log: $LOG_FILE)"
  # Detach fully and send stdout/stderr to a file: the backend prints a lot, and a
  # closed terminal would otherwise break its log processing.
  env GDK_BACKEND=x11 WEBKIT_DISABLE_COMPOSITING_MODE=1 RHYSTIC_ENV=production \
    setsid nohup "$BIN_DST" >"$LOG_FILE" 2>&1 < /dev/null &
  disown || true
fi
echo "Done. You can also start it from your app menu: 'Rhystic Tracker'."

