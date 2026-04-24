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

.PHONY: build build-go build-rust test test-go test-rust fmt vet demo clean

build: build-go build-rust

build-go:
	cd $(GO_DIR) && go build -o ../$(BIN_DIR)/portcap ./cmd/portcap

build-rust:
	cd $(RUST_DIR) && cargo build --release

test: test-go test-rust

test-go:
	cd $(GO_DIR) && go test ./...

test-rust:
	cd $(RUST_DIR) && cargo test

fmt:
	cd $(GO_DIR) && go fmt ./...
	cd $(RUST_DIR) && cargo fmt

vet:
	cd $(GO_DIR) && go vet ./...
	cd $(RUST_DIR) && cargo check

demo:
	cd $(GO_DIR) && go run ./cmd/portcap normalize -in ../samples/redis.log \
		-proto redis -session a1b2c3 -start 1700000000000000000 -step 50000000 \
		-out ../samples/redis.trace
	cd $(RUST_DIR) && cargo run --quiet -- infer -in ../samples/redis.trace

clean:
	rm -rf $(BIN_DIR)
	cd $(RUST_DIR) && cargo clean

# draft note 29
