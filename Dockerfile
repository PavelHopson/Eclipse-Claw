# eclipse-claw — Multi-stage Docker build
# Produces 2 binaries: eclipse-claw (CLI) and eclipse-claw-mcp (MCP server)

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
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy manifests + lock first for better layer caching.
# If only source changes, cargo doesn't re-download deps.
COPY Cargo.toml Cargo.lock ./
COPY crates/eclipse-claw-core/Cargo.toml crates/eclipse-claw-core/Cargo.toml
COPY crates/eclipse-claw-fetch/Cargo.toml crates/eclipse-claw-fetch/Cargo.toml
COPY crates/eclipse-claw-llm/Cargo.toml crates/eclipse-claw-llm/Cargo.toml
COPY crates/eclipse-claw-pdf/Cargo.toml crates/eclipse-claw-pdf/Cargo.toml
COPY crates/eclipse-claw-mcp/Cargo.toml crates/eclipse-claw-mcp/Cargo.toml
COPY crates/eclipse-claw-cli/Cargo.toml crates/eclipse-claw-cli/Cargo.toml

# Copy .cargo config if present (optional build flags)
COPY .cargo .cargo

# Create dummy source files so cargo can resolve deps and cache them.
RUN mkdir -p crates/eclipse-claw-core/src && echo "" > crates/eclipse-claw-core/src/lib.rs \
    && mkdir -p crates/eclipse-claw-fetch/src && echo "" > crates/eclipse-claw-fetch/src/lib.rs \
    && mkdir -p crates/eclipse-claw-llm/src && echo "" > crates/eclipse-claw-llm/src/lib.rs \
    && mkdir -p crates/eclipse-claw-pdf/src && echo "" > crates/eclipse-claw-pdf/src/lib.rs \
    && mkdir -p crates/eclipse-claw-mcp/src && echo "fn main() {}" > crates/eclipse-claw-mcp/src/main.rs \
    && mkdir -p crates/eclipse-claw-cli/src && echo "fn main() {}" > crates/eclipse-claw-cli/src/main.rs

# Pre-build dependencies (this layer is cached until Cargo.toml/lock changes)
RUN cargo build --release 2>/dev/null || true

# Now copy real source and rebuild. Only the final binaries recompile.
COPY crates crates
RUN touch crates/*/src/*.rs \
    && cargo build --release

# ---------------------------------------------------------------------------
# Stage 2: Minimal runtime image
# ---------------------------------------------------------------------------
FROM ubuntu:24.04

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy both binaries
COPY --from=builder /build/target/release/eclipse-claw /usr/local/bin/eclipse-claw
COPY --from=builder /build/target/release/eclipse-claw-mcp /usr/local/bin/eclipse-claw-mcp

# Default: run the CLI
CMD ["eclipse-claw"]
