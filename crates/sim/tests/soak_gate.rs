//! S8 gate: T12 — the seeded random-input soak (plan §6
//! T12, fork-harness 3.9 parity: 1,100 turns x 10 seeds).
//!
//! A seeded xorshift generator draws from the full key
//! vocabulary — 8-way movement + the surviving WASD aliases,
//! wait, open/close/bash arming + direction keys, pickup,
//! drop, the three UI opens, Esc / q (so runs keep returning
//! to MapInput instead of wedging in UiInput), and the S8
//! z-move keys — and drives a real World in-process. Scenes
//! alternate by seed: destructible_street (bashables, items,
//! a vehicle store car) and zombie_yard (street_block + a
//! live zombie: chase / attack / corpse paths).
//!
//! After EVERY input flush: the stream-fed Mirror must
//! equal a fresh Snapshot of the world — asserted both by
//! structural equality (`Mirror::assert_matches`, which
//! names the first divergence) and by an FNV-1a hash
//! compare over a canonical encoding of the mirror state
//! (the plan's "hash compare" form). Seq must advance by
//! exactly 1 per seq-bearing message and epoch must never
//! regress (proto::sync's discipline, checked stream-side).
//!
//! Step cost: mean / p95 per-input wall time are measured
//! and printed; the only assertions are generous named
//! bounds (mean < 250 ms, p95 < 1 s on this VM) whose job
//! is to catch a pathological blow-up (e.g. a quadratic
//! differ), not to spec the product — this VM's absolute
//! numbers are not a product spec.
//!
//! Failing-first: pre-S8 there was no soak driver at all
//! (the file t12_soak.toml carried `status = "s8"` saying
//! exactly that), and the z-move keys in the vocabulary had
//! no rule behind them.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use proto::{InputKind, InputT, MessageT};
use sim::mirror::Mirror;
use sim::{DefRegistry, Scene, World};

const TURNS: usize = 1_100;
const SEEDS: std::ops::RangeInclusive<i64> = 1..=10;

/// The full key vocabulary (see the module docs), as wire
/// key codes. Esc (ui-share's KEY_ESCAPE) appears twice: UI
/// states must be exited often enough that the run spends
/// most turns in MapInput.
const KEYS: &[i32] = &[
    'h' as i32, 'j' as i32, 'k' as i32, 'l' as i32, 'y' as i32, 'u' as i32, 'b' as i32,
    'n' as i32, 'w' as i32, 'a' as i32, // movement
    '.' as i32, ' ' as i32, // wait
    'o' as i32, 'c' as i32, 's' as i32, // arm open / close / bash
    'g' as i32, 'd' as i32, // pickup / drop
    'i' as i32, '&' as i32, ';' as i32, // UI opens
    ui_share::input::KEY_ESCAPE, ui_share::input::KEY_ESCAPE, 'q' as i32, // UI exit / no-op
    '>' as i32, '<' as i32, // z-move (S8)
];

/// xorshift64, the fork harness mock's PRNG family (the
/// same shifts as the sim's own RNG). Seed 0 maps to a
/// fixed nonzero constant.
struct XorShift(u64);

impl XorShift {
    fn new(seed: i64) -> Self {
        let s = seed as u64;
        XorShift(if s == 0 { 0x9E37_79B9_7F4A_7C15 } else { s })
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scenes_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scenes")
}

fn key_input(code: i32) -> InputT {
    let mut input = InputT::default();
    input.kind = InputKind::Key;
    input.key_code = code;
    input.pressed = true;
    input
}

/// FNV-1a over a canonical encoding of the mirror: tiles /
/// entities / vehicles sorted by key, Debug-formatted (the
/// encoding is internal to this gate — determinism is all
/// it needs).
fn mirror_hash(m: &Mirror) -> u64 {
    let mut tiles: Vec<_> = m.tiles.iter().collect();
    tiles.sort_by_key(|(pos, _)| **pos);
    let mut entities: Vec<_> = m.entities.iter().collect();
    entities.sort_by_key(|(id, _)| **id);
    let mut vehicles: Vec<_> = m.vehicles.iter().collect();
    vehicles.sort_by_key(|(id, _)| **id);
    let canon = format!("{tiles:?}|{entities:?}|{vehicles:?}");
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in canon.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The (seq, epoch) a message carries; Effects carries a
/// seq but no epoch field, hence the Option.
fn stream_seq(msg: &MessageT) -> Option<(u64, Option<u64>)> {
    match msg {
        MessageT::Snapshot(s) => Some((s.seq, Some(s.epoch))),
        MessageT::Delta(d) => Some((d.seq, Some(d.epoch))),
        MessageT::LightUpdate(lu) => Some((lu.seq, Some(lu.epoch))),
        MessageT::EnvironmentUpdate(e) => Some((e.seq, Some(e.epoch))),
        MessageT::Effects(fx) => Some((fx.seq, None)),
        _ => None,
    }
}

#[test]
fn t12_soak_1100_turns_x_10_seeds() {
    let reg = DefRegistry::load(&defs_path()).expect("registry loads");
    let mut all_step_times: Vec<Duration> = Vec::new();
    let mut total_shifts = 0usize;

    for seed in SEEDS {
        let scene_name = if seed % 2 == 0 {
            "destructible_street"
        } else {
            "zombie_yard"
        };
        let mut scene =
            Scene::load_file(&scenes_dir().join(format!("{scene_name}.toml"))).expect("scene");
        scene.seed = seed * 7_919;
        let mut world = World::from_scene(&reg, &scene).expect("world builds");
        let snap = world.snapshot(&reg);
        let mut mirror = Mirror::from_snapshot(&snap);
        let mut last_seq = snap.seq;
        let mut last_epoch = snap.epoch;
        let mut gen = XorShift::new(seed * 1_000_003 + 17);
        let mut shifts = 0usize;

        for turn in 0..TURNS {
            let key = KEYS[gen.below(KEYS.len())];
            let t0 = Instant::now();
            let msgs = world.apply_input(&reg, &key_input(key));
            let key = char::from_u32(key as u32).unwrap_or('�');
            all_step_times.push(t0.elapsed());
            for msg in &msgs {
                if let Some((seq, epoch)) = stream_seq(msg) {
                    assert_eq!(
                        seq,
                        last_seq + 1,
                        "seed {seed} turn {turn}: seq must advance by exactly 1 \
                         (got {seq} after {last_seq})"
                    );
                    if let Some(epoch) = epoch {
                        assert!(
                            epoch >= last_epoch,
                            "seed {seed} turn {turn}: epoch regressed {last_epoch} -> {epoch}"
                        );
                        last_epoch = epoch;
                    }
                    last_seq = seq;
                }
                if matches!(msg, MessageT::Snapshot(_)) {
                    shifts += 1;
                }
                mirror
                    .apply(msg)
                    .unwrap_or_else(|e| panic!("seed {seed} turn {turn}: mirror: {e}"));
            }
            // The invariant, after every flush: mirror ==
            // fresh Snapshot, structurally AND by hash.
            let fresh = world.build_snapshot(&reg, 0, 0);
            mirror.assert_matches(&fresh).unwrap_or_else(|e| {
                panic!("seed {seed} turn {turn} (key {key:?}): mirror invariant: {e}")
            });
            let fresh_mirror = Mirror::from_snapshot(&fresh);
            assert_eq!(
                mirror_hash(&mirror),
                mirror_hash(&fresh_mirror),
                "seed {seed} turn {turn} (key {key:?}): mirror hash != fresh snapshot hash"
            );
        }
        println!(
            "soak seed {seed} ({scene_name}): {TURNS} turns ok, \
             {shifts} map shifts, final seq {last_seq}, epoch {last_epoch}"
        );
        total_shifts += shifts;
    }

    all_step_times.sort();
    let n = all_step_times.len();
    let mean = all_step_times.iter().sum::<Duration>() / n as u32;
    let p95 = all_step_times[(n * 95) / 100];
    println!(
        "soak: {} inputs over {} seeds; step time mean {mean:?}, p95 {p95:?}; \
         total map shifts {total_shifts}",
        n,
        SEEDS.count()
    );
    // Generous named bounds (module docs): a blow-up trip,
    // not a product spec.
    assert!(
        mean < Duration::from_millis(250),
        "soak step mean {mean:?} exceeds the generous 250 ms bound"
    );
    assert!(
        p95 < Duration::from_secs(1),
        "soak step p95 {p95:?} exceeds the generous 1 s bound"
    );
}
