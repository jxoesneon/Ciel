---
title: "quic_lib — Specs & ADRs"
project_note: update
type: project-note
tags: ["project-note","update"]
status: active
created: 2026-07-11
updated: 2026-07-11
source: "https://github.com/jxoesneon/quic_lib"
---

# quic_lib — Specs & ADRs

Full inventory of the 22 specifications and 7 Architecture Decision Records in `doc/`.

## Architecture Decision Records (7)

| ADR | Title | Decision | Rationale |
|-----|-------|----------|-----------|
| ADR-001 | Pure Dart, No FFI | Build without `dart:ffi` for broader portability | Maximizes platform reach; accepts perf trade-off. Blocks ECN support. |
| ADR-002 | NewReno Before CUBIC | Implement NewReno first, CUBIC later | NewReno is simpler; CUBIC added in v1.x. |
| ADR-003 | Soft-Fail Certificate Revocation | Default soft-fail policy for P2P/mobile | Avoids blocking connectivity when OCSP/CRL endpoints are unreachable. |
| ADR-004 | Cryptography Primary Crypto Backend | Use `package:cryptography` with `package:pointycastle` fallback | Platform acceleration via cryptography; pointycastle for ASN1/RSA. |
| ADR-005 | Three-Tier Documentation | Organize docs into `research/`, `specs/`, `architecture/` | Separates exploration from normative specs from design docs. |
| ADR-006 | Stream Scheduler Pluggable Interface | Pluggable interface with round-robin default | Allows custom schedulers without modifying core. |
| ADR-007 | Isolate-per-Connection Architecture | Each QUIC connection in its own isolate | Parallelism for 50+ concurrent connections. |

## Specifications (22)

### Core QUIC specs

| Spec | RFC | Status | Key content |
|------|-----|--------|-------------|
| `QUIC_WIRE_SPEC.md` | RFC 9000 | Complete | Varint encoding, long/short headers, 20+ frame types, packet coalescing |
| `QUIC_CRYPTO_SPEC.md` | RFC 9001 | Complete | TLS 1.3 integration, encryption levels (Initial/0-RTT/Handshake/1-RTT), HKDF, AEAD, header protection, key updates, retry integrity, 0-RTT |
| `QUIC_STREAMS_SPEC.md` | RFC 9000 | Complete | 62-bit stream IDs, send/receive state machines, credit-based flow control, reassembly, priority scheduling |
| `QUIC_RECOVERY_SPEC.md` | RFC 9002 | Complete | RTT estimation, sent packet tracking, loss detection (packet + time threshold), PTO, congestion control, persistent congestion |
| `QUIC_TRANSPORT_PARAMETERS_SPEC.md` | RFC 9000 | Complete | 15+ transport parameters, validation rules, defaults, Dart API |

### Extension specs

| Spec | RFC | Status | Key content |
|------|-----|--------|-------------|
| `QUIC_DATAGRAM_SPEC.md` | RFC 9221 | Complete | DATAGRAM frames (0x30/0x31), `max_datagram_frame_size` parameter, unreliable/unordered semantics |
| `CUBIC_SPEC.md` | RFC 9369 | Complete | CUBIC algorithm, state variables, window growth formula |
| `HTTP3_SPEC.md` | RFC 9114 | Partial | Stream mapping, HTTP/3 frames, QPACK integration, priority (RFC 9218), error handling |
| `QPACK_SPEC.md` | RFC 9204 | Partial | Static table (99 entries), dynamic table, encoder/decoder instructions |
| `WEBTRANSPORT_SPEC.md` | RFC 9220 | Complete | Extended CONNECT, session establishment, stream dispatch, datagrams, CLOSE/DRAIN capsules |
| `DCUTR_SPEC.md` | libp2p | Complete | Protobuf schema, Initiator/Responder roles, state machines, UDP hole-punching |
| `LIBP2P_QUIC_SPEC.md` | libp2p | Complete | Multiaddr, TLS 1.3 peer auth (libp2p Public Key Extension, OID 1.3.6.1.4.1.53594.1.1), ALPN "libp2p", multistream-select |

### API and integration specs

| Spec | Status | Key content |
|------|--------|-------------|
| `DART_API_SPEC.md` | Complete | Public Dart API surface, idiomatic Dart principles, layered exposure, zero native deps, async-first |
| `ERROR_REGISTRY.md` | Complete | Unified error codes (QUIC Transport, HTTP/3, Application-defined) |

### Quality and process specs

| Spec | Status | Key content |
|------|--------|-------------|
| `TESTING_SPEC.md` | Complete | Multi-layer strategy (unit, integration, interop, fuzz), QUIC Interop Runner, target implementations |
| `FUZZING_SPEC.md` | Complete | Fuzz targets (packet parser, frame parser, crypto, stream state, HTTP/3), methodology, corpus |
| `TEST_VECTORS.md` | Complete | RFC test vectors, Initial secret derivation, packet protection examples, varint pairs |
| `PERFORMANCE_BENCHMARKING.md` | Complete | Micro/macro benchmarks, methodology, baselines (handshake <50ms, throughput >100MB/s, crypto >200k pkt/sec) |
| `SECURITY_SPEC.md` | Complete | Threat model, TLS 1.3 requirements, certificate handling, amplification protection, replay protection, DoS limits |
| `VERSIONING_POLICY.md` | Complete | SemVer 2.0.0, API stability by phase, deprecation policy, release process, dart_ipfs compatibility |
| `ROADMAP.md` | Complete | 6-phase plan: Phase 0 (Spec - current), Phase 1 (Core QUIC), Phase 2 (HTTP/3), Phase 3 (WebTransport), Phase 4 (libp2p), Phase 5 (Hardening), Phase 6 (dart_ipfs) |

## Architecture documents (6)

| Document | Content |
|----------|---------|
| `MODULE_OVERVIEW.md` | High-level architecture, layer/component responsibility tables |
| `DART_IPFS_INTEGRATION.md` | Integration contract between quic_lib and dart_ipfs |
| `API_SURFACE.md` | Refers to `DART_API_SPEC.md` for authoritative API |
| `CRYPTO_ABSTRACTION.md` | `CryptoBackend` interface design |
| `DATA_FLOW.md` | Send/receive data path visualizations |
| `RFC_NOTES.md` | Consolidated notes from RFCs 9000, 9001, 9002 |

## Other documentation

| Document | Content |
|----------|---------|
| `DOC_STANDARDS.md` | Documentation standards, doc comments, cross-referencing, RFC citation guidelines |
| `EXTENSION_GUIDE.md` | Process for adding new protocol extensions with templates |
| `POINTYCASTLE_4_MIGRATION.md` | Low migration risk to pointycastle 4.0.0 (only ASN1 parsing for RSA keys) |
| `WEB_AND_WASM.md` | Native-only; no web/WASM (browsers block raw UDP). Recommends WebTransport API or WebRTC for browsers. |

## ROADMAP status

- **Phase 0 (Specification):** Current phase. All 22 specs complete, awaiting external review.
- **Phases 1-6:** Planned (8-12 weeks for Phase 1 Core QUIC).
- **v1.x criteria:** All complete as of v1.13.0.
- **Next major:** v2.0.0 (ECN support, performance optimizations).

## Related

- [[ciel/projects/quic_lib/quic_lib.md|quic_lib overview]]
- [[ciel/projects/quic_lib/knowledgebase.md|quic_lib knowledgebase]]
- [[ciel/projects/quic_lib/architecture.md|Architecture & Subsystems]]
- [[ciel/projects/quic_lib/security-and-audits.md|Security & Audits]]
