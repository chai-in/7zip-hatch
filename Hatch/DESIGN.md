# Source study and integration design

Inspected 24 October 2026 against exact source snapshots:

| Component | Version | Commit |
| --- | --- | --- |
| Official 7-Zip | 26.05 | [`b88c6bc`](https://github.com/ip7z/7zip/tree/b88c6bc09339de32f7d18bbab900686b244f42ba) |
| HatchZip | 0.1.2 | [`d8a4edb`](https://github.com/xevrion/hatchzip/tree/d8a4edbc9ec8ef84540e42408916a409a7d3c21b) |

## Findings

HatchZip is an extraction engine, not a format. Both projects already read
ordinary 7z. Its transaction protocol can be integrated without changing
7z signatures, coder IDs, or wire compatibility.

7-Zip supplies archive creation/update, codec/filter chains, dictionary and
solid settings, threading/memory controls, encryption, volume splitting,
metadata settings, and broad format support. HatchZip adds space-aware
destructive extraction with a recovery journal. Their parsers have
different meanings for common options such as `-p`, `-o`, and `-y`.
The integration keeps those parsers separate under one executable:
`7zz hatch ...` enters HatchZip; other commands retain 7-Zip behavior.

## 7-Zip source map

| Concern | Source |
| --- | --- |
| Process entry and signal lifetime | `CPP/7zip/UI/Console/MainAr.cpp` |
| CLI/help/parser | `CPP/7zip/UI/Console/Main.cpp`, `CPP/7zip/UI/Common/ArchiveCommandLine.cpp` |
| Filesystem extraction | `CPP/7zip/UI/Common/Extract.cpp`, `ArchiveExtractCallback.cpp` |
| 7z folders, decoding and CRC | `CPP/7zip/Archive/7z/7zExtract.cpp`, `7zDecode.cpp` |
| Method/solid option handling | `CPP/7zip/Archive/7z/7zHandlerOut.cpp` |
| Grouping/encoding/repacking | `CPP/7zip/Archive/7z/7zUpdate.cpp` |
| Header read/write | `CPP/7zip/Archive/7z/7zIn.cpp`, `7zOut.cpp` |
| Format and method IDs | `DOC/7zFormat.txt`, `DOC/Methods.txt` |
| Complete CLI build | `CPP/7zip/Bundles/Alone2/` |
| Windows UI/shell | `CPP/7zip/UI/FileManager/`, `GUI/`, `Explorer/` |

A 32-byte 7z signature header points to the next header, normally at the
end. Packed streams are described by folders: coder graphs, sizes,
substreams/files, and checksums. A folder can use several packed streams,
as with BCJ2; it is a decoder dependency unit, not a filesystem directory.

`7zUpdate.cpp` groups whole files using `NumSolidFiles` and `NumSolidBytes`,
but always allows at least one file in a group. Thus `-ms=64m` cannot turn
a 50 GB input file into 64 MiB independently releasable folders.

## HatchZip source map

All paths here are relative to `third_party/hatchzip/`.

| Concern | Source |
| --- | --- |
| Session/password/collision/plan API | `crates/core/src/session.rs` |
| Planning, units, batches, parallel work, commit | `crates/core/src/engine.rs` |
| Recovery state and durable saved tail | `crates/core/src/journal.rs` |
| Locking, split volumes, truncation, punching | `crates/core/src/archive/storage.rs` |
| Format units/adapters | `archive/zip.rs`, `sevenz.rs`, `tar.rs` |
| Deflate/gzip restart and tar parser | `archive/resumable.rs`, `targz.rs` |
| Partial-archive restore | `crates/core/src/restore.rs` and format header builders |
| OS flush/allocation/reclamation | `crates/core/src/platform/` |
| CLI/schema | `crates/cli/src/lib.rs`, `cli.rs` |
| Slint UI, task windows, saved settings | `crates/gui/` |
| OS shell registration | `crates/shell/` |

### Space reclamation

ZIP and tar normally release one entry; 7z releases one folder/solid
block. Units are visited in descending physical order. Byte-split volumes
form one logical file, allowing trailing volumes to be deleted once their
contents are committed. A volume boundary is not a decoder reset point.

Before truncation removes an end-of-archive index, it is saved durably.
A virtual reader combines the surviving prefix with that tail; missing
intermediate regions read as zeros, as required for ZIP's backward scan
and tar's termination handling on resume.

The planner models output before reclaiming its corresponding input:

```
extra = saved_tail_cost
for unit in reverse_physical_order:
    peak = max(peak, extra + output_allocation(unit))
    extra += output_allocation(unit) - reclaimed_span(unit)
required_free = peak + keep_free
```

Actual code also handles stream profiles, resume output, filesystem
granularity, and source/destination placement. This is not a promise of
exactly `output - archive` free bytes. Tiny files, filesystem metadata,
concurrent allocations, clones, and snapshots affect physical space.
Runtime checks remain necessary.

### Transaction and crash behavior

The critical ordering is:

1. Decode/check output into part files; rename completed files.
2. Flush output data and containing directories.
3. Atomically replace and flush the journal with the committed boundary.
4. Truncate the archive, or punch only up to the durable restart point.

Before commit, interruption repeats uncommitted work. After commit, it
finishes pending reclamation. Already consumed stream output must never
be deleted as stale uncommitted files. Linux uses a batch `syncfs`; macOS
flushes files/directories; Windows uses file flush and rename semantics.
These rely on real filesystem/drive behavior, not a guarantee against
every hardware failure.

Large unencrypted stored/Deflate ZIP entries and tar files can stream
with a default 256 MiB checkpoint interval. Deflate also needs a block
boundary, bit offset, running checksum, and 32 KiB history window.
Gzip tar saves decoder/parser state, scans the stream before mutation,
and extracts forward while punching consumed input. The engine probes
hole-punch capability by measuring a scratch file's physical allocation.

### Settings and feature reconciliation

| Capability | Native 7-Zip | Embedded HatchZip |
| --- | --- | --- |
| Create/update and codec tuning | Original commands | Use native encoder |
| Nondestructive extraction | Original commands | Separate destructive mode |
| Space planning/reserve | Original behavior | `check`, `--keep-free` (default 2 decimal GB) |
| Integrity checks | `t` | `hatch test`, `--verify-first` |
| Passwords | Original switches | Prompt, option or environment; never journaled |
| Split volumes | Native formats | Byte splits only |
| Consumed-archive resume | None added to native path | Repeat extraction with journal/tail |
| Partial-archive restore | Original behavior | Upstream ZIP/7z/tar restore within its limits |
| Existing output files | Original overwrite options | Refuse collisions, allow unrelated files |
| Parallel work | Original behavior | Rayon batches plus decoder threads |
| GUI/settings/shell | Source retained | Separate UI retained; no unified UI yet |

HatchZip's GUI persists only `keep_free` and `verify_first`. The CLI takes
these as options. The journal stores operational recovery state, not
general preferences. GUI pause/resume calls the same core APIs.

Some upstream docs are stale: `docs/formats.md` says encryption and
nonempty destinations are refused. Current `session.rs`, `sevenz.rs`,
CLI code, and password tests show that both passwords and unrelated
destination files are supported. This fork follows code and tests.

## Integration decisions

- Vendor the exact snapshot, licenses, docs, GUI/shell source, and tests.
- Expose `main_from(args) -> u8`, preserving the standalone Rust CLI.
  Keep core engine, storage, journal, and format code unchanged.
- Export one C ABI function that reads native OS arguments, accepts only
  `hatch`, and returns a negative sentinel for original 7-Zip commands.
  No shell or lossy filename conversion is involved.
- Dispatch before 7-Zip installs signal handlers. Catch Rust panics at
  the ABI boundary; intentional debug aborts still test real recovery.
- Query rustc for native static-library dependencies and link in order.
- Isolate C++ objects by OS, profile, and RAR policy; depend on the Rust
  archive so a changed engine triggers relinking.
- Default FOSS binaries exclude restricted RAR codecs. An explicit
  `--with-rar` build retains upstream functionality and original terms.
- Redirect original core/CLI tests via `SEVENZIP_HATCH_BINARY` to the actual
  C++ executable, then test native-created archives through both engines.

The result has two ZIP/7z decoder implementations, increasing binary size
and maintenance obligations. Replacing Rust codec adapters with 7-Zip
callbacks later would require a unit/stream/metadata bridge. Rewriting
the transaction engine in C++ first would create avoidable recovery risk.

## Future work, not implemented claims

1. Wire the session API into Windows File Manager/Explorer controls and
   expose persistent settings without building shell command strings.
2. Add restart/reclaim inside 7z blocks for huge files. LZMA2 full
   dictionary/state resets are candidates; arbitrary chunk boundaries
   are insufficient. Filters, BCJ2's streams, AES chaining, file offsets,
   and running CRC state must all be addressed before releasing input.
3. Prototype a compatible packing profile with deliberate resets and a
   checkpoint index before proposing additional 7z header properties.
   Smaller whole-file groups already work without a format extension.
4. Extend metadata fidelity and adapt completion/man generators for the
   embedded prefix.
5. Add `.tar.xz`, `.tar.zst`, native split ZIP, or RAR consumption only
   with decoder restart, reclamation, and recovery tests.
6. Expand constrained-volume, snapshot/clone, USB, and hard-drive tests.
   Upstream notes an intermittent APFS punch-reclamation issue; embedding
   the engine does not resolve that behavior.

No upstream speed, compression-ratio, or disk-space benchmark is presented
as a measurement of this fork.
