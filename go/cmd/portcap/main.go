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

