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

# Validates the current branch for a PR before merging to main
check: verify
	@echo "✓ PR verification passed. Branch is ready for merging."

# Helper to ensure Cargo.toml and myst.yml versions are synchronized
verify-versions:
	@echo "Verifying version synchronization..."
	@CARGO_VER=$$(grep '^version = ' Cargo.toml | head -n 1 | sed -E 's/version = "([^"]+)"/\1/'); \
	MYST_VER=$$(grep 'logo_text: Devflow v' docs/myst.yml | head -n 1 | sed -E 's/.*logo_text: Devflow v([0-9\.]+)/\1/'); \
	if [ -z "$$CARGO_VER" ]; then echo "Error: Could not extract version from Cargo.toml"; exit 1; fi; \
	if [ "$$CARGO_VER" != "$$MYST_VER" ]; then \
		echo "Error: Version mismatch! Cargo.toml=$$CARGO_VER, myst.yml=$$MYST_VER"; \
		exit 1; \
	fi; \
	echo "✓ Versions are synchronized ($$CARGO_VER)"

# Helper to ensure there are no uncommitted changes
verify-clean:
	@echo "Verifying working tree is clean..."
	@if [ -n "$$(git status --porcelain)" ]; then \
		echo "Error: Working tree is not clean. Please commit or stash your changes."; \
		exit 1; \
	fi
	@echo "✓ Working tree is clean."

# Automates creating a PR from the current branch to main using the GitHub CLI
pr: check verify-clean
	@echo "Creating Pull Request to main..."
	gh pr create --base main --title "chore: release $$(git branch --show-current | sed 's/rel\///')" --fill

# Automates checking out main, pulling, tagging, and pushing to trigger GitHub Actions
tag:
	@if [ -z "$(VERSION)" ]; then echo "Error: VERSION is not set. Usage: make tag VERSION=v0.2.0"; exit 1; fi
	@echo "Checking out main and pulling latest changes..."
	git checkout main
	git pull origin main
	@echo "Tagging release $(VERSION)..."
	git tag -a "$(VERSION)" -m "chore: release $(VERSION)"
	@echo "Pushing tag $(VERSION) to origin..."
	git push origin "$(VERSION)"
	@echo "✓ Tag $(VERSION) pushed successfully! Release pipelines are triggering."

# Backwards compatibility alias
release:
	@echo "Please use 'make tag VERSION=v0.2.0' instead."

ci-generate:
	cargo run -p devflow-cli -- ci:generate

coverage:
	cargo llvm-cov

bench:
	cargo bench

doc-myst:
	@if command -v myst >/dev/null 2>&1; then \
		echo "📚 Building MyST documentation..."; \
		cd docs && myst build --html; \
		echo "✅ Documentation built successfully."; \
	else \
		echo "⚠️  MyST not found, skipping documentation build."; \
	fi

docs: doc-myst
	cargo doc --no-deps --workspace --open

# Typical development flow: fix formatting, lint, and run tests.
dev: fmt lint test

# Comprehensive check: formatting check, lint, build, and run tests.
# Useful for local verification before pushing.
verify: fmt-check lint build test verify-versions

# The works: formatting, linting, building, testing, and coverage.
all: fmt lint build test coverage docs
