---
name: energy-procurement
description: C&I energy spend management, tariff optimization, and PPA evaluation.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
---

# CIEL ADAPTATION: Energy Procurement (C&I Strategy)

This skill formalizes the management of large-scale energy portfolios. it prioritizes budget certainty and total-cost optimization over simple rate hunting.

## The Cost Pillar

- **Energy (40-55%)**: Procure via Fixed, Index, or Block-and-Index.
- **Demand (20-40%)**: Mitigate via Load Shifting or Battery Storage.
- **Capacity**: Reduce PLC during grid coincident peak hours.

## Procurement Workflow

1. **Profile**: Analyze 15-minute interval data to find peak cost drivers.
2. **Layer**: Use tranche-based purchasing (e.g., 25% tranches) to dollar-cost average.
3. **Hedge**: Target 60-80% hedged, 20-40% index based on risk tolerance.

## Renewable Strategy (PPA/REC)

- **VPPA**: Contract-for-differences. Quantify "Basis Risk" (Node-to-Hub spread) before signing.
- **RECs**: Claim renewable attributes. Prefer bundled RECs (Additionality) for SBTi alignment.

## Anti-Patterns

- **The Ratchet Trap**: Accidental peak spikes locking in high billing demand for 11 months.
- **Node Blindness**: Signing a PPA in a congested zone where basis risk erodes all value.
- **Budget Chasing**: Trying to "call the bottom" of the gas/power market instead of layering.
