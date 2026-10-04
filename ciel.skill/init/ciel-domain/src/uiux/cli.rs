//! Port of `skills/ui-ux-pro-max/scripts/search.py`'s CLI surface:
//! `format_output()` (token-optimized text rendering) plus the argparse
//! dispatch — design-system generation first, then `--stack`, then domain
//! search. `validate` is dispatched as a leading subcommand keyword.

use crate::common::cli::{parse, ArgSpec, Args};
use crate::common::jsonfmt;
use crate::common::py;
use crate::uiux::core::{
    available_stacks, search, search_stack, CSV_CONFIG, STACK_CONFIG, UNTRUNCATED_COLS,
};
use crate::uiux::data::Catalog;
use crate::uiux::design_system::generate_design_system;
use crate::uiux::validate;
use serde_json::{json, Value};
use std::process::exit;

const TRUNCATE_AT: usize = 300;

/// Python `str(value)` for the values that appear in result rows.
fn py_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "None".to_string(),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        Value::Number(n) => n.to_string(),
        other => py::repr(other),
    }
}

fn get<'a>(v: &'a Value, key: &str) -> &'a Value {
    v.get(key).unwrap_or(&Value::Null)
}

fn str_of(v: &Value, key: &str) -> String {
    py_str(get(v, key))
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// `format_output(result, full=False)` — human-readable rendering.
pub fn format_output(result: &Value, full: bool) -> String {
    let err = get(result, "error");
    if !err.is_null() {
        return format!("Error: {}", py_str(err));
    }

    let mut output: Vec<String> = Vec::new();
    let stack = get(result, "stack");
    if truthy(stack) {
        output.push("## UI Pro Max Stack Guidelines".to_string());
        output.push(format!(
            "**Stack:** {} | **Query:** {}",
            py_str(stack),
            str_of(result, "query")
        ));
    } else {
        output.push("## UI Pro Max Search Results".to_string());
        let mut domain_note = str_of(result, "domain");
        if truthy(get(result, "auto_detected")) {
            domain_note.push_str(" (auto-detected");
            let runner_up = get(result, "runner_up_domain");
            if truthy(runner_up) {
                domain_note.push_str(&format!(", runner-up: {}", py_str(runner_up)));
            }
            domain_note.push(')');
        }
        output.push(format!(
            "**Domain:** {} | **Query:** {}",
            domain_note,
            str_of(result, "query")
        ));
    }
    output.push(format!(
        "**Source:** {} | **Found:** {} results\n",
        str_of(result, "file"),
        str_of(result, "count")
    ));

    if get(result, "count").as_i64().unwrap_or(0) == 0 {
        let redirect = get(result, "redirect");
        if truthy(redirect) {
            output.push(format!(
                "This legacy style label is now modeled in the `{}` domain as `{}`. Search that domain instead of treating a page composition as a visual style.",
                str_of(redirect, "domain"),
                str_of(redirect, "id")
            ));
            return output.join("\n");
        }
        output.push(
            "No matches. This is not a match with an empty value -- the query did not hit the database. Retry with broader/different keywords before falling back to general defaults, and say explicitly that no database match was found if you do fall back."
                .to_string(),
        );
        let suggestions = get(result, "suggestions");
        if let Some(list) = suggestions.as_array() {
            if !list.is_empty() {
                let joined: Vec<String> = list.iter().map(py_str).collect();
                output.push(format!("**Closest known terms:** {}", joined.join(", ")));
            }
        }
        return output.join("\n");
    }

    if let Some(rows) = get(result, "results").as_array() {
        for (i, row) in rows.iter().enumerate() {
            output.push(format!("### Result {}", i + 1));
            if let Some(obj) = row.as_object() {
                for (key, value) in obj {
                    let mut value_str = py_str(value);
                    if !full
                        && !UNTRUNCATED_COLS.contains(&key.as_str())
                        && value_str.chars().count() > TRUNCATE_AT
                    {
                        value_str = format!(
                            "{}...",
                            value_str.chars().take(TRUNCATE_AT).collect::<String>()
                        );
                    }
                    output.push(format!("- **{}:** {}", key, value_str));
                }
            }
            output.push(String::new());
        }
    }

    output.join("\n")
}

fn usage_error(prog: &str, msg: &str) -> ! {
    eprintln!("usage: {} [options]", prog);
    eprintln!("{}: error: {}", prog, msg);
    exit(2);
}

/// `search.py __main__` — returns the process exit code.
pub fn run(argv: &[String]) -> i32 {
    // argparse registers `-ds` as its own option string; an exact match takes
    // precedence over splitting it as `-d s`. Normalize before parsing.
    let argv: Vec<String> = argv
        .iter()
        .map(|a| {
            if a == "-ds" {
                "--design-system".to_string()
            } else {
                a.clone()
            }
        })
        .collect();

    let prog = "ciel-uiux";
    let args: Args = parse(
        prog,
        &argv,
        &[
            ArgSpec::positional("query").req(),
            ArgSpec::value("domain", Some('d'), "domain"),
            ArgSpec::value("stack", Some('s'), "stack"),
            ArgSpec::int("max_results", Some('n'), "max-results")
                .range(1, 20)
                .def("3"),
            ArgSpec::flag("json", None, "json"),
            ArgSpec::flag("full", None, "full"),
            ArgSpec::flag("design_system", None, "design-system"),
            ArgSpec::value("project_name", Some('p'), "project-name"),
            ArgSpec::value("format", Some('f'), "format")
                .choices(&["ascii", "markdown"])
                .def("ascii"),
            ArgSpec::flag("persist", None, "persist"),
            ArgSpec::value("page", None, "page"),
            ArgSpec::value("output_dir", Some('o'), "output-dir"),
            ArgSpec::flag("force", None, "force"),
            ArgSpec::int("variance", None, "variance").range(1, 10),
            ArgSpec::int("motion", None, "motion").range(1, 10),
            ArgSpec::int("density", None, "density").range(1, 10),
        ],
    );

    // argparse `choices=` validation for domain / stack (exit 2 on mismatch).
    let quoted = |items: Vec<&str>| -> String {
        items
            .iter()
            .map(|c| format!("'{}'", c))
            .collect::<Vec<_>>()
            .join(", ")
    };
    if let Some(d) = args.get("domain") {
        if !CSV_CONFIG.iter().any(|(name, _, _, _)| *name == d) {
            usage_error(
                prog,
                &format!(
                    "argument --domain/-d: invalid choice: '{}' (choose from {})",
                    d,
                    quoted(CSV_CONFIG.iter().map(|(k, _, _, _)| *k).collect())
                ),
            );
        }
    }
    if let Some(s) = args.get("stack") {
        if !STACK_CONFIG.iter().any(|(name, _)| *name == s) {
            usage_error(
                prog,
                &format!(
                    "argument --stack/-s: invalid choice: '{}' (choose from {})",
                    s,
                    quoted(available_stacks())
                ),
            );
        }
    }

    let query = args.get("query").unwrap_or("");
    let json_out = args.flag("json");
    let catalog = Catalog::new();

    if args.flag("design_system") {
        if let Some(s) = args.get("stack") {
            eprintln!(
                "note: --stack {} is ignored in --design-system mode; run a separate --stack query for stack-specific guidelines",
                s
            );
        }
        let result = generate_design_system(
            &catalog,
            query,
            args.get("project_name"),
            args.get_or("format", "ascii").as_str(),
            args.flag("persist"),
            args.get("page"),
            args.get("output_dir"),
            args.get("variance").and_then(|v| v.parse::<i64>().ok()),
            args.get("motion").and_then(|v| v.parse::<i64>().ok()),
            args.get("density").and_then(|v| v.parse::<i64>().ok()),
            args.flag("force"),
        );
        if json_out {
            println!(
                "{}",
                jsonfmt::dumps_indent(
                    &json!({
                        "design_system": result.design_system,
                        "persistence": result.persistence.unwrap_or(Value::Null),
                    }),
                    2
                )
            );
        } else {
            println!("{}", result.text);
            if args.flag("persist") {
                let persistence = result.persistence.clone().unwrap_or(Value::Null);
                println!("\n{}", "=".repeat(60));
                if persistence.get("status").and_then(Value::as_str) == Some("skipped_exists") {
                    println!(
                        "⚠️  {}",
                        persistence
                            .get("message")
                            .and_then(Value::as_str)
                            .unwrap_or("MASTER.md already exists; not overwritten.")
                    );
                } else {
                    let ds_dir = persistence
                        .get("design_system_dir")
                        .and_then(Value::as_str)
                        .unwrap_or("design-system/<project>");
                    println!("✅ Design system persisted to {}/", ds_dir);
                    if let Some(files) = persistence.get("created_files").and_then(Value::as_array)
                    {
                        for f in files {
                            println!("   📄 {}", f.as_str().unwrap_or(""));
                        }
                    }
                    println!();
                    println!(
                        "📖 Usage: When building a page, check {}/pages/[page].md first.",
                        ds_dir
                    );
                    println!(
                        "   If it exists, its rules override MASTER.md. Otherwise, use MASTER.md."
                    );
                }
                println!("{}", "=".repeat(60));
            }
        }
    } else if let Some(stack) = args.get("stack") {
        let result = search_stack(&catalog, query, stack, args.int("max_results", 3), false);
        if json_out {
            println!("{}", jsonfmt::dumps_indent(&result, 2));
        } else {
            println!("{}", format_output(&result, args.flag("full")));
        }
    } else {
        let result = search(
            &catalog,
            query,
            args.get("domain"),
            args.int("max_results", 3),
            false,
        );
        if json_out {
            println!("{}", jsonfmt::dumps_indent(&result, 2));
        } else {
            println!("{}", format_output(&result, args.flag("full")));
        }
    }
    0
}

/// `ciel-uiux` entry — `validate` is dispatched as a leading keyword,
/// mirroring `python validate_data.py`.
pub fn main(argv: &[String]) -> i32 {
    if argv.first().map(|s| s.as_str()) == Some("validate") {
        return validate::run(&argv[1..]);
    }
    run(argv)
}
