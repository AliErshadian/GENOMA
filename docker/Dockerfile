# syntax=docker/dockerfile:1

FROM rust:1.85-bookworm AS api-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY data/pi ./data/pi
RUN cargo build --release -p genoma-api

FROM node:22-bookworm AS web-builder
WORKDIR /app
RUN corepack enable
COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./
COPY apps/web/package.json apps/web/package.json
COPY packages/shared-types/package.json packages/shared-types/package.json
RUN pnpm install --frozen-lockfile
COPY apps/web ./apps/web
COPY packages/shared-types ./packages/shared-types
RUN pnpm --filter @genoma/web build

FROM debian:bookworm-slim AS api
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=api-builder /app/target/release/genoma-api /usr/local/bin/genoma-api
COPY data /app/data
ENV GENOMA_PI_DIGITS_PATH=/app/data/pi/pi-digits.bin
ENV GENOMA_DEMO_DIR=/app/data/demos
EXPOSE 8080
CMD ["genoma-api"]

FROM node:22-bookworm-slim AS web
WORKDIR /app
ENV NODE_ENV=production
COPY --from=web-builder /app/apps/web/.next/standalone ./
COPY --from=web-builder /app/apps/web/.next/static ./apps/web/.next/static
COPY --from=web-builder /app/apps/web/public ./apps/web/public
EXPOSE 3000
CMD ["node", "apps/web/server.js"]
