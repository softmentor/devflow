.PHONY: setup fmt fmt-check lint build test package check release ci-render

setup:
	cargo run -p devflow-cli -- setup

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

ci-render:
	cargo run -p devflow-cli -- ci:render
