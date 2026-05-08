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
// Rust replay tool can parse it with standard libraries only.
package trace

import (
	"bufio"
	"encoding/base64"
	"fmt"
	"io"
	"strconv"
	"strings"
)

// Magic is the recommended first line of a trace file.
const Magic = "#portsmith-trace v1"

// Direction identifies which side of a session produced a record.
type Direction byte

const (
	// Request is a client-to-server message.
	Request Direction = '>'
	// Response is a server-to-client message.
	Response Direction = '<'
)

func (d Direction) String() string {
	if d == Response {
		return "<"
	}
	return ">"
}

// Record is a single captured message.
type Record struct {
	TimestampNanos int64
	Dir            Direction
	Session        string
	Proto          string
	Payload        []byte
}

// Encode writes the record as a single trace line (no trailing newline).
func (r Record) Encode() string {
	return fmt.Sprintf("V1 %d %s %s %s %d %s",
		r.TimestampNanos,
		r.Dir,
		r.Session,
		r.Proto,
		len(r.Payload),
		base64.StdEncoding.EncodeToString(r.Payload),
	)
}

// ParseError describes a malformed record along with the offending line.
type ParseError struct {
	Line int
	Msg  string
}

func (e *ParseError) Error() string {
	return fmt.Sprintf("trace: line %d: %s", e.Line, e.Msg)
}

// DecodeLine parses a single record line. It returns an error for malformed
// input. Comment and blank lines must be filtered by the caller.
func DecodeLine(line string, lineNo int) (Record, error) {
	fields := strings.SplitN(line, " ", 7)
	if len(fields) != 7 {
		return Record{}, &ParseError{lineNo, fmt.Sprintf("expected 7 fields, got %d", len(fields))}
	}
	if fields[0] != "V1" {
		return Record{}, &ParseError{lineNo, "unknown record version " + strconv.Quote(fields[0])}
	}
	ts, err := strconv.ParseInt(fields[1], 10, 64)
	if err != nil {
		return Record{}, &ParseError{lineNo, "invalid timestamp: " + err.Error()}
	}
	if len(fields[2]) != 1 || (fields[2][0] != '>' && fields[2][0] != '<') {
		return Record{}, &ParseError{lineNo, "invalid direction " + strconv.Quote(fields[2])}
	}
	dir := Direction(fields[2][0])
	declaredLen, err := strconv.Atoi(fields[5])
	if err != nil {
		return Record{}, &ParseError{lineNo, "invalid length: " + err.Error()}
	}
	payload, err := base64.StdEncoding.DecodeString(fields[6])
	if err != nil {
		return Record{}, &ParseError{lineNo, "invalid base64 payload: " + err.Error()}
	}
	if len(payload) != declaredLen {
		return Record{}, &ParseError{lineNo, fmt.Sprintf("length mismatch: declared %d, decoded %d", declaredLen, len(payload))}
	}
	return Record{
		TimestampNanos: ts,
		Dir:            dir,
		Session:        fields[3],
		Proto:          fields[4],
		Payload:        payload,
	}, nil
}

// Writer serializes records to an io.Writer.
type Writer struct {
	w             *bufio.Writer
	wroteMagic    bool
	suppressMagic bool
}

// NewWriter returns a Writer. If suppressMagic is false, the magic header is
// emitted before the first record.
func NewWriter(w io.Writer, suppressMagic bool) *Writer {
	return &Writer{w: bufio.NewWriter(w), suppressMagic: suppressMagic}
}

// Write encodes and writes a single record.
func (tw *Writer) Write(r Record) error {
	if !tw.wroteMagic && !tw.suppressMagic {
		if _, err := tw.w.WriteString(Magic + "\n"); err != nil {
			return err
		}
		tw.wroteMagic = true
	}
	if _, err := tw.w.WriteString(r.Encode()); err != nil {
		return err
	}
	return tw.w.WriteByte('\n')
}

// Flush flushes buffered data.
func (tw *Writer) Flush() error { return tw.w.Flush() }

// Reader streams records from an io.Reader, skipping comments and blank lines.
type Reader struct {
	sc     *bufio.Scanner
	lineNo int
}

// NewReader returns a Reader over r. It supports lines up to 16 MiB.
func NewReader(r io.Reader) *Reader {
	sc := bufio.NewScanner(r)
	sc.Buffer(make([]byte, 0, 64*1024), 16*1024*1024)
	return &Reader{sc: sc}
}

// Next returns the next record. It returns io.EOF when the stream is exhausted.
func (tr *Reader) Next() (Record, error) {
	for tr.sc.Scan() {
		tr.lineNo++
		line := strings.TrimRight(tr.sc.Text(), "\r")
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		return DecodeLine(line, tr.lineNo)
	}
	if err := tr.sc.Err(); err != nil {
		return Record{}, err
	}
	return Record{}, io.EOF
}

// draft note 41
