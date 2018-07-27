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
