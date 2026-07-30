# GROWTH SIGNAL: Agentic Loop Standards & Best Practices

**Date**: 2026-07-30
**Trigger**: User directive & Industry Research Synthesis
**Category**: Core Loop & Autonomy Specification

---

## 1. Industry Research Synthesis: Agentic Loop Best Practices & Standards

Based on modern 2025/2026 AI Agent engineering standards:

### Architectural Topologies & Control Flow
- **Plan-and-Execute with Dynamic Re-Planning**: Generate structured plans upfront; dynamically adjust steps when new findings or failures occur during execution.
- **Reflection & Self-Critique**: Run automated self-evaluation (critic pass) before declaring execution complete.
- **Governance-by-Design & Observability**: Maintain complete trajectory visibility (decision paths, tool arguments, reasoning steps) logged to activity trails.
- **Context & Memory Management**: Maintain strict boundary separation between transient context, episodic memory (`.ciel/`), and long-term knowledge (`~/.ciel/` via `mempalace-rs`).

---

## 2. Core Execution Mandates (Non-Negotiable Rules)

### A. Zero-Deferral & Comprehensive Implementation Mandate
- **No TODOs / No Placeholders**: Every agentic loop MUST produce complete, production-ready, fully implemented code and documentation. Mock responses, stubbed functions, "left for future work", or deferred TODO items are strictly forbidden.
- **No Pending Work**: Any sub-task initiated within a loop must be brought to full resolution before closing the loop.

### B. Adaptive Problem Scope Integration
- **Incorporate Pre-Existing Issues**: If pre-existing bugs, syntax errors, security vulnerabilities, or broken contracts are discovered during research or execution, the loop MUST adapt to resolve or incorporate those findings rather than ignoring them or bypassing them.

### C. Standard Ciel Escalation Path
Execution failures or ambiguities must strictly follow Ciel's 3-step escalation sequence:

```text
Step 1: Deep Online & Local Research
  │ (If research yields insufficient confidence)
  ▼
Step 2: Escalation to Ciel Council of Five
  │ ├── General Systems → Main Ciel Council (Coherence, Capability, Safety, Efficiency, Evolution)
  │ └── UI/UX Specifics → Specialized Design Council (Clarity, Inclusion [Veto ≤ 3], Efficiency, Aesthetics, Actionability)
  │ (If Council cannot resolve, deadlocks, or risk is Critical)
  ▼
Step 3: User Escalation (Last Resort)
  └── Direct user escalation via interactive prompt / ask tool with complete context & proposed options.
```

---

## 3. Council Audit & Self-Improvement Integration

- **Safety & Coherence**: Eliminates half-baked / placeholder code; enforces complete implementation verification.
- **Efficiency**: Reduces back-and-forth loops by enforcing exhaustive initial research before user intervention.
- **Evolution**: Formally incorporated into Ciel's core autonomy (`core/AUTONOMY.md`) and router (`router/ROUTER.md`).

---
**Status**: PROPOSED & APPLIED.
