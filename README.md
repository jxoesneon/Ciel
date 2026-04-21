# Ciel

![Ciel Banner](ciel.skill/assets/images/banner.jpg)

Ciel is a **self-improving, self-researching skill orchestration intelligence** for AI agent runtimes. She doesn't just route requests — she routes, reasons, acquires, harmonizes, and evolves capabilities on behalf of her host.

## What's in this repo

```text
Ciel/
├── ciel.skill/                # the skill source tree (unpacked)
├── scripts/                   # build + validation helpers
├── .github/workflows/         # CI + release automation
├── README.md
└── LICENSE                    # Apache-2.0
```

The entire specification lives in `ciel.skill/`. Each release bundles that directory into a single `.skill` file (a renamed ZIP per the `.skill` format standard).

## Runtimes Supported

| Runtime | Status | Notes |
| --- | --- | --- |
| Claude Code | ✅ full | Hooks, subagents, MCP (lazy schema), Ultraplan, Computer Use, Prompt Cache, Remote Control |
| Gemini CLI | ✅ full | Hooks, parallel subagents, Plan mode, Checkpointing, A2A, Imagen / Veo / Lyria |
| Generic | ✅ probe-and-adapt | Floor capability check + research-driven adapter bootstrap |

## Install

### From a released `.skill` file

Download `Ciel-vX.Y.Z.skill` from the [Releases](../../releases) page, then in your agent runtime:

```bash
# Claude Code
claude skill install ./Ciel-vX.Y.Z.skill

# Gemini CLI
gemini skill install ./Ciel-vX.Y.Z.skill

# Generic — unpack and load ciel.skill/SKILL.md
unzip Ciel-vX.Y.Z.skill -d ciel.skill/
```

On first run, Ciel bootstraps her global home (`~/.ciel/`), installs MemPalace-rs, git-inits her own history, and calibrates to your current project.

### From source

```bash
git clone <this repo>
cd Ciel
./scripts/build-skill.sh 0.0.0-dev
ls dist/
```

## Architecture

- **Master Router** — hybrid: deterministic fast path → LLM reasoning path → tiered acquisition.
- **Council of Five** — Coherence, Capability, Safety, Efficiency, Evolution. Governs acquisitions, self-modifications, promotions, conflicts, and high-risk ops.
- **Two-domain model** — `~/.ciel/` (global, git-versioned) + `<project>/.ciel/` (local, gitignored, isolated MemPalace partition).
- **Memory** — MemPalace-rs primary; SQLite / filesystem / custom fallbacks; AAAK compression.
- **Risk model** — low autonomous / mid judge / high Council / critical user-gated. Critical never auto-approves.
- **Self-improvement** — every meaningful interaction is a growth-signal candidate; git-backed rollback on regression.

See `ciel.skill/SKILL.md` for the canonical entry point, `ciel.skill/MANIFEST.md` for the subfile index, and each directory's index file for deeper spec.

## Releases

Every pushed tag of the form `vMAJOR.MINOR.PATCH` triggers the **release** workflow, which:

1. Validates spec coverage (`scripts/validate-spec.sh`).
2. Builds `dist/Ciel-vX.Y.Z.skill` + `dist/Ciel-vX.Y.Z.skill.sha256`.
3. Creates a GitHub Release with auto-generated notes and the artifact attached.

Release commands:

```bash
# Cut a release
git tag v1.2.3
git push origin v1.2.3
```

The tag's version must match the `version:` in `ciel.skill/SKILL.md` frontmatter; the release job fails otherwise.

## Contributing

Ciel is Council-governed, even in this repository: changes to locked core files (`ciel.skill/core/CONSTITUTION.md`, `ciel.skill/risk/CLASSIFICATION.md`, `ciel.skill/council/rubrics/VETO_CONDITIONS.md`, `ciel.skill/memory/MEMPALACE.md`, `ciel.skill/domains/ISOLATION.md`, `ciel.skill/council/members/SAFETY.md`) require explicit maintainer sign-off in PR review, mirroring the Constitutional amendment procedure.

Standard contributions:

1. Branch.
2. Make changes.
3. Run `./scripts/validate-spec.sh` locally.
4. Open a PR — CI runs spec coverage + shellcheck + YAML validation + SKILL.md frontmatter schema.

## License

Apache-2.0. See `LICENSE`.
