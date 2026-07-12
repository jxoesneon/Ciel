---
title: "IPFS — Dependencies & Monorepo"
project_note: update
type: project-note
tags: [project, IPFS]
created: 2026-07-11
status: active
---

# IPFS — Dependencies & Monorepo

Package manifest, monorepo layout, stability tiers, and migration guidance.

## Umbrella package manifest

`pubspec.yaml` — `dart_ipfs` v1.11.7, Dart SDK `^3.10.0`.

### Core dependencies

| Dependency | Purpose |
|------------|---------|
| `dart_ipfs_core` ^1.11.5 | Stable core primitives (CID, block, codec, crypto) — monorepo |
| `dart_ipfs_quic` ^0.2.0 | QUIC transport via `quic_lib` — monorepo |
| `ipfs_libp2p` ^0.5.6 | libp2p networking layer |
| `crypto` ^3.0.7 | SHA-256, HMAC |
| `cryptography` ^2.9.0 | Ed25519, AES-256-GCM, X25519, WASM-compatible |
| `pointycastle` ^4.0.0 | RSA, ECDSA, ASN.1 pure-Dart |
| `cipherlib` ^0.7.1 | Additional cipher implementations |
| `catalyst_cose` ^1.0.0 | COSE signing |
| `jose` ^0.3.5 | JOSE (JWS/JWE) |
| `hive` ^2.2.3 | Local datastore (VM) |
| `idb_shim` ^2.9.2 | IndexedDB shim for web storage |
| `protobuf` ^6.0.0 | Protocol Buffers |
| `prometheus_client` ^1.0.0+1 | Metrics exposition |
| `http` ^1.6.0, `shelf` ^1.4.2, `shelf_router` ^1.1.4 | Gateway/RPC HTTP layer |
| `http_parser` ^4.1.2, `mime` ^2.0.0 | HTTP parsing, MIME types |
| `multibase` ^1.0.0, `dart_multihash` ^1.0.1, `dart_merkle_lib` ^1.0.1 | Multiformat primitives |
| `base32` ^2.2.0, `murmurhash` ^1.0.1 | Base32, MurmurHash3 (UnixFS HAMT) |
| `cbor` ^6.3.7 | CBOR handling |
| `archive` ^4.0.7 | Archive utilities |
| `multicast_dns` ^0.3.3 | mDNS peer discovery |
| `port_forwarder` ^1.0.0 | NAT port mapping |
| `args` ^2.6.0, `yaml` ^3.1.3, `path` ^1.9.1 | CLI and config parsing |
| `grpc` ^5.1.0 | gRPC support |
| `dart_lz4` ^1.0.0 | LZ4 compression |
| `get_it` ^9.2.1 | Service locator / DI |
| `logging` ^1.3.0 | Structured logging |
| `markdown` ^7.3.0 | Markdown rendering |
| `synchronized` ^3.4.0 | Synchronization primitives |
| `intl` ^0.20.2 | Internationalization |
| `uuid` ^4.5.2 | UUID generation |
| `fixnum` ^1.1.1, `convert` ^3.1.2 | Numeric/conversion utilities |
| `collection` ^1.19.1, `meta` ^1.17.0 | Collections, metadata |
| `async` ^2.13.0 | Async utilities |
| `web` ^1.1.1 | Web platform interop |

### Dev dependencies

- `test` ^1.28.0, `lints` ^6.1.0, `coverage` ^1.15.0
- `mockito` ^5.6.1, `build_runner` ^2.10.4
- `very_good_analysis` ^10.1.0, `flutter_lints` ^6.0.0
- `fake_async` ^1.3.1

### Security overrides

```yaml
dependency_overrides:
  xml: ^7.0.1          # XML parsing vulnerabilities / encoding fixes
  dart_udx: ^2.0.3     # UDP buffer overflow / rate limiting patches
```

**Important:** `xml` and `dart_udx` must remain as `dependency_overrides` only, not direct dependencies. v1.11.7 fixed a regression where they were accidentally promoted to direct deps, breaking downstream consumers like `port_forwarder` (which constrains `xml ^6.5.0`).

## Monorepo packages

| Package | Version | Source | Purpose |
|---------|---------|--------|---------|
| `dart_ipfs_core` | 1.11.5 | `packages/dart_ipfs_core/` | Stable CID, multibase, multicodec, multihash, block, codec, crypto primitives |
| `dart_ipfs_quic` | 0.2.0 | `packages/dart_ipfs_quic/` | QUIC transport foundation, backed by pure-Dart `quic_lib` ^1.13.0 |

### dart_ipfs_core dependencies

`multibase`, `dart_multihash`, `cbor`, `convert`, `crypto`, `cryptography`, `collection`, `meta`

### dart_ipfs_quic dependencies

`ipfs_libp2p` ^0.5.6, `quic_lib` ^1.13.0, `dart_ipfs_core` ^1.11.5, `collection`, `meta`, `logging`, `uuid`

## Monorepo layout

```
.
├── lib/                          # Umbrella package (dart_ipfs)
│   ├── dart_ipfs.dart            # Public barrel including re-exports
│   └── src/                      # Protocol and service implementations
│       ├── core/                 # Core node logic (stays in umbrella)
│       ├── protocols/            # Bitswap, DHT, libp2p, etc.
│       ├── services/             # Gateway, RPC, IPNS, etc.
│       └── ...
├── packages/
│   ├── dart_ipfs_core/           # Stable core primitives (7 test files)
│   └── dart_ipfs_quic/           # QUIC transport foundation (4 test files)
├── melos.yaml                    # Workspace configuration
├── pubspec.yaml                  # Umbrella package
└── doc/monorepo.md               # Monorepo documentation
```

## Stability tiers

| Tier | Location | Stability | Examples |
|------|----------|-----------|----------|
| Tier 1 — Stable Core | `packages/dart_ipfs_core/lib/` | Spec-defined, low churn | `CID`, `Block`, `IBlockStore`, `IPLDCodec`, `CryptoUtils`, `Ed25519Signer` |
| Tier 2 — Umbrella Public | `lib/dart_ipfs.dart` | Public API, may evolve as services stabilize | `IPFSNode`, `GatewayServer`, `RPCServer`, `IPNSHandler` |
| Tier 3 — Unstable Internals | `lib/src/...` | Not part of public API; deprecated | Deep imports such as `package:dart_ipfs/src/core/cid.dart` |

## Dependency direction

- `dart_ipfs_core` has **no dependency** on the umbrella package.
- The umbrella package depends on `dart_ipfs_core` (path during development, published after release).
- `dart_ipfs_quic` depends on `dart_ipfs_core` and `quic_lib`.
- Protocol and service layers remain in the umbrella until they stabilize.

## Public API re-exports

`lib/dart_ipfs.dart` re-exports:

- From `dart_ipfs_core`: `CID`, `MultibaseUtils`, `Multicodec`, `MultihashInfo`, `MultihashUtils`, `Block`, `IBlock`, `BlockStoreResult`, `IBlockStore`, `InMemoryBlockStore`, `IPLDCodec`, `RawCodec`, `DagCborCodec`, `DagJsonCodec`, `CryptoUtils`, `EncryptedData`, `Ed25519Signer`, `KeyPairExtensions`, `ImmutableBytes`, `TypedMap`.
- From `dart_ipfs_quic`: `QuicTransport`, `QuicConnection`, `QuicListener`.
- Umbrella-specific: `IPFSNode`, `IPFSWebNode`, `IPFSConfig`, `CarReader`/`CarWriter`, `PubSubMessage`.

## Migration guide

| Old import | Recommended replacement |
|------------|-------------------------|
| `package:dart_ipfs/src/core/cid.dart` | `package:dart_ipfs/dart_ipfs.dart` or `package:dart_ipfs_core/dart_ipfs_core.dart` |
| `package:dart_ipfs/src/core/data_structures/block.dart` | `package:dart_ipfs/dart_ipfs.dart` or `package:dart_ipfs_core/dart_ipfs_core.dart` |
| `package:dart_ipfs/src/core/crypto/ed25519_signer.dart` | `package:dart_ipfs_core/dart_ipfs_core.dart` |

Deep imports are deprecated as of v2.2.0 and will be removed in v3.0.0.

## Future packages

Potential future extractions (each requires Council approval):

- `dart_ipfs_bitswap`
- `dart_ipfs_dht`
- `dart_ipfs_gateway`
- `dart_ipfs_rpc`

Each new package must:

1. Be approved by a Council deliberation.
2. Depend only on stable packages (`dart_ipfs_core` and other approved packages).
3. Never depend on the umbrella package.
4. Include its own `README.md`, `analysis_options.yaml`, and tests.
5. Be added to `melos.yaml` and the root `pubspec.yaml` path dependency during development.

## Related

- [[ciel/projects/IPFS/knowledgebase.md|IPFS — Knowledgebase]]
- [[ciel/projects/IPFS/architecture.md|IPFS — Architecture]]
- [[ciel/projects/IPFS/specs-and-compliance.md|IPFS — Specs & Compliance]]
- [[ciel/projects/IPFS/build-test-ci.md|IPFS — Build, Test & CI]]
- [[ciel/projects/IPFS/security-and-traps.md|IPFS — Security & Traps]]
- [[ciel/projects/IPFS/git-state.md|IPFS — Git State]]
- [[ciel/projects/quic_lib/knowledgebase.md|quic_lib — Knowledgebase]] (QUIC dependency)
