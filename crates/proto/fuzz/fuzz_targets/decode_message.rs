//! Gate 1.5, full form: the wire decoder must never panic on
//! arbitrary bytes — every malformed input is a `Result::Err`,
//! never a crash. (The proto suite's fuzz-shaped sweep covers
//! single-byte flips + 20k mutations of valid frames; this
//! target is the real libFuzzer run CI executes.)
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Errors are the expected outcome for almost all inputs;
    // the assertion is implicit — reaching the end at all.
    let _ = proto::decode_message(data);
});
