---
title: "IPFS — Specs & Compliance"
project_note: update
type: project-note
tags: [project, IPFS]
created: 2026-07-11
status: active
---

# IPFS — Specs & Compliance

Status of the 26 tracked feature specifications, plus audits, decisions, and roadmap context.

## Implementation inventory summary

As of v1.11.7, **all 26 tracked specs are Complete**: 26/26 (100%), 0 Partial, 0 Missing.

| Spec | Priority | Status | Key files | Assessment |
|------|----------|--------|-----------|------------|
| **CAR_FORMAT_SPEC** | P0 | Complete | `lib/src/core/data_structures/car.dart` | Standard CAR v1/v2 API; cross-codec round-trip tests pass. |
| **CLI_SPEC** | P0 | Complete | `bin/ipfs.dart`, `lib/src/core/ipfs_node/content_manager.dart` | `CommandRunner` with daemon, version, id, healthcheck, add, cat, ls, pin, unpin, swarm, config; clean SIGINT/SIGTERM shutdown. |
| **DAG_CBOR_SPEC** | P0 | Complete | `lib/src/core/cbor/enhanced_cbor_handler.dart`, `lib/src/core/ipld/codecs/standard_codecs.dart` | Tag-42 CIDs, canonical map ordering, big-int tags 2/3, strict decoding, `-2^64` boundary fix. |
| **DAG_JSON_SPEC** | P1 | Complete | `lib/src/core/ipld/dag_json_handler.dart`, `lib/src/core/ipld/codecs/standard_codecs.dart` | Reserved namespace handling, canonical key sorting, unpadded base64url bytes, strict decoding. |
| **DHT_INTEGRATION_SPEC** | P0 | Complete | `lib/src/protocols/dht/dht_client.dart`, `lib/src/protocols/dht/dht_envelope.dart` | `DHTEnvelope` framing, iterative Kademlia `findProviders`/`findPeer`/`getValue`, provider validation, metrics, request/response correlation. Interoperates with Kubo. |
| **DOCKER_SPEC** | P0 | Complete | `Dockerfile` | Multi-stage hardened runtime (cgr.dev/chainguard/glibc-dynamic); multi-arch support (amd64, arm64); Trivy scan, cosign sign. |
| **IPLD_SELECTORS_SPEC** | P0 | Complete | `lib/src/core/ipld/selectors/selector_ast.dart`, `lib/src/core/ipld/selectors/selector_executor.dart`, `lib/src/core/ipfs_node/ipld_handler.dart` | Official selector vocabulary, transparent link following, GraphSync integration. |
| **METRICS_SPEC** | P0 | Complete | `lib/src/core/metrics/metrics_collector.dart`, `lib/src/services/gateway/gateway_server.dart`, `lib/src/services/rpc/rpc_server.dart` | Prometheus counters/gauges/histograms, `/metrics` endpoint, lifecycle wiring, request instrumentation. |
| **MFS_SPEC** | P0 | Complete | `lib/src/core/mfs/mfs_manager.dart`, `lib/src/services/rpc/mfs_handlers.dart`, `lib/src/services/rpc/rpc_server.dart` | flush, mv, chcid, stat/ls, write offset/truncate, RPC routes, lifecycle registration. |
| **REPROVIDE_SPEC** | P1 | Complete | `lib/src/protocols/dht/reprovider.dart`, `lib/src/core/ipfs_node/ipfs_node.dart` | Periodic reprovide with strategies (`pinned`, `roots`, `all`, `pinned+mfs`, `entities`), batching, sweep optimization. |
| **SUBDOMAIN_GATEWAY_SPEC** | P1 | Complete | `lib/src/services/gateway/gateway_handler.dart`, `lib/src/core/config/gateway_config.dart`, `lib/src/services/gateway/gateway_server.dart` | Subdomain detection, CIDv0→CIDv1 conversion, DNSLink/IPNS resolution, CORS, TLS redirect, denylist, trustless negotiation. |
| **TRUSTLESS_GATEWAY_SPEC** | P0 | Complete | `lib/src/services/gateway/gateway_handler.dart`, `lib/src/services/gateway/gateway_trustless_handler.dart`, `lib/src/core/security/denylist_service.dart` | Format negotiation, CAR/raw/DAG-JSON/DAG-CBOR/IPNS-record responses, Bitswap fallback, 451 denylist. |
| **UNIXFS_SPEC** | P0 | Complete | `lib/src/core/unixfs/` | Directory construction with correct `Tsize`, path resolution, HAMT sharding (fanout 256, CIDv1 dag-pb, MurmurHash3 x64-64), symlinks, cycle detection. DAG-PB wire order matches Kubo/Helia. |
| **BITSWAP_HTTP_FALLBACK_SPEC** | P1 | Complete | `lib/src/protocols/bitswap/bitswap_handler.dart`, `lib/src/core/config/bitswap_config.dart`, `lib/src/transport/http_gateway_client.dart` | HTTP gateway block fallback for Bitswap, configurable gateways, timeout, block verification, private-gateway gating, retry logic. |
| **BROWSER_TRANSPORTS_SPEC** | P1 | Complete | `lib/src/transport/webrtc/`, `lib/src/transport/webtransport/`, `lib/src/transport/libp2p_router.dart` | Configurable STUN/TURN, ICE helper, WebRTC stat/scope, WebTransport certhash decoding, no hardcoded STUN. |
| **CIRCUIT_RELAY_SPEC** | P0 | Complete | `lib/src/transport/circuit_relay_client_io.dart`, `lib/src/transport/circuit_relay_client_web.dart`, `lib/src/core/config/network_config.dart` | CONNECT flow, reservation refresh, `CircuitRelayConfig`, max-circuits enforcement, router relayed-connection registration, HOP/STOP client. |
| **CONTENT_BLOCKING_SPEC** | P1 | Complete | `lib/src/core/security/denylist_service.dart`, `lib/src/services/gateway/gateway_handler.dart`, `lib/src/services/rpc/rpc_handlers.dart`, `lib/src/protocols/dht/dht_handler.dart`, `lib/src/protocols/bitswap/bitswap_handler.dart` | BadBits-style compact parser, CID/multihash blocking, gateway/RPC/DHT/Bitswap/MFS 451 integration, persistence and audit log. |
| **GATEWAY_TLS_SPEC** | P1 | Complete | `lib/src/core/config/gateway_config.dart`, `lib/src/services/gateway/gateway_tls_manager.dart`, `lib/src/services/gateway/acme_client.dart`, `lib/src/services/gateway/domain_validator.dart`, `lib/src/platform/http_server_adapter_io.dart` | TLS/AutoTLS config fields, `serveSecure`, TLS manager with AutoTLS flow, ACME v2 HTTP-01 challenge, domain validation, certificate persistence. |
| **GOSSIPSUB_SPEC** | P0 | Complete | `lib/src/protocols/pubsub/gossipsub/` | v1.1 protobuf, handler, config, message signing, message cache, peer scoring. |
| **GRAPHSYNC_SPEC** | P1 | Complete | `lib/src/protocols/graphsync/graphsync_handler.dart`, `lib/src/core/config/graphsync_config.dart`, `lib/src/protocols/graphsync/graphsync_budget.dart` | Unicast responses, budget enforcement, CID prefix helpers, client `requestGraph`, bidirectional pause/resume/cancel, Bitswap fallback. |
| **INTEROP_TESTS_SPEC** | P0 | Complete | `.github/workflows/interop.yml`, `.github/workflows/interop_nightly.yml`, `test/interop/` | P0/P1 workflows, Kubo/Helia compose harnesses, interop test scaffolding. All interop tests pass in CI. |
| **IPNS_SPEC** | P0 | Complete | `lib/src/protocols/ipns/ipns_handler.dart`, `lib/src/protocols/ipns/ipns_record.dart` | DHT-first signed CBOR records, base36 name derivation, signature verification, optional PubSub subscription gating. Interoperates with Kubo. |
| **KUBERNETES_SPEC** | P1 | Complete | `k8s/`, `helm/dart-ipfs/`, `.github/workflows/k8s.yml` | Kustomize base/overlays, Helm chart with hardened deployment, NetworkPolicy, ServiceMonitor, HPA, PDB; CI lint/template validation. |
| **MODULARIZATION_SPEC** | P1 | Complete | `packages/dart_ipfs_core/`, `melos.yaml`, `lib/dart_ipfs.dart` | `packages/dart_ipfs_core` extracted with stable CID/block/codec/crypto/data-structures; umbrella re-exports preserved; Melos workspace; deprecation notice for deep `lib/src/` imports. **Note:** Further modularization (WP-07) abandoned by Council of Five. |
| **PLUGINS_SPEC** | P1 | Complete | `lib/src/core/plugins/` | PluginHost, manifest, capability registry, signing/verification, examples, audit logging, metrics emission. |
| **QUIC_SPEC** | Conditional | Complete | `lib/src/core/config/network_config.dart`, `lib/src/transport/libp2p_router.dart`, `test/transport/quic_transport_test.dart`, `packages/dart_ipfs_quic/`, `doc/specs/QUIC_TRANSPORT_RFC.md` | Config fields, runtime probe, TCP fallback, address synthesis, and tests implemented. `dart_ipfs_quic` backed by pure-Dart `quic_lib` ^1.13.0. |

## Council of Five artifacts

- `doc/specs/audits/` — 6 audit reports (Master, Core Data Layer, Networking P2P 1&2, Operations, Services).
- `doc/specs/decisions/` — 10 binding decisions (CAR migration, config lifecycle, Docker base, interop scope, IPLDCodec reconciliation, plugin security, QUIC peer cert audit, release readiness, gap audit + closure).
- `doc/specs/features/` — 26 per-feature specs.

## Key architecture docs

- `doc/ARCHITECTURE.md` — Manager-Handler pattern.
- `doc/specs/IMPLEMENTATION_INVENTORY.md` — full 26-spec status table.
- `doc/specs/PROTOCOL_COMPLIANCE_SPEC.md` — v2.0 compliance goals.
- `doc/specs/NETWORKING_P2P_SPEC.md` — QUIC, WebTransport, Circuit Relay, Gossipsub.
- `doc/specs/SERVICES_APIS_SPEC.md` — Gateway, RPC, MFS, metrics.
- `doc/specs/OPERATIONS_ECOSYSTEM_SPEC.md` — CLI, Docker, Kubernetes, plugins.
- `doc/specs/QUIC_TRANSPORT_RFC.md` — Native QUIC transport RFC (v1.1, quic_lib selection).
- `doc/specs/dht_routing_abstraction.md` — DHT routing table abstraction.
- `doc/specs/RESOLUTIONS_AGGREGATE.md` — Aggregate resolution report.

## Roadmap highlights

- **v1.10** (done, Feb 2026): IpfsPlatform, IndexedDB, browser SecurityManager.
- **v1.11** (done, May 2026): browser transports (WebRTC/WebTransport), IPNS optimizations, advanced IPLD codecs.
- **v1.11.5** (done, Jun 2026): Monorepo, QUIC via quic_lib, CLI hardened.
- **v1.11.7** (done, Jul 2026): CI green, web compatibility, publishing regression fixed.
- **v2.0** (target Sep 2026): parallel block fetching, smart caching, connection pooling, bandwidth shaping, MFS already landed.
- **v2.1** (target Mar 2027): plugin system, native QUIC, native Ed25519/X25519, hole punching, HSM, multi-sig IPNS, ZK proofs.
- **v2.2** (target Jun 2027): VS Code extension, IDE plugins, code generation, web dashboard.
- **v3.0+** (target Sep 2027+): WASM build, marketplace, desktop native app, IPFS over Bluetooth, AI content discovery, quantum-safe crypto.

**Note:** ROADMAP.md still shows "Current Version: 1.11.5" — needs update to 1.11.7.

## Related

- [[ciel/projects/IPFS/knowledgebase.md|IPFS — Knowledgebase]]
- [[ciel/projects/IPFS/architecture.md|IPFS — Architecture]]
- [[ciel/projects/IPFS/build-test-ci.md|IPFS — Build, Test & CI]]
- [[ciel/projects/IPFS/dependencies-and-monorepo.md|IPFS — Dependencies & Monorepo]]
- [[ciel/projects/IPFS/security-and-traps.md|IPFS — Security & Traps]]
- [[ciel/projects/IPFS/git-state.md|IPFS — Git State]]
