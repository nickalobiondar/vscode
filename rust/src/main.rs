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
\n\
examples:\n\
  portsmith-replay infer  -in samples/redis.trace\n\
  portsmith-replay replay -in samples/redis.trace -target 127.0.0.1:6379\n\
  portsmith-replay cat    -in samples/http.trace\n"
    );
}

/// Minimal flag parser: collects -key value pairs and -flag toggles.
struct Flags {
    map: std::collections::HashMap<String, String>,
    toggles: std::collections::HashSet<String>,
}

impl Flags {
    fn parse(args: &[String]) -> Self {
        let mut map = std::collections::HashMap::new();
        let mut toggles = std::collections::HashSet::new();
        let mut i = 0;
        while i < args.len() {
            let a = &args[i];
            if let Some(key) = a.strip_prefix('-') {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    map.insert(key.to_string(), args[i + 1].clone());
                    i += 2;
                } else {
                    toggles.insert(key.to_string());
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        Flags { map, toggles }
    }
    fn get<'a>(&'a self, k: &str, default: &'a str) -> &'a str {
        self.map.get(k).map(String::as_str).unwrap_or(default)
    }
