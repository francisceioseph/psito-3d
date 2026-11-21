#!/bin/bash
# Builds the APT repository as static files from the .deb files, and signs the
# indexes with the key in gpg. The CI publishes the result on GitHub Pages.
# Usage: tool/build-apt-repo.sh <output folder> <file.deb>...
set -e

OUT="$1"
shift
[ -n "$OUT" ] && [ $# -gt 0 ] || {
    echo "Usage: $0 <output folder> <file.deb>..." >&2
    exit 1
}

POOL="$OUT/pool/main/p/psito-3d"
DIST="$OUT/dists/stable"
BINARY="$DIST/main/binary-amd64"
mkdir -p "$POOL" "$BINARY"
cp "$@" "$POOL/"

(cd "$OUT" && apt-ftparchive packages pool) > "$BINARY/Packages"
gzip -9nkf "$BINARY/Packages"

apt-ftparchive \
    -o APT::FTPArchive::Release::Origin="Psito 3D" \
    -o APT::FTPArchive::Release::Label="Psito 3D" \
    -o APT::FTPArchive::Release::Suite=stable \
    -o APT::FTPArchive::Release::Codename=stable \
    -o APT::FTPArchive::Release::Architectures=amd64 \
    -o APT::FTPArchive::Release::Components=main \
    release "$DIST" > "$DIST/Release.tmp"
mv "$DIST/Release.tmp" "$DIST/Release"

gpg --batch --yes --clearsign -o "$DIST/InRelease" "$DIST/Release"
gpg --batch --yes --armor --detach-sign -o "$DIST/Release.gpg" "$DIST/Release"
gpg --batch --yes --armor --export > "$OUT/psito-3d.asc"
