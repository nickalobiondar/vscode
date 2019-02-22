use portsmith_replay::schema::{infer, Encoding};
use portsmith_replay::trace::{base64_decode, base64_encode, decode_line, Direction, Record};

#[test]
fn base64_round_trips() {
    let cases: &[&[u8]] = &[
        b"",
        b"f",
        b"fo",
        b"foo",
        b"foob",
        b"fooba",
        b"foobar",
        &[0, 255, 16, 200],
    ];
    for c in cases {
        let enc = base64_encode(c);
        let dec = base64_decode(&enc).expect("decode");
        assert_eq!(&dec, c, "round trip failed for {c:?}");
