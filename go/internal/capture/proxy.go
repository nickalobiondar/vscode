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
