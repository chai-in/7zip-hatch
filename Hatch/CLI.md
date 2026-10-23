# Command-line guide

The bundled native executable preserves normal 7-Zip commands and adds
`7zz hatch` for HatchZip space-saving extraction and recovery.
Use `7zz.exe` on Windows.

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
  by the HatchZip engine. Use ordinary extraction when the bundled native
  engine supports the format. Default FOSS binaries omit the unRAR-restricted
  codecs and cannot decompress most compressed RAR entries.
- Restore retains extracted files and rebuilds the unextracted remainder.
  It cannot undo already punched stream data, and currently refuses
  encrypted 7z archives. It does not recreate the original full archive.
- HatchZip metadata preservation differs from 7-Zip's. Use ordinary
  extraction for Windows security descriptors, alternate streams, anti-items,
  or special-file semantics unsupported by HatchZip.

See [the source study and architecture](DESIGN.md) for details.


## Build and test

See [contributing](../CONTRIBUTING.md) for the pinned toolchain, native builds,
Rust checks, and crash/recovery tests. For the desktop, see the
[desktop guide](DESKTOP.md).
