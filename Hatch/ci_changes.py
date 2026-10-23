#!/usr/bin/env python3
"""Choose native CI conservatively; documentation keeps the required CI gate."""
import os
from pathlib import PurePosixPath
import re
import subprocess


def needs_native(paths):
    for name in paths:
        path = PurePosixPath(name)
        if name == ".github/workflows/ci.yml":
            return True
        if name.startswith(".github/"):
            continue
        if path.suffix.lower() == ".md":
            continue
        if name.startswith("Hatch/assets/"):
            continue
        if name.startswith("Hatch/installers/") or name == "Hatch/prepare_installer_payload.py":
            continue  # The installer workflow tests these on Windows/Linux.
        if name in {"Hatch/ci_changes.py", "Hatch/test_ci_changes.py",
                    "Hatch/check_repository.py", "Hatch/test_repository.py",
                    ".editorconfig", ".gitattributes"}:
            continue
        return True  # Unknown and deleted paths must not silently skip builds.
    return False


def main():
    event = os.environ.get("CI_EVENT", "workflow_dispatch")
    base = os.environ.get("CI_BASE", "")
    head = os.environ.get("CI_HEAD", "HEAD")
    code = True
    if event != "workflow_dispatch" and re.fullmatch(r"[0-9a-f]{40}", base) and base != "0" * 40:
        try:
            data = subprocess.check_output(["git", "diff", "--name-only", "--no-renames", "-z", base, head, "--"])
            paths = data.decode().strip("\0").split("\0") if data else []
            code = needs_native(paths)
            print("Changed paths:", *paths, sep="\n")
        except (subprocess.CalledProcessError, UnicodeError):
            print("Cannot determine changes; running native builds.")
    value = str(code).lower()
    print(f"Native build required: {value}")
    output = os.environ.get("GITHUB_OUTPUT")
    if output:
        with open(output, "a") as file:
            file.write(f"code={value}\n")


if __name__ == "__main__":
    main()
