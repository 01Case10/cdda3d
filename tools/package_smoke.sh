#!/bin/sh
# Gate 8.11 (Linux half): clean-directory install smoke.
#
# Assembles a minimal "package" — the game-side binary and the
# frontend client as separate executables plus a VERSION stamp,
# laid out the way an installed tree would be — then, from an
# unrelated working directory with a scrubbed environment, starts
# the game binary and connects the client. Reaching gameplay at
# this level means: handshake accepted, TileDefs received, a
# Snapshot received, and an input round-tripping. (The Windows
# half needs a Windows machine; the live fork is 0.1/0.2's
# condition, so the stub server stands in as the game binary,
# exactly as it has since Phase 1.)
set -eu
ROOT=$(cd "$(dirname "$0")/.." && pwd)
export PATH="$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin"

echo "--- building package contents ---"
(cd "$ROOT/tools/stub-server" && sh build.sh >/dev/null)
cargo build -p proto --bin proto-client --manifest-path "$ROOT/Cargo.toml" 2>/dev/null

PKG=$(mktemp -d /tmp/cdda-voxel-pkg.XXXXXX)
trap 'rm -rf "$PKG"' EXIT
mkdir -p "$PKG/bin"
cp "$ROOT/tools/stub-server/stub-server" "$PKG/bin/game"
cp "$ROOT/target/debug/proto-client" "$PKG/bin/frontend"
# The install stamp (crates/release::PackageStamp's text format).
printf 'frontend=0.1.0\nfork=6a3e59e08842de34ff3f111c7f098abeffb708a5\nprotocol=2\n' > "$PKG/VERSION"

echo "--- installed layout ---"
find "$PKG" -type f | sort

echo "--- launching from a clean directory + environment ---"
WORK=$(mktemp -d /tmp/cdda-voxel-run.XXXXXX)
trap 'rm -rf "$PKG" "$WORK"' EXIT
cd "$WORK"
env -i PATH=/usr/bin:/bin "$PKG/bin/game" "$WORK/game.sock" 2>"$WORK/game.log" &
GAME_PID=$!
# Wait for the socket to appear (bounded).
i=0
while [ ! -S "$WORK/game.sock" ] && [ $i -lt 50 ]; do
  sleep 0.1
  i=$((i + 1))
done
[ -S "$WORK/game.sock" ] || { echo "FAIL: game binary never listened"; cat "$WORK/game.log"; kill $GAME_PID 2>/dev/null || true; exit 1; }

set +e
env -i PATH=/usr/bin:/bin "$PKG/bin/frontend" "$WORK/game.sock" > "$WORK/client.log" 2>&1
CLIENT_RC=$?
set -e
kill $GAME_PID 2>/dev/null || true
wait $GAME_PID 2>/dev/null || true

cat "$WORK/client.log"
[ $CLIENT_RC -eq 0 ] || { echo "FAIL: frontend exited $CLIENT_RC"; exit 1; }
grep -q "HELLO game=" "$WORK/client.log" || { echo "FAIL: no handshake"; exit 1; }
grep -q "TILEDEFS" "$WORK/client.log" || { echo "FAIL: no TileDefs"; exit 1; }
grep -q "SNAPSHOT seq=" "$WORK/client.log" || { echo "FAIL: no snapshot (never reached gameplay)"; exit 1; }
grep -q "INPUT round-trip ok" "$WORK/client.log" || { echo "FAIL: input did not round-trip"; exit 1; }
grep -q "SHUTDOWN sent; client done" "$WORK/client.log" || { echo "FAIL: no clean shutdown"; exit 1; }
grep -q "fork=6a3e59e0" "$PKG/VERSION" || { echo "FAIL: VERSION stamp missing fork commit"; exit 1; }
echo "PACKAGE SMOKE PASS (install -> handshake -> TileDefs -> snapshot -> input -> clean shutdown)"
