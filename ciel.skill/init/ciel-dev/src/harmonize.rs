//! `harmonize-skills` — port of `scripts/harmonize_skills.py`. Normalizes
//! `runtimes` and enriches `tags` in the `ciel.yaml` sidecar when present
//! (else in SKILL.md directly), and purges placeholders from the SKILL.md
//! body. The Python `SKILLS_DIR` is the literal `~\Ciel\skills` — no shell
//! expansion — so bare invocation is a no-op on POSIX just like upstream.

use regex::Regex;
use std::path::{Path, PathBuf};

const DOMAIN_TAGS: &[(&str, &[&str])] = &[
    (
        "web",
        &[
            "web",
            "frontend",
            "html",
            "css",
            "js",
            "react",
            "nextjs",
            "nuxt",
            "gsap",
            "remotion",
            "seo",
            "modern-js",
            "laravel",
        ],
    ),
    (
        "systems",
        &[
            "systems",
            "kernel",
            "os",
            "container",
            "docker",
            "rust",
            "cpp",
            "go",
            "jvm",
            "java",
            "dotnet",
            "database",
            "migrations",
            "perl",
        ],
    ),
    (
        "ai",
        &[
            "ai",
            "agent",
            "llm",
            "neural",
            "prompt",
            "eval",
            "mcp",
            "intelligence",
            "autonomous",
            "orchestration",
            "swarm",
        ],
    ),
    (
        "ops",
        &[
            "ops",
            "deployment",
            "ci",
            "cd",
            "automation",
            "logistics",
            "billing",
            "monitoring",
            "jira",
            "google-workspace",
            "content-distribution",
            "email",
        ],
    ),
    (
        "mobile",
        &[
            "mobile",
            "android",
            "kmp",
            "ios",
            "swift",
            "flutter",
            "compose-multiplatform",
        ],
    ),
    (
        "data",
        &[
            "data",
            "analytical",
            "ml",
            "database",
            "retrieval",
            "context",
            "knowledge",
            "memory",
        ],
    ),
    (
        "security",
        &[
            "security",
            "vulnerability",
            "safety",
            "compliance",
            "audit",
            "crypto",
            "guard",
        ],
    ),
    (
        "quality",
        &[
            "quality",
            "test",
            "verification",
            "debugging",
            "linter",
            "review",
            "eval-harness",
        ],
    ),
    (
        "strategy",
        &[
            "strategy",
            "planning",
            "research",
            "brainstorming",
            "decision",
            "identity",
            "brand",
            "compact",
            "flow",
        ],
    ),
    (
        "design",
        &[
            "design",
            "ui",
            "ux",
            "presentation",
            "animation",
            "asset",
            "gsap",
            "remotion",
        ],
    ),
];

/// `get_domain` — first domain whose keyword hits the skill name, else the
/// content; "strategy" fallback.
fn get_domain(skill_name: &str, content: &str) -> &'static str {
    let name_l = skill_name.to_lowercase();
    let content_l = content.to_lowercase();
    for (domain, keywords) in DOMAIN_TAGS {
        if keywords.iter().any(|kw| name_l.contains(kw)) {
            return domain;
        }
        if keywords.iter().any(|kw| content_l.contains(kw)) {
            return domain;
        }
    }
    "strategy"
}

const RUNTIMES_REPL: &str =
    "runtimes: [\"claude_code\", \"gemini_cli\", \"windsurf\", \"generic\"]";

/// `re.sub(r'runtimes:.*?(?=\n\w)', REPL, s, DOTALL)` — the regex crate has
/// no lookahead, so scan for the first `\n` followed by a word char after
/// each `runtimes:` occurrence; the newline is not consumed.
fn normalize_runtimes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    loop {
        let Some(rel) = rest.find("runtimes:") else {
            out.push_str(rest);
            break;
        };
        let start = rel;
        let scan_from = start + "runtimes:".len();
        let bytes = rest.as_bytes();
        let mut end = None;
        let mut i = scan_from;
        while i + 1 < bytes.len() {
            if bytes[i] == b'\n' {
                let c = bytes[i + 1] as char;
                if c.is_alphanumeric() || c == '_' {
                    end = Some(i);
                    break;
                }
            }
            i += 1;
        }
        match end {
            Some(e) => {
                out.push_str(&rest[..start]);
                out.push_str(RUNTIMES_REPL);
                rest = &rest[e..];
            }
            None => {
                out.push_str(rest);
                break;
            }
        }
    }
    out
}

fn harmonize_skill(file_path: &Path) {
    let skill_dir = file_path.parent().unwrap_or(Path::new("."));
    // Ciel extension fields live in the ciel.yaml sidecar when present.
    let sidecar = skill_dir.join("ciel.yaml");
    let meta_path = if sidecar.is_file() {
        sidecar.clone()
    } else {
        file_path.to_path_buf()
    };
    let Ok(mut meta) = std::fs::read_to_string(&meta_path) else {
        return;
    };

    // 1. Runtime Normalization (H1)
    meta = normalize_runtimes(&meta);

    // 2. Domain Tag Enrichment (H2)
    let skill_name = skill_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let domain = get_domain(&skill_name, &meta);

    let tag_re = Regex::new(r"tags: \[(.*?)\]").unwrap();
    let tag_match = tag_re.captures(&meta).map(|m| {
        (
            m.get(0).unwrap().as_str().to_string(),
            m.get(1).unwrap().as_str().to_string(),
        )
    });
    if let Some((whole, inner)) = tag_match {
        let mut tags: Vec<String> = inner
            .split(',')
            .map(|t| t.trim().trim_matches('"').trim_matches('\'').to_string())
            .collect();
        let dom_tag = format!("domain:{domain}");
        if !tags.iter().any(|t| t == &dom_tag) {
            tags.push(dom_tag);
        }
        tags.retain(|t| t != "ciel" && t != "harmonized");
        // sorted(set(tags)) — dedupe + codepoint order
        let mut sorted: Vec<String> = tags.into_iter().collect();
        sorted.sort();
        sorted.dedup();
        let mut all = vec!["ciel".to_string(), "harmonized".to_string()];
        all.extend(sorted);
        let new_tags = format!(
            "tags: [{}]",
            all.iter()
                .map(|t| format!("\"{t}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
        meta = meta.replace(&whole, &new_tags);
    }

    let mut content;
    if meta_path != file_path {
        let _ = std::fs::write(&meta_path, &meta);
        content = std::fs::read_to_string(file_path).unwrap_or_default();
    } else {
        content = meta;
    }

    // 3. Placeholder Purge (M6) — SKILL.md body only.
    content = content.replace(
        "TODO",
        "Refine implementation logic to align with Ciel 1.0 standards.",
    );
    content = content.replace(
        "FIXME",
        "Resolve architectural debt and ensure deterministic behavior.",
    );
    let bracket_re = Regex::new(r"\[\.\.\.\]").unwrap();
    content = bracket_re
        .replace_all(
            &content,
            "[Comprehensive implementation details following Ciel spec]",
        )
        .into_owned();
    let dots_re = Regex::new(r"(?m)^\s*\.\.\.\s*$").unwrap();
    content = dots_re
        .replace_all(
            &content,
            "    [Continuous integration and verification steps]",
        )
        .into_owned();

    let _ = std::fs::write(file_path, &content);
}

pub fn main_(args: &[String]) -> i32 {
    let mut root = PathBuf::from(r"~\Ciel\skills");
    for a in args {
        match a.as_str() {
            "-h" | "--help" => {
                println!("usage: ciel-dev harmonize-skills [SKILLS_DIR]");
                return 0;
            }
            s => root = PathBuf::from(s),
        }
    }
    // os.walk: top-down, yields dirs containing SKILL.md in scandir order.
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut has_skill = false;
        let mut subdirs = Vec::new();
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                subdirs.push(p);
            } else if e.file_name() == "SKILL.md" {
                has_skill = true;
            }
        }
        if has_skill {
            harmonize_skill(&dir.join("SKILL.md"));
        }
        for d in subdirs.into_iter().rev() {
            stack.push(d);
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_domain_matches_name_then_content_then_default() {
        assert_eq!(get_domain("web-wizard", "nothing"), "web");
        assert_eq!(get_domain("zzz", "about docker containers"), "systems");
        // domains scan in declaration order — web's content hit precedes
        // systems' name hit even though "rust" is inside "rusty".
        assert_eq!(get_domain("rusty", "web stuff"), "web");
        assert_eq!(get_domain("zzz", "nothing"), "strategy");
    }

    #[test]
    fn normalize_runtimes_inline_and_block() {
        let out = normalize_runtimes("runtimes: [\"old\"]\ntags: [x]\n");
        assert_eq!(
            out,
            "runtimes: [\"claude_code\", \"gemini_cli\", \"windsurf\", \"generic\"]\ntags: [x]\n"
        );
        // block-style list ends at the next unindented key
        let out = normalize_runtimes("runtimes:\n  - old\n  - x\ntags: [y]\n");
        assert_eq!(
            out,
            "runtimes: [\"claude_code\", \"gemini_cli\", \"windsurf\", \"generic\"]\ntags: [y]\n"
        );
        // no following `\n<word>` → untouched (matches the lookahead failing)
        let tail = "runtimes: [\"old\"]";
        assert_eq!(normalize_runtimes(tail), tail);
        // trailing blank line then word char still counts
        let out = normalize_runtimes("runtimes: [x]\n\nname: s\n");
        assert!(out.contains("windsurf"));
    }

    #[test]
    fn harmonize_skill_sidecar_and_body() {
        let dir = std::env::temp_dir().join(format!(
            "ciel-dev-harm-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let skill = dir.join("web-wizard");
        std::fs::create_dir_all(&skill).unwrap();
        std::fs::write(
            skill.join("SKILL.md"),
            "---\nname: web-wizard\ndescription: web things\n---\n\nTODO: fill in\n",
        )
        .unwrap();
        std::fs::write(
            skill.join("ciel.yaml"),
            "runtimes: [\"old\"]\ntags: [\"web\"]\n",
        )
        .unwrap();
        harmonize_skill(&skill.join("SKILL.md"));
        let meta = std::fs::read_to_string(skill.join("ciel.yaml")).unwrap();
        assert!(meta.contains("\"generic\""));
        assert!(meta.contains("\"domain:web\""));
        assert!(meta.contains("\"ciel\""));
        let body = std::fs::read_to_string(skill.join("SKILL.md")).unwrap();
        assert!(body.contains("Refine implementation logic"));
        assert!(body.contains("name: web-wizard"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn harmonize_skill_no_sidecar_edits_skill_md() {
        let dir = std::env::temp_dir().join(format!(
            "ciel-dev-harm2-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let skill = dir.join("db-helper");
        std::fs::create_dir_all(&skill).unwrap();
        std::fs::write(
            skill.join("SKILL.md"),
            "runtimes: [\"old\"]\ntags: [\"x\"]\n\nFIXME: later\n",
        )
        .unwrap();
        harmonize_skill(&skill.join("SKILL.md"));
        let content = std::fs::read_to_string(skill.join("SKILL.md")).unwrap();
        assert!(content.contains("\"generic\""));
        assert!(content.contains("Resolve architectural debt"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
