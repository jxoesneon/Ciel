"""Tests for ciel.skill/init/scripts/integrity.py."""

import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = (
    Path(__file__).resolve().parent.parent
    / "ciel.skill" / "init" / "scripts" / "integrity.py"
)


def _git(home: Path, *args: str) -> None:
    subprocess.run(["git", "-C", str(home), *args], check=True,
                   capture_output=True, text=True)


def _run(home: Path, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(SCRIPT), "--home", str(home), *args],
        capture_output=True, text=True,
    )


def _latest_report(home: Path) -> dict:
    reports = sorted((home / "integrity").glob("*.json"))
    return json.loads(reports[-1].read_text(encoding="utf-8"))


class IntegrityTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.home = Path(self.tmp.name)
        _git(self.home, "init", "-q")
        _git(self.home, "config", "user.email", "test@example.com")
        _git(self.home, "config", "user.name", "Test")
        (self.home / "a.txt").write_text("alpha\n")
        (self.home / "b.txt").write_text("bravo\n")
        _git(self.home, "add", "a.txt", "b.txt")
        _git(self.home, "commit", "-qm", "init")

    def test_write_produces_spec_manifest(self):
        proc = _run(self.home, "--write")
        self.assertEqual(proc.returncode, 0, proc.stderr)
        manifest = json.loads((self.home / "INTEGRITY.json").read_text())
        self.assertEqual(manifest["schema"], 1)
        self.assertIn("last_verified", manifest)
        files = manifest["files"]
        self.assertEqual(set(files), {"a.txt", "b.txt"})
        for entry in files.values():
            self.assertIn("sha256", entry)
            self.assertIn("size", entry)
        self.assertNotIn("INTEGRITY.json", files)

    def _write_and_commit_manifest(self):
        self.assertEqual(_run(self.home, "--write").returncode, 0)
        _git(self.home, "add", "INTEGRITY.json")
        _git(self.home, "commit", "-qm", "manifest")

    def test_clean_sweep_all_ok(self):
        self._write_and_commit_manifest()
        proc = _run(self.home)
        self.assertEqual(proc.returncode, 0, proc.stderr)
        sweep = _latest_report(self.home)["sweep"]
        self.assertEqual(sweep["checked"], 2)
        self.assertEqual(sweep["ok"], 2)
        self.assertFalse(sweep["unknown_drift"])
        self.assertFalse(sweep["missing"])

    def test_uncommitted_change_is_unknown_drift(self):
        self._write_and_commit_manifest()
        (self.home / "a.txt").write_text("modified\n")
        proc = _run(self.home)
        self.assertEqual(proc.returncode, 1)
        self.assertEqual(_latest_report(self.home)["sweep"]["unknown_drift"], ["a.txt"])

    def test_committed_change_is_expected_drift(self):
        self._write_and_commit_manifest()
        (self.home / "a.txt").write_text("modified\n")
        _git(self.home, "add", "a.txt")
        _git(self.home, "commit", "-qm", "change a")
        proc = _run(self.home)
        self.assertEqual(proc.returncode, 0)
        self.assertEqual(_latest_report(self.home)["sweep"]["expected_drift"], ["a.txt"])

    def test_deleted_file_is_missing(self):
        self._write_and_commit_manifest()
        (self.home / "a.txt").unlink()
        proc = _run(self.home)
        self.assertEqual(proc.returncode, 1)
        self.assertEqual(_latest_report(self.home)["sweep"]["missing"], ["a.txt"])

    def test_new_committed_file_is_unexpected(self):
        self._write_and_commit_manifest()
        (self.home / "c.txt").write_text("charlie\n")
        _git(self.home, "add", "c.txt")
        _git(self.home, "commit", "-qm", "add c")
        proc = _run(self.home)
        self.assertEqual(proc.returncode, 0)
        self.assertEqual(_latest_report(self.home)["sweep"]["unexpected"], ["c.txt"])

    def test_legacy_flat_hex_manifest(self):
        manifest = {
            "files": {
                "a.txt": hashlib.sha256((self.home / "a.txt").read_bytes()).hexdigest(),
                "b.txt": hashlib.sha256((self.home / "b.txt").read_bytes()).hexdigest(),
            }
        }
        (self.home / "INTEGRITY.json").write_text(json.dumps(manifest))
        proc = _run(self.home)
        self.assertEqual(proc.returncode, 0, proc.stderr)
        sweep = _latest_report(self.home)["sweep"]
        self.assertEqual(sweep["ok"], 2)

    def test_non_git_home_exits_2(self):
        with tempfile.TemporaryDirectory() as bare:
            proc = _run(Path(bare))
        self.assertEqual(proc.returncode, 2)


if __name__ == "__main__":
    unittest.main()
