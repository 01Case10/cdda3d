#!/usr/bin/env bash
# Golden image-diff gate wrapper (Phase 2g.3, gates 2.10 / 5.13).
#
# Usage:
#   tools/golden.sh            run the golden comparison
#   tools/golden.sh --update   regenerate the committed goldens
#                              (UPDATE_GOLDEN=1, mirroring the
#                              voxelizer's 5.1 golden flow)
#
# Goldens live in crates/engine/tests/golden/. On failure the
# harness writes <scene>.actual.png and <scene>.diff.png (magenta
# = mismatched pixel) to target/golden-failures/ and names both
# paths in its output. With no wgpu adapter the harness prints a
# named NOT-RUN banner and writes SKIPPED.txt there instead — a
# no-adapter run proves nothing and is never a pass.
set -u
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
if [ "${1:-}" = "--update" ]; then
  UPDATE_GOLDEN=1 exec cargo test -p engine --test golden_gate gate_2g3_golden_image_diff -- --nocapture
else
  exec cargo test -p engine --test golden_gate -- --nocapture
fi
