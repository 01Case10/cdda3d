#!/usr/bin/env python3
"""Add the voxel_ipc module to the fork's src/CMakeLists.txt.

The pinned upstream file (CDDA master @ 6a3e59e0) collects game sources
with a NON-recursive glob:

    file(GLOB CATACLYSM_DDA_SOURCES
        ${CMAKE_CURRENT_SOURCE_DIR}/*.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/cata_allocator_c.c)

so src/voxel_ipc/*.cpp would silently not build. This patch appends the
module's sources right after that glob, and gives ONLY those sources
src/voxel_ipc as an extra include directory -- that is what lets their
quoted "flatbuffers/flatbuffers.h" resolve to the flatbuffers 24.3.25
headers staged inside src/voxel_ipc/flatbuffers, while every game TU
keeps resolving <flatbuffers/...> to upstream's vendored 1.12.0 under
src/third-party.

Idempotent: if the marker is already present, the file is left alone.
Fails loudly (exit 1) if the anchor glob is not found verbatim, so a
future upstream rebase surfaces here instead of silently dropping the
module from the build.

Usage: patch_cmake.py <path-to-src/CMakeLists.txt>
"""
import sys

MARKER = "# cdda-voxel: voxel_ipc module (tools/pc-build)"

ANCHOR = """file(GLOB CATACLYSM_DDA_SOURCES
    ${CMAKE_CURRENT_SOURCE_DIR}/*.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/cata_allocator_c.c)
"""

INSERT = ANCHOR + """
# cdda-voxel: voxel_ipc module (tools/pc-build) -- non-recursive glob
# above misses src/voxel_ipc/*.cpp; append it explicitly. The include
# dir is per-source so game TUs never see the flatbuffers 24.3.25
# headers staged under src/voxel_ipc/flatbuffers (upstream vendors
# 1.12.0 for its own TUs; the generated IPC header asserts 24.3.25).
file(GLOB VOXEL_IPC_SOURCES ${CMAKE_CURRENT_SOURCE_DIR}/voxel_ipc/*.cpp)
list(APPEND CATACLYSM_DDA_SOURCES ${VOXEL_IPC_SOURCES})
foreach(_voxel_src IN LISTS VOXEL_IPC_SOURCES)
    set_source_files_properties(${_voxel_src} PROPERTIES
        INCLUDE_DIRECTORIES "${CMAKE_CURRENT_SOURCE_DIR}/voxel_ipc")
endforeach()
"""


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: patch_cmake.py <path-to-src/CMakeLists.txt>", file=sys.stderr)
        return 2
    path = sys.argv[1]
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()
    if MARKER in text:
        print("patch_cmake: already applied, leaving %s unchanged" % path)
        return 0
    if ANCHOR not in text:
        print("patch_cmake: ANCHOR NOT FOUND in %s" % path, file=sys.stderr)
        print("expected verbatim:\n" + ANCHOR, file=sys.stderr)
        return 1
    text = text.replace(ANCHOR, INSERT, 1)
    with open(path, "w", encoding="utf-8", newline="") as f:
        f.write(text)
    print("patch_cmake: voxel_ipc added to %s" % path)
    return 0


if __name__ == "__main__":
    sys.exit(main())
