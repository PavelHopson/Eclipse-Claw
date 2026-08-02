# Camofox worker adapter boundary

Eclipse Claw does not install `camofox-browser` and does not use browser rendering as an invisible
fallback. The built-in HTTP extractor remains the first and default path. A browser worker is an
optional capability for a confirmed JS-heavy public page only.

## Minimum adapter contract

- Run in a separate container without the Eclipse workspace, SSH agent, cloud credentials or host
  browser profile.
- Bind to loopback or an internal container network and require a dedicated high-entropy worker
  token.
- Accept only `https://` URLs whose normalized hostname matches an explicit domain allowlist.
- Reject localhost, private/link-local IP ranges, URL credentials, redirects to blocked networks,
  cookie import and uploaded browser profiles.
- Set `CAMOFOX_CRASH_REPORT_ENABLED=false`. Do not forward crash payloads to an external endpoint.
- Return a bounded accessibility snapshot and final URL. Treat both as untrusted data that cannot
  add tools, change the plan or request secrets.
- Provide read-only navigation only. Payments, publishing, messages, account changes, downloads and
  file uploads are outside this adapter.

## Routing

1. Try the local HTTP extractor.
2. If the result is insufficient, explain why the page is considered JS-heavy.
3. Require explicit browser capability configuration and an allowlisted domain.
4. Execute in the isolated worker and pass only its bounded snapshot to the planner.
5. Fail closed. Never fall through to a logged-in host browser or transfer cookies.

The community REST wrapper and Camoufox itself remain separate upstream projects with different
licenses and security defaults. Before adding either dependency, pin the exact version, verify the
license and lockfile, audit install scripts and container contents, and repeat the browser-agent
prompt-injection review.
