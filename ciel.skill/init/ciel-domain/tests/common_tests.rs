//! Unit tests for the shared Python-compat helpers: CSV reader, difflib
//! ratio, argparse-style CLI parser, and `py` value helpers.

use ciel_domain::common::{cli, csvio, difflib, jsonfmt, py, rng::Rng};
use serde_json::json;
use std::io::Write;

fn tmp_path(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("ciel-domain-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

// ---------- csvio ----------

#[test]
fn csv_basic_rows() {
    let (headers, rows) = csvio::parse_csv("a,b\n1,2\n3,4\n");
    assert_eq!(headers, vec!["a", "b"]);
    assert_eq!(rows, vec![vec!["1", "2"], vec!["3", "4"]]);
}

#[test]
fn csv_quoted_fields_commas_newlines_doubled_quotes() {
    let (headers, rows) = csvio::parse_csv("x,y\n\"a,b\",\"line1\nline2\"\n\"say \"\"hi\"\"\",z\n");
    assert_eq!(headers, vec!["x", "y"]);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], vec!["a,b", "line1\nline2"]);
    assert_eq!(rows[1], vec!["say \"hi\"", "z"]);
}

#[test]
fn csv_crlf() {
    let (headers, rows) = csvio::parse_csv("a,b\r\n1,2\r\n");
    assert_eq!(headers, vec!["a", "b"]);
    assert_eq!(rows, vec![vec!["1", "2"]]);
}

#[test]
fn csv_utf8_preserved() {
    // Regression: byte-wise parsing used to corrupt multi-byte sequences.
    let (_, rows) = csvio::parse_csv("h\nLight Mode \u{2713}\n");
    assert_eq!(rows[0][0], "Light Mode \u{2713}");
}

#[test]
fn csv_bom_and_missing_trailing_fields() {
    let path = tmp_path("bom.csv");
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(b"\xEF\xBB\xBFa,b,c\n1,2\n").unwrap();
    drop(f);
    let table = csvio::read_csv(&path).unwrap();
    assert_eq!(table.headers, vec!["a", "b", "c"]);
    assert_eq!(table.rows[0]["c"], ""); // restval=None → ""
}

#[test]
fn csv_trailing_empty_record() {
    // A bare quoted empty field is still a record.
    let (_, rows) = csvio::parse_csv("a\n\"\"\n");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], "");
}

// ---------- difflib ----------

#[test]
fn difflib_ratio_known_values() {
    // CPython: SequenceMatcher(None, "abcd", "abxd").ratio() == 0.75
    assert!((difflib::ratio("abcd", "abxd") - 0.75).abs() < 1e-9);
    assert_eq!(difflib::ratio("", ""), 1.0);
    assert_eq!(difflib::ratio("abc", ""), 0.0);
    // "kitten" vs "sitting": CPython reports ~0.6153846...
    let r = difflib::ratio("kitten", "sitting");
    assert!((r - 0.6153846153846154).abs() < 1e-9, "ratio was {}", r);
}

#[test]
fn difflib_no_underflow_on_edge_matches() {
    // Regression for usize underflow when a match starts at position 0.
    assert_eq!(difflib::ratio("a", "a"), 1.0);
    assert!((difflib::ratio("ab", "abc") - 0.8).abs() < 1e-9);
    let _ = difflib::ratio("nonexistenttermzzz", "color");
}

// ---------- cli ----------

fn specs() -> Vec<cli::ArgSpec> {
    vec![
        cli::ArgSpec::positional("query").req(),
        cli::ArgSpec::value("domain", Some('d'), "domain"),
        cli::ArgSpec::flag("json", None, "json"),
        cli::ArgSpec::int("n", Some('n'), "num")
            .range(1, 20)
            .def("3"),
        cli::ArgSpec::value("fmt", Some('f'), "format")
            .choices(&["a", "b"])
            .def("a"),
        cli::ArgSpec::multi("tags", None, "tags"),
    ]
}

fn argv(s: &[&str]) -> Vec<String> {
    s.iter().map(|x| x.to_string()).collect()
}

#[test]
fn cli_positionals_flags_and_values() {
    let a = cli::parse(
        "t",
        &argv(&["hello", "--json", "-d", "style", "-n", "5"]),
        &specs(),
    );
    assert_eq!(a.get("query"), Some("hello"));
    assert!(a.flag("json"));
    assert_eq!(a.get("domain"), Some("style"));
    assert_eq!(a.int("n", 0), 5);
}

#[test]
fn cli_attached_short_and_eq() {
    let a = cli::parse("t", &argv(&["q", "-dstyle", "--num=7"]), &specs());
    assert_eq!(a.get("domain"), Some("style"));
    assert_eq!(a.int("n", 0), 7);
}

#[test]
fn cli_defaults() {
    let a = cli::parse("t", &argv(&["q"]), &specs());
    assert_eq!(a.int("n", 0), 3);
    assert_eq!(a.get_or("fmt", "?"), "a");
}

#[test]
fn cli_multi() {
    let a = cli::parse("t", &argv(&["q", "--tags", "x", "y", "z"]), &specs());
    assert_eq!(a.multi("tags"), vec!["x", "y", "z"]);
}

#[test]
fn cli_negative_number_positional() {
    let a = cli::parse("t", &argv(&["-1.5"]), &specs());
    assert_eq!(a.get("query"), Some("-1.5"));
}

// ---------- jsonfmt / py ----------

#[test]
fn jsonfmt_python_indent() {
    let v = json!({"a": 1, "b": [true, null]});
    let out = jsonfmt::dumps_indent(&v, 2);
    assert_eq!(
        out,
        "{\n  \"a\": 1,\n  \"b\": [\n    true,\n    null\n  ]\n}"
    );
}

#[test]
fn jsonfmt_compact() {
    assert_eq!(jsonfmt::dumps(&json!({"k": "v"})), "{\"k\": \"v\"}");
}

#[test]
fn py_repr_values() {
    assert_eq!(py::repr(&json!("it's")), "\"it's\"");
    assert_eq!(py::repr(&json!("plain")), "'plain'");
    assert_eq!(py::repr(&json!({"a": 1})), "{'a': 1}");
    assert_eq!(py::repr(&json!(None::<bool>)), "None");
    assert_eq!(py::repr(&json!(true)), "True");
}

#[test]
fn py_round_and_dates() {
    assert_eq!(py::round_py(2.5, 0), 2.0); // banker's rounding
    assert_eq!(py::round_py(3.5, 0), 4.0);
    assert!(py::parse_iso_date("2024-01-31").is_some());
    assert!(py::parse_iso_date("not-a-date").is_none());
    assert!(py::valid_date_not_future("2000-01-01"));
    assert!(!py::valid_date_not_future("2999-01-01"));
}

// ---------- rng ----------

#[test]
fn rng_seeded_determinism() {
    let mut a = Rng::from_seed(42);
    let mut b = Rng::from_seed(42);
    for _ in 0..100 {
        assert_eq!(a.random(), b.random());
    }
    let mut c = Rng::from_seed(43);
    let av: Vec<f64> = (0..10).map(|_| a.random()).collect();
    let cv: Vec<f64> = (0..10).map(|_| c.random()).collect();
    assert_ne!(av, cv);
}

#[test]
fn rng_distributions() {
    let mut r = Rng::from_seed(7);
    for _ in 0..1000 {
        let u = r.uniform(2.0, 3.0);
        assert!((2.0..3.0).contains(&u));
        let g = r.gauss(0.0, 1.0);
        assert!(g.is_finite());
    }
}
