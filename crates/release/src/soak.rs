//! The scripted soak (gates 8.8/8.9): a deterministic dummy
//! session driven through the Phase 8 stores at accelerated
//! clock — the stand-in for 2 h of play until the real fork can
//! run it in-game (0.1/0.2). One step = one game turn (1 s):
//! a vehicle drives and turns, effects fire and expire, a door
//! opens and closes, extra vehicles spawn and despawn, snapshots
//! resync periodically.
//!
//! What it proves at this level: no panics over a long session,
//! bounded state (transient emitters always drain; the vehicle
//! store holds only live vehicles), stable per-step cost (no
//! creeping slowdown), and flat resident memory after warm-up
//! (the leak check — RSS sampled from /proc, Linux).

use std::time::Instant;

use hardcases::{DefTable, TileWorld};
use proto::{DeltaT, EffectEventT, EffectKind, EffectsT, TileChangeT, Vec3iT};
use transient::TransientSystem;
use vehicles::fixtures::truth_vehicle;
use vehicles::VehicleStore;

/// Resident set size in KiB (Linux /proc), None elsewhere.
pub fn rss_kib() -> Option<u64> {
    let text = std::fs::read_to_string("/proc/self/statm").ok()?;
    let resident_pages: u64 = text.split_whitespace().nth(1)?.parse().ok()?;
    Some(resident_pages * 4) // 4 KiB pages on x86_64/arm64 Linux
}

#[derive(Debug, Clone, Copy)]
pub struct SoakConfig {
    /// Steps (game turns) to play. 7200 = 2 h at 1 turn/s.
    pub steps: usize,
}

impl Default for SoakConfig {
    fn default() -> Self {
        Self { steps: 7200 }
    }
}

#[derive(Debug, Clone)]
pub struct SoakStats {
    pub steps: usize,
    pub max_transient_alive: usize,
    pub transient_alive_at_end: usize,
    pub vehicles_at_end: usize,
    pub door_toggles: usize,
    pub rss_start_kib: Option<u64>,
    pub rss_warm_kib: Option<u64>,
    pub rss_end_kib: Option<u64>,
    /// Mean per-step cost (µs) over the first and last full
    /// 600-step blocks — the creep check compares them.
    pub first_block_us_per_step: f64,
    pub last_block_us_per_step: f64,
}

/// xorshift64 — deterministic across runs and platforms.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

fn effects(seq: u64, step: usize) -> EffectsT {
    let mk = |kind: EffectKind, id: &str, from: [i32; 3], to: [i32; 3], dur: u32| {
        let mut e = EffectEventT::default();
        e.kind = kind;
        e.effect_id = Some(id.into());
        e.from_pos = Some(Vec3iT {
            x: from[0],
            y: from[1],
            z: from[2],
        });
        e.to_pos = Some(Vec3iT {
            x: to[0],
            y: to[1],
            z: to[2],
        });
        e.duration_ms = dur;
        e
    };
    let mut fx = EffectsT::default();
    fx.seq = seq;
    fx.events = Some(vec![
        mk(
            EffectKind::Projectile,
            "bullet_rifle",
            [10, 10, 0],
            [10 + (step % 9) as i32, 14, 0],
            120,
        ),
        mk(
            EffectKind::Explosion,
            "explosion",
            [30, 30, 0],
            [30, 30, 0],
            900,
        ),
    ]);
    fx
}

pub fn run_soak(config: SoakConfig) -> SoakStats {
    let defs = proto::samples::sample_tiledefs();
    let table = DefTable::from_tiledefs(&defs);
    let snapshot = proto::samples::sample_snapshot();
    let mut world = TileWorld::default();
    world.terrain.insert([5, 11, 0], proto::samples::TER_DOOR);
    let mut vehicles = VehicleStore::new();
    vehicles.apply_snapshot(&snapshot);
    let mut transient = TransientSystem::new();
    let mut now_ms: u64 = 0;
    let mut rng = Rng(0x9E3779B97F4A7C15);

    let rss_start = rss_kib();
    let mut rss_warm = None;
    let mut max_transient = 0usize;
    let mut door_open = false;
    let mut toggles = 0usize;
    let mut block_costs: Vec<f64> = Vec::new();
    let mut block_time = Instant::now();
    let mut block_steps = 0usize;

    for step in 0..config.steps {
        let t0 = Instant::now();
        // The car drives: pivot creeps, facing sweeps (incl. the
        // 359°→0° wrap once per revolution).
        let face = (step * 3 % 360) as f32;
        let pivot = [20 + (step / 60) as i32 % 8, 20 + (step / 240) as i32 % 4, 0];
        let mut delta = DeltaT::default();
        delta.seq = step as u64 + 1;
        delta.epoch = 1;
        let mut updates = vec![truth_vehicle(7, pivot, face)];
        // Traffic: a second vehicle exists for 250 of every 500
        // steps, then is removed.
        if step % 500 < 250 {
            updates.push(truth_vehicle(42, [40, 40, 0], (step % 360) as f32));
        } else if step % 500 == 250 {
            delta.vehicle_removed = Some(vec![42]);
        }
        delta.vehicle_updates = Some(updates);
        vehicles.apply_delta(&delta);
        // Periodic full resync (the epoch mechanism's steady
        // state), with jitter from the rng.
        if step % 997 == 996 {
            let mut snap = proto::samples::sample_snapshot();
            snap.seq = step as u64 + 1;
            vehicles.apply_snapshot(&snap);
            let _ = rng.next();
        }
        // Effects fire every 7 turns; the clock advances 1 s/turn
        // (advance takes the absolute monotonic time). Advance
        // first, then ingest: emitters are stamped with the
        // system's current time, so a burst ingested after the
        // advance lives from now until its duration elapses.
        now_ms += 1000;
        transient.advance(now_ms);
        if step % 7 == 0 {
            transient.ingest(&effects(step as u64 + 1, step));
        }
        max_transient = max_transient.max(transient.active().len());
        // The sample house's door toggles every 11 turns.
        if step % 11 == 0 {
            door_open = !door_open;
            toggles += 1;
            let mut change = TileChangeT::default();
            change.pos = Some(Vec3iT { x: 5, y: 11, z: 0 });
            change.terrain = if door_open {
                proto::samples::TER_DOOR_OPEN
            } else {
                proto::samples::TER_DOOR
            };
            world.apply_change(&table, &change);
        }
        let _ = t0;
        block_steps += 1;
        if block_steps == 600 {
            block_costs.push(block_time.elapsed().as_secs_f64() * 1e6 / 600.0);
            block_time = Instant::now();
            block_steps = 0;
            if rss_warm.is_none() {
                rss_warm = rss_kib();
            }
        }
    }
    // Drain: after the last effect, 2 s of clock must empty the
    // system (no leftovers, the 8.5 invariant at session scale).
    transient.advance(now_ms + 2000);

    SoakStats {
        steps: config.steps,
        max_transient_alive: max_transient,
        transient_alive_at_end: transient.active().len(),
        vehicles_at_end: vehicles.len(),
        door_toggles: toggles,
        rss_start_kib: rss_start,
        rss_warm_kib: rss_warm,
        rss_end_kib: rss_kib(),
        first_block_us_per_step: block_costs.first().copied().unwrap_or(0.0),
        last_block_us_per_step: block_costs.last().copied().unwrap_or(0.0),
    }
}
