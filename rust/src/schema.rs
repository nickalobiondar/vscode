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
