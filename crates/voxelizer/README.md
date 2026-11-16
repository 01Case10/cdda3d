# voxelizer

Turns a CDDA tileset's sprites into block texture layers,
`materials.json`, a billboard sprite atlas, and attribution
(Phase 5, plan §8). Phase 5 rework (procedural texture pipeline)
modules: `color` (sRGB/OKLab), `analyze` (palette extraction,
baked-shading and orientation measurement), `classify` (material
classifier v1 — see `docs/procedural-texture-pipeline.md` and
`docs/sprite-anatomy.md`).

**Shareable materials between tilesets.** The procedural pipeline
splits *recipes* (how a material class — grass, plaster, brick —
is generated: noise fields, quilting, de-shading) from *palettes*
(what colors that material has in a given tileset, extracted from
its sprites by `analyze`). Recipes live in code and are
tileset-independent; palettes are data extracted per tileset.
Running the same recipes with another tileset's palettes produces
the same material system in that tileset's colors, so a material
definition is written once and shared across tilesets — only the
palette extraction (and any per-id classifier overrides) is
per-tileset work.
