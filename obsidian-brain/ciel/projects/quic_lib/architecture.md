---
title: "quic_lib — Architecture & Subsystems"
project_note: update
type: project-note
tags: ["project-note","update"]
status: active
created: 2026-07-11
updated: 2026-07-11
source: "https://github.com/jxoesneon/quic_lib"
---

# quic_lib — Architecture & Subsystems

Full per-subsystem breakdown from the deep-mine of `lib/src/` (136 files, 12 subsystems).

## Layered architecture

```
Application Layer (HTTP/3, WebTransport, libp2p)
    ↓
Transport Layer (QUIC - QuicConnection, streams, recovery)
    ↓
Crypto Layer (TLS 1.3, packet protection, key management)
    ↓
Wire Format Layer (frames, packets, varints)
    ↓
I/O Layer (UDP sockets, isolates, platform abstraction)
```

## Public API surface (barrel files)

| Barrel | Exports |
|--------|---------|
| `lib/quic_lib.dart` | Full API — all subsystems |
| `lib/quic.dart` | QUIC transport core only (`QuicEndpoint`, `QuicConnection`, congestion controllers, `UdpSocket`, `Multiaddr`, schedulers, isolates, `QuicVersions`) |
| `lib/http3.dart` | HTTP/3 layer (`Http3Connection`, `Http3Request`/`Response`, frames, QPACK instructions, `OriginFrame`, `PriorityUpdateFrame`, capsules) |
| `lib/webtransport.dart` | WebTransport (`WebTransportSession`, `WebTransportSessionManager`, `WebTransportFlowController`, `CapsuleRouter`, capsule types) |
| `lib/libp2p.dart` | libp2p (`PeerId`, `Multiaddr`, `DCUtR*`, `Libp2pQuicTransport`, `MultistreamSelect`, `Libp2pCertificateGenerator`, `WebTransportSession`) |

## Subsystem breakdown

### 1. Connection (12 files)

**Purpose:** Central orchestrator for QUIC connection lifecycle.

**Key classes:**
- `QuicConnection` — central hub orchestrating state machine, recovery, streams, crypto.
- `ConnectionStateMachine` — lifecycle: idle → handshaking → established → closing/drain → closed.
- `ConnectionIdManager` — CID issuance, retirement, stateless reset tokens. Max 8 active CIDs, retired-CID history (max 32).
- `ConnectionRegistry` — maps DCIDs to connections for packet routing.
- `PacketReceiver` / `PacketSender` — incoming/outgoing packet processing.
- `MigrationHelper` — connection migration and NAT rebinding.
- `VersionInformation` / `VersionNegotiation` — RFC 9368 compatible version negotiation.

**Congestion control (4 files):**
- `CongestionController` — abstract interface (strategy pattern).
- `CubicCongestionController` — RFC 8312 CUBIC: `W_cubic(t) = C*(t-K)^3 + W_max`, fast convergence, Hystart++ slow-start exit.
- `BbrCongestionController` — RFC 8382 BBR v1: STARTUP/DRAIN/PROBE_BW/PROBE_RTT states, bandwidth filter over 10 RTT window, RTprop tracking.
- `Hystart` — hybrid slow-start algorithm for early congestion detection.

**Key methods:** `QuicConnection.processIncomingDatagram()`, `QuicConnection.buildEncryptedPacket()`, `openBidirectionalStream()`, `openUnidirectionalStream()`.

### 2. Crypto (35 files)

**Purpose:** TLS 1.3 handshake, packet protection, key derivation, crypto backend abstraction.

**Backend:**
- `CryptoBackend` — abstract interface (AES-GCM, ChaCha20, HKDF, X25519, Ed25519).
- `DefaultCryptoBackend` — implementation using `package:cryptography`.

**Packet protection (10 files):**
- `PacketProtector` — AEAD encrypt/decrypt.
- `HeaderProtection` — header mask generation (AES-ECB or ChaCha20 on 16-byte sample).
- `KeyDerivation` — HKDF-Expand-Label for traffic keys.
- `KeyUpdate` — key phase rotation (RFC 9001 §6) with previous/next generation tracking.
- `NonceGenerator` — packet number nonce construction.
- `SpaceKeys` — per-space key bundles (header + AEAD).
- `ProtectedPacketCodec` — full encode/decode pipeline.
- `RetryIntegrityTag` — retry packet integrity verification.

**TLS 1.3 handshake (24 files):**
- `HandshakeStateMachine` / `HandshakeCoordinator` — orchestrate key exchange and secret derivation.
- `HandshakeKeyExchange` — X25519 ECDH.
- `TranscriptHash` — running hash of handshake messages.
- `ClientHello` / `ServerHello` / `EncryptedExtensions` — TLS message builders.
- `CertificateMessage` / `CertificateVerify` / `FinishedMessage` — certificate chain and signature.
- `CryptoFrameHandler` / `CryptoFrameAssembler` / `CryptoFrameDeliverer` / `CryptoMessageParser` — CRYPTO frame routing.
- `NewSessionTicket` — session resumption.
- `X509Parser` / `CertificateVerifier` — X.509 parsing and chain validation.
- `RevocationPolicy` / `RevocationParser` / `OcspFetcher` / `CrlFetcher` — OCSP/CRL revocation checking.

**Key hierarchy:** Initial → Handshake → Application (1-RTT), with 0-RTT support via `ZeroRttHelper`.

### 3. Recovery (10 files)

**Purpose:** RFC 9002 loss detection, congestion control, RTT estimation, PTO scheduling.

**Key classes:**
- `RecoveryManager` — coordinator for all recovery subsystems.
- `LossDetector` — time threshold (9/8 * RTT) + packet threshold (3) loss detection. Max 10,000 tracked packets.
- `RttEstimator` — smoothed_rtt (7/8 EWMA), rttvar (3/4 EWMA), min_rtt. Initial RTT: 333 ms.
- `PtoScheduler` — PTO = smoothed_rtt + max(4*rttvar, granularity) + max_ack_delay, exponential backoff.
- `CongestionController` — congestion window management (separate from connection CC).
- `SentPacketTracker` — tracks in-flight packets for ACK processing.
- `AckGenerator` — ACK frame building, ACK_FREQUENCY handling.
- `PacingCalculator` / `PacingTimer` — pacing rate computation.
- `PacketNumberSpace` — Initial, Handshake, Application enumeration.

### 4. Streams (8 files)

**Purpose:** Stream multiplexing, flow control, and state machines.

**Key classes:**
- `StreamManager` — routes STREAM frames to streams, manages flow controllers.
- `QuicStream` / `QuicSendStream` / `QuicReceiveStream` — stream abstractions.
- `StreamScheduler` — abstract scheduler interface (ADR-006: pluggable).
- `RoundRobinScheduler` — fair round-robin default.
- `FlowController` — connection and stream flow control. Window updates at 50% threshold. Max 256 MB.
- `SendStateMachine` / `ReceiveStateMachine` — stream state machines.
- `StreamId` / `StreamIdAllocator` — 62-bit stream ID allocation (type bits in [1:0]).
- `ReassemblyBuffer` — out-of-order data reassembly with overlap detection.

### 5. HTTP/3 (27 files)

**Purpose:** RFC 9114 HTTP/3 with QPACK header compression.

**Key classes:**
- `Http3Connection` — connection manager, SETTINGS, GOAWAY, stream lifecycle.
- `Http3Request` / `Http3Response` / `Http3BodyStream` — request/response abstractions.
- `Http3Stream` — stream type classification (control, request, push, QPACK encoder/decoder).

**QPACK (9 files):**
- `QpackEncoder` / `QpackDecoder` — header field encoding/decoding.
- `QpackDynamicTable` — dynamic table with capacity management (32-byte overhead per entry).
- `QpackStaticTable` — RFC 9204 static table (99 entries).
- `QpackEncoderStream` / `QpackDecoderStream` — QPACK control stream instructions.
- `QpackInteger` / `QpackString` / `Huffman` — variable-length integer, string, and Huffman encoding.

**Frames (10 files):**
- `SettingsFrame`, `HeadersFrame`, `DataFrame`, `GoawayFrame`.
- `PushPromiseFrame`, `CancelPushFrame`, `MaxPushIdFrame` — server push.
- `PriorityUpdateFrame` — RFC 9218 priority (urgency 0-7, incremental flag).
- `OriginFrame` — alternative service origins.
- `ExtendedConnectRequest` — WebTransport Extended CONNECT.
- `CapsuleProtocol` — WebTransport capsule handling.

### 6. WebTransport (8 files)

**Purpose:** RFC 9220 WebTransport over HTTP/3 (unreliable datagrams + reliable streams).

**Key classes:**
- `WebTransportSession` — per-session state, capsule routing, flow control.
- `WebTransportSessionManager` — session creation, routing, cleanup.
- `WebTransportFlowController` — session and stream credit management.
- `CapsuleRouter` — capsule type dispatch.

**Capsule types:**
- `DatagramCapsule` — unreliable datagram delivery.
- `StreamCapsule` — stream registration (bidi/uni). Signal values 0x41 (bidi), 0x54 (uni).
- `GoawayCapsule` — session shutdown signal.
- `CloseWebTransportSessionCapsule` — graceful close with error code.
- `DrainWebTransportSessionCapsule` — drain signal.

### 7. libp2p (9 files)

**Purpose:** libp2p QUIC transport with multiaddr, PeerId, and DCUtR NAT traversal.

**Key classes:**
- `Libp2pQuicTransport` — dial/listen API wrapping `QuicEndpoint`.
- `Libp2pQuicConnection` — wrapper around `QuicConnection` with `PeerId`.
- `Multiaddr` / `MultiaddrComponent` — self-describing addresses (`/ip4/addr/udp/port/quic-v1/p2p/peer-id`).
- `PeerId` — multihash of public key (identity for ≤42 bytes, sha2-256 otherwise).
- `MultistreamSelect` — protocol negotiation per stream.
- `Libp2pCertificateGenerator` — self-signed certificate generation.
- `Libp2pTlsExtension` — libp2p TLS extension (signed key, OID 1.3.6.1.4.1.53594.1.1).
- `DCUtRMessage` / `DCUtRHandler` / `DCUtRStateMachine` / `DCUtRUdpCoordinator` — Direct Connection Upgrade through Relay (NAT traversal, UDP hole-punching).

### 8. I/O (12 files)

**Purpose:** UDP socket abstraction, isolate management, platform address handling.

**Key classes:**
- `QuicEndpoint` — primary entry point, binds UDP socket, manages connections.
- `UdpSocket` — UDP socket abstraction (IO vs stub).
- `ConnectionIsolate` / `IsolateSupervisor` — per-connection isolate management (ADR-007).
- `PlatformAddress` — cross-platform address abstraction.
- `UdpRateLimiter` — UDP send rate limiting.

**Platform abstractions (4 IO/stub pairs):** `udp_socket`, `connection_isolate`, `isolate_supervisor`, `platform_address`.

### 9. Wire (12 files)

**Purpose:** QUIC wire format encoding/decoding.

**Key classes:**
- `Frame` / `FrameCodec` — polymorphic frame hierarchy and codec (20+ frame types).
- `PacketHeader` / `LongHeader` / `ShortHeader` / `V2LongHeader` — packet headers.
- `VersionNegotiationPacket` — version negotiation.
- `PacketBuilder` / `CoalescedPacket` / `RetryPacketBuilder` — packet construction.
- `StatelessResetGenerator` — HMAC-SHA256(static_key, connection_id)[0..16].
- `VarInt` — variable-length integer encoding (1/2/4/8 bytes based on 2 MSB).
- `PacketNumber` — packet number reconstruction from truncated values.
- `QuicVersions` — supported QUIC versions (v1, v2).
- `QuicBitGreaser` — reserved bit randomization (RFC 9287).
- `QuicTransportErrorCode` — transport error code enumeration.

**Frame types:** PADDING, PING, ACK, ACK_ECN, RESET_STREAM, STOP_SENDING, CRYPTO, NEW_TOKEN, STREAM, MAX_DATA, MAX_STREAM_DATA, MAX_STREAMS, DATA_BLOCKED, STREAM_DATA_BLOCKED, STREAMS_BLOCKED, NEW_CONNECTION_ID, RETIRE_CONNECTION_ID, PATH_CHALLENGE, PATH_RESPONSE, CONNECTION_CLOSE, APPLICATION_CLOSE, HANDSHAKE_DONE, DATAGRAM, ACK_FREQUENCY.

### 10. Security (2 files)

**Purpose:** Defensive hardening against DoS and amplification attacks.

- `RateLimiter` — sliding-window rate limiter for state transitions (100/sec max).
- `AntiAmplificationLimit` — RFC 9000 anti-amplification (3x receive limit).

### 11. Utils (2 files)

- `bytesToHex()` / `hexToBytes()` — hex encoding/decoding.
- `listEquals()` — element-wise list comparison.

### 12. Logging (1 file)

- `QuicLogger` — configurable log sink (defaults to stdout).

## Notable implementations

### Congestion control
- **CUBIC (RFC 8312):** Cubic function with fast convergence and Hystart++ slow-start exit. Reduces cwnd on CE marks.
- **BBR v1 (RFC 8382):** Model-based with STARTUP/DRAIN/PROBE_BW/PROBE_RTT states. Ignores ECN (v2 would use it).

### TLS 1.3 handshake
- Full X25519 key exchange with HKDF-Extract/Expand-Label.
- Transcript hash for Finished verification.
- Certificate chain validation with OCSP/CRL revocation checking (soft-fail default, ADR-003).
- 0-RTT support with replay protection.
- Key phase rotation with previous/next generation tracking.

### QPACK header compression
- Static table (99 entries from RFC 9204).
- Dynamic table with capacity management and 32-byte overhead per entry.
- Post-base indexed/literal representations.
- Required Insert Count calculation (Errata 8410).

### Flow control
- Connection-level and stream-level controllers.
- Window updates at 50% threshold. Max 256 MB (integer overflow protection).
- Blocking detection via `isBlocked`.

### Connection ID management
- Max 8 active CIDs, sequence number-based retirement.
- Stateless reset token generation.
- Retired CID history (max 32) for duplicate detection.
- Secure random CID generation.

### ECN support
- ECT(0), ECT(1), CE counters and ECN validation state.
- CUBIC reduces cwnd on CE marks; BBR v1 ignores ECN.
- **Blocked:** No `IP_TOS`/`IPV6_TCLASS` socket options in Dart's `RawDatagramSocket` (Issue #10, deferred to v2.0.0).

## Related

- [[ciel/projects/quic_lib/quic_lib.md|quic_lib overview]]
- [[ciel/projects/quic_lib/knowledgebase.md|quic_lib knowledgebase]]
- [[ciel/projects/quic_lib/specs-and-adrs.md|Specs & ADRs]]
- [[ciel/projects/quic_lib/security-and-audits.md|Security & Audits]]
