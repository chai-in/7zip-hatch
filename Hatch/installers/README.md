# Desktop installers

Download installers from the [v0.2.0 release](https://github.com/chai-in/7zip-hatch/releases/tag/v0.2.0).

## Windows x64 setup

Run `7zip-hatch-0.2.0-windows-x64-setup.exe`. It installs for the current
user, without an administrator prompt, to `%LOCALAPPDATA%\Programs\7-Zip Hatch`.
Choose a different folder in the wizard if needed. Start menu entries open
the new 7-Zip Hatch desktop app or the original 7-Zip File Manager. The
File Manager's **Tools > Space-saving extraction** opens the Hatch preview.

The wizard offers a desktop shortcut and an archive right-click submenu.
Windows 11 puts this submenu under **Show more options**. Open With entries
are added without changing default archive associations. The separate
upstream 7-Zip installation is unaffected. The CLI is `7zz.exe` in the
installation directory; the installer does not change PATH.

Use Windows **Settings > Apps > Installed apps > 7-Zip Hatch > Uninstall**
to remove it. Shortcuts and this install's context menu are removed. User
settings, archives and other files added to the installation folder are
preserved. The preview installer and executables are unsigned.

## Linux x86-64 AppImage

```sh
chmod +x 7zip-hatch-0.2.0-linux-x86_64.AppImage
./7zip-hatch-0.2.0-linux-x86_64.AppImage
```

This build requires glibc 2.39 or newer, such as Ubuntu 24.04, and a
graphical X11 or Wayland session. Desktop libraries and the native `7zz`
engine are bundled. The AppImage runtime normally uses FUSE; on a system
without it, run:

```sh
./7zip-hatch-0.2.0-linux-x86_64.AppImage --appimage-extract-and-run
```

Keep the AppImage in a permanent location before enabling **Settings >
Right-click menu**. The menu points at the AppImage file, not its temporary
mount. Disable that setting before deleting or moving the AppImage.
The AppImage does not install system packages or change default archive
handlers. To remove it, disable its right-click menu and delete the file.
User settings remain under the user's `7zip-hatch` configuration directory.

## Building and checking

The installers wrap the unchanged application code from the tested
v0.2.0 portable release, with checksum-pinned input archives. Its binary
source revision is `eb811fa5e922bd5f6552fa92c3f3320183b777f8`, validated in
[the cross-platform build](https://github.com/chai-in/7zip-hatch/actions/runs/40031148983).
The payload receipt is included in each installer. Linux library search
paths are adjusted by linuxdeploy without rebuilding application code.

On Windows, install [Inno Setup 6](https://jrsoftware.org/isinfo.php), then:

```powershell
python Hatch/prepare_installer_payload.py windows
Hatch/installers/windows/build_setup.ps1 build/installers/payload/7zip-hatch
# Only on a disposable Windows test user:
Hatch/installers/windows/test_setup.ps1 build/packages/7zip-hatch-0.2.0-windows-x64-setup.exe build/installers/payload/7zip-hatch
```

On Ubuntu 24.04 x86-64, install `desktop-file-utils`, `patchelf`, `file`,
`curl`, `python3`, `libfontconfig1`, `libfreetype6`, `libstdc++6`,
`libxkbcommon-x11-0`, `libwayland-client0`, `libwayland-cursor0` and
`libwayland-egl1`. Enable `deb-src` repositories matching the installed
libraries. Update the bundled libraries to their current repository versions,
as listed in the workflow, so each binary has an available matching source
package. The builder refuses to substitute a different source version. Then:

```sh
python3 Hatch/prepare_installer_payload.py linux
bash Hatch/installers/linux/build_appimage.sh build/installers/payload/7zip-hatch
# Xvfb, Openbox, xdotool, wmctrl and ImageMagick are needed for this test:
xvfb-run -a sh -c 'openbox >/dev/null 2>&1 & python3 Hatch/installers/linux/test_appimage.py build/packages/7zip-hatch-0.2.0-linux-x86_64.AppImage build/installers/payload/7zip-hatch'
```

To wrap a fresh source build instead, pass its `build/desktop/7zip-hatch`
folder to the platform builder after copying this file as `INSTALLERS.md`
and adding a receipt identifying that build. Output goes to `build/packages`.
Use a clean `build/installers` directory when repeating a build.

The [Desktop installers workflow](../../.github/workflows/installers.yml)
performs these steps on Windows and Linux. Windows tests install, reinstall,
run both GUIs, exercise both engines, and uninstall while preserving a user
file. Linux tests the AppImage without FUSE, checks application code against
the input binaries, exercises both engines, registers and removes menus,
opens a real archive in the GUI, and closes its window normally.

linuxdeploy, appimagetool and the AppImage runtime are pinned to versioned
releases and SHA-256 checksums. Their sources and documentation are at
[linuxdeploy](https://github.com/linuxdeploy/linuxdeploy),
[appimagetool](https://github.com/AppImage/appimagetool) and
[type2-runtime](https://github.com/AppImage/type2-runtime).
Inno Setup's compiler version is logged by the workflow.

All application notices remain in each package. The AppImage also includes
notices for its added libraries. `7zip-hatch-0.2.0-AppImage-sources.tar.gz`
provides the matching Ubuntu library source packages, runtime build recipes,
and libfuse/squashfuse sources. The application and locked Cargo sources
remain in `7zip-hatch-0.2.0-source-with-dependencies.tar.gz`. Extract the
AppImage to replace bundled shared libraries, or use the runtime sources to
rebuild and repack. Packaging does not add restrictions on modified builds.
