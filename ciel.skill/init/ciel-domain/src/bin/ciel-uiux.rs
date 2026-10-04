//! ciel-uiux — unified CLI for the ui-ux-pro-max domain skill.
//!
//! Mirrors `scripts/search.py`: a positional `query` plus `--domain`,
//! `--stack`, `--max-results`, `--json`, `--full`, and the design-system
//! flags. The reserved leading keyword `validate` runs the data-integrity
//! validator (mirrors `scripts/validate_data.py`).

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ciel_domain::uiux::cli::main(&argv));
}
