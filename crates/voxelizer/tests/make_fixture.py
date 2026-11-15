#!/usr/bin/env python3
"""Generate the Phase 5 fixture tileset (deterministic, stdlib only).

Run: python3 make_fixture.py   (writes ./fixture/)
The fixture contains the four pipeline-breaking cases from the test
plan: a multitile entry, an fg-over-bg entry, non-32x32 sprites with
offsets (32x64 tall, 64x64 large), and ids resolvable only via
looks_like / variant / season stages — plus wall/floor/grass, glass,
a fence with alpha holes, and furniture for extrusion.
"""
import json, os, struct, zlib

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "fixture")

def png(path, w, h, pixel_fn):
    raw = b""
    for y in range(h):
        raw += b"\x00" + b"".join(bytes(pixel_fn(x, y)) for x in range(w))
    def chunk(tag, data):
        c = struct.pack(">I", len(data)) + tag + data
        return c + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
    data = (b"\x89PNG\r\n\x1a\n"
            + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9))
            + chunk(b"IEND", b""))
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as f:
        f.write(data)

def solid(r, g, b, a=255):
    return lambda x, y: (r, g, b, a)

def write_json(path, obj):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        json.dump(obj, f, indent=1, sort_keys=True)
        f.write("\n")

N = os.path.join(ROOT, "pngs_normal_32x32")
T = os.path.join(ROOT, "pngs_tall_32x64")
L = os.path.join(ROOT, "pngs_large_64x64")

# --- sprites ---
# grass: fg = sparse green speckles (mostly transparent), bg = dirt
png(os.path.join(N, "grass_bg.png"), 32, 32, solid(101, 67, 33))
png(os.path.join(N, "grass_fg.png"), 32, 32,
    lambda x, y: (34, 139, 34, 255) if (x * 7 + y * 13) % 3 == 0 else (0, 0, 0, 0))
png(os.path.join(N, "wood_floor.png"), 32, 32,
    lambda x, y: (160, 110, 60, 255) if y % 8 else (120, 80, 40, 255))
# wall multitile subtiles: distinct solid colors per subtile/rotation
wall_colors = {
    "t_wall_unconnected": (200, 200, 200), "t_wall_center": (190, 190, 190),
    "t_wall_corner_nw": (1, 2, 3), "t_wall_corner_ne": (4, 5, 6),
    "t_wall_corner_se": (7, 8, 9), "t_wall_corner_sw": (10, 11, 12),
    "t_wall_t_connection_n": (13, 14, 15), "t_wall_t_connection_e": (16, 17, 18),
    "t_wall_t_connection_s": (19, 20, 21), "t_wall_t_connection_w": (22, 23, 24),
    "t_wall_edge_ns": (25, 26, 27), "t_wall_edge_ew": (28, 29, 30),
    "t_wall_end_piece_n": (31, 32, 33), "t_wall_end_piece_e": (34, 35, 36),
    "t_wall_end_piece_s": (37, 38, 39), "t_wall_end_piece_w": (40, 41, 42),
}
for name, (r, g, b) in wall_colors.items():
    png(os.path.join(N, name + ".png"), 32, 32, solid(r, g, b))
png(os.path.join(N, "glass.png"), 32, 32, solid(170, 210, 230, 120))
# fence: vertical slats with alpha holes
png(os.path.join(N, "fence.png"), 32, 32,
    lambda x, y: (120, 90, 50, 255) if x % 8 < 4 else (0, 0, 0, 0))
# furniture: table = filled rect inset by 4; bookcase = full block w/ shelves
png(os.path.join(N, "table.png"), 32, 32,
    lambda x, y: (140, 100, 60, 255) if 4 <= x < 28 and 4 <= y < 28 else (0, 0, 0, 0))
png(os.path.join(N, "bookcase.png"), 32, 32,
    lambda x, y: (90, 60, 30, 255) if y % 10 else (50, 30, 15, 255))
png(os.path.join(N, "dirt.png"), 32, 32, solid(110, 80, 50))
png(os.path.join(N, "rock_red.png"), 32, 32, solid(180, 60, 50))
png(os.path.join(N, "seasonal_spring.png"), 32, 32, solid(80, 200, 90))
png(os.path.join(N, "filler.png"), 32, 32, solid(60, 60, 60))
# tall sprite 32x64: top half red, bottom half blue
png(os.path.join(T, "tall_thing.png"), 32, 64,
    lambda x, y: (220, 40, 40, 255) if y < 32 else (40, 40, 220, 255))
# large sprite 64x64: solid purple
png(os.path.join(L, "big_thing.png"), 64, 64, solid(150, 60, 180))

# --- tile entries ---
write_json(os.path.join(N, "t_grass.json"),
           {"id": "t_grass", "fg": "grass_fg", "bg": "grass_bg"})
write_json(os.path.join(N, "t_floor_wood.json"),
           {"id": "t_floor_wood", "fg": "wood_floor"})
write_json(os.path.join(N, "t_wall.json"), {
    "id": "t_wall", "multitile": True, "fg": "t_wall_unconnected",
    "additional_tiles": [
        {"id": "center", "fg": "t_wall_center"},
        {"id": "corner", "fg": ["t_wall_corner_nw", "t_wall_corner_ne",
                                "t_wall_corner_se", "t_wall_corner_sw"]},
        {"id": "t_connection", "fg": ["t_wall_t_connection_n", "t_wall_t_connection_e",
                                      "t_wall_t_connection_s", "t_wall_t_connection_w"]},
        {"id": "edge", "fg": ["t_wall_edge_ns", "t_wall_edge_ew"]},
        {"id": "end_piece", "fg": ["t_wall_end_piece_n", "t_wall_end_piece_e",
                                   "t_wall_end_piece_s", "t_wall_end_piece_w"]},
        {"id": "unconnected", "fg": "t_wall_unconnected"},
    ]})
write_json(os.path.join(N, "t_glass_window.json"),
           {"id": "t_glass_window", "fg": "glass"})
write_json(os.path.join(N, "t_fence.json"),
           {"id": "t_fence", "fg": "fence"})
write_json(os.path.join(N, "f_table.json"),
           {"id": "f_table", "fg": "table"})
write_json(os.path.join(N, "f_bookcase.json"),
           {"id": "f_bookcase", "fg": "bookcase"})
write_json(os.path.join(N, "t_dirt.json"),
           {"id": "t_dirt", "fg": "dirt"})
# variant-only: t_rock itself has NO entry; t_rock_var_red does
write_json(os.path.join(N, "t_rock_var.json"),
           {"id": "t_rock_var_red", "fg": "rock_red"})
# season-only: t_seasonal resolves via _season_spring at pinned season
write_json(os.path.join(N, "t_seasonal.json"),
           {"id": "t_seasonal_season_spring", "fg": "seasonal_spring"})
write_json(os.path.join(N, "filler.json"),
           {"id": "filler", "fg": "filler"})
write_json(os.path.join(T, "mon_tall.json"),
           {"id": "mon_tall", "fg": "tall_thing"})
write_json(os.path.join(L, "big_thing.json"),
           {"id": "big_thing", "fg": "big_thing"})

write_json(os.path.join(ROOT, "tile_info.json"), [
    {"pixelscale": 1, "width": 32, "height": 32},
    {"normal.png": {}},
    {"tall.png": {"sprite_width": 32, "sprite_height": 64, "sprite_offset_y": -32}},
    {"large.png": {"sprite_width": 64, "sprite_height": 64,
                   "sprite_offset_x": -16, "sprite_offset_y": -32,
                   "sprites_across": 8}},
    {"filler.png": {"sprite_width": 32, "sprite_height": 32, "filler": True,
                    "source": "fixture://filler-source"}},
])
with open(os.path.join(ROOT, "tileset.txt"), "w") as f:
    f.write("#Fixture\n\nNAME: FixtureTileset\nVIEW: Fixture\nJSON: tile_config.json\nTILESET: tiles.png\n")
with open(os.path.join(ROOT, "LICENSE.txt"), "w") as f:
    f.write("Fixture tileset is licensed under the Creative Commons Attribution-ShareAlike 3.0 Unported License. To view a copy of this license, visit http://creativecommons.org/licenses/by-sa/3.0/\n\n"
            "Fixture Font is licensed under the SIL Open Font License.\n\n"
            "FixtureLib is licensed under the MIT license.\n")
print("fixture written to", ROOT)
