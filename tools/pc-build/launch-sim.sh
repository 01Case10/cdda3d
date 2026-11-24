#!/usr/bin/env bash
# Launch the fake engine (sim-server) and the renderer (voxel-engine)
# against it, in a window -- Phase 11 follow-up Lane A's
# --connect-window path, packaged as one command.
#
# Usage:
#   tools/pc-build/launch-sim.sh [--scene <name>] [--seed <n>]
#                                [--record <path>] [--skip-build]
#                                [--debug] [--headless] [--socket <path>]
#                                [--renderer rt|raster]
#
# --renderer picks the windowed renderer: rt (ray tracing) is the
# default since Lane H (Joey's call, 2026-11-24); pass raster for
# the greedy-mesher renderer.
#
# What it does, in order:
#   1. Builds the release binaries (sim-server + voxel-engine).
#      --debug uses the debug profile instead; --skip-build reuses
#      whatever is already built.
#   2. Starts sim-server in the background on a unix socket
#      (default scene: street_block; log: target/sim-launch/server.log).
#   3. Waits for the socket to appear (aborts with the server log if
#      the server dies first).
#   4. Runs voxel-engine --connect-window <socket> in the foreground.
#      Play with the game keys: hjkl / yubn / arrows to move, o/c
#      open/close, s smash, g get, d drop, i inventory, & crafting,
#      ; look, . wait, >/< stairs, Esc back. Close the window to end
#      the session.
#   5. On engine exit, stops sim-server and removes the socket files.
#
# --headless is a smoke mode: instead of a window it runs
# voxel-engine --connect with a few scripted keys (jjojjjQ -- walk
# out of the house and quit), proving the whole sequence end to end
# on a machine with no display.
set -u
cd "$(dirname "$0")/../.."
export PATH="$HOME/.cargo/bin:$PATH"

SCENE=street_block
SEED=""
RECORD=""
PROFILE=release
SKIP_BUILD=0
HEADLESS=0
RENDERER=rt
SOCK=/tmp/cdda-sim.sock

while [ $# -gt 0 ]; do
  case "$1" in
    --scene) SCENE="$2"; shift 2 ;;
    --seed) SEED="$2"; shift 2 ;;
    --record) RECORD="$2"; shift 2 ;;
    --renderer) RENDERER="$2"; shift 2 ;;
    --socket) SOCK="$2"; shift 2 ;;
    --debug) PROFILE=debug; shift ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    --headless) HEADLESS=1; shift ;;
    *) echo "launch-sim.sh: unknown argument: $1" >&2; exit 2 ;;
  esac
done

case "$RENDERER" in
  rt|raster) ;;
  *) echo "launch-sim.sh: --renderer must be rt|raster (got '$RENDERER')" >&2; exit 2 ;;
esac

# The proto crate's build.rs needs flatc exactly 24.3.25 (located via
# $FLATC, project-local copies, then PATH). Fail early with the fix
# instead of dying inside the cargo build.
if [ -z "${FLATC:-}" ] && ! command -v flatc >/dev/null 2>&1 \
   && [ ! -x ../tools/flatbuffers/flatc ] && [ ! -x /tmp/flatc ]; then
  echo "launch-sim.sh: flatc 24.3.25 not found." >&2
  echo "  Install flatbuffers 24.3.25, or set FLATC=/path/to/flatc." >&2
  echo "  (The PowerShell twin downloads the pinned Windows binary automatically.)" >&2
  exit 1
fi

BIN_DIR="target/$PROFILE"
SERVER_LOG="target/sim-launch/server.log"
mkdir -p target/sim-launch

if [ "$SKIP_BUILD" -eq 0 ]; then
  echo "== building sim-server + voxel-engine ($PROFILE) =="
  if [ "$PROFILE" = release ]; then
    cargo build --release -p sim-server -p engine || exit 1
  else
    cargo build -p sim-server -p engine || exit 1
  fi
fi
[ -x "$BIN_DIR/sim-server" ] || { echo "missing $BIN_DIR/sim-server (build first)" >&2; exit 1; }
[ -x "$BIN_DIR/voxel-engine" ] || { echo "missing $BIN_DIR/voxel-engine (build first)" >&2; exit 1; }

rm -f "$SOCK" "$SOCK.ui-shm"

SERVER_ARGS=("$SOCK" --scene "$SCENE" --defs tools/sim-defs/generated/defs.json)
[ -n "$SEED" ] && SERVER_ARGS+=(--seed "$SEED")
[ -n "$RECORD" ] && SERVER_ARGS+=(--record "$RECORD")

echo "== starting sim-server (scene: $SCENE, socket: $SOCK) =="
"$BIN_DIR/sim-server" "${SERVER_ARGS[@]}" >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!
cleanup() {
  kill "$SERVER_PID" 2>/dev/null
  wait "$SERVER_PID" 2>/dev/null
  rm -f "$SOCK" "$SOCK.ui-shm"
}
trap cleanup EXIT

for _ in $(seq 1 150); do
  [ -S "$SOCK" ] && break
  kill -0 "$SERVER_PID" 2>/dev/null || { echo "sim-server exited early; log:" >&2; cat "$SERVER_LOG" >&2; exit 1; }
  sleep 0.2
done
[ -S "$SOCK" ] || { echo "socket $SOCK never appeared; server log:" >&2; cat "$SERVER_LOG" >&2; exit 1; }

if [ "$HEADLESS" -eq 1 ]; then
  echo "== headless smoke: voxel-engine --connect (keys: jjojjjQ) =="
  printf 'jjojjjQ' | "$BIN_DIR/voxel-engine" --connect "$SOCK"
  STATUS=$?
else
  echo "== launching renderer: voxel-engine --connect-window (renderer: $RENDERER; close the window to end) =="
  ENGINE_ARGS=(--connect-window "$SOCK" --renderer "$RENDERER")
  if [ -f assets/ultica_run1/materials.json ]; then
    ENGINE_ARGS+=(--textures assets/ultica_run1)
    echo "   live furniture props: ON (model table found in assets/ultica_run1)"
  else
    echo "   live furniture props: OFF (no assets/ultica_run1/materials.json -- extract the models pack there to enable)"
  fi
  "$BIN_DIR/voxel-engine" "${ENGINE_ARGS[@]}"
  STATUS=$?
fi

echo "== session ended (engine exit $STATUS); server log tail: =="
tail -5 "$SERVER_LOG" 2>/dev/null
exit "$STATUS"
