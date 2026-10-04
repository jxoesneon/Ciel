//! `fix-md-lint` — port of `scripts/fix_md_lint.py`. Walks a root for
//! `*.md` files (minus excluded dirs) and applies the MD012/MD009/MD022/
//! MD026/MD031/MD040/MD032/MD030/MD047 fixups in place.

use crate::py;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

fn heading_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^#+ ").unwrap())
}

fn trailing_punct_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"[:;,.]$").unwrap())
}

fn list_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^(\s*)(\d+\.|[\*\-\+])(\s+)").unwrap())
}

/// Python `str.splitlines()` on `\n`/`\r\n` (the only separators markdown
/// files realistically contain).
fn splitlines(content: &str) -> Vec<String> {
    content.lines().map(|l| l.to_string()).collect()
}

/// Port of `fix_markdown(content)`.
pub fn fix_markdown(content: &str) -> String {
    if content.trim().is_empty() {
        return content.to_string();
    }

    // MD012: No multiple blank lines
    let collapse_re = Regex::new(r"\n{3,}").unwrap();
    let content = collapse_re.replace_all(content, "\n\n").into_owned();

    // MD009: No trailing spaces
    let lines: Vec<String> = splitlines(&content)
        .iter()
        .map(|l| l.trim_end().to_string())
        .collect();

    // MD022: Blanks around headings / MD026: No trailing punctuation
    let mut temp_lines: Vec<String> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if heading_re().is_match(line) {
            // strip, then drop one trailing punctuation char
            let stripped = line.trim();
            let line = trailing_punct_re().replace(stripped, "").into_owned();

            if i > 0 && !temp_lines.is_empty() && !temp_lines.last().unwrap().trim().is_empty() {
                temp_lines.push(String::new());
            }
            temp_lines.push(line);
            if i < lines.len() - 1
                && !lines[i + 1].trim().is_empty()
                && !heading_re().is_match(&lines[i + 1])
            {
                temp_lines.push(String::new());
            }
        } else {
            temp_lines.push(line.clone());
        }
    }

    // MD031: Blanks around fences / MD040: fence language
    let lines = temp_lines;
    let mut temp_lines: Vec<String> = Vec::new();
    let mut in_code_block = false;
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("```") {
            if !in_code_block {
                if i > 0 && !temp_lines.is_empty() && !temp_lines.last().unwrap().trim().is_empty()
                {
                    temp_lines.push(String::new());
                }
                // MD040: default language if missing
                let line = if line.trim() == "```" {
                    "```text".to_string()
                } else {
                    line.clone()
                };
                temp_lines.push(line);
                in_code_block = true;
            } else {
                temp_lines.push(line.clone());
                in_code_block = false;
                if i < lines.len() - 1 && !lines[i + 1].trim().is_empty() {
                    temp_lines.push(String::new());
                }
            }
        } else {
            temp_lines.push(line.clone());
        }
    }

    // MD032: blanks around lists / MD030: spaces after list markers
    let lines = temp_lines;
    let mut final_lines: Vec<String> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if let Some(m) = list_re().captures(line) {
            // MD030: exactly one space after the marker
            let indent = m.get(1).unwrap().as_str();
            let marker = m.get(2).unwrap().as_str();
            let rest = line[m.get(0).unwrap().end()..].trim_start();
            let line = format!("{indent}{marker} {rest}");

            // MD032: blank line above list
            let prev_is_list = i > 0 && list_re().is_match(&lines[i - 1]);
            if !prev_is_list
                && i > 0
                && !final_lines.is_empty()
                && !final_lines.last().unwrap().trim().is_empty()
            {
                final_lines.push(String::new());
            }
            final_lines.push(line);

            // MD032: blank line below list
            if i < lines.len() - 1 {
                let next_is_list = list_re().is_match(&lines[i + 1]);
                if !next_is_list
                    && !lines[i + 1].trim().is_empty()
                    && !lines[i + 1].starts_with("```")
                {
                    final_lines.push(String::new());
                }
            }
        } else {
            final_lines.push(line.clone());
        }
    }

    // MD047: single trailing newline
    let fixed = final_lines.join("\n");
    format!("{}\n", fixed.trim())
}

fn should_exclude(name: &str) -> bool {
    matches!(
        name,
        ".git" | "dist" | "node_modules" | ".ciel" | ".attic" | "archive"
    )
}

fn walk_md(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(root) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if p.is_dir() {
            if !should_exclude(&name) {
                walk_md(&p, out);
            }
        } else if name.ends_with(".md") {
            out.push(p);
        }
    }
}

pub fn main_(args: &[String]) -> i32 {
    // Python root is the literal `~\Ciel` (no expansion) — a bare run is a
    // no-op on POSIX exactly like the source script.
    let mut root = PathBuf::from(r"~\Ciel");
    for a in args {
        match a.as_str() {
            "-h" | "--help" => {
                println!("usage: ciel-dev fix-md-lint [ROOT]");
                return 0;
            }
            s => root = PathBuf::from(s),
        }
    }
    let mut files = Vec::new();
    walk_md(&root, &mut files);
    let mut fixed_count = 0u64;
    for path in files {
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                let fixed = fix_markdown(&content);
                if content != fixed && std::fs::write(&path, &fixed).is_ok() {
                    println!("Fixed {}", py::relpath(&path, &root));
                    fixed_count += 1;
                }
            }
            Err(e) => println!("Error fixing {}: {e}", path.display()),
        }
    }
    println!("\nTotal files fixed: {fixed_count}");
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    // cases lifted from tests/test_md_fixer.py
    #[test]
    fn md047_trailing_newline() {
        let fixed = fix_markdown("## Heading\nNo newline");
        assert!(fixed.ends_with('\n'));
        assert_eq!(fixed.matches('\n').count(), 3);
    }

    #[test]
    fn md012_multiple_blanks() {
        let fixed = fix_markdown("Line 1\n\n\nLine 2");
        assert!(fixed.contains("Line 1\n\nLine 2"));
    }

    #[test]
    fn md026_trailing_punctuation() {
        let fixed = fix_markdown("## Heading:");
        assert_eq!(fixed.trim(), "## Heading");
    }

    #[test]
    fn md022_blanks_around_headings() {
        let fixed = fix_markdown("para\n## Head\nbody");
        assert_eq!(fixed, "para\n\n## Head\n\nbody\n");
    }

    #[test]
    fn md031_md040_fences() {
        let fixed = fix_markdown("text\n```\ncode\n```\nafter");
        assert_eq!(fixed, "text\n\n```text\ncode\n```\n\nafter\n");
    }

    #[test]
    fn md032_md030_lists() {
        let fixed = fix_markdown("para\n-   a\n-  b\nnext");
        assert_eq!(fixed, "para\n\n- a\n- b\n\nnext\n");
    }

    #[test]
    fn md009_trailing_spaces() {
        let fixed = fix_markdown("line with spaces   \nnext");
        assert_eq!(fixed, "line with spaces\nnext\n");
    }

    #[test]
    fn empty_input_unchanged() {
        assert_eq!(fix_markdown(""), "");
        assert_eq!(fix_markdown("   \n\n"), "   \n\n");
    }
}
