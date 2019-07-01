# Contributing to Portsmith

Thanks for looking at Portsmith. Two cooperating tools share one trace format:

- `go/` - `portcap`, the capture proxy and normalizer (Go 1.24+)
- `rust/` - `portsmith-replay`, replay/diff/infer (stable Rust)

## Ground rules

- The trace format is a contract: changes are additive-only and must be
  mirrored in both toolchains in the same PR.
- Determinism: `replay` and `infer` output is byte-stable for identical input.
  No wall-clock, no map iteration order in rendered output.
- `make test` must stay green on both toolchains; `gofmt`, `rustfmt` and
