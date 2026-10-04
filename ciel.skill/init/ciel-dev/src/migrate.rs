//! `migrate-sidecar` — port of `scripts/migrate_skill_sidecar.py`. Moves
//! non-spec SKILL.md frontmatter keys into a `ciel.yaml` sidecar.

use serde_yaml::{Mapping, Value};
use std::path::{Path, PathBuf};

const SIDECAR: &str = "ciel.yaml";
const SIDECAR_HEADER: &str =
    "# Ciel skill extension — fields beyond the Agent Skills spec. SKILL.md stays spec-pure.\n";
const SPEC_KEYS: [&str; 6] = [
    "name",
    "description",
    "license",
    "compatibility",
    "metadata",
    "allowed-tools",
];

fn is_spec_key(k: &Value) -> bool {
    k.as_str().map(|s| SPEC_KEYS.contains(&s)).unwrap_or(false)
}

/// `_split_frontmatter` → (frontmatter_map, body) or (None, text).
fn split_frontmatter(text: &str) -> Result<(Option<Mapping>, String), String> {
    if !text.starts_with("---\n") {
        return Ok((None, text.to_string()));
    }
    let Some(end) = text[4..].find("\n---").map(|i| i + 4) else {
        return Ok((None, text.to_string()));
    };
    let fm_text = &text[4..end];
    let body = &text[end + 1..];
    let body = match body.find('\n') {
        Some(nl) => &body[nl + 1..],
        None => "",
    };
    let parsed: Value =
        serde_yaml::from_str(fm_text).map_err(|e| format!("frontmatter parse: {e}"))?;
    match parsed {
        // `yaml.safe_load("")` → None → `_split_frontmatter` returns
        // (None, body) → "no frontmatter, skipped".
        Value::Null => Ok((None, text.to_string())),
        // PyYAML `yaml.safe_load` returns whatever the doc is; a non-mapping
        // crashes downstream — mirror as an error, not silent loss.
        Value::Mapping(m) => Ok((Some(m), body.to_string())),
        other => Err(format!("frontmatter is not a mapping ({other:?})")),
    }
}

/// `yaml.safe_dump(..., sort_keys=False, allow_unicode=True, width=4096)`.
fn dump_yaml(m: &Mapping) -> String {
    let mut s = serde_yaml::to_string(&Value::Mapping(m.clone())).unwrap_or_default();
    if !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// `migrate_skill` — returns Ok(true) when changes were written.
fn migrate_skill(skill_dir: &Path) -> Result<bool, String> {
    let skill_md = skill_dir.join("SKILL.md");
    let sidecar = skill_dir.join(SIDECAR);
    let text =
        std::fs::read_to_string(&skill_md).map_err(|e| format!("{}: {e}", skill_md.display()))?;
    let (fm, body) = split_frontmatter(&text)?;
    let Some(fm) = fm else {
        eprintln!("[migrate] {}: no frontmatter, skipped", skill_md.display());
        return Ok(false);
    };

    let extra: Vec<(Value, Value)> = fm
        .iter()
        .filter(|(k, _)| !is_spec_key(k))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let already = extra.is_empty() && sidecar.is_file();
    if already {
        return Ok(false);
    }

    if !extra.is_empty() {
        let mut sidecar_data = Mapping::new();
        sidecar_data.insert(Value::String("schema".into()), Value::Number(1.into()));
        for (k, v) in &extra {
            sidecar_data.insert(k.clone(), v.clone());
        }
        std::fs::write(
            &sidecar,
            format!("{}{}", SIDECAR_HEADER, dump_yaml(&sidecar_data)),
        )
        .map_err(|e| format!("{}: {e}", sidecar.display()))?;
    }

    let mut new_fm = Mapping::new();
    for (k, v) in &fm {
        if is_spec_key(k) {
            new_fm.insert(k.clone(), v.clone());
        }
    }
    let mut metadata = new_fm
        .get("metadata")
        .and_then(|v| v.as_mapping())
        .cloned()
        .unwrap_or_default();
    if !metadata.contains_key("ciel-version") {
        let version = extra
            .iter()
            .find(|(k, _)| k.as_str() == Some("version"))
            .map(|(_, v)| v.clone())
            .or_else(|| metadata.get("ciel-version").cloned());
        // `str(extra.get("version", metadata.get("ciel-version", "1.0.0")))`
        let version_str = match version {
            Some(Value::String(s)) => s,
            Some(v) => crate::py::py_str(&v),
            None => "1.0.0".to_string(),
        };
        metadata.insert(
            Value::String("ciel-version".into()),
            Value::String(version_str),
        );
    }
    metadata.insert(
        Value::String("ciel-extension".into()),
        Value::String(SIDECAR.into()),
    );
    new_fm.insert(Value::String("metadata".into()), Value::Mapping(metadata));

    std::fs::write(
        &skill_md,
        format!("---\n{}---\n{}", dump_yaml(&new_fm), body),
    )
    .map_err(|e| format!("{}: {e}", skill_md.display()))?;
    Ok(true)
}

/// `check_skill` — list of problems; empty means conformant.
fn check_skill(skill_dir: &Path) -> Vec<String> {
    let mut problems = Vec::new();
    let skill_md = skill_dir.join("SKILL.md");
    let text = match std::fs::read_to_string(&skill_md) {
        Ok(t) => t,
        Err(e) => {
            problems.push(format!("unreadable SKILL.md: {e}"));
            return problems;
        }
    };
    let fm = match split_frontmatter(&text) {
        Ok((Some(m), _)) => m,
        _ => {
            problems.push("no frontmatter".to_string());
            return problems;
        }
    };
    let extra: Vec<String> = fm
        .keys()
        .filter(|k| !is_spec_key(k))
        .map(|k| k.as_str().map(String::from).unwrap_or_default())
        .collect();
    if !extra.is_empty() {
        problems.push(format!("non-spec keys in SKILL.md: {}", extra.join(", ")));
    }
    let metadata = fm.get("metadata").and_then(|v| v.as_mapping());
    let ext_ok = metadata
        .and_then(|m| m.get("ciel-extension"))
        .and_then(|v| v.as_str())
        == Some(SIDECAR);
    if !ext_ok {
        problems.push("missing metadata.ciel-extension: ciel.yaml".to_string());
    }
    if !skill_dir.join(SIDECAR).is_file() {
        problems.push("missing ciel.yaml sidecar".to_string());
    }
    problems
}

fn skill_dirs(root: &Path) -> Vec<PathBuf> {
    // root.glob("skills/*/SKILL.md") → immediate children only.
    let skills = root.join("skills");
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&skills)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.join("SKILL.md").is_file())
                .collect()
        })
        .unwrap_or_default();
    dirs.sort();
    dirs
}

struct Args {
    apply: bool,
    check: bool,
    root: PathBuf,
}

fn parse_args(args: &[String]) -> Result<Args, i32> {
    let mut a = Args {
        apply: false,
        check: false,
        root: crate::py::repo_root(),
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--apply" => a.apply = true,
            "--check" => a.check = true,
            "--root" => {
                i += 1;
                a.root = args.get(i).map(PathBuf::from).ok_or(2)?;
            }
            "-h" | "--help" => return Err(0),
            _ => return Err(2),
        }
        i += 1;
    }
    if a.apply == a.check {
        eprintln!("ciel-dev migrate-sidecar: one of --apply or --check is required");
        return Err(2);
    }
    Ok(a)
}

pub fn main_(args: &[String]) -> i32 {
    let a = match parse_args(args) {
        Ok(a) => a,
        Err(code) => {
            if code != 0 {
                eprintln!("usage: ciel-dev migrate-sidecar (--apply | --check) [--root DIR]");
            } else {
                println!("usage: ciel-dev migrate-sidecar (--apply | --check) [--root DIR]");
            }
            return code;
        }
    };

    let skills = skill_dirs(&a.root);
    if skills.is_empty() {
        eprintln!("[migrate] no skills under {}/skills/", a.root.display());
        return 1;
    }

    if a.apply {
        let mut changed = 0u64;
        for d in &skills {
            match migrate_skill(d) {
                Ok(true) => changed += 1,
                Ok(false) => {}
                Err(e) => {
                    eprintln!("[migrate] {e}");
                    return 1;
                }
            }
        }
        println!(
            "[migrate] {} skills scanned, {changed} migrated",
            skills.len()
        );
        return 0;
    }

    let mut failures = 0u64;
    for d in &skills {
        let name = d
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        for problem in check_skill(d) {
            eprintln!("[check] {name}: {problem}");
            failures += 1;
        }
    }
    if failures > 0 {
        eprintln!(
            "[check] {failures} problem(s) across {} skills",
            skills.len()
        );
        return 1;
    }
    println!("[check] {} skills conformant", skills.len());
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    const OLD_SKILL: &str = "---\nname: widget\nversion: 2.1.0\nformat: skill/1.0\ndescription: A test widget skill.\nruntimes: [\"claude_code\", \"generic\"]\nlicense: MIT\ntags: [\"ciel\", \"domain:test\"]\ntriggers:\n  - pattern: \"widget.*\"\n    confidence: 0.9\nsource: { tier: 1, origin: test }\ndependencies: { skills: [], mcp: [], system: [] }\nside_effects: [\"filesystem\"]\n---\n\n# Widget\n\nBody text preserved byte-for-byte.\n";

    fn tmp_skill(content: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ciel-dev-mig-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let skill = dir.join("widget");
        std::fs::create_dir_all(&skill).unwrap();
        std::fs::write(skill.join("SKILL.md"), content).unwrap();
        skill
    }

    #[test]
    fn split_frontmatter_variants() {
        let (fm, body) = split_frontmatter("---\nname: x\n---\nbody\n").unwrap();
        assert!(fm.is_some());
        assert_eq!(body, "body\n");
        let (fm, _) = split_frontmatter("no front\n").unwrap();
        assert!(fm.is_none());
        // empty fm doc → yaml.safe_load → None → treated as no frontmatter
        let (fm, _) = split_frontmatter("---\n---\nbody\n").unwrap();
        assert!(fm.is_none());
        // unclosed fm → no frontmatter
        let (fm, _) = split_frontmatter("---\nname: x\nbody\n").unwrap();
        assert!(fm.is_none());
    }

    #[test]
    fn migrate_splits_and_is_idempotent() {
        let skill = tmp_skill(OLD_SKILL);
        assert!(migrate_skill(&skill).unwrap());

        let text = std::fs::read_to_string(skill.join("SKILL.md")).unwrap();
        let (fm_v, body) = split_frontmatter(&text).unwrap();
        let fm = fm_v.unwrap();
        let keys: Vec<String> = fm
            .keys()
            .filter_map(|k| k.as_str().map(String::from))
            .collect();
        let mut keep: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
        keep.retain(|k| *k != "metadata");
        keep.sort();
        assert_eq!(keep, ["description", "license", "name"]);
        let meta = fm.get("metadata").and_then(|v| v.as_mapping()).unwrap();
        assert_eq!(
            meta.get("ciel-extension").and_then(|v| v.as_str()),
            Some("ciel.yaml")
        );
        assert_eq!(
            meta.get("ciel-version").and_then(|v| v.as_str()),
            Some("2.1.0")
        );
        // body preserved byte-for-byte; Python `_split_frontmatter` drops the
        // closing `---` line, leaving one leading newline.
        assert_eq!(body, "\n# Widget\n\nBody text preserved byte-for-byte.\n");

        let sidecar: serde_yaml::Value =
            serde_yaml::from_str(&std::fs::read_to_string(skill.join("ciel.yaml")).unwrap())
                .unwrap();
        assert_eq!(sidecar["schema"], serde_yaml::Value::Number(1.into()));
        for k in [
            "version",
            "format",
            "runtimes",
            "tags",
            "triggers",
            "source",
            "dependencies",
            "side_effects",
        ] {
            assert!(sidecar.get(k).is_some(), "missing {k}");
        }
        assert_eq!(
            sidecar["version"],
            serde_yaml::from_str::<serde_yaml::Value>("2.1.0").unwrap()
        );

        // second run writes nothing
        assert!(!migrate_skill(&skill).unwrap());
        assert!(check_skill(&skill).is_empty());
        let _ = std::fs::remove_dir_all(dir_of(&skill));
    }

    #[test]
    fn check_reports_all_problems() {
        let skill = tmp_skill(OLD_SKILL);
        let problems = check_skill(&skill);
        assert_eq!(
            problems,
            vec![
                "non-spec keys in SKILL.md: version, format, runtimes, tags, triggers, source, dependencies, side_effects".to_string(),
                "missing metadata.ciel-extension: ciel.yaml".to_string(),
                "missing ciel.yaml sidecar".to_string(),
            ]
        );
        let _ = std::fs::remove_dir_all(dir_of(&skill));
    }

    #[test]
    fn migrate_skips_no_frontmatter() {
        let skill = tmp_skill("# just markdown\n");
        assert!(!migrate_skill(&skill).unwrap());
        assert_eq!(check_skill(&skill), vec!["no frontmatter".to_string()]);
        let _ = std::fs::remove_dir_all(dir_of(&skill));
    }

    fn dir_of(skill: &Path) -> PathBuf {
        skill.parent().unwrap().to_path_buf()
    }
}
