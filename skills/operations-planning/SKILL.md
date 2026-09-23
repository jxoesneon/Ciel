---
name: operations-planning
description: CIEL's framework for demand forecasting, inventory replenishment, and production scheduling.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
---

# CIEL ADAPTATION: Operations Planning (Demand & Production)

This skill formalizes "Flow Control" for physical and digital assets. It applies senior-level heuristics to minimize stockouts and maximize throughput at the system constraint.

## The Planning Pillars

1. **Demand Forecasting**: Use Triple Exponential Smoothing (Holt-Winters) for seasonal items. Apply promotional lifts and post-promo dips.
2. **Safety Stock**: Use `SS = Z × σ_d × √(LT + RP)`. Target 95% for A-items; 90% for C-items.
3. **Production Scheduling (TOC)**:
   - **The Drum**: Identify the bottleneck (utilization > 85%).
   - **The Buffer**: Time-buffer (50% of lead time) to protect the drum.
   - **The Rope**: Release new work only at the drum's consumption rate.

## Operational Workflow

- **Re-sequence on Disruption**: If a machine fails, re-sequence only UNLOCKED jobs. Prioritize constraint-feeding tasks.
- **Changeover Optimization**: Sequence light-to-dark or small-to-large to minimize purge times.
- **Inventory Position**: `IP = On-Hand + On-Order − Backorders`. Never order based on on-hand alone.

## Anti-Patterns

- **Local Optima**: Maximizing utilization of a non-bottleneck resource (creates excess WIP).
- **Amnesiac Forecasting**: Ignoring the post-promotional demand dip.
- **Double Ordering**: Placing a new PO while an existing one is still in transit.
