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

```
V1 SP ts_nanos SP dir SP session SP proto SP len SP payload
```

Fields are separated by a single ASCII space (`0x20`). The payload is the final
field; because it is base64 it contains no spaces, so a decoder may split on the
first six spaces (`splitn(7)`).

| Field       | Type    | Description                                                    |
|-------------|---------|----------------------------------------------------------------|
| version     | literal | Always `V1` for this revision.                                 |
| `ts_nanos`  | int64   | Timestamp in nanoseconds since the Unix epoch.                 |
| `dir`       | enum    | `>` = request (client→server), `<` = response (server→client). |
| `session`   | token   | Opaque session id, no spaces.                                  |
| `proto`     | token   | Protocol hint: `tcp`, `http`, `redis`, `raw`, …                |
| `len`       | int     | Decimal byte length of the **decoded** payload.                |
| `payload`   | base64  | Standard base64 (`+`/`/`, `=` padding) of the raw bytes.       |

### Validation rules

A decoder MUST reject a record when:

* the line does not split into exactly 7 fields;
* the version is not `V1`;
* `ts_nanos` is not a valid int64;
* `dir` is neither `>` nor `<`;
* `len` is not a valid non-negative integer;
* the payload is not valid standard base64;
* the decoded payload length does not equal `len`.

The `len` field is redundant with the base64 payload on purpose: it lets a reader
cheaply detect truncation or corruption before allocating.
