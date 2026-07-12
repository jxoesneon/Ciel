---
title: quic_lib — Dependencies
project_note: update
type: project-note
tags: ["project-note","update"]
status: active
created: 2026-07-11
updated: 2026-07-11
source: "https://github.com/jxoesneon/quic_lib"
---

# quic_lib — Dependencies

Full dependency analysis for quic_lib v1.13.0.

## Runtime dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| `asn1lib` | ^1.6.0 | ASN.1/DER parsing for X.509 certificates |
| `cryptography` | ^2.5.0 | Primary crypto backend (AES-GCM, ChaCha20, HKDF, X25519, Ed25519). Platform acceleration. |
| `logging` | ^1.2.0 | Logging infrastructure (`QuicLogger`) |
| `meta` | ^1.0.0 | Meta annotations (`@immutable`, `@visibleForTesting`, etc.) |
| `pointycastle` | ^4.0.0 | Cryptography fallback (ASN1 parsing for RSA public keys). Migrated to 4.0.0. |
| `x509` | ^0.2.3 | X.509 certificate handling |

## Dev dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| `benchmark` | ^0.3.0 | Benchmarking framework for micro-benchmarks |
| `coverage` | ^1.15.1 | Code coverage collection |
| `mocktail` | ^1.0.0 | Mocking library for tests |
| `test` | ^1.24.0 | Testing framework |

## Environment

```yaml
environment:
  sdk: ">=3.5.0 <4.0.0"
```

## Platforms

```yaml
platforms:
  android:
  ios:
  linux:
  macos:
  windows:
  # web is intentionally excluded: this package uses dart:io UDP sockets
  # (RawDatagramSocket) which are unavailable in browser environments.
```

## Dependency notes

- **ADR-004:** `cryptography` is the primary backend (platform acceleration); `pointycastle` is the fallback for ASN1/RSA parsing.
- **pointycastle 4.0.0 migration:** Low risk. Only ASN1 parsing for RSA public keys needs testing. See `doc/POINTYCASTLE_4_MIGRATION.md`.
- **ADR-001:** Zero `dart:ffi` dependencies in core. This is a hard constraint that blocks ECN support (Issue #10).
- **No web/WASM:** Intentionally excluded. Browsers do not expose raw UDP sockets. See `doc/WEB_AND_WASM.md`.

## Example app dependencies

```yaml
# example/pubspec.yaml
name: quic_lib_example
dependencies:
  quic_lib:
    path: ..
```

Only depends on `quic_lib` via path dependency. SDK: ^3.0.0.

## Related

- [[ciel/projects/quic_lib/quic_lib.md|quic_lib overview]]
- [[ciel/projects/quic_lib/knowledgebase.md|quic_lib knowledgebase]]
- [[ciel/projects/quic_lib/build-test-ci.md|Build, Test & CI]]
