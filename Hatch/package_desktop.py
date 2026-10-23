#!/usr/bin/env python3
"""Archive the built desktop distribution without losing executable permissions."""
import platform
from pathlib import Path
import shutil
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]
VERSION = "0.2.0"


def zip_desktop(desktop, destination):
    # Cargo license files can carry pre-1980 timestamps. ZIP cannot encode
    # those dates; clamp them without changing the distributed file contents.
    folder = desktop / "7zip-hatch"
    with zipfile.ZipFile(destination, "w", compression=zipfile.ZIP_DEFLATED,
                         strict_timestamps=False) as archive:
        archive.write(folder, "7zip-hatch")
        for file in sorted(folder.rglob("*")):
            archive.write(file, file.relative_to(desktop).as_posix())


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
    elif system == "Windows":
        destination = output / (name + ".zip")
        zip_desktop(desktop, destination)
    else:
        destination = shutil.make_archive(str(output / name), "gztar",
                                          root_dir=desktop, base_dir="7zip-hatch")
    print(destination)


if __name__ == "__main__":
    main()
