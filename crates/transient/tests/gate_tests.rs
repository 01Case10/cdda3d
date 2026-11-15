//! Phase 8 gate test 8.5 at model level (Part II §11): effects
//! appear at the right tiles and times, and no effects are left
//! over. The in-game half (firing a real weapon in the fork and
//! watching the dispatch forward it) stays blocked on 0.1/0.2.

use proto::{EffectEventT, EffectKind, EffectsT, Vec3iT};
use transient::{params_for, TransientSystem, CLASS_ENERGY, CLASS_FLAME};

fn v3(x: i32, y: i32, z: i32) -> Option<Vec3iT> {
    Some(Vec3iT { x, y, z })
}

// Generated wire structs are non-exhaustive outside the proto
// crate: build via Default + assignment.
fn event(
    kind: EffectKind,
    id: &str,
    from: (i32, i32, i32),
    to: (i32, i32, i32),
    ms: u32,
) -> EffectEventT {
    let mut e = EffectEventT::default();
    e.kind = kind;
    e.effect_id = Some(id.into());
    e.from_pos = v3(from.0, from.1, from.2);
    e.to_pos = v3(to.0, to.1, to.2);
    e.duration_ms = ms;
    e
}

fn effects(seq: u64, events: Vec<EffectEventT>) -> EffectsT {
    let mut e = EffectsT::default();
    e.seq = seq;
    e.events = Some(events);
    e
}

fn projectile(seq: u64) -> EffectsT {
    // The proto sample event (crates/proto samples): a bullet from
    // (1,1,0) to (6,5,0) over 120 ms.
    effects(
        seq,
        vec![event(
            EffectKind::Projectile,
            "bullet",
            (1, 1, 0),
            (6, 5, 0),
            120,
        )],
    )
}

/// Gate 8.5 — a projectile is at its from-tile at spawn, travels
/// the segment, reaches the to-tile at the end of its duration,
/// and is gone at the end instant (no leftover).
#[test]
fn gate_8_5_projectile_right_tiles_and_times_then_gone() {
    let mut sys = TransientSystem::new();
    assert_eq!(sys.ingest(&projectile(4)), 1);
    let e = &sys.active()[0];
    assert_eq!(e.position(0), [1.0, 1.0, 0.0], "spawned at the from tile");
    assert_eq!(e.position(60), [3.5, 3.0, 0.0], "halfway at half duration");
    assert_eq!(e.params.unwrap().render_class, CLASS_ENERGY);

    sys.advance(119);
    assert_eq!(sys.active().len(), 1, "still alive just before the end");
    sys.advance(120);
    assert!(
        sys.is_empty(),
        "at the end instant the effect is gone — no leftover"
    );
}

/// Gate 8.5 — an explosion burst sits on its target tile with
/// flame parameters and expires cleanly; a batch of mixed events
/// all expire (the system returns to empty, not merely smaller).
#[test]
fn gate_8_5_burst_and_batch_leave_nothing_behind() {
    let mut sys = TransientSystem::new();
    let batch = effects(
        9,
        vec![
            event(
                EffectKind::Explosion,
                "explosion",
                (10, 10, 0),
                (10, 10, 0),
                350,
            ),
            event(EffectKind::HitFlash, "hit", (11, 10, 0), (11, 10, 0), 90),
            event(EffectKind::Sound, "gunshot", (9, 10, 0), (9, 10, 0), 1),
        ],
    );
    assert_eq!(sys.ingest(&batch), 3);
    let burst = &sys.active()[0];
    assert_eq!(
        burst.position(0),
        [10.0, 10.0, 0.0],
        "burst sits on its tile"
    );
    assert_eq!(burst.params.unwrap().render_class, CLASS_FLAME);
    assert!(
        burst.params.unwrap().emissive_strength > 1.0,
        "HDR core for bloom"
    );
    assert!(
        sys.active()[2].params.is_none(),
        "sound markers carry no visual emitter"
    );

    sys.advance(90);
    assert_eq!(
        sys.active().len(),
        1,
        "flash and sound expired, burst remains"
    );
    sys.advance(10_000);
    assert!(
        sys.is_empty(),
        "long after every duration: nothing left over"
    );
}

/// Gate 8.5 — ingest is idempotent and ordered: re-sending the
/// same message spawns nothing twice, and a stale (older-seq)
/// message is dropped rather than replayed late.
#[test]
fn gate_8_5_resend_and_stale_messages_do_not_respawn() {
    let mut sys = TransientSystem::new();
    assert_eq!(sys.ingest(&projectile(4)), 1);
    assert_eq!(
        sys.ingest(&projectile(4)),
        0,
        "re-send spawns nothing twice"
    );
    assert_eq!(sys.active().len(), 1);
    sys.advance(500);
    assert!(sys.is_empty());
    assert_eq!(
        sys.ingest(&projectile(3)),
        0,
        "older seq is dropped, not replayed"
    );
    assert!(
        sys.is_empty(),
        "a dropped stale message leaves nothing behind"
    );
    assert_eq!(sys.ingest(&projectile(5)), 1, "the next seq still spawns");
}

/// Parameter resolution: known families win over kind defaults,
/// unknown ids still produce a visible generic emitter.
#[test]
fn params_resolution() {
    assert_eq!(
        params_for(EffectKind::Projectile, "explosion")
            .unwrap()
            .render_class,
        CLASS_FLAME,
        "an explosion id is a flame burst whatever the kind says"
    );
    assert!(params_for(EffectKind::Sound, "explosion").is_none());
    let unknown = params_for(EffectKind::Other, "some_mod_effect").unwrap();
    assert_eq!(unknown.render_class, transient::params::CLASS_GENERIC);
}
