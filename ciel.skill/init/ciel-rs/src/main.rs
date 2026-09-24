mod attribution;
mod compilepolicy;
mod councilverify;
mod jsonfmt;
mod ledger;
mod paths;
mod perms;
mod pretool;
mod promptsubmit;
mod risk;
mod rotate;
mod sanitize;
mod secretscan;
mod sessionstart;
mod shadow;
mod system1;
mod watchdog;

use std::env;
use std::process::ExitCode;

const USAGE: &str = "ciel — agent runtime safety runtime\n\
USAGE:\n\
    ciel <command> [args]\n\
\n\
HOOK BODIES (stdin JSON → stdout JSON):\n\
    pretool --runtime {devin|antigravity}   PreToolUse gate body\n\
    prompt-submit                            UserPromptSubmit scan + canary\n\
    session-start --runtime {devin|antigravity}  SessionStart body (one process)\n\
\n\
PRIMITIVES (used by shell hooks and tests):\n\
    risk-eval        stdin {tool,command,path} → verdict JSON\n\
    risk-check       policy load diagnostics (exit 1 on fallback/empty)\n\
    grant-state      sentinel state + provenance JSON\n\
    secret-scan      stdin text → {hits, categories}\n\
    attribution-scan stdin command → {result, findings, mode}\n\
\n\
SESSION OPERATIONS:\n\
    watchdog [--session ID|--resume [--dry]|--sanitize-pending]\n\
    store-perms      owner-only permission sweep → ok|repaired:N\n\
    ledger {add|done|list|pending} [arg] [--session ID]\n\
    log-rotate       rotate activity.log per retention policy\n\
\n\
OPERATOR:\n\
    sanitize [--scan|--redact] [--dry] [--deep]  transcript-store redaction\n\
    compile-policy [--check]   policy.yaml → policy.json (byte-exact twin)\n\
    council-verify <run|dir>   council run artifact verification\n\
    system1 --ask|--decide     detached shadow-ask / interactive verdict\n\
\n\
Fallback contract: on any error or unknown command the binary exits 2 and\n\
the shell wrappers fall back to the Python implementations.\n";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("");
    let code = match cmd {
        "pretool" => {
            let runtime = args
                .iter()
                .position(|a| a == "--runtime")
                .and_then(|i| args.get(i + 1))
                .map(String::as_str)
                .unwrap_or("devin");
            pretool::main_(runtime)
        }
        "prompt-submit" => promptsubmit::main_(),
        "session-start" => {
            let runtime = args
                .iter()
                .position(|a| a == "--runtime")
                .and_then(|i| args.get(i + 1))
                .map(String::as_str)
                .unwrap_or("devin");
            sessionstart::main_(runtime)
        }
        "watchdog" => watchdog::main_(&args[1..]),
        "store-perms" => perms::main_(),
        "ledger" => ledger::main_(&args[1..]),
        "log-rotate" => rotate::main_(),
        "sanitize" => sanitize::main_(&args[1..]),
        "compile-policy" => compilepolicy::main_(&args[1..]),
        "council-verify" => councilverify::main_(&args[1..]),
        "system1" => system1::main_(&args[1..]),
        "risk-eval" => risk::eval_main(),
        "risk-check" => risk::check_main(),
        "grant-state" => risk::grant_state_main(),
        "secret-scan" => secretscan::main_(),
        "attribution-scan" => attribution::main_(),
        "-h" | "--help" | "help" | "" => {
            print!("{USAGE}");
            0
        }
        _ => {
            eprintln!("ciel: unknown command '{cmd}'");
            eprint!("{USAGE}");
            2
        }
    };
    ExitCode::from(code as u8)
}
