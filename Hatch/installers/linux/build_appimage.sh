#!/bin/bash
# Wrap the tested portable distribution. Run on Ubuntu 24.04 x86-64.
set -euo pipefail
root=$(cd "$(dirname "$0")/../../.." && pwd)
payload=$(realpath "${1:?Pass the unpacked 7zip-hatch desktop folder}")
tools="$root/build/installers/tools"
appdir="$root/build/installers/7-Zip-Hatch.AppDir"
packages="$root/build/packages"
id=io.github.chai-in.7zip-hatch
mkdir -p "$tools" "$packages"

fetch() {
    curl -fL --retry 3 --proto '=https' "$2" -o "$tools/$1"
    echo "$3  $tools/$1" | sha256sum --check -
}
fetch linuxdeploy https://github.com/linuxdeploy/linuxdeploy/releases/download/1-alpha-20251107-1/linuxdeploy-x86_64.AppImage c20cd71e3a4e3b80c3483cef793cda3f4e990aca14014d23c544ca3ce1270b4d
fetch appimagetool https://github.com/AppImage/appimagetool/releases/download/1.9.1/appimagetool-x86_64.AppImage ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0
fetch runtime https://github.com/AppImage/type2-runtime/releases/download/20251108/runtime-x86_64 2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d
chmod +x "$tools/linuxdeploy" "$tools/appimagetool"
export APPIMAGE_EXTRACT_AND_RUN=1 NO_STRIP=1
if [ -e "$appdir" ]; then echo "Choose a clean AppDir: $appdir" >&2; exit 1; fi
mkdir -p "$appdir/usr/bin" "$appdir/usr/share/doc/7zip-hatch"
cp -a "$payload/7zip-hatch" "$payload/7zz" "$appdir/usr/bin/"
cp -a "$payload/Licenses" "$payload/README.md" "$payload/INSTALLERS.md" "$payload/INSTALLER-PAYLOAD.json" "$appdir/usr/share/doc/7zip-hatch/"
cp "$root/third_party/hatchzip/assets/icon/png/hatchzip-256.png" "$tools/$id.png"
# winit loads keyboard and Wayland libraries with dlopen, so ldd cannot find
# them. Also explicitly include the font and C++ libraries excluded by
# linuxdeploy's default system-library list. Graphics drivers stay on the host.
libraries=()
for name in libfontconfig.so.1 libfreetype.so.6 libexpat.so.1 libz.so.1 \
            libstdc++.so.6 libgcc_s.so.1 libxkbcommon.so.0 libxkbcommon-x11.so.0 \
            libwayland-client.so.0 libwayland-cursor.so.0 libwayland-egl.so.1; do
    library=$(ldconfig -p | awk -v name="$name" '$1 == name && /x86-64/ { print $NF }')
    if [ -z "$library" ]; then echo "Missing desktop library: $name" >&2; exit 1; fi
    libraries+=(--library "$library")
done
"$tools/linuxdeploy" --appdir "$appdir" \
    --executable "$appdir/usr/bin/7zip-hatch" --executable "$appdir/usr/bin/7zz" \
    "${libraries[@]}" \
    --desktop-file "$root/Hatch/installers/linux/$id.desktop" \
    --icon-file "$tools/$id.png"
# Its default AppRun is a symlink to the GUI. Replace the link itself.
rm -f "$appdir/AppRun"
cp "$root/Hatch/installers/linux/AppRun" "$appdir/AppRun"
chmod +x "$appdir/AppRun"
python3 "$root/Hatch/installers/linux/collect_sources.py" "$appdir"
desktop-file-validate "$appdir/$id.desktop"
ARCH=x86_64 VERSION=0.2.0 "$tools/appimagetool" --runtime-file "$tools/runtime" \
    "$appdir" "$packages/7zip-hatch-0.2.0-linux-x86_64.AppImage"
chmod +x "$packages/7zip-hatch-0.2.0-linux-x86_64.AppImage"
