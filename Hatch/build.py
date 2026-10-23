#!/usr/bin/env python3
"""Build 7zz with the HatchZip engine statically linked into the executable."""

import argparse
import os
from pathlib import Path
import platform
import re
import shlex
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
WORKSPACE = ROOT / "third_party" / "hatchzip"
BUNDLE = ROOT / "CPP" / "7zip" / "Bundles" / "Alone2"


def build():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--debug", action="store_true", help="enable Rust crash-test hooks")
    parser.add_argument("--jobs", type=int, default=min(os.cpu_count() or 2, 8))
    parser.add_argument("--with-rar", action="store_true",
                        help="include RAR decoders subject to the unRAR restrictions")
    args = parser.parse_args()
    profile = "debug" if args.debug else "release"
    mode = "rar" if args.with_rar else "foss"
    target = ROOT / "build" / "hatch-target"
    env = dict(os.environ, CARGO_TARGET_DIR=str(target))
    if platform.system() == "Darwin":
        # Match 7-Zip's deployment target for C dependencies built by Cargo.
        env.setdefault("MACOSX_DEPLOYMENT_TARGET", "11.0")
    command = ["cargo", "rustc", "-p", "hatchzip", "--lib", "--locked", "-j", str(args.jobs)]
    if not args.debug:
        command.append("--release")
    command += ["--", "--print", "native-static-libs"]
    result = subprocess.run(command, cwd=WORKSPACE, env=env, text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    print(result.stdout, end="", flush=True)
    result.check_returncode()
    libraries = re.findall(r"native-static-libs: (.+)", result.stdout)
    if not libraries:
        raise SystemExit("rustc did not report its native libraries; refusing an incomplete link")

    windows = platform.system() == "Windows"
    library = target / profile / ("hatchzip.lib" if windows else "libhatchzip.a")
    relative_lib = os.path.relpath(library, BUNDLE)
    obj_dir = f"b/hatch-{platform.system().lower()}-{platform.machine()}-{profile}-{mode}"
    # Isolate objects from stock builds and builds with other flags. Make
    # normally does not notice when a preprocessor flag or library changes.
    if windows:
        obj_dir = obj_dir.replace("b/", "")
        if not args.with_rar:
            rar = ["DISABLE_RAR_COMPRESS=1"]
        else:
            rar = []
        command = ["nmake", "/nologo", "/f", "makefile", "PLATFORM=x64",
                   f"O={obj_dir}", f"HATCHZIP_LIB={relative_lib}",
                   f"HATCHZIP_NATIVE_LIBS={libraries[-1]}"] + rar
        executable = BUNDLE / obj_dir / "7zz.exe"
    else:
        if platform.system() == "Darwin":
            if platform.machine() != "arm64":
                makefile = "../../cmpl_clang.mak"
                platform_flags = ["MACOS_MODE=1"]
            else:
                makefile = "../../cmpl_mac_arm64.mak"
                platform_flags = []
        else:
            makefile = "../../cmpl_gcc.mak"
            platform_flags = []
        command = ["make", f"-j{args.jobs}", "-f", makefile, f"O={obj_dir}",
                   f"HATCHZIP_LIB={relative_lib}",
                   f"HATCHZIP_NATIVE_LIBS={libraries[-1]}"] + platform_flags
        if not args.with_rar:
            command.append("DISABLE_RAR_COMPRESS=1")
        executable = BUNDLE / obj_dir / "7zz"
    print("Building", shlex.join(command), flush=True)
    subprocess.run(command, cwd=BUNDLE, check=True)
    output = ROOT / "build" / profile / executable.name
    output.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(executable, output)
    print(f"Built {output}", flush=True)


if __name__ == "__main__":
    build()
