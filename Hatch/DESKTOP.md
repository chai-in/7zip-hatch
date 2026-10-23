# Desktop app

7-Zip Hatch 0.2.0 extends the upstream HatchZip Slint frontend with native
7-Zip creation and ordinary extraction. The original Windows File Manager
also opens the space-saving preview from its Tools menu. Archive data uses
existing formats; no special 7z reader is needed.

## Build

Install Rust 1.95.0, Python 3, and the platform's C/C++ build tools. Linux
also needs `pkg-config` and Fontconfig development headers. Windows builds
must run in a Visual Studio x64 Native Tools prompt. macOS requires Xcode
Command Line Tools.

```sh
python3 Hatch/build_desktop.py --jobs 4
```

The script builds the native CLI engine and desktop executable with the
locked dependencies. Output is `build/desktop/7-Zip Hatch.app` on macOS,
or `build/desktop/7zip-hatch/` on Windows/Linux. Copy the entire app or
portable folder; the native engine must stay beside the GUI executable.
Windows additionally includes the original `7zFM.exe` File Manager.

`--debug` builds debug binaries. `--skip-engine` reuses the matching engine
profile. `--skip-file-manager` omits the Windows File Manager. The desktop
release profile disables LTO to keep GUI builds practical; archive work
still uses the optimized native or Rust extraction engine.

On macOS 27, Rust 1.95's Mach-O stripping can make proc-macro libraries
unloadable ([Rust issue 157750](https://github.com/rust-lang/rust/issues/157750)).
Cargo profiles keep dependency/host build artifacts unstripped. The final
application executable is still stripped in the desktop release profile.

## Using the app

Launch the app, choose **Open archive**, or drop one archive into the window.
Finder's **Open With** also works, including while the app is running.
The app registers as an alternate archive handler and does not change the
default application for archive files.

Ordinary extraction is the initial default and keeps the source archive.
Its destination must be new or empty. Extraction uses a sibling staging
folder and only publishes completed output. Compressed tar is piped through
the native engine without writing an intermediate tar archive. The space
reserve in ordinary mode is a preflight check; HatchZip consuming mode also
checks space during extraction.

Enable **Save disk space by consuming the archive** for supported ZIP, 7z,
tar, and gzip tar archives. The app shows the space plan and requires a
confirmation each time. Set reserved free space and verify-first, then
extract. Pause/resume and **Give archive back** retain upstream recovery
behavior. Keep the journal and saved tail with a paused archive.

Use **Compress files** or **Compress folder** to create a new 7z/ZIP archive.
The dialog exposes level, codec, dictionary, solid block size, threads,
volumes, password, and filename encryption. Advanced fields accept native
`-m` compression properties. New archives are published without replacing
existing files, and inputs are always retained. ZIP passwords use printable
ASCII with a maximum of 99 characters, as required by the native ZIP writer.
7z supports Unicode passwords. Passwords go through private stdin pipes,
never process arguments, and are retained only in memory for the open archive.

The cross-platform frontend browses entries, tests archives, creates new
archives, and extracts whole archives. Use the original Windows File Manager
or `7zz` for updating archives, selected-entry operations, and the full set
of native metadata switches. Space-saving format and metadata limits are
described in the root README.

## Desktop integration

On Windows/Linux, **Settings > Right-click menu** adds user-level archive
entries. **Open archive** opens normally; **Space-saving extraction** opens
the consuming preview. Registration never starts extraction automatically.
Finder supports Open With; adding Finder context-menu actions is not yet
implemented. Settings are saved under the user's `7zip-hatch` configuration
folder and contain the reserve, verification choice, and extraction mode.

## Validation

```sh
python3 Hatch/build.py --debug
cd third_party/hatchzip
SEVENZIP_HATCH_BINARY="$(pwd)/../../build/debug/7zz" \
  CARGO_TARGET_DIR="$(pwd)/../../build/hatch-target" \
  cargo test --locked --workspace
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
```

The headless UI suite uses Slint's software renderer to drive the real
windows in light and dark themes. Set `HATCHZIP_GUI_SHOTS` to an existing
directory to save PNG screenshots. Native tests run the bundled engine on
disposable archives, compare extracted bytes, and exercise encryption,
volumes, compressed tar, cancellation, and output collision protection.

## Licensing

The complete Slint desktop executable is distributed under GPL-3.0, using
Slint's GPL licensing option. [The GPL text](COPYING-GPL-3.0.txt) is bundled
with the application. Original HatchZip component sources retain MIT OR
Apache-2.0 notices. The separate native 7-Zip executable and File Manager
retain LGPL-2.1-or-later and their original exceptions. Default binaries
exclude RAR codecs with the unRAR restriction.

Dependencies' distributed license and attribution files are included in
`Licenses/`. Matching source and build instructions are published with the
project. `python3 Hatch/package_source.py` exports the committed tree plus
all locked Cargo dependency sources into `build/packages/`. Extract that
archive to build with the included vendor configuration; after installing
the documented compiler/toolchain, Cargo can run offline. Release binaries
are accompanied by this source bundle. No upstream ownership or official endorsement is implied.
