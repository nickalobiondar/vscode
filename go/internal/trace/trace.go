// Package trace implements the portsmith line-based trace format (v1).
//
// A trace file is UTF-8 text. The first non-empty line SHOULD be the magic
// header "#portsmith-trace v1". Lines beginning with '#' are comments and are
// ignored by decoders. Every other non-empty line is a record with the shape:
//
//	V1 <ts_unix_nanos> <dir> <session> <proto> <len> <base64-payload>
//
// where:
//   - ts_unix_nanos : int64 nanoseconds since the Unix epoch
//   - dir           : ">" for request (client->server) or "<" for response
//   - session       : opaque session identifier (no spaces)
//   - proto         : protocol hint ("tcp", "http", "redis", "raw", ...)
//   - len           : decimal length in bytes of the decoded payload
//   - payload        : standard base64 encoding of the raw payload bytes
//
// The format is intentionally simple so that both the Go capture tool and the
