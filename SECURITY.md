# Security policy

## Supported versions

Security work targets `main` and the latest published desktop preview
(currently 0.2.0). Older previews do not receive separate backports.
This is a volunteer project; there is no guaranteed response or patch schedule.

Project version numbers differ from the bundled upstream versions. Include
the project version and package filename when reporting a problem.

## Report privately

Use [GitHub's private vulnerability report form](https://github.com/chai-in/7zip-hatch/security/advisories/new).
The repository has private vulnerability reporting enabled. Do not disclose
an unpatched vulnerability or sensitive archive in a public issue.

Include the affected version, OS and package or revision; reproduction steps;
expected and actual results; a minimal non-sensitive archive or proof of
concept; and whether ordinary or consuming extraction is involved. Explain
any likely impact, such as writes outside the output directory, unintended
data loss, password exposure, or code execution, if known.

Maintainers can request more information and coordinate a fix and advisory
through the private report. Disclosure and timing are agreed case by case.
Fixes may be coordinated with 7-Zip, HatchZip, or dependency maintainers where
their code is involved. There is no paid bounty program.

## Expected consuming behavior

Space-saving extraction deliberately changes and eventually removes its input
after confirmation. That documented behavior alone is not a vulnerability.
Unintended loss, traversal, overwrite, or recovery failure should be reported.
Use replaceable copies when investigating and preserve journals and logs.

Preview Windows packages are unsigned. macOS packages are ad hoc signed and
not notarized. Verify downloads using the same release's `SHA256SUMS`; see
[download verification](Hatch/DOWNLOADS.md#verify-a-download).
Ordinary bugs and usage questions use [support channels](SUPPORT.md).
