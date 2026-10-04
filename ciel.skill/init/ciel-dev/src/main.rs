//! `ciel-dev` — single binary port of the scripts/*.py dev/maintenance
//! harnesses. Each Python CLI maps to a subcommand with equivalent flags,
//! stdout shape, and exit codes.

mod embed;
mod eval;
mod export;
mod harmonize;
mod jsonfmt;
mod lintfix;
mod mdlint;
mod migrate;
mod paired;
mod proc;
mod py;
mod registry;
mod review;
mod rlcd;
mod scan;
mod secrets;
mod sys1;

const USAGE: &str = "usage: ciel-dev <command> [<args>]

commands:
  eval             system1_eval.py     — calibration eval over labeled corpora
  export           system1_export.py   — export high-confidence training rows
  review           system1_review.py   — inspect the System-1 event log
  rlcd             ciel_rlcd_pipeline  — build RLCD preference pairs
  paired-eval      paired_eval.py      — control/treatment skill A/B harness
  scan-skills      scan_skills.py      — supply-chain scanner wrapper
  harmonize-skills harmonize_skills.py — normalize skill metadata/tags
  fix-md-lint      fix_md_lint.py      — markdownlint auto-fixes
  lint-fix         lint-fix.py         — repo-wide markdown tidy pass
  migrate-sidecar  migrate_skill_sidecar.py — split frontmatter into ciel.yaml
  registry-index   build_registry_index.py  — build registry/index.json
  embed            system1_embed.py    — bi-encoder top-k (local MiniLM)

Run 'ciel-dev <command> --help' for a command's flags.";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        None | Some("-h") | Some("--help") | Some("help") => {
            println!("{USAGE}");
            0
        }
        Some("eval") => eval::main_(&args[1..]),
        Some("export") => export::main_(&args[1..]),
        Some("review") => review::main_(&args[1..]),
        Some("rlcd") => rlcd::main_(&args[1..]),
        Some("paired-eval") => paired::main_(&args[1..]),
        Some("scan-skills") => scan::main_(&args[1..]),
        Some("harmonize-skills") => harmonize::main_(&args[1..]),
        Some("fix-md-lint") => mdlint::main_(&args[1..]),
        Some("lint-fix") => lintfix::main_(&args[1..]),
        Some("migrate-sidecar") => migrate::main_(&args[1..]),
        Some("registry-index") => registry::main_(&args[1..]),
        Some("embed") => embed::main_(&args[1..]),
        Some(other) => {
            eprintln!("ciel-dev: unknown command '{other}'");
            eprintln!("{USAGE}");
            2
        }
    };
    std::process::exit(code);
}
