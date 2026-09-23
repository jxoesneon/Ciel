---
name: billing-and-monitoring
description: CIEL's framework for customer billing remediation and operational dashboards.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
---

# CIEL ADAPTATION: Billing & Monitoring (The Revenue Layer)

This skill formalizes the intersection of customer health and system observability. it prioritizes reversible remediation and question-driven dashboards.

## Customer Billing Ops

1. **Identify**: Resolve the customer via email or Stripe ID. Map all active/canceled subscriptions and invoices.
2. **Classify**: Distinguish among Duplicate Purchase, Multi-seat Intent, Failed Checkout, or Product Failure.
3. **Remediate**: Perform the safest reversible action first (e.g., Restore self-serve portal access).
4. **Document**: Produce an "Operator Handoff" with the action taken, revenue impact, and follow-up message.

## Dashboard Engineering

- **Operator Questions**: Start with "Is it healthy?", "Where is the bottleneck?", "What changed?".
- **Structure**: Group by Overview -> Performance -> Resources -> Service-Specific.
- **Mandate**: Every panel MUST have units, a title, and meaningful threshold colors (Red/Yellow/Green).
- **Curation**: Remove any panel that doesn't drive a specific operator action.

## Deployment Readiness

- **Canary Check**: Pair billing remediation with a post-fix Canary watch to ensure no regressions.
- **Impact Audit**: Measure the revenue impact of any proposed changes to subscription tiers.

## Anti-Patterns

- **Blind Refund**: Refunding a charge without identifying if it was a deliberate team purchase.
- **Vanity Panels**: Creating a dashboard with 50 CPU graphs but no "Error Rate" or "Latency" metrics.
- **Secrets in Dashboards**: Hardcoding API tokens or DB credentials in Grafana JSON exports.
