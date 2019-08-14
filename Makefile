# portsmith build orchestration
#
# Targets:
#   make build   build both binaries
#   make test    run Go and Rust tests
#   make fmt     format both codebases
#   make vet     static checks (go vet + cargo check)
#   make demo    regenerate sample traces and print an inference report
#   make clean   remove build artifacts

GO_DIR   := go
RUST_DIR := rust
BIN_DIR  := bin

