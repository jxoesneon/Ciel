# Ciel Changelog

## 2026-08-16 — Godot Skill Hook Integration (Self-Improvement, Tier 1)

**Trigger:** User noticed the `godot` skill in `~/.ciel/skills/godot/` lacked hooks/prehooks despite being a Ciel-registry skill. Requested full integration.

**Decision:** Create skill-local hooks for the godot skill and register them in `~/.claude/settings.json` alongside existing Ciel hooks.

**Changes:**
- Created `~/.ciel/skills/godot/hooks/godot_preflight.sh` (PreToolUse): Injects AAA+ Godot 4.x guidance when editing `.gd`, `.tscn`, `.tres`, `.gdshader`, or `project.godot` files. Five guidance profiles: gdscript, scene, resource, shader, project.
- Created `~/.ciel/skills/godot/hooks/godot_postflight.sh` (PostToolUse): Validates Godot-specific anti-patterns after edits. Checks for: deep relative `get_node()` paths (DI violation), untyped var declarations, lambdas in `_process` (memory allocation), excessive `queue_free()` (pooling), `@tool` placement errors, `class_name`/autoload conflicts, `buffer_get_data` in shaders (CPU readback), missing Jolt in project.godot, missing TAA.
- Registered both hooks in `~/.claude/settings.json` under `config.hooks.PreToolUse` and `config.hooks.PostToolUse` with matchers targeting `write|edit|notebook_edit` and `write|edit` respectively.
- All hook activations logged to `~/.ciel/activity.log` with `skill:"godot"` tag.

**Verified:**
- `.gd` file edit → GDScript 2.0 guidance injected (typing, DI, component pattern, Jolt, pooling, FSM)
- `.tscn` file edit → Scene guidance injected (MultiMesh, occlusion culling, GI, LOD)
- `.gdshader` file edit → Shader guidance injected (ORM, TAA, fog, GPU storage buffers)
- `project.godot` edit → Project guidance injected (Jolt, TAA, Forward+, physics interpolation)
- Non-Godot file edit → silent passthrough (no injection)
- Anti-pattern detection: deep get_node, untyped vars, lambda in _process, excessive queue_free, missing Jolt/TAA — all detected and reported
- JSON settings file validated as well-formed

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
