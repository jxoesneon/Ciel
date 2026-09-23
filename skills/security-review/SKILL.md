---
name: security-review
description: A comprehensive security checklist and architectural patterns for CIEL development.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
---

# CIEL ADAPTATION: Security Review (Quality Gate)

This skill formalizes the "Security Gate" for CIEL. It provides non-negotiable patterns for secrets management, input validation, and vulnerability prevention.

## The Security Mandates

### 1. Secrets Management

- **Fail**: Hardcoded keys in source or session notes.
- **Pass**: Use `process.env` or secrets managers. Validate presence at startup.
- **Audit**: Use `grep_search` for common secret patterns (e.g., `sk-`, `password=`) before any commit.

### 2. Input Validation (The Zod Pattern)

- All user-controlled input MUST be validated using a schema-first approach (e.g., Zod, Pydantic).
- **Whitelist Only**: Validate against a list of allowed characters/types, never a blacklist.
- **File Sinks**: Validate size, MIME type, and extension before writing to disk.

### 3. SQL Injection (Parameterized Queries)

- String concatenation in SQL is a CRITICAL violation.
- All database interactions MUST use parameterized queries or type-safe ORMs.

### 4. XSS & CSRF

- **HTML**: Sanitize all user-provided HTML using `isomorphic-dompurify` or equivalent.
- **Cookies**: Use `HttpOnly`, `Secure`, and `SameSite=Strict` for all session tokens.
- **CSRF**: Enforce CSRF token validation for all state-changing `POST/PUT/DELETE` operations.

### 5. Blockchain Safety (Solana/EVM)

- **Signature Check**: Verify wallet signatures before executing on-chain actions.
- **Balance Gate**: Perform pre-transaction balance checks to prevent revert-spamming.

## Anti-Patterns

- **Logging the World**: Printing full objects that might contain `password`, `token`, or `cvv`.
- **Stack-Trace Leaks**: Returning raw error objects to the client.
- **Blind Merging**: Merging dependencies without running `npm audit`.
