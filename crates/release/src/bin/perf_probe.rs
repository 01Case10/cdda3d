//! Gate 8.10 probe: per-operation costs of the Phase 8 surfaces,
//! run in release mode (`cargo run --release -p release
//! --bin perf-probe`). The numbers are recorded in
//! docs/phase8-hardcases.md; the budgets they are checked against
//! live there too. (Frame-time budgets are target-hardware
//! numbers — RX 9070 XT — and stay open with 2c.9.)

use std::time::Instant;

use hardcases::{classify_transition, floor_at, Column, DefTable, Floor};
use proto::DeltaT;
use transient::TransientSystem;
use vehicles::fixtures::truth_vehicle;
use vehicles::VehicleStore;

fn time<F: FnMut()>(label: &str, iters: usize, mut f: F) {
    // Warm-up, then measure.
    for _ in 0..100 {
        f();
    }
    let t = Instant::now();
    for _ in 0..iters {
        f();
    }
    let per_us = t.elapsed().as_secs_f64() * 1e6 / iters as f64;
    println!("{label}: {per_us:.2} us/op over {iters} ops");
}

fn main() {
    // Vehicle delta: 20 vehicles updating every turn.
    let mut store = VehicleStore::new();
    let mut seq = 0u64;
    time("vehicle delta apply (20 vehicles)", 2000, || {
        seq += 1;
        let mut d = DeltaT::default();
        d.seq = seq;
        d.epoch = 1;
        d.vehicle_updates = Some(
            (0..20)
                .map(|i| truth_vehicle(i, [20 + i as i32, 20, 0], (seq % 360) as f32))
                .collect(),
        );
        store.apply_delta(&d);
    });

    // Transient ingest: a 2-event burst, then drain.
    let mut sys = TransientSystem::new();
    let mut now = 0u64;
    time("transient ingest (2 events)", 2000, || {
        now += 1000;
        let mut fx = proto::EffectsT::default();
        fx.seq = now;
        let mut e = proto::EffectEventT::default();
        e.kind = proto::EffectKind::Explosion;
        e.effect_id = Some("explosion".into());
        e.to_pos = Some(proto::Vec3iT { x: 1, y: 1, z: 0 });
        e.duration_ms = 900;
        fx.events = Some(vec![e]);
        sys.ingest(&fx);
        sys.advance(now);
    });

    // Door classification + floor resolution on the sample defs.
    let defs = proto::samples::sample_tiledefs();
    let table = DefTable::from_tiledefs(&defs);
    time("door classify_transition", 100_000, || {
        std::hint::black_box(classify_transition(
            &table,
            proto::TileKind::Terrain,
            proto::samples::TER_DOOR,
            proto::samples::TER_DOOR_OPEN,
        ));
    });
    let col = Column {
        z_min: 0,
        terrain: vec![proto::samples::TER_FLOOR, table.open_air_id],
    };
    time("open-air floor_at", 100_000, || {
        std::hint::black_box(floor_at(&table, &col, 1));
    });
    let f = floor_at(&table, &col, 1);
    println!(
        "floor_at sanity: {}",
        matches!(f, Floor::Solid(_) | Floor::RoofOf(..) | Floor::None)
    );
}
