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
    }
    // Known-answer vector.
    assert_eq!(base64_encode(b"Man"), "TWFu");
    assert_eq!(base64_decode("TWFu").unwrap(), b"Man");
}

#[test]
fn decode_line_matches_encode() {
    let r = Record {
        ts_nanos: 42,
        dir: Direction::Response,
        session: "s1".into(),
        proto: "redis".into(),
        payload: b"+PONG\r\n".to_vec(),
    };
    let line = r.encode();
    let back = decode_line(&line, 1).expect("decode");
    assert_eq!(back, r);
}

#[test]
fn decode_line_rejects_bad_input() {
    assert!(decode_line("V1 100 > s p", 1).is_err()); // too few fields
    assert!(decode_line("V9 100 > s p 4 dGVzdA==", 1).is_err()); // version
    assert!(decode_line("V1 100 ? s p 4 dGVzdA==", 1).is_err()); // direction
    assert!(decode_line("V1 100 > s p 9 dGVzdA==", 1).is_err()); // len mismatch
}

#[test]
fn infer_detects_text_and_tokens() {
    let recs = vec![
        Record {
            ts_nanos: 1,
            dir: Direction::Request,
            session: "a".into(),
            proto: "redis".into(),
            payload: b"GET key1\r\n".to_vec(),
        },
        Record {
            ts_nanos: 2,
            dir: Direction::Request,
            session: "a".into(),
            proto: "redis".into(),
            payload: b"GET key2\r\n".to_vec(),
        },
        Record {
            ts_nanos: 3,
            dir: Direction::Request,
            session: "a".into(),
            proto: "redis".into(),
            payload: b"SET key3 v\r\n".to_vec(),
        },
    ];
    let schema = infer(&recs);
    assert_eq!(schema.groups.len(), 1);
    let g = &schema.groups[0];
    assert_eq!(g.encoding, Encoding::Text);
    assert_eq!(g.terminator, "CRLF");
    assert_eq!(g.samples, 3);
    // GET appears twice and should be the top token.
    assert_eq!(g.top_tokens[0].0, "GET");
    assert_eq!(g.top_tokens[0].1, 2);
}

#[test]
fn infer_detects_binary() {
    let recs = vec![Record {
        ts_nanos: 1,
        dir: Direction::Response,
        session: "a".into(),
        proto: "raw".into(),
        payload: vec![0x00, 0x01, 0x02, 0xff, 0xfe, 0x80, 0x90],
    }];
    let schema = infer(&recs);
    assert_eq!(schema.groups[0].encoding, Encoding::Binary);
}

// draft note 43
