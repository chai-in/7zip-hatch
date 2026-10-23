#!/usr/bin/env python3
"""Export the current committed source with all locked Cargo dependency sources."""
import io
from pathlib import Path
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[1]
WORKSPACE = ROOT / "third_party" / "hatchzip"
VERSION = "0.2.0"


def main():
    vendor = ROOT / "build" / "source-vendor"
    subprocess.run(["cargo", "vendor", "--locked", "--versioned-dirs", str(vendor)],
                   cwd=WORKSPACE, check=True, stdout=subprocess.DEVNULL)
    packages = ROOT / "build" / "packages"
    packages.mkdir(parents=True, exist_ok=True)
    destination = packages / f"7zip-hatch-{VERSION}-source-with-dependencies.tar.gz"
    source_tar = ROOT / "build" / "source-git.tar"
    subprocess.run(["git", "archive", "--format=tar", "--output", str(source_tar), "HEAD"],
                   cwd=ROOT, check=True)
    prefix = f"7zip-hatch-{VERSION}"
    config = b'[source.crates-io]\nreplace-with = "vendored-sources"\n\n[source.vendored-sources]\ndirectory = "vendor"\n'
    with tarfile.open(destination, "w:gz", compresslevel=6) as output:
        with tarfile.open(source_tar) as source:
            for member in source:
                content = source.extractfile(member) if member.isfile() else None
                member.name = f"{prefix}/{member.name}"
                output.addfile(member, content)
        output.add(vendor, arcname=f"{prefix}/third_party/hatchzip/vendor")
        member = tarfile.TarInfo(f"{prefix}/third_party/hatchzip/.cargo/config.toml")
        member.size = len(config)
        member.mode = 0o644
        output.addfile(member, io.BytesIO(config))
        commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT)
        member = tarfile.TarInfo(f"{prefix}/SOURCE-COMMIT.txt")
        member.size = len(commit)
        output.addfile(member, io.BytesIO(commit))
    source_tar.unlink()
    print(destination)


if __name__ == "__main__":
    main()
