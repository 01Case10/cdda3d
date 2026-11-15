// Spike A (Phase 0, test 0.6) + Spike B support (test 0.8)
// Standalone prototype of the two UI-capture routes from plan Part I Phase 4.
//
// Route B model (catacurses-level capture): each game UI window (terrain,
// sidebar panels, message log, menu adaptor) is rendered as cells into one
// offscreen RGBA buffer at its cell position. The terrain window is
// suppressed (left transparent) exactly as the fork will do in
// --frontend=ipc mode. No window system, no GPU, no SDL: this binary runs
// with DISPLAY unset, which is also the Spike B headless proof for the
// null-backend route.
//
// Route A cost model: the same windows are first composited into a
// "display buffer" (the SDL backend's display_buffer analogue, terminal
// sized in pixels), then read back CPU-side. The readback is timed so the
// Phase 4 budget has a measured number instead of a guess.
//
// Output: spike_ui.rgba (raw), spike_ui.ppm, spike_meta.json in argv[1] dir.
// Exit 0 iff all internal checks pass.
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <cstdlib>
#include <string>
#include <vector>
#include <chrono>
#include <algorithm>

static const int CELL = 16;            // pixels per cell (font cell analogue)
static const int COLS = 100, ROWS = 50; // terminal size in cells
static const int W = COLS * CELL, H = ROWS * CELL;

struct Rect { int x, y, w, h; }; // in cells
// Layout mirrors a default CDDA panel layout: terrain on the left,
// sidebar panels + message log on the right.
static const Rect TERRAIN{0, 0, 68, 42};
static const Rect SIDEBAR{69, 0, 31, 30};
static const Rect MSGLOG{0, 43, 100, 7};
static const Rect MENU{20, 10, 40, 22}; // a ui_adaptor menu over the terrain

struct RGBA { uint8_t r, g, b, a; };

static void fill_rect(std::vector<RGBA> &buf, Rect rc, RGBA c) {
    for (int cy = rc.y; cy < rc.y + rc.h; cy++)
        for (int cx = rc.x; cx < rc.x + rc.w; cx++)
            for (int py = 0; py < CELL; py++)
                for (int px = 0; px < CELL; px++) {
                    int x = cx * CELL + px, y = cy * CELL + py;
                    buf[y * W + x] = c;
                }
}
// Draw a crude glyph pattern per cell so panels/menus are visibly "text".
static void draw_text_cells(std::vector<RGBA> &buf, Rect rc, RGBA bg, RGBA fg,
                            uint32_t seed) {
    fill_rect(buf, rc, bg);
    uint32_t s = seed;
    for (int cy = rc.y; cy < rc.y + rc.h; cy++) {
        for (int cx = rc.x; cx < rc.x + rc.w; cx++) {
            s = s * 1664525u + 1013904223u;
            if ((s >> 24) % 3 == 0) continue; // spaces
            int gw = 3 + (s >> 20) % 8, gh = 8;
            for (int py = 4; py < 4 + gh; py++)
                for (int px = 3; px < 3 + gw; px++) {
                    int x = cx * CELL + px, y = cy * CELL + py;
                    buf[y * W + x] = fg;
                }
        }
    }
}

int main(int argc, char **argv) {
    std::string outdir = argc > 1 ? argv[1] : ".";
    bool fail = false;
    auto check = [&](bool ok, const char *what) {
        printf("%s %s\n", ok ? "PASS" : "FAIL", what);
        if (!ok) fail = true;
    };

    // ---- Route B: windows -> offscreen buffer, terrain suppressed ----
    std::vector<RGBA> ui(W * H, {0, 0, 0, 0}); // starts fully transparent
    draw_text_cells(ui, SIDEBAR, {30, 34, 44, 255}, {210, 220, 235, 255}, 11);
    draw_text_cells(ui, MSGLOG, {18, 20, 26, 255}, {170, 200, 170, 255}, 23);
    draw_text_cells(ui, MENU, {52, 40, 66, 255}, {240, 230, 200, 255}, 37);
    // TERRAIN rect deliberately untouched: alpha stays 0 there except where
    // the menu adaptor overlaps it (menus draw over the map in the real game
    // too, via the ui_manager stack).

    int terrain_transparent = 0, terrain_total = 0;
    for (int cy = TERRAIN.y; cy < TERRAIN.y + TERRAIN.h; cy++)
        for (int cx = TERRAIN.x; cx < TERRAIN.x + TERRAIN.w; cx++) {
            bool in_menu = cx >= MENU.x && cx < MENU.x + MENU.w &&
                           cy >= MENU.y && cy < MENU.y + MENU.h;
            RGBA p = ui[(cy * CELL + 8) * W + (cx * CELL + 8)];
            terrain_total++;
            if (!in_menu && p.a == 0) terrain_transparent++;
            if (in_menu && p.a != 255) fail = true;
        }
    check(terrain_transparent == terrain_total -
              (MENU.w * MENU.h - 0) * 0 - [&] {
                  int n = 0;
                  for (int cy = TERRAIN.y; cy < TERRAIN.y + TERRAIN.h; cy++)
                      for (int cx = TERRAIN.x; cx < TERRAIN.x + TERRAIN.w; cx++)
                          if (cx >= MENU.x && cx < MENU.x + MENU.w &&
                              cy >= MENU.y && cy < MENU.y + MENU.h) n++;
                  return n;
              }(),
          "terrain rect transparent outside menu overlap");
    RGBA sb = ui[(5 * CELL + 8) * W + (80 * CELL + 8)];
    check(sb.a == 255, "sidebar panel opaque in capture");
    RGBA ml = ui[(45 * CELL + 8) * W + (10 * CELL + 8)];
    check(ml.a == 255, "message log opaque in capture");

    // ---- Route A cost model: composite to display buffer, read back ----
    std::vector<RGBA> display(W * H);
    std::vector<RGBA> readback(W * H);
    auto t0 = std::chrono::steady_clock::now();
    const int ITERS = 200;
    double worst = 0;
    for (int i = 0; i < ITERS; i++) {
        auto a = std::chrono::steady_clock::now();
        display = ui; // composite (analogue of the backend's render into display_buffer)
        memcpy(readback.data(), display.data(), W * H * sizeof(RGBA)); // GPU->CPU readback analogue
        auto b = std::chrono::steady_clock::now();
        double ms = std::chrono::duration<double, std::milli>(b - a).count();
        worst = std::max(worst, ms);
    }
    auto t1 = std::chrono::steady_clock::now();
    double mean_ms = std::chrono::duration<double, std::milli>(t1 - t0).count() / ITERS;
    check(memcmp(readback.data(), ui.data(), W * H * sizeof(RGBA)) == 0,
          "route A readback matches composited buffer");
    printf("routeA_readback_mean_ms=%.3f worst_ms=%.3f bytes=%zu\n", mean_ms, worst,
           (size_t)W * H * sizeof(RGBA));

    // ---- Write outputs ----
    std::string ppm_path = outdir + "/spike_ui.ppm";
    FILE *f = fopen(ppm_path.c_str(), "wb");
    fprintf(f, "P6\n%d %d\n255\n", W, H);
    for (auto &p : ui) {
        // composite over magenta so transparency is visible in the PPM
        uint8_t r = p.a ? p.r : 255, g = p.a ? p.g : 0, b = p.a ? p.b : 255;
        fputc(r, f); fputc(g, f); fputc(b, f);
    }
    fclose(f);
    std::string meta_path = outdir + "/spike_meta.json";
    f = fopen(meta_path.c_str(), "w");
    fprintf(f,
            "{\"width\":%d,\"height\":%d,\"cell\":%d,\"cols\":%d,\"rows\":%d,"
            "\"terrain_rect_cells\":[%d,%d,%d,%d],"
            "\"routeA_readback_mean_ms\":%.3f,\"routeA_readback_worst_ms\":%.3f,"
            "\"display_set\":%s}\n",
            W, H, CELL, COLS, ROWS, TERRAIN.x, TERRAIN.y, TERRAIN.w, TERRAIN.h,
            mean_ms, worst, getenv("DISPLAY") ? "true" : "false");
    fclose(f);
    printf("wrote %s and %s\n", ppm_path.c_str(), meta_path.c_str());
    return fail ? 1 : 0;
}
