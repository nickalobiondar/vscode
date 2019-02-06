use portsmith_replay::schema::{infer, Encoding};
use portsmith_replay::trace::{base64_decode, base64_encode, decode_line, Direction, Record};

#[test]
fn base64_round_trips() {
    let cases: &[&[u8]] = &[
        b"",
        b"f",
        b"fo",
