---
title: "IPFS — Security & Traps"
project_note: update
type: project-note
tags: [project, IPFS]
created: 2026-07-11
status: active
---

# IPFS — Security & Traps

Security posture, known traps, and work-package boundaries for `dart_ipfs`.

## Security policy

- Vulnerability reporting: `joseeduardox@gmail.com`
- Do not open public issues before coordinated disclosure.
- Disclosure timeline: acknowledgment within 48 hours, investigation/fix 3–14 days, coordinated disclosure 15–30 days, public disclosure after patch.

## Supported versions

| Version | Supported |
|---------|-----------|
| 1.11.x | ✅ |
| 1.10.x | ✅ |
| 1.4.x | ✅ |
| 1.3.x | ✅ |
| < 1.3.0 | ❌ |

## Cryptography

- **Key storage:** AES-256-GCM encryption with PBKDF2 key derivation (100K iterations).
- **Signatures:** Ed25519 for IPNS records and peer identity; RSA and ECDSA also available.
- **Memory handling:** sensitive data (keys, seeds) are zeroed after use.
- **Crypto libraries** (since 2026-01 consolidation):
  - `crypto` — SHA-256, HMAC.
  - `cryptography` — Ed25519, AES-256-GCM, X25519 (WASM-compatible).
  - `pointycastle` — RSA, ECDSA, ASN.1 pure-Dart.
  - `cipherlib` — additional cipher implementations.
  - `catalyst_cose` — COSE signing.
  - `jose` — JOSE (JWS/JWE).
- The `sodium` FFI binding was removed to unblock `dart2wasm` compilation.

## Network security

- **RPC API:** optional API key authentication with constant-time comparison.
- **Gateway:** rate limiting (100 req/60s per IP), XSS protection, restricted CORS.
- **DHT:** Sybil attack mitigation (max 2 peers per IP), provider rate limiting.
- **Private networks:** PNET support with PSK-based encryption (XSalsa20 stream cipher).

## Data integrity

- **Block validation:** CID hash verification on all received blocks.
- **IPNS records:** Ed25519 signatures with expiration timestamps.
- **PubSub:** HMAC-SHA256 message signing.

## Content blocking

`DenylistService` provides BadBits-style compact parser, CID/multihash blocking, and 451 integration across:

- Gateway
- RPC
- DHT
- Bitswap
- MFS

Includes persistence and audit logging.

## Gateway security features

- Denylist 451 responses.
- Subdomain gateway CIDv0→CIDv1 conversion.
- DNSLink/IPNS resolution with TTL-based `Cache-Control`.
- TLS/AutoTLS via `GatewayTlsManager` with ACME v2 (Let's Encrypt).
- HTTP-01 challenge implementation via `AcmeClient`.
- Domain validation via `DomainValidator`.
- Certificate persistence via `AcmePersistence`.
- Staging environment support for testing.

## Plugin security

- Capability-based ACLs (deny-by-default).
- Ed25519 signature verification for plugin manifests.
- Audit logging for plugin activities.
- Metrics emission for capability usage.

## Dependency security overrides

```yaml
dependency_overrides:
  xml: ^7.0.1          # XML parsing vulnerabilities / encoding fixes
  dart_udx: ^2.0.3     # UDP buffer overflow / rate limiting patches
```

**Important:** These must remain as `dependency_overrides` only. v1.11.7 fixed a regression where they were accidentally promoted to direct deps, breaking downstream consumers.

## Known traps

From `ENGINEERING_NOTES.md` and `AGENTS.md`:

1. **CID import trap.** Do not replace local `lib/src/core/cid.dart` imports with `package:dart_ipfs_core/dart_ipfs_core.dart`. The umbrella CID has extra methods (`fromProto`, `toProto`, `computeForData`, `hashType`, `version`) that the core package lacks. A previous WP-07 modularization subagent broke the codebase with 231 analysis errors by ignoring this trap; recovery required restoring files from HEAD and fixing ~260 test imports.

2. **Test import trap.** Test files must use `package:dart_ipfs/src/...` imports, not relative `../../../lib/src/...` imports, to avoid library URI mismatches.

3. **Windows git restore trap.** `git show HEAD:path > file` on Windows can corrupt files to UTF-16; use `git checkout HEAD -- <path>` instead.

4. **IPNS V2 signature trap.** IPNS V2 signatures are computed over `ipns-signature:` + the raw DAG-CBOR `data` bytes (Kubo/boxo v0.40+). When verifying a decoded record, use the original serialized `data` bytes because CBOR key ordering/integer encoding must match exactly. The verifier should accept both prefixed and raw V2 signatures for interop with different record producers.

5. **Dependency override trap.** `xml` and `dart_udx` security pins must stay as `dependency_overrides`, not direct deps. `port_forwarder` constrains `xml ^6.5.0`, so promoting `xml ^7.0.1` to a direct dep breaks downstream consumers.

## Work-package status

| WP | Scope | Status |
|----|-------|--------|
| **WP-06** | AutoNAT + DCUtR + peering lifecycle integration | **Closed** (2026-07-09) |
| **WP-08** | Spec compliance: gateway handlers, UnixFS HAMT, DHT rate limiter, Gossipsub | **Closed** (2026-07-09) |
| **WP-09** | Competitor parity: IPNI, Reframe, Circuit Relay HOP/STOP | **Closed** (2026-07-09) |
| **WP-07** | Core modularization redesign | **Abandoned** by Council of Five (2026-07-09). Adoption-first strategy adopted instead. |

## Related

- [[ciel/projects/IPFS/knowledgebase.md|IPFS — Knowledgebase]]
- [[ciel/projects/IPFS/architecture.md|IPFS — Architecture]]
- [[ciel/projects/IPFS/specs-and-compliance.md|IPFS — Specs & Compliance]]
- [[ciel/projects/IPFS/build-test-ci.md|IPFS — Build, Test & CI]]
- [[ciel/projects/IPFS/dependencies-and-monorepo.md|IPFS — Dependencies & Monorepo]]
- [[ciel/projects/IPFS/git-state.md|IPFS — Git State]]
- [[ciel/kg/decisions/2026-07-09-wp07-final-decision|WP-07 final decision]]
- [[ciel/projects/Ciel/knowledgebase.md|Ciel — Knowledgebase]] (Council of Five governance)
