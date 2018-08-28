//! The portsmith line-based trace format (v1), Rust side.
//!
//! Mirrors the Go `internal/trace` package so both tools interoperate. See
//! `docs/FORMAT.md` for the authoritative specification.

use std::fmt;
use std::io::{self, BufRead};

/// Recommended first line of a trace file.
pub const MAGIC: &str = "#portsmith-trace v1";

/// Which side of a session produced a record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// client -> server
    Request,
    /// server -> client
    Response,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::Request => write!(f, ">"),
            Direction::Response => write!(f, "<"),
        }
    }
}

/// A single captured message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub ts_nanos: i64,
    pub dir: Direction,
    pub session: String,
    pub proto: String,
    pub payload: Vec<u8>,
}

impl Record {
    /// Encode as a single trace line (no trailing newline).
    pub fn encode(&self) -> String {
        format!(
            "V1 {} {} {} {} {} {}",
            self.ts_nanos,
            self.dir,
            self.session,
            self.proto,
            self.payload.len(),
            base64_encode(&self.payload),
        )
    }
}

/// Error while parsing a trace line.
#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub msg: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "trace: line {}: {}", self.line, self.msg)
    }
}

impl std::error::Error for ParseError {}

/// Parse a single record line.
pub fn decode_line(line: &str, line_no: usize) -> Result<Record, ParseError> {
    let err = |m: String| ParseError {
        line: line_no,
        msg: m,
    };
    let fields: Vec<&str> = line.splitn(7, ' ').collect();
    if fields.len() != 7 {
        return Err(err(format!("expected 7 fields, got {}", fields.len())));
    }
    if fields[0] != "V1" {
        return Err(err(format!("unknown record version {:?}", fields[0])));
    }
    let ts_nanos: i64 = fields[1]
        .parse()
        .map_err(|e| err(format!("invalid timestamp: {e}")))?;
    let dir = match fields[2] {
        ">" => Direction::Request,
        "<" => Direction::Response,
        other => return Err(err(format!("invalid direction {other:?}"))),
    };
    let declared_len: usize = fields[5]
        .parse()
        .map_err(|e| err(format!("invalid length: {e}")))?;
    let payload =
        base64_decode(fields[6]).map_err(|e| err(format!("invalid base64 payload: {e}")))?;
    if payload.len() != declared_len {
        return Err(err(format!(
            "length mismatch: declared {declared_len}, decoded {}",
            payload.len()
        )));
    }
    Ok(Record {
        ts_nanos,
        dir,
        session: fields[3].to_string(),
        proto: fields[4].to_string(),
        payload,
    })
}

/// Read all records from a buffered reader, skipping comments and blank lines.
pub fn read_all<R: BufRead>(reader: R) -> io::Result<Vec<Record>> {
    let mut out = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        let line = line.trim_end_matches('\r');
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match decode_line(line, i + 1) {
            Ok(r) => out.push(r),
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string())),
        }
    }
