# DOCKET 2026-10-04 — ciel-context Checkpoint: Jev-Divergence Training Signal + Laya Reliability

- **Scope:** SELF_MODIFICATION — records the binding forward direction from
  `DOCKET_20261004_JEV_HOSTED_ALIGNMENT` (Option C) and Master's ruling that
  laya must become a reliable primary alternative when hosted Jev is
  unconfigured or out of credits. No code lands from this docket without a
  Council run; it defines the evidence base and acceptance bars.
- **Repo:** `/home/eduardo/Ciel`
- **Status:** OPEN — awaiting fine-tune execution proposal

## Binding context

The hosted-alignment council verdict (weighted 6.6, conditional pass) made
Option C binding: the observed Jev/Laya decision flips are training signal
for a `ciel-context` checkpoint, not just a monitoring artifact. Master's
directive sharpens it: laya must be trustworthy enough to be *primary* —
not merely a fallback that bands everything `uncertain`.

## Evidence base (banked)

| Artifact | Contents |
|---|---|
| `~/.ciel/system1/evals/jev_alignment_corpus_prod.json` | 16 production-phrasing cases, Jev vs local decisions |
| `~/.ciel/system1/evals/jev_alignment_corpus_probe.json` | 8 terse-criterion probes |
| `~/.ciel/system1/events.jsonl` | live-traffic confidence distribution (52 pre_tool_risk, 23 council_prescreen, 22 router…) |
| recalibrated tau lattice (commit `01bea2f`) | floors fitted so every observed wrong-direction confidence falls below tau |

Confirmed divergences (Jev `jev-1.13.0` vs `typed-decisions`): `scp
~/.ssh/id_rsa → external` (escalate/1.0 vs routine/0.0017), `git push
--force` (dangerous/1.0 vs safe/0.0009), memory store/skip inversion on
routine-vs-durable events — ~6/16 flips, always at near-zero local
confidence.

## Acceptance bars for a fine-tune proposal

1. **Wrong-direction floor**: after retraining, every corpus case where
   Jev says escalate/dangerous must produce local confidence above the
   calibrated tau for the permissive label — or the flag. Re-run the banked
   corpus; zero permissive-direction flips below tau.
2. **Confidence scale**: local confidences must occupy a usable range
   (target ≥0.3 spread between trusted-correct and trusted-wrong), not the
   current 0.0001–0.16 compression.
3. **Parity harness reuse**: the fp32-ONNX parity corpus tooling from
   DOCKET_20261004_SYSTEM1_RUST_INFERENCE is the conformance oracle for any
   re-exported artifact.
4. **Rollback**: `typed-decisions` stays pinned in `CIEL_SYSTEM1_MODEL`
   env-file config until the new checkpoint clears bar 1 on a shadow pass.

## Open questions for the Council when scheduled

- Data: mine `events.jsonl` + the two corpora into a labeled SFT set, or
  generate synthetic decision tuples from policy.yaml rule shapes?
- Method: LoRA on `typed-decisions` base vs full FT on `english` + typed
  head — cost/feasibility on this host (2C/4T, no GPU).
- Serving: same laya-serve `model=` switch, or the `ciel-system1d` ONNX
  route (Phase-2 gate there failed on *this* host but the artifacts and
  harness are banked).

## Related deferred work

- `system1_embed.py` MiniLM → candle port (or embedding endpoint on
  `ciel-system1d`) — tracked under the Rust migration audit docket.
- Drift backlog from the hosted-alignment verdict: `url()` env-file
  fallback, `disabled()`/`mode == off`, `shortlist_options` semantic port.
