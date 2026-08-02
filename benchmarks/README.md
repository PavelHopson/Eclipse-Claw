# Reproducible benchmark gate

The release gate uses a small, reviewed fixture set under `benchmarks/public-pages/`. Its purpose
is to detect extraction and prompt-boundary regressions without downloading live pages in CI.

## What is pinned

`manifest.json` records, for every fixture:

- a stable fixture ID and page kind;
- the public HTTPS source used as a structural reference;
- the capture date;
- the exact SHA-256 of the local HTML file.

The HTML is intentionally minimal and contains no account data, session state, tracking IDs, or
third-party executable code. Fixtures cover an article, documentation, a product page and an SPA
data island.

## Release checks

```bash
node scripts/verify-benchmark-fixtures.mjs
cargo test -p eclipse-claw-core --test fixed_public_benchmark
cargo test -p eclipse-claw-llm --test fixed_security_benchmark
```

The first command rejects manifest drift or changed fixture hashes. The core test checks 17 fixed
content signals across the four page kinds. The LLM test confirms that instructions embedded in
remote HTML remain inside the untrusted-content boundary.

## What this does not prove

This suite is a regression gate, not a universal accuracy, speed, anti-bot, or token-saving
benchmark. Results from older private or live-page experiments are not release claims. A future
competitive benchmark must include a public runner, pinned input corpus, ground truth, hardware
and dependency versions, raw measurements, and a machine-readable report before numbers are
published in the main README.
