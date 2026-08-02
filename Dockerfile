# eclipse-claw — Multi-stage Docker build
# Produces CLI, MCP, REST server and isolated worker binaries.

# ---------------------------------------------------------------------------
# Stage 1: Build all binaries in release mode
# ---------------------------------------------------------------------------
FROM rust:1.93-bookworm AS builder

# Build dependencies: cmake + clang for BoringSSL (wreq), pkg-config for linking
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    cmake \
    clang \
    nasm \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy manifests + lock first for better layer caching.
# If only source changes, cargo doesn't re-download deps.
COPY Cargo.toml Cargo.lock ./
COPY crates/eclipse-claw-audit/Cargo.toml crates/eclipse-claw-audit/Cargo.toml
COPY crates/eclipse-claw-cdp/Cargo.toml crates/eclipse-claw-cdp/Cargo.toml
COPY crates/eclipse-claw-connectors/Cargo.toml crates/eclipse-claw-connectors/Cargo.toml
COPY crates/eclipse-claw-core/Cargo.toml crates/eclipse-claw-core/Cargo.toml
COPY crates/eclipse-claw-fetch/Cargo.toml crates/eclipse-claw-fetch/Cargo.toml
COPY crates/eclipse-claw-llm/Cargo.toml crates/eclipse-claw-llm/Cargo.toml
COPY crates/eclipse-claw-pdf/Cargo.toml crates/eclipse-claw-pdf/Cargo.toml
COPY crates/eclipse-claw-mcp/Cargo.toml crates/eclipse-claw-mcp/Cargo.toml
COPY crates/eclipse-claw-cli/Cargo.toml crates/eclipse-claw-cli/Cargo.toml
COPY crates/eclipse-claw-server/Cargo.toml crates/eclipse-claw-server/Cargo.toml
COPY crates/eclipse-claw-worker/Cargo.toml crates/eclipse-claw-worker/Cargo.toml

# Copy .cargo config if present (optional build flags)
COPY .cargo .cargo

# Create dummy source files so cargo can resolve deps and cache them.
RUN mkdir -p crates/eclipse-claw-audit/src && echo "" > crates/eclipse-claw-audit/src/lib.rs \
    && mkdir -p crates/eclipse-claw-cdp/src && echo "" > crates/eclipse-claw-cdp/src/lib.rs \
    && mkdir -p crates/eclipse-claw-connectors/src && echo "" > crates/eclipse-claw-connectors/src/lib.rs \
    && mkdir -p crates/eclipse-claw-core/src && echo "" > crates/eclipse-claw-core/src/lib.rs \
    && mkdir -p crates/eclipse-claw-fetch/src && echo "" > crates/eclipse-claw-fetch/src/lib.rs \
    && mkdir -p crates/eclipse-claw-llm/src && echo "" > crates/eclipse-claw-llm/src/lib.rs \
    && mkdir -p crates/eclipse-claw-pdf/src && echo "" > crates/eclipse-claw-pdf/src/lib.rs \
    && mkdir -p crates/eclipse-claw-mcp/src && echo "fn main() {}" > crates/eclipse-claw-mcp/src/main.rs \
    && mkdir -p crates/eclipse-claw-cli/src && echo "fn main() {}" > crates/eclipse-claw-cli/src/main.rs \
    && mkdir -p crates/eclipse-claw-server/src && echo "fn main() {}" > crates/eclipse-claw-server/src/main.rs \
    && mkdir -p crates/eclipse-claw-worker/src && echo "fn main() {}" > crates/eclipse-claw-worker/src/main.rs

# Pre-build dependencies (this layer is cached until Cargo.toml/lock changes)
RUN cargo build --release 2>/dev/null || true

# Now copy real source and rebuild. Only the final binaries recompile.
COPY crates crates
RUN touch crates/*/src/*.rs \
    && cargo build --release

# ---------------------------------------------------------------------------
# Stage 2: Minimal runtime image
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --gid 65532 eclipse \
    && useradd --uid 65532 --gid 65532 --create-home --home-dir /home/eclipse eclipse \
    && mkdir -p /var/lib/eclipse-claw/audit/server \
        /var/lib/eclipse-claw/audit/llm-worker \
        /var/lib/eclipse-claw/audit/cdp-worker \
    && chown -R 65532:65532 /var/lib/eclipse-claw /home/eclipse

# Copy both binaries
COPY --from=builder /build/target/release/eclipse-claw /usr/local/bin/eclipse-claw
COPY --from=builder /build/target/release/eclipse-claw-mcp /usr/local/bin/eclipse-claw-mcp
COPY --from=builder /build/target/release/eclipse-claw-server /usr/local/bin/eclipse-claw-server
COPY --from=builder /build/target/release/eclipse-claw-worker /usr/local/bin/eclipse-claw-worker

WORKDIR /work
USER 65532:65532

# Default: run the CLI
CMD ["eclipse-claw"]

# Optional browser-worker image. Chromium runs only in this separate,
# capability-dropped container and never inside eclipse-claw-server.
FROM runtime AS cdp-worker
USER root
RUN apt-get update && apt-get install -y --no-install-recommends \
    chromium \
    fonts-liberation \
    && rm -rf /var/lib/apt/lists/*
USER 65532:65532
CMD ["eclipse-claw-worker", "--mode", "cdp"]
