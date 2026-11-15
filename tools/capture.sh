#!/usr/bin/env bash
# Headless capture wrapper (Phase 2 deliverable, plan §5 Phase 2).
#
# Usage:
#   tools/capture.sh [--snapshot stub-house] [--camera x,y,yaw,pitch]
#                    [--distance d] [--cutaway z] [--renderer raster|rt]
#                    [--size WxH] --out file.png
#
# Thin wrapper around `cargo run -p engine -- --headless-capture ...`.
# The engine's exit code is propagated: on a machine with no wgpu
# adapter the engine exits non-zero with
# "headless-capture: no wgpu adapter available" and no PNG is written —
# this script does the same.
set -u
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
exec cargo run -q -p engine -- --headless-capture "$@"
