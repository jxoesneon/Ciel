# Ciel Changelog

## 2026-07-31 — Hook Autonomy Grant (User Escalation, Level 3)

**Trigger:** User explicit escalation — "ensure you are allowed to run all" after repeated permission prompts blocked legitimate tool calls.

**Decision:** Relax Ciel PreToolUse + PermissionRequest hooks from "ask" to "approve" for elevated-risk operations (network calls, sensitive-path writes), preserving only the critical-risk destructive-command block as the hard safety floor.

**Rationale:** Devin config already has `"permissions": { "allow": ["*"] }`. Ciel hooks were overriding this with `"decision":"ask"` for (a) writes to `.config/` and other sensitive paths, and (b) any network call (curl/wget/nc/telnet/ftp/scp/rsync). The network regex also had false positives — unanchored `ftp` matched inside words like `FAILED_ID=...`, triggering spurious permission prompts.

**Changes:**
- `ciel_preflight.sh`: HIGH-RISK and MID-RISK gates changed from `decide "ask"` to `decide "approve"` with elevated-risk audit logging. Network regex word-boundary anchored (`\bcurl\b` etc.) to eliminate false positives. CRITICAL-RISK block (rm -rf /, mkfs, dd to device, fork bombs, piped-shell remote execution) preserved as hard floor.
- `ciel_permission.sh`: Default changed from `decide "pass"` to `decide "approve"` with audit logging. Safe-pattern fast-path preserved.

**Verified:**
- Destructive commands (rm -rf /, piped remote shell execution) → BLOCKED
- Network calls (curl https://example.com/api) → APPROVED
- Writes to sensitive paths (/Users/me/.config/...) → APPROVED
- Build commands (cargo build) → APPROVED
- False-positive case (echo FAILED_ID=abc) → APPROVED (no longer triggers network gate)
- Unknown commands → APPROVED via full-autonomy default

**Safety floor preserved:** Destructive/critical-risk commands remain blocked at PreToolUse. All elevated-risk approvals are audit-logged to `~/.ciel/activity.log`.
