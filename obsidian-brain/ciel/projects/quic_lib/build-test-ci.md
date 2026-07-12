---
title: "quic_lib — Build, Test & CI"
project_note: update
type: project-note
tags: ["project-note","update"]
status: active
created: 2026-07-11
updated: 2026-07-11
source: "https://github.com/jxoesneon/quic_lib"
---

# quic_lib — Build, Test & CI

Test infrastructure, CI workflows, benchmarks, and examples for quic_lib v1.13.0.

## Verification commands

```bash
# Run all tests
dart test

# Run with compact reporter
dart test --reporter=compact

# Run by tag
dart test --tags unit
dart test --tags integration
dart test --tags fuzz
dart test --tags interop

# Static analysis
dart analyze

# Format check
dart format --set-exit-if-changed .

# Coverage
dart run coverage:test_with_coverage --out coverage
dart run coverage:format_coverage --lcov --in=coverage --out=coverage/lcov.info --packages=.dart_tool/package_config.json --report-on=lib

# Publish dry run
pana . --exit-code-threshold 0
dart pub publish --dry-run
```

## Test configuration (`dart_test.yaml`)

```yaml
timeout: 2m
tags:
  unit:
  integration:
  interop:
  fuzz:
  performance:
platforms:
  - vm
concurrency: 4
```

- **Platform:** VM only (no browser tests).
- **Timeout:** 2 minutes per test.
- **Concurrency:** 4 concurrent tests.

## Test inventory (~165 files)

| Category | Count | Notes |
|----------|-------|-------|
| Unit tests | ~150 | Across all subsystems |
| Integration tests | ~9 | Real UDP sockets, migration integration |
| E2E tests | 3 | HTTP/3 over encrypted QUIC, QUIC endpoint, RFC 9001 handshake |
| Fuzz tests | 6+ | Crypto, wire, X.509, QPACK, HTTP/3 frames, varint, packet receiver |
| Interop tests | 1 (scaffold) | 5 implementations × 7 features = 35 cells, all `untested` |
| Benchmark harness | 1 | CI-integrated micro-benchmarks |

### Test files by subsystem

| Subsystem | Test files | Highlights |
|-----------|-----------|------------|
| `connection/` | 21 | BBR, CUBIC, Hystart, CID management, migration, ECN, path validation, zero-RTT |
| `crypto/` | 42 | Cipher suites, key management, packet protection, TLS handshake (31 files), OCSP/CRL, X.509 fuzz |
| `http3/` | 37 | All frame types, QPACK (encoder/decoder/tables/streams), server push, capsule protocol, WebTransport integration |
| `libp2p/` | 14 | DCUtR full handshake, NAT traversal, certificates, multiaddr, multistream-select, PeerId |
| `webtransport/` | 9 | Capsule types/routing, flow control, session manager |
| `wire/` | 27 | Frame parsing, packet builder, headers, varint, QUIC v2, bit greaser, stateless reset |
| `io/` | 9 | Connection isolate, isolate supervisor, endpoint, UDP rate limiter, rebind |
| `e2e/` | 3 | HTTP/3 e2e, QUIC e2e, RFC 9001 handshake |
| `fuzz/` | 3 (+4 embedded) | Crypto, wire, plus embedded fuzz tests in crypto/wire/http3 |

### E2E tests

| Test | Purpose |
|------|---------|
| `http3_e2e_test.dart` | Full HTTP/3 request/response over encrypted QUIC (added v1.12.4). Uses deterministic test keys. |
| `quic_e2e_test.dart` | QUIC endpoint operations, encrypted packet round-trips (Initial + 1-RTT). |
| `rfc9001_handshake_test.dart` | Full RFC 9001 TLS 1.3 handshake with real X25519 ECDH, minimal DER X.509 certs, transcript hash verification. |

### Interop test matrix

- **Reference implementations (5):** quic-go, aioquic, ngtcp2, cloudflare-quiche, msquic.
- **Features (7):** handshake, oneRttData, streamMultiplexing, connectionMigration, zeroRtt, http3, webTransport.
- **Matrix:** 5 × 7 = 35 cells, all marked `untested` (scaffold).
- **Behavior:** Tests skip gracefully when reference binaries are not installed.
- **Last formal interop:** v1.0.0.

## Benchmarks

| Harness | Location | Purpose |
|---------|----------|---------|
| `wire_codec_bench.dart` | `benchmark/` | Standalone: VarInt (100k iter), LongHeader (10k), Frame (10k), PacketBuilder (5k). Output: µs/op. |
| `benchmark_harness.dart` | `test/benchmark/` | CI-integrated: VarInt, ConnectionIdManager, InitialSecrets, FrameCodec. Output: ops/sec. Scaffold for Phase 4. |

**Baseline targets (from `PERFORMANCE_BENCHMARKING.md`):**
- Handshake latency: <50ms
- Throughput: >100MB/s
- Crypto: >200k packets/sec

## Examples

| Example | Purpose |
|---------|---------|
| `echo_common.dart` | Shared config: test DCID, "Hello, QUIC!" message, port 12345, `createEchoConnection()` with deterministic keys. |
| `echo_server.dart` | QUIC echo server on `127.0.0.1:12345`. Run: `dart run example/echo_server.dart` |
| `echo_client.dart` | QUIC echo client. Run: `dart run example/echo_client.dart` (with server running) |
| `http3_client.dart` | Minimal HTTP/3 client scaffold. Connects to `127.0.0.1:4433`, exchanges SETTINGS, sends GET. |

**Note:** Echo examples use deterministic test keys to bypass full TLS handshake. No WebTransport or libp2p examples exist.

## CI workflow (`.github/workflows/ci.yml`)

### Triggers
- **Push:** `main` branch
- **Pull request:** `main` branch
- **Schedule:** Nightly fuzz at 02:00 UTC; weekly benchmarks at 03:00 UTC Sundays
- **Manual:** `workflow_dispatch`

### Jobs

| Job | Condition | Timeout | Matrix | Notes |
|-----|-----------|---------|--------|-------|
| Lint | push/PR | 3 min | Ubuntu/macOS/Windows × stable + Ubuntu × beta | `dart analyze` + `dart format`. Beta continue-on-error. |
| Unit | push/PR | 8 min | Same as lint | `dart test --concurrency=4`. Beta continue-on-error. |
| Integration | PR only | 20 min | 3 OS × stable | Docker required. Non-Ubuntu continue-on-error. |
| Coverage | push/PR | 15 min | Ubuntu only | LCOV artifact uploaded. |
| Fuzz | scheduled/manual | 5 min | Ubuntu only | `dart run test/fuzz/fuzz_harness.dart` |
| Performance | scheduled/manual | 5 min | Ubuntu only | `dart run test/benchmark/benchmark_harness.dart` |
| Changelog | push/PR | 3 min | Ubuntu only | Verifies `[Unreleased]` section and version headings. |

### Concurrency
- Group: `${{ github.workflow }}-${{ github.ref }}`
- Cancel in-progress: Yes

### Pinned actions
- `actions/checkout@9c091bb...` (v7.0.0)
- `dart-lang/setup-dart@65eb853...` (v1.7.2)
- `actions/cache@55cc834...` (v6.1.0)
- `actions/upload-artifact@043fb46...` (v7.0.1)

## Analysis options

- **Strict-casts:** true
- **Strict-raw-types:** true
- **Implicit-casts:** false
- **Implicit-dynamic:** false
- **Errors:** `missing_required_param` and `missing_return` as errors; `dead_code`, `invalid_assignment`, `unused_import` as warnings.
- **Exclude:** `test/**` (looser types in tests).
- **Lints:** ~85 rules (core + recommended + project-specific: `avoid_slow_async_io`, `cancel_subscriptions`, `close_sinks`, `unnecessary_await_in_return`, `public_member_api_docs`).

## Coverage targets

- Alpha: ≥70%
- Beta: ≥80%
- Stable/RC: ≥90%

## Related

- [[ciel/projects/quic_lib/quic_lib.md|quic_lib overview]]
- [[ciel/projects/quic_lib/knowledgebase.md|quic_lib knowledgebase]]
- [[ciel/projects/quic_lib/dependencies.md|Dependencies]]
