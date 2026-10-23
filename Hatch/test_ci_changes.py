import unittest
from ci_changes import needs_native


class NativeCISelection(unittest.TestCase):
    def test_docs_and_project_metadata_do_not_recompile(self):
        self.assertFalse(needs_native(["README.md", "Hatch/DOWNLOADS.md", ".github/ISSUE_TEMPLATE/bug_report.yml"]))

    def test_native_rust_unknown_and_build_workflow_changes_do_compile(self):
        for name in ["CPP/7zip/Archive/Zip/ZipHandler.cpp", "C/LzmaDec.c",
                     "third_party/hatchzip/crates/core/src/engine.rs", "third_party/hatchzip/Cargo.lock",
                     "Hatch/build.py", ".github/workflows/ci.yml", "new-build-tool"]:
            with self.subTest(path=name):
                self.assertTrue(needs_native(["README.md", name]))

    def test_installer_changes_have_their_own_platform_workflow(self):
        self.assertFalse(needs_native(["Hatch/installers/windows/setup.iss", "Hatch/prepare_installer_payload.py"]))


if __name__ == "__main__":
    unittest.main()
