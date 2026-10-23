# Contributing

Contributions are welcome through issues and pull requests in
[chai-in/7zip-hatch](https://github.com/chai-in/7zip-hatch). This is an
independent fork. Report fork-specific behavior here rather than assuming
the upstream projects own it. Follow the [code of conduct](CODE_OF_CONDUCT.md).

## Start here

- Search existing issues before reporting a bug. Include the package version,
  OS, archive format, chosen operation, reproduction steps, and expected result.
- Use [Discussions](https://github.com/chai-in/7zip-hatch/discussions) for
  questions and early design ideas.
- Report vulnerabilities [privately](SECURITY.md). Do not post private archive
  contents, passwords, tokens, or personal filesystem paths in public logs.
- Discuss substantial architecture, format, dependency, or recovery changes
  before spending time on a large implementation.

Documentation and accessibility work are useful contributions. Maintainers
apply `good first issue` only when an issue has enough scope and context for
a newcomer; the label is not a blanket promise of difficulty.

## Repository layout

| Path | Responsibility |
| --- | --- |
| `C/`, `CPP/` | Native 7-Zip engine and Windows File Manager |
| `Hatch/` | Integration, build/packaging scripts, tests, and fork documentation |
| `third_party/hatchzip/crates/core/` | Vendored extraction, storage, journal, and archive formats |
| `third_party/hatchzip/crates/gui/` | Slint desktop frontend |
| `third_party/hatchzip/crates/shell/` | User-level desktop context menus |
| `.github/` | Community templates, maintenance automation, and CI |

Read [the design study](Hatch/DESIGN.md) and the applicable
[`AGENTS.md`](third_party/hatchzip/AGENTS.md) before changing vendored HatchZip
code. Preserve upstream notices and keep unrelated upstream formatting intact.

## Build

Install Python 3, Rust 1.95.0 through rustup, and the platform's C/C++ tools.
Linux requires GCC/make, `pkg-config`, and Fontconfig development headers.
macOS requires Xcode Command Line Tools. On Windows, use a Visual Studio x64
Native Tools prompt with MSVC and nmake.

```sh
git clone https://github.com/chai-in/7zip-hatch.git
cd 7zip-hatch
python3 Hatch/build.py --debug --jobs 4
python3 Hatch/build_desktop.py --debug --skip-engine --jobs 4
```

Use `python` on Windows if that is the installed command. The build uses
locked Cargo dependencies. See [desktop builds](Hatch/DESKTOP.md) and
[installer builds](Hatch/installers/README.md) for outputs and release profiles.

## Checks

From the repository root:

```sh
python3 Hatch/check_repository.py
python3 Hatch/test_packaging.py
python3 Hatch/installers/test_payload.py
python3 Hatch/test_integration.py build/debug/7zz --crash-tests
```

For Rust changes, from `third_party/hatchzip`:

```sh
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
SEVENZIP_HATCH_BINARY="$(pwd)/../../build/debug/7zz" \
  CARGO_TARGET_DIR="$(pwd)/../../build/hatch-target" \
  cargo test --locked --workspace
```

On Windows, use `7zz.exe` and PowerShell's `$env:NAME = 'value'` syntax.
CI runs the native and Rust checks on Linux, macOS, and Windows. Installer
changes also need the real install/launch checks in the installer workflow.
Documentation changes run the repository checks without rebuilding unchanged
engines. Keep any generated screenshots free of private data.

## Archive safety and compatibility

The core invariant is **durable output, then journal commit, then source
truncation**. The journal must never claim output that is not durable, and
the archive must never be truncated beyond committed output.

Use disposable, reproducible archives for all consuming-mode tests. Compare
output byte for byte. Changes to storage, journal, recovery, passwords, or
archive parsing need meaningful failure and resume cases. Test solid-block
boundaries rather than assuming that compression block settings impose a
hard limit on space use.

Preserve standard 7-Zip command behavior and archive compatibility. Keep
platform-specific behavior in the platform modules described by the vendored
instructions. Avoid introducing new archive formats as an incidental change.

## Pull requests

Create a focused branch and describe the problem, resulting behavior, checks
run, and any format/recovery impact. Include a minimal fixture where useful.
Keep unrelated refactoring out of the change. Do not use skip-CI directives
on a pull request that changes code or workflows.

Maintainers review changes and merge with a squash commit after required
checks pass and review conversations are resolved. The project currently
has one maintainer, [@chai-in](https://github.com/chai-in); an extra approving
review is not required until another reviewer is available. This avoids
requiring maintainers to approve their own contributions.

## Licensing

Keep existing notices. Contributions use the applicable existing component
terms; this project does not require a separate contributor license agreement.
The complete desktop application is GPL-3.0, native 7-Zip components retain
LGPL terms and original exceptions, and original HatchZip sources retain
MIT OR Apache-2.0. See [the license map](README.md#free-software-and-attribution).
New dependencies need compatible terms, bundled notices, and corresponding
source where required. Dependency updates go through review and CI.
