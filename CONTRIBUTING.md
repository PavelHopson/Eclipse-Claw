# Contributing to Eclipse Claw

Thanks for your interest in contributing. This document covers the essentials.

## Development Setup

1. Install Rust 1.85+ (edition 2024 required):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Clone and build:
   ```bash
   git clone https://github.com/PavelHopson/eclipse-claw.git
   cd eclipse-claw
   cargo build --release
   ```

   RUSTFLAGS are configured in `.cargo/config.toml` -- no manual flags needed.

3. Optional: run `./setup.sh` for environment bootstrapping.

## Running Tests

```bash
cargo test --workspace          # All crates
cargo test -p eclipse-claw-core      # Single crate
```

## Linting

```bash
cargo clippy --all -- -D warnings
cargo fmt --check --all
```

Both must pass cleanly before submitting a PR.

## Code Style

- Rust edition 2024, formatted with `rustfmt` (see `rustfmt.toml`, `style_edition = "2024"`)
- `eclipse-claw-core` has zero network dependencies -- keep it WASM-safe
- `eclipse-claw-llm` uses plain `reqwest` — LLM APIs don't need TLS fingerprinting
- Prefer returning `Result` over panicking. No `.unwrap()` on untrusted input.
- Doc comments on all public items. Explain *why*, not *what*.

## Pull Request Process

1. Fork the repository and create a feature branch:
   ```bash
   git checkout -b feat/my-feature
   ```

2. Make your changes. Write tests for new functionality.

3. Ensure all checks pass:
   ```bash
   cargo test --workspace
   cargo clippy --all -- -D warnings
   cargo fmt --check --all
   ```

4. Push and open a pull request against `main`.

5. PRs require review before merging. Keep changes focused -- one concern per PR.

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add PDF table extraction
fix: handle malformed sitemap XML gracefully
refactor: simplify crawler BFS loop
docs: update MCP setup instructions
test: add glob_match edge cases
chore: bump dependencies
```

Use the imperative mood ("add", not "added"). Keep the subject under 72 characters.
Body is optional but encouraged for non-trivial changes.

## Reporting Issues

- Search existing issues before opening a new one
- Include: Rust version, OS, steps to reproduce, expected vs actual behavior
- For extraction bugs: include the URL (or HTML snippet) and the output format used
- Security issues: email directly instead of opening a public issue

## Architecture

```
eclipse-claw (this repo)
├── crates/
│   ├── eclipse-claw-core/    # Pure extraction engine (HTML → markdown/json/text)
│   ├── eclipse-claw-fetch/   # HTTP client + crawler + sitemap + batch
│   ├── eclipse-claw-llm/     # LLM provider chain (Ollama → OpenAI → Anthropic)
│   ├── eclipse-claw-pdf/     # PDF text extraction
│   ├── eclipse-claw-cli/     # CLI binary
│   └── eclipse-claw-mcp/     # MCP server binary
│
└── [patch.crates-io]    # Points to eclipse-claw-tls for TLS fingerprinting
```

TLS fingerprinting lives in a separate repo: [eclipse-claw-tls](https://github.com/PavelHopson/eclipse-claw-tls). The `[patch.crates-io]` section in `Cargo.toml` overrides rustls, h2, hyper, hyper-util, and reqwest with our patched forks for browser-grade JA4 + HTTP/2 Akamai fingerprinting.

## Crate Boundaries

Changes that cross crate boundaries need extra care:

| Crate | Network? | Key constraint |
|-------|----------|----------------|
| eclipse-claw-core | No | Zero network deps, WASM-safe |
| eclipse-claw-fetch | Yes (eclipse-claw-http) | Uses [eclipse-claw-tls](https://github.com/PavelHopson/eclipse-claw-tls) for TLS fingerprinting |
| eclipse-claw-llm | Yes (reqwest) | Plain reqwest — LLM APIs don't need TLS fingerprinting |
| eclipse-claw-pdf | No | Minimal, wraps pdf-extract |
| eclipse-claw-cli | Yes | Depends on all above |
| eclipse-claw-mcp | Yes | MCP server via rmcp |
