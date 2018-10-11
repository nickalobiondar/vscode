// Command portcap captures and normalizes protocol traffic into the portsmith
// line-based trace format.
//
// Subcommands:
//
//	portcap capture   -listen :9000 -target host:6379 -proto redis -out cap.trace
//	portcap normalize -in session.log -proto http -out out.trace
//	portcap stats     -in cap.trace
//	portcap version
package main

import (
	"flag"
	"fmt"
	"io"
	"os"
	"time"

	"github.com/portsmith/portcap/internal/capture"
	"github.com/portsmith/portcap/internal/normalize"
	"github.com/portsmith/portcap/internal/trace"
)

const version = "1.0.0"

func main() {
	if len(os.Args) < 2 {
		usage()
		os.Exit(2)
	}
