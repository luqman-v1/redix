SHELL := /bin/bash
CARGO_ENV := source ~/.cargo/env &&

.PHONY: dev build test test-rust test-e2e check clean

# Development
dev:
	$(CARGO_ENV) pnpm tauri dev

# Production build
build:
	$(CARGO_ENV) pnpm tauri build

# All tests
test: test-rust check test-e2e

# Rust unit tests
test-rust:
	$(CARGO_ENV) cd src-tauri && cargo test

# Frontend type check
check:
	pnpm check

# E2E tests
test-e2e:
	pnpm test:e2e

# Integration tests (requires Docker Redis)
test-integration:
	docker compose -f docker-compose.test.yml up -d
	$(CARGO_ENV) cd src-tauri && cargo test -- --ignored; \
	docker compose -f docker-compose.test.yml down

# Clean build artifacts
clean:
	$(CARGO_ENV) cd src-tauri && cargo clean
	rm -rf node_modules/.vite build
