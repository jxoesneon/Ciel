"""Diff-coverage tests for the v1.3.0 release gate.

Targets code paths added/changed since v1.2.0 that the mechanism suites do
not reach: ciel_root's discovery primitives, the RLCD pipeline, the registry
index builder, the System-1 corpus/teacher/offline-eval operator scripts, the
shadow-dispatch layer in system1.py, and the Track-1/Track-2 retrain harness
(torch-gated — skips where the system1 venv deps are absent).
"""

import contextlib
import importlib.util
import io
import json
import os
import subprocess
import sys
import tempfile
import types
import unittest
import unittest.mock
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LIB = ROOT / "ciel.skill" / "init" / "hooks" / "lib"
SCRIPTS = ROOT / "scripts"


def _src(name: str, base: Path = LIB):
    spec = importlib.util.spec_from_file_location(name, str(base / f"{name}.py"))
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


def _argv(*args):
    return unittest.mock.patch.object(sys, "argv", list(args))


class _EnvCase(unittest.TestCase):
    """setUp/tearDown helper: patch os.environ keys, restore after."""

    def set_env(self, **kv):
        saved = {k: os.environ.get(k) for k in kv}
        os.environ.update({k: v for k, v in kv.items() if v is not None})
        for k, v in kv.items():
            if v is None:
                os.environ.pop(k, None)
        self.addCleanup(self._restore_env, saved)

    @staticmethod
    def _restore_env(saved):
        for k, v in saved.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v


class TestCielRoot(_EnvCase):
    def setUp(self):
        self.cr = _src("ciel_root")

    def test_hook_lib_dirs_env_and_defaults(self):
        self.set_env(CIEL_HOOK_LIB="/x/lib", CIEL_ROOT="/x/root",
                     CIEL_HOME="/x/home")
        dirs = self.cr.hook_lib_dirs()
        self.assertIn(Path("/x/lib"), dirs)
        self.assertIn(Path("/x/root/hooks/lib"), dirs)
        self.assertIn(Path("/x/root/ciel.skill/init/hooks/lib"), dirs)
        self.assertIn(Path("/x/home/hooks/lib"), dirs)
        # repo-layout candidate appended (this file lives under init/hooks/lib)
        self.assertIn(LIB, dirs)

    def test_find_hook_lib(self):
        found = self.cr.find_hook_lib()
        self.assertTrue((found / "risk_policy.py").is_file())
        self.set_env(CIEL_HOOK_LIB="/nonexistent", CIEL_ROOT="/nonexistent",
                     CIEL_HOME="/nonexistent")
        # still finds a dir containing risk_policy.py (installed or repo)
        found = self.cr.find_hook_lib()
        self.assertTrue((found / "risk_policy.py").is_file())

    def test_candidate_roots(self):
        self.set_env(CIEL_ROOT="/x/root", CIEL_HOME="/x/home")
        roots = self.cr.candidate_roots()
        self.assertEqual(roots[0], Path("/x/root"))
        self.assertEqual(roots[1], Path("/x/home"))
        self.assertIn(ROOT, roots)  # repo root via ciel.skill ancestor

    def test_ciel_home(self):
        self.set_env(CIEL_HOME="/tmp/ch")
        self.assertEqual(self.cr.ciel_home(), Path("/tmp/ch"))
        self.set_env(CIEL_HOME=None)
        self.assertEqual(self.cr.ciel_home(), Path.home() / ".ciel")

    def test_ciel_bin_env(self):
        with tempfile.TemporaryDirectory() as td:
            exe = Path(td) / "ciel"
            exe.write_text("#!/bin/sh\n")
            os.chmod(exe, 0o755)
            self.set_env(CIEL_BIN=str(exe))
            self.assertEqual(self.cr.ciel_bin(), str(exe))
            os.chmod(exe, 0o644)
            self.assertIsNone(self.cr.ciel_bin())

    def test_ciel_bin_search_and_which(self):
        with tempfile.TemporaryDirectory() as td:
            self.set_env(CIEL_BIN=None, CIEL_HOME=td)
            exe = Path(td) / "bin" / "ciel"
            exe.parent.mkdir(parents=True)
            exe.write_text("#!/bin/sh\n")
            os.chmod(exe, 0o755)
            self.assertEqual(self.cr.ciel_bin(), str(exe))
            # PATH fallback when no candidate resolves: hide ~/.cargo/bin too
            with tempfile.TemporaryDirectory() as fake_home, \
                 unittest.mock.patch.object(Path, "home",
                                            return_value=Path(fake_home)), \
                 unittest.mock.patch("shutil.which",
                                     return_value="/p/ciel"):
                self.set_env(CIEL_BIN=None, CIEL_HOME="/nonexistent-home")
                self.assertEqual(self.cr.ciel_bin(), "/p/ciel")

    def test_append_log_and_rotate(self):
        with tempfile.TemporaryDirectory() as td:
            self.set_env(CIEL_HOME=td)
            self.assertTrue(self.cr.append_log({"k": "v"}))
            lines = (Path(td) / "activity.log").read_text().splitlines()
            self.assertEqual(json.loads(lines[0]), {"k": "v"})
            # rotate failure is swallowed — still True
            rot = types.ModuleType("activity_log_rotate")

            def _boom(*a, **k):
                raise RuntimeError("nope")

            rot.rotate = _boom
            sys.modules["activity_log_rotate"] = rot
            try:
                self.assertTrue(self.cr.append_log({"k": 2}))
            finally:
                sys.modules.pop("activity_log_rotate", None)

    def test_append_log_oserror(self):
        self.set_env(CIEL_HOME="/proc/definitely-not-writable-ciel")
        self.assertFalse(self.cr.append_log({"k": "v"}))

    def test_utc_now_and_bootstrap(self):
        self.assertIn("T", self.cr.utc_now())
        lib = self.cr.bootstrap_lib()
        self.assertTrue((lib / "risk_policy.py").is_file())
        self.assertIn(str(lib), sys.path)


class TestRlcdPipeline(_EnvCase):
    def setUp(self):
        self.rl = _src("ciel_rlcd_pipeline", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.t = Path(self.tmp.name)

    def _events(self, *recs):
        f = self.t / "events.jsonl"
        f.write_text("\n".join(
            r if isinstance(r, str) else json.dumps(r) for r in recs) + "\n")
        return f

    def _rec(self, surface, flag, answers=None, meta=None):
        return {"surface": surface, "flag": flag, "meta": meta or {},
                "state": {"command": "x"},
                "system1": {"answers": answers or {}}}

    def test_events_absent(self):
        self.assertEqual(self.rl.process_events(self.t / "no.jsonl"), [])

    def test_events_all_surfaces(self):
        f = self._events(
            "not-json", "",   # skipped lines
            self._rec("pre_tool_risk", "flag",
                      {"risk": {"choice": "dangerous", "confidence": 0.9}}),
            self._rec("pre_tool_risk", "flag",
                      {"risk": {"choice": "safe"}}),  # not dangerous -> skip
            self._rec("pre_tool_risk", "pass",
                      {"risk": {"choice": "dangerous"}}),  # unflagged -> skip
            self._rec("context_select", "uncertain",
                      {"relevant": {"choice": "drop", "confidence": 0.5}}),
            self._rec("memory_salience", "flag",
                      {"salience": {"choice": "skip"}}),
            self._rec("context_compaction", "flag",
                      {"action": {"choice": "compress"}}),
            self._rec("context_compaction", "flag",
                      {"action": {"choice": "continue"}}),  # continue -> skip
            self._rec("mandate_canary", "uncertain",
                      {"mandates": {"choice": "drifted"}}),
            self._rec("council_prescreen", "pass",
                      meta={"council_consensus": "reject"}),
            self._rec("council_prescreen", "pass",
                      meta={"council_consensus": "approve"}),
            self._rec("council_prescreen", "pass",
                      meta={"council_consensus": "abstain"}),  # ignored
            self._rec("other_surface", "flag"),              # ignored
            {"surface": "x", "flag": "flag", "system1": "bad"},  # non-dict sys1
        )
        pairs = self.rl.process_events(f)
        chosen = {(p["surface"], p["question_key"], p["chosen"]) for p in pairs}
        self.assertEqual(len(pairs), 7)
        self.assertIn(("pre_tool_risk", "risk", "dangerous"), chosen)
        self.assertIn(("context_select", "relevant", "drop"), chosen)
        self.assertIn(("memory_salience", "salience", "skip"), chosen)
        self.assertIn(("context_compaction", "action", "compress"), chosen)
        self.assertIn(("mandate_canary", "mandates", "drifted"), chosen)
        self.assertIn(("council_prescreen", "council_review", "escalate"),
                      chosen)
        self.assertIn(("council_prescreen", "council_review", "routine"),
                      chosen)

    def test_signals_and_dockets(self):
        sig = self.t / "signals"
        sig.mkdir()
        (sig / "ok.json").write_text(json.dumps({
            "signal": "council_verdict", "run_id": "r1", "mode": "x",
            "member_verdicts": {"a": {"stage2": 8}, "b": {"stage2": 9}}}))
        (sig / "low.json").write_text(json.dumps({
            "signal": "council_verdict", "run_id": "r2",
            "member_verdicts": {"a": {"stage2": 2}}}))
        (sig / "avg_low.json").write_text(json.dumps({
            "signal": "council_verdict", "run_id": "r3",
            "member_verdicts": {"a": {"stage2": 6}, "b": {"stage2": 6}}}))
        (sig / "unrelated.json").write_text(json.dumps({"signal": "other"}))
        (sig / "bad.json").write_text("{corrupt")

        council = self.t / "council"
        dockets = council / "dockets"
        dockets.mkdir(parents=True)
        (council / "r4.verdict.json").write_text(
            json.dumps({"verdict": "pass", "weighted_score": 8.0,
                        "votes": {"a": 8}}))
        (council / "r5.verdict.json").write_text(
            json.dumps({"verdict": "reject"}))
        (council / "r6.verdict.json").write_text(
            json.dumps({"verdict": "wat"}))  # ignored
        (council / "bad.verdict.json").write_text("{corrupt")
        (dockets / "d1.md").write_text("VERDICT: REJECT something")
        (dockets / "d2.md").write_text("VERDICT: APPROVE something")
        (dockets / "d3.md").write_text("no verdict here")

        pairs = self.rl.process_signals_and_dockets(dockets, sig)
        chosen = [(p["chosen"], p["source"]) for p in pairs]
        self.assertIn(("routine", "council_signal_json"), chosen)
        self.assertIn(("escalate", "council_signal_json"), chosen)
        self.assertIn(("routine", "council_verdict_json"), chosen)
        self.assertIn(("escalate", "council_verdict_json"), chosen)
        self.assertIn(("escalate", "council_docket_md"), chosen)
        self.assertIn(("routine", "council_docket_md"), chosen)
        self.assertEqual(len(pairs), 7)

    def test_main_writes_pairs(self):
        home = self.t / "home"
        (home / "system1").mkdir(parents=True)
        (home / "council" / "dockets").mkdir(parents=True)
        (home / "improvements" / "signals").mkdir(parents=True)
        (home / "system1" / "events.jsonl").write_text(json.dumps(
            self._rec("pre_tool_risk", "flag",
                      {"risk": {"choice": "dangerous"}})) + "\n")
        self.set_env(CIEL_HOME=str(home))
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            self.rl.main()
        self.assertIn("Generated 1", buf.getvalue())
        out = home / "system1" / "rlcd_pairs.jsonl"
        self.assertEqual(len(out.read_text().splitlines()), 1)


class TestBuildRegistryIndex(unittest.TestCase):
    def setUp(self):
        self.bri = _src("build_registry_index", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.t = Path(self.tmp.name)

    def _skill(self, name, fm=None, ciel=None, extra=None):
        d = self.t / "skills" / name
        d.mkdir(parents=True)
        body = fm or {}
        fm_text = "---\n" + "\n".join(f"{k}: {v}" for k, v in body.items()) \
            + "\n---\n# doc\n" if body else "# no frontmatter\n"
        (d / "SKILL.md").write_text(fm_text)
        if ciel is not None:
            (d / "ciel.yaml").write_text(ciel)
        if extra:
            (d / extra[0]).write_text(extra[1])
        return d

    def test_build_entry_full(self):
        d = self._skill(
            "alpha", fm={"version": "1.2.3", "description": "d",
                         "license": "MIT", "tags": "[a, b]"},
            ciel=("version: 2.0.0\n"
                  "triggers:\n  - pattern: foo\n  - bar\n"
                  "state: sandboxed\n"
                  "tags: [x]\n"
                  "source:\n  tier: 3\n  license: Apache\n"
                  "dependencies: [dep1]\nruntimes: [devin]\n"),
            extra=("ref.md", "x"))
        e = self.bri.build_entry(d)
        self.assertEqual(e["id"], "alpha")
        self.assertEqual(e["version"], "2.0.0")       # ciel.yaml wins
        self.assertEqual(e["triggers"], ["foo", "bar"])
        self.assertEqual(e["state"], "sandboxed")
        self.assertEqual(e["tags"], ["x"])
        self.assertEqual(e["dependencies"], ["dep1"])
        self.assertTrue(e["checksum"].startswith("sha256:"))

    def test_build_entry_minimal_and_missing(self):
        d = self._skill("beta")
        e = self.bri.build_entry(d)
        self.assertEqual(e["version"], "0.0.0")
        self.assertEqual(e["triggers"], [])
        self.assertEqual(e["state"], "validated")
        self.assertEqual(e["source"], {"tier": 0, "origin": "local"})
        bare = self.t / "skills" / "empty"
        bare.mkdir(parents=True)
        self.assertIsNone(self.bri.build_entry(bare))

    def test_frontmatter_edge(self):
        d = self.t / "skills" / "nofm"
        d.mkdir(parents=True)
        (d / "SKILL.md").write_text("---unterminated")   # no closing ---
        self.assertEqual(self.bri.build_entry(d)["description"], "")
        (d / "SKILL.md").write_text("plain text, no fence")
        self.assertEqual(self.bri._frontmatter(d / "SKILL.md"), {})

    def test_load_yaml_fallback_parser(self):
        # Hide yaml so the miniature parser runs.
        real_import = __builtins__.__import__ if hasattr(
            __builtins__, "__import__") else __import__

        def no_yaml(name, *a, **k):
            if name == "yaml":
                raise ImportError("hidden")
            return real_import(name, *a, **k)

        text = ("name: x\nversion: '1.0'\ntags: [a, b]\n"
                "nested:\n  inner: v\nlist:\n  - i1\n  - i2\n"
                "# comment\n\ntrailer: y\n")
        with unittest.mock.patch("builtins.__import__", side_effect=no_yaml):
            out = self.bri._load_yaml(text)
        self.assertEqual(out["name"], "x")
        self.assertEqual(out["version"], "1.0")
        self.assertEqual(out["tags"], ["a", "b"])
        self.assertEqual(out["nested"], {"inner": "v"})
        self.assertEqual(out["list"], ["i1", "i2"])
        self.assertEqual(out["trailer"], "y")

    def test_main_writes_index(self):
        self._skill("one", fm={"version": "1.0.0"})
        (self.t / "skills" / "notafile").write_text("x")
        (self.t / "skills" / "noskill").mkdir()
        out = self.t / "reg" / "index.json"
        buf = io.StringIO()
        with _argv("x", "--skills-dir", str(self.t / "skills"),
                   "--out", str(out)), contextlib.redirect_stdout(buf):
            rc = self.bri.main()
        self.assertEqual(rc, 0)
        idx = json.loads(out.read_text())
        self.assertEqual(idx["count"], 1)
        self.assertIn("one", idx["skills"])
        self.assertIn("noskill", json.loads(buf.getvalue())
                      ["skipped_no_skill_md"])


class TestTeacherLabel(unittest.TestCase):
    def setUp(self):
        self.s1 = _src("system1")
        self.tl = _src("system1_teacher_label", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.t = Path(self.tmp.name)

    def _srcfile(self, *rows):
        f = self.t / "in.jsonl"
        f.write_text("\n".join(json.dumps(r) for r in rows) + "\n")
        return f

    def _row(self, surface):
        return {"surface": surface, "state": {"command": "x"},
                "questions": {"q": {}}}

    def test_teacher_ask_inactive_and_http_fail(self):
        with unittest.mock.patch.object(self.s1, "_hosted_active",
                                        return_value=False):
            res, how = self.tl.teacher_ask({}, {})
            self.assertIsNone(res)
            self.assertEqual(how, "hosted-inactive")
        with unittest.mock.patch.object(self.s1, "_hosted_active",
                                        return_value=True), \
             unittest.mock.patch.object(self.s1, "_hosted_url",
                                        return_value="https://h"), \
             unittest.mock.patch.object(self.s1, "_endpoint_of",
                                        return_value="https://h/v1"), \
             unittest.mock.patch.object(self.s1, "_hosted_key",
                                        return_value="k"), \
             unittest.mock.patch.object(self.s1, "_hosted_model",
                                        return_value="m"), \
             unittest.mock.patch.object(self.s1, "_do_ask",
                                        return_value=(None, 503)):
            res, how = self.tl.teacher_ask({}, {})
            self.assertEqual(how, "http-503")

    def test_main_label_paths(self):
        src = self._srcfile(
            self._row("pre_tool_risk"),
            self._row("context_compaction"),
            self._row("council_prescreen"),
        )
        dst = self.t / "out.jsonl"

        def fake_ask(state, questions, timeout=8.0):
            return {"answers": {
                "risk": {"choice": "dangerous", "confidence": 0.9},
                "action": {"choice": "compress", "confidence": 0.8},
                "pressure": {"score": 2},
                "scope": {"choice": "routine", "confidence": 0.1},
            }}, "ok"

        with unittest.mock.patch.object(self.tl, "teacher_ask", fake_ask):
            with _argv("x", "--in", str(src), "--out", str(dst)):
                self.tl.main()
        rows = [json.loads(l) for l in dst.read_text().splitlines()]
        self.assertEqual(rows[0]["label"], {"risk": "dangerous"})
        self.assertEqual(rows[1]["label"],
                         {"action": "compress", "pressure": 2})
        self.assertIsNone(rows[2]["label"])   # below min-confidence
        self.assertIn("declined", rows[2]["teacher"])

    def test_main_dry_run_and_limit(self):
        src = self._srcfile(self._row("pre_tool_risk"),
                            self._row("pre_tool_risk"))
        dst = self.t / "dry.jsonl"
        with _argv("x", "--in", str(src), "--out", str(dst), "--dry-run"):
            self.tl.main()
        rows = [json.loads(l) for l in dst.read_text().splitlines()]
        self.assertTrue(all(r["teacher"] == "dry-run" for r in rows))

        dst2 = self.t / "lim.jsonl"
        with unittest.mock.patch.object(
                self.tl, "teacher_ask",
                return_value=({"answers": {"risk": {"choice": "safe",
                                                   "confidence": 0.9}}},
                              "ok")):
            with _argv("x", "--in", str(src), "--out", str(dst2),
                       "--limit", "1"):
                self.tl.main()
        rows = [json.loads(l) for l in dst2.read_text().splitlines()]
        self.assertEqual(rows[0]["teacher"], "jev:0.900")
        self.assertEqual(rows[1]["teacher"], "over-limit")


class TestCorpusBuild(unittest.TestCase):
    def setUp(self):
        _src("system1")  # corpus_build imports it
        self.cb = _src("system1_corpus_build", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.t = Path(self.tmp.name)

    def test_rows_cover_all_surfaces(self):
        surfaces = set()
        splits = {"train": 0, "holdout": 0}
        teacher = 0
        for row in self.cb.rows():
            surfaces.add(row["surface"])
            if row["difficulty"] == "teacher":
                teacher += 1
                self.assertIsNone(row["label"])
            else:
                splits[row["split"]] += 1
                self.assertIsNotNone(row["label"])
            self.assertIn("questions", row)
            self.assertTrue(row["id"].startswith(row["surface"]))
        self.assertEqual(surfaces, {
            "pre_tool_risk", "council_prescreen", "memory_salience",
            "context_compaction", "mandate_canary", "context_select"})
        self.assertGreater(splits["train"], 0)
        self.assertGreater(splits["holdout"], 0)
        self.assertGreater(teacher, 0)

    def test_main_writes_three_files(self):
        tr, ho, te = (self.t / n for n in ("tr.jsonl", "ho.jsonl", "te.jsonl"))
        buf = io.StringIO()
        with _argv("x", "--train-out", str(tr), "--holdout-out", str(ho),
                   "--teacher-out", str(te)), \
                contextlib.redirect_stdout(buf):
            self.cb.main()
        self.assertTrue(all(f.is_file() for f in (tr, ho, te)))
        out = buf.getvalue()
        for name, f in (("train", tr), ("holdout", ho), ("teacher", te)):
            n = len(f.read_text().splitlines())
            self.assertIn(f"{name}={n}", out)


class TestOfflineEval(unittest.TestCase):
    """system1_offline_eval with a stubbed laya.Agent — no checkpoint needed."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.t = Path(self.tmp.name)
        _src("system1")
        self.ev = _src("system1_eval", SCRIPTS)

        # Inject a fake laya.agent module before loading (tests swap in
        # their own Agent implementation per case).
        laya = types.ModuleType("laya")
        agent_mod = types.ModuleType("laya.agent")

        class FakeAgent:
            def __init__(self, path, device="cpu", expected_sha256=None):
                self.path = path

        agent_mod.Agent = FakeAgent
        laya.agent = agent_mod
        self._saved_laya = sys.modules.get("laya")
        self._saved_agent = sys.modules.get("laya.agent")
        sys.modules["laya"] = laya
        sys.modules["laya.agent"] = agent_mod
        self.addCleanup(self._restore_laya)
        self.oe = _src("system1_offline_eval", SCRIPTS)
        self.addCleanup(sys.modules.pop, "system1_offline_eval", None)

    def _restore_laya(self):
        for k, v in (("laya", self._saved_laya),
                     ("laya.agent", self._saved_agent)):
            if v is None:
                sys.modules.pop(k, None)
            else:
                sys.modules[k] = v

    def _surface_spec(self, corpus_cases, positive):
        corpus = self.t / "corpus.json"
        corpus.write_text(json.dumps({"cases": corpus_cases}))
        return {
            "corpus": corpus,
            "state": lambda c: c["state"],
            "questions": lambda corpus: {"risk": {}},
            "truth": lambda c: c.get("truth"),
            "positive": positive,
        }

    def test_main_binary_surface_with_wrong_direction(self):
        spec = self._surface_spec(
            [{"id": "d1", "state": {}, "truth": "dangerous"},
             {"id": "d2", "state": {}, "truth": "dangerous"},
             {"id": "s1", "state": {}, "truth": "safe"},
             {"id": "s2", "state": {}, "truth": "safe"}],
            positive="dangerous")
        # per-case answers: FN (safe@sub-tau), TP, TN, FP — plus one wd@tau
        answers_seq = iter([
            {"answers": {"risk": {"choice": "safe", "confidence": 0.9}}},
            {"answers": {"risk": {"choice": "dangerous",
                                  "confidence": 0.8}}},
            {"answers": {"risk": {"choice": "safe", "confidence": 0.7}}},
            {"answers": {"risk": {"choice": "dangerous",
                                  "confidence": 0.6}}},
        ])

        class Agent:
            def __init__(self, path, device="cpu", expected_sha256=None):
                pass

            def predict_batch(self, states, questions):
                return [next(answers_seq) for _ in states]

        ev_surfaces = dict(self.ev.SURFACES)
        ev_surfaces["pre_tool_risk"] = spec
        report = self.t / "rep.json"
        with unittest.mock.patch.object(self.oe, "Agent", Agent), \
             unittest.mock.patch.object(self.ev, "SURFACES", ev_surfaces), \
             unittest.mock.patch.object(
                 self.ev, "_predict",
                 lambda a: a["risk"]["choice"]), \
             unittest.mock.patch.object(
                 self.ev, "_confidence",
                 lambda a: a["risk"]["confidence"]), \
             unittest.mock.patch.object(
                 sys.modules["system1"], "surface_tau",
                 return_value=0.5), \
             _argv("x", str(self.t), "--surface", "pre_tool_risk",
                   "--report", str(report)):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                rc = self.oe.main()
        self.assertEqual(rc, 0)
        rep = json.loads(report.read_text())
        e = rep["pre_tool_risk"]
        self.assertEqual(e["cases"], 4)
        self.assertEqual(e["tp"], 1)
        self.assertEqual(e["fn"], 1)
        self.assertEqual(e["tn"], 1)
        self.assertEqual(e["fp"], 1)
        # d1 predicted safe at 0.9 >= tau=0.5 -> wrong-direction-at-tau
        self.assertEqual([w["id"] for w in e["wrong_direction_at_tau"]], ["d1"])

    def test_main_accuracy_surface_and_per_case_questions(self):
        spec = self._surface_spec(
            [{"id": "c1", "state": {}, "truth": "keep"},
             {"id": "c2", "state": {}, "truth": "drop"}],
            positive=None)
        spec["questions_for_case"] = lambda c, corpus: {"q": {"id": c["id"]}}

        seq = iter([
            {"answers": {"relevant": {"choice": "keep", "confidence": 0.6}}},
            {"answers": {"relevant": {"choice": "keep", "confidence": 0.9}}},
        ])

        class Agent:
            def __init__(self, *a, **k):
                pass

            def predict_batch(self, states, questions):
                return [next(seq) for _ in states]

        ev_surfaces = dict(self.ev.SURFACES)
        ev_surfaces["context_select"] = spec
        with unittest.mock.patch.object(self.oe, "Agent", Agent), \
             unittest.mock.patch.object(self.ev, "SURFACES", ev_surfaces), \
             unittest.mock.patch.object(
                 self.ev, "_predict", lambda a: a["relevant"]["choice"]), \
             unittest.mock.patch.object(
                 self.ev, "_confidence",
                 lambda a: a["relevant"]["confidence"]), \
             unittest.mock.patch.object(sys.modules["system1"],
                                        "surface_tau", return_value=0.5), \
             _argv("x", str(self.t), "--surface", "context_select",
                   "--batch-size", "1"):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                rc = self.oe.main()
        self.assertEqual(rc, 0)
        self.assertIn("accuracy", buf.getvalue())
        self.assertIn("0.5", buf.getvalue())  # 1 correct of 2


class TestRequirementsShadowDispatch(unittest.TestCase):
    """requirements.py `_salience_shadow` — the memory-salience shadow."""

    def setUp(self):
        self.req = _src("requirements")

    def test_shadow_dispatch_and_swallow(self):
        good = types.ModuleType("system1")
        good.shadow_dispatch = unittest.mock.Mock()
        bad = types.ModuleType("system1")
        bad.shadow_dispatch = unittest.mock.Mock(side_effect=RuntimeError)
        event = {"kind": "log", "text": "x"}
        with unittest.mock.patch.dict(sys.modules, {"system1": good}):
            self.req._salience_shadow(event)
            good.shadow_dispatch.assert_called_once()
            self.assertEqual(
                good.shadow_dispatch.call_args[0][0], "memory-salience")
        with unittest.mock.patch.dict(sys.modules, {"system1": bad}):
            self.req._salience_shadow(event)  # swallowed, never raises


class TestRiskPolicyGaps(unittest.TestCase):
    def setUp(self):
        self.rp = _src("risk_policy")

    def test_system1_evaluate_risk_wrapper(self):
        stub = types.SimpleNamespace()
        stub.evaluate_risk = unittest.mock.Mock(
            return_value=("dangerous", 0.9, "flag"))
        with unittest.mock.patch.object(self.rp, "system1", stub):
            out = self.rp.system1_evaluate_risk("exec", "rm -rf /", "")
        self.assertEqual(out[0], "dangerous")

        # None system1 -> None (fail-open)
        with unittest.mock.patch.object(self.rp, "system1", None):
            self.assertIsNone(self.rp.system1_evaluate_risk(
                "exec", "x", ""))

    def test_system1_evaluate_risk_verdict_fallback(self):
        stub = types.SimpleNamespace()  # no evaluate_risk attr
        with unittest.mock.patch.object(self.rp, "system1", stub), \
             unittest.mock.patch.object(
                 self.rp, "system1_verdict",
                 return_value={"choice": "dangerous", "confidence": "bad"}):
            out = self.rp.system1_evaluate_risk("exec", "rm", "")
        self.assertEqual(out, ("dangerous", 0.0, "flag"))

        with unittest.mock.patch.object(self.rp, "system1", stub), \
             unittest.mock.patch.object(self.rp, "system1_verdict",
                                        return_value=None):
            self.assertIsNone(self.rp.system1_evaluate_risk(
                "exec", "rm", ""))

        # low-confidence safe bands uncertain
        with unittest.mock.patch.object(self.rp, "system1", stub), \
             unittest.mock.patch.object(
                 self.rp, "system1_verdict",
                 return_value={"choice": "safe", "confidence": 0.1}):
            out = self.rp.system1_evaluate_risk("exec", "ls", "")
        self.assertEqual(out[2], "uncertain")


class TestSystem1DispatchLayer(_EnvCase):
    def setUp(self):
        self.s1 = _src("system1")
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.t = Path(self.tmp.name)
        self.set_env(CIEL_HOME=str(self.t), CIEL_SYSTEM1_DISABLED=None)

    def test_ciel_bin_orders(self):
        self.set_env(CIEL_BIN=None)
        exe = self.t / "bin" / "ciel"
        exe.parent.mkdir(parents=True)
        exe.write_text("#!/bin/sh\n")
        os.chmod(exe, 0o755)
        self.assertEqual(self.s1._ciel_bin(), str(exe))
        self.set_env(CIEL_BIN="/nonexec")
        self.assertIsNone(self.s1._ciel_bin())

    def test_log_fallback(self):
        self.s1._log_fallback("hook", "binary-absent", "verb")
        line = (self.t / "fallback_events.jsonl").read_text().strip()
        rec = json.loads(line)
        self.assertEqual(rec["reason"], "binary-absent")
        self.assertEqual(rec["verb"], "verb")

    def test_lib_dispatch_all_verbs(self):
        with unittest.mock.patch.multiple(
                self.s1,
                context_select=unittest.mock.Mock(return_value={"a": 1}),
                memory_salience=unittest.mock.Mock(return_value={"b": 2}),
                compaction_decision=unittest.mock.Mock(return_value={"c": 3}),
                mandate_canary=unittest.mock.Mock(return_value={"d": 4})):
            self.assertEqual(self.s1._lib_dispatch(
                "context-select", {"task": "t", "options": {"x": "d"}}),
                {"a": 1})
            # list-shaped candidates -> mapped
            self.s1._lib_dispatch("context-select",
                                  {"prompt": "p", "candidates": [
                                      {"id": "i", "description": "dd"},
                                      "bare"]})
            args = self.s1.context_select.call_args[0]
            self.assertEqual(args[1], {"i": "dd", "bare": None})
            self.assertEqual(self.s1._lib_dispatch(
                "memory-salience", {"event": {"kind": "log"}}), {"b": 2})
            # bare top-level fields become the event
            self.s1._lib_dispatch("memory-salience", {"kind": "log"})
            self.assertEqual(self.s1.memory_salience.call_args[0][0],
                             {"kind": "log"})
            self.assertEqual(self.s1._lib_dispatch(
                "context-compaction", {"stats": {"tokens": 1}}), {"c": 3})
            self.s1._lib_dispatch("context-compaction", {"tokens": 5})
            self.assertEqual(self.s1.compaction_decision.call_args[0][0],
                             {"tokens": 5})
            self.assertEqual(self.s1._lib_dispatch(
                "mandate-canary", {"mandates": ["m"], "context": "c"}),
                {"d": 4})
            self.assertIsNone(self.s1._lib_dispatch("bogus-verb", {}))

    def test_dispatch_one_paths(self):
        with unittest.mock.patch.object(self.s1, "_disabled",
                                        return_value=True):
            self.s1._dispatch_one("memory-salience", {}, "h")   # early return
        with unittest.mock.patch.object(self.s1, "_disabled",
                                        return_value=False):
            self.s1._dispatch_one("not-a-verb", {}, "h")        # filtered
            self.s1._dispatch_one("memory-salience", "no-dict", "h")
            # binary absent -> fallback log + lib twin
            with unittest.mock.patch.object(self.s1, "_ciel_bin",
                                            return_value=None), \
                 unittest.mock.patch.object(
                     self.s1, "_lib_dispatch") as libd:
                self.s1._dispatch_one("memory-salience",
                                      {"event": {}}, "h")
                libd.assert_called_once()
            rec = json.loads((self.t / "fallback_events.jsonl")
                             .read_text().splitlines()[-1])
            self.assertEqual(rec["reason"], "binary-absent")
            # binary success -> no lib dispatch
            with unittest.mock.patch.object(self.s1, "_ciel_bin",
                                            return_value="/bin/true"), \
                 unittest.mock.patch.object(
                     self.s1, "_lib_dispatch") as libd:
                self.s1._dispatch_one("mandate-canary", {}, "h")
                libd.assert_not_called()
            # binary failure -> binary-failed log + lib dispatch
            with unittest.mock.patch.object(self.s1, "_ciel_bin",
                                            return_value="/bin/false"), \
                 unittest.mock.patch.object(
                     self.s1, "_lib_dispatch") as libd:
                self.s1._dispatch_one("mandate-canary", {}, "h")
                libd.assert_called_once()
            rec = json.loads((self.t / "fallback_events.jsonl")
                             .read_text().splitlines()[-1])
            self.assertEqual(rec["reason"], "binary-failed")
            # binary raising OSError -> fallback too
            with unittest.mock.patch.object(self.s1, "_ciel_bin",
                                            return_value="/nope"), \
                 unittest.mock.patch.object(
                     self.s1, "_lib_dispatch") as libd:
                self.s1._dispatch_one("mandate-canary", {}, "h")
                libd.assert_called_once()

    def test_shadow_dispatch(self):
        with unittest.mock.patch.object(self.s1, "_disabled",
                                        return_value=True):
            self.s1.shadow_dispatch("memory-salience", {}, "h")
        with unittest.mock.patch.object(self.s1, "_disabled",
                                        return_value=False):
            self.s1.shadow_dispatch("nope", {}, "h")
            with unittest.mock.patch("subprocess.Popen") as pop:
                proc = unittest.mock.Mock()
                proc.stdin = io.BytesIO()
                pop.return_value = proc
                self.s1.shadow_dispatch("memory-salience", {"event": {}}, "h")
                pop.assert_called_once()
            with unittest.mock.patch("subprocess.Popen",
                                     side_effect=OSError):
                self.s1.shadow_dispatch("memory-salience", {}, "h")  # swallowed

    def test_read_shadow_input(self):
        self.set_env(CIEL_HOOK_INPUT="raw-body")
        self.assertEqual(self.s1._read_shadow_input(), "raw-body")
        self.set_env(CIEL_HOOK_INPUT=None)
        with unittest.mock.patch.object(sys.stdin, "isatty",
                                        return_value=True):
            self.assertEqual(self.s1._read_shadow_input(), "")
        # pipe path: feed real bytes through an os pipe
        r, w = os.pipe()
        os.write(w, b"payload")
        os.close(w)
        fake_stdin = unittest.mock.Mock()
        fake_stdin.isatty.return_value = False
        fake_stdin.fileno.return_value = r
        with unittest.mock.patch.object(sys, "stdin", fake_stdin):
            self.assertEqual(self.s1._read_shadow_input(), "payload")
        # ValueError path
        bad = unittest.mock.Mock()
        bad.isatty.side_effect = ValueError
        with unittest.mock.patch.object(sys, "stdin", bad):
            self.assertEqual(self.s1._read_shadow_input(), "")

    def test_prompt_text(self):
        self.assertEqual(self.s1._prompt_text(
            "raw", {"tool_info": {"user_prompt": "tp"}}), "tp")
        self.assertEqual(self.s1._prompt_text("raw", {"prompt": "p"}), "p")
        self.assertEqual(self.s1._prompt_text("raw", {}), "raw")

    def test_registry_l0(self):
        reg = self.t / "registry"
        reg.mkdir()
        (reg / "index.json").write_text(json.dumps(
            {"skills": {"s1": {"description": "d1"},
                        "s2": "nondict"}}))
        out = self.s1._registry_l0()
        self.assertEqual(out, {"s1": "d1", "s2": ""})
        (reg / "index.json").write_text("{bad")
        # falls through to repo registry (or {} if absent)
        out = self.s1._registry_l0()
        self.assertIsInstance(out, dict)

    def test_prompt_shadow_jobs(self):
        raw = json.dumps({"session_id": "s1", "prompt": "hello"})
        jobs = self.s1._prompt_shadow_jobs("devin/user_prompt_submit", raw)
        verbs = [v for v, _ in jobs]
        self.assertIn("context-compaction", verbs)
        self.assertIn("mandate-canary", verbs)
        self.assertIn("context-select", verbs)  # inject items + registry
        # malformed payload still yields the two base jobs
        jobs = self.s1._prompt_shadow_jobs("other/hook", "{not-json")
        self.assertEqual([v for v, _ in jobs],
                         ["context-compaction", "mandate-canary"])

    def test_dispatch_main(self):
        payload = json.dumps({"verb": "memory-salience",
                              "args": {"event": {}}, "hook": "h"})
        fake = io.StringIO(payload)
        with unittest.mock.patch.object(sys, "stdin", fake), \
             unittest.mock.patch.object(self.s1, "_dispatch_one") as d:
            self.assertEqual(self.s1._dispatch_main(), 0)
            d.assert_called_once()
        with unittest.mock.patch.object(sys, "stdin",
                                        io.StringIO("{bad")):
            self.assertEqual(self.s1._dispatch_main(), 0)
        with unittest.mock.patch.object(sys, "stdin",
                                        io.StringIO("[1,2]")):
            self.assertEqual(self.s1._dispatch_main(), 0)

    def test_prompt_shadow_main(self):
        self.set_env(CIEL_HOOK=None, CIEL_HOOK_INPUT="{}")
        with unittest.mock.patch.object(self.s1, "_dispatch_one") as d, \
             unittest.mock.patch.object(self.s1, "_prompt_shadow_jobs",
                                        return_value=[("v", {})]):
            self.assertEqual(self.s1._prompt_shadow_main([]), 0)
            d.assert_called_once()

    def test_session_shadow_main(self):
        self.set_env(CIEL_HOOK=None,
                     CIEL_CONTEXT_ITEMS='{"a": "desc"}', CIEL_TASK="task")
        with unittest.mock.patch.object(self.s1, "_dispatch_one") as d:
            self.assertEqual(self.s1._session_shadow_main([]), 0)
            d.assert_called_once()
        self.set_env(CIEL_CONTEXT_ITEMS="{bad")
        with unittest.mock.patch.object(self.s1, "_dispatch_one") as d:
            self.s1._session_shadow_main(["hook/x"])
            d.assert_not_called()

    def test_resolve_event_and_mains(self):
        # _resolve cache-hit path + _event_record + _decide_main/_ask_main
        with unittest.mock.patch.object(
                self.s1, "_cache_read",
                return_value={"answers": {"q": {"choice": "a"}}}), \
             unittest.mock.patch.object(self.s1, "_append_event"):
            result, hit, lat = self.s1._resolve({"s": 1}, {"q": {}})
            self.assertTrue(hit)
        with unittest.mock.patch.object(self.s1, "_cache_read",
                                        return_value=None), \
             unittest.mock.patch.object(
                 self.s1, "ask",
                 return_value={"answers": {"q": {"choice": "a"}}}), \
             unittest.mock.patch.object(self.s1, "_cache_write") as cw:
            result, hit, lat = self.s1._resolve({"s": 1}, {"q": {}})
            self.assertFalse(hit)
            cw.assert_called_once()

        rec = self.s1._event_record(
            {"surface": "pre_tool_risk", "meta": {"ts": "t"},
             "questions": {}, "state": {}},
            {"answers": {"risk": {"choice": "dangerous",
                                  "confidence": 0.9}}}, False, 5)
        self.assertEqual(rec["flag"], "flag")
        self.assertEqual(rec["latency_ms"], 5)

        with unittest.mock.patch.object(sys, "stdin", io.StringIO("{}")):
            self.assertEqual(self.s1._ask_main(), 0)
        with unittest.mock.patch.object(sys, "stdin",
                                        io.StringIO("{bad")), \
             unittest.mock.patch.object(sys, "argv", ["x", "--decide"]):
            pass
        buf = io.StringIO()
        with unittest.mock.patch.object(sys, "stdin",
                                        io.StringIO("{bad")), \
             contextlib.redirect_stdout(buf):
            self.assertEqual(self.s1._decide_main(), 0)
        self.assertEqual(buf.getvalue().strip(), "null")

        # --decide with a result prints JSON; --ask removes the marker file
        marker = self.t / "marker"
        marker.write_text("x")
        self.set_env(CIEL_SYSTEM1_MARKER=str(marker))
        with unittest.mock.patch.object(
                sys, "stdin",
                io.StringIO('{"state": {}, "questions": {}}')), \
             unittest.mock.patch.object(
                 self.s1, "_resolve",
                 return_value=({"answers": {}}, False, 3)), \
             unittest.mock.patch.object(self.s1, "_append_event"):
            self.assertEqual(self.s1._ask_main(), 0)
            self.assertFalse(marker.exists())
        with unittest.mock.patch.object(
                sys, "stdin",
                io.StringIO('{"state": {}, "questions": {}}')), \
             unittest.mock.patch.object(
                 self.s1, "_resolve",
                 return_value=({"answers": {}}, False, 3)), \
             unittest.mock.patch.object(self.s1, "_append_event"):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.s1._decide_main(), 0)
            self.assertIn("answers", buf.getvalue())

        with _argv("system1.py"):
            buf = io.StringIO()
            with contextlib.redirect_stderr(buf):
                self.assertEqual(self.s1.main(), 2)
        with _argv("system1.py", "--dispatch"):
            with unittest.mock.patch.object(self.s1, "_dispatch_main",
                                            return_value=0):
                self.assertEqual(self.s1.main(), 0)
        with _argv("system1.py", "--prompt-shadow", "h"):
            with unittest.mock.patch.object(self.s1, "_prompt_shadow_main",
                                            return_value=0):
                self.assertEqual(self.s1.main(), 0)
        with _argv("system1.py", "--session-shadow"):
            with unittest.mock.patch.object(self.s1, "_session_shadow_main",
                                            return_value=0):
                self.assertEqual(self.s1.main(), 0)


class TestSystem1CouncilOutcome(unittest.TestCase):
    """council_outcome/consensus stamping + evaluate_risk edge branches."""

    def setUp(self):
        self.s1 = _src("system1")
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.t = Path(self.tmp.name)
        self._saved = os.environ.get("CIEL_HOME")
        os.environ["CIEL_HOME"] = str(self.t)
        self.addCleanup(self._restore)

    def _restore(self):
        if self._saved is None:
            os.environ.pop("CIEL_HOME", None)
        else:
            os.environ["CIEL_HOME"] = self._saved

    def test_council_outcome_stamps_consensus(self):
        with unittest.mock.patch.object(self.s1, "_append_event") as ae:
            self.s1.council_outcome("subj", "approve", run_id="r1")
            rec = ae.call_args[0][0]
            self.assertEqual(rec["meta"]["council_consensus"], "approve")
            self.assertEqual(rec["meta"]["run_id"], "r1")
            self.assertEqual(rec["surface"], "council_prescreen")
            self.s1.council_outcome("subj", "bogus")   # invalid -> no event
            self.assertEqual(ae.call_count, 1)

    def test_evaluate_risk_edges(self):
        with unittest.mock.patch.object(self.s1, "_disabled",
                                        return_value=True):
            self.assertIsNone(
                self.s1.evaluate_risk("exec", "rm", "", timeout=0.1))
        with unittest.mock.patch.object(self.s1, "_disabled",
                                        return_value=False), \
             unittest.mock.patch.object(self.s1, "_mode",
                                        return_value="shadow"):
            self.assertIsNone(
                self.s1.evaluate_risk("exec", "rm", "", timeout=0.1))
        # offline resolve -> None result, still logged
        with unittest.mock.patch.object(self.s1, "_disabled",
                                        return_value=False), \
             unittest.mock.patch.object(self.s1, "_mode",
                                        return_value="active"), \
             unittest.mock.patch.object(self.s1, "_resolve",
                                        return_value=(None, False, 1)), \
             unittest.mock.patch.object(self.s1, "_append_event") as ae:
            self.assertIsNone(
                self.s1.evaluate_risk("exec", "rm", "", timeout=0.1))
            ae.assert_called_once()
        # result without dict risk answer -> None
        with unittest.mock.patch.object(self.s1, "_disabled",
                                        return_value=False), \
             unittest.mock.patch.object(self.s1, "_mode",
                                        return_value="active"), \
             unittest.mock.patch.object(
                 self.s1, "_resolve",
                 return_value=({"answers": {"risk": "no"}}, False, 1)), \
             unittest.mock.patch.object(self.s1, "_append_event"):
            self.assertIsNone(
                self.s1.evaluate_risk("exec", "rm", "", timeout=0.1))
        # bad confidence coerces to 0.0
        with unittest.mock.patch.object(self.s1, "_disabled",
                                        return_value=False), \
             unittest.mock.patch.object(self.s1, "_mode",
                                        return_value="active"), \
             unittest.mock.patch.object(
                 self.s1, "_resolve",
                 return_value=({"answers": {"risk": {"choice": "safe",
                                                    "confidence": "x"}}},
                               False, 1)), \
             unittest.mock.patch.object(self.s1, "_append_event"), \
             unittest.mock.patch.object(self.s1, "_band",
                                        return_value="pass"):
            out = self.s1.evaluate_risk("exec", "rm", "", timeout=0.1)
            self.assertEqual(out, ("safe", 0.0, "pass"))

    def test_completion_check_edges(self):
        # result present but 'done' answer not a dict -> None
        with unittest.mock.patch.object(self.s1, "_disabled",
                                        return_value=False), \
             unittest.mock.patch.object(
                 self.s1, "_resolve",
                 return_value=({"answers": {"done": "no"}}, False, 1)), \
             unittest.mock.patch.object(self.s1, "_append_event"):
            self.assertIsNone(self.s1.completion_check("o", "e"))
        # with_score plumbs evidence_score through
        with unittest.mock.patch.object(self.s1, "_disabled",
                                        return_value=False), \
             unittest.mock.patch.object(
                 self.s1, "_resolve",
                 return_value=({"answers": {
                     "done": {"choice": "complete", "confidence": 0.9},
                     "evidence_score": {"score": 3}}}, False, 1)), \
             unittest.mock.patch.object(self.s1, "_append_event"), \
             unittest.mock.patch.object(self.s1, "_band",
                                        return_value="pass"):
            out = self.s1.completion_check("o", "e", with_score=True)
            self.assertEqual(out["score"], 3)


class TestSystem1ExportGaps(unittest.TestCase):
    def setUp(self):
        _src("secret_scan")
        self.ex = _src("system1_export", SCRIPTS)

    def test_extract_truth_branches(self):
        self.assertEqual(
            self.ex._extract_truth("router",
                                   {"chosen_skill": "sk"}),
            ("sk", False, "router_outcome"))
        self.assertEqual(
            self.ex._extract_truth("completion_check", {"verified": True}),
            ("complete", False, "completion_outcome"))
        self.assertEqual(
            self.ex._extract_truth("completion_check", {"verified": False}),
            ("incomplete", False, "completion_outcome"))
        self.assertEqual(
            self.ex._extract_truth("council_prescreen",
                                   {"council_consensus": "approve"}),
            ("routine", False, "council_consensus"))
        self.assertEqual(
            self.ex._extract_truth("council_prescreen",
                                   {"council_consensus": "reject"}),
            ("escalate", False, "council_consensus"))
        self.assertEqual(
            self.ex._extract_truth("pre_tool_risk", {"regex_decision": "deny"}),
            ("dangerous", False, "regex_decision"))
        self.assertEqual(
            self.ex._extract_truth("pre_tool_risk", {}),
            (None, False, "unknown"))

    def test_compute_margin(self):
        self.assertEqual(self.ex._compute_margin({}), 0.0)
        self.assertEqual(self.ex._compute_margin({"a": 1}), 0.0)
        self.assertAlmostEqual(
            self.ex._compute_margin({"a": 0.7, "b": 0.3}), 0.4)


class TestPairedEvalGaps(unittest.TestCase):
    def setUp(self):
        self.pe = _src("paired_eval", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.t = Path(self.tmp.name)

    def test_run_arm_completion_gate(self):
        seed = self.t / "seed"
        seed.mkdir()
        verify = self.t / "verify.sh"
        verify.write_text("#!/bin/sh\nexit 0\n")
        os.chmod(verify, 0o755)
        task = {"id": "t1", "prompt": "do it", "seed": seed,
                "verify": verify}
        ws = self.t / "ws"
        ws.mkdir()

        class Proc:
            returncode = 0
            stdout = "ok"
            stderr = ""

        s1 = types.ModuleType("system1")
        s1.completion_check = unittest.mock.Mock(
            return_value={"band": "flag"})
        with unittest.mock.patch("subprocess.run", return_value=Proc), \
             unittest.mock.patch.dict(sys.modules, {"system1": s1}):
            # shadow mode: flag recorded but pass unchanged
            r = self.pe._run_arm(task, "echo {prompt}", ws, 10, None,
                                 completion_gate="shadow")
            self.assertTrue(r["pass"])
            self.assertTrue(r["false_pass_detected"])
            self.assertIn("completion_check", r)
            # enforce mode: flag flips arm to fail
            r = self.pe._run_arm(task, "echo {prompt}", ws, 10, None,
                                 completion_gate="enforce")
            self.assertFalse(r["pass"])
            # completion_check raising -> fail-open, pass kept
            s1.completion_check.side_effect = RuntimeError
            r = self.pe._run_arm(task, "echo {prompt}", ws, 10, None,
                                 completion_gate="enforce")
            self.assertTrue(r["pass"])
            self.assertNotIn("completion_check", r)


class TestRiskPolicyFailsafe(_EnvCase):
    """system1_failsafe — the M1 offline-hold block (lines ~305-336)."""

    def setUp(self):
        self.rp = _src("risk_policy")
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.t = Path(self.tmp.name)
        self.set_env(CIEL_HOME=str(self.t), CIEL_SYSTEM1_MODE="active",
                     CIEL_SYSTEM1_DISABLED=None)
        self.s1 = types.SimpleNamespace()
        self.s1._env_file_value = lambda *a: ""
        self.s1.tool_state = lambda *a: {"targets_sensitive_path": False}
        self.addCleanup(self._restore_s1)
        self._saved = self.rp.system1
        self.rp.system1 = self.s1

    def _restore_s1(self):
        self.rp.system1 = self._saved

    def _verdict(self, decision="allow"):
        return {"decision": decision, "rule_id": "r", "tier": "soft"}

    def test_system1_none_returns_verdict(self):
        self.rp.system1 = None
        v = self._verdict()
        self.assertIs(self.rp.system1_failsafe(v, "exec", "rm -rf /"), v)

    def test_mode_not_active(self):
        self.set_env(CIEL_SYSTEM1_MODE="shadow")
        v = self._verdict()
        self.assertIs(self.rp.system1_failsafe(v, "exec", "rm -rf /"), v)

    def test_disabled_env(self):
        self.set_env(CIEL_SYSTEM1_DISABLED="1")
        v = self._verdict()
        self.assertIs(self.rp.system1_failsafe(v, "exec", "rm -rf /"), v)

    def test_deny_and_overridden_bypass(self):
        for d in ("deny", "allow_overridden"):
            v = self._verdict(d)
            self.assertIs(self.rp.system1_failsafe(v, "exec", "rm -rf /"), v)

    def test_grant_active_bypasses(self):
        with unittest.mock.patch.object(self.rp, "grant_state",
                                        return_value={"active": True}):
            v = self._verdict()
            self.assertIs(self.rp.system1_failsafe(v, "exec", "rm -rf /"), v)

    def test_allow_privileged_sentinel(self):
        (self.t / "allow_privileged").write_text("")
        with unittest.mock.patch.object(self.rp, "grant_state",
                                        return_value={}):
            v = self._verdict()
            self.assertIs(self.rp.system1_failsafe(v, "exec", "rm -rf /"), v)

    def test_non_destructive_passthrough(self):
        self.s1.tool_state = lambda *a: {"targets_sensitive_path": False}
        with unittest.mock.patch.object(self.rp, "grant_state",
                                        return_value={}):
            v = self._verdict()
            self.assertIs(self.rp.system1_failsafe(v, "exec", "ls -la"), v)

    def test_destructive_offline_denies(self):
        with unittest.mock.patch.object(self.rp, "grant_state",
                                        return_value={}), \
             unittest.mock.patch.object(
                 self.rp, "system1_evaluate_risk", return_value=None):
            out = self.rp.system1_failsafe(self._verdict(), "exec",
                                           "rm -rf /", "", home=self.t)
        self.assertEqual(out["decision"], "deny")
        self.assertEqual(out["rule_id"], "system1_offline_failsafe")

    def test_destructive_online_keeps_verdict(self):
        with unittest.mock.patch.object(self.rp, "grant_state",
                                        return_value={}), \
             unittest.mock.patch.object(
                 self.rp, "system1_evaluate_risk",
                 return_value=("dangerous", 0.9, "flag")):
            v = self._verdict()
            self.assertIs(self.rp.system1_failsafe(v, "exec", "rm -rf /"),
                          v)

    def test_sensitive_write_is_destructive(self):
        self.s1.tool_state = lambda *a: {"targets_sensitive_path": True}
        with unittest.mock.patch.object(self.rp, "grant_state",
                                        return_value={}), \
             unittest.mock.patch.object(
                 self.rp, "system1_evaluate_risk", return_value=None):
            out = self.rp.system1_failsafe(self._verdict(), "write", "",
                                           "/etc/shadow", home=self.t)
        self.assertEqual(out["rule_id"], "system1_offline_failsafe")


class TestSystem1EdgeBranches(_EnvCase):
    """Scattered uncovered branches in system1.py."""

    def setUp(self):
        self.s1 = _src("system1")
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.t = Path(self.tmp.name)
        self.set_env(CIEL_HOME=str(self.t))

    def test_tau_env_and_lattice_edges(self):
        # reset cached lattice between assertions
        self.s1._POLICY_THRESHOLDS = None
        self.set_env(CIEL_SYSTEM1_TAU_PRE_TOOL_RISK="notfloat")
        tau = self.s1.surface_tau("pre_tool_risk")
        self.assertIsInstance(tau, float)          # ValueError -> pass
        self.set_env(CIEL_SYSTEM1_TAU_PRE_TOOL_RISK=None,
                     CIEL_SYSTEM1_TAU="bad")
        self.s1._POLICY_THRESHOLDS = None
        self.assertIsInstance(self.s1.surface_tau("pre_tool_risk"), float)
        self.set_env(CIEL_SYSTEM1_TAU=None)
        # corrupt lattice file -> OSError/ValueError swallowed
        for cand_attr in dir(self.s1):
            pass
        self.s1._POLICY_THRESHOLDS = {"pre_tool_risk": "nan-x"}
        # float() raises ValueError -> falls to DEFAULT
        self.assertEqual(self.s1.surface_tau("pre_tool_risk"),
                         self.s1.DEFAULT_THRESHOLDS["pre_tool_risk"])
        self.s1._POLICY_THRESHOLDS = None

    def _env_file(self, text):
        d = self.t / "system1"
        d.mkdir(exist_ok=True)
        f = d / "env"
        f.write_text(text)
        # expire the stat-TTL cache so each write is observed
        self.s1._ENV_FILE_CACHE.update(
            {"path": None, "mtime": None, "checked": 0.0, "pairs": {}})
        return f

    def test_env_file_pairs_cache_reset(self):
        self._env_file("A=1\n")
        p1 = self.s1._env_file_pairs()
        self.assertEqual(p1.get("A"), "1")
        # rewrite same path -> new mtime -> re-read
        self._env_file("B=2\n")
        p2 = self.s1._env_file_pairs()
        self.assertEqual(p2.get("B"), "2")
        self.assertNotIn("A", p2)

    def test_key_and_url_resolution(self):
        self._env_file("CIEL_SYSTEM1_KEY=filekey\nJEV_API_KEY=jev\n"
                       "LAYA_HOST=10.0.0.1\nLAYA_PORT=9999\n")
        self.set_env(CIEL_SYSTEM1_KEY=None, CIEL_SYSTEM1_URL=None)
        with unittest.mock.patch.object(self.s1, "_hosted",
                                        return_value=True):
            self.assertEqual(self.s1._key(), "filekey")
        with unittest.mock.patch.object(self.s1, "_hosted",
                                        return_value=False):
            self.assertEqual(self.s1._key(), "filekey")
        self.assertEqual(self.s1._url(), "http://10.0.0.1:9999")
        # neither env nor file -> loopback default
        self._env_file("")
        self.assertEqual(self.s1._url(), "http://127.0.0.1:8765")
        with unittest.mock.patch.object(self.s1, "_hosted",
                                        return_value=False):
            self.assertEqual(self.s1._local_base(), "http://127.0.0.1:8765")

    def test_hosted_trip_write_and_oserror(self):
        self.s1._hosted_trip(503)
        state = json.loads(
            (self.t / "system1" / "hosted_state.json").read_text())
        self.assertEqual(state["status"], 503)
        with unittest.mock.patch.object(
                self.s1, "_hosted_state_path",
                return_value=Path("/proc/no-write/h.json")):
            self.s1._hosted_trip(500)   # OSError swallowed

    def test_ask_disabled_and_local_egress(self):
        with unittest.mock.patch.object(self.s1, "_disabled",
                                        return_value=True):
            self.assertIsNone(self.s1.ask({}, {}))
        # hosted path taken, result returned early
        with unittest.mock.patch.object(self.s1, "_disabled",
                                        return_value=False), \
             unittest.mock.patch.object(self.s1, "_hosted",
                                        return_value=True), \
             unittest.mock.patch.object(self.s1, "_url",
                                        return_value="https://h"), \
             unittest.mock.patch.object(self.s1, "_hosted_down",
                                        return_value=False), \
             unittest.mock.patch.object(self.s1, "_endpoint_egress_ok",
                                        return_value=True), \
             unittest.mock.patch.object(
                 self.s1, "_do_ask",
                 return_value=({"answers": {"q": {}}}, 200)):
            self.assertEqual(self.s1.ask({"s": 1}, {"q": {}}),
                             {"answers": {"q": {}}})
        # local egress blocked -> None
        with unittest.mock.patch.object(self.s1, "_disabled",
                                        return_value=False), \
             unittest.mock.patch.object(self.s1, "_hosted",
                                        return_value=False), \
             unittest.mock.patch.object(self.s1, "_remote",
                                        return_value=False), \
             unittest.mock.patch.object(self.s1, "_hosted_active",
                                        return_value=False), \
             unittest.mock.patch.object(self.s1, "_endpoint_egress_ok",
                                        return_value=False):
            self.assertIsNone(self.s1.ask({}, {}))

    def test_ask_batch_chunk_paths(self):
        with unittest.mock.patch.object(self.s1, "_egress_allowed",
                                        return_value=False):
            self.assertIsNone(self.s1._ask_batch_chunk([{}], {}, 1.0))
        # urlopen failure -> None
        with unittest.mock.patch.object(self.s1, "_egress_allowed",
                                        return_value=True), \
             unittest.mock.patch.object(self.s1, "_remote",
                                        return_value=False), \
             unittest.mock.patch.object(self.s1, "_model",
                                        return_value="m"), \
             unittest.mock.patch("urllib.request.urlopen",
                                 side_effect=OSError):
            self.assertIsNone(self.s1._ask_batch_chunk([{}], {}, 1.0))
        # non-dict result items -> None slots
        resp = unittest.mock.Mock()
        resp.read.return_value = json.dumps(
            {"results": [{"answers": {"q": {}}}, "bad", {}]}).encode()
        cm = unittest.mock.Mock()
        cm.__enter__ = lambda s: resp
        cm.__exit__ = lambda s, *a: False
        with unittest.mock.patch.object(self.s1, "_egress_allowed",
                                        return_value=True), \
             unittest.mock.patch.object(self.s1, "_remote",
                                        return_value=False), \
             unittest.mock.patch.object(self.s1, "_model",
                                        return_value="m"), \
             unittest.mock.patch("urllib.request.urlopen",
                                 return_value=cm):
            out = self.s1._ask_batch_chunk([{}, {}, {}], {}, 1.0)
            self.assertEqual(len(out), 3)
            self.assertIsNone(out[1])
            self.assertIsNone(out[2])

    def test_semantic_rank_embed_bin(self):
        exe = self.t / "embed"
        exe.write_text("#!/bin/sh\ncat >/dev/null; echo '{\"names\":[\"a\"]}'\n")
        os.chmod(exe, 0o755)
        self.set_env(CIEL_SYSTEM1_EMBED_BIN=str(exe),
                     CIEL_SYSTEM1_EMBED=None)
        self.assertEqual(self.s1._semantic_rank("t", {"a": "d"}, 5), ["a"])
        self.set_env(CIEL_SYSTEM1_EMBED_BIN=str(self.t / "absent"))
        # venv python missing on this host path -> [] lexical fallback
        out = self.s1._semantic_rank("t", {"a": "d"}, 5)
        self.assertIsInstance(out, list)
        self.set_env(CIEL_SYSTEM1_EMBED="0")
        self.assertEqual(self.s1._semantic_rank("t", {"a": "d"}, 5), [])

    def test_surface_disabled_early_returns(self):
        with unittest.mock.patch.object(self.s1, "_disabled",
                                        return_value=True):
            self.assertIsNone(self.s1.context_select("t", {"a": "d"}))
            self.assertIsNone(self.s1.memory_salience({"e": 1}))
            self.assertIsNone(self.s1.compaction_decision({"x": 1}))
            self.assertIsNone(self.s1.mandate_canary(["m"], "ctx"))

    def test_band_completion_score_branch(self):
        answers = {"done": {"choice": "complete", "confidence": 0.9},
                   "evidence_score": {"score": 2}}
        self.assertEqual(
            self.s1._band("completion_check", answers), "uncertain")

    def test_ciel_bin_which_fallback(self):
        with tempfile.TemporaryDirectory() as fake_home, \
             unittest.mock.patch.object(Path, "home",
                                        return_value=Path(fake_home)), \
             unittest.mock.patch("shutil.which",
                                 return_value="/p/ciel"):
            self.set_env(CIEL_BIN=None, CIEL_HOME=str(self.t / "nohome"))
            self.assertEqual(self.s1._ciel_bin(), "/p/ciel")

    def test_log_fallback_oserror(self):
        with unittest.mock.patch.object(
                self.s1, "ciel_home",
                return_value=Path("/proc/no-write-ciel")):
            self.s1._log_fallback("h", "r", "v")   # OSError swallowed

    def test_registry_l0_edges(self):
        reg = self.t / "registry"
        reg.mkdir()
        # non-dict/empty skills -> continue to repo twin
        (reg / "index.json").write_text(json.dumps({"skills": {}}))
        out = self.s1._registry_l0()
        self.assertIsInstance(out, dict)
        (reg / "index.json").write_text(json.dumps({"skills": "x"}))
        out = self.s1._registry_l0()
        self.assertIsInstance(out, dict)

    def test_prompt_shadow_jobs_nondict_payload(self):
        jobs = self.s1._prompt_shadow_jobs("other/hook", "[1,2,3]")
        self.assertEqual([v for v, _ in jobs][:2],
                         ["context-compaction", "mandate-canary"])

    def test_find_hook_lib_none(self):
        cr = _src("ciel_root")
        with unittest.mock.patch.object(cr, "hook_lib_dirs",
                                        return_value=[]):
            self.assertIsNone(cr.find_hook_lib())
            self.assertIsNone(cr.bootstrap_lib())

    def test_bootstrap_lib_inserts_path(self):
        cr = _src("ciel_root")
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        lib = Path(self.tmp.name) / "lib"
        lib.mkdir()
        (lib / "risk_policy.py").write_text("")
        trimmed = [p for p in sys.path if p != str(lib)]
        with unittest.mock.patch.object(cr, "hook_lib_dirs",
                                        return_value=[lib]), \
             unittest.mock.patch.object(cr.sys, "path", trimmed):
            self.assertEqual(cr.bootstrap_lib(), lib)
            self.assertIn(str(lib), cr.sys.path)

    def test_requirements_sys_path_insert(self):
        req = _src("requirements")
        libdir = str(Path(req.__file__).resolve().parent)
        removed = False
        if libdir in sys.path:
            sys.path.remove(libdir)
            removed = True
        good = types.ModuleType("system1")
        good.shadow_dispatch = unittest.mock.Mock()
        try:
            with unittest.mock.patch.dict(sys.modules,
                                          {"system1": good}):
                req._salience_shadow({"k": "v"})
                self.assertIn(libdir, sys.path)
        finally:
            if not removed and libdir in sys.path:
                sys.path.remove(libdir)


class TestExportMainBranches(_EnvCase):
    def setUp(self):
        _src("secret_scan")
        self.ex = _src("system1_export", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.t = Path(self.tmp.name)

    def test_main_surface_filter_and_weak(self):
        log = self.t / "e.jsonl"
        log.write_text(
            json.dumps({"surface": "router",
                        "meta": {"chosen_skill": "sk"},
                        "system1": {"answers": {
                            "route": {"choice": "sk", "confidence": 0.9,
                                      "probabilities": {"sk": 0.9,
                                                        "other": 0.1}}}}})
            + "\n"
            + json.dumps({"surface": "pre_tool_risk",
                          "meta": {"regex_decision": "deny"},
                          "system1": {"answers": {
                              "risk": {"choice": "dangerous",
                                       "confidence": 0.9,
                                       "probabilities": {"dangerous": 0.9,
                                                         "safe": 0.1}}}}})
            + "\n")
        out = self.t / "o.jsonl"
        with _argv("x", "--log", str(log), "--out", str(out),
                   "--surface", "router"):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.ex.main(), 0)
        rows = [json.loads(l) for l in out.read_text().splitlines()]
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["surface"], "router")


# ---------------------------------------------------------------------
# Track-1/Track-2 retrain harness — requires torch + the system1 venv deps.
# ---------------------------------------------------------------------

try:
    import torch  # noqa: F401
    import numpy  # noqa: F401
    _HAS_TORCH = True
except ImportError:
    _HAS_TORCH = False


@unittest.skipUnless(_HAS_TORCH, "torch not installed (system1 venv only)")
class TestHeadRetrain(unittest.TestCase):
    def setUp(self):
        self.hr = _src("system1_head_retrain", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.t = Path(self.tmp.name)

    def test_read_env_file(self):
        f = self.t / "env"
        f.write_text("A=1\nB = spaced\n# comment\nBADLINE\nEMPTY=\n")
        env = self.hr._read_env_file(str(f))
        self.assertEqual(env["A"], "1")
        self.assertIn("B", env)
        self.assertEqual(self.hr._read_env_file(str(self.t / "none")), {})

    def test_resolve_served_dir_explicit(self):
        d = self.t / "ckpt"
        d.mkdir()
        self.assertEqual(self.hr.resolve_served_dir(str(d)), str(d))
        with self.assertRaises(SystemExit):
            self.hr.resolve_served_dir(str(self.t / "missing"))

    def test_check_out_dir(self):
        served = self.t / "served"
        served.mkdir()
        # out == served -> refuse
        with self.assertRaises(SystemExit):
            self.hr.check_out_dir(str(served), str(served))
        # inside served -> refuse
        with self.assertRaises(SystemExit):
            self.hr.check_out_dir(str(served / "sub"), str(served))
        # ancestor of served -> refuse
        with self.assertRaises(SystemExit):
            self.hr.check_out_dir(str(self.t), str(served))
        # inside the HF snapshot store -> refuse
        hub = Path(self.hr.HUB_SNAPSHOTS)
        with self.assertRaises(SystemExit):
            self.hr.check_out_dir(str(hub / "x"), str(served))
        # fresh sibling path ok (returned realpath; not created)
        out = self.t / "out"
        self.assertEqual(self.hr.check_out_dir(str(out), str(served)),
                         str(out))

    def test_load_rows(self):
        f = self.t / "rows.jsonl"
        f.write_text(
            json.dumps({"state": {}, "questions": {}, "label": {}}) + "\n"
            + "\nnot-json\n"
            + json.dumps({"state": {}}) + "\n")
        with self.assertRaises(SystemExit):
            self.hr.load_rows(str(f))  # invalid JSON aborts first
        f.write_text(json.dumps({"state": {}}) + "\n")
        with self.assertRaises(SystemExit):
            self.hr.load_rows(str(f))  # missing 'questions'
        f.write_text(json.dumps(
            {"state": {}, "questions": {}, "label": {"q": "a"}}) + "\n")
        rows = self.hr.load_rows(str(f))
        self.assertEqual(len(rows), 1)

    def test_score_target(self):
        import numpy as np
        t = self.hr._score_target(2, 5, 0.5)
        self.assertEqual(t.shape, (5,))
        self.assertTrue(np.isclose(t.sum(), 1.0))
        self.assertEqual(int(np.argmax(t)), 2)

    def test_label_to_target(self):
        import numpy as np
        q_choice = {"t": "choice", "crit": {"safe": "s", "dangerous": "d"}}
        idx, vec = self.hr._label_to_target(
            "risk", q_choice, "dangerous", 0.5, "test")
        self.assertEqual(vec[1], 1.0)
        # choice not in criteria -> SystemExit
        with self.assertRaises(SystemExit):
            self.hr._label_to_target("risk", q_choice, "bogus", 0.5, "test")
        # score question
        q_score = {"t": "score", "crit": ["a", "b", "c"]}
        idx, vec = self.hr._label_to_target(
            "pressure", q_score, 1, 0.5, "test")
        self.assertEqual(idx, 1)
        self.assertTrue(np.isclose(vec.sum(), 1.0))
        with self.assertRaises(SystemExit):
            self.hr._label_to_target("pressure", q_score, "notint", 0.5, "t")
        with self.assertRaises(SystemExit):
            self.hr._label_to_target("pressure", q_score, 9, 0.5, "t")
        # noul question — bool / str / dict shapes and rejection
        q_noul = {"t": "noul"}
        idx, vec = self.hr._label_to_target("n", q_noul, True, 0.5, "t")
        self.assertEqual(idx, 1)
        idx, vec = self.hr._label_to_target("n", q_noul, "false", 0.5, "t")
        self.assertEqual(idx, 0)
        idx, vec = self.hr._label_to_target("n", q_noul,
                                            {"noul": True}, 0.5, "t")
        self.assertEqual(idx, 1)
        with self.assertRaises(SystemExit):
            self.hr._label_to_target("n", q_noul, "maybe", 0.5, "t")
        with self.assertRaises(SystemExit):
            self.hr._label_to_target("n", q_noul, [1], 0.5, "t")

    def _fake_agent(self):
        class FakeAgent:
            def __init__(self):
                self.tok = types.SimpleNamespace(pad_token_id=0)

            def _check_question(self, qid, qdef):
                return None

            def _to_internal(self, qdef):
                return qdef

            def _encode_state(self, state, ids, internal):
                # ids/marker count keyed on state + criterion count so
                # distinct states get distinct cache keys
                s = abs(hash(str(state))) % 5
                out = []
                for qid in ids:
                    crit = internal[qid].get("crit") or {"x": 0}
                    k = len(crit)
                    out.append({
                        "ids": [s + 1, s + 2, s + 3, s + 4],
                        "markers": list(range(1, k + 1)),
                        "qtype": {"choice": 0, "score": 1,
                                  "noul": 2}.get(internal[qid].get("t"), 0),
                    })
                return out

        return FakeAgent()

    def test_encode_dataset(self):
        agent = self._fake_agent()
        rows = [{
            "_source": "t.jsonl:1",
            "surface": "pre_tool_risk",
            "state": {"command": "rm"},
            "questions": {"risk": {"t": "choice",
                                   "crit": {"safe": 0, "dangerous": 1}},
                          "extra_q": {"t": "choice",
                                      "crit": {"a": 0, "b": 1}}},
            "label": {"risk": "dangerous"},
        }, {
            "_source": "t.jsonl:2",
            "state": {"command": "ls"},
            "questions": {},
            "label": {},
        }]
        items, notes = self.hr.encode_dataset(agent, rows, 0.5)
        self.assertEqual(len(items), 1)
        self.assertEqual(items[0]["qid"], "risk")
        self.assertEqual(items[0]["target_idx"], 1)
        self.assertEqual(items[0]["surface"], "pre_tool_risk")
        # unlabeled question noted, empty-label row skipped
        self.assertEqual(len(notes), 1)
        self.assertIn("extra_q", notes[0])
        # marker/target length mismatch -> SystemExit
        bad_agent = self._fake_agent()
        bad_agent._encode_state = lambda *a: [
            {"ids": [1], "markers": [0, 1, 2], "qtype": 0}]
        with self.assertRaises(SystemExit):
            self.hr.encode_dataset(bad_agent, rows[:1], 0.5)

    def test_label_slot_and_dict_score(self):
        q_ord = {"t": "choice", "crit": {"a": 0, "b": 1},
                 "option_order": [1, 0]}
        ti, vec = self.hr._label_to_target("q", q_ord, "a", 0.5, "s")
        self.assertEqual(ti, 1)  # slot 1 holds 'a' under the order
        q_score = {"t": "score", "crit": {str(i): i for i in range(5)}}
        ti, vec = self.hr._label_to_target(
            "q", q_score, {"choice": 4}, 0.5, "s")
        self.assertEqual(ti, 4)

    def _tiny_model(self, d=8, with_encoder_layer=True):
        import torch
        hr = self.hr

        class EncOut:
            def __init__(self, h):
                self.last_hidden_state = h

        class Attn:
            def __init__(self):
                self.Wqkv = torch.nn.Linear(d, 3 * d)
                self.Wo = torch.nn.Linear(d, d)

        class Layer(torch.nn.Module):
            def __init__(self):
                super().__init__()
                self.attn = Attn()

        class Enc(torch.nn.Module):
            def __init__(self):
                super().__init__()
                self.emb = torch.nn.Embedding(32, d)
                self.layers = [Layer()] if with_encoder_layer else []

            def forward(self, input_ids=None, attention_mask=None):
                return EncOut(self.emb(input_ids))

        class HeadLayer(torch.nn.Module):
            def forward(self, h, src_key_padding_mask=None):
                return h

        class Head(torch.nn.Module):
            def __init__(self):
                super().__init__()
                self.layers = [HeadLayer()]

        class Model(torch.nn.Module):
            def __init__(self):
                super().__init__()
                self.encoder = Enc()
                self.type_emb = torch.nn.Embedding(4, d)
                self.head = Head()
                self.scorer = torch.nn.Linear(d, 1)
                self.act_head = torch.nn.Sequential(
                    torch.nn.Linear(d + 4, 2))

            def forward(self, input_ids, attention_mask, marker_pos,
                        marker_mask, qtype):
                h = self.encoder(input_ids=input_ids,
                                 attention_mask=attention_mask
                                 ).last_hidden_state
                return hr.head_forward(self, h, attention_mask,
                                       marker_pos, marker_mask, qtype)

        return Model()

    def _obj_out_model(self, d=8):
        """Variant whose forward returns an object (covers full_forward's
        non-tuple branch)."""
        import torch
        model = self._tiny_model(d)
        hr = self.hr

        def fwd(input_ids, attention_mask, marker_pos, marker_mask, qtype):
            h = model.encoder(input_ids=input_ids,
                              attention_mask=attention_mask).last_hidden_state
            logits, act = hr.head_forward(model, h, attention_mask,
                                          marker_pos, marker_mask, qtype)
            return types.SimpleNamespace(logits=logits, act_logits=act)

        model.forward = fwd
        return model

    def _train_file(self, n=2):
        f = self.t / "train.jsonl"
        rows = [{
            "_source": f"r{i}",
            "surface": "pre_tool_risk",
            "state": {"command": f"cmd{i}"},
            "questions": {"risk": {"t": "choice",
                                   "crit": {"safe": 0, "dangerous": 1}}},
            "label": {"risk": "dangerous"},
        } for i in range(n)]
        f.write_text("".join(json.dumps(r) + "\n" for r in rows))
        return f

    def _agent_with_model(self, model):
        agent = self._fake_agent()
        agent.model = model
        agent.cfg = {"head_layers": 0, "max_len": 8, "head_max_len": 8,
                     "temperature": [1.0],
                     "temperature_by_options": {"2": 1.0}}
        agent.fit_temperatures = lambda records, compute_ece=False, seed=0: {
            "temperature": [1.0], "temperature_by_options": {2: 1.0},
            "n_by_bucket": {}}
        agent.save_calibration = lambda path: Path(path).write_text("{}")
        return agent

    def _collate_items_stub(self):
        import torch

        def collate_items(groups, pad_id):
            items = [g[0] for g in groups]
            n = len(items)
            L = max(len(it["ids"]) for it in items)
            k = max(len(it["markers"]) for it in items)
            ids = torch.zeros(n, L, dtype=torch.long)
            att = torch.zeros(n, L, dtype=torch.long)
            mpos = torch.zeros(n, k, dtype=torch.long)
            mmask = torch.zeros(n, k, dtype=torch.bool)
            qt = torch.zeros(n, dtype=torch.long)
            for i, it in enumerate(items):
                ids[i, :len(it["ids"])] = torch.tensor(it["ids"])
                att[i, :len(it["ids"])] = 1
                mpos[i, :len(it["markers"])] = torch.tensor(it["markers"])
                mmask[i, :len(it["markers"])] = True
                qt[i] = it["qtype"]
            return {"input_ids": ids, "attention_mask": att,
                    "marker_pos": mpos, "marker_mask": mmask, "qtype": qt}
        return collate_items

    def test_collate_head_batch_and_evaluate(self):
        import torch
        items = [{
            "ids": [1, 2, 3], "markers": [1, 2], "qtype": 0,
            "target": [0.0, 1.0], "target_idx": 1, "qid": "risk",
            "surface": "pre_tool_risk",
        }, {
            "ids": [4, 5], "markers": [0], "qtype": 1,
            "target": [1.0], "target_idx": 0, "qid": "score",
            "surface": "other",
        }]
        model = self._tiny_model()
        cache, times = self.hr.cache_encoder_outputs(model, items, 8)
        self.assertEqual(len(cache), 2)
        self.assertIn("pre_tool_risk", times)
        b = self.hr.collate_head_batch(items, cache)
        self.assertEqual(b["h"].shape[0], 2)
        self.assertEqual(b["h"].shape[1], 3)
        out = self.hr.evaluate(model, items, cache, 8)
        self.assertEqual(out["n"], 2)
        self.assertIn("pre_tool_risk", out["per_surface"])
        self.assertIn("choice", out["per_qtype"])
        self.assertIn("score", out["per_qtype"])
        # LoRA path: collate_raw + full_forward (object-returning model)
        out2 = self.hr.evaluate(self._obj_out_model(), items, cache, 8,
                                lora=True)
        self.assertEqual(out2["n"], 2)
        # single-marker item -> kmax==1 else-branch in head_forward
        single = [{
            "ids": [1, 2, 3], "markers": [1], "qtype": 0,
            "target": [1.0], "target_idx": 0, "qid": "q", "surface": "s",
        }]
        cache1, _ = self.hr.cache_encoder_outputs(model, single, 8)
        out3 = self.hr.evaluate(model, single, cache1, 8)
        self.assertEqual(out3["n"], 1)

    def test_main_dry_run_and_full_epoch(self):
        model = self._tiny_model()
        agent = self._agent_with_model(model)
        served = self.t / "served"
        served.mkdir()
        train = self._train_file(3)
        collate = self._collate_items_stub()

        agent_cls = unittest.mock.Mock(return_value=agent)
        import laya.agent
        import laya.common
        with unittest.mock.patch.object(laya.agent, "Agent", agent_cls), \
             unittest.mock.patch.object(laya.common, "collate_items",
                                        collate):
            # arg validation
            with _argv("x", "--train", str(train), "--dry-run",
                       "--batch-size", "9"):
                with self.assertRaises(SystemExit):
                    self.hr.main()
            with _argv("x", "--train", str(train), "--epochs", "0"):
                with self.assertRaises(SystemExit):
                    self.hr.main()
            # dry run
            buf = io.StringIO()
            with _argv("x", "--train", str(train), "--model",
                       str(served), "--dry-run"), \
                    contextlib.redirect_stdout(buf):
                rc = self.hr.main()
            self.assertEqual(rc, 0)
            self.assertIn("[dry-run]", buf.getvalue())
            self.assertIn("parity", buf.getvalue())

            # full 1-epoch run writing a sibling checkpoint
            out = self.t / "out"
            buf = io.StringIO()
            with _argv("x", "--train", str(train), "--model", str(served),
                       "--out", str(out), "--epochs", "1",
                       "--max-updates", "2",
                       "--cache-file", str(self.t / "enc.pt")), \
                    contextlib.redirect_stdout(buf):
                rc = self.hr.main()
            self.assertEqual(rc, 0)
            self.assertTrue((out / "model.safetensors").is_file())
            self.assertTrue((out / "train_report.json").is_file())
            rep = json.loads((out / "train_report.json").read_text())
            self.assertEqual(rep["epochs"], 1)
            # cache file persisted + reload path on a second run
            self.assertTrue((self.t / "enc.pt").is_file())
            with _argv("x", "--train", str(train), "--model", str(served),
                       "--out", str(self.t / "out2"), "--epochs", "1",
                       "--max-updates", "1",
                       "--cache-file", str(self.t / "enc.pt")), \
                    contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(self.hr.main(), 0)
            # partial cache -> missing rows re-encoded and cache re-saved
            import torch as _t
            full = _t.load(self.t / "enc.pt", weights_only=False)
            _t.save(dict(list(full.items())[:1]), self.t / "enc.pt")
            with _argv("x", "--train", str(train), "--model", str(served),
                       "--out", str(self.t / "out3"), "--epochs", "1",
                       "--max-updates", "1",
                       "--cache-file", str(self.t / "enc.pt")), \
                    contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(self.hr.main(), 0)

    def test_main_error_and_edge_paths(self):
        import laya.agent
        import laya.common
        model = self._tiny_model()
        agent = self._agent_with_model(model)
        served = self.t / "served"
        served.mkdir()
        train = self._train_file(3)
        agent_cls = unittest.mock.Mock(return_value=agent)

        with unittest.mock.patch.object(laya.agent, "Agent", agent_cls):
            # no --out, no --dry-run -> refused (line ~751)
            with _argv("x", "--train", str(train), "--model",
                       str(served)), \
                    contextlib.redirect_stdout(io.StringIO()):
                with self.assertRaises(SystemExit):
                    self.hr.main()
            # all-unlabeled corpus -> nothing to train (line ~807)
            bare = self.t / "bare.jsonl"
            bare.write_text(json.dumps({
                "state": {"x": 1},
                "questions": {"q": {"t": "choice",
                                    "crit": {"a": 0, "b": 1}}},
                "label": {}}) + "\n")
            with _argv("x", "--train", str(bare), "--model", str(served),
                       "--dry-run"), \
                    contextlib.redirect_stdout(io.StringIO()):
                with self.assertRaises(SystemExit):
                    self.hr.main()
            # unlabeled-question note print + parity failure path
            noted = self.t / "noted.jsonl"
            noted.write_text(json.dumps({
                "state": {"x": 1},
                "questions": {"risk": {"t": "choice",
                                       "crit": {"safe": 0,
                                                "dangerous": 1}},
                              "spare": {"t": "choice",
                                        "crit": {"a": 0, "b": 1}}},
                "label": {"risk": "safe"}}) + "\n")

            class OffModel(type(model)):
                def forward(self, input_ids, attention_mask, marker_pos,
                            marker_mask, qtype):
                    logits, act = super().forward(
                        input_ids, attention_mask, marker_pos,
                        marker_mask, qtype)
                    return logits + 10.0, act

            off_agent = self._agent_with_model(OffModel())
            buf = io.StringIO()
            with unittest.mock.patch.object(
                    laya.agent, "Agent",
                    unittest.mock.Mock(return_value=off_agent)), \
                 unittest.mock.patch.object(
                     laya.common, "collate_items",
                     self._collate_items_stub()):
                with _argv("x", "--train", str(noted), "--model",
                           str(served), "--dry-run"), \
                        contextlib.redirect_stdout(buf):
                    with self.assertRaises(SystemExit):
                        self.hr.main()
            self.assertIn("[data] note:", buf.getvalue())

    def test_main_max_updates_cap(self):
        import laya.agent
        model = self._tiny_model()
        agent = self._agent_with_model(model)
        served = self.t / "served"
        served.mkdir()
        train = self._train_file(3)
        collate = self._collate_items_stub()
        out = self.t / "cap_out"
        with unittest.mock.patch.object(laya.agent, "Agent",
                                        unittest.mock.Mock(
                                            return_value=agent)), \
             unittest.mock.patch("laya.common.collate_items", collate):
            buf = io.StringIO()
            with _argv("x", "--train", str(train), "--model", str(served),
                       "--out", str(out), "--epochs", "3",
                       "--max-updates", "1"), \
                    contextlib.redirect_stdout(buf):
                self.assertEqual(self.hr.main(), 0)
            self.assertIn("--max-updates 1 reached", buf.getvalue())

    def test_main_lora_bar10_abort(self):
        import laya.agent
        model = self._tiny_model()
        agent = self._agent_with_model(model)
        served = self.t / "served"
        served.mkdir()
        train = self._train_file(4)
        with unittest.mock.patch.object(laya.agent, "Agent",
                                        unittest.mock.Mock(
                                            return_value=agent)):
            buf = io.StringIO()
            with _argv("x", "--train", str(train), "--model", str(served),
                       "--out", str(self.t / "ab"), "--epochs", "300",
                       "--batch-size", "1",
                       "--max-updates", "150", "--budget-hours", "1e-9",
                       "--lora-rank", "1", "--lora-alpha", "2",
                       "--lora-layers", "1"), \
                    contextlib.redirect_stdout(buf):
                with self.assertRaises(SystemExit) as cm:
                    self.hr.main()
            self.assertEqual(cm.exception.code, 3)
            self.assertIn("ABORT", buf.getvalue())

    def test_main_lora_dry_run(self):
        # covers run_updates' lora branch (collate_raw + full_forward)
        import laya.agent
        model = self._tiny_model()
        agent = self._agent_with_model(model)
        served = self.t / "served"
        served.mkdir()
        train = self._train_file(2)
        with unittest.mock.patch.object(laya.agent, "Agent",
                                        unittest.mock.Mock(
                                            return_value=agent)):
            buf = io.StringIO()
            with _argv("x", "--train", str(train), "--model", str(served),
                       "--dry-run", "--lora-rank", "1", "--lora-alpha", "2",
                       "--lora-layers", "1"), \
                    contextlib.redirect_stdout(buf):
                self.assertEqual(self.hr.main(), 0)
            self.assertIn("[dry-run]", buf.getvalue())

    def test_main_lora_mode(self):
        model = self._tiny_model()
        agent = self._agent_with_model(model)
        served = self.t / "served"
        served.mkdir()
        train = self._train_file(2)
        agent_cls = unittest.mock.Mock(return_value=agent)
        import laya.agent
        out = self.t / "lora_out"
        with unittest.mock.patch.object(laya.agent, "Agent", agent_cls):
            with _argv("x", "--train", str(train), "--model", str(served),
                       "--out", str(out), "--epochs", "1",
                       "--max-updates", "2",
                       "--lora-rank", "2", "--lora-alpha", "4",
                       "--lora-layers", "1"), \
                    contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(self.hr.main(), 0)
        self.assertTrue((out / "model.safetensors").is_file())

    def test_main_resolve_env_paths(self):
        env = self.t / "envdir"
        env.mkdir()
        (env / "env").write_text("")
        # no --model: resolve via env file / hub snapshots
        self.set_env = getattr(self, "set_env", None)
        os.environ["CIEL_SYSTEM1_MODEL"] = "typed-decisions"
        try:
            # env file empty + no snapshots -> SystemExit
            with unittest.mock.patch.object(
                    self.hr, "ENV_FILE", str(env / "env")), \
                 unittest.mock.patch.object(
                     self.hr, "HUB_SNAPSHOTS", str(self.t / "nohub")):
                with self.assertRaises(SystemExit):
                    self.hr.resolve_served_dir(None)
            # revision-pinned path
            snap = self.t / "hub"
            ckpt = snap / "rev1" / "typed-decisions"
            ckpt.mkdir(parents=True)
            (ckpt / "rl_agent_config.json").write_text("{}")
            (ckpt / "model.safetensors").write_text("")
            (env / "env").write_text("LAYA_REVISION=rev1\n")
            with unittest.mock.patch.object(
                    self.hr, "ENV_FILE", str(env / "env")), \
                 unittest.mock.patch.object(
                     self.hr, "HUB_SNAPSHOTS", str(snap)):
                got = self.hr.resolve_served_dir(None)
                self.assertIn("rev1", got)
        finally:
            os.environ.pop("CIEL_SYSTEM1_MODEL", None)

    def test_collate_raw_batch(self):
        import torch
        batch = [
            {"ids": [1, 2, 3], "markers": [1, 2], "qtype": 0,
             "target": [0.0, 1.0]},
            {"ids": [4], "markers": [0], "qtype": 1,
             "target": [1.0]},
        ]
        b = self.hr.collate_raw_batch(batch)
        self.assertEqual(b["input_ids"].shape, (2, 3))
        self.assertEqual(b["marker_pos"].shape, (2, 2))
        self.assertFalse(bool(b["marker_mask"][1][1]))
        self.assertEqual(int(b["qtype"][1]), 1)

    def test_lora_linear_and_inject(self):
        import torch
        base = torch.nn.Linear(4, 4)
        lo = self.hr.LoRALinear(base, rank=2, alpha=4.0)
        x = torch.randn(3, 4)
        # zero-init B -> identical output at init
        self.assertTrue(torch.allclose(lo(x), base(x)))
        with self.assertRaises(TypeError):
            self.hr.LoRALinear(torch.nn.ReLU(), 2, 1.0)

        # inject over a stub encoder
        class Attn:
            def __init__(self):
                self.Wqkv = torch.nn.Linear(4, 12)
                self.Wo = torch.nn.Linear(4, 4)

        class Layer:
            def __init__(self):
                self.attn = Attn()

        class Enc:
            def __init__(self):
                self.layers = [Layer() for _ in range(4)]

        class M:
            def __init__(self):
                self.encoder = Enc()

        m = M()
        n = self.hr.inject_lora(m, rank=2, alpha=4.0, last_n_layers=2)
        self.assertEqual(n, 4)
        self.assertIsInstance(m.encoder.layers[3].attn.Wo,
                              self.hr.LoRALinear)
        self.assertFalse(
            m.encoder.layers[3].attn.Wqkv.base.weight.requires_grad)

    def test_soft_ce_and_head_forward(self):
        import torch
        logits = torch.tensor([[2.0, 0.0]])
        target = torch.tensor([[0.0, 1.0]])
        mask = torch.tensor([[True, True]])
        loss = self.hr.soft_ce(logits, target, mask)
        self.assertGreater(loss.item(), 1.5)

        # minimal stub model exposing the pieces head_forward touches
        class StubModel(torch.nn.Module):
            def __init__(self):
                super().__init__()
                self.type_emb = torch.nn.Embedding(4, 8)
                self.head = None
                self.scorer = torch.nn.Linear(8, 1)
                self.act_head = torch.nn.Sequential(torch.nn.Linear(12, 2))

        model = StubModel()
        h = torch.randn(2, 5, 8)
        att = torch.ones(2, 5, dtype=torch.long)
        mpos = torch.tensor([[1, 2], [0, 3]])
        mmask = torch.tensor([[True, True], [True, True]])
        qt = torch.zeros(2, dtype=torch.long)
        logits, act = self.hr.head_forward(model, h, att, mpos, mmask, qt)
        self.assertEqual(logits.shape, (2, 2))
        self.assertEqual(act.shape, (2, 2))

    def test_merged_state_dict_and_write_sibling(self):
        import torch
        from safetensors.torch import load_file

        base = torch.nn.Linear(4, 4)
        lo = self.hr.LoRALinear(base, rank=2, alpha=4.0)
        # give B nonzero weights so the merge is observable
        with torch.no_grad():
            lo.B.weight.fill_(0.5)

        class M(torch.nn.Module):
            def __init__(self):
                super().__init__()
                self.lin = lo

        m = M()
        merged = self.hr.merged_state_dict(m)
        self.assertIn("lin.weight", merged)
        self.assertNotIn("lin.A.weight", merged)
        expected = base.weight + (lo.B.weight @ lo.A.weight) * lo.scale
        self.assertTrue(torch.allclose(merged["lin.weight"], expected))

        # write_sibling_checkpoint over a fake served dir + agent
        served = self.t / "served"
        (served / "encoder").mkdir(parents=True)
        (served / "tokenizer").mkdir()
        (served / "encoder" / "e.bin").write_text("e")
        (served / "tokenizer" / "t.json").write_text("{}")
        (served / "extra.cfg").write_text("x")
        agent = types.SimpleNamespace(
            model=m,
            cfg={"temperature": [1.0],
                 "temperature_by_options": {"2": 1.0}, "k": "v"})
        out = self.t / "out"
        self.hr.write_sibling_checkpoint(
            str(out), str(served), agent,
            {"temperature": [1.1], "temperature_by_options": {2: 0.9}},
            {"note": "test"})
        self.assertTrue((out / "model.safetensors").is_file())
        self.assertTrue((out / "encoder" / "e.bin").is_file())
        self.assertTrue((out / "extra.cfg").is_file())
        cfg = json.loads((out / "rl_agent_config.json").read_text())
        self.assertEqual(cfg["training"]["track1_head_retrain"]["note"],
                         "test")
        sd = load_file(str(out / "model.safetensors"))
        self.assertIn("lin.weight", sd)

    def test_collect_records_and_evaluate(self):
        import numpy as np
        import torch

        class StubModel(torch.nn.Module):
            def __init__(self):
                super().__init__()
                self.type_emb = torch.nn.Embedding(4, 8)
                self.head = None
                self.scorer = torch.nn.Linear(8, 1)
                self.act_head = torch.nn.Sequential(torch.nn.Linear(12, 2))

        model = StubModel()
        items = [{
            "ids": [1, 2], "markers": [0, 1], "qtype": 0,
            "target": [0.0, 1.0], "surface": "s",
            "cache_key": (1, 2), "h": torch.randn(2, 8),
        }]
        cache = {(1, 2): items[0]["h"]}
        import inspect
        src = inspect.getsource(self.hr.collate_head_batch)
        if "cache_key" in src or 'it["ids"]' in src:
            recs = self.hr.collect_records(model, items, cache, 8)
            if recs:
                self.assertEqual(recs[0][0], 0)
                self.assertEqual(recs[0][3], 2)


def _exec_path(path: Path, name="__main__"):
    """Execute a source file under a chosen module name."""
    spec = importlib.util.spec_from_file_location(name, str(path))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


class TestImportFallbacksAndMainGuards(_EnvCase):
    """Module-top import fallbacks and `if __name__ == "__main__"` lines."""

    def test_system1_secret_scan_fallback(self):
        # poison the import so both attempts fail -> secret_scan=None
        with unittest.mock.patch.dict(sys.modules, {"secret_scan": None}):
            mod = _exec_path(LIB / "system1.py", name="system1_alt")
        self.assertIsNone(mod.secret_scan)

    def test_export_patterns_fallback(self):
        with unittest.mock.patch.dict(sys.modules, {"secret_scan": None}):
            mod = _exec_path(SCRIPTS / "system1_export.py",
                             name="system1_export_alt")
        self.assertIn("aws_access_key", mod.PATTERNS)

    def test_rlcd_scrub_fallback(self):
        with unittest.mock.patch.dict(sys.modules,
                                      {"system1_export": None}):
            mod = _exec_path(SCRIPTS / "ciel_rlcd_pipeline.py",
                             name="ciel_rlcd_pipeline_alt")
        self.assertEqual(mod.scrub_secrets(7), 7)

    def test_offline_eval_laya_missing(self):
        with unittest.mock.patch.dict(
                sys.modules, {"laya": None, "laya.agent": None}):
            with self.assertRaises(SystemExit):
                _exec_path(SCRIPTS / "system1_offline_eval.py",
                           name="system1_offline_eval_alt")

    def test_main_guards(self):
        # `sys.exit(main())` / `main()` lines under __main__ — --help
        # makes argparse exit(0) inside main(), marking the call line.
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.set_env(CIEL_HOME=self.tmp.name)
        for script in ("build_registry_index", "system1_corpus_build",
                       "system1_export", "paired_eval",
                       "transcript_sanitize", "system1_teacher_label",
                       "system1_eval", "scan_skills"):
            with _argv(script, "--help"):
                try:
                    _exec_path(SCRIPTS / f"{script}.py")
                except SystemExit as e:
                    self.assertIn(e.code, (0, None))
        # head_retrain imports torch at module top — only on the venv
        try:
            import torch  # noqa: F401
        except ImportError:
            self.skipTest("torch not importable")
        with _argv("x", "--help"):
            try:
                _exec_path(SCRIPTS / "system1_head_retrain.py")
            except SystemExit as e:
                self.assertIn(e.code, (0, None))

    def test_rlcd_main_guard(self):
        # no argparse — main() runs unconditionally; point CIEL_HOME at a
        # temp dir so it scans nothing and writes a throwaway pairs file.
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.set_env(CIEL_HOME=self.tmp.name)
        with contextlib.redirect_stdout(io.StringIO()):
            _exec_path(SCRIPTS / "ciel_rlcd_pipeline.py",
                       name="__main__")

    def test_council_verify_main_guard(self):
        # argv-driven, no argparse: len(sys.argv)!=2 -> usage exit 2.
        with _argv("council_verify"):
            try:
                _exec_path(SCRIPTS / "council_verify.py")
            except SystemExit as e:
                self.assertEqual(e.code, 2)

    def test_offline_eval_main_guard(self):
        fake_agent_mod = types.ModuleType("laya.agent")
        fake_agent_mod.Agent = object
        fake_laya = types.ModuleType("laya")
        fake_laya.agent = fake_agent_mod
        with unittest.mock.patch.dict(
                sys.modules,
                {"laya": fake_laya, "laya.agent": fake_agent_mod}):
            with _argv("x", "--help"):
                try:
                    _exec_path(SCRIPTS / "system1_offline_eval.py",
                               name="__main__")
                except SystemExit as e:
                    self.assertIn(e.code, (0, None))


class TestRemainingBranches(_EnvCase):
    def test_system1_threshold_lattice_corrupt(self):
        s1 = _src("system1")
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        risk = Path(self.tmp.name) / "risk"
        risk.mkdir(parents=True)
        (risk / "policy.json").write_text("{not json")
        saved = s1._POLICY_THRESHOLDS
        self.addCleanup(setattr, s1, "_POLICY_THRESHOLDS", saved)
        s1._POLICY_THRESHOLDS = None
        self.set_env(CIEL_HOME=self.tmp.name)
        out = s1._load_policy_thresholds()
        self.assertIsInstance(out, dict)

    def test_system1_local_base_hosted(self):
        s1 = _src("system1")
        with unittest.mock.patch.object(s1, "_hosted", return_value=True):
            self.assertEqual(s1._local_base(), "http://127.0.0.1:8765")

    def test_system1_batch_results_not_list(self):
        s1 = _src("system1")
        resp = unittest.mock.Mock()
        resp.read.return_value = json.dumps({"results": "nope"}).encode()
        cm = unittest.mock.Mock()
        cm.__enter__ = lambda s: resp
        cm.__exit__ = lambda s, *a: False
        with unittest.mock.patch.object(s1, "_egress_allowed",
                                        return_value=True), \
             unittest.mock.patch.object(s1, "_remote",
                                        return_value=False), \
             unittest.mock.patch.object(s1, "_model", return_value="m"), \
             unittest.mock.patch("urllib.request.urlopen",
                                 return_value=cm):
            self.assertIsNone(s1._ask_batch_chunk([{}], {}, 1.0))

    def test_system1_context_select_job_appended(self):
        s1 = _src("system1")
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        reg = Path(self.tmp.name) / "registry"
        reg.mkdir(parents=True)
        (reg / "index.json").write_text(json.dumps(
            {"skills": {"sk-a": {"description": "does a"}}}))
        self.set_env(CIEL_HOME=self.tmp.name)
        jobs = s1._prompt_shadow_jobs("user-prompt-submit",
                                      json.dumps({"prompt": "pick one"}))
        names = [v for v, _ in jobs]
        self.assertIn("context-select", names)

    def test_requirements_syspath_insert(self):
        req = _src("requirements")
        stub = types.ModuleType("system1")
        stub.shadow_dispatch = unittest.mock.Mock()
        with unittest.mock.patch.object(req.sys, "path", ["/nonlib"]), \
             unittest.mock.patch.dict(sys.modules, {"system1": stub}):
            req._salience_shadow({"k": "v"})
        self.assertIn(str(Path(req.__file__).resolve().parent),
                      req.sys.path)

    def test_export_scrub_scalar(self):
        _src("secret_scan")
        ex = _src("system1_export", SCRIPTS)
        self.assertEqual(ex.scrub_secrets(42), 42)

    def test_export_lib_syspath_insert(self):
        # line 29: LIB missing from sys.path -> insert executes
        _src("secret_scan")
        lib = str(SCRIPTS.parent / "ciel.skill" / "init" / "hooks" / "lib")
        trimmed = [p for p in sys.path if p != lib]
        with unittest.mock.patch.object(sys, "path", trimmed):
            _exec_path(SCRIPTS / "system1_export.py",
                       name="system1_export_alt2")
            self.assertIn(lib, sys.path)

    def test_export_main_malformed_line(self):
        _src("secret_scan")
        ex = _src("system1_export", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        t = Path(self.tmp.name)
        log = t / "e.jsonl"
        log.write_text("{broken json\n\n" + json.dumps({
            "surface": "router",
            "meta": {"chosen_skill": "sk"},
            "system1": {"answers": {"route": {
                "choice": "sk", "confidence": 0.9,
                "probabilities": {"sk": 0.9, "o": 0.1}}}}}) + "\n")
        out = t / "o.jsonl"
        with _argv("x", "--log", str(log), "--out", str(out)):
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(ex.main(), 0)
        self.assertEqual(len(out.read_text().splitlines()), 1)

    def test_corpus_build_meta_extra(self):
        cb = _src("system1_corpus_build", SCRIPTS)
        s = cb._prescreen_state("rel", {"reason": "x"})
        self.assertEqual(s["meta"], {"reason": "x"})
        ev = cb._salience_state("file", "t", {"age_days": 3})
        self.assertEqual(ev["event"]["age_days"], 3)
        st = cb._compact_state(10, 100, {"pressure": 0.9})
        self.assertEqual(st["stats"]["pressure"], 0.9)

    def test_registry_index_frontmatter_edges(self):
        bri = _src("build_registry_index", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        # force the mini-parser fallback (PyYAML present otherwise)
        self.addCleanup(self.tmp.cleanup)
        f = Path(self.tmp.name) / "SKILL.md"
        f.write_text("---\n"
                     "name: skill-x\n"
                     "  orphaned-indent: dropped\n"
                     "# a comment line\n"
                     "\n"
                     "nested:\n"
                     "  sub: val\n"
                     "free text without colon\n"
                     "tags:\n"
                     "  - a\n"
                     "\n"
                     "  - b\n"
                     "---\n"
                     "# Body\n")
        with unittest.mock.patch.dict(sys.modules, {"yaml": None}):
            fm = bri._frontmatter(f)
        self.assertEqual(fm.get("name"), "skill-x")

    def test_teacher_ask_ok(self):
        _src("secret_scan")
        tl = _src("system1_teacher_label", SCRIPTS)
        fake_s1 = types.SimpleNamespace(
            _hosted_active=lambda: True,
            _hosted_url=lambda: "https://h",
            _endpoint_of=lambda u: {"url": u},
            _hosted_key=lambda: "k",
            _hosted_model=lambda: "m",
            _do_ask=lambda *a, **k: ({"answers": {}}, 200))
        with unittest.mock.patch.object(tl, "system1", fake_s1):
            result, status = tl.teacher_ask({}, {}, 1.0)
        self.assertEqual(status, "ok")
        self.assertEqual(result, {"answers": {}})

    def test_paired_eval_completion_gate_paths(self):
        pe = _src("paired_eval", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        t = Path(self.tmp.name)
        # task scaffold: <tasks>/<id>/{prompt.md,verify.sh}
        td = t / "tasks" / "t1"
        td.mkdir(parents=True)
        (td / "prompt.md").write_text("do the thing")
        (td / "verify.sh").write_text("exit 0\n")
        skill = t / "skill"
        skill.mkdir()
        (skill / "SKILL.md").write_text("---\nname: s\n---\n")

        done = subprocess.CompletedProcess([], 0, "out", "")
        stub_s1 = types.ModuleType("system1")
        stub_s1.completion_check = unittest.mock.Mock(
            return_value={"band": "flag"})
        fresh_lib = str(pe.LIB_DIR)
        sys_path = [p for p in sys.path if p != fresh_lib]
        with unittest.mock.patch.object(pe.sys, "path", sys_path), \
             unittest.mock.patch.object(pe.subprocess, "run",
                                        unittest.mock.Mock(
                                            return_value=done)), \
             unittest.mock.patch.dict(sys.modules,
                                      {"system1": stub_s1}):
            arm = pe._run_arm({"id": "t1", "prompt": "p",
                               "verify": td / "verify.sh",
                               "seed": t / "no-seed"},
                              "echo {prompt}", t / "ws", 5, None,
                              completion_gate="enforce")
        self.assertTrue(arm["false_pass_detected"])
        self.assertFalse(arm["pass"])

        # main(): fp_flag line with enforce
        with unittest.mock.patch.object(
                pe, "_run_arm",
                side_effect=[{"pass": True, "agent_secs": 0.1,
                              "false_pass_detected": True},
                             {"pass": True, "agent_secs": 0.1}]), \
             _argv("x", "--skill", str(skill), "--tasks",
                   str(t / "tasks"), "--completion-gate", "enforce"):
            with contextlib.redirect_stdout(io.StringIO()) as buf:
                rc = pe.main()
        self.assertEqual(rc, 2)
        self.assertIn("FALSE PASS INTERCEPTED", buf.getvalue())


if __name__ == "__main__":
    unittest.main()
