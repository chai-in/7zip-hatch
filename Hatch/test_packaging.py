#!/usr/bin/env python3
"""Regression checks for portable desktop packaging, without building Rust."""
import os
from pathlib import Path
import tempfile
import unittest
import zipfile

from package_desktop import zip_desktop


class Packaging(unittest.TestCase):
    def test_windows_zip_preserves_files_with_old_timestamps(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            desktop = root / "desktop"
            folder = desktop / "7zip-hatch"
            notices = folder / "Licenses" / "Rust" / "example"
            notices.mkdir(parents=True)
            binary = folder / "7zip-hatch.exe"
            binary.write_bytes(b"disposable binary fixture\0\xff")
            license_file = notices / "LICENSE"
            license_file.write_bytes(b"Disposable license fixture\n")
            os.utime(license_file, (0, 0))
            os.utime(notices, (0, 0))
            archive_path = root / "desktop.zip"
            zip_desktop(desktop, archive_path)
            with zipfile.ZipFile(archive_path) as archive:
                self.assertIsNone(archive.testzip())
                name = "7zip-hatch/Licenses/Rust/example/LICENSE"
                self.assertEqual(archive.read(name), license_file.read_bytes())
                self.assertEqual(archive.getinfo(name).date_time, (1980, 1, 1, 0, 0, 0))
                self.assertEqual(archive.read("7zip-hatch/7zip-hatch.exe"), binary.read_bytes())


if __name__ == "__main__":
    unittest.main()
