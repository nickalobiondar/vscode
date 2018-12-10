// Package normalize converts loosely-structured session logs into canonical
// trace records and computes summary statistics over a trace stream.
package normalize

import (
	"bufio"
	"io"
	"sort"
	"strings"
	"time"

	"github.com/portsmith/portcap/internal/trace"
)

// FromRawLog parses a simple line-oriented session log into trace records.
//
// Each line is "<dir> <text>" where <dir> is ">" or "<". Consecutive lines
// with the same direction are coalesced into a single record. Timestamps are
// synthesized monotonically starting at startNanos with a fixed step. The
// resulting payloads are the raw UTF-8 bytes of the text (a trailing newline is
// appended so replayed line protocols behave correctly).
func FromRawLog(r io.Reader, proto, session string, startNanos, stepNanos int64) ([]trace.Record, error) {
	sc := bufio.NewScanner(r)
	sc.Buffer(make([]byte, 0, 64*1024), 8*1024*1024)

	var records []trace.Record
	var cur *strings.Builder
	var curDir trace.Direction
	ts := startNanos

	flush := func() {
		if cur == nil {
			return
		}
		records = append(records, trace.Record{
			TimestampNanos: ts,
			Dir:            curDir,
			Session:        session,
			Proto:          proto,
			Payload:        []byte(cur.String()),
		})
		ts += stepNanos
		cur = nil
	}

	for sc.Scan() {
		line := strings.TrimRight(sc.Text(), "\r")
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		dir := trace.Request
		body := line
		switch {
		case line == ">":
			dir, body = trace.Request, ""
		case line == "<":
			dir, body = trace.Response, ""
		case strings.HasPrefix(line, "> "):
			dir, body = trace.Request, line[2:]
		case strings.HasPrefix(line, "< "):
