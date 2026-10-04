//! Port of distill_3d_instincts.py — instinct consolidation engine that
//! clusters recurring defect telemetry into distilled workspace rules.

use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};

use crate::common::cli::{self, ArgSpec};
use crate::common::jsonfmt;
use crate::common::py;

fn title_case(s: &str) -> String {
    // Python .title(): uppercase first letter of each word, rest lowercase.
    let mut out = String::new();
    let mut start_of_word = true;
    for c in s.chars() {
        if c.is_alphabetic() {
            if start_of_word {
                out.extend(c.to_uppercase());
            } else {
                out.extend(c.to_lowercase());
            }
            start_of_word = false;
        } else {
            out.push(c);
            start_of_word = true;
        }
    }
    out
}

fn consolidate_instincts(instinct_log_path: &str, rules_out_path: &str) -> serde_json::Value {
    if !std::path::Path::new(instinct_log_path).exists() {
        return json!({
            "status": "NO_DATA",
            "message": format!("Instinct log {} not found.", instinct_log_path)
        });
    }

    let mut total_observations = 0usize;
    let mut defect_freq: HashMap<String, i64> = HashMap::new();
    let mut defect_order: Vec<String> = Vec::new();
    // Preserve insertion order for generative model output.
    let mut generative_order: Vec<String> = Vec::new();
    let mut generative_models: HashMap<String, (i64, i64)> = HashMap::new();
    let mut uv_td_variance: Vec<f64> = Vec::new();

    if let Ok(file) = fs::File::open(instinct_log_path) {
        for line in BufReader::new(file).lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(obs) = serde_json::from_str::<serde_json::Value>(&line) {
                total_observations += 1;

                if let Some(defects) = obs.get("defects").and_then(|d| d.as_object()) {
                    for (k, v) in defects {
                        if let Some(n) = v.as_i64() {
                            if n > 0 {
                                if !defect_freq.contains_key(k) {
                                    defect_order.push(k.clone());
                                }
                                *defect_freq.entry(k.clone()).or_insert(0) += n;
                            }
                        }
                    }
                }

                if let Some(src) = obs.get("model_source").and_then(|m| m.as_str()) {
                    if !generative_models.contains_key(src) {
                        generative_order.push(src.to_string());
                    }
                    let entry = generative_models.entry(src.to_string()).or_insert((0, 0));
                    entry.0 += 1;
                    entry.1 += obs
                        .get("pruned_disconnected_shells")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                }

                if obs.get("domain").and_then(|d| d.as_str()) == Some("uv_texel_density") {
                    uv_td_variance.push(
                        obs.get("variance_pct")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0),
                    );
                }
            }
        }
    }

    if total_observations == 0 {
        return json!({"status": "NO_DATA", "message": "No valid observations found."});
    }

    let mut rules_md = format!(
        r#"# CIEL Workspace Rules: 3D Spatial & Engineering
**Auto-Distilled by `distill_3d_instincts.py`**
**Last Updated:** {ts}
**Total Observations Analyzed:** {total}

## 1. Identified Topological Defect Patterns
Based on historical geometric audits, the following defects must be aggressively guarded against:
"#,
        ts = py::utcnow_iso_z(),
        total = total_observations
    );

    if !defect_freq.is_empty() {
        // Python sorted(items, key=count, reverse=True) is stable → ties keep
        // first-seen insertion order.
        defect_order.sort_by(|a, b| defect_freq[b].cmp(&defect_freq[a]));
        for defect in &defect_order {
            let count = defect_freq[defect];
            rules_md.push_str(&format!(
                "- **{}**: Encountered {} times. Ensure `geometry_qa_validator.py --fix` is invoked.\n",
                title_case(&defect.replace('_', " ")),
                count
            ));
        }
    } else {
        rules_md.push_str(
            "- *No significant topological defects recorded.* Geometry generation is stable.\n",
        );
    }

    rules_md.push_str("\n## 2. Generative 3D Foundation Model Profiles\n");
    if !generative_models.is_empty() {
        for src in &generative_order {
            let (total, shells) = generative_models[src];
            let avg_shells = if total > 0 {
                shells as f64 / total as f64
            } else {
                0.0
            };
            rules_md.push_str(&format!(
                "- **{}**: Averages {:.1} internal floating shells per mesh. Mandate high-pass DSU pruning.\n",
                src, avg_shells
            ));
        }
    } else {
        rules_md.push_str("- *No foundation model telemetry recorded yet.*\n");
    }

    rules_md.push_str("\n## 3. UV & Texel Density Heuristics\n");
    if !uv_td_variance.is_empty() {
        let avg_var = uv_td_variance.iter().sum::<f64>() / uv_td_variance.len() as f64;
        rules_md.push_str(&format!("- **Average TD Variance**: {:.1}%. ", avg_var));
        if avg_var > 15.0 {
            rules_md.push_str("WARNING: High variance detected across asset portfolio. Ensure UDIM packing scripts (`uv_texel_analyzer.py`) enforce uniform island scaling.\n");
        } else {
            rules_md.push_str("Variance is within acceptable AAA bounds (<15%).\n");
        }
    } else {
        rules_md.push_str("- *No UV texel density telemetry recorded yet.*\n");
    }

    if let Some(parent) = std::path::Path::new(rules_out_path).parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(rules_out_path, rules_md);

    let defect_map: serde_json::Map<String, serde_json::Value> = defect_freq
        .iter()
        .map(|(k, v)| (k.clone(), json!(v)))
        .collect();

    json!({
        "status": "SUCCESS",
        "total_observations": total_observations,
        "rules_file": rules_out_path,
        "defect_frequencies": serde_json::Value::Object(defect_map)
    })
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-studio distill",
        argv,
        &[ArgSpec::flag("json", None, "json")],
    );

    let instinct_path = py::expanduser("~/.ciel/instincts/3d_studio_observations.jsonl")
        .to_string_lossy()
        .into_owned();
    let rules_path = py::expanduser("~/.ciel/rules/3d_spatial_rules.md")
        .to_string_lossy()
        .into_owned();
    let foundation_path = py::expanduser("~/.ciel/instincts/3d_foundation_model_telemetry.jsonl")
        .to_string_lossy()
        .into_owned();

    let mut combined_log_path = py::expanduser("~/.ciel/instincts/combined_3d_instincts.jsonl")
        .to_string_lossy()
        .into_owned();
    {
        let parent_ok = std::path::Path::new(&combined_log_path)
            .parent()
            .map(|p| fs::create_dir_all(p).is_ok())
            .unwrap_or(false);
        let mut combined = String::new();
        if let Ok(c) = fs::read_to_string(&instinct_path) {
            combined.push_str(&c);
        }
        if let Ok(c) = fs::read_to_string(&foundation_path) {
            combined.push_str(&c);
        }
        if parent_ok && fs::write(&combined_log_path, &combined).is_err() {
            combined_log_path = instinct_path.clone();
        }
    }

    let res = consolidate_instincts(&combined_log_path, &rules_path);

    if args.flag("json") {
        println!("{}", jsonfmt::dumps_indent(&res, 2));
    } else {
        println!("\n[Instinct Consolidation Engine] 3D Studio Telemetry Analysis:");
        if res["status"].as_str() == Some("SUCCESS") {
            println!(" -> Processed {} observations.", res["total_observations"]);
            println!(
                " -> Distilled Rules Generated: {}",
                res["rules_file"].as_str().unwrap_or("")
            );
        } else {
            println!(" -> {}", res["message"].as_str().unwrap_or(""));
        }
        println!();
    }
    0
}
