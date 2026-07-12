---
title: "quic_lib — Security & Audits"
project_note: update
type: project-note
tags: ["project-note","update"]
status: active
created: 2026-07-11
updated: 2026-07-11
source: "https://github.com/jxoesneon/quic_lib"
---

# quic_lib — Security & Audits

Security posture, audit history, and hardening measures for quic_lib v1.13.0.

## Current posture

**Zero remaining findings** after 3 blue team + 2 red team audit loops. 30 security fixes applied across the codebase.

## Audit reports (5)

### Blue Team (defensive)

| Report | Findings | Status |
|--------|----------|--------|
| `SECURITY_AUDIT_BLUE_TEAM_DOS.md` | 21 findings: unbounded memory, flow control, replay protection, rate limiting | All fixed |
| `SECURITY_AUDIT_BLUE_TEAM_META.md` | Meta-analysis: zero systemic findings after fixes | Clean |
| `SECURITY_AUDIT_BLUE_TEAM_V2.md` | 12 new findings: CryptoFrameAssembler, uncaught exceptions, state-machine validation | All fixed |
| `SECURITY_AUDIT_BLUE_TEAM_V3.md` | Zero remaining findings | Clean |

### Red Team (offensive)

| Report | Findings | Status |
|--------|----------|--------|
| `SECURITY_AUDIT_RED_TEAM_FUZZ.md` | Integer overflow/DoS, information disclosure, TOCTOU, race conditions | All fixed |
| `SECURITY_AUDIT_RED_TEAM_NOVEL.md` | Timing side channels, partial frame injection, `toString()` disclosure | All fixed |
| `SECURITY_AUDIT_RED_TEAM_V2.md` | Zero remaining findings after 30 security fixes | Clean |

### Other

| Report | Content |
|--------|---------|
| `CIEL_COUNCIL_PROTOCOL_AUDIT_2026.md` | Historical snapshot of protocol completeness audit, RFC coverage status |

## Hardening measures

### Rate limiting
- **State transition rate limit:** 100/sec max via sliding-window `RateLimiter`.
- **UDP send rate limiting:** `UdpRateLimiter` in I/O subsystem.

### Anti-amplification
- RFC 9000 3x receive limit enforced via `AntiAmplificationLimit` before address validation.

### Resource caps
- **Tracked packets:** Max 10,000 in `LossDetector` (DoS protection).
- **Flow-control windows:** Max 256 MB (integer overflow protection).
- **Active connection IDs:** Max 8.
- **Retired CID history:** Max 32 for duplicate detection.

### Crypto hardening
- Constant-time AEAD tag comparison, HMAC, and signature verification.
- No logging of secrets or raw payloads.
- Soft-fail revocation policy (ADR-003) for P2P/mobile — does not block connectivity when OCSP/CRL endpoints unreachable.

### Replay protection (0-RTT)
- Idempotent-only 0-RTT.
- Single-use session tickets.
- Time window validation.
- Application awareness of replay risk.

### DoS limits
- `maxConnections`, `handshakeTimeout`, `maxStreamResetRate`, `maxMemoryPerConnection`.

### Stateless reset
- `HMAC-SHA256(static_key, connection_id)[0..16]` — 16-byte token.

### Downgrade protection
- Version negotiation enforced via `VersionInformation` (RFC 9368).

## Threat model (from SECURITY_SPEC.md)

- **Passive attackers:** Defeated by TLS 1.3 encryption (AEAD + header protection).
- **Active attackers:** Defeated by TLS 1.3 handshake authentication, anti-amplification, address validation.
- **Off-path attackers:** Defeated by connection IDs, stateless reset tokens, retry integrity.

## Known limitations

- **ECN:** Not enforced at the socket level (deferred to v2.0.0, Issue #10). ECN counters and validation state exist in code but cannot set `IP_TOS`/`IPV6_TCLASS` from pure Dart.
- **Pure-Dart crypto:** Slower than native implementations (accepted trade-off per ADR-001).

## Related

- [[ciel/projects/quic_lib/quic_lib.md|quic_lib overview]]
- [[ciel/projects/quic_lib/knowledgebase.md|quic_lib knowledgebase]]
- [[ciel/projects/quic_lib/architecture.md|Architecture & Subsystems]]
- [[ciel/projects/quic_lib/specs-and-adrs.md|Specs & ADRs]]
