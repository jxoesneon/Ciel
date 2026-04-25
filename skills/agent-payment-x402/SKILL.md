---
name: agent-payment-x402
version: 1.0.0
format: skill/1.0
description: Protocol and tools for autonomous agent payments using x402, featuring strict spending policies and non-custodial wallets.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(agent|autonomous).*(payment|pay|wallet|spend|budget)"

    confidence: 0.9

  - pattern: "x402"

    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Agent Payment Execution (x402)

This skill enables CIEL sub-agents to execute autonomous payments via the x402 HTTP payment protocol. It prioritizes safety through mandatory spending policies and non-custodial wallet management.

## Core Mandates

### 1. Policy-First Delegation

The Orchestrator MUST set a `SpendingPolicy` before delegating any payment authority to a sub-agent. Sub-agents are prohibited from setting or escalating their own spending limits.

- **Per-Task Budget**: Maximum spend for a single atomic action.
- **Per-Session Budget**: Cumulative limit for the entire epic.
- **Allowlist**: Restricted set of verified recipients or service endpoints.

### 2. Non-Custodial Security

CIEL agents use non-custodial smart accounts (e.g., ERC-4337). Private keys are managed by the harness, and the sub-agent only has signed permission to spend within the pre-defined policy bounds.

### 3. Fail-Closed Protocol

If the payment service (x402) is unreachable, the transaction MUST fail. CIEL agents MUST NOT attempt to bypass payment gates or fall back to unmetered access without explicit user authorization.

## Workflow

1. **Pre-Check**: Query remaining budget (`check_spending`) before initiating any paid action.
2. **Negotiation**: Handle `402 Payment Required` responses by negotiating price and checking against the Per-Task Budget.
3. **Execution**: Invoke `send_payment` via the designated MCP tool.
4. **Audit**: Log all transactions (`list_transactions`) to the MemPalace Diary for session auditing.

## Anti-Patterns

- **Unlimited Spend**: Delegating to an agent without a hard session budget.
- **Self-Escalation**: Allowing an agent to modify its own spending policy.
- **Speculative Payment**: Sending funds before receiving a verified `402` or invoice.
