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
