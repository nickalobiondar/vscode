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
    for r in records {
        if !by_session.contains_key(&r.session) {
            order.push(r.session.clone());
        }
        by_session.entry(r.session.clone()).or_default().push(r);
    }

    let mut report = ReplayReport::default();
    for session in order {
        let recs = &by_session[&session];
        replay_session(&session, recs, cfg, &mut report);
    }
    report
}

fn replay_session(session: &str, recs: &[&Record], cfg: &ReplayConfig, report: &mut ReplayReport) {
    let stream = match TcpStream::connect(&cfg.target) {
        Ok(s) => s,
        Err(e) => {
            report.errors += 1;
            report.exchanges.push(Exchange {
                session: session.to_string(),
                request: Vec::new(),
                response: Vec::new(),
                error: Some(format!("connect {}: {e}", cfg.target)),
            });
            return;
        }
    };
    let _ = stream.set_read_timeout(Some(cfg.read_timeout));
    let mut stream = stream;

    let mut prev_ts: Option<i64> = None;
    for r in recs {
        if r.dir != Direction::Request {
            prev_ts = Some(r.ts_nanos);
            continue;
        }
        if cfg.preserve_timing {
            if let Some(pt) = prev_ts {
                let delta = r.ts_nanos.saturating_sub(pt);
                if delta > 0 {
                    let capped = delta.min(2_000_000_000); // cap at 2s
                    std::thread::sleep(Duration::from_nanos(capped as u64));
                }
            }
        }
        prev_ts = Some(r.ts_nanos);

        let mut ex = Exchange {
            session: session.to_string(),
            request: r.payload.clone(),
            response: Vec::new(),
            error: None,
        };
        if let Err(e) = stream.write_all(&r.payload) {
            ex.error = Some(format!("write: {e}"));
            report.errors += 1;
            report.exchanges.push(ex);
            return;
        }
        let _ = stream.flush();
        ex.response = read_response(&mut stream, cfg.max_wait);
        report.exchanges.push(ex);
    }
}

fn read_response(stream: &mut TcpStream, max_wait: Duration) -> Vec<u8> {
    let start = std::time::Instant::now();
    let mut out = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                out.extend_from_slice(&buf[..n]);
                if n < buf.len() {
                    break;
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                break;
            }
            Err(_) => break,
        }
        if start.elapsed() >= max_wait {
            break;
        }
    }
    out
}

// draft note 66
