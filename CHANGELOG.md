# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- planning: TLS-aware capture mode for encrypted session plumbing

## [1.0.0] - 2026-07-08

### Added
- **Trace format v1 frozen** - documented line-based, base64-framed trace format
  shared by both tools (`docs/FORMAT.md`).
- **`portcap` (Go)**: transparent TCP `capture` proxy recording both directions,
  plus `normalize` for raw `> / <` session logs.
- **`portsmith-replay` (Rust)**: `replay` a capture against a recorded trace and
  diff the two; protocol `infer` for schema discovery.
- Sample captures and traces for HTTP and RESP/Redis sessions.

### Verified
- `make test` green on both toolchains (go test -race, cargo test).
- gofmt / clippy / rustfmt clean.
