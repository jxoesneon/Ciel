---
name: ciel-ml-and-data-patterns
version: 1.0.0
format: skill/1.0
description: CIEL's framework for PyTorch ML patterns and Postgres data optimization.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(design|build|optimize).*(pytorch|tensor|postgres|sql|ml)"
    confidence: 0.95
  - pattern: "device-agnostic code"
    confidence: 1.0
---

# CIEL ADAPTATION: ML & Data (The Intelligence Layer)

This skill manages high-performance data patterns, from deep learning training loops to SQL query optimization.

## PyTorch ML Patterns
1. **Device-Agnostic**: ALWAYS use `device = torch.device(...)`. Prohibit hardcoded `.cuda()` calls.
2. **Reproducibility**: Set seeds for `torch`, `np`, and `random` in a central `set_seed()` function.
3. **Shape Integrity**: Annotate and verify tensor shapes in the `forward()` pass comments.
4. **Efficiency**: Use `optimizer.zero_grad(set_to_none=True)` and `model.eval()` for validation.

## Postgres Data patterns
- **Indexing**: Equality columns first, then range columns. Use `GIN` for JSONB and `BRIN` for time-series.
- **Types**: Use `bigint` for IDs, `timestamptz` for times, and `text` for variable strings.
- **Pagination**: Use keyset/cursor pagination (`WHERE id > $last_id`) instead of `OFFSET` for O(1) performance.
- **Security**: Wrap RLS policies in `(SELECT auth.uid()) = user_id` for optimization.

## Memory Management
- **AMP**: Use `torch.amp.GradScaler` for mixed-precision performance.
- **Checkpointing**: Save `model_state_dict` AND `optimizer_state_dict` to allow resuming training.

## Anti-Patterns
- **In-place Mutation**: Using `x += residual` in PyTorch (breaks autograd). Use `x = x + residual`.
- **Small Inserts**: Performing SQL inserts in a loop instead of batching.
- **Select \***: Reading every column in SQL (causes unnecessary I/O bloat).
