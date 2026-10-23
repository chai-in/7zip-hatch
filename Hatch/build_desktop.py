#!/usr/bin/env python3
"""Build the 7-Zip Hatch desktop app, its engine, and native Windows File Manager."""

import argparse
import os
from pathlib import Path
import platform
import plistlib
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
WORKSPACE = ROOT / "third_party" / "hatchzip"
VERSION = "0.2.0"


def licenses(destination):
    destination.mkdir(parents=True, exist_ok=True)
    for source, name in [(ROOT / "LICENSE", "7zip-LGPL-2.1.txt"),
                         (ROOT / "NOTICE", "NOTICE.txt"),
                         (ROOT / "Hatch" / "COPYING-GPL-3.0.txt", "GPL-3.0.txt"),
                         (ROOT / "DOC" / "License.txt", "7zip-notices.txt"),
                         (WORKSPACE / "LICENSE-MIT", "HatchZip-MIT.txt"),
                         (WORKSPACE / "LICENSE-APACHE", "HatchZip-Apache-2.0.txt")]:
        shutil.copy2(source, destination / name)
    cargo_home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    # Cargo has already downloaded the exact source packages used by this
    # build. Preserve their distributed licenses and attribution files.
    registry = cargo_home / "registry" / "src"
    for source in registry.glob("*/*"):
        if not source.is_dir():
            continue
        for file in source.rglob("*"):
            if file.is_symlink() or not file.is_file():
                continue
            if not file.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE", "COPYRIGHT")):
                continue
            if file.stat().st_size > 1024 * 1024:
                continue
            target = destination / "Rust" / source.name / file.relative_to(source)
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(file, target)
    (destination / "Desktop-license.txt").write_text(
        "The complete Slint-based desktop application is distributed under GPL-3.0.\n"
        "7-Zip components retain LGPL-2.1-or-later and their documented exceptions.\n"
        "HatchZip components retain MIT OR Apache-2.0. Slint is used under GPL-3.0.\n"
        "The command-line engine is a separate LGPL distribution.\n"
        "Sources and build instructions: https://github.com/chai-in/7zip-hatch\n")


def build():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--debug", action="store_true")
    parser.add_argument("--jobs", type=int, default=min(os.cpu_count() or 2, 8))
    parser.add_argument("--skip-engine", action="store_true", help="reuse a previously built engine")
    parser.add_argument("--skip-file-manager", action="store_true", help="Windows: build only the cross-platform desktop frontend")
    parser.add_argument("--output-dir", type=Path, default=ROOT / "build" / "desktop")
    args = parser.parse_args()
    env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "build" / "hatch-target"), CARGO_TERM_COLOR="never")
    if platform.system() == "Darwin":
        env.setdefault("MACOSX_DEPLOYMENT_TARGET", "11.0")
    profile = "debug" if args.debug else "desktop"
    engine_profile = "debug" if args.debug else "release"
    windows = platform.system() == "Windows"
    suffix = ".exe" if windows else ""
    if not args.skip_engine:
        command = [sys.executable, str(ROOT / "Hatch" / "build.py"), "--jobs", str(args.jobs)]
        if args.debug:
            command.append("--debug")
        subprocess.run(command, cwd=ROOT, env=env, check=True)
    engine = ROOT / "build" / engine_profile / ("7zz" + suffix)
    if not engine.is_file():
        raise SystemExit(f"Build the engine first: {engine}")
    command = ["cargo", "build", "--locked", "-p", "hatchzip-gui", "-j", str(args.jobs)]
    if not args.debug:
        command += ["--profile", "desktop"]
    subprocess.run(command, cwd=WORKSPACE, env=env, check=True)
    gui = ROOT / "build" / "hatch-target" / profile / ("hatchzip-gui" + suffix)
    output = args.output_dir.resolve()
    output.mkdir(parents=True, exist_ok=True)
    if platform.system() == "Darwin":
        app = output / "7-Zip Hatch.app"
        if app.exists():
            shutil.rmtree(app)
        contents = app / "Contents"
        binaries = contents / "MacOS"
        resources = contents / "Resources"
        binaries.mkdir(parents=True)
        resources.mkdir()
        shutil.copy2(gui, binaries / "7zip-hatch")
        shutil.copy2(engine, binaries / "7zz")
        shutil.copy2(WORKSPACE / "assets" / "icon" / "hatchzip.icns", resources / "AppIcon.icns")
        types = ["public.zip-archive", "org.7-zip.7-zip-archive", "public.tar-archive", "org.gnu.gnu-zip-archive", "public.archive"]
        info = {
            "CFBundleExecutable": "7zip-hatch", "CFBundleIdentifier": "io.github.chai-in.7zip-hatch",
            "CFBundleName": "7-Zip Hatch", "CFBundleDisplayName": "7-Zip Hatch",
            "CFBundleIconFile": "AppIcon", "CFBundlePackageType": "APPL",
            "CFBundleShortVersionString": VERSION, "CFBundleVersion": VERSION,
            "LSMinimumSystemVersion": "11.0", "NSHighResolutionCapable": True,
            "LSApplicationCategoryType": "public.app-category.utilities",
            "NSHumanReadableCopyright": "GPL-3.0 desktop; includes 7-Zip and HatchZip components",
            "CFBundleDocumentTypes": [{"CFBundleTypeName": "Archive", "CFBundleTypeRole": "Viewer",
                                       "LSHandlerRank": "Alternate", "LSItemContentTypes": types,
                                       "CFBundleTypeExtensions": ["7z", "zip", "tar", "gz", "tgz", "xz", "bz2", "zst", "001", "rar"]}],
            "UTImportedTypeDeclarations": [{"UTTypeIdentifier": "org.7-zip.7-zip-archive",
                "UTTypeDescription": "7-Zip archive", "UTTypeConformsTo": ["public.data", "public.archive"],
                "UTTypeTagSpecification": {"public.filename-extension": ["7z"], "public.mime-type": "application/x-7z-compressed"}}],
        }
        with (contents / "Info.plist").open("wb") as f:
            plistlib.dump(info, f)
        licenses(resources / "Licenses")
        shutil.copy2(ROOT / "README.md", resources / "README.md")
        # Finder/iCloud can attach FinderInfo as soon as a known .app path
        # reappears. It is build-folder metadata and prevents code signing.
        for attribute in ("com.apple.FinderInfo", "com.apple.ResourceFork"):
            subprocess.run(["xattr", "-r", "-d", attribute, str(app)],
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        subprocess.run(["codesign", "--force", "--deep", "--sign", "-", str(app)], check=True)
        subprocess.run(["codesign", "--verify", "--deep", "--strict", str(app)], check=True)
        print(f"Built {app}")
    else:
        folder = output / "7zip-hatch"
        folder.mkdir(exist_ok=True)
        shutil.copy2(gui, folder / ("7zip-hatch" + suffix))
        shutil.copy2(engine, folder / ("7zz" + suffix))
        if windows and not args.skip_file_manager:
            bundle = ROOT / "CPP" / "7zip" / "Bundles" / "Fm"
            obj = "hatch-desktop-x64-" + profile
            subprocess.run(["nmake", "/nologo", "/f", "makefile", "PLATFORM=x64", f"O={obj}", "DISABLE_RAR_COMPRESS=1"], cwd=bundle, check=True)
            shutil.copy2(bundle / obj / "7zFM.exe", folder / "7zFM.exe")
            langs = ROOT / "CPP" / "7zip" / "UI" / "FileManager" / "Lang"
            if langs.exists():
                shutil.copytree(langs, folder / "Lang", dirs_exist_ok=True)
        if not windows:
            (folder / "7zip-hatch.desktop").write_text(
                "[Desktop Entry]\nType=Application\nName=7-Zip Hatch\nExec=7zip-hatch %f\n"
                "Icon=7zip-hatch\nCategories=Utility;Archiving;\nMimeType=application/zip;application/x-7z-compressed;application/x-tar;\n")
        licenses(folder / "Licenses")
        shutil.copy2(ROOT / "README.md", folder / "README.md")
        print(f"Built {folder}")


if __name__ == "__main__":
    build()
