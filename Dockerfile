# syntax=docker/dockerfile:1
# Черновик этапа 0: сборка фронта, бинаря и runtime-слоя.
# Проверка и финальные теги (версия rust/node, healthcheck, embed статики) — на этапе 6.

FROM node:24-alpine AS frontend
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

FROM rust:slim-bookworm AS backend
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY --from=frontend /app/frontend/dist ./frontend/dist
RUN cargo build --release -p todooshka

FROM debian:bookworm-slim
RUN useradd --system --uid 10001 --create-home todooshka \
    && mkdir -p /data \
    && chown todooshka:todooshka /data
COPY --from=backend /app/target/release/todooshka /usr/local/bin/todooshka
USER todooshka
ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080
VOLUME ["/data"]
ENTRYPOINT ["/usr/local/bin/todooshka"]
