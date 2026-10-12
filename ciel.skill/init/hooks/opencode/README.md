# opencode — Ciel runtime adapter (hook payloads)

Unlike every other runtime here, **OpenCode V2 hooks are in-process**, not shell
scripts. This directory therefore ships a **plugin**, not `*.sh` bodies:

```text
init/hooks/opencode/
├── ciel.ts      # the hooks bridge (copy of the canonical plugin source)
└── install.sh   # deploys the plugin + AGENTS.md + commands for opencode
```

`install.sh` (the top-level one) copies this directory into
`~/.ciel/hooks/opencode/`; the runtime deploy step then runs
`~/.ciel/hooks/opencode/install.sh`, which writes:

| Artifact | Path |
| --- | --- |
| Plugin | `~/.config/opencode/plugins/ciel.ts` |
| Identity anchor | `~/.config/opencode/AGENTS.md` |
| Workflow commands | `~/.config/opencode/commands/ciel*.md` |

OpenCode auto-discovers files under `~/.config/opencode/plugins/`, so no
`plugins` entry is added to `opencode.jsonc`.

## Verify

```sh
~/.ciel/hooks/opencode/install.sh
cat ~/.ciel/opencode-plugin.state.json          # written when the server loads the plugin
opencode mcp list                                # unrelated, but a quick liveness probe
~/.ciel/bin/ciel session-start --runtime opencode | python3 -m json.tool
```

See `../../adapters/opencode/HOOKS.md` for the full hook mapping and payload
shapes.
