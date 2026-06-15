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

	r := NewReader(&buf)
	var got []Record
	for {
		rec, err := r.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			t.Fatalf("read: %v", err)
		}
		got = append(got, rec)
	}

	if len(got) != len(recs) {
		t.Fatalf("got %d records, want %d", len(got), len(recs))
	}
	for i := range recs {
		if got[i].TimestampNanos != recs[i].TimestampNanos ||
			got[i].Dir != recs[i].Dir ||
			got[i].Session != recs[i].Session ||
			got[i].Proto != recs[i].Proto ||
			!bytes.Equal(got[i].Payload, recs[i].Payload) {
			t.Errorf("record %d mismatch: got %+v want %+v", i, got[i], recs[i])
		}
	}
}

func TestDecodeLineErrors(t *testing.T) {
	cases := []string{
		"V1 100 > s p",             // too few fields
		"V2 100 > s p 4 dGVzdA==",  // bad version
		"V1 xx > s p 4 dGVzdA==",   // bad timestamp
		"V1 100 ? s p 4 dGVzdA==",  // bad direction
		"V1 100 > s p z dGVzdA==",  // bad length
		"V1 100 > s p 4 !!!notb64", // bad base64
		"V1 100 > s p 9 dGVzdA==",  // length mismatch
	}
	for i, line := range cases {
		if _, err := DecodeLine(line, i+1); err == nil {
			t.Errorf("case %d %q: expected error, got nil", i, line)
		}
	}
}

// draft note 72
