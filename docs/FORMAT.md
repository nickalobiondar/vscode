# portsmith trace format (v1)

The trace format is the contract between the Go capture tool (`portcap`) and the
Rust inference/replay tool (`portsmith-replay`). It is a line-oriented UTF-8 text
format designed to be trivially parseable with each language's standard library.

## File structure

```
#portsmith-trace v1        <- recommended magic header (a comment)
# any number of comment lines beginning with '#'
<record>
<record>
...
```

* Encoding: UTF-8.
* Line separator: `\n`. A trailing `\r` is tolerated on read.
* Blank lines are ignored.
* Lines beginning with `#` are comments and are ignored by decoders.
* Every other non-empty line is exactly one **record**.

## Record grammar
