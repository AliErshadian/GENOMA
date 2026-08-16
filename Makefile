# GENOMA development commands
# Windows: use Git Bash, WSL, or the equivalent PowerShell commands in the README.

.PHONY: help dev up down test test-rust test-web lint fmt pi-digits demos api web

help:
	@echo "GENOMA development targets"
	@echo "  make up         Start PostgreSQL, Redis, MinIO"
	@echo "  make down       Stop local services"
	@echo "  make pi-digits  Generate bundled pi digit dataset"
	@echo "  make demos      Generate demo files"
	@echo "  make test       Run Rust and frontend tests"
	@echo "  make api        Run the Axum API"
	@echo "  make web        Run the Next.js app"
	@echo "  make dev        Start services, API, and web (see README for Windows)"

up:
	docker compose up -d postgres redis minio minio-init

down:
	docker compose down

pi-digits:
	python scripts/generate_fixtures.py --pi

demos:
	python scripts/generate_fixtures.py --demos

test-rust:
	cargo test --workspace

test-web:
	pnpm test

test: test-rust test-web

lint:
	cargo clippy --workspace --all-targets -- -D warnings
	pnpm lint

fmt:
	cargo fmt --all
	pnpm --filter @genoma/web exec prettier --write .

api:
	cargo run -p genoma-api

web:
	pnpm --filter @genoma/web dev

dev: up
	@echo "Services started. In separate terminals run: make api   and   make web"
