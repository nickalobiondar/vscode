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
			dir, body = trace.Response, line[2:]
		}
		if cur != nil && dir != curDir {
			flush()
		}
		if cur == nil {
			cur = &strings.Builder{}
			curDir = dir
		}
		cur.WriteString(body)
		cur.WriteByte('\n')
	}
	flush()
	if err := sc.Err(); err != nil {
		return nil, err
	}
	return records, nil
}

// Stats summarizes a set of records.
type Stats struct {
	Records    int
	Requests   int
	Responses  int
	Sessions   int
	TotalBytes int
	ByProto    map[string]int
	FirstNanos int64
	LastNanos  int64
}

// Compute walks a trace.Reader and returns aggregate statistics.
func Compute(tr *trace.Reader) (Stats, error) {
	s := Stats{ByProto: map[string]int{}, FirstNanos: -1}
	sessions := map[string]struct{}{}
	for {
		rec, err := tr.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return s, err
		}
		s.Records++
		s.TotalBytes += len(rec.Payload)
		s.ByProto[rec.Proto]++
		sessions[rec.Session] = struct{}{}
		if rec.Dir == trace.Response {
			s.Responses++
		} else {
			s.Requests++
		}
		if s.FirstNanos < 0 || rec.TimestampNanos < s.FirstNanos {
			s.FirstNanos = rec.TimestampNanos
		}
		if rec.TimestampNanos > s.LastNanos {
			s.LastNanos = rec.TimestampNanos
		}
	}
	s.Sessions = len(sessions)
	if s.FirstNanos < 0 {
		s.FirstNanos = 0
	}
	return s, nil
}

// SortedProtos returns proto keys in descending count order for stable output.
func (s Stats) SortedProtos() []string {
	keys := make([]string, 0, len(s.ByProto))
	for k := range s.ByProto {
		keys = append(keys, k)
	}
	sort.Slice(keys, func(i, j int) bool {
		if s.ByProto[keys[i]] != s.ByProto[keys[j]] {
			return s.ByProto[keys[i]] > s.ByProto[keys[j]]
		}
		return keys[i] < keys[j]
	})
	return keys
}

// Duration returns the wall-clock span covered by the trace.
func (s Stats) Duration() time.Duration {
	if s.Records == 0 {
		return 0
	}
	return time.Duration(s.LastNanos - s.FirstNanos)
}

// draft note 70
