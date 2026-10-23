#!/usr/bin/env python3
"""Payload verification must refuse tampering and traversal before writing files."""
import hashlib
from pathlib import Path
import sys
import tempfile
import unittest
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from prepare_installer_payload import unpack


class PayloadSecurity(unittest.TestCase):
    def test_checksum_mismatch_writes_nothing(self):
        with tempfile.TemporaryDirectory() as tmp:
            archive, output = Path(tmp) / "payload.zip", Path(tmp) / "out"
            with zipfile.ZipFile(archive, "w") as writer:
                writer.writestr("7zip-hatch/7zz.exe", b"tampered")
            with self.assertRaisesRegex(ValueError, "Checksum mismatch"):
                unpack(archive, output, "0" * 64)
            self.assertFalse(output.exists())

    def test_all_paths_are_checked_before_extraction(self):
        with tempfile.TemporaryDirectory() as tmp:
            archive, output = Path(tmp) / "payload.zip", Path(tmp) / "out"
            with zipfile.ZipFile(archive, "w") as writer:
                writer.writestr("7zip-hatch/good.exe", b"good")
                writer.writestr("7zip-hatch/../../outside.exe", b"bad")
            with self.assertRaisesRegex(ValueError, "Unexpected archive path"):
                unpack(archive, output, hashlib.sha256(archive.read_bytes()).hexdigest())
            self.assertFalse(output.exists())
            self.assertFalse((Path(tmp) / "outside.exe").exists())


if __name__ == "__main__":
    unittest.main()
