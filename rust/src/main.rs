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
    if args.len() < 2 {
        usage();
        exit(2);
    }
    let result = match args[1].as_str() {
        "infer" => cmd_infer(&args[2..]),
        "replay" => cmd_replay(&args[2..]),
        "cat" => cmd_cat(&args[2..]),
        "version" | "-v" | "--version" => {
            println!("portsmith-replay {VERSION}");
            Ok(())
        }
        "help" | "-h" | "--help" => {
            usage();
            Ok(())
        }
        other => {
            eprintln!("portsmith-replay: unknown command {other:?}");
            usage();
            exit(2);
        }
    };
    if let Err(e) = result {
        eprintln!("portsmith-replay: {e}");
        exit(1);
    }
}

fn usage() {
    eprint!(
        "portsmith-replay - schema inference & replay (portsmith trace v1)\n\
\n\
usage:\n\
  portsmith-replay infer  -in FILE\n\
  portsmith-replay replay -in FILE -target HOST:PORT [-timing] [-timeout MS] [-wait MS]\n\
  portsmith-replay cat    -in FILE\n\
  portsmith-replay version\n\
