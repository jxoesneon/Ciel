//! Detached System-1 shadow spawn — replicates `risk_policy
//! .system1_shadow_async` semantics parent-side: build the pre_tool_risk
//! payload (deterministic `tool_state` enrichment + question set) and hand
//! it to `system1::ask_async`, which owns the marker/inflight-cap/detached
//! child contract.

use serde_json::{json, Value};

use crate::system1;

fn questions() -> Value {
    json!({
        "risk": {
            "type": "choice",
            "instructions": "Is this tool call dangerous? Judge BOTH the command AND the file path it targets — a write to a sensitive path is dangerous even with no command.",
            "criteria": {
                "safe": "routine, reversible, or read-only operation on non-sensitive paths",
                "dangerous": "destructive, irreversible, privilege-escalating, credential/sensitive-path writing, or data-exfiltrating operation",
            },
        }
    })
}

/// Fire-and-forget shadow ask — mirror of `risk_policy.system1_shadow_async`:
/// payload shape identical (surface, tool_state, questions, meta) then
/// `system1.ask_async`.
pub fn shadow_async(
    runtime: &str,
    ts: &str,
    tool: &str,
    command: &str,
    path: &str,
    regex_decision: &str,
    rule_id: &Value,
) {
    system1::ask_async(&json!({
        "surface": "pre_tool_risk",
        "state": system1::tool_state(tool, command, path),
        "questions": questions(),
        "meta": {
            "ts": ts,
            "runtime": runtime,
            "regex_decision": regex_decision,
            "rule_id": rule_id,
        },
    }));
}
