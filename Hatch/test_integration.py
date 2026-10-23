#!/usr/bin/env python3
"""Exercise both engines through the same binary, using disposable archives."""

import argparse
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import unittest
import zipfile

BINARY = None
CRASH_TESTS = False


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree(path):
    return {p.relative_to(path).as_posix(): digest(p)
            for p in path.rglob("*") if p.is_file()}


class Integration(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="7zip-hatch-")
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.source = self.root / "input"
        self.source.mkdir()
        (self.source / "empty-dir").mkdir()
        (self.source / "nested").mkdir()
        (self.source / "nested" / "naïve 日本語.txt").write_text("archive data\n" * 1000)
        for i in range(4):
            (self.source / f"{i}.bin").write_bytes(os.urandom(256 * 1024))
        (self.source / "empty").touch()
        self.expected = tree(self.source)
        self.out = self.root / "output with spaces"

    def run7z(self, *args, code=0, env=None, cwd=None):
        actual_env = os.environ.copy()
        # CI runs the upstream suites with these variables; do not let them
        # accidentally redirect or crash unrelated integration cases.
        for name in list(actual_env):
            if name.startswith("HATCHZIP_"):
                actual_env.pop(name)
        actual_env.update(env or {})
        result = subprocess.run([str(BINARY), *map(str, args)], cwd=cwd,
                                env=actual_env, capture_output=True, timeout=180)
        if code is not None:
            self.assertEqual(result.returncode, code,
                             (result.stdout + result.stderr).decode(errors="replace"))
        return result

    def archive(self, suffix=".7z", *options):
        archive = self.root / ("archive with spaces 日本語" + suffix)
        self.run7z("a", archive, ".", "-mx1", "-mmt2", *options, cwd=self.source)
        return archive

    def extract(self, archive, *options, code=0, env=None):
        return self.run7z("hatch", "extract", archive, "-o", self.out,
                          "--keep-free", "0", "--yes", *options, code=code, env=env)

    def assert_output(self):
        self.assertEqual(tree(self.out), self.expected)
        self.assertTrue((self.out / "empty-dir").is_dir())
        self.assertFalse(list(self.root.glob("*.hatchzip*")))

    def test_both_help_and_normal_7zip_extraction(self):
        self.assertIn(b"hatch", self.run7z("--help").stdout)
        self.assertIn(b"restore", self.run7z("hatch", "--help").stdout)
        archive = self.archive()
        self.run7z("t", archive)
        self.run7z("x", archive, f"-o{self.out}", "-y")
        self.assertTrue(archive.exists())
        self.assert_output()

    def test_zip_tar_and_7z_are_consumed(self):
        for suffix, options in [(".zip", []), (".tar", []), (".7z", ["-ms=512k"])]:
            with self.subTest(suffix=suffix):
                archive = self.archive(suffix, *options)
                before = digest(archive)
                self.run7z("hatch", "check", archive, "--keep-free", "0")
                self.run7z("hatch", "list", archive)
                self.run7z("hatch", "test", archive)
                self.assertEqual(digest(archive), before)
                self.extract(archive, "--verify-first")
                self.assertFalse(archive.exists())
                self.assert_output()
                shutil.rmtree(self.out)

    def test_gzip_tar(self):
        archive = self.root / "archive.tar.gz"
        with tarfile.open(archive, "w:gz") as writer:
            for path in self.source.iterdir():
                writer.add(path, arcname=path.name)
        self.extract(archive)
        self.assertFalse(archive.exists())
        self.assert_output()

    def test_encrypted_headers_and_wrong_password(self):
        archive = self.archive(".7z", "-pdisposable-test-password", "-mhe=on", "-ms=512k")
        before = digest(archive)
        self.extract(archive, "--password", "wrong", code=5)
        self.assertEqual(digest(archive), before)
        self.assertFalse(list(self.root.glob("*.hatchzip*")))
        self.extract(archive, "--password", "disposable-test-password")
        self.assert_output()

    def test_split_volume_from_middle_part(self):
        archive = self.archive(".7z", "-ms=512k", "-v200k")
        parts = sorted(self.root.glob(archive.name + ".[0-9]*"))
        self.assertGreater(len(parts), 2)
        self.extract(parts[1])
        self.assertFalse(any(p.exists() for p in parts))
        self.assert_output()

    def test_refusals_preserve_archive(self):
        archive = self.archive()
        before = digest(archive)
        self.run7z("hatch", "check", archive, "--keep-free", "900TB", code=3)
        self.run7z("hatch", "extract", archive, "--keep-free", "0", code=1)
        self.out.mkdir()
        (self.out / "0.bin").write_bytes(b"existing")
        self.extract(archive, code=1)
        self.assertEqual((self.out / "0.bin").read_bytes(), b"existing")
        self.assertEqual(digest(archive), before)
        self.assertFalse(list(self.root.glob("*.hatchzip*")))

    def test_damaged_and_traversal_zip_refused(self):
        archive = self.root / "damaged.zip"
        with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_STORED) as writer:
            writer.writestr("file", b"unique data to corrupt")
        data = archive.read_bytes().replace(b"unique data", b"broken data", 1)
        archive.write_bytes(data)
        self.extract(archive, "--verify-first", code=4)
        self.assertEqual(archive.read_bytes(), data)
        with zipfile.ZipFile(archive, "w") as writer:
            writer.writestr("../outside.txt", b"must not escape")
        before = digest(archive)
        self.extract(archive, code=1)
        self.assertEqual(digest(archive), before)
        self.assertFalse((self.root / "outside.txt").exists())

    def test_resume_and_restore_after_real_abort(self):
        if not CRASH_TESTS:
            self.skipTest("pass --crash-tests with a --debug build")
        for point in ["after-extract", "after-journal"]:
            for restore in [False, True]:
                with self.subTest(point=point, restore=restore):
                    archive = self.archive(".7z", "-ms=256k")
                    crashed = self.extract(archive, code=None, env={
                        "HATCHZIP_BATCH_ENTRIES": "1", "HATCHZIP_CRASH_AT": point,
                        "HATCHZIP_CRASH_AFTER": "1"})
                    self.assertNotEqual(crashed.returncode, 0)
                    self.assertTrue(Path(str(archive) + ".hatchzip").exists())
                    if restore:
                        self.run7z("hatch", "restore", archive)
                        self.run7z("t", archive)
                        self.run7z("x", archive, f"-o{self.out}", "-y")
                        archive.unlink()
                    else:
                        self.extract(archive)
                    self.assert_output()
                    shutil.rmtree(self.out)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--crash-tests", action="store_true")
    args = parser.parse_args()
    BINARY = args.binary.resolve()
    CRASH_TESTS = args.crash_tests
    unittest.main(argv=[__file__], verbosity=2)
