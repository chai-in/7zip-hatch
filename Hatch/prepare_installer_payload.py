#!/usr/bin/env python3
"""Unpack the checksum-pinned, tested v0.2.0 desktop binaries for installers."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import tarfile
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[1]
VERSION = "0.2.0"
PAYLOADS = {
    "windows": ("windows-amd64.zip", "b925096510017510e490cd82e7be67bc25f9561183a4a7df97ac582f8890f972"),
    "linux": ("linux-x86_64.tar.gz", "d885a99506d94672e4bfd292cfc0256f5f275c8a103ba32b2de8489cdad834e9"),
}


def check_path(name):
    path = PurePosixPath(name)
    if (path.is_absolute() or ".." in path.parts or "\\" in name
            or not path.parts or path.parts[0] != "7zip-hatch"):
        raise ValueError(f"Unexpected archive path: {name}")


def unpack(archive, output, expected):
    actual = hashlib.sha256(archive.read_bytes()).hexdigest()
    if actual != expected:
        raise ValueError(f"Checksum mismatch for {archive}: {actual}")
    # Validate every entry before writing any payload files.
    if archive.name.endswith(".zip"):
        with zipfile.ZipFile(archive) as src:
            for member in src.infolist():
                check_path(member.filename)
                if (member.external_attr >> 16) & 0o170000 == 0o120000:
                    raise ValueError("Payload symlinks are not supported")
            src.extractall(output)
    else:
        with tarfile.open(archive) as src:
            for member in src:
                check_path(member.name)
                if not (member.isfile() or member.isdir()):
                    raise ValueError("Payload links and special files are not supported")
            src.extractall(output)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("platform", choices=PAYLOADS)
    parser.add_argument("--archive", type=Path, help="use an already downloaded release asset")
    parser.add_argument("--output", type=Path, default=ROOT / "build" / "installers" / "payload")
    args = parser.parse_args()
    suffix, digest = PAYLOADS[args.platform]
    name = f"7zip-hatch-{VERSION}-{suffix}"
    archive = args.archive
    if archive is None:
        downloads = ROOT / "build" / "installers" / "downloads"
        downloads.mkdir(parents=True, exist_ok=True)
        archive = downloads / name
        url = f"https://github.com/chai-in/7zip-hatch/releases/download/v{VERSION}/{name}"
        urllib.request.urlretrieve(url, archive)
    if args.output.exists():
        raise SystemExit(f"Output already exists; choose a fresh payload directory: {args.output}")
    unpack(archive, args.output, digest)
    payload = args.output / "7zip-hatch"
    required = ["README.md", "Licenses/GPL-3.0.txt", "Licenses/NOTICE.txt"]
    required += ["7zip-hatch.exe", "7zz.exe", "7zFM.exe"] if args.platform == "windows" else ["7zip-hatch", "7zz"]
    for name in required:
        if not (payload / name).is_file():
            raise SystemExit(f"Missing desktop payload file: {name}")
    shutil.copy2(ROOT / "Hatch" / "installers" / "README.md", payload / "INSTALLERS.md")
    (payload / "INSTALLER-PAYLOAD.json").write_text(json.dumps({
        "version": VERSION, "asset": archive.name, "sha256": digest,
        "binary_source_commit": "eb811fa5e922bd5f6552fa92c3f3320183b777f8",
        "binary_ci_run": "https://github.com/chai-in/7zip-hatch/actions/runs/40031148983",
    }, indent=2) + "\n")
    print(payload)


if __name__ == "__main__":
    main()
