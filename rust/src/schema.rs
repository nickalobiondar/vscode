//! Schema inference: discover protocol structure from a set of trace records.
//!
//! The inference is deliberately heuristic and dependency-free. For each
//! (proto, direction) group it computes:
//!   * whether payloads look textual or binary
//!   * the dominant line/record terminator (CRLF, LF, none)
//!   * the leading token distribution (e.g. HTTP methods, Redis verbs)
//!   * byte-length statistics
//!
//! The result is a human-readable report and a stable, machine-parseable
//! summary suitable for downstream tooling.

use std::collections::BTreeMap;

use crate::trace::{Direction, Record};

/// Textual classification of a payload group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Text,
    Binary,
}

impl std::fmt::Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Encoding::Text => write!(f, "text"),
            Encoding::Binary => write!(f, "binary"),
        }
    }
}

/// Inferred schema for one (proto, direction) group.
#[derive(Debug, Clone)]
pub struct GroupSchema {
    pub proto: String,
    pub dir: Direction,
    pub samples: usize,
    pub encoding: Encoding,
    pub terminator: String,
    pub min_len: usize,
    pub max_len: usize,
    pub mean_len: f64,
    /// Leading token -> count, sorted by count descending in `top_tokens`.
    pub top_tokens: Vec<(String, usize)>,
}

/// Full inferred schema across all groups.
#[derive(Debug, Clone, Default)]
pub struct Schema {
    pub groups: Vec<GroupSchema>,
}

/// Infer a schema from records.
pub fn infer(records: &[Record]) -> Schema {
    // Group by (proto, dir).
    let mut groups: BTreeMap<(String, u8), Vec<&Record>> = BTreeMap::new();
    for r in records {
        let key = (r.proto.clone(), dir_key(r.dir));
        groups.entry(key).or_default().push(r);
    }

    let mut schema = Schema::default();
    for ((proto, dk), recs) in groups {
        schema.groups.push(infer_group(&proto, key_dir(dk), &recs));
    }
    schema
}

fn dir_key(d: Direction) -> u8 {
    match d {
        Direction::Request => b'>',
        Direction::Response => b'<',
    }
}

fn key_dir(k: u8) -> Direction {
    if k == b'<' {
        Direction::Response
    } else {
        Direction::Request
    }
}

fn infer_group(proto: &str, dir: Direction, recs: &[&Record]) -> GroupSchema {
    let samples = recs.len();
    let mut printable = 0usize;
    let mut total_bytes = 0usize;
    let mut min_len = usize::MAX;
    let mut max_len = 0usize;
    let mut crlf = 0usize;
    let mut lf = 0usize;
    let mut token_counts: BTreeMap<String, usize> = BTreeMap::new();

    for r in recs {
        let p = &r.payload;
        total_bytes += p.len();
        min_len = min_len.min(p.len());
        max_len = max_len.max(p.len());

        let printable_ct = p.iter().filter(|&&b| is_printable(b)).count();
        if p.is_empty() || printable_ct * 100 >= p.len() * 90 {
            printable += 1;
        }

        if p.ends_with(b"\r\n") {
            crlf += 1;
        } else if p.ends_with(b"\n") {
            lf += 1;
        }

        if let Some(tok) = leading_token(p) {
            *token_counts.entry(tok).or_insert(0) += 1;
        }
    }

    if min_len == usize::MAX {
        min_len = 0;
    }
    let encoding = if samples > 0 && printable * 100 >= samples * 80 {
        Encoding::Text
    } else {
        Encoding::Binary
    };
    let terminator = if crlf >= lf && crlf * 2 >= samples {
        "CRLF".to_string()
    } else if lf * 2 >= samples {
        "LF".to_string()
    } else {
        "none".to_string()
    };
    let mean_len = if samples > 0 {
        total_bytes as f64 / samples as f64
    } else {
        0.0
    };

    let mut top_tokens: Vec<(String, usize)> = token_counts.into_iter().collect();
    top_tokens.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    top_tokens.truncate(8);

    GroupSchema {
        proto: proto.to_string(),
        dir,
        samples,
        encoding,
        terminator,
        min_len,
        max_len,
        mean_len,
        top_tokens,
    }
}

fn is_printable(b: u8) -> bool {
    b == b'\t' || b == b'\n' || b == b'\r' || (0x20..=0x7e).contains(&b)
}

/// Extract the first whitespace-delimited token if the payload is textual.
fn leading_token(payload: &[u8]) -> Option<String> {
    let end = payload
        .iter()
        .position(|&b| b == b' ' || b == b'\r' || b == b'\n' || b == b'\t')
        .unwrap_or(payload.len());
    if end == 0 {
        return None;
    }
    let slice = &payload[..end];
    if slice.iter().all(|&b| is_printable(b) && b != b' ') {
        // Cap token length so binary noise never becomes a "token".
        if slice.len() <= 32 {
            return Some(String::from_utf8_lossy(slice).into_owned());
        }
    }
    None
}

impl Schema {
    /// Render a human-readable multi-line report.
