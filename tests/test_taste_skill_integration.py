import os
import unittest
import yaml

class TestTasteSkillIntegration(unittest.TestCase):
    def setUp(self):
        self.domain_skill_path = os.path.expanduser("~/Ciel/ciel.skill/domains/taste/SKILL.md")
        self.active_skill_path = os.path.expanduser("~/.ciel/skills/taste/SKILL.md")
        self.trigger_registry_path = os.path.expanduser("~/Ciel/ciel.skill/router/TRIGGER_REGISTRY.md")
        self.route_registry_path = os.path.expanduser("~/Ciel/ciel.skill/router/ROUTE_REGISTRY.md")

    def test_canonical_skill_exists_and_valid_frontmatter(self):
        self.assertTrue(os.path.isfile(self.domain_skill_path), "Domain skill must exist")
        self.assertTrue(os.path.isfile(self.active_skill_path), "Active runtime skill must exist")

        with open(self.domain_skill_path, "r", encoding="utf-8") as f:
            content = f.read()

        parts = content.split("---")
        self.assertGreaterEqual(len(parts), 3, "Frontmatter must be present")
        fm = yaml.safe_load(parts[1])

        self.assertEqual(fm.get("name"), "taste")
        self.assertEqual(fm.get("domain"), "design")
        self.assertIn("anti-slop", fm.get("triggers", []))
        self.assertIn("ui", fm.get("tags", []))

    def test_core_taste_prohibitions_present(self):
        with open(self.domain_skill_path, "r", encoding="utf-8") as f:
            content = f.read()

        # Check for banned AI tells
        self.assertIn("Complete Em-Dash Ban", content)
        self.assertIn("Single Accent Color", content)
        self.assertIn("No Purple AI Glow", content)
        self.assertIn("FULL OUTPUT ENFORCEMENT", content)
        self.assertIn("44x44px", content)

    def test_trigger_registry_includes_taste(self):
        with open(self.trigger_registry_path, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("skill: taste", content)
        self.assertIn("anti.?slop", content)

    def test_route_registry_includes_taste(self):
        with open(self.route_registry_path, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("target_skill: taste", content)
        self.assertIn("route_taste_domain_v1", content)

    def test_council_docket_and_verdict_exist(self):
        design_verdict = os.path.expanduser("~/.ciel/council/20261001-taste-design-council/verdict.json")
        ciel_verdict = os.path.expanduser("~/.ciel/council/20261001-taste-ciel-council/verdict.json")
        docket = os.path.expanduser("~/.ciel/council/dockets/DOCKET_20261001_TASTE_CIEL_COUNCIL.md")

        self.assertTrue(os.path.isfile(design_verdict), "Design council verdict must exist")
        self.assertTrue(os.path.isfile(ciel_verdict), "Ciel council verdict must exist")
        self.assertTrue(os.path.isfile(docket), "Council docket must exist")

if __name__ == "__main__":
    unittest.main()
