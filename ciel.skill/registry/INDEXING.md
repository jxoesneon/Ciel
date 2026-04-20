# INDEXING

How skills are indexed for fast-path retrieval.

## Structures

1. **Trigger trie** — normalized trigger tokens → skill-id set. O(k) lookup.
2. **Tag inverted index** — tag → skill-id set. O(tag_count) intersection.
3. **Description embedding** — MemPalace partition `ciel/registry/embeddings/` for semantic recall.
4. **Contract fingerprint** — hash of io_contract shape → skill-id set. Used to find candidates by input shape.

## Build

On registry mutation, `seed_skills/skill_installer/SKILL.md`:

1. Updates `index.json` atomically (write tmp, rename).
2. Re-computes trie / tag sets.
3. Updates embedding in MemPalace if description changed.
4. Persists checksum.

## Fast-Path Query

See `router/FAST_PATH.md`. Order: trigger trie → tag set intersection → contract fingerprint filter → confidence scoring.

## Maintenance

- Periodic sweep (`COHERENCE_SWEEP.md`) rebuilds indices from source-of-truth files.
- If sweep detects drift, a self-improvement proposal is created with the rebuild diff.

## Tag Normalization

- Lowercase.
- Kebab-case.
- Namespaced with `:` where appropriate.
- Tags not in the taxonomy are allowed but flagged to Coherence member at next Council opportunity.

## Performance Targets

- Trigger trie lookup: < 5ms at 10k skills.
- Tag intersection: < 10ms at 10k skills, 5-tag query.
- Semantic search (MemPalace): < 100ms top-10.

Regressions trigger improvement.
