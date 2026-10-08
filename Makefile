SHELL := /bin/bash
CARGO_ENV := if [ -f ~/.cargo/env ]; then source ~/.cargo/env; fi;
BUN := bun

.PHONY: dev build test test-rust test-unit test-e2e check clean install

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
test: test-rust test-unit check test-e2e

# Rust unit tests
test-rust:
	$(CARGO_ENV) cd src-tauri && cargo test

# Frontend unit tests
test-unit:
	$(BUN) run test:unit

check:
	$(BUN) run check

# E2E tests
test-e2e:
	$(BUN) run test:e2e

# Integration tests. Prefers Docker; falls back to a local redis-server on
# port 6399 when the Docker daemon is unavailable (e.g. Docker Desktop off).
test-integration:
	@if docker info >/dev/null 2>&1; then \
		docker compose -f docker-compose.test.yml up -d; \
		$(CARGO_ENV) cd src-tauri && cargo test -- --ignored; \
		docker compose -f docker-compose.test.yml down; \
	else \
		echo "docker unavailable; using a local redis-server on :6399"; \
		redis-server --port 6399 --daemonize yes --save '' --appendonly no; \
		$(CARGO_ENV) cd src-tauri && cargo test -- --ignored; \
		redis-cli -p 6399 shutdown nosave 2>/dev/null || true; \
	fi

# Clean build artifacts
clean:
	$(CARGO_ENV) cd src-tauri && cargo clean
	rm -rf node_modules/.vite build
