// Package capture provides a transparent TCP proxy that records both
// directions of traffic to the portsmith trace format.
package capture

import (
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"io"
	"net"
	"sync"
	"time"

	"github.com/portsmith/portcap/internal/trace"
)

// Proxy accepts connections on Listen, dials Target, and copies bytes in both
// directions while writing each chunk to a trace.Writer.
type Proxy struct {
	Listen string
	Target string
	Proto  string

	mu     sync.Mutex
	writer *trace.Writer
	now    func() time.Time
}

// New creates a Proxy that writes captured records via w.
func New(listen, target, proto string, w *trace.Writer) *Proxy {
	return &Proxy{
		Listen: listen,
		Target: target,
		Proto:  proto,
		writer: w,
		now:    time.Now,
	}
}

// Serve accepts up to maxConns connections (0 == unlimited) and blocks until
// the listener is closed or the connection limit is reached.
func (p *Proxy) Serve(maxConns int) error {
	ln, err := net.Listen("tcp", p.Listen)
	if err != nil {
		return fmt.Errorf("listen %s: %w", p.Listen, err)
	}
	defer ln.Close()

	var wg sync.WaitGroup
	handled := 0
	for {
		conn, err := ln.Accept()
		if err != nil {
			wg.Wait()
			return nil
		}
		wg.Add(1)
		go func(c net.Conn) {
