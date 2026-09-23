#!/usr/bin/env python3
"""Embedding shortlist helper — run under the laya venv interpreter, not the
system python: ``~/.ciel/system1/venv/bin/python system1_embed.py``.

Reads ``{"task": str, "candidates": {id: criterion}, "k": int}`` on stdin,
encodes task + candidates with a local bi-encoder, and prints the top-k
candidate names by cosine similarity as ``{"names": [...]}``. All errors
exit non-zero with nothing on stdout so the caller can fall back to the
pure-lexical scorer."""

import json
import sys


def main() -> int:
    try:
        payload = json.loads(sys.stdin.read() or "{}")
        task = payload.get("task") or ""
        candidates = payload.get("candidates") or {}
        k = int(payload.get("k") or 10)
        if not task or not candidates:
            return 1
        from sentence_transformers import SentenceTransformer
        model = SentenceTransformer(
            "sentence-transformers/all-MiniLM-L6-v2")
        names = list(candidates)
        docs = model.encode([f"{n}: {candidates[n]}" for n in names],
                            normalize_embeddings=True)
        q = model.encode([task], normalize_embeddings=True)[0]
        sims = docs @ q
        order = sims.argsort()[::-1][:k]
        print(json.dumps({"names": [names[i] for i in order]}))
        return 0
    except Exception:
        return 1


if __name__ == "__main__":
    sys.exit(main())
