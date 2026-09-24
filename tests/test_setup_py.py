"""Smoke test for ciel.skill/init/scripts/setup.py in a sandboxed CIEL_HOME."""

import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SETUP = ROOT / "ciel.skill" / "init" / "scripts" / "setup.py"


class TestSetupPy(unittest.TestCase):
    def test_bootstrap_installs_hooks_and_policy(self):
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp) / "ciel-home"
            # Stub PATH without cargo/mempalace-rs: a host with cargo but no
            # mempalace-rs makes setup.py run a real `cargo install` that
            # outlives the timeout.
            stub_bin = Path(tmp) / "stub-bin"
            stub_bin.mkdir()
            for tool in ("bash", "git", "sqlite3"):
                tool_path = shutil.which(tool)
                if tool_path:
                    os.symlink(tool_path, stub_bin / tool)
            env = dict(os.environ, CIEL_HOME=str(home), PATH=str(stub_bin))
            proc = subprocess.run(
                [sys.executable, str(SETUP)],
                capture_output=True, text=True, env=env, timeout=120, check=False,
            )
            self.assertEqual(0, proc.returncode, proc.stdout + proc.stderr)
            # skeleton
            self.assertTrue((home / "skills").is_dir())
            self.assertTrue((home / "council").is_dir())
            # hooks payload incl. lib
            self.assertTrue((home / "hooks" / "lib" / "risk_policy.py").is_file())
            self.assertTrue((home / "hooks" / "devin" / "pre_tool_use.sh").is_file())
            # risk policy payload
            self.assertTrue((home / "risk" / "policy.json").is_file())
            self.assertTrue((home / "risk" / "policy.yaml").is_file())
            # seeds
            self.assertTrue((home / "INTEGRITY.json").is_file())
            self.assertTrue((home / "activity.log").is_file())


if __name__ == "__main__":
    unittest.main()
