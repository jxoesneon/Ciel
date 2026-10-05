mod agentscripts;
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
    pretool --runtime {devin|antigravity|opencode}   PreToolUse gate body\n\
    prompt-submit                            UserPromptSubmit scan + canary\n\
    session-start --runtime {devin|antigravity|opencode}  SessionStart body (one process)\n\
\n\
PRIMITIVES (used by shell hooks and tests):\n\
    risk-eval        stdin {tool,command,path} → verdict JSON\n\
    risk-check       policy load diagnostics (exit 1 on fallback/empty)\n\
    grant-state      sentinel state + provenance JSON\n\
    secret-scan      stdin text → {hits, categories}\n\
    attribution-scan stdin command → {result, findings, mode}\n\
    route-choice     stdin {task, options} → verdict JSON\n\
    context-select   stdin {task, options|context_items, k} → keep/drop map\n\
    memory-salience  stdin {event} → write-back verdict JSON\n\
    context-compaction stdin {stats|context|budget} → compaction verdict\n\
    mandate-canary   stdin {mandates, context} → canary verdict JSON\n\
    verify-completion [--objective ..] [--evidence ..] → verification JSON\n\
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
    preflight        stdin {tool,command,path} → normalized verdict (agent gate)\n\
    audit            activity.log writer — argv/stdin fields → {} (never fails)\n\
    verify-evidence  completion-gate runner (banner + System-1 verdict)\n\
    integrity [--home|--write|--json]  git-tracked file sweep vs INTEGRITY.json\n\
    post-tool / post-invocation / permission-request / session-end / stop\n\
                       hook bodies — stdin payload → activity.log (+ contract\n\
                       output where the runtime expects it)\n\
    setup [--src DIR]  bootstrap a CIEL_HOME (install.sh/setup.py twin)\n\
\n\
Fallback contract: on any error or unknown command the binary exits 2 and\n\
the shell wrappers fall back to the Python implementations.\n";

fn arg_runtime(args: &[String]) -> &str {
    args.iter()
        .position(|a| a == "--runtime")
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
        .unwrap_or("devin")
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("");
    let code = match cmd {
        "pretool" => pretool::main_(arg_runtime(&args)),
        "prompt-submit" => promptsubmit::main_(),
        "session-start" => sessionstart::main_(arg_runtime(&args)),
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
        "route-choice" => system1::route_choice_main(),
        "context-select" => system1::context_select_main(),
        "memory-salience" => system1::memory_salience_main(),
        "context-compaction" => system1::context_compaction_main(),
        "mandate-canary" => system1::mandate_canary_main(),
        "verify-completion" => system1::verify_completion_main(&args[1..]),
        "preflight" => agentscripts::preflight_main(&args[1..]),
        "audit" => agentscripts::audit_main(&args[1..]),
        "verify-evidence" => agentscripts::verify_evidence_main(&args[1..]),
        "integrity" => agentscripts::integrity_main(&args[1..]),
        "post-tool" => agentscripts::post_tool_main(arg_runtime(&args[1..])),
        "post-invocation" => agentscripts::post_invocation_main(),
        "permission-request" => agentscripts::permission_request_main(),
        "session-end" => agentscripts::session_end_main(arg_runtime(&args[1..])),
        "stop" => agentscripts::stop_main(arg_runtime(&args[1..])),
        "setup" => agentscripts::setup_main(&args[1..]),
        "verify-3d" => agentscripts::verify_3d_main(&args[1..]),
        "config-heal" => agentscripts::config_heal_main(),
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
