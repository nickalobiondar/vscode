<!-- portsmith — protocol cartographer's field manual -->
<p align="center">
  <img src="docs/assets/hero.svg" alt="portsmith — a protocol cartographer's field manual: an oscilloscope tracing packets through capture, normalize, infer and replay" width="100%">
</p>

<h1 align="center">portsmith</h1>

<p align="center">
  <strong>Protocol discovery &amp; replay for line-based network traffic.</strong><br>
  <em>Put an unknown wire protocol on the bench, watch its waveform, chart its schema, and replay it.</em>
</p>

<p align="center">
  <code>capture</code> &#8594; <code>normalize</code> &#8594; <code>infer</code> &#8594; <code>replay</code> &#183;
  <code>Go</code> + <code>Rust</code> &#183; standard libraries only &#183; MIT
</p>

---

## Field manual

You have a TCP service with no spec — an undocumented daemon, a vendor box that
speaks its own dialect, a Redis-flavoured thing you want to regression-test. You
have a socket, not a document.

**portsmith is the bench kit for that socket.** It treats an opaque protocol the
way an electronics workshop treats an unknown signal: clip a probe onto the wire
(`capture`), tidy the recording into a clean trace (`normalize`), read the
waveform and chart its structure (`infer`), then drive the same stimulus back
into a live device (`replay`). No protocol-specific code, no dependency tree —
two small binaries, each in the language that suits its job, exchanging one
documented text format.

> **Not** a Wireshark, a fuzzer, or a full reverse-engineering suite. portsmith
> is a focused, auditable toolchain for line-oriented request/response traffic
> you can read end-to-end in an afternoon.

---

## Table of contents

- [Why portsmith](#why-portsmith)
- [The two instruments](#the-two-instruments)
- [Install &amp; build](#install--build)
- [Bench session (annotated transcript)](#bench-session-annotated-transcript)
- [The trace format, walked field by field](#the-trace-format-walked-field-by-field)
- [Architecture](#architecture)
- [Trace flow](#trace-flow)
- [Command reference](#command-reference)
- [Workflows](#workflows)
- [Use cases](#use-cases)
- [How inference actually works](#how-inference-actually-works)
- [How replay actually works](#how-replay-actually-works)
- [Design choices](#design-choices)
- [Comparison](#comparison)
- [Limitations](#limitations)
- [Troubleshooting](#troubleshooting)
- [Repository layout](#repository-layout)
- [Roadmap](#roadmap)
- [License](#license)

---

## Why portsmith

Most protocol tooling assumes you already know the protocol. portsmith assumes
you do not, and optimizes for the *discovery loop*:

- **Two languages, deliberately.** Capture and normalization are I/O-and-goroutine
  heavy — Go's home turf (`portcap`). Inference and replay want a tight,
  allocation-conscious core with strong pattern matching — Rust
  (`portsmith-replay`). Neither reaches for a third-party crate or module.
- **One contract between them.** The `*.trace` file is the *only* coupling. Both
  sides implement the same v1 grammar against their standard libraries, including
  a hand-rolled standard-base64 codec on the Rust side.
- **Everything is inspectable.** Traces are UTF-8 text; payloads are base64 so a
  single grep-able line survives binary data and embedded NULs. Schema reports and
  replay output are plain text, made for humans and pipes alike.

---

## The two instruments

| Instrument | Language | Role on the bench | Subcommands |
|------------|----------|-------------------|-------------|
| **`portcap`** | Go | The probe &amp; recorder. Sits in front of a TCP service as a transparent proxy and records both directions; also normalizes loose logs and summarizes traces. | `capture`, `normalize`, `stats`, `version` |
| **`portsmith-replay`** | Rust | The scope &amp; signal generator. Reads a trace, infers the protocol's structure, pretty-prints it, and replays captured requests against a live target. | `infer`, `replay`, `cat`, `version` |

Both tools report version `1.0.0`.

---

## Install &amp; build

Prerequisites: **Go 1.24** and a **stable Rust toolchain** (`cargo`).

```sh
make build        # builds the Go binary (bin/portcap) and the Rust release binary
make test         # runs `go test ./...` and `cargo test`
```

Prefer to drive each toolchain yourself:

