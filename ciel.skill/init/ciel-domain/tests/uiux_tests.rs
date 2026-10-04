//! Unit tests for the ui-ux-pro-max ports: BM25/tokenization, query
//! normalization, the reasoning contract, `format_output`, fixture-catalog
//! search, the design-system generator, and the data validator (against the
//! real repository catalog).

use ciel_domain::uiux::cli::format_output;
use ciel_domain::uiux::core::{normalize, search, search_stack, Bm25};
use ciel_domain::uiux::data::Catalog;
use ciel_domain::uiux::design_system::generate_design_system;
use ciel_domain::uiux::reasoning::{apply_decision_rules, parse_decision_rules};
use ciel_domain::uiux::validate::validate;
use serde_json::{json, Value};
use std::io::Write;

fn tmp_dir(name: &str) -> std::path::PathBuf {
    let dir =
        std::env::temp_dir().join(format!("ciel-domain-uiux-{}-{}", std::process::id(), name));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// ---------- normalize / tokenize ----------

#[test]
fn normalize_synonyms() {
    assert_eq!(normalize("a11y audit"), "accessibility audit");
    assert_eq!(normalize("darkmode"), "dark");
    assert_eq!(normalize("the colour palette"), "the color palette");
    // Word-boundary: "anavar" must not expand the "nav" synonym.
    assert!(!normalize("anavar").contains("navigation"));
}

#[test]
fn bm25_tokenize() {
    let bm = Bm25::new();
    let toks = bm.tokenize("The quick, brown fox! (jumped)");
    assert_eq!(toks, vec!["quick", "brown", "fox", "jumped"]);
    // Single-char and stopword filtering.
    assert!(bm.tokenize("a b c the and").is_empty());
}

#[test]
fn bm25_fit_and_score() {
    let mut bm = Bm25::new();
    bm.fit(&[
        "glassmorphism frosted glass blur".to_string(),
        "brutalist heavy borders monochrome".to_string(),
        "minimalist whitespace clean".to_string(),
    ]);
    let scores = bm.score("glass blur");
    assert_eq!(scores.len(), 3);
    assert_eq!(scores[0].0, 0, "top doc should be glassmorphism");
    assert!(scores[0].1 > scores[1].1);
    // Unmatched query scores zero for all docs.
    assert!(bm.score("zzz").iter().all(|(_, s)| *s == 0.0));
}

#[test]
fn bm25_empty_corpus() {
    let mut bm = Bm25::new();
    bm.fit(&[]);
    assert!(bm.score("anything").is_empty());
}

// ---------- reasoning contract ----------

#[test]
fn decision_rules_valid() {
    let rules = parse_decision_rules(
        r#"{"must_have":["style:glassmorphism","constraint:contrast-text"],"if_dashboard":["pattern:kpi-dashboard","mode:dark"]}"#,
    )
    .unwrap();
    assert_eq!(rules.len(), 2);
}

#[test]
fn decision_rules_reject_duplicate_keys() {
    let err =
        parse_decision_rules(r#"{"must_have":["style:a"],"must_have":["style:b"]}"#).unwrap_err();
    assert!(err.contains("duplicate"), "err: {}", err);
}

#[test]
fn decision_rules_reject_bad_prefix() {
    assert!(parse_decision_rules(r#"{"must_have":["hack:evil"]}"#).is_err());
    // Unknown condition names are also rejected.
    assert!(parse_decision_rules(r#"{"if_dark":["mode:dark"]}"#).is_err());
}

#[test]
fn decision_rules_reject_bad_mode() {
    assert!(parse_decision_rules(r#"{"must_have":["mode:purple"]}"#).is_err());
    assert!(parse_decision_rules(r#"{"must_have":["mode:dark"]}"#).is_ok());
}

#[test]
fn decision_rules_empty_is_empty_map() {
    assert!(parse_decision_rules("").unwrap().is_empty());
    assert!(parse_decision_rules("{}").unwrap().is_empty());
}

#[test]
fn apply_rules_must_have_and_conditionals() {
    let rules = parse_decision_rules(
        r#"{"must_have":["style:flat-design"],"if_dashboard":["constraint:keyboard","pattern:kpi-dash"],"if_luxury":["mode:dark"]}"#,
    )
    .unwrap();
    let applied = apply_decision_rules(&rules, "analytics dashboard");
    assert_eq!(applied.style_ids, vec!["flat-design"]);
    assert_eq!(applied.constraints, vec!["keyboard"]);
    assert_eq!(applied.pattern.as_deref(), Some("kpi-dash"));
    assert_eq!(applied.mode, None); // "if_luxury" not triggered by this query
    assert_eq!(applied.activated.len(), 2);
}

#[test]
fn apply_rules_later_mode_wins() {
    let rules =
        parse_decision_rules(r#"{"must_have":["mode:light"],"if_mobile":["mode:dark"]}"#).unwrap();
    let applied = apply_decision_rules(&rules, "mobile app");
    assert_eq!(applied.mode.as_deref(), Some("dark"));
}

// ---------- format_output ----------

#[test]
fn format_output_error() {
    let v = json!({"error": "boom", "domain": "style"});
    assert_eq!(format_output(&v, false), "Error: boom");
}

#[test]
fn format_output_truncates_long_fields() {
    let long = "x".repeat(400);
    let v = json!({
        "domain": "style", "query": "q", "file": "styles.csv", "count": 1,
        "results": [{"Big": long, "Small": "ok"}],
    });
    let out = format_output(&v, false);
    assert!(out.contains(&format!("{}...", "x".repeat(300))));
    assert!(!out.contains(&"x".repeat(301)));
    let full = format_output(&v, true);
    assert!(full.contains(&"x".repeat(400)));
}

#[test]
fn format_output_no_matches_with_suggestions() {
    let v = json!({
        "domain": "color", "query": "zzz", "file": "colors.csv", "count": 0,
        "suggestions": ["red", "blue"],
    });
    let out = format_output(&v, false);
    assert!(out.contains("No matches"));
    assert!(out.contains("red, blue"));
}

#[test]
fn format_output_stack_header() {
    let v = json!({
        "stack": "react", "query": "q", "file": "stacks/react.csv", "count": 1,
        "results": [{"Guideline": "g"}],
    });
    assert!(format_output(&v, false).contains("Stack Guidelines"));
}

// ---------- fixture catalog ----------

#[test]
fn catalog_fixture_search() {
    let dir = tmp_dir("fixture");
    let mut f = std::fs::File::create(dir.join("styles.csv")).unwrap();
    writeln!(f, "No,Style Category,Style ID,Keywords,Best For").unwrap();
    writeln!(
        f,
        "1,Zzz Teststyle,zzz-teststyle,uniqueterm frosted,testing"
    )
    .unwrap();
    drop(f);
    let catalog = Catalog::at(dir.clone());
    let table = catalog.table("styles.csv").unwrap();
    assert_eq!(table.rows.len(), 1);
    // Second read must hit the cache path.
    let rows = catalog.rows("styles.csv").unwrap();
    assert_eq!(rows[0]["Style ID"], "zzz-teststyle");
}

#[test]
fn catalog_missing_file() {
    let dir = tmp_dir("empty");
    let catalog = Catalog::at(dir);
    assert!(catalog.table("styles.csv").is_err());
    assert!(catalog.rows_or_empty("styles.csv").is_empty());
}

// ---------- real-catalog behavior (repo data is present) ----------

#[test]
fn real_catalog_exact_style_search() {
    let catalog = Catalog::new();
    if !catalog.path("styles.csv").exists() {
        eprintln!("skipping: ui-ux data dir not found");
        return;
    }
    let result = search(&catalog, "glassmorphism", None, 3, false);
    assert_eq!(result["domain"], Value::String("style".into()));
    assert_eq!(result["count"].as_i64().unwrap(), 1);
    assert_eq!(result["auto_detected"], Value::Bool(true));
    let rows = result["results"].as_array().unwrap();
    assert_eq!(rows[0]["Style ID"], Value::String("glassmorphism".into()));
}

#[test]
fn real_catalog_stack_search() {
    let catalog = Catalog::new();
    if !catalog.path("stacks/react.csv").exists() {
        eprintln!("skipping: ui-ux data dir not found");
        return;
    }
    let result = search_stack(&catalog, "useEffect cleanup", "react", 3, false);
    assert!(result["count"].as_i64().unwrap() >= 1);
    assert_eq!(result["stack"], Value::String("react".into()));
}

#[test]
fn real_catalog_bad_max_results() {
    let catalog = Catalog::new();
    let r = search(&catalog, "q", None, 25, false);
    assert!(r.get("error").is_some());
}

#[test]
fn real_catalog_validate_is_clean() {
    let catalog = Catalog::new();
    if !catalog.path("data-provenance.json").exists() {
        eprintln!("skipping: ui-ux data dir not found");
        return;
    }
    let problems = validate(&catalog);
    assert!(problems.is_empty(), "validator reported: {:?}", problems);
}

#[test]
fn validate_missing_files_reports() {
    let dir = tmp_dir("broken");
    let catalog = Catalog::at(dir);
    let problems = validate(&catalog);
    assert!(!problems.is_empty());
    assert!(problems.iter().any(|p| p.contains("missing file")));
}

// ---------- design system ----------

#[test]
fn design_system_structure() {
    let catalog = Catalog::new();
    if !catalog.path("ui-reasoning.csv").exists() {
        eprintln!("skipping: ui-ux data dir not found");
        return;
    }
    let gen = generate_design_system(
        &catalog,
        "saas dashboard",
        None,
        "ascii",
        false,
        None,
        None,
        None,
        None,
        None,
        false,
    );
    let ds = &gen.design_system;
    for key in [
        "project_name",
        "category",
        "pattern",
        "style",
        "colors",
        "typography",
        "dials",
        "severity",
    ] {
        assert!(ds.get(key).is_some(), "missing key {}", key);
    }
    assert!(gen.text.contains("TARGET:"));
    assert!(gen.persistence.is_none());
}

#[test]
fn design_system_persist_and_skip() {
    let catalog = Catalog::new();
    if !catalog.path("ui-reasoning.csv").exists() {
        eprintln!("skipping: ui-ux data dir not found");
        return;
    }
    let dir = tmp_dir("persist");
    let out = dir.to_string_lossy().into_owned();
    let gen = generate_design_system(
        &catalog,
        "my cool app",
        None,
        "markdown",
        true,
        Some("settings"),
        Some(&out),
        None,
        None,
        None,
        false,
    );
    let p = gen.persistence.unwrap();
    assert_eq!(p["status"], Value::String("success".into()));
    let master = dir.join("design-system/my-cool-app/MASTER.md");
    let page = dir.join("design-system/my-cool-app/pages/settings.md");
    assert!(master.exists());
    assert!(page.exists());
    let master_text = std::fs::read_to_string(&master).unwrap();
    assert!(master_text.contains("Design System Master File"));

    // Second run without --force → skipped_exists.
    let gen2 = generate_design_system(
        &catalog,
        "my cool app",
        None,
        "markdown",
        true,
        None,
        Some(&out),
        None,
        None,
        None,
        false,
    );
    assert_eq!(
        gen2.persistence.unwrap()["status"],
        Value::String("skipped_exists".into())
    );

    // With --force → rewritten, success.
    let gen3 = generate_design_system(
        &catalog,
        "my cool app",
        None,
        "markdown",
        true,
        None,
        Some(&out),
        None,
        None,
        None,
        true,
    );
    assert_eq!(
        gen3.persistence.unwrap()["status"],
        Value::String("success".into())
    );
}

#[test]
fn design_system_dials() {
    let catalog = Catalog::new();
    if !catalog.path("ui-reasoning.csv").exists() {
        eprintln!("skipping: ui-ux data dir not found");
        return;
    }
    let gen = generate_design_system(
        &catalog,
        "data dashboard",
        None,
        "ascii",
        false,
        None,
        None,
        Some(9),
        Some(8),
        Some(9),
        false,
    );
    let dials = &gen.design_system["dials"];
    assert_eq!(dials["variance"], serde_json::json!(9));
    assert_eq!(dials["motion"], serde_json::json!(8));
    assert_eq!(dials["density"], serde_json::json!(9));
}
