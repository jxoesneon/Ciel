---
title: "IPFS — Platform Abstraction, Utilities & CLI"
project_note: subsystem
type: project-note
tags: ["project-note","subsystem"]
status: active
created: 2026-07-09
updated: 2026-07-09
source: "https://github.com/jxoesneon/IPFS"
---

# IPFS — Platform Abstraction, Utilities & CLI

This note covers the `lib/src/platform/`, `lib/src/utils/`, and `bin/ipfs.dart` subsystems: platform abstraction, cross-platform HTTP server adapters, network-handler platform switching, utility libraries, and the command-line interface.

## Scope

- `lib/src/platform/` — `IpfsPlatform`, HTTP server adapters, libsodium setup, network handler platform exports.
- `lib/src/utils/` — encoding, cryptography, keystore, caching, logging, DNSLink, CAR re-exports.
- `bin/ipfs.dart` — CLI entry point and commands.

## Platform abstraction

`IpfsPlatform` (`lib/src/platform/platform_stub.dart`) defines a unified interface selected at compile time via conditional imports.

```dart
export 'platform_stub.dart'
    if (dart.library.io) 'platform_io.dart'
    if (dart.library.html) 'platform_web.dart'
    show getPlatform;
```

### Interface

- Platform detection: `isWeb`, `isIO`, `operatingSystem`, `version`, `pathSeparator`.
- File system: `writeBytes`, `writeString`, `readBytes`, `readString`, `exists`, `delete`, `createDirectory`, `createTempDirectory`, `listDirectory`, `getLength`.
- System: `promptPassword`.

### IO implementation

`IpfsPlatformIO` (`lib/src/platform/platform_io.dart`) uses `dart:io`:

- Native `File`/`Directory` I/O.
- Platform-specific path separators.
- Secure terminal password prompt with `stdin.echoMode = false`.

### Web implementation

`IpfsPlatformWeb` (`lib/src/platform/platform_web.dart`) uses IndexedDB via `idb_shim`:

- Database `ipfs_storage`, store `files`.
- All operations async.
- Directories emulated via key-prefix matching.
- `promptPassword` returns `null` (no secure terminal in browser).

## HTTP server adapters

| File | Role |
|------|------|
| `http_server_adapter.dart` | Abstract `HttpServerAdapter` + `IpfsHttpServerInstance`. |
| `http_server_adapter_io.dart` | Binds `dart:io` `HttpServer`/`HttpServer.bindSecure` and pipes `shelf` requests. |
| `http_server_adapter_web.dart` | Stub: browsers cannot bind TCP ports. |
| `http_server_adapter_stub.dart` | Fallback stub. |

## Network handler platform switching

`lib/src/core/ipfs_node/network_handler.dart` conditionally exports:

- `network_handler_io.dart` — full P2P networking, circuit relay, AutoNAT dialback, DHT integration.
- `network_handler_web.dart` — stub; raw TCP/UDP is unavailable in browsers (WebRTC not yet implemented).

## Libsodium setup

`lib/src/platform/libsodium_setup.dart` conditionally exports:

- `libsodium_setup_io.dart` — checks for native libsodium; on Windows attempts `winget` install; prints manual instructions.
- `libsodium_setup_stub.dart` — returns `true` on web; pure-Dart crypto is used.

## Utility libraries (`lib/src/utils/`)

| File | Purpose |
|------|---------|
| `base58.dart` | Bitcoin/IPFS Base58 encoding/decoding with BigInt arithmetic. |
| `encoding.dart` | Multibase, CID validation, multicodec registry, hash functions, base32. |
| `varint.dart` | Unsigned LEB128 varint encoding/decoding. |
| `private_key.dart` | ECDSA secp256k1 private key wrapper (Bitcoin/Ethereum compatible). |
| `keystore.dart` | In-memory IPNS keystore with named Ed25519 key pairs and JSON serialization. |
| `password_prompt.dart` | Secure terminal password prompt with strength validation. |
| `generic_lru_cache.dart` | Generic LRU cache + `TimedLRUCache` with TTL; `getOrCompute`. |
| `car_reader.dart` / `car_writer.dart` | Re-exports of `core/data_structures/car.dart`. |
| `dnslink_resolver.dart` | DNSLink → CID via dnslink.io API with timeout. |
| `logger.dart` | Hierarchical logging with levels, JSON option, file logging on IO only. |
| `generate_message_id.dart` | UUID v4 generation. |

## CLI (`bin/ipfs.dart`)

The CLI is compiled with:

```bash
dart compile exe bin/ipfs.dart -o build/ipfs
```

### Command structure

```dart
final runner = CommandRunner<void>('ipfs', 'dart_ipfs command-line interface')
  ..addCommand(DaemonCommand())
  ..addCommand(VersionCommand())
  ..addCommand(IdCommand())
  ..addCommand(HealthcheckCommand())
  ..addCommand(AddCommand())
  ..addCommand(CatCommand())
  ..addCommand(LsCommand())
  ..addCommand(PinCommand())
  ..addCommand(UnpinCommand())
  ..addCommand(SwarmCommand())
  ..addCommand(ConfigCommand());
```

### Key commands

| Command | Role |
|---------|------|
| `daemon` | Starts IPFS daemon with RPC API and HTTP gateway; signal handling for graceful shutdown. |
| `version` | Prints package version. |
| `id` | Prints node identity (peer ID, public key, addresses). |
| `healthcheck` | Queries daemon RPC API. |
| `add` | Adds files/directories to IPFS. |
| `cat` | Retrieves content by CID. |
| `ls` | Lists directory contents. |
| `pin` / `unpin` | Pin/unpin content. |
| `swarm` | P2P network management. |
| `config` | Configuration management. |

### Daemon command flow

1. Parse `--api-addr`, `--gateway-addr`, `--swarm-addr`.
2. Validate localhost binding for RPC (security warning otherwise).
3. Build and merge configuration from file and env vars.
4. Create and start `IPFSNode`.
5. Start gateway server if enabled.
6. Start RPC server if enabled.
7. Wait for `SIGTERM`/`SIGINT`.
8. Graceful shutdown sequence.

## Platform / CLI component diagram

```mermaid
graph TB
    subgraph "CLI"
        CLI[bin/ipfs.dart]
        Daemon[DaemonCommand]
        Other[Other commands]
    end

    subgraph "Platform Abstraction"
        Platform[IpfsPlatform]
        HTTP[HttpServerAdapter]
        NH[NetworkHandler export]
        Libsodium[LibsodiumSetup]
    end

    subgraph "IO Platform"
        Pio[IpfsPlatformIO]
        Hio[HttpServerAdapterIO]
        Nio[NetworkHandlerIO]
        Lio[LibsodiumSetup IO]
        FS[File System]
        TCPS[HttpServer.bind]
        Stdio[stdin/stdout]
    end

    subgraph "Web Platform"
        Pweb[IpfsPlatformWeb]
        Hweb[HttpServerAdapterWeb Stub]
        Nweb[NetworkHandlerWeb Stub]
        Lweb[LibsodiumSetup Stub]
        IDB[IndexedDB]
        NoTCP[No TCP binding]
    end

    CLI --> Daemon
    CLI --> Other
    Daemon --> Platform
    Daemon --> HTTP
    Daemon --> NH

    Platform -.->|conditional| Pio
    Platform -.->|conditional| Pweb
    HTTP -.->|conditional| Hio
    HTTP -.->|conditional| Hweb
    NH -.->|conditional| Nio
    NH -.->|conditional| Nweb
    Libsodium -.->|conditional| Lio
    Libsodium -.->|conditional| Lweb

    Pio --> FS
    Pio --> Stdio
    Hio --> TCPS
    Nio --> TCPS

    Pweb --> IDB
    Hweb --> NoTCP
    Nweb --> NoTCP

    style Platform fill:#e1f5ff
    style Pio fill:#c8e6c9
    style Pweb fill:#ffccbc
```

## Key files

| File | Purpose |
|------|---------|
| `lib/src/platform/platform.dart` | Conditional export entry point. |
| `lib/src/platform/platform_stub.dart` | `IpfsPlatform` interface. |
| `lib/src/platform/platform_io.dart` | IO platform implementation. |
| `lib/src/platform/platform_web.dart` | Web platform implementation. |
| `lib/src/platform/http_server_adapter.dart` | HTTP adapter interface. |
| `lib/src/platform/http_server_adapter_io.dart` | IO HTTP server adapter. |
| `lib/src/platform/http_server_adapter_web.dart` | Web HTTP server stub. |
| `lib/src/core/ipfs_node/network_handler.dart` | Conditional network-handler export. |
| `lib/src/core/ipfs_node/network_handler_io.dart` | IO network handler. |
| `lib/src/core/ipfs_node/network_handler_web.dart` | Web network handler stub. |
| `lib/src/platform/libsodium_setup.dart` | Conditional libsodium export. |
| `bin/ipfs.dart` | CLI entry point and command runner. |

## Related

- [[ciel/projects/IPFS/knowledgebase.md|IPFS — Knowledgebase]]
- [[ciel/projects/IPFS/subsystems/core.md|IPFS — Core subsystem]]
- [[ciel/projects/IPFS/subsystems/storage.md|IPFS — Storage subsystem]]
- [[ciel/projects/IPFS/subsystems/transport.md|IPFS — Transport subsystem]]
