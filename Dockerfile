FROM node:20.19.5-bookworm-slim AS frontend

WORKDIR /app/ui

COPY ui/package.json ui/package-lock.json ./
COPY ui/vendor/brace-expansion-compat ./vendor/brace-expansion-compat
RUN npm ci

COPY ui/ ./
RUN npm run build

FROM rust:1.89.0-bookworm AS builder

WORKDIR /app

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates build-essential pkg-config libssl-dev \
  && rm -rf /var/lib/apt/lists/*

COPY . .
COPY --from=frontend /app/ui/out /app/ui/out
RUN cargo build --locked --release -p sol-safekey-ui

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates curl \
  && rm -rf /var/lib/apt/lists/* \
  && groupadd --gid 10001 sol-safekey \
  && useradd --uid 10001 --gid 10001 --home-dir /app --no-create-home --no-log-init --shell /usr/sbin/nologin sol-safekey \
  && install -d -o 10001 -g 10001 /app/data

COPY --from=builder /app/build-cache/release/sol-safekey-ui /usr/local/bin/sol-safekey-ui

# The wallet API is intentionally local-only. Run with host networking when a
# container is required; publishing this loopback port with -p will not work.
ENV HOST=127.0.0.1
ENV PORT=3841
ENV SOL_SAFEKEY_DB_PATH=/app/data/sol-safekey.sqlite3

VOLUME ["/app/data"]

USER 10001:10001

HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
  CMD curl --fail --silent --show-error http://127.0.0.1:3841/api/health >/dev/null || exit 1

CMD ["sol-safekey-ui"]
