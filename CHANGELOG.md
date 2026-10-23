# Changelog

User-visible changes to this independent 7-Zip/HatchZip fork are recorded here.
Release assets and notices are published on [GitHub Releases](https://github.com/chai-in/7zip-hatch/releases).

## Unreleased

- Improve the landing page, download instructions, and real desktop screenshot.
- Add contribution, support, conduct, security, and release-maintenance guidance.
- Add issue and pull request templates, contribution checks, and dependency maintenance.
- Configure repository security and protected development rules.

## 0.2.0 desktop preview

- Add a cross-platform Slint desktop app for browsing and testing archives,
  ordinary extraction, and opt-in space-saving extraction.
- Add 7z/ZIP creation controls for compression level, codec, dictionary,
  solid blocks, threads, volumes, encryption, and advanced native options.
- Keep ordinary extraction and creation inputs, with staged output and
  collision protection. Preserve HatchZip planning, reserve, verification,
  pause/resume, and partial restore for consuming mode.
- Add macOS Finder Open With, including when the app is already running.
- Add **Tools > Space-saving extraction** to the original Windows File Manager.
- Publish Windows x64 setup and portable ZIP, Linux x86-64 AppImage and portable
  tar.gz, and macOS Apple Silicon DMG and app ZIP.
- Include notices, checksums, application/Cargo sources, and corresponding
  sources for libraries added to the AppImage.
- Verify both engines on Linux, macOS, and Windows; test real Windows
  install/reinstall/uninstall and GUI launches, plus AppImage desktop launch,
  menus, and native engine integration without FUSE.

Based on 7-Zip 26.05 and HatchZip 0.1.2, with standard archive formats.
See [known limits](Hatch/CLI.md#limits), [licensing](README.md#free-software-and-attribution),
and the [0.2.0 release](https://github.com/chai-in/7zip-hatch/releases/tag/v0.2.0).
