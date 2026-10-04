//! `lint-fix` — port of `scripts/lint-fix.py`. Fixes MD040 (fenced code
//! block language) and MD060 (table pipe spacing) across `ROOT/**/*.md`
//! plus `ROOT/../README.md`.

use crate::py;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

fn sniff_language(block: &str) -> &'static str {
    let b = block.trim();
    let first = b.lines().next().unwrap_or("");
    const BOX_CHARS: &[char] = &[
        '→', '←', '↓', '─', '│', '┌', '└', '┐', '┘', '┬', '┴', '├', '┤',
    ];
    if b.chars().any(|c| BOX_CHARS.contains(&c)) {
        return "text";
    }
    if first.contains("```") {
        return "text";
    }
    if first.starts_with("$ ") || first.starts_with("#!/") || b.contains("\n$ ") {
        return "bash";
    }
    static SQL_RE: OnceLock<Regex> = OnceLock::new();
    let sql_re = SQL_RE
        .get_or_init(|| Regex::new(r"(?i)\b(SELECT|CREATE TABLE|UPDATE|INSERT|DROP)\s").unwrap());
    if sql_re.is_match(b) {
        return "sql";
    }
    if (first.trim_start().starts_with('{') || first.trim_start().starts_with('['))
        && (b.trim_end().ends_with('}') || b.trim_end().ends_with(']'))
    {
        // Only call it JSON if it parses; otherwise text.
        return if serde_json::from_str::<serde_json::Value>(b).is_ok() {
            "json"
        } else {
            "text"
        };
    }
    // YAML heuristic: >=2 top-level block mapping keys AND no prose markers.
    static YAML_KEY_RE: OnceLock<Regex> = OnceLock::new();
    let yaml_key_re =
        YAML_KEY_RE.get_or_init(|| Regex::new(r"(?m)^[A-Za-z_][A-Za-z0-9_]*:\s").unwrap());
    static PROSE_RE: OnceLock<Regex> = OnceLock::new();
    let prose_re = PROSE_RE.get_or_init(|| Regex::new(r"[.?!]\s+[A-Z]").unwrap());
    let yaml_keys = yaml_key_re.find_iter(b).count();
    let prose_sentences = prose_re.find_iter(b).count();
    if yaml_keys >= 2 && prose_sentences == 0 {
        return "yaml";
    }
    "text"
}

fn fence_bare() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^(\s*)```\s*$").unwrap())
}

fn fence_lang() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^(\s*)```\s*\S+").unwrap())
}

/// `fix_md040` — add a language tag to bare ``` opening fences.
fn fix_md040(text: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if let Some(m) = fence_bare().captures(line) {
            // Opening fence — collect body until the closing bare fence.
            let indent = m.get(1).unwrap().as_str().to_string();
            let mut body_lines: Vec<&str> = Vec::new();
            let mut j = i + 1;
            while j < lines.len() {
                if fence_bare().is_match(lines[j]) {
                    break;
                }
                body_lines.push(lines[j]);
                j += 1;
            }
            let body = body_lines.join("\n");
            let lang = sniff_language(&body);
            out.push(format!("{indent}```{lang}"));
            out.extend(body_lines.iter().map(|s| s.to_string()));
            if j < lines.len() {
                out.push(lines[j].to_string());
            }
            i = j + 1;
            continue;
        }
        if fence_lang().is_match(line) {
            // Already labelled — pass the block through untouched.
            out.push(line.to_string());
            i += 1;
            while i < lines.len() {
                out.push(lines[i].to_string());
                if fence_bare().is_match(lines[i]) {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }
        out.push(line.to_string());
        i += 1;
    }
    out.join("\n")
}

fn table_row_start() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^\s*\|").unwrap())
}

fn leading_ws() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^(\s*)").unwrap())
}

fn sep_cell() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^[\s:\-]+$").unwrap())
}

fn code_fence() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^\s*```").unwrap())
}

/// `fix_md060` — normalize markdown tables to `| cell | cell |` style.
fn fix_md060(text: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    let mut in_code = false;
    while i < lines.len() {
        let line = lines[i];
        if code_fence().is_match(line) {
            in_code = !in_code;
            out.push(line.to_string());
            i += 1;
            continue;
        }
        if in_code {
            out.push(line.to_string());
            i += 1;
            continue;
        }
        if table_row_start().is_match(line) && line.matches('|').count() >= 2 {
            // Collect the contiguous table block.
            let mut block: Vec<&str> = Vec::new();
            while i < lines.len()
                && table_row_start().is_match(lines[i])
                && lines[i].matches('|').count() >= 2
            {
                block.push(lines[i]);
                i += 1;
            }
            let leading = leading_ws()
                .captures(block[0])
                .and_then(|m| m.get(1))
                .map(|m| m.as_str())
                .unwrap_or("")
                .to_string();
            let mut norm_rows: Vec<String> = Vec::new();
            for row in &block {
                let mut stripped = row.trim().to_string();
                if !stripped.starts_with('|') {
                    stripped = format!("|{stripped}");
                }
                if !stripped.ends_with('|') {
                    stripped.push('|');
                }
                let cells: Vec<&str> = stripped.split('|').collect();
                let inner = &cells[1..cells.len().saturating_sub(1)];
                let mut norm_cells: Vec<String> = Vec::new();
                for c in inner {
                    let c = c.trim();
                    if sep_cell().is_match(c) && !c.is_empty() {
                        // Alignment separator: keep colon sides, pad hyphens.
                        let left = c.starts_with(':');
                        let right = c.ends_with(':');
                        let mut core: String =
                            c.replace(':', "").chars().filter(|ch| *ch == '-').collect();
                        if core.len() < 3 {
                            core = "---".to_string();
                        }
                        norm_cells.push(format!(
                            "{}{}{}",
                            if left { ":" } else { "" },
                            core,
                            if right { ":" } else { "" }
                        ));
                    } else {
                        norm_cells.push(c.to_string());
                    }
                }
                norm_rows.push(format!("{leading}| {} |", norm_cells.join(" | ")));
            }
            out.extend(norm_rows);
            continue;
        }
        out.push(line.to_string());
        i += 1;
    }
    out.join("\n")
}

/// `ROOT.rglob("*.md")` — depth-first, no symlink recursion.
fn rglob_md(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(root) else {
        return;
    };
    let entries: Vec<_> = rd.flatten().collect();
    for e in &entries {
        let p = e.path();
        if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("md") {
            out.push(p);
        }
    }
    for e in &entries {
        let p = e.path();
        // is_dir() follows symlinks; check file_type for a real dir.
        if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            rglob_md(&p, out);
        }
    }
}

pub fn main_(args: &[String]) -> i32 {
    let mut root: Option<PathBuf> = None;
    for a in args {
        match a.as_str() {
            "-h" | "--help" => {
                println!("usage: ciel-dev lint-fix [ROOT]");
                return 0;
            }
            s => root = Some(PathBuf::from(s)),
        }
    }
    let root = root.unwrap_or_else(|| py::repo_root().join("ciel.skill"));
    let mut targets = Vec::new();
    rglob_md(&root, &mut targets);
    if let Some(parent) = root.parent() {
        targets.push(parent.join("README.md"));
    }
    let mut fixed = 0u64;
    for path in targets {
        let Ok(original) = std::fs::read_to_string(&path) else {
            continue;
        };
        let new = fix_md060(&fix_md040(&original));
        if new != original && std::fs::write(&path, &new).is_ok() {
            fixed += 1;
        }
    }
    println!("fixed {fixed} files");
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniff_bash_sql_json_yaml_text() {
        assert_eq!(sniff_language("$ ls -la\n"), "bash");
        assert_eq!(sniff_language("#!/bin/bash\nx"), "bash");
        assert_eq!(sniff_language("first\n$ echo hi"), "bash");
        assert_eq!(sniff_language("SELECT * FROM t;"), "sql");
        assert_eq!(sniff_language("create table x (a int)"), "sql");
        assert_eq!(sniff_language("{\"a\": 1}"), "json");
        assert_eq!(sniff_language("[1, 2]"), "json");
        // JSON-looking but unparseable → text
        assert_eq!(sniff_language("{not json}"), "text");
        assert_eq!(sniff_language("key: 1\nother: 2"), "yaml");
        assert_eq!(sniff_language("just prose here"), "text");
        // box-drawing chars → text before anything else
        assert_eq!(sniff_language("$ x → y"), "text");
    }

    #[test]
    fn fix_md040_tags_bare_fences() {
        let out = fix_md040("# t\n```\n$ echo hi\n```\ndone\n");
        assert_eq!(out, "# t\n```bash\n$ echo hi\n```\ndone\n");
    }

    #[test]
    fn fix_md040_keeps_labelled_and_unclosed() {
        let out = fix_md040("```rust\ncode\n```\n");
        assert_eq!(out, "```rust\ncode\n```\n");
        // unclosed bare fence still gets tagged; body runs to EOF
        let out = fix_md040("```\n$ tail\n");
        assert_eq!(out, "```bash\n$ tail\n");
    }

    #[test]
    fn fix_md060_normalizes_tables() {
        let out = fix_md060("|a|b|\n|---|---|\n| 1|  2 |\n");
        assert_eq!(out, "| a | b |\n| --- | --- |\n| 1 | 2 |\n");
    }

    #[test]
    fn fix_md060_skips_code_blocks() {
        let src = "```\n|a|b|\n```\n|x|y|\n";
        let out = fix_md060(src);
        assert_eq!(out, "```\n|a|b|\n```\n| x | y |\n");
    }

    #[test]
    fn fix_md060_separator_alignment() {
        let out = fix_md060("|a|b|\n|:-|-:|\n");
        assert_eq!(out, "| a | b |\n| :--- | ---: |\n");
    }
}
