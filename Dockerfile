FROM rust:1.94-bookworm AS builder

WORKDIR /app

RUN apt-get update \
  && apt-get install -y --no-install-recommends curl ca-certificates build-essential pkg-config libssl-dev \
  && curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
  && apt-get install -y --no-install-recommends nodejs \
  && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cd ui && npm ci && npm run build
RUN cargo build --release -p sol-safekey-ui

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/build-cache/release/sol-safekey-ui /usr/local/bin/sol-safekey-ui

ENV HOST=0.0.0.0
ENV PORT=3841
ENV SOL_SAFEKEY_DB_PATH=/app/data/sol-safekey.sqlite3

VOLUME ["/app/data"]
EXPOSE 3841

CMD ["sol-safekey-ui"]
