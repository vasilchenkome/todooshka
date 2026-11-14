# syntax=docker/dockerfile:1

FROM node:26-alpine AS frontend
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY frontend/ ./
RUN npm run build

FROM rust:1.99-slim-bookworm AS backend
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY --from=frontend /app/frontend/dist ./frontend/dist
RUN cargo build --release --locked -p todooshka \
    && strip target/release/todooshka

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home todooshka \
    && mkdir -p /data \
    && chown todooshka:todooshka /data

COPY --from=backend /app/target/release/todooshka /usr/local/bin/todooshka

USER todooshka
ENV BIND_ADDR=0.0.0.0:8080 \
    DATABASE_URL=sqlite:///data/todooshka.db \
    RUST_LOG=info \
    LOG_FORMAT=json

EXPOSE 8080
VOLUME ["/data"]
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["todooshka", "healthcheck"]
ENTRYPOINT ["todooshka"]
