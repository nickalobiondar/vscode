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
