---
name: crypto-and-trading-security
version: 1.0.0
format: skill/1.0
description: CIEL's framework for trading agent safety, spend limits, and Solidity integrity.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(build|secure|audit).*(trading|bot|wallet|solidity|transaction)"
    confidence: 0.95
  - pattern: "spend limit guard"
    confidence: 1.0
---

# CIEL ADAPTATION: Crypto & Trading Security (The Asset Layer)

This skill manages the extreme threat model of autonomous trading agents and DeFi contracts.

## Trading Agent Guardrails
1. **Injection Shield**: Sanitize all on-chain data (Token names, pair labels) to prevent instruction overrides.
2. **Hard Spend Limits**: Enforce `MAX_SINGLE_TX_USD` and `MAX_DAILY_SPEND_USD` outside the LLM context.
3. **Simulation Mandate**: ALWAYS `eth_call` or simulate a transaction before sending. check `actual_out >= min_amount_out`.
4. **Circuit Breaker**: Halt agent execution on > 3 consecutive losses or > 5% hourly drawdown.

## Solidity Integrity (DeFi)
- **CEI Order**: Checks-Effects-Interactions. update state before transferring tokens.
- **Reentrancy**: Use `nonReentrant` on all external transfer entry points.
- **Decimal Safety**: Prohibit assuming 18 decimals. Query `decimals()` at runtime and cache by `(chain_id, token_address)`.
- **WAD Normalization**: Standardize internal math to 18 decimals internally.

## Wallet Isolation
- **Hot Wallet**: Agent MUST only have access to a dedicated hot wallet. PROHIBIT access to treasury or primary keys.
- **Private RPC**: Use `flashbots` or protected RPCs to prevent frontrunning/MEV.

## Anti-Patterns
- **Model-Controlled Budget**: Letting the LLM decide its own spend limits.
- **Naive mulDiv**: Performing `a * b / c` without checking for overflow in high-value reserves.
- **Silent Revert**: Catching transaction errors without halting the agent or notifying the user.
