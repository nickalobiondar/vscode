//! Command-line entry point for portsmith-replay.
//!
//! Subcommands:
//!   infer  -in FILE                 discover protocol structure from a trace
//!   replay -in FILE -target ADDR    replay requests against a live target
//!   cat    -in FILE                 pretty-print a trace as human-readable text

use std::fs::File;
use std::io::{self, BufReader, Write};
use std::process::exit;
use std::time::Duration;

use portsmith_replay::replay::{replay, ReplayConfig};
use portsmith_replay::schema::infer;
use portsmith_replay::trace::{self, Direction, Record};

const VERSION: &str = "1.0.0";

fn main() {
    let args: Vec<String> = std::env::args().collect();
