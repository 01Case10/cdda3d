//! Phase 1 gate tests that run purely in Rust (plan Part II §4).
//! Cross-language gates (1.2, 1.3, 1.11, 1.12 over a real socket) live in
//! cross_language.rs.

use proto::samples::*;
use proto::*;

// Tiny deterministic PRNG (xorshift64) — no external crates, fixed seed.
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
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn string(&mut self, max: usize) -> String {
        let len = self.below(max as u64 + 1) as usize;
        (0..len)
            .map(|_| (b'a' + self.below(26) as u8) as char)
            .collect()
    }
}

// ---------------------------------------------------------------- 1.1
#[test]
fn t1_1_round_trip_all_messages() {
    for msg in all_sample_messages() {
        let bytes = encode_message(&msg);
        let back = decode_message(&bytes).expect("decode own message");
        assert_eq!(back, msg);
    }
}

#[test]
fn t1_1_round_trip_randomized() {
    let mut rng = Rng(0x1234_5678_9abc_def1);
    for _ in 0..200 {
        // Random snapshot with sparse fields, connect data lives in defs.
        let mut block = sample_house_block();
        let mut fields = Vec::new();
        let mut field_index = vec![-1i32; TILES];
        for tile in 0..TILES {
            if rng.below(4) == 0 {
                let start = fields.len() as i32;
                field_index[tile] = start;
                for _ in 0..1 + rng.below(3) {
                    let mut f = FieldOnTileT::default();
                    f.tile_index = tile as u16;
                    f.type_ = 30 + rng.below(2) as u32;
                    f.intensity = rng.below(4) as u8;
                    fields.push(f);
                }
            }
        }
        block.fields = Some(fields);
        block.field_index = Some(field_index);
        let mut seen = vec![0u8; TILES];
        for s in seen.iter_mut() {
            *s = rng.below(256) as u8;
        }
        block.seen = Some(seen);
        let mut snap = sample_snapshot();
        snap.blocks = Some(vec![block]);
        snap.seq = rng.next();
        snap.epoch = rng.next();
        let msg = MessageT::Snapshot(Box::new(snap));
        assert_eq!(decode_message(&encode_message(&msg)).unwrap(), msg);

        // Random TileDefs with connect-group indices and fg/bg = -1 cases.
        let mut defs = sample_tiledefs();
        if let Some(list) = defs.defs.as_mut() {
            for d in list.iter_mut() {
                d.fg_ref = if rng.below(2) == 0 {
                    -1
                } else {
                    rng.below(500) as i32
                };
                d.bg_ref = if rng.below(2) == 0 {
                    -1
                } else {
                    rng.below(500) as i32
                };
                d.hot_flags = rng.next();
                d.name = Some(rng.string(12));
            }
        }
        let msg = MessageT::TileDefs(Box::new(defs));
        assert_eq!(decode_message(&encode_message(&msg)).unwrap(), msg);

        // Random entity: facing is only ever NONE/LEFT/RIGHT.
        let mut e = sample_entity();
        e.facing = match rng.below(3) {
            0 => Facing::NONE,
            1 => Facing::LEFT,
            _ => Facing::RIGHT,
        };
        e.overlays = Some((0..rng.below(4)).map(|_| rng.string(8)).collect());
        let mut snap = sample_snapshot();
        snap.entities = Some(vec![e]);
        let msg = MessageT::Snapshot(Box::new(snap));
        assert_eq!(decode_message(&encode_message(&msg)).unwrap(), msg);
    }
}

// ---------------------------------------------------------------- 1.3 (committed goldens)
// The golden files in tests/golden/ were written by the C++ side
// (tools/stub-server/cpp-roundtrip write). Regenerate them with that tool
// after any intentional schema/sample change. The live both-directions
// check is in cross_language.rs.
#[test]
fn t1_3_committed_cpp_goldens_decode_in_rust() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let snap = std::fs::read(dir.join("golden_snapshot.bin")).unwrap();
    assert_eq!(
        decode_message(&snap).unwrap(),
        MessageT::Snapshot(Box::new(sample_snapshot()))
    );
    let defs = std::fs::read(dir.join("golden_tiledefs.bin")).unwrap();
    assert_eq!(
        decode_message(&defs).unwrap(),
        MessageT::TileDefs(Box::new(sample_tiledefs()))
    );
    let hello = std::fs::read(dir.join("golden_hello.bin")).unwrap();
    assert_eq!(
        decode_message(&hello).unwrap(),
        MessageT::Hello(Box::new(sample_hello()))
    );
}

// ---------------------------------------------------------------- 1.4
#[test]
fn t1_4_verifier_rejects_corrupt_input() {
    let golden = encode_message(&MessageT::Hello(Box::new(sample_hello())));
    let original = MessageT::Hello(Box::new(sample_hello()));
    // Every truncation must be an error, never a panic — with one precisely
    // understood exception: the builder appends up to 3 trailing alignment
    // padding bytes that nothing references, so truncating only those still
    // yields a complete buffer. Such a truncation must decode to *exactly*
    // the original message, never to different data.
    for len in 0..golden.len() {
        match decode_message(&golden[..len]) {
            Err(_) => {}
            Ok(m) => assert_eq!(m, original, "truncation to {len} decoded to different data"),
        }
    }
    // Truncations that cut into the message body proper (all but the last
    // few padding bytes) are hard errors.
    for len in 0..golden.len() - 4 {
        assert!(
            decode_message(&golden[..len]).is_err(),
            "truncation to {len} decoded"
        );
    }
    // Bit flips in the structural prefix (root offset, file identifier,
    // vtable) must be errors — or, for flips in vtable entries of *absent*
    // fields, decode to exactly the original message. Never different data,
    // never a panic. (decode_message additionally checks the CDDA file
    // identifier and rejects empty envelopes, which the raw flatbuffers
    // verifier alone does not — see lib.rs.)
    for pos in 0..16 {
        for bit in 0..8 {
            let mut bad = golden.clone();
            bad[pos] ^= 1 << bit;
            match decode_message(&bad) {
                Err(_) => {}
                Ok(m) => assert_eq!(
                    m, original,
                    "flip at byte {pos} bit {bit} changed the message"
                ),
            }
        }
    }
    // Flips in the file identifier itself are always hard errors.
    for pos in 4..8 {
        for bit in 0..8 {
            let mut bad = golden.clone();
            bad[pos] ^= 1 << bit;
            assert!(
                decode_message(&bad).is_err(),
                "identifier flip at byte {pos} bit {bit} decoded"
            );
        }
    }
    // Random garbage must be an error (file identifier mismatch).
    let mut rng = Rng(42);
    for _ in 0..1000 {
        let len = rng.below(64) as usize;
        let buf: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
        assert!(decode_message(&buf).is_err());
    }
}

#[test]
fn t1_4_and_1_5_no_panic_on_any_flip() {
    // Fuzz-shaped sweep (gate 1.5's local stand-in; cargo-fuzz itself is not
    // run in this environment): every single-byte corruption of a snapshot
    // golden, plus random multi-byte corruptions, must never panic. Payload
    // *content* bytes (e.g. inside a string) may still verify with altered
    // data — FlatBuffers has no checksum — so the assertion here is
    // "no panic", and structural rejection is asserted in t1_4.
    let golden = encode_message(&MessageT::Snapshot(Box::new(sample_snapshot())));
    for pos in 0..golden.len() {
        let mut bad = golden.clone();
        bad[pos] ^= 0xff;
        let _ = decode_message(&bad); // must not panic
    }
    let mut rng = Rng(7);
    for _ in 0..20_000 {
        let mut bad = golden.clone();
        for _ in 0..1 + rng.below(8) {
            let pos = rng.below(bad.len() as u64) as usize;
            bad[pos] = rng.next() as u8;
        }
        let _ = decode_message(&bad); // must not panic
                                      // Also: pure random buffers through both decode and framing.
        let len = rng.below(256) as usize;
        let buf: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
        let _ = decode_message(&buf);
        let mut dec = FrameDecoder::new();
        dec.push(&buf);
        let _ = dec.next_frame();
    }
}

// ---------------------------------------------------------------- 1.6
#[test]
fn t1_6_framing_split_and_coalesce() {
    let msgs: Vec<Vec<u8>> = all_sample_messages().iter().map(encode_message).collect();
    let mut wire = Vec::new();
    for m in &msgs {
        wire.extend_from_slice(&FrameEncoder::encode(m));
    }
    // One byte at a time.
    let mut dec = FrameDecoder::new();
    let mut out = Vec::new();
    for b in &wire {
        dec.push(&[*b]);
        while let Some(f) = dec.next_frame().unwrap() {
            out.push(f);
        }
    }
    assert_eq!(out, msgs);
    // All coalesced in one read.
    let mut dec = FrameDecoder::new();
    dec.push(&wire);
    let mut out = Vec::new();
    while let Some(f) = dec.next_frame().unwrap() {
        out.push(f);
    }
    assert_eq!(out, msgs);
    // Random chunk boundaries.
    let mut rng = Rng(99);
    let mut dec = FrameDecoder::new();
    let mut out = Vec::new();
    let mut i = 0;
    while i < wire.len() {
        let n = 1 + rng.below(37) as usize;
        let end = (i + n).min(wire.len());
        dec.push(&wire[i..end]);
        i = end;
        while let Some(f) = dec.next_frame().unwrap() {
            out.push(f);
        }
    }
    assert_eq!(out, msgs);
}

// ---------------------------------------------------------------- 1.7
#[test]
fn t1_7_oversize_length_rejected() {
    let mut dec = FrameDecoder::new();
    dec.push(&u32::MAX.to_le_bytes());
    match dec.next_frame() {
        Err(ProtoError::OversizeFrame(n)) => assert_eq!(n, u32::MAX),
        other => panic!("expected OversizeFrame, got {other:?}"),
    }
    // One-over-the-cap is also rejected, from the prefix alone.
    let mut dec = FrameDecoder::new();
    dec.push(&(MAX_FRAME_LEN + 1).to_le_bytes());
    assert!(matches!(
        dec.next_frame(),
        Err(ProtoError::OversizeFrame(_))
    ));
    // Exactly at the cap is accepted as a length (waits for payload).
    let mut dec = FrameDecoder::new();
    dec.push(&MAX_FRAME_LEN.to_le_bytes());
    assert_eq!(dec.next_frame().unwrap(), None);
}

// ---------------------------------------------------------------- 1.8
#[test]
fn t1_8_handshake_versions() {
    assert!(check_handshake(PROTOCOL_VERSION).is_ok());
    for bad in [0, PROTOCOL_VERSION + 1, u32::MAX] {
        match check_handshake(bad) {
            Err(ProtoError::VersionMismatch { ours, theirs }) => {
                assert_eq!(ours, PROTOCOL_VERSION);
                assert_eq!(theirs, bad);
            }
            other => panic!("expected VersionMismatch for {bad}, got {other:?}"),
        }
    }
    // Missing hello: empty input is a decode error, not a hang or panic.
    assert!(decode_message(&[]).is_err());
}

// ---------------------------------------------------------------- 1.9
#[test]
fn t1_9_forward_compatibility_unknown_field() {
    use proto::generated_v2::cdda::ipc as v2;
    // Build a Hello with the v2-only field set.
    let mut hello = v2::HelloT::default();
    hello.protocol_version = PROTOCOL_VERSION;
    hello.game_version = Some("future-game".into());
    hello.tileset_id = Some("UltiCa".into());
    hello.tile_pixel_size = 32;
    hello.z_min = -10;
    hello.z_max = 10;
    hello.future_field = Some("brand new data the v1 reader has never seen".into());
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let mut env = v2::EnvelopeT::default();
    env.message = v2::MessageT::Hello(Box::new(hello));
    let off = env.pack(&mut fbb);
    v2::finish_envelope_buffer(&mut fbb, off);
    let bytes = fbb.finished_data().to_vec();
    // The v1 reader must decode it and see the fields it knows, unchanged.
    match decode_message(&bytes).expect("v1 reader must ignore unknown field") {
        MessageT::Hello(h) => {
            assert_eq!(h.protocol_version, PROTOCOL_VERSION);
            assert_eq!(h.game_version.as_deref(), Some("future-game"));
            assert_eq!(h.tile_pixel_size, 32);
            assert_eq!(h.z_min, -10);
            assert_eq!(h.z_max, 10);
        }
        other => panic!("expected Hello, got {other:?}"),
    }
}

// ---------------------------------------------------------------- 1.10
#[test]
fn t1_10_heartbeat_timeout() {
    let mut hb = HeartbeatTracker::new(1_000);
    assert!(!hb.is_timed_out(1_000));
    assert!(!hb.is_timed_out(4_000)); // exactly at the 3 s timeout: not yet
    assert!(hb.is_timed_out(4_001)); // past it: dead
    hb.note_seen(5_000); // any message revives
    assert!(!hb.is_timed_out(8_000));
    assert!(hb.is_timed_out(8_001));
    // Ping cadence: every interval_ms.
    let mut hb = HeartbeatTracker::new(0);
    assert!(!hb.should_ping(999));
    assert!(hb.should_ping(1_000));
    hb.note_ping_sent(1_000);
    assert!(!hb.should_ping(1_999));
    assert!(hb.should_ping(2_000));
}

// ---------------------------------------------------------------- 1.13
#[test]
fn t1_13_shared_memory_header_layout() {
    assert_eq!(std::mem::size_of::<UiShmHeader>(), UiShmHeader::SIZE);
    assert_eq!(UiShmHeader::SIZE, 48);
    assert_eq!(std::mem::offset_of!(UiShmHeader, magic), 0);
    assert_eq!(std::mem::offset_of!(UiShmHeader, width), 4);
    assert_eq!(std::mem::offset_of!(UiShmHeader, stride), 12);
    assert_eq!(std::mem::offset_of!(UiShmHeader, seq), 16);
    assert_eq!(std::mem::offset_of!(UiShmHeader, dirty_x), 24);
    assert_eq!(std::mem::offset_of!(UiShmHeader, dirty_h), 36);
    assert_eq!(std::mem::offset_of!(UiShmHeader, buffer_index), 40);

    let mut hdr = UiShmHeader::new(1600, 800, 6400);
    hdr.seq = 123;
    hdr.dirty_x = 10;
    hdr.dirty_y = 20;
    hdr.dirty_w = 300;
    hdr.dirty_h = 200;
    hdr.buffer_index = 1;
    let bytes = hdr.to_bytes();
    let back = UiShmHeader::from_bytes(&bytes).expect("parse own header");
    assert_eq!(back, hdr);
    // Wrong magic is rejected; a truncated buffer is rejected.
    let mut bad = bytes;
    bad[0] ^= 0xff;
    assert_eq!(UiShmHeader::from_bytes(&bad), None);
    assert_eq!(UiShmHeader::from_bytes(&bytes[..20]), None);
}

// ---------------------------------------------------------------- 1.14
#[test]
fn t1_14_seq_epoch_semantics() {
    let mut tr = SeqTracker::new();
    // Delta before any snapshot: needs a snapshot.
    assert_eq!(tr.observe_delta(1, 1), DeltaDecision::NeedsSnapshot);
    // Snapshot (seq 10, epoch 1) becomes the base.
    assert!(tr.accept_snapshot(10, 1));
    // In-order delta applies.
    assert_eq!(tr.observe_delta(11, 1), DeltaDecision::Apply);
    // Duplicate: discarded, applied zero times.
    assert_eq!(tr.observe_delta(11, 1), DeltaDecision::Discard);
    // Older seq: discarded.
    assert_eq!(tr.observe_delta(5, 1), DeltaDecision::Discard);
    // Skipped seq: needs a fresh snapshot, and state does not advance.
    assert_eq!(tr.observe_delta(13, 1), DeltaDecision::NeedsSnapshot);
    assert_eq!(tr.last_seq(), 11);
    // Old epoch: discarded.
    assert!(tr.accept_snapshot(20, 2));
    assert_eq!(tr.observe_delta(21, 1), DeltaDecision::Discard);
    // Delta for an epoch whose snapshot we never saw: needs snapshot.
    assert_eq!(tr.observe_delta(21, 3), DeltaDecision::NeedsSnapshot);
    // Stale snapshot does not roll the tracker back.
    assert!(!tr.accept_snapshot(1, 1));
    assert_eq!(tr.epoch(), Some(2));
}
