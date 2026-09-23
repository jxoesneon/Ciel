---
name: crypto-and-trading-security
description: CIEL's framework for trading agent safety, spend limits, and Solidity integrity.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
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
