import unittest
import os
import sys
from scripts.fix_md_lint import fix_markdown

class TestMarkdownFixer(unittest.TestCase):
    def test_md047_trailing_newline(self):
        content = "## Heading\nNo newline"
        fixed = fix_markdown(content)
        self.assertTrue(fixed.endswith('\n'))
        # Heading\n\nNo newline\n
        self.assertEqual(fixed.count('\n'), 3)

    def test_md012_multiple_blanks(self):
        content = "Line 1\n\n\nLine 2"
        fixed = fix_markdown(content)
        self.assertIn("Line 1\n\nLine 2", fixed)

    def test_md026_trailing_punctuation(self):
        content = "## Heading:"
        fixed = fix_markdown(content)
        self.assertEqual(fixed.strip(), "## Heading")

if __name__ == '__main__':
    unittest.main()
