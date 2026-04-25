---
name: analytical-data-ops
version: 1.0.0
format: skill/1.0
description: ClickHouse analytics patterns, OLAP optimization, and data pipelines.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(design|optimize|query).*(clickhouse|olap|analytics|mergetree)"
    confidence: 0.95
  - pattern: "materialized view"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Analytical Data Ops (ClickHouse & Pipelines)

This skill formalizes high-performance analytics. it prioritizes column-oriented efficiency and real-time pre-aggregation.

## Table Design (MergeTree)
1. **Engines**: Default to `MergeTree`. Use `ReplacingMergeTree` for deduplication; `AggregatingMergeTree` for pre-computed stats.
2. **Partitioning**: Partition by time (e.g., `toYYYYMM(date)`). Avoid excessive partitions (> 1000).
3. **Ordering Key**: Put frequently filtered, high-cardinality columns first.

## Query Optimization
- **Indexed Filter**: Use primary key columns first in `WHERE` clauses.
- **Aggregate Merge**: Use `sumMerge`, `countMerge`, etc., when querying `AggregatingMergeTree` tables.
- **Quantiles**: Use `quantile(0.95)(value)` for efficient percentile calculation.

## Ingestion Pipelines
- **Batch Mandate**: NEVER perform individual `INSERT` in a loop. Batch 1000+ rows per request.
- **Materialized Views**: Use `CREATE MATERIALIZED VIEW ... TO table` for real-time aggregation.

## Anti-Patterns
- **SELECT \***: Reading every column in an OLAP database (causes massive I/O bloat).
- **Too Many Joins**: Performing complex joins on large tables. Denormalize early for analytics.
- **Small Inserts**: Frequent, low-row inserts causing background merge overload.
