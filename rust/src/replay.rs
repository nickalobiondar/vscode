//! Replay: send captured request records to a live target and collect responses.
//!
//! The replayer connects once per session, writes each request payload in order,
//! and reads whatever the server returns before the next request (bounded by a
//! read timeout). It optionally preserves the original inter-request delays.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::trace::{Direction, Record};

/// Configuration for a replay run.
#[derive(Debug, Clone)]
pub struct ReplayConfig {
    pub target: String,
    pub read_timeout: Duration,
    pub preserve_timing: bool,
    /// Upper bound on how long we wait after each request for a response.
    pub max_wait: Duration,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        ReplayConfig {
            target: String::new(),
            read_timeout: Duration::from_millis(200),
            preserve_timing: false,
            max_wait: Duration::from_millis(500),
        }
    }
}

/// Outcome of replaying a single request.
#[derive(Debug, Clone)]
pub struct Exchange {
    pub session: String,
    pub request: Vec<u8>,
    pub response: Vec<u8>,
    pub error: Option<String>,
}

/// Aggregate results of a replay run.
#[derive(Debug, Clone, Default)]
pub struct ReplayReport {
    pub exchanges: Vec<Exchange>,
    pub errors: usize,
}

impl ReplayReport {
    pub fn summary(&self) -> String {
        let bytes_in: usize = self.exchanges.iter().map(|e| e.response.len()).sum();
        let bytes_out: usize = self.exchanges.iter().map(|e| e.request.len()).sum();
        format!(
            "replayed {} request(s), {} error(s), sent {} byte(s), received {} byte(s)",
            self.exchanges.len(),
            self.errors,
            bytes_out,
            bytes_in
        )
    }
}

/// Replay all request records grouped by session against cfg.target.
///
/// Response records in the input are ignored for sending but could be used by
/// callers for comparison. Returns an error only for setup failures; per-request
/// failures are captured in the report.
pub fn replay(records: &[Record], cfg: &ReplayConfig) -> ReplayReport {
    // Preserve session ordering by first appearance.
    let mut order: Vec<String> = Vec::new();
    let mut by_session: BTreeMap<String, Vec<&Record>> = BTreeMap::new();
