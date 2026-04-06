# Eclipse Claw

Rust workspace: CLI + MCP server for web content extraction into LLM-optimized formats.

## Architecture

```
eclipse-claw/
  crates/
    eclipse-claw-core/     # Pure extraction engine. WASM-safe. Zero network deps.
                      # + ExtractionOptions (include/exclude CSS selectors)
                      # + diff engine (change tracking)
                      # + brand extraction (DOM/CSS analysis)
    eclipse-claw-fetch/    # HTTP client via primp. Crawler. Sitemap discovery. Batch ops.
                      # + proxy pool rotation (per-request)
                      # + PDF content-type detection
                      # + document parsing (DOCX, XLSX, CSV)
    eclipse-claw-llm/      # LLM provider chain (Ollama -> OpenAI -> Anthropic)
                      # + JSON schema extraction, prompt extraction, summarization
    eclipse-claw-pdf/      # PDF text extraction via pdf-extract
    eclipse-claw-mcp/      # MCP server (Model Context Protocol) for AI agents
    eclipse-claw-cli/      # CLI binary
```

Two binaries: `eclipse-claw` (CLI), `eclipse-claw-mcp` (MCP server).

### Core Modules (`eclipse-claw-core`)
- `extractor.rs` — Readability-style scoring: text density, semantic tags, link density penalty
- `noise.rs` — Shared noise filter: tags, ARIA roles, class/ID patterns. Tailwind-safe.
- `data_island.rs` — JSON data island extraction for React SPAs, Next.js, Contentful CMS
- `markdown.rs` — HTML to markdown with URL resolution, asset collection
- `llm.rs` — 9-step LLM optimization pipeline (image strip, emphasis strip, link dedup, stat merge, whitespace collapse)
- `domain.rs` — Domain detection from URL patterns + DOM heuristics
- `metadata.rs` — OG, Twitter Card, standard meta tag extraction
- `types.rs` — Core data structures (ExtractionResult, Metadata, Content)
- `filter.rs` — CSS selector include/exclude filtering (ExtractionOptions)
- `diff.rs` — Content change tracking engine (snapshot diffing)
- `brand.rs` — Brand identity extraction from DOM structure and CSS

### Fetch Modules (`eclipse-claw-fetch`)
- `client.rs` — FetchClient with primp TLS impersonation
- `browser.rs` — Browser profiles: Chrome (142/136/133/131), Firefox (144/135/133/128)
- `crawler.rs` — BFS same-origin crawler with configurable depth/concurrency/delay
- `sitemap.rs` — Sitemap discovery and parsing (sitemap.xml, robots.txt)
- `batch.rs` — Multi-URL concurrent extraction
- `proxy.rs` — Proxy pool with per-request rotation
- `document.rs` — Document parsing: DOCX, XLSX, CSV auto-detection and extraction
- `search.rs` — Web search via Serper.dev with parallel result scraping

### LLM Modules (`eclipse-claw-llm`)
- Provider chain: Ollama (local-first) -> OpenAI -> Anthropic
- JSON schema extraction, prompt-based extraction, summarization

### PDF Modules (`eclipse-claw-pdf`)
- PDF text extraction via pdf-extract crate

### MCP Server (`eclipse-claw-mcp`)
- Model Context Protocol server over stdio transport
- 8 tools: scrape, crawl, map, batch, extract, summarize, diff, brand
- Works with Claude Desktop, Claude Code, and any MCP client
- Uses `rmcp` crate (official Rust MCP SDK)

## Hard Rules

- **Core has ZERO network dependencies** — takes `&str` HTML, returns structured output. Keep it WASM-compatible.
- **primp requires `[patch.crates-io]`** for patched rustls/h2 forks at workspace level.
- **RUSTFLAGS are set in `.cargo/config.toml`** — no need to pass manually.
- **eclipse-claw-llm uses plain reqwest** (NOT primp-patched). LLM APIs don't need TLS fingerprinting.
- **qwen3 thinking tags** (`<think>`) are stripped at both provider and consumer levels.

## Build & Test

```bash
cargo build --release           # Both binaries
cargo test --workspace          # All tests
cargo test -p eclipse-claw-core      # Core only
cargo test -p eclipse-claw-llm       # LLM only
```

## CLI

```bash
# Basic extraction
eclipse-claw https://example.com
eclipse-claw https://example.com --format llm

# Content filtering
eclipse-claw https://example.com --include "article" --exclude "nav,footer"
eclipse-claw https://example.com --only-main-content

# Batch + proxy rotation
eclipse-claw url1 url2 url3 --proxy-file proxies.txt
eclipse-claw --urls-file urls.txt --concurrency 10

# Sitemap discovery
eclipse-claw https://docs.example.com --map

# Crawling (with sitemap seeding)
eclipse-claw https://docs.example.com --crawl --depth 2 --max-pages 50 --sitemap

# Change tracking
eclipse-claw https://example.com -f json > snap.json
eclipse-claw https://example.com --diff-with snap.json

# Brand extraction
eclipse-claw https://example.com --brand

# LLM features (Ollama local-first)
eclipse-claw https://example.com --summarize
eclipse-claw https://example.com --extract-prompt "Get all pricing tiers"
eclipse-claw https://example.com --extract-json '{"type":"object","properties":{"title":{"type":"string"}}}'

# PDF (auto-detected via Content-Type)
eclipse-claw https://example.com/report.pdf

# Browser impersonation: chrome (default), firefox, random
eclipse-claw https://example.com --browser firefox

# Local file / stdin
eclipse-claw --file page.html
cat page.html | eclipse-claw --stdin
```

## Key Thresholds

- Scoring minimum: 50 chars text length
- Semantic bonus: +50 for `<article>`/`<main>`, +25 for content class/ID
- Link density: >50% = 0.1x score, >30% = 0.5x
- Data island fallback triggers when DOM word count < 30
- Eyebrow text max: 80 chars

## MCP Setup

Add to Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "eclipse-claw": {
      "command": "/path/to/eclipse-claw-mcp"
    }
  }
}
```

## Skills

- `/scrape <url>` — extract content from a URL
- `/benchmark [url]` — run extraction performance benchmarks
- `/research <url>` — deep web research via crawl + extraction
- `/crawl <url>` — crawl a website
- `/commit` — conventional commit with change analysis

## Git

- Remote: `git@github.com:PavelHopson/eclipse-claw.git`
- Use `/commit` skill for commits
