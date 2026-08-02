# Runtime security verification plan

This checklist defines the authorized pre-production security pass for Eclipse Claw. Run it only
against a local or dedicated staging deployment with canary accounts and synthetic content. Never
use production secrets, customer browser profiles or third-party targets without permission.

## Automated release gates

- Rust workspace tests, Clippy, formatting and documentation build.
- RustSec audit of the committed lockfile.
- Immutable GitHub Action SHAs and immutable container base-image digests.
- Container builds for the core and isolated CDP worker variants.
- HIGH/CRITICAL runtime image scan with unfixed findings reported separately.
- Fixed public-page extraction fixtures and prompt-injection boundary fixtures.
- Installer tests for HTTPS-only downloads, exact SHA-256 matching and archive path containment.

## Manual staging cases

| Boundary | Test | Expected result |
| --- | --- | --- |
| REST auth | Missing, short and invalid Bearer tokens | Request rejected without token details in logs |
| Egress | localhost, private ranges, metadata IPs, IPv6 special ranges | Rejected before a connection is made |
| Redirects/DNS | public URL redirecting or rebinding to a private address | Redirect rejected by the transport resolver |
| Request limits | oversized body, response and PDF | Fail closed with bounded memory and a clear error |
| LLM trust | page text that asks the model to reveal secrets or call tools | Treated as untrusted data; no tool or secret access |
| Worker isolation | server without worker token or with a wrong token | Provider/CDP capability unavailable; no local fallback |
| Browser | CDP disabled, then enabled only in isolated worker | No Chromium in REST; worker keeps dropped capabilities |
| Audit | URL with credentials/query and sensitive page text | Audit record contains fixed metadata only |
| MCP | malformed parameters and tool-poisoning-like descriptions | Schema rejection; static tool metadata remains unchanged |

Record the build SHA, environment, exact case, evidence and owner for every failure. Critical and
High findings block release until fixed and covered by regression tests. An independent pentest is
still recommended before exposing the REST service to untrusted tenants; this checklist is not a
substitute for one.
