# TIER_3 — Web Extraction + Synthesis

When Tiers 1 and 2 miss, go to the open web. Highest risk, lowest trust, strictest sandbox.

## Flow

1. **Research** — `seed_skills/research/SKILL.md` performs multi-source synthesis (GitHub, docs, blogs, Stack Overflow) on the gap description.
2. **Extract candidates** — code snippets, shell sequences, API calls that achieve the goal.
3. **Synthesize** — use `prompts/acquisition/web_extraction.md` to assemble a draft `.skill` bundle.
4. **Harmonize** — `HARMONIZATION.md`.
5. **Trust judge** — `prompts/acquisition/trust_judgment.md` pre-screens: is this safe to sandbox-test? If hard-no, discard now.
6. **Sandbox** — `SANDBOX.md`. Isolated run with synthetic inputs.
7. **Council** — `council/invocation_scopes/SKILL_INTEGRATION.md` with full trace.
8. On pass: register. On reject: discard + lower source trust.

## Source Ranking

- **GitHub** with stars, activity, tests, CI: highest.
- **Official docs** of a tool: high.
- **Well-known tech blogs** with code: medium.
- **Stack Overflow** with accepted + upvoted answer: medium.
- **Random blog / gist**: low; requires extra sandbox rigor.

## Web Fetch

`seed_skills/web_fetch/SKILL.md`. All domains logged. Fetches respect robots.txt. Bodies stored hashed in MemPalace for audit.

## Citation

Every synthesized skill carries a `source.origin` list of URLs + fetch hashes. Safety member rejects any skill with missing provenance.

## License Sniffing

`seed_skills/dependency_audit/SKILL.md` scans for license markers in fetched content. Incompatible licenses (GPL into Apache-2.0 Ciel) → reject before Council.

## Never Auto-Execute During Fetch

Content is treated as text. No execution occurs before harmonization, trust gate, and sandbox.

## Cost

Tier 3 typically consumes the most tokens and wall time. Budgets in `acquisition.config.md` hard-cap the session.
