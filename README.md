# 7zip-hatch

A 7-Zip fork with HatchZip's space-saving extraction engine linked into the
same command-line executable. It uses standard archive formats, including `.7z`.

Based on [7-Zip 26.05](https://github.com/ip7z/7zip) by Igor Pavlov and
[HatchZip 0.1.2](https://github.com/xevrion/hatchzip) by its contributors.
This is an independent project, not an official release of either upstream.

## Features

Normal 7-Zip commands still work. The new `hatch` command extracts ZIP,
7z, tar, and gzip-compressed tar while releasing consumed archive space.
It includes HatchZip's space check, reserved free-space setting, passwords,
verification, progress, interruption/resume, and partial-archive restore.
Byte-split `.001`, `.002`, ... archives are supported.

The engine is statically linked, with no helper executable or shell command.
This initial integration is for the **CLI**. The original 7-Zip Windows GUI
and HatchZip's separate GUI sources are retained; this mode has not yet
been added to 7-Zip's File Manager or Explorer menus.

## Build

Requires Python 3, Rust 1.95.0 (rustup reads the pinned toolchain), a C/C++
compiler, and make. Cargo downloads dependencies using the lockfile.

```sh
python3 Hatch/build.py
build/release/7zz --help
build/release/7zz hatch --help
```

On Windows x64, run `python Hatch/build.py` in a Visual Studio x64 Native
Tools prompt; the output is `build/release/7zz.exe`. macOS requires Xcode
Command Line Tools. Linux uses GCC and make.

## Use

```sh
# Normal 7-Zip commands and compression settings
7zz a backup.7z files/
7zz x backup.7z

# Standard 7z with smaller solid blocks for space-saving extraction
7zz a distribution.7z files/ -t7z -m0=lzma2 -ms=64m

# Read-only planning, listing, and integrity check
7zz hatch check distribution.7z --keep-free 2GB
7zz hatch list distribution.7z
7zz hatch test distribution.7z

# DESTRUCTIVE: the archive is consumed as files become durable
7zz hatch extract distribution.7z -o extracted --verify-first --keep-free 2GB

# Repeat extract to resume after interruption, or rebuild the remaining archive
7zz hatch restore distribution.7z
```

`hatch extract` asks before starting. Pass `--yes` for unattended use.
It never overwrites an existing file; unrelated files in the destination
are allowed. Keep `<archive>.hatchzip` and `<archive>.hatchzip-tail` until
extraction or restore finishes. An incomplete consumed archive needs these
side files and cannot be read independently by normal archive tools.

Passwords are prompted, or accepted with `--password` or `HATCHZIP_PASSWORD`.
The hidden prompt avoids passwords in process arguments. Passwords are not
journaled; resume asks again. `hatch` uses HatchZip's switches and exit codes,
separately from 7-Zip: success 0, general failure 1, usage 2, insufficient
space 3, verification failure 4, password failure 5, paused 130.

Use `7zz hatch <command> --help` for all options. The upstream `hatch man`
and `hatch completions` generators are retained; their generated standalone
`hatchzip` command names are not yet adapted for the `7zz hatch` prefix.

## Limits

- One solid 7z block must finish before it can be released. A single huge
  block can need as much free space as ordinary extraction.
- `-ms=64m` limits grouping of whole files. It does not split a huge file
  into small independent blocks, so it is not a guaranteed space bound.
- Large stored/Deflate ZIP entries, tar entries, and `.tar.gz` can release
  bytes within a stream on filesystems supporting hole punching. This is
  not yet implemented inside 7z blocks. FAT/exFAT use whole-unit truncation.
- Reclaiming the source helps the output only on the same filesystem.
  Snapshots/clones and filesystem behavior can delay actual reclamation.
- Destructive mode does not cover every format normal 7-Zip can read.
  RAR, native split `.z01` ZIP, `.tar.xz`, and `.tar.zst` are not supported
  by the HatchZip engine; use ordinary extraction for them.
- Restore retains extracted files and rebuilds the unextracted remainder.
  It cannot undo already punched stream data, and currently refuses
  encrypted 7z archives. It does not recreate the original full archive.
- HatchZip metadata preservation differs from 7-Zip's. Use ordinary
  extraction for Windows security descriptors, alternate streams, anti-items,
  or special-file semantics unsupported by HatchZip.

See [the source study and architecture](Hatch/DESIGN.md) for details.

## Tests

```sh
python3 Hatch/build.py --debug
python3 Hatch/test_integration.py build/debug/7zz --crash-tests

# Run original HatchZip core/CLI tests through the combined executable
cd third_party/hatchzip
SEVENZIP_HATCH_BINARY="$(pwd)/../../build/debug/7zz" \
  CARGO_TARGET_DIR="$(pwd)/../../build/hatch-target" \
  cargo test --locked -p hatchzip -p hatchzip-core
```

Use `7zz.exe` and PowerShell environment syntax on Windows. Crash tests
intentionally abort child processes at journal boundaries and verify resumed
output byte for byte. Tests create only disposable archives. CI builds and
tests on Linux, macOS, and Windows; its actual results establish support.

## Free software and attribution

7-Zip and the integration use **LGPL-2.1-or-later**, with upstream BSD and
public-domain exceptions preserved. HatchZip is **MIT OR Apache-2.0**.
Neither upstream is relicensed. See [LICENSE](LICENSE), [NOTICE](NOTICE),
[7-Zip's license](DOC/License.txt), and [HatchZip's licenses](third_party/hatchzip/LICENSE-MIT).

Default builds exclude RAR decompression codecs carrying the unRAR
restriction, using 7-Zip's `DISABLE_RAR_COMPRESS`. RAR listing and stored
entries remain available. `python3 Hatch/build.py --with-rar` opts into
those codecs under their original additional terms; it is not the default
FOSS distribution. All original source notices are retained.
