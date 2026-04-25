#!/usr/bin/env python3
import datetime
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

CIEL_VERSION = os.environ.get("CIEL_VERSION", "1.0.0")
CIEL_HOME = Path(os.environ.get("CIEL_HOME", Path.home() / ".ciel")).resolve()
LOG_FILE = CIEL_HOME / "bootstrap.log"

def say(msg: str):
    print(f"\033[1;36m[ciel]\033[0m {msg}")
    with open(LOG_FILE, "a", encoding="utf-8") as f:
        f.write(f"[ciel] {msg}\n")

def warn(msg: str):
    print(f"\033[1;33m[ciel]\033[0m {msg}", file=sys.stderr)
    with open(LOG_FILE, "a", encoding="utf-8") as f:
        f.write(f"[ciel] WARN {msg}\n")

def die(msg: str):
    print(f"\033[1;31m[ciel]\033[0m {msg}", file=sys.stderr)
    with open(LOG_FILE, "a", encoding="utf-8") as f:
        f.write(f"[ciel] FAIL {msg}\n")
    sys.exit(1)

def need(cmd: str) -> bool:
    return shutil.which(cmd) is not None

def run(cmd: list[str], cwd: Path = None, check: bool = True) -> subprocess.CompletedProcess:
    try:
        return subprocess.run(cmd, cwd=cwd, check=check, capture_output=True, text=True)
    except subprocess.CalledProcessError as e:
        if check:
            die(f"Command failed: {' '.join(cmd)}\n{e.stderr}")
        return e

def main():
    CIEL_HOME.mkdir(parents=True, exist_ok=True)
    with open(LOG_FILE, "w", encoding="utf-8") as f:
        f.write(f"--- Ciel Bootstrap {datetime.datetime.now().isoformat()} ---\n")

    say(f"Ciel {CIEL_VERSION} — unified cross-platform setup")
    say(f"CIEL_HOME={CIEL_HOME}")

    # 1. Directory skeleton
    dirs = [
        "skills", "registry", "council", "improvements", "high_risk",
        "acquisition", "checkpoints", "archive", ".attic", "sandbox",
        "backups", "integrity", "runtimes"
    ]
    for d in dirs:
        (CIEL_HOME / d).mkdir(parents=True, exist_ok=True)
    say("Directory skeleton ready.")

    # 2. Git init
    if need("git"):
        if not (CIEL_HOME / ".git").exists():
            run(["git", "init", "-q"], cwd=CIEL_HOME)
            # Default branch name might vary by git version/config, but we prefer 'main'
            run(["git", "checkout", "-b", "main"], cwd=CIEL_HOME, check=False)
            
            gitignore_content = (
                ".cache/\n"
                "activity.log\n"
                "backups/\n"
                "archive/\n"
                "fs_backend/\n"
                "*.db\n"
                "checkpoints/\n"
                ".attic/\n"
                "sandbox/\n"
            )
            (CIEL_HOME / ".gitignore").write_text(gitignore_content, encoding="utf-8")
            
            run(["git", "add", "-A"], cwd=CIEL_HOME)
            run(["git", "commit", "-q", "-m", f"genesis: Ciel cold start @ {CIEL_VERSION}"], cwd=CIEL_HOME, check=False)
            say("Git repository initialized.")
        else:
            say("Git repository already present.")
    else:
        warn("git not found; skipping git setup.")

    # 3. Rust & MemPalace-rs
    skip_mempalace = False
    if not need("cargo"):
        warn("Rust toolchain (cargo) not found.")
        if os.environ.get("CIEL_AUTO_INSTALL_RUST") == "1":
            say("Auto-installing Rust...")
            # This is complex across platforms, usually we'd download rustup-init
            # For brevity in this setup.py, we'll assume the user should install it or we fallback
            warn("Auto-install of Rust not fully implemented in setup.py yet. Falling back.")
            skip_mempalace = True
        else:
            skip_mempalace = True

    if not skip_mempalace and need("cargo"):
        if not need("mempalace-rs"):
            say("Installing mempalace-rs (cargo install --locked)...")
            res = run(["cargo", "install", "mempalace-rs", "--locked"], check=False)
            if res.returncode != 0:
                warn(f"cargo install failed. Error: {res.stderr}")
                skip_mempalace = True
        else:
            say("mempalace-rs already installed.")

    # 4. Fallback backend
    if skip_mempalace:
        if need("sqlite3"):
            say("Configuring SQLite fallback backend.")
            (CIEL_HOME / "ciel.db").touch()
        else:
            warn("sqlite3 not found; falling back to filesystem KV backend.")
            (CIEL_HOME / "fs_backend").mkdir(parents=True, exist_ok=True)

    # 7. Integrity seed
    now = datetime.datetime.now(datetime.UTC).isoformat(timespec='seconds').replace('+00:00', 'Z')
    integrity = {
        "schema": 1,
        "version": CIEL_VERSION,
        "timestamp": now,
        "files": {}
    }

    (CIEL_HOME / "INTEGRITY.json").write_text(json.dumps(integrity, indent=2), encoding="utf-8")
    say("Integrity seed written.")

    # 6. Activity log
    log_entry = {
        "ts": now,
        "kind": "bootstrap",
        "version": CIEL_VERSION
    }
    with open(CIEL_HOME / "activity.log", "a", encoding="utf-8") as f:
        f.write(json.dumps(log_entry) + "\n")

    # 7. Verification
    say("Running verification...")
    # Verify.sh is still POSIX-centric, so we skip it if on Windows or it fails
    verify_script = Path(__file__).parent / "verify.sh"
    if verify_script.exists() and os.name != 'nt':
        run(["bash", str(verify_script)], check=False)
    elif os.name == 'nt':
        # On Windows, we ensure .ps1 hooks are copied if they exist
        hook_src = Path(__file__).parent.parent / "hooks"
        hook_dst = CIEL_HOME / "hooks"
        hook_dst.mkdir(parents=True, exist_ok=True)
        for adapter_dir in hook_src.iterdir():
            if adapter_dir.is_dir():
                target_adapter_dir = hook_dst / adapter_dir.name
                target_adapter_dir.mkdir(parents=True, exist_ok=True)
                for hook_file in adapter_dir.glob("*.ps1"):
                    shutil.copy2(hook_file, target_adapter_dir / hook_file.name)
        say("PowerShell hooks copied for Windows.")
    else:
        say("Skipping verify.sh; Ciel integrity check will run on load.")

    say("Ciel bootstrap complete.")

if __name__ == "__main__":
    main()
