//! Minimal argparse-compatible CLI parsing. Supports the surface these tools
//! use: positionals, `--long value`, `--long=value`, `-s value`, `-s=value`,
//! `-sVALUE` (attached short), store-true flags, `nargs='+'` lists, typed
//! int/float conversion, `choices`, `required`, and argparse's `exit(2)` on
//! usage errors.

use std::collections::HashMap;
use std::process::exit;

#[derive(Clone, Copy, PartialEq)]
pub enum ArgKind {
    /// `store_true`
    Flag,
    /// single string value
    Value,
    /// `type=int`
    Int,
    /// `type=float`
    Float,
    /// `nargs='+'` — consumes following non-flag tokens
    Multi,
    /// positional argument
    Positional,
}

#[derive(Clone)]
pub struct ArgSpec {
    pub name: &'static str,
    pub short: Option<char>,
    pub long: Option<&'static str>,
    pub kind: ArgKind,
    pub required: bool,
    pub default: Option<&'static str>,
    pub choices: Option<&'static [&'static str]>,
    pub int_range: Option<(i64, i64)>,
}

impl ArgSpec {
    pub fn flag(name: &'static str, short: Option<char>, long: &'static str) -> Self {
        ArgSpec {
            name,
            short,
            long: Some(long),
            kind: ArgKind::Flag,
            required: false,
            default: None,
            choices: None,
            int_range: None,
        }
    }
    pub fn value(name: &'static str, short: Option<char>, long: &'static str) -> Self {
        ArgSpec {
            name,
            short,
            long: Some(long),
            kind: ArgKind::Value,
            required: false,
            default: None,
            choices: None,
            int_range: None,
        }
    }
    pub fn int(name: &'static str, short: Option<char>, long: &'static str) -> Self {
        ArgSpec {
            name,
            short,
            long: Some(long),
            kind: ArgKind::Int,
            required: false,
            default: None,
            choices: None,
            int_range: None,
        }
    }
    pub fn float(name: &'static str, short: Option<char>, long: &'static str) -> Self {
        ArgSpec {
            name,
            short,
            long: Some(long),
            kind: ArgKind::Float,
            required: false,
            default: None,
            choices: None,
            int_range: None,
        }
    }
    pub fn multi(name: &'static str, short: Option<char>, long: &'static str) -> Self {
        ArgSpec {
            name,
            short,
            long: Some(long),
            kind: ArgKind::Multi,
            required: false,
            default: None,
            choices: None,
            int_range: None,
        }
    }
    pub fn positional(name: &'static str) -> Self {
        ArgSpec {
            name,
            short: None,
            long: None,
            kind: ArgKind::Positional,
            required: false,
            default: None,
            choices: None,
            int_range: None,
        }
    }
    pub fn req(mut self) -> Self {
        self.required = true;
        self
    }
    pub fn def(mut self, v: &'static str) -> Self {
        self.default = Some(v);
        self
    }
    pub fn choices(mut self, c: &'static [&'static str]) -> Self {
        self.choices = Some(c);
        self
    }
    pub fn range(mut self, lo: i64, hi: i64) -> Self {
        self.int_range = Some((lo, hi));
        self
    }
}

pub struct Args {
    values: HashMap<String, Vec<String>>,
    flags: HashMap<String, bool>,
}

fn usage_error(prog: &str, msg: &str) -> ! {
    eprintln!("usage: {} [options]", prog);
    eprintln!("{}: error: {}", prog, msg);
    exit(2);
}

fn looks_like_flag(tok: &str) -> bool {
    tok.starts_with('-') && tok != "-" && tok.parse::<f64>().is_err()
}

pub fn parse(prog: &str, argv: &[String], specs: &[ArgSpec]) -> Args {
    let mut values: HashMap<String, Vec<String>> = HashMap::new();
    let mut flags: HashMap<String, bool> = HashMap::new();
    for spec in specs {
        match spec.kind {
            ArgKind::Flag => {
                flags.insert(spec.name.to_string(), false);
            }
            _ => {
                if let Some(d) = spec.default {
                    values.insert(spec.name.to_string(), vec![d.to_string()]);
                }
            }
        }
    }

    let positionals: Vec<&ArgSpec> = specs
        .iter()
        .filter(|s| s.kind == ArgKind::Positional)
        .collect();
    let mut pos_idx = 0usize;
    let mut i = 0usize;
    let mut end_of_opts = false;

    while i < argv.len() {
        let tok = &argv[i];
        if end_of_opts {
            if pos_idx < positionals.len() {
                values
                    .entry(positionals[pos_idx].name.to_string())
                    .or_default()
                    .push(tok.clone());
                pos_idx += 1;
            } else {
                usage_error(prog, &format!("unrecognized arguments: {}", tok));
            }
            i += 1;
            continue;
        }
        if tok == "--" {
            end_of_opts = true;
            i += 1;
            continue;
        }
        if tok == "-h" || tok == "--help" {
            println!("usage: {} [options]", prog);
            exit(0);
        }

        let matched: Option<(&ArgSpec, Option<String>)> = if let Some(body) = tok.strip_prefix("--")
        {
            let (name, inline) = match body.find('=') {
                Some(eq) => (&body[..eq], Some(body[eq + 1..].to_string())),
                None => (body, None),
            };
            match specs.iter().find(|s| s.long == Some(name)) {
                Some(s) => Some((s, inline)),
                None => usage_error(prog, &format!("unrecognized arguments: {}", tok)),
            }
        } else if tok.starts_with('-') && tok.len() >= 2 {
            let chars: Vec<char> = tok[1..].chars().collect();
            let c = chars[0];
            let spec = specs.iter().find(|s| s.short == Some(c));
            match spec {
                Some(s) => {
                    let rest: String = chars[1..].iter().collect();
                    let inline = if rest.is_empty() {
                        None
                    } else if let Some(stripped) = rest.strip_prefix('=') {
                        Some(stripped.to_string())
                    } else {
                        Some(rest)
                    };
                    Some((s, inline))
                }
                None => {
                    // Could be a negative number positional value
                    if tok.parse::<f64>().is_ok() {
                        None
                    } else {
                        usage_error(prog, &format!("unrecognized arguments: {}", tok));
                    }
                }
            }
        } else {
            None
        };

        match matched {
            Some((spec, inline)) => match spec.kind {
                ArgKind::Flag => {
                    if inline.is_some() {
                        usage_error(
                            prog,
                            &format!(
                                "argument --{} takes no value",
                                spec.long.unwrap_or(spec.name)
                            ),
                        );
                    }
                    flags.insert(spec.name.to_string(), true);
                    i += 1;
                }
                ArgKind::Multi => {
                    let mut items: Vec<String> = Vec::new();
                    if let Some(v) = inline {
                        items.push(v);
                    } else {
                        let mut j = i + 1;
                        while j < argv.len() && !looks_like_flag(&argv[j]) {
                            items.push(argv[j].clone());
                            j += 1;
                        }
                        i = j - 1;
                    }
                    if items.is_empty() && spec.required {
                        usage_error(
                            prog,
                            &format!(
                                "argument --{}: expected at least one argument",
                                spec.long.unwrap_or(spec.name)
                            ),
                        );
                    }
                    values.insert(spec.name.to_string(), items);
                    i += 1;
                }
                _ => {
                    let raw = match inline {
                        Some(v) => {
                            i += 1;
                            v
                        }
                        None => {
                            if i + 1 >= argv.len() {
                                usage_error(
                                    prog,
                                    &format!(
                                        "argument --{}: expected one argument",
                                        spec.long.unwrap_or(spec.name)
                                    ),
                                );
                            }
                            i += 2;
                            argv[i - 1].clone()
                        }
                    };
                    match spec.kind {
                        ArgKind::Int => {
                            if raw.trim().parse::<i64>().is_err() {
                                usage_error(
                                    prog,
                                    &format!(
                                        "argument --{}: invalid int value: '{}'",
                                        spec.long.unwrap_or(spec.name),
                                        raw
                                    ),
                                );
                            }
                        }
                        ArgKind::Float if raw.trim().parse::<f64>().is_err() => {
                            usage_error(
                                prog,
                                &format!(
                                    "argument --{}: invalid float value: '{}'",
                                    spec.long.unwrap_or(spec.name),
                                    raw
                                ),
                            );
                        }
                        _ => {}
                    }
                    values.insert(spec.name.to_string(), vec![raw]);
                }
            },
            None => {
                if pos_idx < positionals.len() {
                    values
                        .entry(positionals[pos_idx].name.to_string())
                        .or_default()
                        .push(tok.clone());
                    pos_idx += 1;
                } else {
                    usage_error(prog, &format!("unrecognized arguments: {}", tok));
                }
                i += 1;
            }
        }
    }

    // Validate choices / ranges / required
    let opt_label = |spec: &ArgSpec| -> String {
        match (spec.long, spec.short) {
            (Some(l), Some(s)) => format!("--{}/-{}", l, s),
            (Some(l), None) => format!("--{}", l),
            (None, Some(s)) => format!("-{}", s),
            (None, None) => spec.name.to_string(),
        }
    };
    for spec in specs {
        if let Some(vs) = values.get(spec.name) {
            for v in vs {
                if let Some(choices) = spec.choices {
                    if !choices.contains(&v.as_str()) {
                        let rendered: Vec<String> =
                            choices.iter().map(|c| format!("'{}'", c)).collect();
                        usage_error(
                            prog,
                            &format!(
                                "argument {}: invalid choice: '{}' (choose from {})",
                                opt_label(spec),
                                v,
                                rendered.join(", ")
                            ),
                        );
                    }
                }
                if let Some((lo, hi)) = spec.int_range {
                    if let Ok(n) = v.parse::<i64>() {
                        if n < lo || n > hi {
                            // argparse renders choices=range(lo, hi+1) as the
                            // full enumerated list.
                            let rendered: Vec<String> = (lo..=hi).map(|x| x.to_string()).collect();
                            usage_error(
                                prog,
                                &format!(
                                    "argument {}: invalid choice: {} (choose from {})",
                                    opt_label(spec),
                                    n,
                                    rendered.join(", ")
                                ),
                            );
                        }
                    }
                }
            }
        }
        let present = match spec.kind {
            ArgKind::Flag => flags.get(spec.name).copied().unwrap_or(false),
            _ => values.contains_key(spec.name),
        };
        if spec.required && !present {
            usage_error(
                prog,
                &format!(
                    "the following arguments are required: {}",
                    spec.long.unwrap_or(spec.name)
                ),
            );
        }
    }

    Args { values, flags }
}

impl Args {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.values
            .get(name)
            .and_then(|v| v.first())
            .map(|s| s.as_str())
    }
    pub fn get_or(&self, name: &str, default: &str) -> String {
        self.get(name).unwrap_or(default).to_string()
    }
    pub fn flag(&self, name: &str) -> bool {
        self.flags.get(name).copied().unwrap_or(false)
    }
    pub fn int(&self, name: &str, default: i64) -> i64 {
        self.get(name)
            .and_then(|s| s.trim().parse::<i64>().ok())
            .unwrap_or(default)
    }
    pub fn float(&self, name: &str, default: f64) -> f64 {
        self.get(name)
            .and_then(|s| s.trim().parse::<f64>().ok())
            .unwrap_or(default)
    }
    pub fn multi(&self, name: &str) -> Vec<String> {
        self.values.get(name).cloned().unwrap_or_default()
    }
}
