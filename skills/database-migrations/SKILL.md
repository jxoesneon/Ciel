---
name: database-migrations
version: 1.0.0
format: skill/1.0
description: Safe, zero-downtime database schema changes and data backfills.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(create|alter|migrate|update).*(table|schema|database|column)"
    confidence: 0.95
  - pattern: "database migrations"
    confidence: 1.0
---

# CIEL ADAPTATION: Database Migrations (Schema Safety)

This skill formalizes the process for making reversible, zero-downtime database changes. It prioritizes table safety over implementation speed.

## Core Mandates
1. **Forward-Only**: In production, rollbacks use a NEW forward migration.
2. **Downtime Prevention**:
   - **Adding Column**: Must be nullable or have a default (Postgres 11+).
   - **Adding Index**: Use `CREATE INDEX CONCURRENTLY` (Postgres).
   - **Renaming**: Use the **Expand-Contract Pattern** (Add new -> Backfill -> Read/Write Both -> Contract).
3. **Data/Schema Split**: Never mix DDL (Schema) and DML (Data) in a single migration.

## Batch Data Migration
For large tables (>10M rows), the Orchestrator MUST use a batching loop:
- Update in chunks of 5,000-10,000 rows.
- Use `FOR UPDATE SKIP LOCKED` where appropriate.
- Log progress and include a manual halt mechanism.

## Tool-Specific Standards
- **Prisma**: Use `prisma migrate dev` for schema; manual SQL for `CONCURRENTLY`.
- **Drizzle**: Generate migrations into dedicated files; verify ordering.
- **Django**: Use `SeparateDatabaseAndState` to decouple model state from DB state during complex drops.

## Anti-Patterns
- **Inline Table Rewrites**: Adding a `NOT NULL` column without a default to an existing large table.
- **Manual SQL**: Executing DDL directly in production without a migration file.
- **Amnesiac Rollbacks**: Marking a migration as irreversible without documenting the specific reason.
