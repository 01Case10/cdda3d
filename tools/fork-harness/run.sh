#!/bin/sh
# Build + run the Phase 3 fork harness (exporter core vs mock world).
set -e
cd "$(dirname "$0")"
ROOT=../..
CXX="${CXX:-g++}"
CXXFLAGS="-std=c++17 -O1 -Wall -Wextra -I$ROOT/fork/src/voxel_ipc -I$ROOT/tools/stub-server/third_party -I."
mkdir -p out
$CXX $CXXFLAGS \
  fork_harness.cpp mock_world.cpp \
  $ROOT/fork/src/voxel_ipc/voxel_exporter.cpp \
  $ROOT/fork/src/voxel_ipc/voxel_server.cpp \
  $ROOT/fork/src/voxel_ipc/voxel_input.cpp \
  $ROOT/fork/src/voxel_ipc/voxel_frontend.cpp \
  -o out/fork-harness
./out/fork-harness out/snapshot.bin
echo "--- Rust proto-crate decode check ---"
export PATH="$HOME/.cargo/bin:$PATH"
if [ -f rust-decode/Cargo.toml ] && command -v cargo >/dev/null 2>&1; then
  (cd rust-decode && cargo run --quiet -- ../out/snapshot.bin)
else
  echo "RUST-DECODE SKIP: cargo not found"
fi
