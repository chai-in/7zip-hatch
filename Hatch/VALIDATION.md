# Validation

Local validation on macOS arm64, 24 October 2026:

- Official 7-Zip 26.05 compiled from the pinned source.
- Original HatchZip core/CLI suite: 104 tests passed.
- The same core/CLI suite through the combined C++ executable: 104 passed.
- Combined-executable integration suite: 8 tests passed, including deliberate
  process aborts followed by resume and restore.
- Release executable: integration tests passed, with the debug-only crash
  case skipped as intended.
- Rust formatting and Clippy passed for the core/CLI, including test targets.
- The vendored core directory matches the upstream snapshot byte for byte.

The integration cases create disposable archives with the fork's native
7-Zip encoder, exercise the embedded HatchZip reader/extractor, and compare
output SHA-256 values. They cover ZIP, tar, gzip tar, solid 7z, Unicode and
spaces in paths, AES-encrypted 7z headers, incorrect passwords, byte-split
volumes opened from a middle part, insufficient space, collisions, missing
unattended confirmation, corrupt data, path traversal, and crash boundaries.
Restored 7z remainders are checked and extracted by the native 7-Zip engine.

The [CI workflow](https://github.com/chai-in/7zip-hatch/actions) runs on Linux, macOS, and Windows, building
both debug and release executables. Consult its actual status for those
platforms. Local tests do not establish Windows/Linux behavior, measured
near-full-volume performance, exact memory bounds, or immunity to hardware
failure. No new minimum-space or throughput benchmark is claimed.
