//! `registry-index` — port of `scripts/build_registry_index.py`. Builds
//! `<home>/.ciel/registry/index.json` from `*/SKILL.md` frontmatter merged
//! with sibling `ciel.yaml`, written atomically (tmp + rename).

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use crate::{jsonfmt, py};

fn yaml_to_json(v: serde_yaml::Value) -> Value {
    match v {
        serde_yaml::Value::Null => Value::Null,
        serde_yaml::Value::Bool(b) => json!(b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                json!(i)
            } else if let Some(u) = n.as_u64() {
                json!(u)
            } else {
                json!(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_yaml::Value::String(s) => json!(s),
        serde_yaml::Value::Sequence(a) => Value::Array(a.into_iter().map(yaml_to_json).collect()),
        serde_yaml::Value::Mapping(m) => {
            let mut out = Map::new();
            for (k, v) in m {
                let key = match k {
                    serde_yaml::Value::String(s) => s,
                    other => py::py_str(&other),
                };
                out.insert(key, yaml_to_json(v));
            }
            Value::Object(out)
        }
        serde_yaml::Value::Tagged(t) => yaml_to_json(t.value),
    }
}

fn load_yaml(text: &str) -> Map<String, Value> {
    match serde_yaml::from_str::<serde_yaml::Value>(text) {
        Ok(v) => match yaml_to_json(v) {
            Value::Object(m) => m,
            _ => Map::new(),
        },
        Err(_) => Map::new(),
    }
}

fn frontmatter(path: &Path) -> Map<String, Value> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Map::new();
    };
    if !text.starts_with("---") {
        return Map::new();
    }
    let Some(end) = text[3..].find("\n---").map(|i| i + 3) else {
        return Map::new();
    };
    load_yaml(&text[3..end])
}

fn rglob_files(path: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(path) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            rglob_files(&p, out);
        } else if p.is_file() {
            out.push(p);
        }
    }
}

fn checksum(path: &Path) -> String {
    let mut files = Vec::new();
    rglob_files(path, &mut files);
    files.sort();
    let mut h = Sha256::new();
    for f in files {
        if let Ok(rel) = f.strip_prefix(path) {
            h.update(rel.to_string_lossy().as_bytes());
        }
        if let Ok(bytes) = std::fs::read(&f) {
            h.update(&bytes);
        }
    }
    let digest: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    format!("sha256:{digest}")
}

fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64() != Some(0.0),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
    }
}

/// `str(v)` for the version chain.
fn str_or(v: Option<&Value>) -> Option<String> {
    v.filter(|v| truthy(Some(v))).map(|v| match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => {
            if *b {
                "True".into()
            } else {
                "False".into()
            }
        }
        Value::Null => "None".into(),
        other => jsonfmt::dumps_raw(other),
    })
}

fn stat_iso(path: &Path, ctime: bool) -> String {
    let Ok(md) = path.metadata() else {
        return py::iso_now_utc();
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let (secs, nanos) = if ctime {
            (md.ctime(), md.ctime_nsec())
        } else {
            (md.mtime(), md.mtime_nsec())
        };
        py::iso_from_unix(secs as f64 + nanos as f64 / 1e9)
    }
    #[cfg(not(unix))]
    {
        let t = if ctime {
            md.created().ok()
        } else {
            md.modified().ok()
        };
        t.map(|t| {
            let d = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
            py::iso_from_unix(d.as_secs_f64())
        })
        .unwrap_or_else(py::iso_now_utc)
    }
}

fn build_entry(skill_dir: &Path) -> Option<Value> {
    let skill_md = skill_dir.join("SKILL.md");
    if !skill_md.is_file() {
        return None;
    }
    let fm = frontmatter(&skill_md);
    let ciel = skill_dir.join("ciel.yaml");
    let ext = if ciel.is_file() {
        std::fs::read_to_string(&ciel)
            .map(|t| load_yaml(&t))
            .unwrap_or_default()
    } else {
        Map::new()
    };
    let triggers = ext
        .get("triggers")
        .and_then(|t| t.as_array())
        .cloned()
        .unwrap_or_default();
    let trig_patterns: Vec<Value> = triggers
        .iter()
        .map(|t| {
            if let Value::Object(o) = t {
                o.get("pattern").cloned().unwrap_or(json!(""))
            } else {
                str_or(Some(t)).map(Value::String).unwrap_or(Value::Null)
            }
        })
        .collect();

    let version = str_or(ext.get("version"))
        .or_else(|| str_or(fm.get("version")))
        .unwrap_or_else(|| "0.0.0".into());
    let tags = if truthy(ext.get("tags")) {
        ext.get("tags").cloned()
    } else if truthy(fm.get("tags")) {
        fm.get("tags").cloned()
    } else {
        None
    }
    .unwrap_or(json!([]));
    let license = if truthy(fm.get("license")) {
        fm.get("license").cloned()
    } else {
        ext.get("source")
            .and_then(|s| s.as_object())
            .and_then(|s| s.get("license"))
            .cloned()
    }
    .unwrap_or(Value::Null);
    let source = if truthy(ext.get("source")) {
        ext.get("source").cloned().unwrap()
    } else {
        json!({"tier": 0, "origin": "local"})
    };
    let state = ext.get("state").cloned().unwrap_or(json!("validated"));

    let mut entry = json!({
        "id": skill_dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        "version": version,
        "description": fm.get("description").cloned().unwrap_or(json!("")),
        "triggers": trig_patterns,
        "tags": tags,
        "license": license,
        "source": source,
        "install_path": format!("{}/", skill_dir.to_string_lossy()),
        "state": state,
        "checksum": checksum(skill_dir),
        "created": stat_iso(&skill_md, true),
        "last_updated": stat_iso(&skill_md, false),
    });
    let obj = entry.as_object_mut().unwrap();
    for opt in ["dependencies", "side_effects", "runtimes"] {
        if let Some(v) = ext.get(opt) {
            obj.insert(opt.to_string(), v.clone());
        }
    }
    Some(entry)
}

struct Args {
    skills_dir: PathBuf,
    out: PathBuf,
}

fn parse_args(args: &[String]) -> Result<Args, i32> {
    let home = py::home_dir();
    let mut a = Args {
        skills_dir: std::env::var("CIEL_SKILLS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".ciel").join("skills")),
        out: home.join(".ciel").join("registry").join("index.json"),
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--skills-dir" => {
                i += 1;
                a.skills_dir = args.get(i).map(PathBuf::from).ok_or(2)?;
            }
            "--out" => {
                i += 1;
                a.out = args.get(i).map(PathBuf::from).ok_or(2)?;
            }
            "-h" | "--help" => return Err(0),
            _ => return Err(2),
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
                eprintln!("usage: ciel-dev registry-index [--skills-dir DIR] [--out PATH]");
            } else {
                println!("usage: ciel-dev registry-index [--skills-dir DIR] [--out PATH]");
            }
            return code;
        }
    };

    let mut entries = Map::new();
    let mut skipped: Vec<String> = Vec::new();
    let rd = match std::fs::read_dir(&a.skills_dir) {
        Ok(rd) => rd,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    let mut dirs: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
    dirs.sort();
    for d in dirs {
        if !d.is_dir() {
            continue;
        }
        match build_entry(&d) {
            None => skipped.push(
                d.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            ),
            Some(e) => {
                entries.insert(
                    e.get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    e,
                );
            }
        }
    }

    let index = json!({
        "schema": 1,
        "built": py::iso_now_utc(),
        "skills_dir": a.skills_dir.to_string_lossy(),
        "count": entries.len(),
        "skills": entries,
    });

    if let Some(parent) = a.out.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // mkstemp-in-out-dir + rename (atomic publish per INDEXING.md).
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp = a.out.parent().unwrap_or(Path::new(".")).join(format!(
        "tmp{}{}.tmp",
        std::process::id(),
        nanos
    ));
    if let Err(e) = std::fs::write(&tmp, jsonfmt::dumps_indent(&index, 1)) {
        eprintln!("{e}");
        return 1;
    }
    if let Err(e) = std::fs::rename(&tmp, &a.out) {
        eprintln!("{e}");
        let _ = std::fs::remove_file(&tmp);
        return 1;
    }

    let summary = json!({
        "indexed": index["count"].clone(),
        "skipped_no_skill_md": skipped,
        "out": a.out.to_string_lossy(),
    });
    println!("{}", jsonfmt::dumps_indent_ascii(&summary, 1));
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ciel-dev-reg-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn truthy_and_str_or() {
        assert!(!truthy(None));
        assert!(!truthy(Some(&json!(0))));
        assert!(!truthy(Some(&json!(""))));
        assert!(!truthy(Some(&json!([]))));
        assert!(truthy(Some(&json!("x"))));
        assert_eq!(str_or(Some(&json!("1.2"))).as_deref(), Some("1.2"));
        assert_eq!(str_or(Some(&json!(2))).as_deref(), Some("2"));
        assert_eq!(str_or(Some(&json!(true))).as_deref(), Some("True"));
        assert_eq!(str_or(Some(&json!(0))), None); // falsy filtered out
    }

    #[test]
    fn checksum_is_stable_and_prefix() {
        let dir = tmpdir("sum");
        std::fs::write(dir.join("b.txt"), "beta").unwrap();
        std::fs::create_dir(dir.join("sub")).unwrap();
        std::fs::write(dir.join("sub").join("a.txt"), "alpha").unwrap();
        let c1 = checksum(&dir);
        let c2 = checksum(&dir);
        assert_eq!(c1, c2);
        assert!(c1.starts_with("sha256:"));
        assert_eq!(c1.len(), 7 + 64);
        // contents change → checksum changes
        std::fs::write(dir.join("b.txt"), "changed").unwrap();
        assert_ne!(checksum(&dir), c1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn build_entry_merges_sidecar() {
        let dir = tmpdir("entry");
        let skill = dir.join("myskill");
        std::fs::create_dir_all(&skill).unwrap();
        std::fs::write(
            skill.join("SKILL.md"),
            "---\nname: myskill\ndescription: does things\nlicense: MIT\n---\nbody\n",
        )
        .unwrap();
        std::fs::write(
            skill.join("ciel.yaml"),
            "version: 1.2.3\ntags: [a, b]\nstate: draft\ntriggers:\n  - pattern: \"x.*\"\n",
        )
        .unwrap();
        let e = build_entry(&skill).unwrap();
        assert_eq!(e["id"], "myskill");
        assert_eq!(e["version"], "1.2.3");
        assert_eq!(e["description"], "does things");
        assert_eq!(e["license"], "MIT");
        assert_eq!(e["state"], "draft");
        assert_eq!(e["tags"], json!(["a", "b"]));
        assert_eq!(e["triggers"], json!(["x.*"]));
        assert_eq!(e["source"], json!({"tier": 0, "origin": "local"}));
        assert!(e["install_path"].as_str().unwrap().ends_with("myskill/"));
        assert!(e["checksum"].as_str().unwrap().starts_with("sha256:"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn build_entry_none_without_skill_md() {
        let dir = tmpdir("noskill");
        let skill = dir.join("empty");
        std::fs::create_dir_all(&skill).unwrap();
        assert!(build_entry(&skill).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn yaml_to_json_scalars_and_maps() {
        let v: serde_yaml::Value =
            serde_yaml::from_str("a: 1\nb: [x, y]\nc:\n  d: true\n").unwrap();
        let j = yaml_to_json(v);
        assert_eq!(j["a"], 1);
        assert_eq!(j["b"], json!(["x", "y"]));
        assert_eq!(j["c"]["d"], true);
        assert_eq!(
            yaml_to_json(serde_yaml::from_str("~").unwrap()),
            Value::Null
        );
    }
}
