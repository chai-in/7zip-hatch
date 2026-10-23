#!/usr/bin/env python3
"""Collect notices and corresponding sources for libraries added by AppImage packaging."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[3]


def fetch(url, target, expected=None):
    urllib.request.urlretrieve(url, target)
    digest = hashlib.sha256(target.read_bytes()).hexdigest()
    if expected and digest != expected:
        raise RuntimeError(f"Checksum mismatch: {target}")
    return {"url": url, "file": target.name, "sha256": digest}


def main():
    appdir = Path(sys.argv[1]).resolve()
    sources = ROOT / "build" / "installers" / "appimage-sources"
    sources.mkdir(parents=True, exist_ok=False)
    licenses = appdir / "usr/share/doc/7zip-hatch/Licenses/AppImage"
    licenses.mkdir(parents=True)
    shutil.copytree("/usr/share/common-licenses", licenses / "common-licenses")
    packages = {}
    for library in sorted((appdir / "usr/lib").rglob("*")):
        if not library.is_file() or library.is_symlink():
            continue
        owners = subprocess.check_output(["dpkg-query", "-S", "*/" + library.name], text=True)
        owner = owners.splitlines()[0].split(": ")[0]
        package, version = subprocess.check_output([
            "dpkg-query", "-W", "-f=${source:Package} ${source:Version}", owner,
        ], text=True).split()
        packages[package] = version
        copyright_file = Path("/usr/share/doc") / owner.split(":")[0] / "copyright"
        shutil.copy2(copyright_file, licenses / (package + "-copyright.txt"))
    for package, version in packages.items():
        subprocess.run(["apt-get", "source", "--download-only", f"{package}={version}"],
                       cwd=sources, check=True)

    downloads = []
    downloads.append(fetch("https://github.com/AppImage/type2-runtime/archive/refs/tags/20251108.tar.gz",
                           sources / "type2-runtime-20251108.tar.gz"))
    downloads.append(fetch("https://github.com/libfuse/libfuse/releases/download/fuse-3.15.0/fuse-3.15.0.tar.xz",
                           sources / "fuse-3.15.0.tar.xz",
                           "70589cfd5e1cff7ccd6ac91c86c01be340b227285c5e200baa284e401eea2ca0"))
    downloads.append(fetch("https://github.com/vasi/squashfuse/archive/0.5.2.tar.gz",
                           sources / "squashfuse-0.5.2.tar.gz",
                           "db0238c5981dabbd80ee09ae15387f390091668ca060a7bc38047912491443d3"))
    for archive, names in [
        ("type2-runtime-20251108.tar.gz", ["LICENSE"]),
        ("fuse-3.15.0.tar.xz", ["LICENSE", "LGPL2.txt", "GPL2.txt"]),
        ("squashfuse-0.5.2.tar.gz", ["LICENSE"]),
    ]:
        with tarfile.open(sources / archive) as src:
            for member in src:
                if member.name.count("/") == 1 and Path(member.name).name in names:
                    (licenses / (archive.split(".tar")[0] + "-" + Path(member.name).name)).write_bytes(
                        src.extractfile(member).read())
    # The pinned runtime also statically links these permissively licensed libraries.
    for name, url in {
        "musl-COPYRIGHT": "https://git.musl-libc.org/cgit/musl/plain/COPYRIGHT?h=v1.2.5",
        "zstd-LICENSE": "https://raw.githubusercontent.com/facebook/zstd/v1.5.6/LICENSE",
        "zlib-LICENSE": "https://raw.githubusercontent.com/madler/zlib/v1.3.1/LICENSE",
        "mimalloc-LICENSE": "https://raw.githubusercontent.com/microsoft/mimalloc/v2.1.9/LICENSE",
    }.items():
        downloads.append(fetch(url, licenses / name))
    manifest = {"ubuntu_sources": packages, "runtime_sources_and_notices": downloads}
    for folder in (sources, licenses):
        (folder / "SOURCES.json").write_text(json.dumps(manifest, indent=2) + "\n")
    (sources / "README.txt").write_text(
        "Corresponding sources for libraries added to the 7-Zip Hatch 0.2.0 AppImage.\n"
        "Ubuntu .dsc files identify each exact source package; unpack with dpkg-source -x.\n"
        "The AppImage runtime is the unmodified 20251108 upstream release. Its source,\n"
        "patches, Docker/chroot build recipes and LGPL libfuse 3.15.0 are included.\n"
        "squashfuse 0.5.2 is included. Other runtime libraries carry permissive licenses.\n"
        "Follow type2-runtime's BUILD.md to rebuild its static runtime.\n"
        "Libraries under AppDir/usr/lib can be replaced after --appimage-extract;\n"
        "Hatch/installers/linux/build_appimage.sh documents repacking with appimagetool.\n"
        "Application, 7-Zip, HatchZip and Cargo sources are in the separate\n"
        "7zip-hatch-0.2.0-source-with-dependencies.tar.gz release asset.\n")
    shutil.copytree(licenses, sources / "notices")
    destination = ROOT / "build/packages/7zip-hatch-0.2.0-AppImage-sources.tar.gz"
    with tarfile.open(destination, "w:gz") as output:
        output.add(sources, arcname="7zip-hatch-0.2.0-AppImage-sources")
    print(destination)


if __name__ == "__main__":
    main()
