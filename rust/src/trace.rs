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
    Ok(out)
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard base64 encoder (with padding), std-only.
pub fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64[(n >> 18 & 63) as usize] as char);
        out.push(B64[(n >> 12 & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(B64[(n >> 6 & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(B64[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Standard base64 decoder (with padding), std-only.
pub fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    fn val(c: u8) -> Result<u32, String> {
        match c {
            b'A'..=b'Z' => Ok((c - b'A') as u32),
            b'a'..=b'z' => Ok((c - b'a' + 26) as u32),
            b'0'..=b'9' => Ok((c - b'0' + 52) as u32),
            b'+' => Ok(62),
            b'/' => Ok(63),
            _ => Err(format!("invalid base64 char {:?}", c as char)),
        }
    }
    let bytes = s.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return Err("length not a multiple of 4".to_string());
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        let pad = chunk.iter().filter(|&&c| c == b'=').count();
        let mut n = 0u32;
        for (i, &c) in chunk.iter().enumerate() {
            let v = if c == b'=' { 0 } else { val(c)? };
            n |= v << (18 - 6 * i);
        }
        out.push((n >> 16 & 0xff) as u8);
        if pad < 2 {
            out.push((n >> 8 & 0xff) as u8);
        }
        if pad < 1 {
            out.push((n & 0xff) as u8);
        }
    }
    Ok(out)
}

// draft note 4
