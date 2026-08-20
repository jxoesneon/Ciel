#!/usr/bin/env python3
"""
distill_3d_instincts.py - Autonomous 3D Studio Instinct Consolidation Engine

Processes raw telemetry logs (~/.ciel/instincts/3d_studio_observations.jsonl) from geometry
validators and UV analyzers. Clusters recurring topological failure modes, correlates defect
rates with generative models, and distills high-confidence behavioral instincts into
workspace rules (~/.ciel/rules/3d_spatial_rules.md).
"""

import sys
import os
import json
import argparse
from datetime import datetime
from collections import defaultdict

def consolidate_instincts(instinct_log_path, rules_out_path):
    if not os.path.exists(instinct_log_path):
        return {"status": "NO_DATA", "message": f"Instinct log {instinct_log_path} not found."}

    total_observations = 0
    defect_freq = defaultdict(int)
    generative_models = defaultdict(lambda: {"total": 0, "pruned_shells": 0})
    uv_td_variance = []

    with open(instinct_log_path, 'r', encoding='utf-8') as f:
        for line in f:
            if not line.strip():
                continue
            try:
                obs = json.loads(line)
                total_observations += 1

                # Parse QA Validator defects
                if "defects" in obs:
                    for k, v in obs["defects"].items():
                        if v > 0:
                            defect_freq[k] += v

                # Parse Foundation Model telemetry
                if "model_source" in obs:
                    src = obs["model_source"]
                    generative_models[src]["total"] += 1
                    generative_models[src]["pruned_shells"] += obs.get("pruned_disconnected_shells", 0)

                # Parse UV telemetry
                if "domain" in obs and obs["domain"] == "uv_texel_density":
                    uv_td_variance.append(obs.get("variance_pct", 0.0))

            except Exception:
                pass

    if total_observations == 0:
        return {"status": "NO_DATA", "message": "No valid observations found."}

    # Synthesize Rules
    rules_md = f"""# CIEL Workspace Rules: 3D Spatial & Engineering
**Auto-Distilled by `distill_3d_instincts.py`**
**Last Updated:** {datetime.utcnow().isoformat()}Z
**Total Observations Analyzed:** {total_observations}

## 1. Identified Topological Defect Patterns
Based on historical geometric audits, the following defects must be aggressively guarded against:
"""
    
    if defect_freq:
        for defect, count in sorted(defect_freq.items(), key=lambda x: x[1], reverse=True):
            rules_md += f"- **{defect.replace('_', ' ').title()}**: Encountered {count} times. Ensure `geometry_qa_validator.py --fix` is invoked.\n"
    else:
        rules_md += "- *No significant topological defects recorded.* Geometry generation is stable.\n"

    rules_md += "\n## 2. Generative 3D Foundation Model Profiles\n"
    if generative_models:
        for src, stats in generative_models.items():
            avg_shells = stats["pruned_shells"] / stats["total"] if stats["total"] > 0 else 0
            rules_md += f"- **{src}**: Averages {avg_shells:.1f} internal floating shells per mesh. Mandate high-pass DSU pruning.\n"
    else:
        rules_md += "- *No foundation model telemetry recorded yet.*\n"

    rules_md += "\n## 3. UV & Texel Density Heuristics\n"
    if uv_td_variance:
        avg_var = sum(uv_td_variance) / len(uv_td_variance)
        rules_md += f"- **Average TD Variance**: {avg_var:.1f}%. "
        if avg_var > 15.0:
            rules_md += "WARNING: High variance detected across asset portfolio. Ensure UDIM packing scripts (`uv_texel_analyzer.py`) enforce uniform island scaling.\n"
        else:
            rules_md += "Variance is within acceptable AAA bounds (<15%).\n"
    else:
        rules_md += "- *No UV texel density telemetry recorded yet.*\n"

    # Save to workspace rules
    os.makedirs(os.path.dirname(rules_out_path), exist_ok=True)
    with open(rules_out_path, 'w', encoding='utf-8') as f:
        f.write(rules_md)

    return {
        "status": "SUCCESS",
        "total_observations": total_observations,
        "rules_file": rules_out_path,
        "defect_frequencies": dict(defect_freq)
    }

def main():
    parser = argparse.ArgumentParser(description="Autonomous 3D Studio Instinct Consolidation Engine")
    parser.add_argument("--json", action="store_true", help="Output JSON telemetry")
    args = parser.parse_args()

    instinct_path = os.path.expanduser("~/.ciel/instincts/3d_studio_observations.jsonl")
    rules_path = os.path.expanduser("~/.ciel/rules/3d_spatial_rules.md")
    
    # Fallback to current directory for foundation telemetry if combined instinct log exists
    foundation_path = os.path.expanduser("~/.ciel/instincts/3d_foundation_model_telemetry.jsonl")
    
    # Combine logs for analysis
    combined_log_path = os.path.expanduser("~/.ciel/instincts/combined_3d_instincts.jsonl")
    try:
        os.makedirs(os.path.dirname(combined_log_path), exist_ok=True)
        with open(combined_log_path, 'w', encoding='utf-8') as cout:
            if os.path.exists(instinct_path):
                with open(instinct_path, 'r', encoding='utf-8') as fin:
                    cout.write(fin.read())
            if os.path.exists(foundation_path):
                with open(foundation_path, 'r', encoding='utf-8') as fin:
                    cout.write(fin.read())
    except Exception:
        combined_log_path = instinct_path

    res = consolidate_instincts(combined_log_path, rules_path)

    if args.json:
        print(json.dumps(res, indent=2))
    else:
        print("\n[Instinct Consolidation Engine] 3D Studio Telemetry Analysis:")
        if res["status"] == "SUCCESS":
            print(f" -> Processed {res['total_observations']} observations.")
            print(f" -> Distilled Rules Generated: {res['rules_file']}")
        else:
            print(f" -> {res['message']}")
        print()

if __name__ == "__main__":
    main()
