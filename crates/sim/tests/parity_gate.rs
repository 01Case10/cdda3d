//! S1 parity gates (the core of the slice): the sim's Snapshot,
//! built from a scene file + the real def registry, must be
//! SEMANTICALLY equal to the corresponding `proto::samples`
//! fixture. Ids differ between the two tables (sample ids are
//! hand-assigned; registry ids are per-kind sorted), so every id
//! is translated to its string id through its OWN table before
//! comparison: sample ids via the sample TileDefs, sim ids via
//! the registry. Everything else — fields, items, entities,
//! vehicles, view — is compared directly.
//!
//! S3 AMENDMENT (2026-11-23): lit/seen/outside are no longer
//! compared against the fixtures. The samples encode the Phase
//! 1/10 stub's documented simplifications (lit by z-level rule;
//! stub_house outside = "perimeter ring is exposed"), which the
//! real exporter would not produce either: the sim now computes
//! all three from the game's own models (apparent_light_at,
//! build_outside_cache's INDOORS stamp — which marks the stub
//! house's wall ring NOT outside, through the wall, by design).
//! The computed values are gated against hand-derived
//! expectations in visibility_gate.rs / environment_gate.rs;
//! here we assert only that the arrays are present and full
//! length (the sim always emits them, as the real exporter
//! does).

use std::collections::HashMap;
use std::path::PathBuf;

use proto::samples::{
    sample_implicit_roof_snapshot, sample_roofed_house_snapshot, sample_roofed_tiledefs,
    sample_snapshot, sample_tiledefs,
};
use proto::{SnapshotT, TileDefsT, TileKind};
use sim::{DefRegistry, Kind, Scene, World};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("scenes/{name}.toml"))
}

fn kind_of(tk: TileKind) -> Kind {
    match tk {
        TileKind::Terrain => Kind::Terrain,
        TileKind::Furniture => Kind::Furniture,
        TileKind::Trap => Kind::Trap,
        TileKind::Field => Kind::Field,
        TileKind::Monster => Kind::Monster,
        TileKind::Item => Kind::Item,
        TileKind::VehiclePart => Kind::VehiclePart,
        other => panic!("unknown TileKind {other:?} in sample defs"),
    }
}

/// Sample-side translation table: (kind, sample id) -> string id,
/// merged over the given sample tables in order.
///
/// NOTE (fixture quirk): `sample_roofed_tiledefs()` contains no
/// Item def at all, yet the roofed snapshots place ITEM_ROCK
/// (id 40) — that id is only defined in `sample_tiledefs()`. The
/// samples share one hand-assigned id space (TER_FLOOR=1 in both
/// tables, etc.), so the roofed parity checks merge both tables;
/// the ids agree wherever both define them.
fn sample_strings(tds: &[&TileDefsT]) -> HashMap<(Kind, u32), String> {
    let mut m = HashMap::new();
    for td in tds {
        for d in td.defs.as_ref().unwrap() {
            m.insert(
                (kind_of(d.kind), d.id),
                d.string_id.clone().unwrap_or_default(),
            );
        }
    }
    m
}

struct Cmp<'a> {
    reg: &'a DefRegistry,
    sample: &'a HashMap<(Kind, u32), String>,
    diffs: Vec<String>,
}

impl Cmp<'_> {
    fn sim_str(&self, kind: Kind, id: u32) -> String {
        if id == 0 {
            return "<none>".to_string();
        }
        self.reg
            .string_of(kind, id)
            .unwrap_or("<unknown>")
            .to_string()
    }

    fn sample_str(&self, kind: Kind, id: u32) -> String {
        if id == 0 {
            return "<none>".to_string();
        }
        self.sample
            .get(&(kind, id))
            .cloned()
            .unwrap_or_else(|| "<unknown>".to_string())
    }

    fn ids(&mut self, ctx: &str, kind: Kind, sim: &[u32], sample: &[u32]) {
        if sim.len() != sample.len() {
            self.diffs
                .push(format!("{ctx}: length {} vs {}", sim.len(), sample.len()));
            return;
        }
        let mut n = 0;
        for (i, (a, b)) in sim.iter().zip(sample.iter()).enumerate() {
            let (sa, sb) = (self.sim_str(kind, *a), self.sample_str(kind, *b));
            if sa != sb {
                n += 1;
                if n <= 10 {
                    self.diffs.push(format!(
                        "{ctx}[{i}] (x={}, y={}): sim {sa} vs sample {sb}",
                        i % 12,
                        i / 12
                    ));
                }
            }
        }
        if n > 10 {
            self.diffs.push(format!("{ctx}: ... and {} more", n - 10));
        }
    }

    fn eq<T: PartialEq + std::fmt::Debug>(&mut self, ctx: &str, sim: &T, sample: &T) {
        if sim != sample {
            self.diffs
                .push(format!("{ctx}: sim {sim:?} vs sample {sample:?}"));
        }
    }
}

fn assert_parity(scene_name: &str, sample: &SnapshotT, sample_defs: &[&TileDefsT]) {
    let reg = DefRegistry::load(&defs_path()).expect("registry loads");
    let scene = Scene::load_file(&scene_path(scene_name)).expect("scene loads");
    let mut world = World::from_scene(&reg, &scene).expect("world builds");
    assert!(
        !world.dirty_submaps().is_empty(),
        "world starts all-dirty (first send)"
    );
    let sim = world.snapshot(&reg);
    assert!(
        world.dirty_submaps().is_empty(),
        "snapshot clears the dirty set"
    );

    let strings = sample_strings(sample_defs);
    let mut c = Cmp {
        reg: &reg,
        sample: &strings,
        diffs: Vec::new(),
    };

    c.eq("view_origin", &sim.view_origin, &sample.view_origin);
    c.eq("view_radius", &sim.view_radius, &sample.view_radius);
    c.eq("seq", &sim.seq, &sample.seq);
    c.eq("epoch", &sim.epoch, &sample.epoch);
    c.eq(
        "environment presence",
        &sim.environment.is_some(),
        &sample.environment.is_some(),
    );

    let sim_blocks = sim.blocks.as_ref().unwrap();
    let sample_blocks = sample.blocks.as_ref().unwrap();
    c.eq("block count", &sim_blocks.len(), &sample_blocks.len());
    for (bi, (sb, pb)) in sim_blocks.iter().zip(sample_blocks.iter()).enumerate() {
        let ctx = format!("block {bi}");
        c.eq(&format!("{ctx} origin"), &sb.origin, &pb.origin);
        c.ids(
            &format!("{ctx} terrain"),
            Kind::Terrain,
            sb.terrain.as_ref().unwrap(),
            pb.terrain.as_ref().unwrap(),
        );
        c.ids(
            &format!("{ctx} furniture"),
            Kind::Furniture,
            sb.furniture.as_ref().unwrap(),
            pb.furniture.as_ref().unwrap(),
        );
        c.ids(
            &format!("{ctx} trap"),
            Kind::Trap,
            sb.trap.as_ref().unwrap(),
            pb.trap.as_ref().unwrap(),
        );
        // Fields: exact wire-order list of (tile, type string, intensity).
        let sf: Vec<(u16, String, u8)> = sb
            .fields
            .as_ref()
            .unwrap()
            .iter()
            .map(|f| (f.tile_index, c.sim_str(Kind::Field, f.type_), f.intensity))
            .collect();
        let pf: Vec<(u16, String, u8)> = pb
            .fields
            .as_ref()
            .unwrap()
            .iter()
            .map(|f| {
                (
                    f.tile_index,
                    c.sample_str(Kind::Field, f.type_),
                    f.intensity,
                )
            })
            .collect();
        c.eq(&format!("{ctx} fields"), &sf, &pf);
        c.eq(
            &format!("{ctx} field_index"),
            &sb.field_index,
            &pb.field_index,
        );
        // Displayed field per tile: (type string, intensity).
        let sd: Vec<(String, u8)> = sb
            .displayed_field
            .as_ref()
            .unwrap()
            .iter()
            .map(|d| (c.sim_str(Kind::Field, d.field_type), d.intensity))
            .collect();
        let pd: Vec<(String, u8)> = pb
            .displayed_field
            .as_ref()
            .unwrap()
            .iter()
            .map(|d| (c.sample_str(Kind::Field, d.field_type), d.intensity))
            .collect();
        c.eq(&format!("{ctx} displayed_field"), &sd, &pd);
        c.ids(
            &format!("{ctx} top_item"),
            Kind::Item,
            sb.top_item.as_ref().unwrap(),
            pb.top_item.as_ref().unwrap(),
        );
        c.eq(&format!("{ctx} item_count"), &sb.item_count, &pb.item_count);
        // lit/seen/outside: computed by the sim (S3), stub
        // simplifications in the fixtures — see the module docs.
        // Assert presence + shape only.
        for (name, len) in [
            ("lit", sb.lit.as_ref().map(|v| v.len())),
            ("seen", sb.seen.as_ref().map(|v| v.len())),
            ("outside", sb.outside.as_ref().map(|v| v.len())),
            ("light_frac", sb.light_frac.as_ref().map(|v| v.len())),
        ] {
            c.eq(&format!("{ctx} {name} length"), &len, &Some(144usize));
        }
        c.eq(
            &format!("{ctx} light_tint length"),
            &sb.light_tint.as_ref().map(|v| v.len()),
            &Some(432usize),
        );
        c.eq(
            &format!("{ctx} has_graffiti"),
            &sb.has_graffiti,
            &pb.has_graffiti,
        );
    }

    // Entities: sprite, pos, facing, overlays, hp_frac, flags, and
    // the type def as a string.
    let sim_ents = sim.entities.clone().unwrap_or_default();
    let sample_ents = sample.entities.clone().unwrap_or_default();
    c.eq("entity count", &sim_ents.len(), &sample_ents.len());
    for (i, (se, pe)) in sim_ents.iter().zip(sample_ents.iter()).enumerate() {
        let ctx = format!("entity {i}");
        c.eq(&format!("{ctx} id"), &se.id, &pe.id);
        c.eq(
            &format!("{ctx} type"),
            &c.sim_str(Kind::Monster, se.type_def),
            &c.sample_str(Kind::Monster, pe.type_def),
        );
        c.eq(&format!("{ctx} sprite"), &se.sprite_id, &pe.sprite_id);
        c.eq(&format!("{ctx} pos"), &se.pos, &pe.pos);
        c.eq(&format!("{ctx} facing"), &se.facing, &pe.facing);
        c.eq(&format!("{ctx} overlays"), &se.overlays, &pe.overlays);
        c.eq(&format!("{ctx} hp_frac"), &se.hp_frac, &pe.hp_frac);
        c.eq(&format!("{ctx} flags"), &se.flags, &pe.flags);
    }

    // Vehicles: id, name, continuous face_deg, parts as strings.
    let sim_vehs = sim.vehicles.clone().unwrap_or_default();
    let sample_vehs = sample.vehicles.clone().unwrap_or_default();
    c.eq("vehicle count", &sim_vehs.len(), &sample_vehs.len());
    for (i, (sv, pv)) in sim_vehs.iter().zip(sample_vehs.iter()).enumerate() {
        let ctx = format!("vehicle {i}");
        c.eq(&format!("{ctx} id"), &sv.id, &pv.id);
        c.eq(&format!("{ctx} name"), &sv.name, &pv.name);
        c.eq(&format!("{ctx} face_deg"), &sv.face_deg, &pv.face_deg);
        let sp: Vec<(String, _, _, u8, u32)> = sv
            .parts
            .as_ref()
            .map(|ps| {
                ps.iter()
                    .map(|p| {
                        (
                            c.sim_str(Kind::VehiclePart, p.vpart),
                            p.abs_pos.clone(),
                            p.mount.clone(),
                            p.hp_frac,
                            p.flags,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        let pp: Vec<(String, _, _, u8, u32)> = pv
            .parts
            .as_ref()
            .map(|ps| {
                ps.iter()
                    .map(|p| {
                        (
                            c.sample_str(Kind::VehiclePart, p.vpart),
                            p.abs_pos.clone(),
                            p.mount.clone(),
                            p.hp_frac,
                            p.flags,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        c.eq(&format!("{ctx} parts"), &sp, &pp);
    }

    assert!(
        c.diffs.is_empty(),
        "scene '{scene_name}' parity diffs ({}):\n{}",
        c.diffs.len(),
        c.diffs.join("\n")
    );
}

#[test]
fn parity_stub_house() {
    assert_parity("stub_house", &sample_snapshot(), &[&sample_tiledefs()]);
}

#[test]
fn parity_roofed_house_explicit() {
    assert_parity(
        "roofed_house_explicit",
        &sample_roofed_house_snapshot(),
        &[&sample_tiledefs(), &sample_roofed_tiledefs()],
    );
}

#[test]
fn parity_roofed_house_implicit() {
    assert_parity(
        "roofed_house_implicit",
        &sample_implicit_roof_snapshot(),
        &[&sample_tiledefs(), &sample_roofed_tiledefs()],
    );
}
