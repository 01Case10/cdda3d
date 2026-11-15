#!/usr/bin/env bash
# CDDA Voxel Frontend -- PC build for gates 0.1 / 0.2 and the Phase 3 fork tests.
#
# Runs INSIDE an MSYS2 UCRT64 shell (Build-CddaVoxel.ps1 launches it that
# way). It can also be run by hand from a UCRT64 terminal:
#   MSYSTEM=UCRT64 bash -l tools/pc-build/build-cdda-voxel.sh
#
# Stages (CDDA_STAGE=all|baseline|fork, default all):
#   deps     pacman-install the UCRT64 toolchain (skip with CDDA_SKIP_DEPS=1)
#   clone    full checkout of CleverRaven/Cataclysm-DDA at the pinned commit
#            (fork-src/cdda is a SPARSE checkout -- it cannot build or run
#            cata_test, which needs data/, lang/ and the full tests/ tree)
#   baseline configure + build the PRISTINE curses build (TILES=OFF,
#            CURSES=ON, TESTS=ON), prove 0.1 (binary runs), run the full
#            cata_test suite and record the 0.2 baseline (JUnit XML +
#            summary) under results/
#   fork     stage fork/src/voxel_ipc + the Catch2 cases into the checkout,
#            patch src/CMakeLists.txt (build lines only -- hooks H1-H7 are
#            NOT applied, see README), build, run cata_test "[voxel]"
#            (gates 3.1/3.3/3.5/3.5b/3.6/3.7/3.8 in-game), then re-run the
#            full suite and diff the failure set against the baseline
#            (gate 3.19: no NEW failures)
#
# Inputs (environment, set by the .ps1 wrapper or by hand):
#   CDDAVOXEL_REPO_WIN  Windows path of this cdda-voxel repo (fork/ lives here)
#   CDDA_WORK_WIN       Windows path of the work dir (clone, builds, results)
#   CDDA_STAGE          all | baseline | fork        (default all)
#   CDDA_JOBS           parallel jobs                (default nproc, cap 12)
#   CDDA_SKIP_DEPS      1 = do not run pacman
#   CDDA_CLEAN          1 = delete build dirs before configuring
#   CDDA_RUN_FULL       1 = run full cata_test suites (default 1; 0 = quick:
#                       baseline build + 0.1 only, fork [voxel] only)
#   CDDA_DRY_RUN        1 = print the plan and exit before touching anything
#
# Exit codes: 0 = every stage that ran passed its gate; 2 = environment
# problem (wrong shell, missing tool); 3 = build failure; 4 = a test gate
# failed (details in results/). Test FAILURES inside cata_test are data,
# not script errors: the baseline records whatever upstream does, and the
# fork gate compares against that record.
set -u

PINNED_COMMIT="6a3e59e08842de34ff3f111c7f098abeffb708a5"
UPSTREAM_URL="https://github.com/CleverRaven/Cataclysm-DDA.git"
UPSTREAM_TARBALL="https://github.com/CleverRaven/Cataclysm-DDA/archive/${PINNED_COMMIT}.tar.gz"
BUILD_TYPE="RelWithDebInfo"
RNG_SEED="12345"

STAGE="${CDDA_STAGE:-all}"
SKIP_DEPS="${CDDA_SKIP_DEPS:-0}"
CLEAN="${CDDA_CLEAN:-0}"
RUN_FULL="${CDDA_RUN_FULL:-1}"
DRY_RUN="${CDDA_DRY_RUN:-0}"

if [ -z "${CDDA_JOBS:-}" ]; then
    JOBS="$(nproc 2>/dev/null || echo 4)"
    if [ "$JOBS" -gt 12 ]; then JOBS=12; fi
else
    JOBS="$CDDA_JOBS"
fi

# --- path resolution -------------------------------------------------------
win_to_msys() {
    # $1 = Windows path (C:\a\b or C:/a/b) -> MSYS path (/c/a/b).
    if command -v cygpath >/dev/null 2>&1; then
        cygpath -u "$1"
    else
        printf '%s' "$1" | sed -e 's|\\|/|g' -e 's|^\([A-Za-z]\):|/\L\1|'
    fi
}

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if [ -n "${CDDAVOXEL_REPO_WIN:-}" ]; then
    REPO="$(win_to_msys "$CDDAVOXEL_REPO_WIN")"
else
    REPO="$(cd "$SCRIPT_DIR/../.." && pwd)"
fi
if [ -n "${CDDA_WORK_WIN:-}" ]; then
    WORK="$(win_to_msys "$CDDA_WORK_WIN")"
else
    WORK="$HOME/cdda-voxel-build"
fi

SRC="$WORK/Cataclysm-DDA"
BUILD_BASE="$WORK/build-baseline"
BUILD_FORK="$WORK/build-fork"
RESULTS="$WORK/results"
LOGS="$WORK/logs"
STAMP="$(date +%Y%m%d-%H%M%S)"

say()  { printf '\n=== %s ===\n' "$*"; }
info() { printf '%s\n' "$*"; }
die()  { printf 'ERROR: %s\n' "$*" >&2; exit "${2:-2}"; }

if [ "$DRY_RUN" = "1" ]; then
    say "DRY RUN -- plan only, nothing will be touched"
    info "repo (fork/ source) : $REPO"
    info "work dir            : $WORK"
    info "CDDA checkout       : $SRC  @ $PINNED_COMMIT"
    info "stage               : $STAGE   jobs: $JOBS   full suites: $RUN_FULL"
    info "1. deps: pacman -S --needed (UCRT64 gcc/cmake/ninja/ccache/ncurses/zlib/gettext/pkgconf, msys git/python)"
    info "2. clone: git clone --filter=blob:none $UPSTREAM_URL (tarball fallback), checkout $PINNED_COMMIT, verify rev-parse + pristine tree"
    info "3. baseline: cmake -G Ninja -DCMAKE_BUILD_TYPE=$BUILD_TYPE -DTILES=OFF -DCURSES=ON -DSOUND=OFF -DTESTS=ON ; build cataclysm + cata_test"
    info "4. gate 0.1: cataclysm.exe --help (fallback --jsonverify) from the source dir"
    info "5. gate 0.2: full cata_test (JUnit XML) -> results/baseline-summary.txt + baseline-failed.txt"
    info "6. fork: copy fork/src/voxel_ipc (+ flatbuffers 24.3.25 headers from tools/stub-server/third_party) and fork/tests/voxel_exporter_tests.cpp in; patch src/CMakeLists.txt via patch_cmake.py"
    info "7. fork build in a SEPARATE build dir; cata_test \"[voxel]\" -> gates 3.1/3.3/3.5/3.5b/3.6/3.7/3.8"
    info "8. gate 3.19: full cata_test again; failed-test set diffed vs baseline -> results/comparison.txt"
    exit 0
fi

# --- environment gate ------------------------------------------------------
[ "${MSYSTEM:-}" = "UCRT64" ] || die "MSYSTEM is '${MSYSTEM:-unset}', not UCRT64. Run via Build-CddaVoxel.ps1 or an MSYS2 UCRT64 shell. (Upstream's CMake MSYS2 branch hardcodes the UCRT ncurses import lib; CLANG64/MINGW64 will misconfigure.)"
[ -d "$REPO/fork/src/voxel_ipc" ] || die "fork/src/voxel_ipc not found under repo: $REPO"
[ -f "$REPO/tools/stub-server/third_party/flatbuffers/flatbuffers.h" ] || die "vendored flatbuffers 24.3.25 headers missing: $REPO/tools/stub-server/third_party"
[ -f "$SCRIPT_DIR/patch_cmake.py" ] || die "patch_cmake.py missing next to this script: $SCRIPT_DIR"

mkdir -p "$WORK" "$RESULTS" "$LOGS"
exec > >(tee -a "$LOGS/run-$STAMP.log") 2>&1
info "log: $LOGS/run-$STAMP.log"
info "repo: $REPO"
info "work: $WORK  stage: $STAGE  jobs: $JOBS  build type: $BUILD_TYPE"

# --- deps ------------------------------------------------------------------
if [ "$SKIP_DEPS" != "1" ]; then
    say "deps: pacman (UCRT64 toolchain for a curses + tests build; NO SDL3 -- TILES=OFF never links it)"
    pacman -S --needed --noconfirm \
        git python \
        mingw-w64-ucrt-x86_64-gcc \
        mingw-w64-ucrt-x86_64-cmake \
        mingw-w64-ucrt-x86_64-ninja \
        mingw-w64-ucrt-x86_64-ccache \
        mingw-w64-ucrt-x86_64-ncurses \
        mingw-w64-ucrt-x86_64-zlib \
        mingw-w64-ucrt-x86_64-gettext \
        mingw-w64-ucrt-x86_64-pkgconf \
        || die "pacman failed -- install MSYS2 from https://www.msys2.org and re-run, or pass -SkipDeps if the toolchain is already in place" 2
else
    say "deps: skipped (CDDA_SKIP_DEPS=1)"
fi
for tool in git cmake ninja g++ python3; do
    command -v "$tool" >/dev/null 2>&1 || die "required tool not on PATH in the UCRT64 shell: $tool" 2
done
g++ --version | head -1
cmake --version | head -1

# --- clone / checkout ------------------------------------------------------
say "clone: Cataclysm-DDA @ $PINNED_COMMIT"
if [ -d "$SRC/.git" ]; then
    # This checkout is script-owned scratch: reset exactly the paths the
    # fork stage touches, so a rerun starts from pristine upstream.
    git -C "$SRC" checkout -- src/CMakeLists.txt 2>/dev/null || true
    rm -rf "$SRC/src/voxel_ipc"
    rm -f "$SRC/tests/voxel_exporter_tests.cpp"
    dirty="$(git -C "$SRC" status --porcelain --untracked-files=no)"
    if [ -n "$dirty" ]; then
        info "$dirty"
        die "checkout has local modifications outside the script-owned paths above -- refusing to build a 'baseline' on a dirty tree. Inspect $SRC, or delete it and re-run for a fresh clone." 2
    fi
    git -C "$SRC" checkout -q "$PINNED_COMMIT" 2>/dev/null || {
        git -C "$SRC" fetch -q origin "$PINNED_COMMIT" && git -C "$SRC" checkout -q "$PINNED_COMMIT"
    } || die "could not check out the pinned commit in the existing clone" 3
elif [ -e "$SRC" ]; then
    die "$SRC exists but is not a git clone -- move it aside and re-run" 2
else
    if ! git clone --filter=blob:none "$UPSTREAM_URL" "$SRC"; then
        # Fallback seen on the recon VM: git's pack fetch can fail while a
        # plain HTTPS tarball of the same commit succeeds.
        info "git clone failed -- falling back to the commit tarball"
        mkdir -p "$WORK/dl"
        curl -L -o "$WORK/dl/cdda.tar.gz" "$UPSTREAM_TARBALL" || die "both git clone and tarball download failed" 3
        mkdir -p "$SRC"
        tar -xzf "$WORK/dl/cdda.tar.gz" -C "$SRC" --strip-components=1 || die "tarball extract failed" 3
        git -C "$SRC" init -q && git -C "$SRC" add -A && \
            git -C "$SRC" -c user.email=build@local -c user.name=build commit -qm "pinned upstream $PINNED_COMMIT (tarball)" \
            || die "tarball checkout could not be initialised" 3
        info "NOTE: tarball checkout -- commit verified by tarball name, not rev-parse."
    fi
    git -C "$SRC" checkout -q "$PINNED_COMMIT" || die "checkout of pinned commit failed" 3
fi
ACTUAL="$(git -C "$SRC" rev-parse HEAD 2>/dev/null || echo tarball)"
info "HEAD: $ACTUAL"
if [ "$ACTUAL" != "$PINNED_COMMIT" ] && [ "$ACTUAL" != "tarball" ]; then
    die "HEAD is $ACTUAL, expected $PINNED_COMMIT -- Appendix A evidence is pinned to that commit; do not build a different one under this gate" 3
fi
# Sanity: a buildable tree, not the sparse recon checkout.
for p in CMakeLists.txt src/CMakeLists.txt tests/CMakeLists.txt data/json lang; do
    [ -e "$SRC/$p" ] || die "checkout is missing $p -- this looks like a sparse/partial tree, not a full clone" 3
done

configure() { # $1 = build dir
    cmake -S "$SRC" -B "$1" -G Ninja \
        -DCMAKE_BUILD_TYPE="$BUILD_TYPE" \
        -DTILES=OFF -DCURSES=ON -DSOUND=OFF -DTESTS=ON \
        || die "cmake configure failed in $1 -- first suspects: ncurses (mingw-w64-ucrt-x86_64-ncurses) and gettext; see the configure log above" 3
}

build_targets() { # $1 = build dir
    cmake --build "$1" --parallel "$JOBS" --target cataclysm cata_test \
        || die "build failed in $1 -- the log tail above is the evidence; for the fork build, docs/phase3-fork.md section 4 lists the expected first-compile fix candidates" 3
}

find_bin() { # $1 = build dir, $2 = exe name; echoes path
    local hit
    hit="$(find "$1" -name "$2" -type f 2>/dev/null | head -1)"
    [ -n "$hit" ] || die "built binary not found under $1: $2" 3
    printf '%s' "$hit"
}

# run_suite <build-dir> <label> [catch2 args...]
# Writes results/<label>.xml (JUnit), <label>.log, <label>-summary.txt,
# <label>-failed.txt. Returns the cata_test exit code (nonzero = failures,
# which the CALLER interprets -- a baseline with failures is still a
# recorded baseline; a fork [voxel] failure is a gate failure).
run_suite() {
    local bdir="$1" label="$2"; shift 2
    local testbin; testbin="$(find_bin "$bdir" cata_test.exe)"
    say "cata_test [$label]: $*"
    ( cd "$SRC" && "$testbin" "$@" --rng-seed "$RNG_SEED" --order lex \
        --user-dir="$bdir/test_user_dir" \
        -r junit -o "$RESULTS/$label.xml" ) 2>&1 | tee "$RESULTS/$label.log"
    local rc=${PIPESTATUS[0]}
    python3 - "$RESULTS/$label.xml" "$RESULTS/$label-summary.txt" "$RESULTS/$label-failed.txt" <<'PYEOF'
import sys, xml.etree.ElementTree as ET
xml_path, summary_path, failed_path = sys.argv[1], sys.argv[2], sys.argv[3]
root = ET.parse(xml_path).getroot()
tests = failures = errors = skipped = 0
failed_names = []
for suite in root.iter("testsuite"):
    tests += int(suite.get("tests", 0)); failures += int(suite.get("failures", 0))
    errors += int(suite.get("errors", 0)); skipped += int(suite.get("skipped", 0))
for case in root.iter("testcase"):
    if case.find("failure") is not None or case.find("error") is not None:
        failed_names.append(case.get("name", "?"))
with open(summary_path, "w") as f:
    f.write("tests=%d failures=%d errors=%d skipped=%d passed=%d\n"
            % (tests, failures, errors, skipped, tests - failures - errors - skipped))
    for n in sorted(set(failed_names)):
        f.write("FAILED: %s\n" % n)
with open(failed_path, "w") as f:
    for n in sorted(set(failed_names)):
        f.write(n + "\n")
print(open(summary_path).read())
PYEOF
    return "$rc"
}

GATE_FAIL=0

# --- stage: baseline (0.1 / 0.2) -------------------------------------------
if [ "$STAGE" = "all" ] || [ "$STAGE" = "baseline" ]; then
    say "BASELINE: pristine curses build (this is the ~1,500 TU compile -- tens of minutes on a 9800X3D, ccache makes reruns fast)"
    [ "$CLEAN" = "1" ] && rm -rf "$BUILD_BASE"
    configure "$BUILD_BASE"
    build_targets "$BUILD_BASE"
    GAMEBIN="$(find_bin "$BUILD_BASE" cataclysm.exe)"
    info "cataclysm: $GAMEBIN"

    say "gate 0.1: the curses binary runs"
    if ( cd "$SRC" && TERM=xterm timeout 60 "$GAMEBIN" --help ) > "$RESULTS/gate-0.1.log" 2>&1; then
        info "0.1 PASS (--help exited 0; log: results/gate-0.1.log)"
        echo "0.1 PASS via --help" > "$RESULTS/gate-0.1.txt"
    elif ( cd "$SRC" && TERM=xterm timeout 300 "$GAMEBIN" --jsonverify ) >> "$RESULTS/gate-0.1.log" 2>&1; then
        info "0.1 PASS via --jsonverify (--help did not exit 0; log: results/gate-0.1.log)"
        echo "0.1 PASS via --jsonverify" > "$RESULTS/gate-0.1.txt"
    else
        info "0.1 FAIL -- binary built but neither --help nor --jsonverify exited 0; see results/gate-0.1.log"
        echo "0.1 FAIL" > "$RESULTS/gate-0.1.txt"
        GATE_FAIL=4
    fi

    if [ "$RUN_FULL" = "1" ]; then
        run_suite "$BUILD_BASE" baseline
        brc=$?
        { echo "commit=$ACTUAL"; echo "date=$(date -Is)"; echo "cata_test exit=$brc";
          cat "$RESULTS/baseline-summary.txt"; } > "$RESULTS/baseline-record.txt"
        say "gate 0.2: baseline recorded (results/baseline-record.txt)"
        info "Upstream failures, if any, are PART of the baseline -- 0.2 passes by being recorded, not by being green."
    else
        info "gate 0.2: full suite skipped (CDDA_RUN_FULL=0) -- no baseline recorded; the fork regression diff will have nothing to compare against."
    fi
fi

# --- stage: fork (Phase 3 in-game gates) ------------------------------------
if [ "$STAGE" = "all" ] || [ "$STAGE" = "fork" ]; then
    say "FORK: stage voxel_ipc + Catch2 cases (build lines only; hooks H1-H7 are NOT applied -- the [voxel] cases link the exporter directly and do not need them)"
    rm -rf "$SRC/src/voxel_ipc"
    rm -f "$SRC/tests/voxel_exporter_tests.cpp"
    cp -r "$REPO/fork/src/voxel_ipc" "$SRC/src/voxel_ipc"
    # FlatBuffers include isolation: the generated header asserts flatbuffers
    # 24.3.25, but upstream vendors 1.12.0 under src/third-party (its own
    # TUs include <flatbuffers/...> and must keep getting 1.12). Copying the
    # 24.3.25 headers INTO src/voxel_ipc/flatbuffers makes the generated
    # header's quoted "flatbuffers/flatbuffers.h" resolve locally, and
    # patch_cmake.py adds src/voxel_ipc as an include dir for the voxel
    # sources ONLY, so no game TU ever sees the 24 headers.
    cp -r "$REPO/tools/stub-server/third_party/flatbuffers" "$SRC/src/voxel_ipc/flatbuffers"
    cp "$REPO/fork/tests/voxel_exporter_tests.cpp" "$SRC/tests/voxel_exporter_tests.cpp"
    python3 "$SCRIPT_DIR/patch_cmake.py" "$SRC/src/CMakeLists.txt" \
        || die "src/CMakeLists.txt patch failed -- the pinned file's GLOB anchor changed; see patch_cmake.py" 3

    [ "$CLEAN" = "1" ] && rm -rf "$BUILD_FORK"
    configure "$BUILD_FORK"
    say "FORK: build (first compile of src/voxel_ipc -- expect fix-up errors here on the first ever run; docs/phase3-fork.md section 4 lists the known candidates)"
    build_targets "$BUILD_FORK"

    run_suite "$BUILD_FORK" voxel "[voxel]"
    vrc=$?
    if [ "$vrc" = "0" ]; then
        info "voxel gates PASS in-game: 3.1 / 3.3 / 3.5 / 3.5b / 3.6 / 3.7 / 3.8"
    else
        info "voxel gates FAIL (cata_test exit $vrc) -- failed cases in results/voxel-failed.txt, log in results/voxel.log"
        GATE_FAIL=4
    fi

    if [ "$RUN_FULL" = "1" ]; then
        if [ -f "$RESULTS/baseline-failed.txt" ]; then
            run_suite "$BUILD_FORK" fork-full
            NEW_FAILED="$(comm -13 "$RESULTS/baseline-failed.txt" "$RESULTS/fork-full-failed.txt")"
            { echo "gate 3.19 -- new failures vs the 0.2 baseline:";
              if [ -z "$NEW_FAILED" ]; then echo "(none)"; else printf '%s\n' "$NEW_FAILED"; fi
              echo; echo "baseline summary:"; cat "$RESULTS/baseline-summary.txt";
              echo "fork summary:";     cat "$RESULTS/fork-full-summary.txt";
            } | tee "$RESULTS/comparison.txt"
            if [ -n "$NEW_FAILED" ]; then
                info "gate 3.19 FAIL -- the fork introduced new test failures (results/comparison.txt)"
                GATE_FAIL=4
            else
                info "gate 3.19 PASS -- no new failures vs baseline"
            fi
        else
            info "gate 3.19 SKIPPED -- no baseline record (run the baseline stage with full suites first)"
        fi
    fi
fi

say "DONE (stage=$STAGE)"
info "results: $RESULTS"
ls -1 "$RESULTS"
exit "$GATE_FAIL"
