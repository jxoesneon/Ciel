//! ciel-convo — find and extract Devin (ACP) conversations from local
//! Windsurf/Devin storage. Mirrors
//! `skills/devin-conversation-recovery/scripts/find_devin_convo.py`.

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ciel_domain::convo::run(&argv));
}
