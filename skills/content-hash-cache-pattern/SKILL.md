---
name: content-hash-cache-pattern
version: 1.0.0
format: skill/1.0
description: An architectural pattern for caching expensive file processing results using SHA-256 content hashes.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "cache (expensive|file|processing|pdf|ocr|parsing)"
    confidence: 0.9
  - pattern: "content-hash caching"
    confidence: 1.0
---

# CIEL ADAPTATION: Content-Hash Cache (I/O Optimization)

This skill provides a standard architectural pattern for caching the results of expensive, deterministic file processing operations (e.g., PDF parsing, OCR, large-file indexing).

## Core Pattern: Content over Path
Unlike traditional path-based caching, this pattern uses the SHA-256 hash of the **file content** as the cache key.
- **Rename/Move Resilience**: Moving a file results in a cache hit.
- **Automatic Invalidation**: Changing a single byte in a file triggers a cache miss.
- **No Index File**: Cache files are named `{hash}.json`, allowing O(1) lookup without a central registry.

## Implementation Standard
1. **Compute Hash**: Use chunked reads (64KB) for large files to avoid memory exhaustion.
2. **Read/Check**: Check if `.cache/{hash}.json` exists before processing.
3. **Process (Pure)**: The processing logic itself should be a pure function with no knowledge of the cache.
4. **Write**: Store the result alongside the `source_path` and `file_hash` in a structured JSON file.

## Design Decisions
- **Fail Gracefully**: Treat corrupted or malformed JSON in the cache as a miss; never crash.
- **Service Layer**: Implement caching as a wrapper/service layer around the core processing logic.
- **Chunked I/O**: Mandatory for files >10MB to maintain efficiency.

## Anti-Patterns
- **Path-Based Caching**: Caching by filename, which leads to stale data after edits or renames.
- **In-Memory Only**: Using simple dictionaries for cache that disappear after a session restart.
- **Impure Cache Hooks**: Mixing caching logic directly into business logic/parsers.
