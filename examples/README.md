# Examples

Practical examples showing what eclipse-claw can do. Each example is a self-contained command you can run immediately.

## Basic Extraction

```bash
# Extract as markdown (default)
eclipse-claw https://example.com

# Multiple output formats
eclipse-claw https://example.com -f markdown    # Clean markdown
eclipse-claw https://example.com -f json        # Full structured JSON
eclipse-claw https://example.com -f text        # Plain text (no formatting)
eclipse-claw https://example.com -f llm         # Token-optimized for LLMs (67% fewer tokens)

# Bare domains work (auto-prepends https://)
eclipse-claw example.com
```

## Content Filtering

```bash
# Only extract main content (skip nav, sidebar, footer)
eclipse-claw https://docs.rs/tokio --only-main-content

# Include specific CSS selectors
eclipse-claw https://news.ycombinator.com --include ".titleline,.score"

# Exclude specific elements
eclipse-claw https://example.com --exclude "nav,footer,.ads,.sidebar"

# Combine both
eclipse-claw https://docs.rs/reqwest --only-main-content --exclude ".sidebar"
```

## Brand Identity Extraction

```bash
# Extract colors, fonts, logos from any website
eclipse-claw --brand https://stripe.com
# Output: { "name": "Stripe", "colors": [...], "fonts": ["Sohne"], "logos": [...] }

eclipse-claw --brand https://github.com
# Output: { "name": "GitHub", "colors": [{"hex": "#1F2328", ...}], "fonts": ["Mona Sans"], ... }

eclipse-claw --brand wikipedia.org
# Output: 10 colors, 5 fonts, favicon, logo URL
```

## Sitemap Discovery

```bash
# Discover all URLs from a site's sitemaps
eclipse-claw --map https://sitemaps.org
# Output: one URL per line (84 URLs found)

# JSON output with metadata
eclipse-claw --map https://sitemaps.org -f json
# Output: [{ "url": "...", "last_modified": "...", "priority": 0.8 }]
```

## Recursive Crawling

```bash
# Crawl a site (default: depth 1, max 20 pages)
eclipse-claw --crawl https://example.com

# Control depth and page limit
eclipse-claw --crawl --depth 2 --max-pages 50 https://docs.rs/tokio

# Crawl with sitemap seeding (finds more pages)
eclipse-claw --crawl --sitemap --depth 2 https://docs.rs/tokio

# Filter crawl paths
eclipse-claw --crawl --include-paths "/api/*,/guide/*" https://docs.example.com
eclipse-claw --crawl --exclude-paths "/changelog/*,/blog/*" https://docs.example.com

# Control concurrency and delay
eclipse-claw --crawl --concurrency 10 --delay 200 https://example.com
```

## Change Detection (Diff)

```bash
# Step 1: Save a snapshot
eclipse-claw https://example.com -f json > snapshot.json

# Step 2: Later, compare against the snapshot
eclipse-claw --diff-with snapshot.json https://example.com
# Output:
#   Status: Same
#   Word count delta: +0

# If the page changed:
#   Status: Changed
#   Word count delta: +42
#   --- old
#   +++ new
#   @@ -1,3 +1,3 @@
#   -Old content here
#   +New content here
```

## PDF Extraction

```bash
# PDF URLs are auto-detected via Content-Type
eclipse-claw https://example.com/report.pdf

# Control PDF mode
eclipse-claw --pdf-mode auto https://example.com/report.pdf  # Error on empty (catches scanned PDFs)
eclipse-claw --pdf-mode fast https://example.com/report.pdf  # Return whatever text is found
```

## Batch Processing

```bash
# Multiple URLs in one command
eclipse-claw https://example.com https://httpbin.org/html https://rust-lang.org

# URLs from a file (one per line, # comments supported)
eclipse-claw --urls-file urls.txt

# Batch with JSON output
eclipse-claw --urls-file urls.txt -f json

# Proxy rotation for large batches
eclipse-claw --urls-file urls.txt --proxy-file proxies.txt --concurrency 10
```

## Local Files & Stdin

```bash
# Extract from a local HTML file
eclipse-claw --file page.html

# Pipe HTML from another command
curl -s https://example.com | eclipse-claw --stdin

# Chain with other tools
eclipse-claw https://example.com -f text | wc -w    # Word count
eclipse-claw https://example.com -f json | jq '.metadata.title'  # Extract title with jq
```

## Cloud API Mode

When you have a eclipse-claw API key, the CLI can route through the cloud for bot protection bypass, JS rendering, and proxy rotation.

```bash
# Set API key (one time)
export ECLIPSE_CLAW_API_KEY=wc_your_key_here

# Automatic fallback: tries local first, cloud on bot detection
eclipse-claw https://protected-site.com

# Force cloud mode (skip local, always use API)
eclipse-claw --cloud https://spa-site.com

# Cloud mode works with all features
eclipse-claw --cloud --brand https://stripe.com
eclipse-claw --cloud -f json https://producthunt.com
eclipse-claw --cloud --crawl --depth 2 https://protected-docs.com
```

## Browser Impersonation

```bash
# Chrome (default) — latest Chrome TLS fingerprint
eclipse-claw https://example.com

# Firefox fingerprint
eclipse-claw --browser firefox https://example.com

# Random browser per request (good for batch)
eclipse-claw --browser random --urls-file urls.txt
```

## Custom Headers & Cookies

```bash
# Custom headers
eclipse-claw -H "Authorization: Bearer token123" https://api.example.com
eclipse-claw -H "Accept-Language: de-DE" https://example.com

# Cookies
eclipse-claw --cookie "session=abc123; theme=dark" https://example.com

# Multiple headers
eclipse-claw -H "X-Custom: value" -H "Authorization: Bearer token" https://example.com
```

## LLM-Powered Features

These require an LLM provider (Ollama local, or OpenAI/Anthropic API key).

```bash
# Summarize a page (default: 3 sentences)
eclipse-claw --summarize https://example.com

# Control summary length
eclipse-claw --summarize 5 https://example.com

# Extract structured JSON with a schema
eclipse-claw --extract-json '{"type":"object","properties":{"title":{"type":"string"},"price":{"type":"number"}}}' https://example.com/product

# Extract with a schema from file
eclipse-claw --extract-json @schema.json https://example.com/product

# Extract with natural language prompt
eclipse-claw --extract-prompt "Get all pricing tiers with name, price, and features" https://stripe.com/pricing

# Use a specific LLM provider
eclipse-claw --llm-provider ollama --summarize https://example.com
eclipse-claw --llm-provider openai --llm-model gpt-4o --extract-prompt "..." https://example.com
eclipse-claw --llm-provider anthropic --summarize https://example.com
```

## Raw HTML Output

```bash
# Get the raw fetched HTML (no extraction)
eclipse-claw --raw-html https://example.com

# Useful for debugging extraction issues
eclipse-claw --raw-html https://example.com > raw.html
eclipse-claw --file raw.html  # Then extract locally
```

## Metadata & Verbose Mode

```bash
# Include YAML frontmatter with metadata
eclipse-claw --metadata https://example.com
# Output:
#   ---
#   title: "Example Domain"
#   source: "https://example.com"
#   word_count: 20
#   ---
#   # Example Domain
#   ...

# Verbose logging (debug extraction pipeline)
eclipse-claw -v https://example.com
```

## Proxy Usage

```bash
# Single proxy
eclipse-claw --proxy http://user:pass@proxy.example.com:8080 https://example.com

# SOCKS5 proxy
eclipse-claw --proxy socks5://proxy.example.com:1080 https://example.com

# Proxy rotation from file (one per line: host:port:user:pass)
eclipse-claw --proxy-file proxies.txt https://example.com

# Auto-load proxies.txt from current directory
echo "proxy1.com:8080:user:pass" > proxies.txt
eclipse-claw https://example.com  # Automatically detects and uses proxies.txt
```

## MCP Server (AI Agent Integration)

```bash
# Start the MCP server (stdio transport)
eclipse-claw-mcp

# Configure in Claude Desktop (~/.config/claude/claude_desktop_config.json):
# {
#   "mcpServers": {
#     "eclipse-claw": {
#       "command": "/path/to/eclipse-claw-mcp",
#       "env": {
#         "ECLIPSE_CLAW_API_KEY": "wc_your_key"  // optional, enables cloud fallback
#       }
#     }
#   }
# }

# Available tools: scrape, crawl, map, batch, extract, summarize, diff, brand, research, search
```

## Real-World Recipes

### Monitor competitor pricing

```bash
# Save today's pricing
eclipse-claw --extract-json '{"type":"array","items":{"type":"object","properties":{"plan":{"type":"string"},"price":{"type":"string"}}}}' \
  https://competitor.com/pricing -f json > pricing-$(date +%Y%m%d).json
```

### Build a documentation search index

```bash
# Crawl docs and extract as LLM-optimized text
eclipse-claw --crawl --sitemap --depth 3 --max-pages 500 -f llm https://docs.example.com > docs.txt
```

### Extract all images from a page

```bash
eclipse-claw https://example.com -f json | jq -r '.content.images[].src'
```

### Get all external links

```bash
eclipse-claw https://example.com -f json | jq -r '.content.links[] | select(.href | startswith("http")) | .href'
```

### Compare two pages

```bash
eclipse-claw https://site-a.com -f json > a.json
eclipse-claw https://site-b.com --diff-with a.json
```
