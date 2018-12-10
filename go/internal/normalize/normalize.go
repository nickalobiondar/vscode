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

