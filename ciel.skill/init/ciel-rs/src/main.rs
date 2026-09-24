mod attribution;
mod paths;
mod pretool;
mod promptsubmit;
mod risk;
mod secretscan;
mod shadow;

use std::env;
use std::process::ExitCode;

const USAGE: &str = "ciel — agent runtime safety runtime\n\
USAGE:\n\
    ciel <command> [args]\n\
\n\
HOOK BODIES (stdin JSON → stdout JSON):\n\
    pretool --runtime {devin|antigravity}   PreToolUse gate body\n\
    prompt-submit                            UserPromptSubmit scan + canary\n\
\n\
PRIMITIVES (used by shell hooks and tests):\n\
    risk-eval        stdin {tool,command,path} → verdict JSON\n\
    risk-check       policy load diagnostics (exit 1 on fallback/empty)\n\
    grant-state      sentinel state + provenance JSON\n\
    secret-scan      stdin text → {hits, categories}\n\
    attribution-scan stdin command → {result, findings, mode}\n\
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
        "risk-eval" => risk::eval_main(),
        "risk-check" => risk::check_main(),
        "grant-state" => risk::grant_state_main(),
        "secret-scan" => secretscan::main_(),
        "attribution-scan" => attribution::main_(),
        "-h" | "--help" | "help" | "" => {
            print!("{USAGE}");
            if cmd.is_empty() || cmd == "-h" || cmd == "--help" || cmd == "help" {
                0
            } else {
                0
            }
        }
        _ => {
            eprintln!("ciel: unknown command '{cmd}'");
            eprint!("{USAGE}");
            2
        }
    };
    ExitCode::from(code as u8)
}
