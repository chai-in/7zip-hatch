# Project images

- `header.svg` is the README's editable vector banner. Its archive-box
  illustration is decorative artwork, not an application screenshot or an
  upstream project logo.
- `desktop-linux.png` is an unmodified screenshot of the published 0.2.0
  AppImage opening a disposable ZIP archive during
  [installer CI](https://github.com/chai-in/7zip-hatch/actions/runs/40036590961).
  It shows real application behavior, rather than a product mockup.
- `social-preview.svg` is the editable vector layout for the repository's
  social card. Its PNG export is `social-preview.png`, at 1280 by 640 pixels.
  The SVG embeds the original icon so it can render without network access.
  Use an SVG-capable renderer/editor and preserve the canvas dimensions.

The archive icon is the original HatchZip icon from
[`third_party/hatchzip/assets/icon`](../../third_party/hatchzip/assets/icon),
used under the upstream [MIT terms](../../third_party/hatchzip/LICENSE-MIT).
It is credited to HatchZip's contributors. It is not an official 7-Zip logo.
The card identifies this project as an independent fork.

Keep screenshots and preview images free of private archives, personal paths,
tokens, and passwords. Update product/version claims alongside the relevant
documentation. Do not use a rendered image to imply an unimplemented feature.
