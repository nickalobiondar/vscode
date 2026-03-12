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
			defer wg.Done()
			if e := p.handle(c); e != nil {
				fmt.Println("# capture: session error:", e)
			}
		}(conn)
		handled++
		if maxConns > 0 && handled >= maxConns {
			// Wait for in-flight sessions then stop.
			go func() { wg.Wait(); ln.Close() }()
		}
	}
}

func (p *Proxy) handle(client net.Conn) error {
	defer client.Close()
	session := newSessionID()

	upstream, err := net.Dial("tcp", p.Target)
	if err != nil {
		return fmt.Errorf("dial target %s: %w", p.Target, err)
	}
	defer upstream.Close()

	var wg sync.WaitGroup
	wg.Add(2)
	// client -> upstream : Request records
	go func() {
		defer wg.Done()
		p.pump(client, upstream, session, trace.Request)
		if tc, ok := upstream.(*net.TCPConn); ok {
			_ = tc.CloseWrite()
		}
	}()
	// upstream -> client : Response records
	go func() {
		defer wg.Done()
		p.pump(upstream, client, session, trace.Response)
		if tc, ok := client.(*net.TCPConn); ok {
			_ = tc.CloseWrite()
		}
	}()
	wg.Wait()
	return nil
}

// pump copies src->dst, emitting one trace record per read.
func (p *Proxy) pump(src io.Reader, dst io.Writer, session string, dir trace.Direction) {
	buf := make([]byte, 32*1024)
	for {
		n, err := src.Read(buf)
		if n > 0 {
			chunk := make([]byte, n)
			copy(chunk, buf[:n])
			p.record(trace.Record{
				TimestampNanos: p.now().UnixNano(),
				Dir:            dir,
				Session:        session,
				Proto:          p.Proto,
				Payload:        chunk,
			})
			if _, werr := dst.Write(chunk); werr != nil {
				return
			}
		}
		if err != nil {
			return
		}
	}
}

func (p *Proxy) record(r trace.Record) {
	p.mu.Lock()
	defer p.mu.Unlock()
	if err := p.writer.Write(r); err == nil {
		_ = p.writer.Flush()
	}
}

func newSessionID() string {
	var b [6]byte
	if _, err := rand.Read(b[:]); err != nil {
		// Fall back to a time-derived id; still unique enough for a trace.
		return fmt.Sprintf("%012x", time.Now().UnixNano())
	}
	return hex.EncodeToString(b[:])
}

// draft note 9
