"""Tests for scripts/harmonize_skills.py (sidecar-aware metadata harmonizer)."""

import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))

import harmonize_skills


class TestHarmonizeSkill(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.skill_dir = Path(self.tmp.name) / "web-wizard"
        self.skill_dir.mkdir()
        self.skill_md = self.skill_dir / "SKILL.md"
        self.sidecar = self.skill_dir / "ciel.yaml"

    def test_sidecar_normalized_skill_body_untouched(self):
        self.skill_md.write_text(
            "---\nname: web-wizard\ndescription: web things\n---\n\nTODO: fill in\n",
            encoding="utf-8",
        )
        self.sidecar.write_text(
            'runtimes: ["old"]\ntags: ["web"]\n', encoding="utf-8"
        )
        harmonize_skills.harmonize_skill(str(self.skill_md))

        meta = self.sidecar.read_text(encoding="utf-8")
        self.assertIn('"generic"', meta)  # runtimes normalized
        self.assertIn('"domain:web"', meta)  # domain tag added
        self.assertIn('"ciel"', meta)
        body = self.skill_md.read_text(encoding="utf-8")
        self.assertIn("Refine implementation logic", body)  # placeholder purged
        self.assertIn("name: web-wizard", body)  # frontmatter preserved

    def test_no_sidecar_falls_back_to_skill_md(self):
        self.skill_md.write_text(
            'runtimes: ["old"]\ntags: ["web"]\n\nFIXME: later\n',
            encoding="utf-8",
        )
        harmonize_skills.harmonize_skill(str(self.skill_md))
        content = self.skill_md.read_text(encoding="utf-8")
        self.assertIn('"generic"', content)
        self.assertIn("Resolve architectural debt", content)

    def test_placeholder_bracket_replaced(self):
        self.skill_md.write_text("step 1\n[...]\nstep 3\n", encoding="utf-8")
        harmonize_skills.harmonize_skill(str(self.skill_md))
        self.assertIn("[Comprehensive implementation details",
                      self.skill_md.read_text(encoding="utf-8"))

    def test_get_domain_default(self):
        self.assertEqual("strategy", harmonize_skills.get_domain("zzz", "nothing"))


if __name__ == "__main__":
    unittest.main()
