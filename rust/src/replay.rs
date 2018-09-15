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
