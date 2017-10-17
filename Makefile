# Cairn build orchestration.
#
# Targets build the Rust engine and the Go wrapper, run both test suites, and
# run the end-to-end demo.

# Detect Windows for the correct binary extension.
ifeq ($(OS),Windows_NT)
	EXE := .exe
else
	EXE :=
endif

RUST_BIN := rust/target/release/cairn$(EXE)
GO_BIN   := bin/cairn-run$(EXE)

.PHONY: all build build-rust build-go test test-rust test-go fmt vet clean demo

all: build

build: build-rust build-go ## Build both binaries

build-rust: ## Build the Rust engine (release)
	cd rust && cargo build --release

build-go: ## Build the Go wrapper
	@mkdir -p bin
	cd go && go build -o ../$(GO_BIN) ./cmd/cairn-run

test: test-rust test-go ## Run all tests

test-rust: ## Run Rust tests
	cd rust && cargo test

