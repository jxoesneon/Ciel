#!/usr/bin/env python3
"""Track-1 head-only retrain + temperature refit for the Laya typed-decisions checkpoint.

Implements DOCKET_20261005_CIEL_CONTEXT_EXECUTION Track 1: the encoder is frozen,
only the typed head (``type_emb`` + ``head`` transformer layers + ``scorer`` +
``act_head``) is trained, and the per-cardinality temperature map is refit on the
train split afterwards.

The served checkpoint is NEVER written to. ``--out`` must name a different
directory; the script refuses if it resolves to (or contains) the served dir.

Input rows (JSONL)::

    {"state": {...}, "questions": {...}, "label": {"<qid>": "<choice>" |
     {"choice": "<choice>", "score": <int>, "noul": true}},
     "surface": "<optional surface name>"}

``state``/``questions`` are exactly the shapes ``hooks/lib/system1.py`` posts.

Model facts (verified against laya 0.3.26, ``laya.common.DecisionModel``):

  forward(input_ids, attention_mask, marker_pos, marker_mask, qtype)
      h = encoder(input_ids, attention_mask).last_hidden_state
      h = h + type_emb(qtype)[:, None, :]
      h = head.layers[i](h, src_key_padding_mask=~attention_mask.bool()) x head_layers
      m = gather(h, 1, marker_pos.clamp(min=0)[..., None].expand(-1, -1, d))
      logits = scorer(m).squeeze(-1).masked_fill(~marker_mask, -1e4)
      act_logits = act_head(cat([h[:, 0], softmax(logits.detach()) features]))

  The head consumes ``encoder.last_hidden_state`` and the padding mask only, so
  encoder outputs are cached once per unique sequence and every optimizer update
  runs head-only — the encoder forward is the expensive part and runs exactly
  once per training row (docket §Track-1 step 1).

CPU-only, batch <= 8, no gradient checkpointing, num_workers 0.

    python scripts/system1_head_retrain.py \
        --train train.jsonl --holdout holdout.jsonl --out /path/to/ciel-context \
        --epochs 3 --lr 5e-4 --seed 0
    python scripts/system1_head_retrain.py --train t.jsonl --dry-run
"""

from __future__ import annotations

import argparse
import contextlib
import json
import os
import random
import shutil
import sys
import time
from collections import Counter, defaultdict
from typing import Any

import numpy as np
import torch

# ----------------------------------------------------------------------------
# Checkpoint location
# ----------------------------------------------------------------------------

SYSTEM1_DIR = os.path.expanduser("~/.ciel/system1")
ENV_FILE = os.path.join(SYSTEM1_DIR, "env")
HUB_SNAPSHOTS = os.path.expanduser(
    "~/.cache/huggingface/hub/models--convaiinnovations--laya/snapshots"
)


def _read_env_file(path: str = ENV_FILE) -> dict[str, str]:
    """Parse the KEY=VALUE lines of ~/.ciel/system1/env (no shell sourcing)."""
    env: dict[str, str] = {}
    try:
        with open(path) as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#") or "=" not in line:
                    continue
                k, _, v = line.partition("=")
                v = v.strip().strip("'").strip('"')
                env[k.strip()] = v
    except OSError:
        pass
    return env


def resolve_served_dir(model_arg: str | None) -> str:
    """Directory the live laya-serve actually loads for CIEL_SYSTEM1_MODEL.

    Resolution order: explicit --model path; then the revision pinned in
    ~/.ciel/system1/env (LAYA_REVISION + CIEL_SYSTEM1_MODEL) inside the HF
    snapshot cache; then the newest snapshot carrying that subfolder.
    """
    if model_arg:
        d = os.path.realpath(os.path.expanduser(model_arg))
        if not os.path.isdir(d):
            raise SystemExit(f"--model path does not exist: {d}")
        return d

    env = _read_env_file()
    model_name = env.get("CIEL_SYSTEM1_MODEL") or os.environ.get(
        "CIEL_SYSTEM1_MODEL", "typed-decisions"
    )
    revision = env.get("LAYA_REVISION") or os.environ.get("LAYA_REVISION", "")

    candidates: list[str] = []
    if revision:
        candidates.append(os.path.join(HUB_SNAPSHOTS, revision, model_name))
    if os.path.isdir(HUB_SNAPSHOTS):
        for snap in sorted(
            (os.path.join(HUB_SNAPSHOTS, s) for s in os.listdir(HUB_SNAPSHOTS)),
            key=lambda p: os.path.getmtime(p),
            reverse=True,
        ):
            candidates.append(os.path.join(snap, model_name))

    for cand in candidates:
        if os.path.isfile(os.path.join(cand, "rl_agent_config.json")) and os.path.isfile(
            os.path.join(cand, "model.safetensors")
        ):
            return os.path.realpath(cand)
    raise SystemExit(
        f"could not locate the served checkpoint for {model_name!r} under {HUB_SNAPSHOTS}; "
        "pass --model DIR"
    )


def check_out_dir(out: str, served_dir: str) -> str:
    """Refuse any --out that could touch the live checkpoint."""
    out_real = os.path.realpath(os.path.expanduser(out))
    served_real = os.path.realpath(served_dir)
    if out_real == served_real:
        raise SystemExit(
            f"refusing --out {out_real}: it IS the served checkpoint. "
            "Track-1 writes a sibling directory; the live pin never moves here."
        )
    if out_real.startswith(served_real + os.sep):
        raise SystemExit(
            f"refusing --out {out_real}: inside the served checkpoint {served_real}."
        )
    if served_real.startswith(out_real + os.sep):
        raise SystemExit(
            f"refusing --out {out_real}: it is an ancestor of the served checkpoint "
            f"{served_real}."
        )
    # Never write into the HF blob/snapshot store the live checkpoint is served from.
    if os.path.commonpath([out_real, os.path.realpath(HUB_SNAPSHOTS)]) == os.path.realpath(
        HUB_SNAPSHOTS
    ):
        raise SystemExit(
            f"refusing --out {out_real}: inside the HuggingFace snapshot store."
        )
    return out_real


# ----------------------------------------------------------------------------
# Data
# ----------------------------------------------------------------------------


def load_rows(path: str) -> list[dict[str, Any]]:
    rows = []
    with open(path) as f:
        for lineno, line in enumerate(f, 1):
            line = line.strip()
            if not line:
                continue
            try:
                row = json.loads(line)
            except json.JSONDecodeError as e:
                raise SystemExit(f"{path}:{lineno}: invalid JSON: {e}") from e
            if not isinstance(row, dict) or "state" not in row or "questions" not in row:
                raise SystemExit(
                    f"{path}:{lineno}: row must have 'state' and 'questions' keys"
                )
            row["_source"] = f"{os.path.basename(path)}:{lineno}"
            rows.append(row)
    return rows


def _score_target(n: int, k: int, sigma: float) -> np.ndarray:
    """Ordinal soft target for a `score` question: normalised Gaussian over the
    k rubric levels centred on the labelled level. `laya` decodes a score answer
    as the expected index `sum_i i * p_i`, so a peaked-but-smooth target is the
    matching supervision; it is also a valid probability vector for the
    temperature fitter (calibrate._validated_pair requires sum == 1)."""
    idx = np.arange(k, dtype=np.float32)
    w = np.exp(-((idx - n) ** 2) / (2.0 * sigma * sigma))
    return w / w.sum()


def _label_to_target(
    qid: str, qdef: dict[str, Any], raw: Any, score_sigma: float, source: str
) -> tuple[int, np.ndarray]:
    """Map one label entry to (target_index_in_slot_order, target_vector over k slots)."""
    t = qdef["t"]
    crit = qdef.get("crit")
    order = qdef.get("option_order")

    def slot_of(option_idx: int) -> int:
        if order is None:
            return option_idx
        return list(order).index(option_idx)

    if t == "score":
        val = raw
        if isinstance(raw, dict):
            val = raw.get("score", raw.get("choice"))
        try:
            n = int(val)
        except (TypeError, ValueError):
            raise SystemExit(
                f"{source}: question {qid!r}: score label must be an integer level, got {raw!r}"
            ) from None
        k = len(crit)
        if not 0 <= n < k:
            raise SystemExit(
                f"{source}: question {qid!r}: score label {n} outside [0, {k})"
            )
        canon = _score_target(n, k, score_sigma)
        target = np.zeros(k, dtype=np.float32)
        for i in range(k):
            target[slot_of(i)] = canon[i]
        return slot_of(n), target

    if t == "noul":
        val = raw
        if isinstance(raw, dict):
            val = raw.get("noul", raw.get("choice", raw.get("score")))
        if isinstance(val, bool):
            idx = int(val)
        elif isinstance(val, str):
            v = val.strip().lower()
            if v not in ("true", "false"):
                raise SystemExit(
                    f"{source}: question {qid!r}: noul label must be true/false, got {raw!r}"
                )
            idx = int(v == "true")
        else:
            raise SystemExit(
                f"{source}: question {qid!r}: noul label must be true/false, got {raw!r}"
            )
        target = np.zeros(2, dtype=np.float32)
        target[slot_of(idx)] = 1.0
        return slot_of(idx), target

    # choice
    keys = list(crit.keys()) if isinstance(crit, dict) else []
    val = raw.get("choice", raw.get("score")) if isinstance(raw, dict) else raw
    val = str(val)
    if val not in keys:
        raise SystemExit(
            f"{source}: question {qid!r}: choice label {val!r} not in criteria {keys}"
        )
    idx = keys.index(val)
    k = len(keys)
    target = np.zeros(k, dtype=np.float32)
    target[slot_of(idx)] = 1.0
    return slot_of(idx), target


def encode_dataset(
    agent, rows: list[dict[str, Any]], score_sigma: float
) -> tuple[list[dict[str, Any]], list[str]]:
    """Turn labeled rows into per-question items via the agent's own encoder path.

    Each item is one full sequence ([CLS] head [SEP] state [SEP]); a state with
    N questions yields N items. Returns (items, skip-notes).
    """
    items: list[dict[str, Any]] = []
    notes: list[str] = []
    for row in rows:
        state, questions = row["state"], row["questions"]
        labels = row.get("label") or {}
        surface = row.get("surface") or (row.get("meta") or {}).get("surface") or "unspecified"
        ids = [qid for qid in questions if qid in labels]
        skipped = [qid for qid in questions if qid not in labels]
        if skipped:
            notes.append(
                f"{row['_source']}: no label for {skipped}; those questions are skipped"
            )
        if not ids:
            continue
        for qid in ids:
            agent._check_question(qid, questions[qid])
        internal = {qid: agent._to_internal(questions[qid]) for qid in ids}
        enc = agent._encode_state(state, ids, internal)
        for j, qid in enumerate(ids):
            item = dict(enc[j])
            k = len(item["markers"])
            tgt_idx, tgt_vec = _label_to_target(
                qid, internal[qid], labels[qid], score_sigma, row["_source"]
            )
            if len(tgt_vec) != k:
                raise SystemExit(
                    f"{row['_source']}: question {qid!r}: label maps to {len(tgt_vec)} "
                    f"options but the encoded item has {k} markers"
                )
            item["target_idx"] = tgt_idx
            item["target"] = tgt_vec
            item["qid"] = qid
            item["surface"] = surface
            item["source"] = row["_source"]
            items.append(item)
    return items, notes


# ----------------------------------------------------------------------------
# Encoder cache + head-only forward
# ----------------------------------------------------------------------------


@torch.no_grad()
def cache_encoder_outputs(
    model, items: list[dict[str, Any]], batch_size: int
) -> tuple[dict[tuple, torch.Tensor], dict[str, float]]:
    """Run the frozen encoder once per unique sequence; return {ids: h[L,d]} and
    per-surface encode seconds (amortised per item)."""
    model.encoder.eval()
    unique: dict[tuple, int] = {}
    for it in items:
        key = tuple(it["ids"])
        if key not in unique:
            unique[key] = len(unique)
    keys = list(unique.keys())
    cache: dict[tuple, torch.Tensor] = {}
    surf_time: dict[str, float] = defaultdict(float)
    # amortise batch wall time over the items each sequence serves
    seq_surfaces: dict[tuple, list[str]] = defaultdict(list)
    for it in items:
        seq_surfaces[tuple(it["ids"])].append(it["surface"])

    for start in range(0, len(keys), batch_size):
        part = keys[start : start + batch_size]
        L = max(len(k) for k in part)
        ids = torch.zeros(len(part), L, dtype=torch.long)
        att = torch.zeros(len(part), L, dtype=torch.long)
        for i, k in enumerate(part):
            ids[i, : len(k)] = torch.tensor(list(k), dtype=torch.long)
            att[i, : len(k)] = 1
        t0 = time.perf_counter()
        h = model.encoder(input_ids=ids, attention_mask=att).last_hidden_state
        dt = time.perf_counter() - t0
        h = h.float()
        for i, k in enumerate(part):
            cache[k] = h[i, : len(k)].clone()
            for s in seq_surfaces[k]:
                surf_time[s] += dt / len(part)
    return cache, dict(surf_time)


def collate_head_batch(
    batch: list[dict[str, Any]], cache: dict[tuple, torch.Tensor]
) -> dict[str, torch.Tensor]:
    """Pad cached encoder outputs and marker metadata for one update/eval batch."""
    n = len(batch)
    hs = [cache[tuple(it["ids"])] for it in batch]
    L = max(h.shape[0] for h in hs)
    d = hs[0].shape[-1]
    kmax = max(len(it["markers"]) for it in batch)
    h_pad = torch.zeros(n, L, d)
    att = torch.zeros(n, L, dtype=torch.long)
    mpos = torch.zeros(n, kmax, dtype=torch.long)
    mmask = torch.zeros(n, kmax, dtype=torch.bool)
    qtype = torch.zeros(n, dtype=torch.long)
    target = torch.zeros(n, kmax)
    for i, (it, h) in enumerate(zip(batch, hs, strict=True)):
        h_pad[i, : h.shape[0]] = h
        att[i, : h.shape[0]] = 1
        k = len(it["markers"])
        mpos[i, :k] = torch.tensor(it["markers"], dtype=torch.long)
        mmask[i, :k] = True
        qtype[i] = int(it["qtype"])
        target[i, :k] = torch.tensor(it["target"], dtype=torch.float32)
    return {
        "h": h_pad,
        "attention_mask": att,
        "marker_pos": mpos,
        "marker_mask": mmask,
        "qtype": qtype,
        "target": target,
    }


def collate_raw_batch(batch: list[dict[str, Any]]) -> dict[str, torch.Tensor]:
    """Pad raw token ids + marker metadata — the Track-2 path, where the
    encoder runs inside the update so cached hidden states can't be used."""
    n = len(batch)
    L = max(len(it["ids"]) for it in batch)
    kmax = max(len(it["markers"]) for it in batch)
    ids = torch.zeros(n, L, dtype=torch.long)
    att = torch.zeros(n, L, dtype=torch.long)
    mpos = torch.zeros(n, kmax, dtype=torch.long)
    mmask = torch.zeros(n, kmax, dtype=torch.bool)
    qtype = torch.zeros(n, dtype=torch.long)
    target = torch.zeros(n, kmax)
    for i, it in enumerate(batch):
        ids[i, : len(it["ids"])] = torch.tensor(it["ids"], dtype=torch.long)
        att[i, : len(it["ids"])] = 1
        k = len(it["markers"])
        mpos[i, :k] = torch.tensor(it["markers"], dtype=torch.long)
        mmask[i, :k] = True
        qtype[i] = int(it["qtype"])
        target[i, :k] = torch.tensor(it["target"], dtype=torch.float32)
    return {"input_ids": ids, "attention_mask": att, "marker_pos": mpos,
            "marker_mask": mmask, "qtype": qtype, "target": target}


def head_forward(model, h_pad, attention_mask, marker_pos, marker_mask, qtype):  # noqa: PLR0917
    """The head half of DecisionModel.forward, run on cached encoder outputs.

    Mirrors laya/common.py DecisionModel.forward line for line from the
    `type_emb` add onward, so the trained artifact is bit-compatible with the
    monolithic path (verified in --dry-run against model(...) itself).
    """
    h = h_pad + model.type_emb(qtype)[:, None, :]
    pad = ~attention_mask.bool()
    if model.head is not None:
        for layer in model.head.layers:
            h = layer(h, src_key_padding_mask=pad)
    idx = marker_pos.clamp(min=0)[:, :, None].expand(-1, -1, h.size(-1))
    m = torch.gather(h, 1, idx)
    logits = model.scorer(m).squeeze(-1).float()
    logits = logits.masked_fill(~marker_mask, -1e4)

    p = torch.softmax(logits.detach(), -1)
    k = marker_mask.sum(-1).clamp(min=2).float()
    ent = -(p * torch.log(p.clamp_min(1e-9))).sum(-1) / torch.log(k)
    if p.size(-1) >= 2:
        top2 = p.topk(2, -1).values
    else:
        top1 = p.topk(1, -1).values
        top2 = torch.cat([top1, torch.zeros_like(top1)], dim=-1)
    feats = torch.stack([top2[:, 0], top2[:, 0] - top2[:, 1], ent, k / 255.0], -1)
    pooled = h[:, 0].float()
    act_input = torch.cat([pooled, feats], -1)
    if not torch.is_autocast_enabled(h.device.type):
        act_input = act_input.to(model.act_head[0].weight.dtype)
    act_logits = model.act_head(act_input)
    return logits, act_logits


def soft_ce(logits: torch.Tensor, target: torch.Tensor, marker_mask: torch.Tensor):
    """Per-row cross-entropy with (possibly soft) targets over valid options."""
    lp = torch.log_softmax(logits, -1)
    per_row = -(target * lp).sum(-1)
    return per_row


# ----------------------------------------------------------------------------
# Track-2: LoRA adapters on the encoder's fused attention projections
# ----------------------------------------------------------------------------
# ModernBERT layers: encoder.layers[i].attn.Wqkv (1024->3072) and .Wo (1024->
# 1024). Adapters are zero-initialised so the wrapped model starts identical
# to the served checkpoint. When --lora is on, encoder outputs can no longer
# be cached — every update is a full forward+backward through the encoder.


class LoRALinear(torch.nn.Module):
    """Frozen base Linear + trainable rank-decomposed delta (B@A * alpha/r)."""

    def __init__(self, base: torch.nn.Linear, rank: int, alpha: float):
        super().__init__()
        if not isinstance(base, torch.nn.Linear):
            raise TypeError(f"LoRA can only wrap nn.Linear, got {type(base)}")
        self.base = base
        self.A = torch.nn.Linear(base.in_features, rank, bias=False)
        self.B = torch.nn.Linear(rank, base.out_features, bias=False)
        self.scale = alpha / rank
        torch.nn.init.kaiming_uniform_(self.A.weight, a=5 ** 0.5)
        torch.nn.init.zeros_(self.B.weight)  # identity at init
        for p in self.base.parameters():
            p.requires_grad_(False)

    def forward(self, x):
        return self.base(x) + self.B(self.A(x)) * self.scale


def inject_lora(model, rank: int, alpha: float, last_n_layers: int) -> int:
    """Wrap Wqkv/Wo of the LAST `last_n_layers` encoder layers in LoRALinear.
    Returns the number of adapters installed. The base Linear's weights are
    frozen; only A/B train (they're the only encoder-side requires_grad
    params after this)."""
    layers = model.encoder.layers
    lo, hi = max(0, len(layers) - last_n_layers), len(layers)
    n = 0
    for i in range(lo, hi):
        attn = layers[i].attn
        attn.Wqkv = LoRALinear(attn.Wqkv, rank, alpha)
        attn.Wo = LoRALinear(attn.Wo, rank, alpha)
        n += 2
    return n


def full_forward(model, ids, attention_mask, marker_pos, marker_mask, qtype):  # noqa: PLR0917
    """Model() forward keeping autograd — same signature as head_forward but
    runs the encoder too (for LoRA updates). Returns (logits, act_logits)."""
    out = model(input_ids=ids, attention_mask=attention_mask,
                marker_pos=marker_pos, marker_mask=marker_mask, qtype=qtype)
    if isinstance(out, tuple):
        return out[0], out[1]
    return out.logits, getattr(out, "act_logits", None)


# ----------------------------------------------------------------------------
# Eval
# ----------------------------------------------------------------------------

QTYPE_NAMES = {0: "choice", 1: "score", 2: "noul"}


@torch.no_grad()
def evaluate(
    model,
    items: list[dict[str, Any]],
    cache: dict[tuple, torch.Tensor],
    batch_size: int,
    lora: bool = False,
) -> dict[str, Any]:
    """Argmax accuracy overall / per surface / per qtype, plus a confusion tally
    and mean |E[p] - label| for score rows (how `laya` decodes scores)."""
    model.head.eval()
    model.scorer.eval()
    model.act_head.eval()
    model.type_emb.eval()
    correct = 0
    per_surface: dict[str, list[int]] = defaultdict(lambda: [0, 0])
    per_qtype: dict[str, list[int]] = defaultdict(lambda: [0, 0])
    confusion: Counter = Counter()
    score_abs_err: list[float] = []
    for start in range(0, len(items), batch_size):
        part = items[start : start + batch_size]
        with torch.no_grad():
            if lora:
                b = collate_raw_batch(part)
                logits, _act = full_forward(
                    model, b["input_ids"], b["attention_mask"], b["marker_pos"],
                    b["marker_mask"], b["qtype"])
            else:
                b = collate_head_batch(part, cache)
                logits, _act = head_forward(
                    model, b["h"], b["attention_mask"], b["marker_pos"], b["marker_mask"], b["qtype"]
                )
        for i, it in enumerate(part):
            k = len(it["markers"])
            z = logits[i, :k]
            pred = int(z.argmax().item())
            ok = int(pred == it["target_idx"])
            correct += ok
            per_surface[it["surface"]][0] += ok
            per_surface[it["surface"]][1] += 1
            qn = QTYPE_NAMES[int(it["qtype"])]
            per_qtype[qn][0] += ok
            per_qtype[qn][1] += 1
            if not ok:
                confusion[f"{it['surface']}/{it['qid']}: {it['target_idx']}->{pred}"] += 1
            if qn == "score":
                p = torch.softmax(z, -1)
                exp = float((torch.arange(k) * p).sum().item())
                score_abs_err.append(abs(exp - float(it["target_idx"])))
    return {
        "n": len(items),
        "acc": correct / max(1, len(items)),
        "per_surface": {
            s: {"n": c[1], "acc": c[0] / c[1]} for s, c in sorted(per_surface.items())
        },
        "per_qtype": {
            q: {"n": c[1], "acc": c[0] / c[1]} for q, c in sorted(per_qtype.items())
        },
        "score_mae": float(np.mean(score_abs_err)) if score_abs_err else None,
        "confusion": dict(confusion.most_common(20)),
    }


def set_head_training(model, training: bool) -> None:
    for mod in (model.head, model.scorer, model.act_head, model.type_emb):
        if mod is not None:
            mod.train(training)


def collect_records(model, items, cache, batch_size, lora: bool = False) -> list[tuple[int, np.ndarray, np.ndarray, int]]:
    """(qtype, logits[:k], target[:k], k) over the train split — the record shape
    `agent.fit_temperatures` consumes."""
    set_head_training(model, False)
    records = []
    for start in range(0, len(items), batch_size):
        part = items[start : start + batch_size]
        with torch.no_grad():
            if lora:
                b = collate_raw_batch(part)
                logits, _act = full_forward(
                    model, b["input_ids"], b["attention_mask"], b["marker_pos"],
                    b["marker_mask"], b["qtype"])
            else:
                b = collate_head_batch(part, cache)
                logits, _act = head_forward(
                    model, b["h"], b["attention_mask"], b["marker_pos"], b["marker_mask"], b["qtype"]
                )
        for i, it in enumerate(part):
            k = len(it["markers"])
            z = logits[i, :k].float().cpu().numpy()
            t = np.asarray(it["target"], dtype=np.float32)[:k]
            records.append((int(it["qtype"]), z, t, k))
    return records


# ----------------------------------------------------------------------------
# Checkpoint write
# ----------------------------------------------------------------------------


def merged_state_dict(model) -> dict[str, torch.Tensor]:
    """State dict with LoRA adapters folded back into their base Linears
    (W' = W + (B @ A) * scale) — the emitted checkpoint keeps the stock
    DecisionModel key layout so it loads without the adapter classes."""
    merged = {}
    sd = model.state_dict()
    lora_bases = set()
    for name in sd:
        if ".A.weight" in name:
            lora_bases.add(name.rsplit(".A.weight", 1)[0])
    skip = {b + suffix for b in lora_bases
            for suffix in (".A.weight", ".B.weight", ".base.weight", ".base.bias")}
    for name, t in sd.items():
        if name in skip:
            continue
        merged[name] = t
    for base_name in lora_bases:
        W = sd[base_name + ".base.weight"]
        bias = sd.get(base_name + ".base.bias")
        A = sd[base_name + ".A.weight"]
        B = sd[base_name + ".B.weight"]
        mod = model
        for attr in base_name.split("."):
            mod = mod[int(attr)] if attr.isdigit() else getattr(mod, attr)
        merged[base_name + ".weight"] = W + (B @ A) * mod.scale
        if bias is not None:
            merged[base_name + ".bias"] = bias
    return merged


def write_sibling_checkpoint(
    out_dir: str, served_dir: str, agent, temperatures: dict[str, Any], meta: dict[str, Any]
) -> None:
    """Emit <out>/ with the served layout: model.safetensors (full state dict —
    encoder values untouched, head retrained), encoder/, tokenizer/, and an
    rl_agent_config.json carrying the refit temperatures.

    Snapshot entries are symlinks into the HF blob store; copies dereference
    them so the blob store is never opened for writing. Weights are saved fp32
    (the runtime's own precision after load_state_dict upcasts the shipped fp16
    file); `Agent` loads fp32 and fp16 identically.
    """
    from safetensors.torch import save_file

    os.makedirs(out_dir, exist_ok=os.path.isdir(out_dir))
    for sub in ("encoder", "tokenizer"):
        src = os.path.join(served_dir, sub)
        dst = os.path.join(out_dir, sub)
        if os.path.isdir(src):
            os.makedirs(dst, exist_ok=True)
            for name in os.listdir(src):
                shutil.copyfile(os.path.join(src, name), os.path.join(dst, name))
    # Any other top-level side files except the two we rewrite.
    for name in os.listdir(served_dir):
        src = os.path.join(served_dir, name)
        if os.path.isfile(src) and name not in ("model.safetensors", "rl_agent_config.json"):
            shutil.copyfile(src, os.path.join(out_dir, name))

    save_file(merged_state_dict(agent.model), os.path.join(out_dir, "model.safetensors"))

    cfg = dict(agent.cfg)
    cfg["temperature"] = [float(x) for x in temperatures["temperature"]]
    cfg["temperature_by_options"] = {
        str(k): float(v) for k, v in temperatures["temperature_by_options"].items()
    }
    training = dict(cfg.get("training") or {})
    training["track1_head_retrain"] = meta
    cfg["training"] = training
    cfg_path = os.path.join(out_dir, "rl_agent_config.json")
    with open(cfg_path, "w") as f:
        json.dump(cfg, f, indent=2)
        f.write("\n")


# ----------------------------------------------------------------------------
# Main
# ----------------------------------------------------------------------------


def main() -> int:  # noqa: PLR0912, PLR0915 -- the train loop is one linear flow; splitting it would scatter the budget/abort checks
    ap = argparse.ArgumentParser(
        description="Track-1 head-only retrain for the Laya typed-decisions checkpoint."
    )
    ap.add_argument("--train", required=True, help="labeled JSONL train file")
    ap.add_argument("--holdout", default=None, help="labeled JSONL holdout file (optional)")
    ap.add_argument("--out", default=None, help="output checkpoint dir (must differ from served)")
    ap.add_argument("--epochs", type=int, default=3)
    ap.add_argument("--lr", type=float, default=5e-4)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--batch-size", type=int, default=8, help="<= 8 by design")
    ap.add_argument("--weight-decay", type=float, default=0.01)
    ap.add_argument(
        "--score-sigma",
        type=float,
        default=0.5,
        help="Gaussian width for ordinal score targets",
    )
    ap.add_argument("--model", default=None, help="override checkpoint dir (default: served)")
    ap.add_argument("--lora-rank", type=int, default=0,
                    help="Track 2: rank of LoRA adapters on encoder attn "
                         "Wqkv/Wo of the last --lora-layers layers (0=off)")
    ap.add_argument("--lora-alpha", type=float, default=16.0)
    ap.add_argument("--lora-layers", type=int, default=8,
                    help="number of trailing encoder layers to adapt")
    ap.add_argument("--max-updates", type=int, default=0,
                    help="bar-10 abort: cap total optimizer updates (0=epoch count)")
    ap.add_argument("--budget-hours", type=float, default=4.0,
                    help="bar-10 abort: after 100 updates, projected runtime "
                         "beyond this aborts with exit 3")
    ap.add_argument(
        "--cache-file", default=None,
        help="persist the encoder-output cache here (.pt); reload on re-runs "
             "so the once-per-unique-sequence encoder pass is paid once",
    )
    ap.add_argument(
        "--dry-run",
        action="store_true",
        help="load everything, run 10 updates, report update-rate; writes nothing",
    )
    args = ap.parse_args()

    if args.batch_size > 8:
        raise SystemExit("--batch-size must be <= 8 (CPU track-1 constraint)")
    if args.epochs < 1 and not args.dry_run:
        raise SystemExit("--epochs must be >= 1")

    env = _read_env_file()
    threads = env.get("LAYA_THREADS")
    if threads:
        with contextlib.suppress(ValueError):
            torch.set_num_threads(int(threads))

    random.seed(args.seed)
    np.random.seed(args.seed)
    torch.manual_seed(args.seed)

    served_dir = resolve_served_dir(args.model)
    out_dir = check_out_dir(args.out, served_dir) if args.out else None
    if not args.dry_run and out_dir is None:
        raise SystemExit("--out is required unless --dry-run")
    print(f"[load] served checkpoint: {served_dir}")
    if out_dir:
        print(f"[load] output checkpoint: {out_dir}")

    # Local-dir load. expected_sha256={} deliberately suppresses the
    # LAYA_SHA256_DIGESTS env fallback (revisions.verify_digests) — a nested map
    # names artifacts under the hub id, not this directory, and would fail the load.
    from laya.agent import Agent

    t0 = time.perf_counter()
    agent = Agent(served_dir, device="cpu", expected_sha256={})
    print(f"[load] agent ready in {time.perf_counter() - t0:.1f}s "
          f"(cfg head_layers={agent.cfg.get('head_layers')}, "
          f"max_len={agent.cfg.get('max_len')}, head_max_len={agent.cfg.get('head_max_len')})")

    model = agent.model
    lora_mode = args.lora_rank > 0
    if lora_mode:
        n_adapters = inject_lora(model, args.lora_rank, args.lora_alpha,
                                 args.lora_layers)
        print(f"[lora] injected {n_adapters} rank-{args.lora_rank} adapters "
              f"(alpha={args.lora_alpha}) on last {args.lora_layers} encoder layers")
    # Freeze EVERY encoder parameter; train everything else (type_emb, head,
    # scorer, act_head — the typed head per named_children). In LoRA mode the
    # adapter A/B matrices are the only encoder-side trainables.
    n_frozen = n_train = 0
    head_param_names = []
    for name, p in model.named_parameters():
        if name.startswith("encoder."):
            p.requires_grad_(name.endswith((".A.weight", ".B.weight")))
            n_frozen += p.numel() if not p.requires_grad else 0
            n_train += p.numel() if p.requires_grad else 0
        else:
            p.requires_grad_(True)
            n_train += p.numel()
            head_param_names.append(name)
    model.encoder.eval()
    print(f"[load] frozen encoder params: {n_frozen:,}; trainable head params: {n_train:,}")
    head_modules = [n for n, _ in model.named_children() if n != "encoder"]
    print(f"[load] head submodules: {head_modules}")

    train_rows = load_rows(args.train)
    holdout_rows = load_rows(args.holdout) if args.holdout else []
    print(f"[data] train rows: {len(train_rows)}; holdout rows: {len(holdout_rows)}")

    t0 = time.perf_counter()
    train_items, notes = encode_dataset(agent, train_rows, args.score_sigma)
    holdout_items, hnotes = encode_dataset(agent, holdout_rows, args.score_sigma)
    for n_ in notes + hnotes:
        print(f"[data] note: {n_}")
    print(
        f"[data] encoded in {time.perf_counter() - t0:.1f}s -> "
        f"{len(train_items)} train items, {len(holdout_items)} holdout items"
    )
    if not train_items:
        raise SystemExit("no labeled training items after encoding; nothing to do")

    t0 = time.perf_counter()
    all_items = train_items + holdout_items
    cache = {}
    surf_enc_time = {}
    if lora_mode:
        # LoRA deltas change encoder outputs — no reuse possible.
        print("[cache] lora mode: encoder cache disabled (full forward per update)")
    else:
        if args.cache_file and os.path.exists(args.cache_file):
            cache = torch.load(args.cache_file, weights_only=False)
            print(f"[cache] loaded {len(cache)} cached sequences from {args.cache_file}")
        if not cache:
            cache, surf_enc_time = cache_encoder_outputs(
                model, all_items, args.batch_size)
            if args.cache_file:
                torch.save(cache, args.cache_file)
                print(f"[cache] saved {len(cache)} sequences to {args.cache_file}")
        else:
            # any rows whose sequence isn't cached yet get encoded now
            missing = [it for it in all_items if tuple(it["ids"]) not in cache]
            if missing:
                extra, _ = cache_encoder_outputs(model, missing, args.batch_size)
                cache.update(extra)
                if args.cache_file:
                    torch.save(cache, args.cache_file)
            surf_enc_time = {}
    enc_s = time.perf_counter() - t0
    cache_mb = sum(h.numel() * 4 for h in cache.values()) / 1e6
    print(
        f"[cache] {len(cache)} unique sequences, {cache_mb:.0f} MB fp32, "
        f"encoder total {enc_s:.1f}s"
    )
    for s, sec in sorted(surf_enc_time.items()):
        n_s = sum(1 for it in all_items if it["surface"] == s)
        print(f"[cache]   surface={s}: {n_s} items, encoder {sec:.2f}s "
              f"({1000 * sec / max(1, n_s):.0f} ms/item amortised)")

    opt = torch.optim.AdamW(
        [p for p in model.parameters() if p.requires_grad],
        lr=args.lr,
        weight_decay=args.weight_decay,
    )

    def run_updates(indices: list[int], n_updates: int) -> list[float]:
        """Run up to n_updates optimiser steps over indices; returns per-update seconds."""
        set_head_training(model, True)
        times = []
        upd = 0
        pos = 0
        order = list(indices)
        while upd < n_updates:
            if pos >= len(order):
                random.shuffle(order)
                pos = 0
            part = order[pos : pos + args.batch_size]
            pos += args.batch_size
            batch = [train_items[i] for i in part]
            if lora_mode:
                b = collate_raw_batch(batch)
                t0 = time.perf_counter()
                logits, _act = full_forward(
                    model, b["input_ids"], b["attention_mask"], b["marker_pos"],
                    b["marker_mask"], b["qtype"])
            else:
                b = collate_head_batch(batch, cache)
                t0 = time.perf_counter()
                logits, _act = head_forward(
                    model, b["h"], b["attention_mask"], b["marker_pos"],
                    b["marker_mask"], b["qtype"])
            per_row = soft_ce(logits, b["target"], b["marker_mask"])
            loss = per_row.mean()
            opt.zero_grad()
            loss.backward()
            torch.nn.utils.clip_grad_norm_(
                [p for p in model.parameters() if p.requires_grad], 1.0
            )
            opt.step()
            times.append(time.perf_counter() - t0)
            upd += 1
            # bar 10 — update-rate checkpoint after the first 100 updates:
            # extrapolate and abort if the projected total exceeds the cap.
            if (  # pragma: no cover — probe runs ≤10 updates; the live
                lora_mode and upd == 100 and args.max_updates > 100   # bar-10 check is the main loop's below
            ):
                rate = sum(times) / len(times)
                projected_h = rate * args.max_updates / 3600
                print(f"[bar10] update-rate after 100: {rate:.2f}s/update; "
                      f"projected {projected_h:.1f}h vs {args.budget_hours}h cap")
                if projected_h > args.budget_hours:
                    print("[bar10] ABORT — projected runtime exceeds cap; "
                          "escalating rather than running to exhaustion")
                    raise SystemExit(3)
        return times

    # -- Parity check: head_forward on cached h must equal the monolithic
    # model(). Skipped in LoRA mode — there is no split path; adapters are
    # zero-initialised so the wrapped model IS the served model at init.
    d_logits = d_act = 0.0
    if not lora_mode:
        parity_items = all_items[: args.batch_size]
        b = collate_head_batch(parity_items, cache)
        pad_id = agent.tok.pad_token_id
        from laya.common import collate_items

        mono = collate_items([[it] for it in parity_items], pad_id)
        with torch.no_grad():
            model.eval()
            logits_mono, act_mono = model(
                mono["input_ids"], mono["attention_mask"], mono["marker_pos"],
                mono["marker_mask"], mono["qtype"],
            )
            logits_head, act_head_out = head_forward(
                model, b["h"], b["attention_mask"], b["marker_pos"], b["marker_mask"], b["qtype"]
            )
        d_logits = float((logits_mono - logits_head).abs().max())
        d_act = float((act_mono - act_head_out).abs().max())
        print(f"[parity] head_forward vs model(): max|dlogits|={d_logits:.2e} "
              f"max|dact|={d_act:.2e}")
        if d_logits > 1e-3 or d_act > 1e-3:
            raise SystemExit(
                "head-forward parity check FAILED — the split does not reproduce "
                "DecisionModel.forward; refusing to train on a mismatched head path"
            )
    model.encoder.eval()  # model.eval() above reset it anyway; keep it explicit
    set_head_training(model, True)

    train_idx = list(range(len(train_items)))

    if args.dry_run:
        n_upd = min(10, max(1, -(-len(train_idx) // args.batch_size) * 2))
        n_upd = min(10, max(n_upd, 1))
        times = run_updates(train_idx, 10 if len(train_idx) >= 1 else 1)
        rate = len(times) / sum(times)
        print(f"[dry-run] {len(times)} updates of batch<= {args.batch_size}: "
              f"{rate:.2f} updates/s ({1000 * sum(times) / len(times):.0f} ms/update avg)")
        report = {
            "mode": "dry-run",
            "updates": len(times),
            "updates_per_sec": round(rate, 3),
            "ms_per_update": round(1000 * sum(times) / len(times), 1),
            "encoder_cache": {
                "unique_sequences": len(cache),
                "mb_fp32": round(cache_mb, 1),
                "seconds": round(enc_s, 2),
                "per_surface_s": {k: round(v, 3) for k, v in surf_enc_time.items()},
            },
            "parity": {"max_abs_dlogits": d_logits, "max_abs_dact": d_act},
            "train_items": len(train_items),
            "holdout_items": len(holdout_items),
        }
        print("[dry-run] report:")
        print(json.dumps(report, indent=2))
        print("[dry-run] nothing written; exiting 0")
        return 0

    # ----------------------------- training ---------------------------------
    history = []
    total_upd = 0
    upd_times: list[float] = []
    upd_budget = args.max_updates or 10**9
    abort_done = False
    stop_training = False
    for epoch in range(1, args.epochs + 1):
        if stop_training:
            break
        random.shuffle(train_idx)
        set_head_training(model, True)
        ep_loss = 0.0
        n_upd = 0
        t0 = time.perf_counter()
        for pos in range(0, len(train_idx), args.batch_size):
            if total_upd >= upd_budget:
                print(f"[cap] --max-updates {args.max_updates} reached; stopping")
                stop_training = True
                break
            batch = [train_items[i] for i in train_idx[pos : pos + args.batch_size]]
            t_upd = time.perf_counter()
            if lora_mode:
                b = collate_raw_batch(batch)
                logits, _act = full_forward(
                    model, b["input_ids"], b["attention_mask"], b["marker_pos"],
                    b["marker_mask"], b["qtype"])
            else:
                b = collate_head_batch(batch, cache)
                logits, _act = head_forward(
                    model, b["h"], b["attention_mask"], b["marker_pos"], b["marker_mask"], b["qtype"]
                )
            loss = soft_ce(logits, b["target"], b["marker_mask"]).mean()
            opt.zero_grad()
            loss.backward()
            torch.nn.utils.clip_grad_norm_(
                [p for p in model.parameters() if p.requires_grad], 1.0
            )
            opt.step()
            ep_loss += float(loss.item())
            n_upd += 1
            total_upd += 1
            upd_times.append(time.perf_counter() - t_upd)
            # bar 10 — update-rate checkpoint after the first 100 updates:
            # extrapolate and abort if the projected total exceeds the cap.
            if (
                lora_mode and not abort_done and total_upd >= 100
                and args.max_updates > 100
            ):
                abort_done = True
                rate = sum(upd_times) / len(upd_times)
                projected_h = rate * upd_budget / 3600
                print(f"[bar10] update-rate after {total_upd}: {rate:.2f}s/update; "
                      f"projected {projected_h:.1f}h vs {args.budget_hours}h cap")
                if projected_h > args.budget_hours:
                    print("[bar10] ABORT — projected runtime exceeds cap; "
                          "escalating rather than running to exhaustion")
                    raise SystemExit(3)
        train_eval = evaluate(model, train_items, cache, args.batch_size, lora=lora_mode)
        hold_eval = (
            evaluate(model, holdout_items, cache, args.batch_size, lora=lora_mode) if holdout_items else None
        )
        history.append(
            {
                "epoch": epoch,
                "loss": ep_loss / max(1, n_upd),
                "updates": n_upd,
                "train_acc": train_eval["acc"],
                "holdout_acc": hold_eval["acc"] if hold_eval else None,
                "seconds": time.perf_counter() - t0,
            }
        )
        print(
            f"[epoch {epoch}] loss={ep_loss / max(1, n_upd):.4f} "
            f"train_acc={train_eval['acc']:.3f} "
            f"holdout_acc={hold_eval['acc'] if hold_eval else float('nan'):.3f} "
            f"({time.perf_counter() - t0:.1f}s)"
        )

    final_train = evaluate(model, train_items, cache, args.batch_size, lora=lora_mode)
    final_hold = evaluate(model, holdout_items, cache, args.batch_size, lora=lora_mode) if holdout_items else None

    print("[eval] confusion summary (train):")
    for k, v in final_train["confusion"].items():
        print(f"[eval]   {k}: {v}")

    # --------------------- temperature refit (train split) ------------------
    records = collect_records(model, train_items, cache, args.batch_size, lora=lora_mode)
    fit = agent.fit_temperatures(records, compute_ece=False, seed=args.seed)
    print(f"[calib] fitted temperatures: {fit['temperature']} "
          f"by_options={fit['temperature_by_options']} n_by_bucket={fit['n_by_bucket']}")

    meta = {
        "updates": int(sum(h["updates"] for h in history)),
        "epochs_completed": int(args.epochs),
        "lr": args.lr,
        "seed": args.seed,
        "batch_size": args.batch_size,
        "score_sigma": args.score_sigma,
        "train_rows": len(train_rows),
        "train_items": len(train_items),
        "holdout_items": len(holdout_items),
        "encoder_frozen": True,
        "trained_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }
    write_sibling_checkpoint(out_dir, served_dir, agent, fit, meta)
    agent.save_calibration(os.path.join(out_dir, "calibration.json"))
    print(f"[out] sibling checkpoint written to {out_dir}")

    report = {
        "epochs": args.epochs,
        "train_acc": final_train["acc"],
        "holdout_acc": final_hold["acc"] if final_hold else None,
        "history": history,
        "per_surface": {
            "train": final_train["per_surface"],
            "holdout": final_hold["per_surface"] if final_hold else {},
        },
        "per_qtype": final_train["per_qtype"],
        "score_mae_train": final_train["score_mae"],
        "confusion_train": final_train["confusion"],
        "temperatures": {
            "temperature": [float(x) for x in fit["temperature"]],
            "temperature_by_options": {
                str(k): float(v) for k, v in fit["temperature_by_options"].items()
            },
            "n_by_bucket": fit["n_by_bucket"],
        },
        "parity": {"max_abs_dlogits": d_logits, "max_abs_dact": d_act},
        "encoder_cache_mb": round(cache_mb, 1),
        "served_dir": served_dir,
        "out_dir": out_dir,
    }
    report_path = os.path.join(out_dir, "train_report.json")
    with open(report_path, "w") as f:
        json.dump(report, f, indent=2)
        f.write("\n")
    print("[report] final:")
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
