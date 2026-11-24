//! World state + Snapshot builder (slice S1).
//!
//! The `World` holds tile state per submap block in *registry*
//! wire ids (per-kind, from [`crate::defs::DefRegistry`]), the
//! entity/vehicle lists, and a dirty-submap set — the shape the
//! S2 turn loop needs: mutate world -> mark submap dirty -> diff
//! -> Delta. S1 only builds the first Snapshot (every submap
//! starts dirty, i.e. the all-dirty first send) and clears the
//! set once the Snapshot is taken.
//!
//! S1 visibility is a documented placeholder rule, replaced by
//! the real LOS/roof computation in S3:
//! - `lit`/`seen` by z level: z < 0 -> DARK / 0 (never seen),
//!   z == 0 -> LIT / 255, z > 0 -> BRIGHT / 255 (open sky at the
//!   sample environment's noon).
//! - `outside[]` is only emitted when the scene sets
//!   `emit_outside`: z < 0 -> all false, z > 0 -> all true, and at
//!   z == 0 an enclosure flood: starting from the block border,
//!   flood through tiles whose terrain is not a barrier; barrier
//!   tiles (authored flags contain WALL, DOOR, or WINDOW — the
//!   perimeter set) and reached tiles are outside, unreached
//!   interior tiles are not. This reproduces the stub house's
//!   documented rule (interior floor sheltered, perimeter
//!   exposed) from the defs' own flags, not from sample values.
//!
//! `displayed_field` follows the sample rule (§3.2): fire is
//! displayed over other fields; otherwise the highest intensity
//! wins; ties keep the first entry.

use std::collections::HashSet;

use proto::samples::sample_environment;
use proto::{
    DisplayedFieldT, EntityT, Facing, FieldOnTileT, LitLevel, SnapshotT, SubmapBlockT, Vec3iT,
    VehiclePartStateT, VehicleT,
};

use crate::defs::{DefRegistry, Kind};
use crate::scenes::{Scene, SUBMAP};

pub const TILES: usize = SUBMAP * SUBMAP;

#[derive(Debug, Clone, Default)]
pub struct TileState {
    pub terrain: u32,
    pub furniture: u32,
    pub trap: u32,
    /// (field def id, intensity) in wire order.
    pub fields: Vec<(u32, u8)>,
    pub top_item: u32,
    pub item_count: u16,
    pub graffiti: bool,
}

#[derive(Debug, Clone)]
pub struct BlockState {
    pub origin: Vec3iT,
    pub tiles: Vec<TileState>,
}

#[derive(Debug)]
pub struct World {
    pub name: String,
    pub blocks: Vec<BlockState>,
    /// (origin x, y, z) of submaps changed since the last send.
    dirty: HashSet<(i32, i32, i32)>,
    pub entities: Vec<EntityT>,
    pub vehicles: Vec<VehicleT>,
    pub player_pos: Option<[i32; 3]>,
    pub view_origin: Vec3iT,
    pub view_radius: u32,
    pub environment: bool,
    pub emit_outside: bool,
}

fn v3(x: i32, y: i32, z: i32) -> Vec3iT {
    Vec3iT { x, y, z }
}

fn idx(x: usize, y: usize) -> usize {
    y * SUBMAP + x
}

impl World {
    pub fn from_scene(reg: &DefRegistry, scene: &Scene) -> Result<World, String> {
        let mut blocks: Vec<BlockState> = Vec::new();
        for zs in &scene.zsections {
            let mut tiles = vec![TileState::default(); TILES];
            let layer = |rows: &[String], x: usize, y: usize| -> char {
                rows[y]
                    .chars()
                    .nth(x)
                    .expect("layer rows are width-checked")
            };
            for y in 0..SUBMAP {
                for x in 0..SUBMAP {
                    let t = &mut tiles[idx(x, y)];
                    let c = layer(&zs.map, x, y);
                    let sid = zs.legend.get(&c).expect("map chars are legend-checked");
                    t.terrain = reg.id_of(Kind::Terrain, sid);
                    if t.terrain == 0 {
                        return Err(format!(
                            "scene '{}': legend char '{c}' -> '{sid}' is not a Terrain def",
                            scene.name
                        ));
                    }
                    if let Some(furn) = &zs.furn {
                        let fc = layer(furn, x, y);
                        if fc != '.' {
                            let fsid = zs.legend.get(&fc).expect("furn chars are legend-checked");
                            t.furniture = reg.id_of(Kind::Furniture, fsid);
                            if t.furniture == 0 {
                                return Err(format!(
                                    "scene '{}': furn char '{fc}' -> '{fsid}' is not a Furniture def",
                                    scene.name
                                ));
                            }
                        }
                    }
                    if let Some(trap) = &zs.trap {
                        let tc = layer(trap, x, y);
                        if tc != '.' {
                            let tsid = zs.legend.get(&tc).expect("trap chars are legend-checked");
                            t.trap = reg.id_of(Kind::Trap, tsid);
                            if t.trap == 0 {
                                return Err(format!(
                                    "scene '{}': trap char '{tc}' -> '{tsid}' is not a Trap def",
                                    scene.name
                                ));
                            }
                        }
                    }
                }
            }
            blocks.push(BlockState {
                origin: v3(0, 0, zs.z),
                tiles,
            });
        }

        // Sparse per-tile extras, addressed by absolute pos. The
        // block containing the pos must exist (single-submap S1
        // scenes: x/y in 0..12, z one of the sections).
        fn place(
            blocks: &[BlockState],
            scene_name: &str,
            pos: [i32; 3],
            what: &str,
        ) -> Result<(usize, usize), String> {
            let b = blocks
                .iter()
                .position(|b| b.origin.z == pos[2])
                .ok_or_else(|| {
                    format!(
                        "scene '{scene_name}': {what} at {pos:?} has no [z{}] section",
                        pos[2]
                    )
                })?;
            if !(0..SUBMAP as i32).contains(&pos[0]) || !(0..SUBMAP as i32).contains(&pos[1]) {
                return Err(format!(
                    "scene '{scene_name}': {what} at {pos:?} is outside the 12x12 submap (S1 scenes are one submap)"
                ));
            }
            Ok((b, idx(pos[0] as usize, pos[1] as usize)))
        }
        for f in &scene.fields {
            let (b, i) = place(&blocks, &scene.name, f.pos, "field")?;
            let id = reg.id_of(Kind::Field, &f.type_id);
            if id == 0 {
                return Err(format!(
                    "scene '{}': field type '{}' is not a Field def",
                    scene.name, f.type_id
                ));
            }
            blocks[b].tiles[i].fields.push((id, f.intensity));
        }
        for it in &scene.items {
            let (b, i) = place(&blocks, &scene.name, it.pos, "items")?;
            let id = reg.id_of(Kind::Item, &it.top);
            if id == 0 {
                return Err(format!(
                    "scene '{}': item '{}' is not an Item def",
                    scene.name, it.top
                ));
            }
            blocks[b].tiles[i].top_item = id;
            blocks[b].tiles[i].item_count = it.count;
        }
        for g in &scene.graffiti {
            let (b, i) = place(&blocks, &scene.name, *g, "graffiti")?;
            blocks[b].tiles[i].graffiti = true;
        }

        // Entities in file order, ids 1.. — author the player
        // first (the real adapter emits the avatar first).
        let mut entities = Vec::new();
        for (n, s) in scene.spawns.iter().enumerate() {
            let (type_def, sprite) = if s.type_id == "player" {
                (0, s.sprite.clone().unwrap_or_else(|| "player".to_string()))
            } else {
                let id = reg.id_of(Kind::Monster, &s.type_id);
                if id == 0 {
                    return Err(format!(
                        "scene '{}': spawn type '{}' is not a Monster def",
                        scene.name, s.type_id
                    ));
                }
                (id, s.sprite.clone().unwrap_or_else(|| s.type_id.clone()))
            };
            let facing = match s.facing.as_str() {
                "NONE" => Facing::NONE,
                "LEFT" => Facing::LEFT,
                "RIGHT" => Facing::RIGHT,
                other => {
                    return Err(format!(
                        "scene '{}': spawn facing '{other}' is not NONE/LEFT/RIGHT",
                        scene.name
                    ))
                }
            };
            // Generated table types are #[non_exhaustive] outside
            // the proto crate: Default + field assignment.
            let mut e = EntityT::default();
            e.id = (n + 1) as u64;
            e.type_def = type_def;
            e.sprite_id = Some(sprite);
            e.pos = Some(v3(s.pos[0], s.pos[1], s.pos[2]));
            e.facing = facing;
            e.overlays = if s.overlays.is_empty() {
                None
            } else {
                Some(s.overlays.clone())
            };
            e.hp_frac = s.hp_frac;
            e.flags = s.flags;
            entities.push(e);
        }

        let mut vehicles = Vec::new();
        for v in &scene.vehicles {
            let parts: Vec<VehiclePartStateT> = scene
                .vparts
                .iter()
                .filter(|p| p.vehicle == v.id)
                .map(|p| -> Result<VehiclePartStateT, String> {
                    let id = reg.id_of(Kind::VehiclePart, &p.type_id);
                    if id == 0 {
                        return Err(format!(
                            "scene '{}': vpart type '{}' is not a VehiclePart def",
                            scene.name, p.type_id
                        ));
                    }
                    let mut part = VehiclePartStateT::default();
                    part.vpart = id;
                    part.abs_pos = Some(v3(p.pos[0], p.pos[1], p.pos[2]));
                    part.mount = Some(v3(p.mount[0], p.mount[1], p.mount[2]));
                    part.hp_frac = p.hp_frac;
                    part.flags = p.flags;
                    Ok(part)
                })
                .collect::<Result<_, _>>()?;
            let mut veh = VehicleT::default();
            veh.id = v.id;
            veh.name = if v.name.is_empty() {
                None
            } else {
                Some(v.name.clone())
            };
            veh.face_deg = v.face_deg;
            veh.parts = if parts.is_empty() { None } else { Some(parts) };
            vehicles.push(veh);
        }

        let dirty = blocks
            .iter()
            .map(|b| (b.origin.x, b.origin.y, b.origin.z))
            .collect();
        Ok(World {
            name: scene.name.clone(),
            blocks,
            dirty,
            entities,
            vehicles,
            player_pos: scene.player,
            view_origin: v3(
                scene.view_origin[0],
                scene.view_origin[1],
                scene.view_origin[2],
            ),
            view_radius: scene.view_radius,
            environment: scene.environment,
            emit_outside: scene.emit_outside,
        })
    }

    /// Submaps changed since the last Snapshot/Delta send. S1:
    /// every submap is dirty until the first Snapshot.
    pub fn dirty_submaps(&self) -> &HashSet<(i32, i32, i32)> {
        &self.dirty
    }

    pub fn mark_dirty(&mut self, origin: (i32, i32, i32)) {
        self.dirty.insert(origin);
    }

    /// Build the Snapshot (seq 1, epoch 1 in S1) and clear the
    /// dirty set — the all-dirty first send.
    pub fn snapshot(&mut self, reg: &DefRegistry) -> SnapshotT {
        let blocks: Vec<SubmapBlockT> = self
            .blocks
            .iter()
            .map(|b| self.build_block(reg, b))
            .collect();
        self.dirty.clear();
        let mut snap = SnapshotT::default();
        snap.view_origin = Some(self.view_origin.clone());
        snap.view_radius = self.view_radius;
        snap.blocks = Some(blocks);
        snap.memory_blocks = None;
        snap.entities = if self.entities.is_empty() {
            None
        } else {
            Some(self.entities.clone())
        };
        snap.vehicles = if self.vehicles.is_empty() {
            None
        } else {
            Some(self.vehicles.clone())
        };
        snap.seq = 1;
        snap.epoch = 1;
        // S1 placeholder: the canonical sample environment
        // (noon/clear/spring). Time/weather simulation is S3.
        snap.environment = if self.environment {
            Some(Box::new(sample_environment()))
        } else {
            None
        };
        snap
    }

    fn build_block(&self, reg: &DefRegistry, b: &BlockState) -> SubmapBlockT {
        let z = b.origin.z;
        let mut terrain = Vec::with_capacity(TILES);
        let mut furniture = Vec::with_capacity(TILES);
        let mut trap = Vec::with_capacity(TILES);
        let mut fields: Vec<FieldOnTileT> = Vec::new();
        let mut field_index = vec![-1i32; TILES];
        let mut displayed = vec![
            DisplayedFieldT {
                field_type: 0,
                intensity: 0
            };
            TILES
        ];
        let mut top_item = Vec::with_capacity(TILES);
        let mut item_count = Vec::with_capacity(TILES);
        let mut graffiti = Vec::with_capacity(TILES);
        for (i, t) in b.tiles.iter().enumerate() {
            terrain.push(t.terrain);
            furniture.push(t.furniture);
            trap.push(t.trap);
            top_item.push(t.top_item);
            item_count.push(t.item_count);
            graffiti.push(t.graffiti);
            if !t.fields.is_empty() {
                field_index[i] = fields.len() as i32;
                for &(type_, intensity) in &t.fields {
                    let mut f = FieldOnTileT::default();
                    f.tile_index = i as u16;
                    f.type_ = type_;
                    f.intensity = intensity;
                    fields.push(f);
                }
                let (dt, di) = displayed_field(reg, &t.fields);
                displayed[i] = DisplayedFieldT {
                    field_type: dt,
                    intensity: di,
                };
            }
        }
        let (lit_level, seen_val) = match z {
            z if z < 0 => (LitLevel::DARK, 0u8),
            0 => (LitLevel::LIT, 255u8),
            _ => (LitLevel::BRIGHT, 255u8),
        };
        let mut block = SubmapBlockT::default();
        block.origin = Some(b.origin.clone());
        block.terrain = Some(terrain);
        block.furniture = Some(furniture);
        block.trap = Some(trap);
        block.fields = Some(fields);
        block.field_index = Some(field_index);
        block.displayed_field = Some(displayed);
        block.top_item = Some(top_item);
        block.item_count = Some(item_count);
        block.lit = Some(vec![lit_level; TILES]);
        block.seen = Some(vec![seen_val; TILES]);
        block.has_graffiti = Some(graffiti);
        block.outside = if self.emit_outside {
            Some(self.outside_for(reg, b))
        } else {
            None
        };
        block
    }

    fn outside_for(&self, reg: &DefRegistry, b: &BlockState) -> Vec<bool> {
        let z = b.origin.z;
        if z < 0 {
            return vec![false; TILES];
        }
        if z > 0 {
            return vec![true; TILES];
        }
        // z == 0 enclosure flood (see module docs).
        let barrier: Vec<bool> = b
            .tiles
            .iter()
            .map(|t| {
                reg.def_by_id(Kind::Terrain, t.terrain).is_some_and(|d| {
                    d.flags
                        .iter()
                        .any(|f| f == "WALL" || f == "DOOR" || f == "WINDOW")
                })
            })
            .collect();
        let mut reached = vec![false; TILES];
        let mut stack: Vec<(usize, usize)> = Vec::new();
        for i in 0..SUBMAP {
            for (x, y) in [(i, 0), (i, SUBMAP - 1), (0, i), (SUBMAP - 1, i)] {
                if !barrier[idx(x, y)] && !reached[idx(x, y)] {
                    reached[idx(x, y)] = true;
                    stack.push((x, y));
                }
            }
        }
        while let Some((x, y)) = stack.pop() {
            for (nx, ny) in [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ] {
                if nx < SUBMAP && ny < SUBMAP && !barrier[idx(nx, ny)] && !reached[idx(nx, ny)] {
                    reached[idx(nx, ny)] = true;
                    stack.push((nx, ny));
                }
            }
        }
        (0..TILES).map(|i| barrier[i] || reached[i]).collect()
    }
}

/// The displayed-field rule (§3.2, as in the sample house): fire
/// is displayed over other fields; otherwise the highest
/// intensity; ties keep the first entry. (0, 0) = none.
fn displayed_field(reg: &DefRegistry, fields: &[(u32, u8)]) -> (u32, u8) {
    let fire_id = reg.id_of(Kind::Field, "fd_fire");
    let mut best: Option<(u32, u8)> = None;
    for &(type_, intensity) in fields {
        best = match best {
            None => Some((type_, intensity)),
            Some((bt, bi)) => {
                let cur_is_fire = type_ == fire_id && fire_id != 0;
                let best_is_fire = bt == fire_id && fire_id != 0;
                if (cur_is_fire && !best_is_fire) || (cur_is_fire == best_is_fire && intensity > bi)
                {
                    Some((type_, intensity))
                } else {
                    best
                }
            }
        };
    }
    best.unwrap_or((0, 0))
}
