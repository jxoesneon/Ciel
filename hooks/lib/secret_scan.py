#!/usr/bin/env python3
"""Deterministic secret-in-text scanner — local only, no persistence.

Rescoped per council-20260923-conversation-audit (M8): prompt-injection
detection had zero evidence in the audit corpus; the real uncovered ingress
is the *user pasting credentials into chat*, which transcript-side hygiene
cannot prevent. This scanner runs deterministic regexes only — no model
call, no network — and reports hit *categories*, never matched content.

Stdin: raw text. Stdout: ``{"hits": N, "categories": [...]}``.
"""

import json
import re
import sys

PATTERNS = {
    "github_token": r"\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{20,}\b|\bgithub_pat_[A-Za-z0-9_]{20,}\b",
    "aws_access_key": r"\bAKIA[0-9A-Z]{16}\b",
    "api_key_prefixed": r"\b(?:sk|pk|key|api|tok)_[A-Za-z0-9_-]{20,}\b|\bsk-[A-Za-z0-9_-]{20,}\b",
    "private_key_block": r"-----BEGIN [A-Z ]*PRIVATE KEY",
    "slack_token": r"\bxox[baprs]-[0-9A-Za-z-]{10,}\b",
    "gcp_api_key": r"\bAIza[0-9A-Za-z_-]{35}\b",
    "jwt": r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b",
    "npm_token": r"\bnpm_[A-Za-z0-9]{36}\b",
    "crates_token": r"\bcio[0-9A-Za-z]{25,}\b",
    "password_assignment": r"(?i)\b(?:sudo\s+)?(?:password|passwd|passphrase)\s*(?:is|:|=)\s*['\"]?[^\s'\"]{4,}",
    "secret_assignment": r"(?i)\b(?:api[_-]?key|access[_-]?token|auth[_-]?token|secret[_-]?key|client[_-]?secret)\s*[:=]\s*['\"]?[A-Za-z0-9_\-]{8,}",
    "generic_secret_kv": r"(?i)\b(?:token|secret)\s+is\s+['\"]?[A-Za-z0-9_\-]{8,}",
}

_COMPILED = {name: re.compile(pat) for name, pat in PATTERNS.items()}


def scan(text: str) -> dict:
    categories = sorted(
        name for name, rx in _COMPILED.items() if rx.search(text)
    )
    return {"hits": len(categories), "categories": categories}


def main() -> int:
    text = sys.stdin.read()
    print(json.dumps(scan(text)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
