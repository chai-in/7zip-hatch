#!/usr/bin/env python3
"""Archive the built desktop distribution without losing executable permissions."""
import platform
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
VERSION = "0.2.0"


def main():
    output = ROOT / "build" / "packages"
    output.mkdir(parents=True, exist_ok=True)
    system = platform.system()
    arch = platform.machine().lower()
    name = f"7zip-hatch-{VERSION}-{system.lower()}-{arch}"
    desktop = ROOT / "build" / "desktop"
    if system == "Darwin":
        destination = output / (name + ".zip")
        subprocess.run(["ditto", "-c", "-k", "--sequesterRsrc", "--keepParent",
                        str(desktop / "7-Zip Hatch.app"), str(destination)], check=True)
    else:
        archive_format = "zip" if system == "Windows" else "gztar"
        destination = shutil.make_archive(str(output / name), archive_format,
                                          root_dir=desktop, base_dir="7zip-hatch")
    print(destination)


if __name__ == "__main__":
    main()
