#!/usr/bin/env python3
"""Tests for ciel_laya_serve — the laya-serve registry-overlay entrypoint.

Validation logic is exercised with a stubbed ``laya.router`` module so the
suite runs without the system1 venv; one auto-route regression test uses
the real Router when laya is importable and skips otherwise.
"""

import json
import os
import stat
import sys
import tempfile
import types
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INIT_SYSTEM1 = ROOT / "ciel.skill" / "init" / "system1"
sys.path.insert(0, str(INIT_SYSTEM1))

import ciel_laya_serve as shim


def _stub_laya_router():
    """Install a minimal laya.router stub with the real registry shape."""
    router = types.ModuleType("laya.router")
    router.DEFAULT_MODELS = {
        "english": ("convaiinnovations/laya", None),
        "multilingual": ("convaiinnovations/laya", "multilingual"),
        "typed-decisions": ("convaiinnovations/laya", "typed-decisions"),
    }
    router._ALIASES = {"en": "english", "typed": "typed-decisions"}
    laya = types.ModuleType("laya")
    laya.router = router
    return {"laya": laya, "laya.router": router}


class OverlayTestBase(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.dir = Path(self.tmp.name)
        self.ckpt = self.dir / "ckpt"
        self.ckpt.mkdir()
        self.overlay = self.dir / "models.local.json"
        self._saved_env = os.environ.get("LAYA_SHA256_DIGESTS")
        self.addCleanup(self._restore_env)
        self._stubs = _stub_laya_router()
        self._saved_mods = {k: sys.modules.get(k) for k in self._stubs}
        sys.modules.update(self._stubs)
        self.addCleanup(self._restore_mods)

    def _restore_env(self):
        if self._saved_env is None:
            os.environ.pop("LAYA_SHA256_DIGESTS", None)
        else:
            os.environ["LAYA_SHA256_DIGESTS"] = self._saved_env

    def _restore_mods(self):
        for k, v in self._saved_mods.items():
            if v is None:
                sys.modules.pop(k, None)
            else:
                sys.modules[k] = v

    def write_overlay(self, data):
        self.overlay.write_text(json.dumps(data))
        self.overlay.chmod(0o600)

    def good_entry(self):
        return {"path": str(self.ckpt),
                "sha256": {"model.safetensors": "ab" * 32}}


class TestOverlayLoading(OverlayTestBase):
    def test_absent_file_returns_empty(self):
        self.assertEqual(shim.load_overlay(self.dir / "missing.json"), {})

    def test_valid_entry_loads(self):
        self.write_overlay({"ciel-context-lora": self.good_entry()})
        entries = shim.load_overlay(self.overlay)
        self.assertEqual(entries["ciel-context-lora"]["path"], str(self.ckpt))

    def test_group_writable_refused(self):
        self.write_overlay({"x-model": self.good_entry()})
        self.overlay.chmod(0o640)
        with self.assertRaises(SystemExit):
            shim.load_overlay(self.overlay)

    def test_world_writable_refused(self):
        self.write_overlay({"x-model": self.good_entry()})
        self.overlay.chmod(0o606)
        with self.assertRaises(SystemExit):
            shim.load_overlay(self.overlay)

    def test_malformed_json_refused(self):
        self.overlay.write_text("{not json")
        self.overlay.chmod(0o600)
        with self.assertRaises(SystemExit):
            shim.load_overlay(self.overlay)

    def test_non_object_top_level_refused(self):
        self.write_overlay(["ciel-context-lora"])
        with self.assertRaises(SystemExit):
            shim.load_overlay(self.overlay)

    def test_invalid_name_refused(self):
        self.write_overlay({"Bad Name!": self.good_entry()})
        with self.assertRaises(SystemExit):
            shim.load_overlay(self.overlay)

    def test_collision_with_default_refused(self):
        self.write_overlay({"english": self.good_entry()})
        with self.assertRaises(SystemExit):
            shim.load_overlay(self.overlay)

    def test_collision_with_alias_refused(self):
        self.write_overlay({"en": self.good_entry()})
        with self.assertRaises(SystemExit):
            shim.load_overlay(self.overlay)

    def test_relative_path_refused(self):
        e = self.good_entry()
        e["path"] = "relative/ckpt"
        self.write_overlay({"x-model": e})
        with self.assertRaises(SystemExit):
            shim.load_overlay(self.overlay)

    def test_missing_checkpoint_dir_refused(self):
        e = self.good_entry()
        e["path"] = str(self.dir / "nonexistent")
        self.write_overlay({"x-model": e})
        with self.assertRaises(SystemExit):
            shim.load_overlay(self.overlay)

    def test_missing_sha256_refused(self):
        e = self.good_entry()
        del e["sha256"]
        self.write_overlay({"x-model": e})
        with self.assertRaises(SystemExit):
            shim.load_overlay(self.overlay)

    def test_empty_sha256_refused(self):
        e = self.good_entry()
        e["sha256"] = {}
        self.write_overlay({"x-model": e})
        with self.assertRaises(SystemExit):
            shim.load_overlay(self.overlay)

    def test_non_dict_spec_refused(self):
        self.write_overlay({"x-model": "just-a-path"})
        with self.assertRaises(SystemExit):
            shim.load_overlay(self.overlay)


class TestApplyOverlay(OverlayTestBase):
    def test_registers_in_default_models(self):
        entries = {"ciel-context-lora": self.good_entry()}
        shim.apply_overlay(entries)
        self.assertEqual(
            sys.modules["laya.router"].DEFAULT_MODELS["ciel-context-lora"],
            (str(self.ckpt), None))

    def test_digests_merge_into_env(self):
        os.environ["LAYA_SHA256_DIGESTS"] = json.dumps(
            {"english": {"model.safetensors": "ff" * 32}})
        entries = {"ciel-context-lora": self.good_entry()}
        shim.apply_overlay(entries)
        merged = json.loads(os.environ["LAYA_SHA256_DIGESTS"])
        self.assertEqual(merged["english"]["model.safetensors"], "ff" * 32)
        self.assertEqual(merged["ciel-context-lora"]
                         ["model.safetensors"], "ab" * 32)

    def test_flat_existing_digests_not_merged(self):
        # a flat {artifact: digest} map is laya.revisions' own reading —
        # the overlay does not rewrite it
        os.environ["LAYA_SHA256_DIGESTS"] = json.dumps(
            {"model.safetensors": "cc" * 32})
        shim.apply_overlay({"x-model": self.good_entry()})
        merged = json.loads(os.environ["LAYA_SHA256_DIGESTS"])
        self.assertEqual(merged["x-model"]["model.safetensors"], "ab" * 32)


class TestOverlayPath(unittest.TestCase):
    def test_env_override(self):
        os.environ["CIEL_LAYA_MODELS_FILE"] = "/tmp/custom-overlay.json"
        try:
            self.assertEqual(shim._overlay_path(),
                             Path("/tmp/custom-overlay.json"))
        finally:
            del os.environ["CIEL_LAYA_MODELS_FILE"]

    def test_default_under_ciel_home(self):
        os.environ.pop("CIEL_LAYA_MODELS_FILE", None)
        saved = os.environ.get("CIEL_HOME")
        os.environ["CIEL_HOME"] = "/tmp/fake-ciel-home"
        try:
            self.assertEqual(shim._overlay_path(),
                             Path("/tmp/fake-ciel-home/system1/models.local.json"))
        finally:
            if saved is None:
                del os.environ["CIEL_HOME"]
            else:
                os.environ["CIEL_HOME"] = saved


class TestAutoRouteExclusion(unittest.TestCase):
    """m4 regression pin: overlay names must never be auto-selected."""

    def test_generic_state_never_routes_to_overlay(self):
        try:
            import laya.router as lr
        except ImportError:
            self.skipTest("laya not importable (system1 venv absent)")
        ckpt = tempfile.mkdtemp()
        lr.DEFAULT_MODELS["test-overlay-ckpt"] = (ckpt, None)
        try:
            router = lr.Router(default="english", auto_task_detection=True)
            decision = router.route(state={"task": "write me a poem about trees"})
            self.assertEqual(decision["model"], "english")
            decision = router.route(state={"tool": "exec", "command": "ls"})
            self.assertNotEqual(decision["model"], "test-overlay-ckpt")
        finally:
            del lr.DEFAULT_MODELS["test-overlay-ckpt"]


if __name__ == "__main__":
    unittest.main()
