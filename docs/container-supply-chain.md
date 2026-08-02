# Container supply-chain policy

Eclipse Claw treats container tags as readable labels, not as integrity controls. Every external
`FROM` image in `Dockerfile` and `Dockerfile.ci` must include an immutable multi-platform
`sha256` digest. `docker-compose.yml` also refuses to start Ollama unless `OLLAMA_IMAGE` is set to
an explicit digest.

## Current pins

Verified on 2026-08-02 against the official Docker Hub library pages:

| Image | Multi-platform digest |
| --- | --- |
| `rust:1.93-bookworm` | `sha256:7c4ae649a84014c467d79319bbf17ce2632ae8b8be123ac2fb2ea5be46823f31` |
| `debian:bookworm-slim` | `sha256:7b140f374b289a7c2befc338f42ebe6441b7ea838a042bbd5acbfca6ec875818` |

Sources: [official Rust image](https://hub.docker.com/_/rust) and
[official Debian image](https://hub.docker.com/_/debian).

## Updating a pin

1. Read the upstream image release notes and confirm the tag is still the intended distro/toolchain.
2. Resolve the multi-platform index, not one architecture manifest:

   ```bash
   docker buildx imagetools inspect rust:1.93-bookworm
   docker buildx imagetools inspect debian:bookworm-slim
   ```

3. Replace the tag and digest together in every Dockerfile.
4. Run `node scripts/verify-container-supply-chain.mjs` and build both `core` and `cdp` stages.
5. Review the image scan. Do not ignore a finding without an expiry, owner and documented reason.

Digest pinning prevents a tag from silently changing. It does not freeze packages installed by
`apt-get`, prove that an image is vulnerability-free, or replace release provenance. Rust
dependencies remain covered by the committed `Cargo.lock` and RustSec audit; runtime OS layers are
scanned separately in CI.
