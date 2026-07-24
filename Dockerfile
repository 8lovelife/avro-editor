# ---------- Stage 1: build the wasm bundle ----------
FROM rust:1.97-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked trunk

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY index.html ./index.html

RUN trunk build --release

# ---------- Stage 2: serve with static-web-server ----------
FROM joseluisq/static-web-server:2-alpine

COPY --from=builder /app/dist /var/public

ENV SERVER_ROOT=/var/public
ENV SERVER_PORT=80
ENV SERVER_COMPRESSION=true
ENV SERVER_LOG_LEVEL=info
ENV SERVER_CACHE_CONTROL_HEADERS=true

EXPOSE 80
