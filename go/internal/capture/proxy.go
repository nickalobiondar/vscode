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
