# Release maintenance

## Prepare

1. Decide whether the version is a preview or stable release. Keep preview
   status until supported platforms and recovery behavior justify otherwise.
2. Update app/package versions, changelog, payload receipts, download links,
   requirements, and known limits together. Record upstream revisions in NOTICE.
3. Finish changes through a pull request. Require native CI for code or build
   changes, repository checks, relevant installer checks, and review of security
   alerts. Never test consuming extraction on irreplaceable data.
4. Tag the exact final source revision. Do not move a published version tag.

## Build and verify

Use the pinned Rust toolchain, Cargo lockfile, explicit upstream source
revisions, and SHA-pinned packaging tools. Preserve all licenses and notices.
Default FOSS artifacts exclude the unRAR-restricted codecs.

Build and test the platform packages on their target OS. Installer checks
must install, run, upgrade/reinstall, and remove Windows setup; launch the
AppImage, verify the bundled engines and menus, and open a real archive. Keep
the Mac signing/notarization state and Linux glibc floor explicit.

Export matching application and locked dependency sources with
`python3 Hatch/package_source.py`. Include corresponding sources and notices
for libraries added to binary packages. Verify SOURCE-COMMIT.txt and each
INSTALLER-PAYLOAD.json against build artifacts and CI revisions.

## Publish

GitHub release immutability is enabled. The published 0.2.0 preview is also
immutable. Published assets cannot be added, replaced, or removed, and the
release tag cannot be moved or reused. Finish the draft and verify every
asset before publishing.

1. Create a draft release on the verified tag. Upload tested artifacts and
   matching source bundles; do not rebuild an untested local binary for upload.
2. Produce SHA256SUMS from final files. Verify GitHub's uploaded sizes and
   SHA-256 asset digests, then fetch the manifest and compare it again.
3. Write release notes with direct platform downloads, requirements, signing
   status, source provenance, known limits, and links to successful CI runs.
4. Publish with the chosen preview/stable status. Test the public download
   links and update the README. Do not promote a preview just to make it appear
   under GitHub's stable `releases/latest` URL.

Published tags and assets are immutable. Correct a binary, missing source,
or bundled notice with a new release version. Titles and release notes can
be clarified without replacing assets; keep their claims tied to the exact
published files. The 0.2.0 installers and source additions were completed
before the preview became immutable. Their recorded revisions are explained
in [download provenance](../Hatch/DOWNLOADS.md#source-and-build-provenance).

Do not claim reproducible bit-for-bit builds, signing, certification, platform
coverage, or performance improvements without evidence for those exact artifacts.
