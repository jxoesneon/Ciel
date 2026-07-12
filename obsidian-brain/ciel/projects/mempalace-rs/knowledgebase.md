---
title: mempalace-rs — Knowledgebase
project_note: knowledgebase
type: project-note
tags: ["project-note","knowledgebase"]
status: active
created: 2026-07-09
updated: 2026-07-09
source: "https://github.com/jxoesneon/mempalace-rs"
---

# mempalace-rs — Knowledgebase

Synthesized expansion from the read-only subagent exploration of the local clone.

## Summary

`mempalace-rs` is a high-performance, offline-first AI memory and retrieval system written in Rust. It mines local projects and conversation transcripts into a structured “palace” combining vector search, relational SQLite storage, and a temporal knowledge graph. It exposes 20 MCP tools and a CLI for mining, search, repair, compression, and benchmarks. The Rust port has achieved 100% parity with the upstream Python implementation while introducing a pure-Rust storage engine and advanced compression.

## Local clone

| Field | Value |
|-------|-------|
| GitHub | `jxoesneon/mempalace-rs` |
| Local path | `C:/Users/josee/mempalace-rs` |
| Version | 0.5.0 |
| Language | Rust (edition 2021) |
| Visibility | PUBLIC |
| License | MIT |
| Stars | 34 |

## Top-level structure

- `Cargo.toml` — binary crate with storage, async, MCP, CLI dependencies; security patches.
- `build.rs` — Windows-specific workaround for usearch `MAP_FAILED` issue.
- `src/` — 70+ modules including storage, vector storage, knowledge graph, MCP server, CLI, dialect, miner, searcher, embedder factory, entity detector, palace graph, etc.
- `tests/`, `benches/`, `examples/`, `scripts/`.
- `hooks/` — Claude Code integration hooks.
- `patches/` — dependency patches.
- `upstream/` — Python upstream reference implementation and RFCs.
- `docs/` — parity reports, ADRs.
- `README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `SKILLS_GUIDE.md`, `src/AGENTS.md`.

## Architecture

### Dual storage engine

1. **VectorStorage** (`src/vector_storage.rs`)
   - Embeddings: `fastembed` (AllMiniLML6V2, 384-dim, CPU/ONNX).
   - Vector index: `usearch` HNSW ANN.
   - Metadata: SQLite relational source of truth.
   - Temporal validity (`valid_from`/`valid_to`), decay-based importance scoring, auto-repair.
   - Capacity: 100,000 memories.

2. **Relational Storage** (`src/storage.rs`)
   - SQLite for wings, rooms, diary entries, palace metadata.
   - Memory stack L0–L3:
     - L0 Identity (~100 tokens)
     - L1 Essential events (~500–800 tokens, recency-biased)
     - L2 On-demand similarity search
     - L3 Raw semantic search

### Knowledge graph

- SQLite-backed temporal KG (`src/knowledge_graph.rs`).
- Entities and triples with `valid_from`/`valid_to`, confidence, source.
- Bidirectional relationship queries.

### API surfaces

- **MCP server** (`src/mcp_server.rs`): 20 tools prefixed `mempalace_`, JSON-RPC over stdio, WAL logging, auto-repair.
- **CLI** (`src/main.rs`): `init`, `mine`, `search`, `repair`, `compress`, `wakeup`, `split`, `prune`, `status`, `instructions`, `mcp-server`, `benchmark`.

### AAAK dialect

- `src/dialect.rs` — compression format V:3.2, ~30x token reduction.
- Entity codes, emotion codes, adaptive density, delta encoding, temporal decay, faithfulness scoring.

## Build / test / verify

```bash
cargo build
cargo build --release
cargo test
MEMPALACE_TEST_MODE=1 cargo test

# Coverage
cargo llvm-cov --lib --summary-only
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Lint
cargo fmt
cargo clippy --lib --bin mempalace-rs

# Benchmarks
cargo run -- benchmark ruler --k 10
cargo run -- benchmark structmem --hints
cargo run -- benchmark babilong --tokens 1000000
cargo run -- benchmark beam
```

Verification checklist (from `src/AGENTS.md`):

```bash
cargo build --lib
cargo build --bin mempalace-rs
cargo test --lib
cargo test --bin mempalace-rs
cargo clippy --lib --bin mempalace-rs
```

## Recent git state (manual snapshot)

- **Current version:** 0.5.0 (2026-05-01).
- **Working tree:** many untracked new source modules (`src/AGENTS.md`, `backups.rs`, `closet_llm.rs`, `collision_scan.rs`, `convo_scanner.rs`, `corpus_origin.rs`, `daemon.rs`, `dedup.rs`, `diary_ingest.rs`, `dynamics.rs`, `embedding.rs`, `exporter.rs`, `fact_checker.rs`, `format_miner.rs`, `general_extractor.rs`, `hallways.rs`, `hooks_cli.rs`, `i18n.rs`, `instructions_cli.rs`, `layers.rs`, `llm_client.rs`, `llm_refine.rs`, `migrate.rs`, `project_scanner.rs`, `query_sanitizer.rs`, `room_detector_local.rs`, `service.rs`, `shared.rs`, `sources.rs`, `sweeper.rs`, `sync.rs`, `wal.rs`, etc.) plus `upstream/`.
- **Recent commits:**
  - `a9d36d8` docs: remove AI/agent attributions from documentation and source comments
  - `b177612` fix(mcp): remove AI attribution from diary_write and accept summary
  - `437b174` docs: autonomously update 2026 benchmarks [skip ci]
  - `8fb16e4` chore: apply formatting and clippy fixes for v0.5.0 release
  - `36b873b` docs: autonomously update 2026 benchmarks [skip ci]

## Performance / benchmarks

- 2026 Gold Standard: perfect 1.000 on RULER, BABILong, BEAM, StructMemEval.
- Micro-benchmarks (Apple M4): AAAK compression ~1808 ops/sec, entity detection ~267k ops/sec, token counting ~3.8M ops/sec.
- Release binary: 7.9 MB; cold start ~300 ms; baseline memory ~50 MB.

## Security patches

- usearch Windows MAP_FAILED fix.
- `core2` RUSTSEC-2026-0099.
- `tungstenite` RUSTSEC-2026-0097.
- `rustls-webpki` RUSTSEC-2026-0098/0099.

Code fixes: service token leak in `/health`, WAL redaction, `add_drawer` preview redaction, arbitrary-path ingestion, config path canonicalization.

## Relationship to Obsidian brain

No direct Obsidian migration exists in the Rust codebase. RFC 002 (Source Adapter Plugin Spec) in `upstream/` lists Obsidian as a planned knowledge-work source adapter; the architecture supports pluggable adapters but no Obsidian-specific logic is present yet.

## Key files for deeper context

1. `src/AGENTS.md` — rules, verification, security audit.
2. `src/storage.rs` — memory stack L0–L3.
3. `src/vector_storage.rs` — pure-Rust vector engine.
4. `src/knowledge_graph.rs` — temporal KG.
5. `src/mcp_server.rs` — MCP tool handlers.
6. `src/dialect.rs` — AAAK V:3.2 compression.
7. `src/searcher.rs` — hybrid BM25 + vector search.
8. `src/miner.rs` — project/conversation mining.
9. `docs/parity_report.md` — Rust vs Python parity.
10. `upstream/docs/rfcs/002-source-adapter-plugin-spec.md` — source adapter spec.

## Related

- [[ciel/projects/mempalace-rs/mempalace-rs.md|mempalace-rs overview]]
- [[ciel/projects.md|Projects index]]
