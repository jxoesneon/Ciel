---
title: quic_lib
project_note: hub
type: project
tags: ["project"]
status: active
priority: 1
created: "2026-06-27T13:08:00.000Z"
updated: 2026-07-11
source: "https://github.com/jxoesneon/quic_lib"
language: Dart
visibility: PUBLIC
license: MIT License
topics: None
---

# quic_lib

A pure-Dart QUIC, HTTP/3, WebTransport, and libp2p transport stack.

## Metadata

| Field | Value |
|-------|-------|
| Owner | jxoesneon |
| Repository | https://github.com/jxoesneon/quic_lib |
| Homepage | — |
| Default branch | main |
| Primary language | Dart |
| Visibility | PUBLIC |
| License | MIT License |
| Stars | 0 |
| Forks | 0 |
| Created | 2026-06-27T13:08:00Z |
| Updated | 2026-07-07T22:35:40Z |
| Archived | False |
| Fork | False |

## Topics

None

## Use and scope

A pure-Dart implementation of modern networking protocols, intended to support HTTP/3, WebTransport, and libp2p transports in Dart and Flutter applications. It is the primary transport dependency for `dart_ipfs`.

## Local clone snapshot

- Path: `C:/Users/josee/dart_quic`
- Version: 1.13.0 (final v1.x release, 2026-07-07, commit `0a1f02b`)
- Stack: pure Dart, zero FFI in core; isolate-per-connection architecture.
- SDK: Dart >=3.5.0 <4.0.0
- Platform support: Android, iOS, Linux, macOS, Windows. Web/WASM not supported.
- Working tree: one modified file (`lib/src/crypto/tls/crl_fetcher.dart`); otherwise clean.
- Codebase: 136 Dart files in `lib/src/` across 12 subsystems; ~165 test files; 22 specs; 7 ADRs; 5 security audits.
- Dependencies: `asn1lib`, `cryptography`, `pointycastle` ^4.0.0, `x509`, `logging`, `meta`.

## Expanded knowledge

- [[ciel/projects/quic_lib/knowledgebase.md|quic_lib — Knowledgebase]]
- [[ciel/projects/quic_lib/architecture.md|quic_lib — Architecture & Subsystems]]
- [[ciel/projects/quic_lib/specs-and-adrs.md|quic_lib — Specs & ADRs]]
- [[ciel/projects/quic_lib/security-and-audits.md|quic_lib — Security & Audits]]
- [[ciel/projects/quic_lib/build-test-ci.md|quic_lib — Build, Test & CI]]
- [[ciel/projects/quic_lib/dependencies.md|quic_lib — Dependencies]]

## Related

- [[ciel/projects.md|Projects index]]
- [[ciel/projects/IPFS/IPFS.md|IPFS]]
