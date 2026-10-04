//! `scan-skills` — port of `scripts/scan_skills.py`. Aggregates per-skill
//! `skillfrisk scan <dir> --json` results into one report. Advisory by
//! default; exit 2 when any skill reports `failed: true` or `--fail-on`
//! severity is met (non-baselined findings only).

use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::{jsonfmt, py};

fn severity_rank(findings: &[Value]) -> u32 {
    let mut rank = 0u32;
    for f in findings {
        let sev = f
            .get("severity")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let r = match sev.as_str() {
            "none" => 0,
            "high" => 2,
            _ => 1, // any other label maps to "any" rank like Python's .get(sev, 1)
        };
        rank = rank.max(r);
    }
    rank
}

fn severity_order(name: &str) -> u32 {
    match name {
        "none" => 0,
        "any" => 1,
        "high" => 2,
        _ => 0,
    }
}

fn scan_cmd(dir_path: &Path) -> Vec<String> {
    if let Ok(override_cmd) = std::env::var("SCAN_SKILLS_CMD") {
        if !override_cmd.is_empty() {
            let mut parts: Vec<String> =
                override_cmd.split_whitespace().map(String::from).collect();
            parts.push(dir_path.to_string_lossy().into_owned());
            parts.push("--json".into());
            return parts;
        }
    }
    vec![
        "uvx".into(),
        "skillfrisk".into(),
        "scan".into(),
        dir_path.to_string_lossy().into_owned(),
        "--json".into(),
    ]
}

fn which(bin: &str) -> bool {
    if let Ok(paths) = std::env::var("PATH") {
        for dir in paths.split(':') {
            let p = Path::new(dir).join(bin);
            if p.is_file() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if p.metadata()
                        .map(|m| m.permissions().mode() & 0o111 != 0)
                        .unwrap_or(false)
                    {
                        return true;
                    }
                }
                #[cfg(not(unix))]
                {
                    return true;
                }
            }
        }
    }
    false
}

fn tool_available() -> bool {
    std::env::var("SCAN_SKILLS_CMD")
        .map(|s| !s.is_empty())
        .unwrap_or(false)
        || which("uvx")
}

fn scan_one(dir_path: &Path) -> Value {
    let argv = scan_cmd(dir_path);
    let out = Command::new(&argv[0])
        .args(&argv[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
    let name = dir_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    match out {
        Err(e) => json!({"skill": name, "error": e.to_string()}),
        Ok(proc) => {
            let stdout = String::from_utf8_lossy(&proc.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&proc.stderr).into_owned();
            if !proc.status.success() && stdout.trim().is_empty() {
                let err = if stderr.trim().is_empty() {
                    format!("exit {}", proc.status.code().unwrap_or(-1))
                } else {
                    stderr.trim().to_string()
                };
                return json!({"skill": name, "error": err});
            }
            match serde_json::from_str::<Value>(&stdout) {
                Ok(mut data) => {
                    if let Some(m) = data.as_object_mut() {
                        m.insert("skill".into(), json!(name));
                        data
                    } else {
                        json!({"skill": name, "error": "scanner output is not an object"})
                    }
                }
                Err(e) => json!({"skill": name, "error": format!("{e}")}),
            }
        }
    }
}

fn skill_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for base in [
        root.join("skills"),
        root.join("ciel.skill").join("seed_skills"),
    ] {
        if base.is_dir() {
            let mut entries: Vec<PathBuf> = std::fs::read_dir(&base)
                .map(|rd| {
                    rd.flatten()
                        .map(|e| e.path())
                        .filter(|p| p.join("SKILL.md").is_file())
                        .collect()
                })
                .unwrap_or_default();
            entries.sort();
            dirs.extend(entries);
        }
    }
    dirs
}

fn load_baseline(path: &Path) -> Vec<Map<String, Value>> {
    let data: Value = match std::fs::read_to_string(path).and_then(|t| {
        serde_json::from_str(&t)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[scan] baseline unreadable ({e}); ignoring");
            return Vec::new();
        }
    };
    data.get("accepted")
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| e.as_object())
                .filter(|e| {
                    e.get("skill")
                        .and_then(|v| v.as_str())
                        .is_some_and(|s| !s.is_empty())
                        && e.get("rule_id")
                            .and_then(|v| v.as_str())
                            .is_some_and(|s| !s.is_empty())
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

fn baselined<'b>(
    skill: &str,
    finding: &Value,
    baseline: &'b [Map<String, Value>],
) -> Option<&'b Map<String, Value>> {
    for entry in baseline {
        if entry.get("skill").and_then(|v| v.as_str()) != Some(skill) {
            continue;
        }
        if entry.get("rule_id") != finding.get("rule_id") {
            continue;
        }
        if let Some(p) = entry.get("path") {
            // `if entry.get("path") and entry["path"] != finding.get("path")`
            // — falsy path values carry no constraint.
            let truthy = !(p.is_null()
                || p == &Value::Bool(false)
                || p == &json!("")
                || p.as_f64() == Some(0.0)
                || p == &json!([])
                || p == &json!({}));
            if truthy && Some(p) != finding.get("path") {
                continue;
            }
        }
        return Some(entry);
    }
    None
}

struct Args {
    paths: Vec<PathBuf>,
    all: bool,
    fail_on: String,
    report: Option<PathBuf>,
    baseline: Option<PathBuf>,
    no_baseline: bool,
    strict: bool,
}

fn parse_args(args: &[String]) -> Result<Args, i32> {
    let mut a = Args {
        paths: Vec::new(),
        all: false,
        fail_on: "none".into(),
        report: None,
        baseline: None,
        no_baseline: false,
        strict: false,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--all" => a.all = true,
            "--fail-on" => {
                i += 1;
                let v = args.get(i).cloned().ok_or(2)?;
                if !["high", "any", "none"].contains(&v.as_str()) {
                    eprintln!("ciel-dev scan-skills: invalid --fail-on '{v}'");
                    return Err(2);
                }
                a.fail_on = v;
            }
            "--report" => {
                i += 1;
                a.report = args.get(i).map(PathBuf::from);
            }
            "--baseline" => {
                i += 1;
                a.baseline = args.get(i).map(PathBuf::from);
            }
            "--no-baseline" => a.no_baseline = true,
            "--strict" => a.strict = true,
            "-h" | "--help" => return Err(0),
            s => {
                if s.starts_with('-') {
                    return Err(2);
                }
                a.paths.push(PathBuf::from(s));
            }
        }
        i += 1;
    }
    Ok(a)
}

pub fn main_(args: &[String]) -> i32 {
    let a = match parse_args(args) {
        Ok(a) => a,
        Err(code) => {
            if code != 0 {
                eprintln!("usage: ciel-dev scan-skills [paths...] [--all] [--fail-on high|any|none] [--report PATH] [--baseline PATH] [--no-baseline] [--strict]");
            } else {
                println!("usage: ciel-dev scan-skills [paths...] [--all] [--fail-on high|any|none] [--report PATH] [--baseline PATH] [--no-baseline] [--strict]");
            }
            return code;
        }
    };

    let root = py::repo_root();
    let mut baseline_path = a.baseline.clone();
    if baseline_path.is_none() && !a.no_baseline {
        let default = root
            .join("ciel.skill")
            .join("risk")
            .join("skill_scan_baseline.json");
        if default.is_file() {
            baseline_path = Some(default);
        }
    }
    let baseline = baseline_path
        .as_ref()
        .map(|p| load_baseline(p))
        .unwrap_or_default();

    let mut targets = a.paths.clone();
    if a.all {
        targets.extend(skill_dirs(&root));
    }
    if targets.is_empty() {
        eprintln!("[scan] no skill dirs given (use --all or pass paths)");
        return if a.strict { 2 } else { 0 };
    }
    if !tool_available() {
        let msg = "[scan] skillfrisk unavailable (uvx missing); advisory pass";
        if a.strict {
            eprintln!("{msg} — strict mode: failing");
            return 2;
        }
        println!("{msg}");
        return 0;
    }

    let mut scanned = 0u64;
    let mut raw_failed = std::collections::HashSet::new();
    let mut findings_by_skill: Map<String, Value> = Map::new();
    let mut accepted: Vec<Value> = Vec::new();
    let mut errors: Vec<Value> = Vec::new();

    for d in &targets {
        let result = scan_one(d);
        let name = d
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if let Some(err) = result.get("error") {
            errors.push(result.clone());
            eprintln!(
                "[scan] {name}: scanner error: {}",
                err.as_str().unwrap_or("")
            );
            continue;
        }
        scanned += 1;
        if result.get("failed").map(py::json_truthy).unwrap_or(false) {
            raw_failed.insert(name.clone());
        }
        let mut remaining = Vec::new();
        for finding in result
            .get("findings")
            .and_then(|f| f.as_array())
            .cloned()
            .unwrap_or_default()
        {
            if let Some(entry) = baselined(&name, &finding, &baseline) {
                accepted.push(json!({
                    "skill": name,
                    "finding": finding,
                    "justification": entry.get("justification").cloned().unwrap_or(json!("")),
                }));
            } else {
                remaining.push(finding);
            }
        }
        if !remaining.is_empty() {
            findings_by_skill.insert(name.clone(), json!(remaining));
        }
    }

    let threshold = severity_order(&a.fail_on);
    let mut failed: Vec<String> = findings_by_skill
        .iter()
        .filter(|(name, findings)| {
            raw_failed.contains(*name)
                || (threshold > 0
                    && severity_rank(findings.as_array().map(|v| v.as_slice()).unwrap_or(&[]))
                        >= threshold)
        })
        .map(|(name, _)| name.clone())
        .collect();
    failed.sort();

    let report = json!({
        "ts": py::iso_now_utc(),
        "scanned": scanned,
        "failed": failed,
        "findings_by_skill": findings_by_skill,
        "accepted": accepted,
        "capability": {
            "tool": "skillfrisk",
            "invocation": "uvx skillfrisk scan <dir> --json",
            "fail_on": a.fail_on,
            "baseline": baseline_path.as_ref().map(|p| p.to_string_lossy().into_owned()),
            "errors": errors,
        },
    });
    if let Some(report_path) = &a.report {
        let _ = std::fs::write(
            report_path,
            format!("{}\n", jsonfmt::dumps_indent_ascii(&report, 2)),
        );
    }

    let total_findings: usize = findings_by_skill
        .values()
        .map(|v| v.as_array().map(|a| a.len()).unwrap_or(0))
        .sum();
    let mut line = format!(
        "[scan] scanned={scanned} failed={} findings={total_findings}",
        report["failed"].as_array().map(|f| f.len()).unwrap_or(0)
    );
    let accepted_n = report["accepted"].as_array().map(|v| v.len()).unwrap_or(0);
    if accepted_n > 0 {
        line += &format!(" accepted={accepted_n}");
    }
    let errors_n = report["capability"]["errors"]
        .as_array()
        .map(|v| v.len())
        .unwrap_or(0);
    if errors_n > 0 {
        line += &format!(" errors={errors_n}");
    }
    println!("{line}");
    if report["failed"]
        .as_array()
        .map(|f| !f.is_empty())
        .unwrap_or(false)
    {
        2
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_order_thresholds() {
        assert_eq!(severity_order("none"), 0);
        assert_eq!(severity_order("any"), 1);
        assert_eq!(severity_order("high"), 2);
        assert_eq!(severity_order("bogus"), 0);
    }

    #[test]
    fn severity_rank_max_and_unknown() {
        let findings = vec![json!({"severity": "low"}), json!({"severity": "high"})];
        assert_eq!(severity_rank(&findings), 2);
        let findings = vec![json!({"severity": "none"})];
        assert_eq!(severity_rank(&findings), 0);
        // Unknown label → rank 1 ("any"), matching SEVERITY_ORDER.get(sev, 1).
        let findings = vec![json!({"severity": "weird"}), json!({}), json!("x")];
        assert_eq!(severity_rank(&findings), 1);
        assert_eq!(severity_rank(&[]), 0);
    }

    fn baseline(entries: Value) -> Vec<Map<String, Value>> {
        entries
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|e| e.as_object().cloned())
            .collect()
    }

    #[test]
    fn baselined_skill_rule_match() {
        let bl = baseline(json!([
            {"skill": "s1", "rule_id": "R1", "justification": "ok"},
        ]));
        let finding = json!({"rule_id": "R1", "severity": "high"});
        assert!(baselined("s1", &finding, &bl).is_some());
        assert!(baselined("s2", &finding, &bl).is_none());
        assert!(baselined("s1", &json!({"rule_id": "R2"}), &bl).is_none());
    }

    #[test]
    fn baselined_path_constraint() {
        let bl = baseline(json!([
            {"skill": "s1", "rule_id": "R1", "path": "SKILL.md"},
            {"skill": "s2", "rule_id": "R2", "path": ""},
            {"skill": "s3", "rule_id": "R3", "path": null},
        ]));
        // Truthy path must equal the finding's path.
        assert!(baselined("s1", &json!({"rule_id": "R1", "path": "SKILL.md"}), &bl).is_some());
        assert!(baselined("s1", &json!({"rule_id": "R1", "path": "other.md"}), &bl).is_none());
        // Falsy path values ("", null) carry no constraint.
        assert!(baselined("s2", &json!({"rule_id": "R2", "path": "any.md"}), &bl).is_some());
        assert!(baselined("s3", &json!({"rule_id": "R3"}), &bl).is_some());
    }

    #[test]
    fn load_baseline_filters_invalid_entries() {
        let dir = std::env::temp_dir().join(format!("ciel-dev-scan-bl-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("b.json");
        std::fs::write(
            &p,
            json!({"accepted": [
                {"skill": "s", "rule_id": "r"},
                {"skill": "s"},
                "not-a-dict",
                {"skill": "", "rule_id": "r"},
            ]})
            .to_string(),
        )
        .unwrap();
        let bl = load_baseline(&p);
        assert_eq!(bl.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
