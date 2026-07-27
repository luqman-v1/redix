SHELL := /bin/bash
CARGO_ENV := if [ -f ~/.cargo/env ]; then source ~/.cargo/env; fi;
PNPM := npx pnpm

.PHONY: dev build test test-rust test-e2e check clean install

# Install dependencies
install:
	$(PNPM) install

# Development
dev: install
	$(CARGO_ENV) $(PNPM) tauri dev

# Production build
build: install
	$(CARGO_ENV) $(PNPM) tauri build

# All tests
test: test-rust check test-e2e

# Rust unit tests
test-rust:
	$(CARGO_ENV) cd src-tauri && cargo test

# Frontend type check
check:
	$(PNPM) check

# E2E tests
test-e2e:
	$(PNPM) test:e2e

# Integration tests (requires Docker Redis)
test-integration:
	docker compose -f docker-compose.test.yml up -d
	$(CARGO_ENV) cd src-tauri && cargo test -- --ignored; \
	docker compose -f docker-compose.test.yml down

# Clean build artifacts
clean:
	$(CARGO_ENV) cd src-tauri && cargo clean
	rm -rf node_modules/.vite build
