# REMOTE_CONTROL — Claude Code

Claude Code's Remote Control surface allows session management from a phone / web client, including push notifications for long-running tasks. Ciel opts in for:

- Ultraplan completion notifications.
- Escalation events (she needs user input now).
- Council deadlock notifications.
- Constitutional-amendment approval requests.

## Enrolment

```yaml
claude_code:
  remote_control:
    enabled: true
    notification_channels: [push]
    quiet_hours: "22:00-08:00"   # optional
```

During quiet hours, Ciel queues non-critical notifications and batches a digest when the window opens. Critical-risk escalations override quiet hours.

## Event Types

| Event | Priority |
| --- | --- |
| Escalation (user input needed) | critical |
| Council deadlock | high |
| Ultraplan complete | normal |
| Self-improvement proposal queued | low |
| Activity digest | low |

## Message Shape

```json
{
  "project": "...",
  "event": "escalation",
  "summary": "proposed `npm publish` — needs approval",
  "deeplink": "claude://session/<id>?resume=escalation-<hash>"
}
```

Deeplink resumes the session at the exact pending approval.

## Remote Approval

Users can approve/reject from the notification. Remote approvals are signed and logged with the same rigor as local approvals. Safety member vetoes are **never** overridable remotely — they require an in-terminal session.

## Failure

If Remote Control is unreachable, Ciel falls back to in-terminal blocking prompts as normal.
