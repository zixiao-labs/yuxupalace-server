# syntax=docker/dockerfile:1.7
#
# Multi-stage Rust build for yuxu-server.
#
# The build context is the whole Cargo workspace (repo root), so we reference
# this file with `docker build -f crates/yuxu-server/Dockerfile .`.
#
# cargo-chef lets us cache the dependency build separately from the app code —
# without it, any source edit forces a full recompile (~8+ min on cold cache).

ARG RUST_VERSION=1.91
ARG DEBIAN_VERSION=bookworm

# ---- Stage 1: cargo-chef base ----
FROM rust:${RUST_VERSION}-slim-${DEBIAN_VERSION} AS chef
RUN cargo install --locked cargo-chef
WORKDIR /app

# ---- Stage 2: plan the dependency graph ----
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---- Stage 3: build ----
FROM chef AS builder
ARG FEATURES=sqlite

# protoc: raidian's build scripts regenerate when the committed source drifts.
# libsqlite3-dev, pkg-config: sqlx's sqlite feature links against libsqlite3.
# ca-certificates: cargo may fetch crates over HTTPS.
RUN apt-get update && apt-get install -y --no-install-recommends \
      protobuf-compiler \
      libsqlite3-dev \
      pkg-config \
      ca-certificates \
  && rm -rf /var/lib/apt/lists/*

# Cook dependencies first — this layer is cached unless Cargo.toml/Cargo.lock change.
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json \
      -p yuxu-server --no-default-features --features ${FEATURES}

# Copy source and build the binary. Because dependencies are already compiled,
# this step is roughly proportional to yuxu-server's own code size.
COPY . .
RUN cargo build --release -p yuxu-server --no-default-features --features ${FEATURES} \
 && strip target/release/yuxu-server

# ---- Stage 4: runtime ----
FROM debian:${DEBIAN_VERSION}-slim AS runtime

# libsqlite3-0: shared lib for the sqlite feature at run time.
# ca-certificates: needed for outbound TLS (GitHub OAuth, webhooks, ...).
# curl: healthcheck probes /health.
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates \
      libsqlite3-0 \
      curl \
  && rm -rf /var/lib/apt/lists/*

# Run as a non-root user with a stable uid so bind-mounted /data inherits
# consistent ownership across hosts.
RUN useradd -r -u 10001 -m -d /home/yuxu yuxu \
 && mkdir -p /data \
 && chown yuxu:yuxu /data

COPY --from=builder /app/target/release/yuxu-server /usr/local/bin/yuxu-server

USER yuxu
WORKDIR /data

# Defaults; override at `docker run -e ...` or via compose.
ENV YUXU_BIND=0.0.0.0:8080
ENV DATABASE_URL=sqlite:///data/yuxu.db?mode=rwc
ENV RUST_LOG=yuxu_server=info,tower_http=info

# YUXU_JWT_SECRET is required in production; the server aborts without it
# unless YUXU_DEV_MODE=1. Intentionally NOT set in the image.

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
  CMD curl -fsS http://127.0.0.1:8080/health || exit 1

CMD ["/usr/local/bin/yuxu-server"]
