# Install and verify

Use the [0.2.0 preview release](https://github.com/chai-in/7zip-hatch/releases/tag/v0.2.0).
It contains Windows x64, macOS Apple Silicon, and Linux x86-64 packages.
Other architectures are not currently published. All release files are listed
in the release's `SHA256SUMS` file.

## Windows

Run `7zip-hatch-0.2.0-windows-x64-setup.exe`. The wizard installs for the
current user without administrator rights. It adds Start menu entries for
**7-Zip Hatch** and **7-Zip File Manager**. A desktop shortcut and archive
right-click submenu are optional. Windows 11 shows the submenu under
**Show more options**. Open With entries leave existing default apps alone.

The preview setup and executables are unsigned. Verify the download against
the checksums published with this release. The independent fork is not an
official 7-Zip release.

The original File Manager opens the Hatch preview through **Tools >
Space-saving extraction**. The installed CLI is `7zz.exe` in the installation
folder; setup does not alter PATH.

Remove it through **Settings > Apps > Installed apps > 7-Zip Hatch > Uninstall**.
Uninstall removes installed binaries and shortcuts while retaining user
settings and user-added files. The separate upstream 7-Zip installation is
unaffected.

For the portable ZIP, extract the whole folder and keep `7zip-hatch.exe`,
`7zz.exe`, `7zFM.exe`, and the license notices together.

## macOS

Open `7zip-hatch-0.2.0-macos-arm64.dmg` and drag **7-Zip Hatch.app** to
Applications. Launch it from Finder, or use **Open With** on an archive.
The package is for Apple Silicon. It is ad hoc signed and not notarized by
Apple, so macOS may require explicit approval in **System Settings > Privacy
& Security** after the first launch attempt. Only approve the release you
intended to download and verified.

To remove it, quit the app and move it out of Applications. User settings
are retained. A portable app ZIP is also available.

## Linux

The AppImage requires glibc 2.39+ and a graphical X11/Wayland session, such
as Ubuntu 24.04. It bundles the native engine and font, keyboard, Wayland,
and GNU C++ libraries. Graphics drivers and the desktop remain system services.

```sh
chmod +x 7zip-hatch-0.2.0-linux-x86_64.AppImage
./7zip-hatch-0.2.0-linux-x86_64.AppImage

# On a system without FUSE:
./7zip-hatch-0.2.0-linux-x86_64.AppImage --appimage-extract-and-run
```

Keep the AppImage in a permanent location before enabling **Settings >
Right-click menu**. Its menu points to that file. Normal context-menu launches
use the AppImage runtime's normal FUSE path. Disable the menu before moving
or deleting the AppImage. To remove the app, disable its menu and delete the
file. User settings remain in the user's `7zip-hatch` configuration folder.

For the portable tar.gz, extract the whole folder and keep `7zip-hatch` and
`7zz` together. The portable package also needs system Fontconfig/FreeType
and GNU C++ runtime libraries.

## Verify a download

Download `SHA256SUMS` from the **same release** as the installer. Calculate
the file's SHA-256, then compare it with the complete hash on the matching
filename's line in `SHA256SUMS`.

Windows PowerShell:

```powershell
Get-FileHash .\7zip-hatch-0.2.0-windows-x64-setup.exe -Algorithm SHA256
```

macOS:

```sh
shasum -a 256 7zip-hatch-0.2.0-macos-arm64.dmg
```

Linux:

```sh
sha256sum 7zip-hatch-0.2.0-linux-x86_64.AppImage
```

Matching checksums confirm that the downloaded bytes match the published
release asset. They are not a separate publisher signature. The Windows
preview is unsigned; macOS uses an ad hoc signature.

## Source and build provenance

- `7zip-hatch-0.2.0-source-with-dependencies.tar.gz` includes the application
  source, locked Cargo dependency sources, vendor configuration, and installer
  scripts. `SOURCE-COMMIT.txt` identifies its exact revision.
- `7zip-hatch-0.2.0-AppImage-sources.tar.gz` supplies corresponding Ubuntu
  library sources, the AppImage runtime build recipes, and libfuse/squashfuse
  sources. Its `SOURCES.json` records versions and downloads.
- `INSTALLER-PAYLOAD.json` inside each installer records the checksum-pinned
  portable input, original binary source revision, and validating CI run.
  The 0.2.0 installers were added after the desktop preview tag; their payload
  receipts and source archive identify those revisions explicitly.

The GitHub-generated source ZIP/tarball is a snapshot of the tag and does
not include vendored Cargo dependencies or later installer scripts. Use the
named source bundles above to reproduce the complete published packaging.

[Desktop build instructions](DESKTOP.md), [installer build instructions](installers/README.md),
and [contribution checks](../CONTRIBUTING.md) describe the toolchain and tests.
