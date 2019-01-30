package trace

import (
	"bytes"
	"io"
	"testing"
)

func TestEncodeDecodeRoundTrip(t *testing.T) {
	recs := []Record{
		{TimestampNanos: 1000, Dir: Request, Session: "abc123", Proto: "redis", Payload: []byte("PING\r\n")},
		{TimestampNanos: 2000, Dir: Response, Session: "abc123", Proto: "redis", Payload: []byte("+PONG\r\n")},
		{TimestampNanos: 3000, Dir: Request, Session: "abc123", Proto: "raw", Payload: []byte{0x00, 0xff, 0x10}},
	}

	var buf bytes.Buffer
	w := NewWriter(&buf, false)
	for _, r := range recs {
		if err := w.Write(r); err != nil {
			t.Fatalf("write: %v", err)
		}
	}
	if err := w.Flush(); err != nil {
		t.Fatalf("flush: %v", err)
	}
