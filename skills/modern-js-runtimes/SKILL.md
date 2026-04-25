---
name: modern-js-runtimes
version: 1.0.0
format: skill/1.0
description: Fast JS/TS development using Bun and specialized hashing safety for Ethereum.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(build|setup|run).*(bun|runtime|hashing|keccak)"
    confidence: 0.9
  - pattern: "bun install"
    confidence: 1.0
---

# CIEL ADAPTATION: Modern JS Runtimes (Bun & Keccak)

This skill manages high-speed JS/TS development using Bun and enforces cryptographic correctness for specialized domains (Ethereum).

## The Bun Stack
- **All-in-One**: Use Bun as the Runtime, Package Manager (`bun install`), and Test Runner (`bun test`).
- **Natiive TS**: Run `.ts` files directly without a separate transpilation step.
- **Performance**: Prefer Bun's native APIs (e.g., `Bun.file()`, `Bun.serve()`) over Node equivalents for hot paths.

## Hashing Integrity (Ethereum)
- **Hard Gate**: For Ethereum contexts (selectors, signatures, storage slots), NEVER use Node's `crypto.createHash('sha3-256')` (NIST SHA3).
- **Mandate**: Use `keccak256` from `ethers`, `viem`, or `web3`.
- **Audit**: Grep for `createHash.*sha3` in Ethereum-related packages to find silent bugs.

## Operational Standards
- **Lockfiles**: ALWAYS commit `bun.lock` (text) or `bun.lockb` (binary) for reproducible installs.
- **Vercel**: Explicitly set the runtime to Bun in project settings for 2x deployment speed.

## Anti-Patterns
- **Mixing Managers**: Using `npm` or `yarn` in a Bun-initialized project (corrupts lockfiles).
- **Amnesiac Hashing**: Assuming "SHA3" means "Keccak" in JS (NIST SHA3 != Keccak-256).
- **Silent Bun Failure**: Ignoring Bun-specific dependency issues; fallback to Node if compatibility is broken.
