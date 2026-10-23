#!/usr/bin/env python3
"""Run under xvfb-run on Ubuntu 24.04, without requiring FUSE."""
import hashlib
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import zipfile


def run(*args, **kwargs):
    return subprocess.run(list(map(str, args)), check=True, timeout=180, **kwargs)


def main():
    appimage = Path(sys.argv[1]).resolve()
    payload = Path(sys.argv[2]).resolve()
    root = Path(__file__).resolve().parents[3]
    assert appimage.read_bytes()[8:11] == b"AI\x02", "Not a type 2 AppImage"
    with tempfile.TemporaryDirectory(prefix="appimage smoke ") as tmp:
        tmp = Path(tmp)
        env = dict(os.environ, APPIMAGE_EXTRACT_AND_RUN="1", HOME=str(tmp / "home"),
                   XDG_CONFIG_HOME=str(tmp / "config"), XDG_DATA_HOME=str(tmp / "data"),
                   XDG_CACHE_HOME=str(tmp / "cache"), SLINT_BACKEND="winit-software")
        Path(env["HOME"]).mkdir()
        help_text = subprocess.check_output([appimage, "--help"], env=env, timeout=60)
        assert b"space-saving" in help_text
        run(appimage, "--appimage-extract", cwd=tmp, stdout=subprocess.DEVNULL, env=env)
        appdir = tmp / "squashfs-root"
        for name in ("7zip-hatch", "7zz"):
            # linuxdeploy adjusts ELF library paths; executable code must match.
            def code(binary):
                return subprocess.check_output(["objcopy", "-O", "binary", "--only-section=.text", binary, "/dev/stdout"])
            assert code(appdir / "usr/bin" / name) == code(payload / name), f"Changed executable code: {name}"
        assert (appdir / "usr/share/doc/7zip-hatch/Licenses/GPL-3.0.txt").is_file()
        assert (appdir / "usr/share/doc/7zip-hatch/Licenses/AppImage/SOURCES.json").is_file()
        binary = appdir / "usr/bin/7zz"
        native_env = dict(env, LD_LIBRARY_PATH=str(appdir / "usr/lib"))
        run(sys.executable, root / "Hatch/test_integration.py", binary, env=native_env)
        run(appimage, "--register-shell", env=env)
        entries = list((tmp / "data").rglob("*"))
        menus = [p for p in entries if p.is_file()]
        assert menus, "No context menu files registered"
        assert any(str(appimage) in p.read_text() for p in menus), "Menus must point at the persistent AppImage"
        run(appimage, "--unregister-shell", env=env)
        assert not any(p.is_file() for p in (tmp / "data").rglob("*")), "Context menu cleanup failed"
        archive = tmp / "open archive.zip"
        with zipfile.ZipFile(archive, "w") as output:
            output.writestr("desktop smoke.txt", "The packaged GUI opened this archive.\n")
        before = hashlib.sha256(archive.read_bytes()).digest()
        with (tmp / "gui.log").open("w+") as log:
            process = subprocess.Popen([appimage, "--open", archive], env=env, stdout=log, stderr=log)
            try:
                window = None
                for _ in range(60):
                    if process.poll() is not None:
                        log.seek(0)
                        raise RuntimeError(f"GUI exited {process.returncode}: {log.read()}")
                    found = subprocess.run(["xdotool", "search", "--onlyvisible", "--name", "7-Zip Hatch"], capture_output=True)
                    if found.returncode == 0:
                        window = found.stdout.splitlines()[0].decode()
                        break
                    time.sleep(0.5)
                assert window, "The AppImage did not show a desktop window"
                time.sleep(2)
                screenshot = root / "build/packages/AppImage-desktop-smoke.png"
                run("import", "-window", window, screenshot)
                # A window manager translates this into the app's normal close request.
                run("wmctrl", "-ic", hex(int(window)))
                assert process.wait(timeout=20) == 0, "GUI did not close cleanly"
            finally:
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=15)
        assert hashlib.sha256(archive.read_bytes()).digest() == before, "Opening must preserve the archive"
    print("AppImage runtime, bundled bytes, engine, shell integration and desktop launch passed.")


if __name__ == "__main__":
    main()
