# CRITICAL_RISK

`locked: true`.

Critical-risk operations always require user approval. No config value downgrades them.

## Examples

- Destructive filesystem operations outside sandbox.
- Force-push / history rewrite on a shared branch.
- Package publication to public registries.
- External state-mutating API calls (email send, payment, SMS).
- Production database migration / destructive query.
- Infrastructure-as-code apply affecting prod.
- Modifying OS or system services.
- Activating computer use with destructive intent.

## Flow

1. Classify operation — critical.
2. Even if Council were convened, record its analysis but do not pre-authorize the action.
3. Construct user escalation envelope:
    - operation + rendered command,
    - evidence trail of how we arrived here,
    - risk composite score + axis breakdown,
    - dry-run preview if available,
    - reversibility analysis,
    - proposed alternative (always one lower-risk alternative proposed if feasible).
4. Block execution until user explicitly approves.
5. User approval is recorded with timestamp, operator id, and (if remote) device binding.

## Post-Execution

- Mandatory watch for at least `critical.watch_hours` (default 24).
- Mandatory post-mortem written to `~/.ciel/high_risk/<run_id>.md` whether success or failure.
- Outcome feeds self-improvement; Ciel prefers to acquire a scripted-and-safer equivalent for future use.

## Forbidden

- Auto-approving on prior pattern. "You did this last week" is not consent for today.
- Delegating critical approval to another agent.
- Running critical ops headless without explicit approval flag.

## Config

```yaml
critical:
  watch_hours: 24
  require_post_mortem: true
  require_alternative_proposal: true
  accept_remote_approval: false   # restricted to in-terminal
```
