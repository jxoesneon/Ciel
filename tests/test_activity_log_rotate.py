"""Tests for ciel.skill/init/hooks/lib/activity_log_rotate.py."""

import gzip
import importlib.util
import json
import subprocess
import tempfile
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path

MODULE_PATH = (
    Path(__file__).resolve().parent.parent
    / "ciel.skill" / "init" / "hooks" / "lib" / "activity_log_rotate.py"
)


def _load_module():
    spec = importlib.util.spec_from_file_location("activity_log_rotate", MODULE_PATH)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _utcnow():
    return datetime.now(timezone.utc)


def _decompress(path: Path) -> bytes:
    if path.suffix == ".zst":
        try:
            from compression import zstd
            return zstd.decompress(path.read_bytes())
        except ImportError:
            return subprocess.run(
                ["zstd", "-d", "-c", str(path)],
                capture_output=True, check=True,
            ).stdout
    return gzip.decompress(path.read_bytes())


class RotateTests(unittest.TestCase):
    def setUp(self):
        self.mod = _load_module()
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.home = Path(self.tmp.name)
        self.log = self.home / "activity.log"

    def _write_log(self, lines):
        self.log.write_text("\n".join(lines) + "\n", encoding="utf-8")

    def _today_entry(self):
        return json.dumps({"ts": _utcnow().isoformat(), "kind": "event"})

    def _archives(self):
        archive = self.home / "archive" / "logs"
        return sorted(archive.glob("activity-*.log.*")) if archive.is_dir() else []

    def test_small_log_today_no_rotation(self):
        entry = self._today_entry()
        self._write_log([entry])
        marker = self.mod.rotate(self.home, _utcnow())
        self.assertIsNone(marker)
        self.assertFalse((self.home / "archive" / "logs").exists())
        self.assertEqual(self.log.read_text().strip(), entry)

    def test_size_trigger_rotates_and_compresses(self):
        self.mod.MAX_BYTES = 1024
        payload = self._today_entry() + " " + ("x" * 2048)
        self._write_log([payload])
        original = self.log.read_bytes()

        marker = self.mod.rotate(self.home, _utcnow())

        self.assertIsNotNone(marker)
        self.assertEqual(marker["reason"], "size")
        self.assertEqual(marker["kind"], "sweep")
        self.assertEqual(marker["op"], "log_rotate")
        self.assertEqual(marker["bytes"], len(original))

        archives = self._archives()
        self.assertEqual(len(archives), 1)
        self.assertRegex(archives[0].name, r"\.log\.(zst|gz)$")
        self.assertEqual(_decompress(archives[0]), original)

        fresh_lines = self.log.read_text().splitlines()
        self.assertEqual(len(fresh_lines), 1)
        self.assertEqual(json.loads(fresh_lines[0]), marker)

    def test_daily_trigger_on_stale_first_line(self):
        yesterday = (_utcnow() - timedelta(days=1)).isoformat()
        self._write_log([json.dumps({"ts": yesterday, "kind": "event"})])

        marker = self.mod.rotate(self.home, _utcnow())

        self.assertIsNotNone(marker)
        self.assertEqual(marker["reason"], "daily")

    def test_retention_prunes_old_archives(self):
        self._write_log([self._today_entry()])
        archive = self.home / "archive" / "logs"
        archive.mkdir(parents=True)
        old = archive / "activity-20200101-000000.log.zst"
        old.write_bytes(b"stale")
        recent_name = _utcnow().strftime("activity-%Y%m%d-000000.log.zst")
        recent = archive / recent_name
        recent.write_bytes(b"fresh")

        self.mod.MAX_BYTES = 1  # force rotation
        self.mod.rotate(self.home, _utcnow())

        self.assertFalse(old.exists())
        self.assertTrue(recent.exists())

    def test_non_json_first_line_small_no_rotation(self):
        self._write_log(["not json at all"])
        self.assertIsNone(self.mod.rotate(self.home, _utcnow()))
        self.assertEqual(self._archives(), [])

    def test_missing_log_returns_none(self):
        self.assertIsNone(self.mod.rotate(self.home, _utcnow()))
        self.assertFalse((self.home / "archive").exists())


if __name__ == "__main__":
    unittest.main()
