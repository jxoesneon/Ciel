---
title: quic_lib — Knowledgebase
project_note: knowledgebase
type: project-note
tags: ["project-note","knowledgebase"]
status: active
created: 2026-07-09
updated: 2026-07-11
source: "https://github.com/jxoesneon/quic_lib"
---

# quic_lib — Knowledgebase

Synthesized from a full deep-mine of the local clone at `C:/Users/josee/dart_quic` (v1.13.0, 2026-07-11).

## Summary

`quic_lib` is a mature, pure-Dart implementation of QUIC (RFC 9000/9001/9002), HTTP/3 (RFC 9114), WebTransport (RFC 9220), and the libp2p QUIC transport. It is the final v1.x release (v1.13.0) and the primary transport dependency for `dart_ipfs`. The project commits to zero `dart:ffi` dependencies in the core library, accepting a performance trade-off for portability and maintenance control.

## Local clone

| Field | Value |
|-------|-------|
| GitHub | `jxoesneon/quic_lib` |
| Local path | `C:/Users/josee/dart_quic` |
| Version | 1.13.0 (final v1.x release, 2026-07-07, commit `0a1f02b`) |
| License | MIT |
| Visibility | PUBLIC |
| SDK | Dart >=3.5.0 <4.0.0 |
| Working tree | One modified file (`lib/src/crypto/tls/crl_fetcher.dart`); otherwise clean |

## Codebase scale

| Metric | Count |
|--------|-------|
| `lib/src/` Dart files | 136 |
| Test files | ~165 |
| Specs (`doc/specs/`) | 22 |
| ADRs (`doc/decisions/`) | 7 |
| Security audit reports | 5 |
| Architecture docs | 6 |
| Example apps | 4 |
| Benchmark harnesses | 2 |

## Top-level structure

- `pubspec.yaml` — dependencies: `asn1lib`, `cryptography`, `pointycastle` ^4.0.0, `x509`, `logging`, `meta`.
- `README.md`, `ARCHITECTURE.md`, `ROADMAP.md`, `CHANGELOG.md`, `AGENTS.md`, `SECURITY.md`, `SECURITY_FIXES.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`.
- `doc/` — architecture docs (6), specs (22), ADRs (7), security audits (5), research, archive, migration notes.
- `lib/` — public barrel files (`quic_lib.dart`, `quic.dart`, `http3.dart`, `libp2p.dart`, `webtransport.dart`) + `src/` implementation.
- `lib/src/` subsystems (12):
  - `connection/` (12) — connection lifecycle, CID management, migration, congestion control (CUBIC, BBR, Hystart).
  - `crypto/` (35) — TLS 1.3 handshake, packet protection, key derivation, certificate verification, OCSP/CRL.
  - `recovery/` (10) — loss detection, RTT, PTO, pacing, congestion controller.
  - `streams/` (8) — stream multiplexing, flow control, reassembly, scheduling.
  - `http3/` (27) — frames, QPACK, server push, Extended CONNECT for WebTransport.
  - `webtransport/` (8) — session, capsule routing, flow control.
  - `libp2p/` (9) — multiaddr, PeerId, DCUtR, libp2p TLS extension, multistream-select.
  - `io/` (12) — UDP socket, endpoint, per-connection isolates, rate limiter.
  - `wire/` (12) — packet/frame codec, varint, stateless reset, QUIC v2, bit greaser.
  - `security/` (2) — rate limiting, anti-amplification.
  - `utils/` (2) — hex, list equality.
  - `logging/` (1) — `QuicLogger`.
- `test/` — unit (~150), integration (~9), e2e (3), fuzz (6+), interop (1 scaffold), benchmark (1).
- `benchmark/` — `wire_codec_bench.dart` (standalone).
- `example/` — echo client/server, HTTP/3 client, shared config.

## Architecture

Layer stack (top to bottom):

```
Application adapters (HTTP/3, WebTransport, libp2p)
    ↓
Stream Manager (multiplexing, flow control, scheduling)
    ↓
Connection Manager (state machine, migration, CID management)
    ↓
Recovery / Packet Engine / TLS Engine
    ↓
Wire Codec
    ↓
UDP I/O
```

See [[ciel/projects/quic_lib/architecture.md|Architecture & Subsystems]] for the full per-subsystem breakdown.

### Core classes

- `QuicConnection` — central orchestrator (connection lifecycle, frame dispatch).
- `QuicEndpoint` — UDP socket binding and connection routing.
- `Libp2pQuicTransport` — libp2p-style dial/listen API.
- `CongestionController` / `CubicCongestionController` / `BbrCongestionController` / `Hystart`.
- `PacketProtector`, `HeaderProtection`, `KeyDerivation`, `KeyManager`, `HandshakeCoordinator`.
- `StreamManager`, `QuicStream`, `FlowController`, `ReassemblyBuffer`, `RoundRobinScheduler`.
- `Http3Connection`, `QpackEncoder`/`QpackDecoder`, `WebTransportSession`, `WebTransportSessionManager`.
- `DCUtRHandler`, `DCUtRStateMachine`, `DCUtRUdpCoordinator`.

### Architectural patterns

- **State machines**: Connection, TLS handshake, Stream send/receive, DCUtR.
- **Coordinator**: `QuicConnection`, `RecoveryManager`, `HandshakeCoordinator`, `Http3Connection`.
- **Strategy**: `CongestionController` (CUBIC/BBR), `StreamScheduler` (RoundRobin/custom), `CryptoBackend`.
- **Codec**: `FrameCodec`, `ProtectedPacketCodec`, `QpackEncoder`/`QpackDecoder`.
- **Platform abstraction**: IO vs stub pairs for `UdpSocket`, `ConnectionIsolate`, `IsolateSupervisor`, `PlatformAddress`.

### Platform support

| Platform | Support |
|----------|---------|
| Android, iOS, Linux, macOS, Windows | Full native UDP sockets + isolates |
| Web / WASM | Not supported — browsers do not expose raw UDP sockets |

## Build / test / verify

```bash
dart test
dart analyze
dart format --set-exit-if-changed .
pana . --exit-code-threshold 0
dart pub publish --dry-run
```

See [[ciel/projects/quic_lib/build-test-ci.md|Build, Test & CI]] for the full test inventory, CI jobs, and examples.

## Recent git state

- **Current version:** 1.13.0 (2026-07-07) — final v1.x release; all v1.x criteria complete. Next phase v2.0.0.
- **Working tree:** one modified file (`lib/src/crypto/tls/crl_fetcher.dart`); otherwise clean.
- **Recent commits:**
  - `f7226b4` ci: upgrade actions/upload-artifact to v7.0.1 (Node 24)
  - `e70f8a2` ci: fix stale actions/upload-artifact SHA pin
  - `7fd5207` ci: fix stale GitHub Actions SHA pins in ci.yml
  - `0a1f02b` chore: v1.13.0 final v1.x release
  - `9f7170f` Implement QUIC v2-specific behaviors (ROADMAP #4)
  - `ec900c0` feat(crypto): implement OCSP/CRL fetching and validation (ROADMAP #1)
  - `1c61ef6` Implement HTTP/3 server push over the network (ROADMAP #2)
  - `deafcf8` test(interop): add interop test matrix scaffold (ROADMAP #8)

## Relationship to dart_ipfs

- `dart_ipfs` is the primary downstream consumer.
- Four-class integration contract: `Libp2pQuicTransport`, `Libp2pConnection`, `Libp2pStream`, `PeerId`.
- Provides peer discovery, authenticated libp2p TLS 1.3 handshake, bidirectional streams, connection multiplexing (50+), connection migration.
- See `doc/architecture/DART_IPFS_INTEGRATION.md` for the integration contract.

## Known limitations / deferred work

- **No Web/WASM support** (architectural, not temporary — browsers block raw UDP).
- **ECN support** deferred to v2.0.0 — blocked on missing `IP_TOS`/`IPV6_TCLASS` socket options in Dart's `RawDatagramSocket` (Issue #10; would require relaxing ADR-001 FFI constraint).
- **Interop tests** are a scaffold only — all 35 cells (5 implementations × 7 features) marked `untested`. Last formal interop was v1.0.0.
- **Pure-Dart crypto** is slower than native (accepted portability trade-off).
- **No substantive TODOs/FIXMEs** in the codebase (only 5 trivial matches, all internal helpers or code-completion artifacts).

## Security posture

- Zero remaining findings after 3 blue team + 2 red team audit loops (30 security fixes applied).
- Rate limiting on state transitions (100/sec max), anti-amplification (3x receive limit).
- Caps on tracked resources (10,000 packets, 256 MB flow-control windows, 8 active CIDs).
- Constant-time comparisons for crypto verification; no logging of secrets.
- Soft-fail revocation policy (ADR-003) for P2P/mobile use cases.

See [[ciel/projects/quic_lib/security-and-audits.md|Security & Audits]] for the full audit history.

## Specs & ADRs

See [[ciel/projects/quic_lib/specs-and-adrs.md|Specs & ADRs]] for the full inventory of 22 specs and 7 ADRs.

## Dependencies

See [[ciel/projects/quic_lib/dependencies.md|Dependencies]] for the full dependency analysis.

## Key files for deeper context

1. `AGENTS.md` — verification and release checklist.
2. `ARCHITECTURE.md` — system architecture.
3. `doc/architecture/MODULE_OVERVIEW.md` — module catalog.
4. `doc/architecture/DART_IPFS_INTEGRATION.md` — integration contract.
5. `doc/specs/DART_API_SPEC.md` — public API design.
6. `doc/decisions/ADR-001_Pure_Dart_No_FFI.md` / `ADR-007_Isolate_per_Connection_Architecture.md`.
7. `lib/src/connection/quic_connection.dart` — central orchestrator.
8. `lib/src/io/quic_endpoint.dart` — primary entry point.
9. `lib/src/libp2p/libp2p_quic_transport.dart` — libp2p transport.
10. `doc/INDEX.md` — complete documentation index.

## Related

- [[ciel/projects/quic_lib/quic_lib.md|quic_lib overview]]
- [[ciel/projects.md|Projects index]]
- [[ciel/projects/IPFS/IPFS.md|IPFS]] (primary downstream consumer)
- [[ciel/projects/dart_ipfs/dart_ipfs.md|dart_ipfs overview]]
