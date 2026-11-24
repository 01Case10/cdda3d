//! proto-client: connects to the stub server (or the real fork server) over
//! a Unix domain socket, handshakes, logs TileDefs + the Snapshot, sends an
//! Input event and verifies the stub echoes it back, pings, then shuts the
//! peer down cleanly.
//!
//! Usage: proto-client <socket-path>

use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
#[cfg(windows)]
use uds_windows::UnixStream;
use std::time::Duration;

use proto::samples::{sample_hello_ack, sample_input};
use proto::{
    check_handshake, decode_message, encode_message, FrameDecoder, FrameEncoder, MessageT,
};

fn send(stream: &mut UnixStream, msg: &MessageT) -> std::io::Result<()> {
    let framed = FrameEncoder::encode(&encode_message(msg));
    stream.write_all(&framed)
}

fn recv(stream: &mut UnixStream, dec: &mut FrameDecoder) -> proto::Result<MessageT> {
    loop {
        if let Some(frame) = dec.next_frame()? {
            return decode_message(&frame);
        }
        let mut buf = [0u8; 8192];
        let n = stream.read(&mut buf)?;
        if n == 0 {
            return Err(proto::ProtoError::Malformed(
                "peer closed connection".into(),
            ));
        }
        dec.push(&buf[..n]);
    }
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: proto-client <socket-path>");
        std::process::exit(2);
    });
    let mut stream = UnixStream::connect(&path).expect("connect to stub server");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut dec = FrameDecoder::new();

    // 1. Handshake: expect Hello, check version, answer HelloAck.
    let hello = recv(&mut stream, &mut dec).expect("receive Hello");
    match &hello {
        MessageT::Hello(h) => {
            check_handshake(h.protocol_version).expect("handshake version check");
            println!(
                "HELLO game={:?} tileset={:?} tile_px={} z=[{}..{}] proto={}",
                h.game_version,
                h.tileset_id,
                h.tile_pixel_size,
                h.z_min,
                h.z_max,
                h.protocol_version
            );
        }
        other => panic!("expected Hello, got {other:?}"),
    }
    send(
        &mut stream,
        &MessageT::HelloAck(Box::new(sample_hello_ack())),
    )
    .expect("send HelloAck");
    println!("HELLOACK sent");

    // 2. TileDefs.
    match recv(&mut stream, &mut dec).expect("receive TileDefs") {
        MessageT::TileDefs(td) => {
            let defs = td.defs.unwrap_or_default();
            println!(
                "TILEDEFS {} defs, connect_groups={:?}",
                defs.len(),
                td.connect_groups.unwrap_or_default()
            );
            for d in defs.iter().take(4) {
                println!(
                    "  def {} {:?} fg_ref={} bg_ref={} open_result={:?}",
                    d.id, d.string_id, d.fg_ref, d.bg_ref, d.open_result
                );
            }
        }
        other => panic!("expected TileDefs, got {other:?}"),
    }

    // 3. Snapshot: pretty-log the house.
    match recv(&mut stream, &mut dec).expect("receive Snapshot") {
        MessageT::Snapshot(s) => {
            println!(
                "SNAPSHOT seq={} epoch={} blocks={}",
                s.seq,
                s.epoch,
                s.blocks.as_ref().map_or(0, |b| b.len())
            );
            for b in s.blocks.unwrap_or_default() {
                let terrain = b.terrain.unwrap_or_default();
                for y in 0..12 {
                    let mut row = String::new();
                    for x in 0..12 {
                        row.push(match terrain[y * 12 + x] {
                            1 => '.',
                            2 => '#',
                            3 => '+',
                            4 => '=',
                            _ => '?',
                        });
                    }
                    println!("  {row}");
                }
                for f in b.fields.unwrap_or_default() {
                    println!(
                        "  field tile={} type={} intensity={}",
                        f.tile_index, f.type_, f.intensity
                    );
                }
                let counts = b.item_count.unwrap_or_default();
                let tops = b.top_item.unwrap_or_default();
                for i in 0..counts.len() {
                    if counts[i] > 0 {
                        println!("  items tile={i} top={} count={}", tops[i], counts[i]);
                    }
                }
            }
            for e in s.entities.unwrap_or_default() {
                println!(
                    "  entity {} {:?} pos={:?} facing={:?} overlays={:?}",
                    e.id, e.sprite_id, e.pos, e.facing, e.overlays
                );
            }
        }
        other => panic!("expected Snapshot, got {other:?}"),
    }

    // 4. Input round-trip: the stub echoes Input back verbatim.
    let input = sample_input();
    send(&mut stream, &MessageT::Input(Box::new(input.clone()))).expect("send Input");
    match recv(&mut stream, &mut dec).expect("input echo") {
        MessageT::Input(echo) => {
            assert_eq!(*echo, input, "echoed input differs");
            println!(
                "INPUT round-trip ok: key_code={} pressed={}",
                echo.key_code, echo.pressed
            );
        }
        other => panic!("expected Input echo, got {other:?}"),
    }

    // 5. Ping/Pong.
    let mut ping = proto::PingT::default();
    ping.nonce = 77;
    ping.sent_ms = 1000;
    send(&mut stream, &MessageT::Ping(Box::new(ping))).expect("send Ping");
    match recv(&mut stream, &mut dec).expect("receive Pong") {
        MessageT::Pong(p) => {
            assert_eq!(p.nonce, 77);
            println!("PONG nonce={}", p.nonce);
        }
        other => panic!("expected Pong, got {other:?}"),
    }

    // 6. Clean shutdown.
    let mut shutdown = proto::ShutdownT::default();
    shutdown.reason = Some("client done".into());
    send(&mut stream, &MessageT::Shutdown(Box::new(shutdown))).expect("send Shutdown");
    println!("SHUTDOWN sent; client done");
}
