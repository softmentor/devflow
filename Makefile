.PHONY: all verify setup clean setup-tools fmt fmt-check lint build test package check release ci-generate coverage bench docs

setup:
	cargo run -p devflow-cli -- setup

clean:
	cargo clean

setup-tools:
	cargo install cargo-llvm-cov
	cargo install cargo-criterion

fmt:
	cargo run -p devflow-cli -- fmt:fix

fmt-check:
	cargo run -p devflow-cli -- fmt:check

lint:
	cargo run -p devflow-cli -- lint:static

build:
	cargo run -p devflow-cli -- build:debug

test:
	cargo run -p devflow-cli -- test:unit

package:
	cargo run -p devflow-cli -- package:artifact

check:
	cargo run -p devflow-cli -- check:pr

release:
	cargo run -p devflow-cli -- release:candidate

ci-generate:
	cargo run -p devflow-cli -- ci:generate

coverage:
	cargo llvm-cov

bench:
	cargo bench

docs:
	cargo doc --no-deps --workspace --open

# Typical development flow: fix formatting, lint, and run tests.
dev: fmt lint test

# Comprehensive check: formatting check, lint, build, and run tests.
# Useful for local verification before pushing.
verify: fmt-check lint build test

# The works: formatting, linting, building, testing, and coverage.
all: fmt lint build test coverage docs
