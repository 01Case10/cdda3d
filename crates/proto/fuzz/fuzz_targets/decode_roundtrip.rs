//! Companion target: any input that DOES decode must survive
//! the round trip — re-encoding the decoded message and
//! decoding again must succeed and encode identically (the
//! sync invariant's wire half: a message the engine accepted
//! cannot change meaning by being relayed).
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(msg) = proto::decode_message(data) {
        let bytes = proto::encode_message(&msg);
        let msg2 = proto::decode_message(&bytes)
            .expect("a message we encoded must decode");
        let bytes2 = proto::encode_message(&msg2);
        assert_eq!(bytes, bytes2, "round trip must be byte-stable");
    }
});
