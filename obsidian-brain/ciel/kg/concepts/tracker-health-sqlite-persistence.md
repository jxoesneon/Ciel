---
title: Tracker Health SQLite Persistence
type: concept
tags: [concept, x-seed, flutter, sqlite, persistence]
created: 2026-07-11
status: active
---

# Tracker Health SQLite Persistence

## Definition

A pattern for persisting in-memory tracker health statistics to SQLite so they survive app restarts, while keeping the in-memory collection as the source of truth during a session.

## The Pattern

1. **Provide a database access object** through the class constructor (e.g., `SqfliteService? sqfliteService`).
2. **Hydrate on construction.**
   - Call `load()` / `_load()` to read existing rows into the in-memory map.
3. **Persist on every update.**
   - After any `_stats` assignment, `_persist(stats)` writes the row to SQLite.
   - Await the write before continuing so crashes do not lose data.
4. **Delete on eviction.**
   - When a tracker is pruned as dead, delete its persisted row.
5. **Test both paths.**
   - Unit tests for the monitor: load, successful/failed probe save, prune deletion, and fallback when no DB is wired.
   - Unit tests for the database service: insert/update, delete, and null-timestamp round-trip.

## Why It Matters

Tracker health is built up over many scraping cycles. Losing it on restart forces expensive re-probing. SQLite persistence keeps the health model warm and user-visible latency low.

## Related

- [[ciel/diary/2026-07-11-xseed-tracker-health-persistence]]
- [[ciel/projects/X-Seed/X-Seed]]
- [[verification-commands]]
