#!/usr/bin/env python3
"""Migrate Ciel-specific SKILL.md frontmatter keys to ciel.yaml sidecars.

The Agent Skills spec allows only `name`, `description`, `license`,
`compatibility`, `metadata`, `allowed-tools` in SKILL.md frontmatter.
Everything else (version, format, runtimes, tags, triggers, source,
dependencies, side_effects) moves verbatim into `ciel.yaml` next to the
SKILL.md, and `metadata.ciel-extension: "ciel.yaml"` /
`metadata.ciel-version` are recorded in the frontmatter.

Usage:
    migrate_skill_sidecar.py --apply   # perform the migration
    migrate_skill_sidecar.py --check   # exit 1 if any skill is unmigrated
"""

import argparse
import sys
from pathlib import Path

import yaml

SPEC_KEYS = {"name", "description", "license", "compatibility", "metadata", "allowed-tools"}
SIDECAR = "ciel.yaml"
SIDECAR_HEADER = "# Ciel skill extension — fields beyond the Agent Skills spec. SKILL.md stays spec-pure.\n"


def _split_frontmatter(text: str):
    """Return (frontmatter_dict, body) or (None, text) if no frontmatter."""
    if not text.startswith("---\n"):
        return None, text
    end = text.find("\n---", 4)
    if end == -1:
        return None, text
    fm_text = text[4:end]
    body = text[end + 1:]
    # Body resumes after the closing '---' line
    nl = body.find("\n")
    body = body[nl + 1:] if nl != -1 else ""
    return yaml.safe_load(fm_text), body


def _dump_frontmatter(fm: dict) -> str:
    return yaml.safe_dump(fm, sort_keys=False, allow_unicode=True, width=4096)


def migrate_skill(skill_dir: Path) -> bool:
    """Migrate one skill dir. Returns True if changes were written."""
    skill_md = skill_dir / "SKILL.md"
    sidecar = skill_dir / SIDECAR
    text = skill_md.read_text(encoding="utf-8")
    fm, body = _split_frontmatter(text)
    if fm is None:
        print(f"[migrate] {skill_md}: no frontmatter, skipped", file=sys.stderr)
        return False

    extra = {k: v for k, v in fm.items() if k not in SPEC_KEYS}
    already = not extra and sidecar.is_file()
    if already:
        return False

    if extra:
        sidecar_data = {"schema": 1}
        sidecar_data.update(extra)
        sidecar.write_text(
            SIDECAR_HEADER + yaml.safe_dump(sidecar_data, sort_keys=False, allow_unicode=True, width=4096),
            encoding="utf-8",
        )

    new_fm = {k: v for k, v in fm.items() if k in SPEC_KEYS}
    metadata = dict(new_fm.get("metadata") or {})
    if "ciel-version" not in metadata:
        metadata["ciel-version"] = str(extra.get("version", metadata.get("ciel-version", "1.0.0")))
    metadata["ciel-extension"] = SIDECAR
    new_fm["metadata"] = metadata

    skill_md.write_text("---\n" + _dump_frontmatter(new_fm) + "---\n" + body, encoding="utf-8")
    return True


def check_skill(skill_dir: Path) -> list[str]:
    """Return a list of problems; empty means conformant."""
    problems = []
    skill_md = skill_dir / "SKILL.md"
    fm, _ = _split_frontmatter(skill_md.read_text(encoding="utf-8"))
    if fm is None:
        problems.append("no frontmatter")
        return problems
    extra = [k for k in fm if k not in SPEC_KEYS]
    if extra:
        problems.append(f"non-spec keys in SKILL.md: {', '.join(extra)}")
    metadata = fm.get("metadata") or {}
    if metadata.get("ciel-extension") != SIDECAR:
        problems.append("missing metadata.ciel-extension: ciel.yaml")
    if not (skill_dir / SIDECAR).is_file():
        problems.append("missing ciel.yaml sidecar")
    return problems


def _skill_dirs(root: Path) -> list[Path]:
    return sorted(p.parent for p in root.glob("skills/*/SKILL.md"))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--apply", action="store_true", help="perform the migration")
    group.add_argument("--check", action="store_true", help="verify conformance, exit 1 on failure")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent,
                        help="repo root (default: parent of scripts/)")
    args = parser.parse_args()

    skills = _skill_dirs(args.root)
    if not skills:
        print(f"[migrate] no skills under {args.root}/skills/", file=sys.stderr)
        return 1

    if args.apply:
        changed = sum(1 for d in skills if migrate_skill(d))
        print(f"[migrate] {len(skills)} skills scanned, {changed} migrated")
        return 0

    failures = 0
    for d in skills:
        for problem in check_skill(d):
            print(f"[check] {d.name}: {problem}", file=sys.stderr)
            failures += 1
    if failures:
        print(f"[check] {failures} problem(s) across {len(skills)} skills", file=sys.stderr)
        return 1
    print(f"[check] {len(skills)} skills conformant")
    return 0


if __name__ == "__main__":
    sys.exit(main())
