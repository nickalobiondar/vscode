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
