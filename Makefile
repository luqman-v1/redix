SHELL := /bin/bash
CARGO_ENV := if [ -f ~/.cargo/env ]; then source ~/.cargo/env; fi;
BUN := bun

.PHONY: dev build test test-rust test-e2e check clean install

# Install dependencies
install:
	$(BUN) install

# Development
dev:
	$(CARGO_ENV) $(BUN) tauri dev

# Production build
build:
	$(CARGO_ENV) $(BUN) tauri build

# All tests
test: test-rust check test-e2e

# Rust unit tests
test-rust:
	$(CARGO_ENV) cd src-tauri && cargo test

check:
	$(BUN) run check

# E2E tests
test-e2e:
	$(BUN) run test:e2e

# Integration tests (requires Docker Redis)
test-integration:
	docker compose -f docker-compose.test.yml up -d
	$(CARGO_ENV) cd src-tauri && cargo test -- --ignored; \
	docker compose -f docker-compose.test.yml down

# Clean build artifacts
clean:
	$(CARGO_ENV) cd src-tauri && cargo clean
	rm -rf node_modules/.vite build
