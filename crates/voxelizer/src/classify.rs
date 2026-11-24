//! Material classifier v1 (Phase 5 rework, Stage B; draft §4.2).
//!
//! Every tile id gets a [`MaterialClass`] plus the **rule that fired**,
//! so coverage audits can see *why* an id landed where it did and
//! which rules are doing the work. Rules run in a fixed order, first
//! match wins:
//!
//! 1. **Override** — `overrides.toml` entry with a `material = "..."`
//!    key (the voxelizer's existing override mechanism). The name must
//!    match a [`MaterialClass`] name (case-insensitive); an unmatched
//!    name is reported as `override-invalid` and does *not* silently
//!    fall through, so typos surface in audits.
//! 2. **Kind** — the fork TileDef kind, when the caller has it. The
//!    tileset loader's `TileEntry` carries no kind, so in v1 this
//!    stage only gates the non-block kinds (monsters/items are
//!    billboards, never block materials) and otherwise defers to the
//!    pattern stage; the id prefix (`t_`/`f_`) is the in-data proxy.
//! 3. **Id patterns** — ordered substring/prefix table seeded from the
//!    enumerated UltiCa id universe (see
//!    `examples/classify_coverage.rs` output): walls first (with a
//!    material sub-scan), then doors/gates/fences, vegetation, ground
//!    families, floors.
//! 4. **Name keywords** — a small generic keyword set (wood, metal,
//!    glass, stone, ...) for ids the pattern table missed, mostly
//!    furniture and vehicle parts.
//! 5. **Color fallback** — nearest class centroid in OKLab to the
//!    sprite family's palette centroid. Always answers when a palette
//!    exists; the distance is recorded in the result detail.
//! 6. **Unknown** — no rule fired and no palette to fall back on.
//!
//! The classifier is pure: same (id, kind, palette, overrides) in,
//! same classification out.

use crate::analyze::PaletteColor;
use crate::color::Oklab;
use crate::overrides::Overrides;

/// The material classes Stage C+ recipes know how to generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterialClass {
    Grass,
    Dirt,
    Sand,
    Road,
    Water,
    WoodFloor,
    Wood,
    Plaster,
    Concrete,
    Brick,
    Stone,
    Metal,
    Glass,
    Foliage,
    Carpet,
    Linoleum,
    Snow,
    Ice,
    Roof,
    Unknown,
}

impl MaterialClass {
    pub fn name(self) -> &'static str {
        match self {
            MaterialClass::Grass => "grass",
            MaterialClass::Dirt => "dirt",
            MaterialClass::Sand => "sand",
            MaterialClass::Road => "road",
            MaterialClass::Water => "water",
            MaterialClass::WoodFloor => "wood_floor",
            MaterialClass::Wood => "wood",
            MaterialClass::Plaster => "plaster",
            MaterialClass::Concrete => "concrete",
            MaterialClass::Brick => "brick",
            MaterialClass::Stone => "stone",
            MaterialClass::Metal => "metal",
            MaterialClass::Glass => "glass",
            MaterialClass::Foliage => "foliage",
            MaterialClass::Carpet => "carpet",
            MaterialClass::Linoleum => "linoleum",
            MaterialClass::Snow => "snow",
            MaterialClass::Ice => "ice",
            MaterialClass::Roof => "roof",
            MaterialClass::Unknown => "unknown",
        }
    }

    pub fn from_name(s: &str) -> Option<MaterialClass> {
        let s = s.to_ascii_lowercase();
        Some(match s.as_str() {
            "grass" => MaterialClass::Grass,
            "dirt" => MaterialClass::Dirt,
            "sand" => MaterialClass::Sand,
            "road" => MaterialClass::Road,
            "water" => MaterialClass::Water,
            "wood_floor" | "woodfloor" => MaterialClass::WoodFloor,
            "wood" => MaterialClass::Wood,
            "plaster" => MaterialClass::Plaster,
            "concrete" => MaterialClass::Concrete,
            "brick" => MaterialClass::Brick,
            "stone" => MaterialClass::Stone,
            "metal" => MaterialClass::Metal,
            "glass" => MaterialClass::Glass,
            "foliage" => MaterialClass::Foliage,
            "carpet" => MaterialClass::Carpet,
            "linoleum" => MaterialClass::Linoleum,
            "snow" => MaterialClass::Snow,
            "ice" => MaterialClass::Ice,
            "roof" => MaterialClass::Roof,
            "unknown" => MaterialClass::Unknown,
            _ => return None,
        })
    }

    /// Representative sRGB anchor color for the class, used for the
    /// color fallback centroids and preview swatch bands. Anchors are
    /// hand-picked midpoints of each family's UltiCa palettes.
    pub fn anchor_srgb(self) -> [u8; 3] {
        match self {
            MaterialClass::Grass => [94, 128, 54],
            MaterialClass::Dirt => [110, 82, 54],
            MaterialClass::Sand => [196, 178, 124],
            MaterialClass::Road => [74, 74, 78],
            MaterialClass::Water => [52, 96, 150],
            MaterialClass::WoodFloor => [168, 128, 78],
            MaterialClass::Wood => [110, 78, 46],
            MaterialClass::Plaster => [206, 198, 182],
            MaterialClass::Concrete => [158, 158, 152],
            MaterialClass::Brick => [152, 84, 64],
            MaterialClass::Stone => [128, 126, 118],
            MaterialClass::Metal => [140, 148, 156],
            MaterialClass::Glass => [168, 200, 210],
            MaterialClass::Foliage => [62, 102, 44],
            MaterialClass::Carpet => [122, 60, 70],
            MaterialClass::Linoleum => [178, 172, 150],
            MaterialClass::Snow => [238, 240, 244],
            MaterialClass::Ice => [186, 216, 228],
            MaterialClass::Roof => [120, 100, 88],
            MaterialClass::Unknown => [255, 0, 255],
        }
    }

    pub fn centroid(self) -> Oklab {
        Oklab::from_srgb_u8(self.anchor_srgb())
    }

    /// All classes the color fallback may return (excludes Unknown).
    pub fn fallback_classes() -> &'static [MaterialClass] {
        &[
            MaterialClass::Grass,
            MaterialClass::Dirt,
            MaterialClass::Sand,
            MaterialClass::Road,
            MaterialClass::Water,
            MaterialClass::WoodFloor,
            MaterialClass::Wood,
            MaterialClass::Plaster,
            MaterialClass::Concrete,
            MaterialClass::Brick,
            MaterialClass::Stone,
            MaterialClass::Metal,
            MaterialClass::Glass,
            MaterialClass::Foliage,
            MaterialClass::Carpet,
            MaterialClass::Linoleum,
            MaterialClass::Snow,
            MaterialClass::Ice,
            MaterialClass::Roof,
        ]
    }
}

/// Coarse tile kind. The fork's TileDef carries the authoritative
/// kind; the tileset data does not, so v1 callers pass `None` or
/// derive it from the id prefix with [`TileKind::from_id_prefix`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileKind {
    Terrain,
    Furniture,
    Monster,
    Field,
    VehiclePart,
    Other,
}

impl TileKind {
    pub fn from_id_prefix(id: &str) -> TileKind {
        if id.starts_with("t_") {
            TileKind::Terrain
        } else if id.starts_with("f_") {
            TileKind::Furniture
        } else if id.starts_with("mon_") {
            TileKind::Monster
        } else if id.starts_with("fd_") {
            TileKind::Field
        } else if id.starts_with("vp_") {
            TileKind::VehiclePart
        } else {
            TileKind::Other
        }
    }
}

/// Which rule produced a classification (audit trail).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rule {
    /// overrides.toml `material = "..."` matched a class name.
    Override,
    /// overrides.toml named a material that is not a class name.
    OverrideInvalid,
    /// TileDef kind says this id is never a block material.
    KindNotBlock,
    /// Id-pattern table; payload is the pattern's audit name.
    Pattern(&'static str),
    /// Keyword table; payload is the matched keyword.
    Keyword(&'static str),
    /// Nearest OKLab centroid; payload is the distance (x1000, so the
    /// value stays comparable/printable without float formatting
    /// surprises in audit files).
    ColorFallback(u32),
    /// Nothing fired.
    Unknown,
}

impl Rule {
    pub fn name(&self) -> String {
        match self {
            Rule::Override => "override".to_string(),
            Rule::OverrideInvalid => "override-invalid".to_string(),
            Rule::KindNotBlock => "kind-not-block".to_string(),
            Rule::Pattern(p) => format!("pattern:{p}"),
            Rule::Keyword(k) => format!("keyword:{k}"),
            Rule::ColorFallback(d) => format!("color-fallback(d={:.3})", *d as f32 / 1000.0),
            Rule::Unknown => "unknown".to_string(),
        }
    }

    /// Coarse bucket for coverage statistics.
    pub fn bucket(&self) -> &'static str {
        match self {
            Rule::Override | Rule::OverrideInvalid => "override",
            Rule::KindNotBlock => "kind",
            Rule::Pattern(_) => "pattern",
            Rule::Keyword(_) => "keyword",
            Rule::ColorFallback(_) => "color-fallback",
            Rule::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Classification {
    pub class: MaterialClass,
    pub rule: Rule,
}

/// Material sub-scan shared by wall/fence/door patterns: look for a
/// material keyword inside the id. Returns None when the id names no
/// recognizable material (caller decides the default).
fn material_in(id: &str) -> Option<MaterialClass> {
    let has = |s: &str| id.contains(s);
    if has("glass") {
        Some(MaterialClass::Glass)
    } else if has("brick") || has("adobe") {
        Some(MaterialClass::Brick)
    } else if has("strconc") || has("sconc") || has("thconc") || has("concrete") || has("conc_wall") {
        Some(MaterialClass::Concrete)
    } else if has("metal") || has("chain") || has("iron") || has("steel") {
        Some(MaterialClass::Metal)
    } else if has("wood") || has("log") || has("palisade") || has("bamboo") {
        Some(MaterialClass::Wood)
    } else if has("rock") || has("stone") {
        Some(MaterialClass::Stone)
    } else if has("ice") {
        Some(MaterialClass::Ice)
    } else if has("paper") || has("plaster") {
        Some(MaterialClass::Plaster)
    } else {
        None
    }
}

/// The pattern stage. `id` must already be lowercased.
fn pattern_stage(id: &str) -> Option<(MaterialClass, &'static str)> {
    use MaterialClass::*;
    let has = |s: &str| id.contains(s);
    let pat = |c: MaterialClass, n: &'static str| Some((c, n));

    // Air is not a material; classifying it Unknown-with-a-name keeps
    // it out of the "unexplained" bucket in audits.
    if id == "t_open_air" || id.starts_with("t_open_air_") {
        return pat(Unknown, "open-air");
    }
    // Keycard readers / control panels read as metal boxes.
    if id.starts_with("t_card_") {
        return pat(Metal, "card-reader");
    }
    // Glass before walls/doors: glass walls, glass doors, reinforced
    // glass all sample as glass.
    if has("glass") {
        return pat(Glass, "glass");
    }
    // Walls: material sub-scan, plaster default (t_wall, t_wall_*,
    // *_wall in UltiCa are overwhelmingly painted drywall/plaster).
    if has("wall") {
        return match material_in(id) {
            Some(m) => pat(m, "wall-material"),
            None => pat(Plaster, "wall-default-plaster"),
        };
    }
    // Windows are glass panes in a frame; the pane dominates.
    if has("window") {
        return pat(Glass, "window");
    }
    // Curtains before doors (t_door_grass_curtain is a grass curtain).
    if has("curtain") {
        return if has("grass") {
            pat(Foliage, "grass-curtain")
        } else {
            pat(Carpet, "curtain-fabric")
        };
    }
    // Doors and gates: metal if named/chained, wood otherwise.
    if has("door") || has("gate") {
        return match material_in(id) {
            Some(m) => pat(m, "door-material"),
            None => pat(Wood, "door-default-wood"),
        };
    }
    // Fences, railings, palisades.
    if has("fence") || has("railing") || has("guardrail") || has("palisade") {
        return match material_in(id) {
            Some(m) => pat(m, "fence-material"),
            None => pat(Wood, "fence-default-wood"),
        };
    }
    // Railroad: ties are wood, track is steel, rubble is stone.
    if has("railroad") {
        return if has("tie") {
            pat(Wood, "railroad-tie")
        } else if has("rubble") {
            pat(Stone, "railroad-rubble")
        } else {
            pat(Metal, "railroad-track")
        };
    }
    // Vegetation.
    if has("grass") {
        return pat(Grass, "grass");
    }
    if has("tree") || has("shrub") || has("fern") || has("underbrush") || has("fungus") || has("moss") {
        return pat(Foliage, "foliage");
    }
    // Ground families.
    if has("dirt") || has("clay") || has("mud") {
        return pat(Dirt, "dirt");
    }
    if has("sand") {
        return pat(Sand, "sand");
    }
    if has("snow") {
        return pat(Snow, "snow");
    }
    if has("ice") {
        return pat(Ice, "ice");
    }
    if id.starts_with("t_") && (has("water") || has("puddle")) {
        return pat(Water, "water");
    }
    if has("lava") {
        return pat(Stone, "lava-cooled-rock");
    }
    // Floors and floor coverings. Carpet/linoleum first: a carpet's
    // visible surface is the carpet, whatever it lies on.
    if has("carpet") {
        return pat(Carpet, "carpet");
    }
    if has("linoleum") {
        return pat(Linoleum, "linoleum");
    }
    if has("sidewalk") || has("concrete") || has("_conc") || has("strconc") {
        return pat(Concrete, "concrete");
    }
    if has("pavement") || has("asphalt") {
        return pat(Road, "pavement");
    }
    if has("floor") {
        return if has("metal") {
            pat(Metal, "floor-metal")
        } else if has("resin") {
            pat(Plaster, "floor-resin")
        } else {
            // UltiCa t_floor* are painted wood plank floors.
            pat(WoodFloor, "floor-wood")
        };
    }
    if has("roof") {
        return pat(Roof, "roof");
    }
    if has("brick") {
        return pat(Brick, "brick");
    }
    if has("rock") || has("stone") || has("boulder") {
        return pat(Stone, "stone");
    }
    if has("metal_floor") || has("deck_plate") {
        return pat(Metal, "metal-surface");
    }
    None
}

/// The keyword stage: generic material words anywhere in the id.
fn keyword_stage(id: &str) -> Option<(MaterialClass, &'static str)> {
    use MaterialClass::*;
    // Deliberately no "plastic": v1 has no plastic class, and mapping
    // it to metal would be a silent lie in audits — plastic ids fall
    // through to the color fallback instead.
    const TABLE: &[(&str, MaterialClass)] = &[
        ("wooden", Wood),
        ("wood", Wood),
        ("metal", Metal),
        ("steel", Metal),
        ("iron", Metal),
        ("glass", Glass),
        ("stone", Stone),
        ("rock", Stone),
        ("boulder", Stone),
        ("brick", Brick),
        ("concrete", Concrete),
    ];
    for (kw, class) in TABLE {
        if id.contains(kw) {
            return Some((*class, kw));
        }
    }
    None
}

/// Classify one id. `palette` is the sprite family's extracted
/// palette (may be empty/None when no sprite exists); `kind` is the
/// fork TileDef kind when available.
pub fn classify(
    id: &str,
    kind: Option<TileKind>,
    palette: Option<&[PaletteColor]>,
    overrides: &Overrides,
) -> Classification {
    // 1. Override.
    if let Some(entry) = overrides.entries.get(id) {
        if let Some(mat) = &entry.material {
            return match MaterialClass::from_name(mat) {
                Some(class) => Classification {
                    class,
                    rule: Rule::Override,
                },
                None => Classification {
                    class: MaterialClass::Unknown,
                    rule: Rule::OverrideInvalid,
                },
            };
        }
    }
    // 2. Kind gate: monsters and items are billboards in this
    //    pipeline, never block materials.
    if let Some(k) = kind {
        if matches!(k, TileKind::Monster) {
            return Classification {
                class: MaterialClass::Unknown,
                rule: Rule::KindNotBlock,
            };
        }
    }
    let lower = id.to_ascii_lowercase();
    // 3. Id patterns.
    if let Some((class, name)) = pattern_stage(&lower) {
        return Classification {
            class,
            rule: Rule::Pattern(name),
        };
    }
    // 4. Keywords.
    if let Some((class, kw)) = keyword_stage(&lower) {
        return Classification {
            class,
            rule: Rule::Keyword(kw),
        };
    }
    // 5. Color fallback: nearest class centroid to the palette
    //    centroid in OKLab.
    if let Some(palette) = palette {
        if let Some(centroid) = crate::analyze::palette_centroid(palette) {
            let mut best = MaterialClass::Unknown;
            let mut best_d = f32::INFINITY;
            for class in MaterialClass::fallback_classes() {
                let d = centroid.distance(class.centroid());
                if d < best_d {
                    best_d = d;
                    best = *class;
                }
            }
            return Classification {
                class: best,
                rule: Rule::ColorFallback((best_d * 1000.0).round() as u32),
            };
        }
    }
    // 6. Unknown.
    Classification {
        class: MaterialClass::Unknown,
        rule: Rule::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyze::PaletteColor;
    use crate::color::Oklab;
    use crate::overrides::parse_overrides;

    fn no_ov() -> Overrides {
        Overrides::default()
    }

    fn class_of(id: &str) -> MaterialClass {
        classify(id, None, None, &no_ov()).class
    }

    #[test]
    fn pattern_table_core_families() {
        assert_eq!(class_of("t_grass"), MaterialClass::Grass);
        assert_eq!(class_of("t_grass_long"), MaterialClass::Grass);
        assert_eq!(class_of("t_dirt"), MaterialClass::Dirt);
        assert_eq!(class_of("t_dirtfloor"), MaterialClass::Dirt);
        assert_eq!(class_of("t_floor"), MaterialClass::WoodFloor);
        assert_eq!(class_of("t_floor_blue"), MaterialClass::WoodFloor);
        assert_eq!(class_of("t_metal_floor"), MaterialClass::Metal);
        assert_eq!(class_of("t_wall"), MaterialClass::Plaster);
        assert_eq!(class_of("t_concrete_wall"), MaterialClass::Concrete);
        assert_eq!(class_of("t_adobe_brick_wall"), MaterialClass::Brick);
        assert_eq!(class_of("t_wall_wood"), MaterialClass::Wood);
        assert_eq!(class_of("t_water_shallow"), MaterialClass::Water);
        assert_eq!(class_of("t_swater"), MaterialClass::Water); // "swater" contains "water"
        assert_eq!(class_of("t_tree_pine"), MaterialClass::Foliage);
        assert_eq!(class_of("t_door_c"), MaterialClass::Wood);
        assert_eq!(class_of("t_door_metal_c"), MaterialClass::Metal);
        assert_eq!(class_of("t_chaingate_c"), MaterialClass::Metal);
        assert_eq!(class_of("t_fence"), MaterialClass::Wood);
        assert_eq!(class_of("t_chainfence"), MaterialClass::Metal);
        assert_eq!(class_of("t_sidewalk"), MaterialClass::Concrete);
        assert_eq!(class_of("t_pavement"), MaterialClass::Road);
        assert_eq!(class_of("t_carpet_red"), MaterialClass::Carpet);
        assert_eq!(class_of("t_carpet_base_concrete"), MaterialClass::Carpet);
        assert_eq!(class_of("t_linoleum_gray"), MaterialClass::Linoleum);
        assert_eq!(class_of("t_rock"), MaterialClass::Stone);
        assert_eq!(class_of("t_reinforced_glass"), MaterialClass::Glass);
        assert_eq!(class_of("t_window"), MaterialClass::Glass);
        assert_eq!(class_of("t_railroad_track"), MaterialClass::Metal);
        assert_eq!(class_of("t_railroad_tie"), MaterialClass::Wood);
        assert_eq!(class_of("t_card_science"), MaterialClass::Metal);
        assert_eq!(class_of("t_open_air"), MaterialClass::Unknown);
    }

    #[test]
    fn rule_names_are_auditable() {
        let c = classify("t_concrete_wall", None, None, &no_ov());
        assert_eq!(c.rule.name(), "pattern:wall-material");
        let c = classify("t_open_air", None, None, &no_ov());
        assert_eq!(c.rule.bucket(), "pattern");
        assert_eq!(c.class, MaterialClass::Unknown);
    }

    #[test]
    fn override_wins_and_invalid_override_is_loud() {
        let ov = parse_overrides("[t_grass]\nmaterial = \"stone\"\n").unwrap();
        let c = classify("t_grass", None, None, &ov);
        assert_eq!(c.class, MaterialClass::Stone);
        assert_eq!(c.rule, Rule::Override);
        let ov = parse_overrides("[t_grass]\nmaterial = \"unobtanium\"\n").unwrap();
        let c = classify("t_grass", None, None, &ov);
        assert_eq!(c.class, MaterialClass::Unknown);
        assert_eq!(c.rule, Rule::OverrideInvalid);
    }

    #[test]
    fn kind_gate_for_monsters() {
        let c = classify("mon_zombie", Some(TileKind::Monster), None, &no_ov());
        assert_eq!(c.rule, Rule::KindNotBlock);
    }

    #[test]
    fn color_fallback_nearest_centroid() {
        // A palette of pure UltiCa grass green must fall back to Grass.
        let palette = vec![PaletteColor {
            lab: Oklab::from_srgb_u8([94, 128, 54]),
            srgb: [94, 128, 54],
            weight: 1.0,
            count: 100,
        }];
        let c = classify("zxqy_unpatterned", None, Some(&palette), &no_ov());
        assert_eq!(c.class, MaterialClass::Grass);
        assert_eq!(c.rule.bucket(), "color-fallback");
        // Blue palette -> Water.
        let palette = vec![PaletteColor {
            lab: Oklab::from_srgb_u8([52, 96, 150]),
            srgb: [52, 96, 150],
            weight: 1.0,
            count: 100,
        }];
        let c = classify("zxqy_unpatterned", None, Some(&palette), &no_ov());
        assert_eq!(c.class, MaterialClass::Water);
    }

    #[test]
    fn unknown_when_nothing_fires() {
        let c = classify("zxqy_unpatterned", None, None, &no_ov());
        assert_eq!(c.class, MaterialClass::Unknown);
        assert_eq!(c.rule, Rule::Unknown);
    }

    #[test]
    fn determinism_classify_twice() {
        let palette = vec![PaletteColor {
            lab: Oklab::from_srgb_u8([150, 140, 130]),
            srgb: [150, 140, 130],
            weight: 1.0,
            count: 10,
        }];
        for id in ["t_grass", "t_wall_wood", "f_chair", "zxqy", "t_door_metal_o"] {
            let a = classify(id, None, Some(&palette), &no_ov());
            let b = classify(id, None, Some(&palette), &no_ov());
            assert_eq!(a, b, "classify {id} must be deterministic");
        }
    }
}
