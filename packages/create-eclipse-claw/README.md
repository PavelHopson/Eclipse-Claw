<p align="center">
  <a href="https://webclaw.io">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/PavelHopson/eclipse-claw/main/.github/banner.png" />
      <img src="https://raw.githubusercontent.com/PavelHopson/eclipse-claw/main/.github/banner.png" alt="eclipse-claw" width="700" />
    </picture>
  </a>
</p>

<h3 align="center">
  One command to give your AI agent reliable web access.<br/>
  <sub>Fast HTTP extraction first. Isolated browser worker only when explicitly enabled.</sub>
</h3>

<p align="center">
  <a href="https://www.npmjs.com/package/create-eclipse-claw"><img src="https://img.shields.io/npm/dt/create-eclipse-claw?style=for-the-badge&logo=npm&logoColor=white&label=Installs&color=CB3837" alt="npm installs" /></a>
  <a href="https://github.com/PavelHopson/eclipse-claw"><img src="https://img.shields.io/github/stars/PavelHopson/eclipse-claw?style=for-the-badge&logo=github&logoColor=white&label=Stars&color=181717" alt="Stars" /></a>
  <a href="https://github.com/PavelHopson/eclipse-claw/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-AGPL--3.0-10B981?style=for-the-badge" alt="License" /></a>
</p>

---

## Quick Start

```bash
npx create-eclipse-claw
```

The installer shows the detected tools, asks before changing them, downloads a release archive,
verifies its SHA-256, and preserves the original config in a one-time `.eclipse-claw.bak` backup.
No API key is required for local extraction.

Works with **Claude Desktop**, **Claude Code**, **Cursor**, **Windsurf**, **VS Code**, **OpenCode**, **Codex CLI**, and **Antigravity**.

---

## The Problem

Your AI agent calls `fetch()` and may get a 403 because some sites compare TLS, HTTP/2 and other
request signals before returning content.

When it does work, you get 100KB+ of raw HTML — navigation, ads, cookie banners, scripts. Your agent burns 4,000+ tokens parsing noise.

## The Fix

eclipse-claw can use browser-like TLS and HTTP/2 profiles when a normal HTTP client is rejected.
Results depend on the target, its policy, rate limits and current anti-bot rules; no universal
bypass rate is promised.

It then removes common navigation and page noise and returns compact markdown. Token savings vary
by page and model tokenizer.

```
                     Raw HTML                          eclipse-claw
┌──────────────────────────────────┐    ┌──────────────────────────────────┐
│ <div class="ad-wrapper">         │    │ # Breaking: AI Breakthrough      │
│ <nav class="global-nav">         │    │                                  │
│ <script>window.__NEXT_DATA__     │    │ Researchers achieved 94%         │
│ ={...8KB of JSON...}</script>    │    │ accuracy on cross-domain         │
│ <div class="social-share">       │    │ reasoning benchmarks.            │
│ <!-- 142,847 characters -->      │    │                                  │
│                                  │    │ ## Key Findings                  │
│         4,820 tokens             │    │         1,590 tokens             │
└──────────────────────────────────┘    └──────────────────────────────────┘
```

---

## What It Does

```bash
npx create-eclipse-claw
```

1. Detects installed AI tools (Claude, Cursor, Windsurf, VS Code, OpenCode, Codex, Antigravity).
2. Downloads the latest macOS/Linux release archive and verifies it against the release
   `SHA256SUMS` file. Other platforms use a tag-pinned, locked Cargo build when Rust is installed.
3. Asks for your API key (optional — **local extraction works without one**).
4. Refuses malformed JSON, keeps a one-time local backup, writes each config atomically and reports
   every changed path. If you enter an API key, it is stored in those local MCP configs.

## 11 MCP Tools

After setup, your AI agent has access to:

| Tool | What it does | API key needed? |
|------|-------------|-----------------|
| **scrape** | Extract content from any URL | No |
| **crawl** | Recursively crawl a website | No |
| **search** | Web search + parallel scrape | Yes (Serper) |
| **map** | Discover URLs from sitemaps | No |
| **batch** | Extract multiple URLs in parallel | No |
| **extract** | LLM-powered structured extraction | Yes |
| **summarize** | Content summarization | Yes |
| **diff** | Track content changes | No |
| **brand** | Extract brand identity | No |
| **research** | Deep multi-page research | Yes |
| **doctor** | Read-only connector readiness, data boundaries, and fallback policy | No |

**9 of 11 tools can run without Eclipse Cloud.** Run `doctor` before research to see
which connectors are ready and whether automatic cloud transfer is explicitly enabled.

## Supported Tools

| Tool | Config location |
|------|----------------|
| Claude Desktop | `~/Library/Application Support/Claude/claude_desktop_config.json` |
| Claude Code | `~/.claude.json` |
| Cursor | `.cursor/mcp.json` |
| Windsurf | `~/.codeium/windsurf/mcp_config.json` |
| VS Code (Continue) | `~/.continue/config.json` |
| OpenCode | `~/.opencode/config.json` |
| Codex CLI | `~/.codex/config.json` |
| Antigravity | `~/.antigravity/mcp.json` |

## What to expect

Start with public pages you are allowed to access. Some sites require the optional isolated browser
worker, authenticated sessions or stricter rate limits; others prohibit automated access. Run the
`doctor` tool to see which connectors and data boundaries are active before a research task.

## Alternative Install Methods

### Homebrew

A public Homebrew tap is not available yet. Use the verified release archives until it is published.

### Docker

```bash
docker run --rm --read-only --cap-drop=ALL \
  ghcr.io/pavelhopson/eclipse-claw:v0.4.2 https://example.com
```

### Cargo

```bash
cargo install --git https://github.com/PavelHopson/eclipse-claw.git \
  --tag v0.4.2 --locked eclipse-claw-cli
```

### Prebuilt Binaries

Download from [GitHub Releases](https://github.com/PavelHopson/eclipse-claw/releases) for macOS (arm64, x86_64) and Linux (x86_64, aarch64).

---

## Links

- [Website](https://webclaw.io)
- [Documentation](https://webclaw.io/docs)
- [GitHub](https://github.com/PavelHopson/eclipse-claw)
- [TLS Library](https://github.com/PavelHopson/eclipse-claw-tls)
- [Discord](https://discord.gg/KDfd48EpnW)
- [Status](https://status.webclaw.io)

## License

AGPL-3.0
