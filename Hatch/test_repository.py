from pathlib import Path
import tempfile
import unittest
from check_repository import action_errors, link_errors


class RepositoryChecks(unittest.TestCase):
    def test_missing_anchor_and_missing_image_are_reported(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            target = root / "guide.md"
            target.write_text("# Install\n")
            source = root / "README.md"
            source.write_text("[Good](guide.md#install) [Bad](guide.md#absent) ![Screen](absent.png)")
            errors = link_errors(source, root)
            self.assertEqual(len(errors), 2)
            self.assertTrue(any("anchor" in error for error in errors))
            self.assertTrue(any("absent.png" in error for error in errors))

    def test_floating_actions_and_privileged_pr_events_are_refused(self):
        self.assertTrue(action_errors("uses: actions/checkout@v6"))
        self.assertTrue(action_errors("pull_request_target:\npermissions: write-all"))
        self.assertFalse(action_errors("uses: actions/checkout@" + "a" * 40))


if __name__ == "__main__":
    unittest.main()
