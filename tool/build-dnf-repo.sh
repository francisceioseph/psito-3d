#!/bin/bash
# Builds the DNF repository as static files from the .rpm files, signing the
# packages and the index with the key in gpg. The CI publishes it on GitHub Pages.
# Usage: tool/build-dnf-repo.sh <output folder> <public URL> <file.rpm>...
set -e

OUT="$1"
URL="$2"
shift 2 || true
[ -n "$OUT" ] && [ -n "$URL" ] && [ $# -gt 0 ] || {
    echo "Usage: $0 <output folder> <public URL> <file.rpm>..." >&2
    exit 1
}

mkdir -p "$OUT/rpm"
cp "$@" "$OUT/rpm/"

KEY="$(gpg --list-secret-keys --with-colons | awk -F: '/^fpr/ { print $10; exit }')"
rpmsign --addsign --define "_gpg_name $KEY" --define "__gpg $(command -v gpg)" "$OUT"/rpm/*.rpm >/dev/null

createrepo_c --quiet "$OUT/rpm"
gpg --batch --yes --armor --detach-sign -o "$OUT/rpm/repodata/repomd.xml.asc" "$OUT/rpm/repodata/repomd.xml"
gpg --batch --yes --armor --export > "$OUT/psito-3d.asc"

cat > "$OUT/psito-3d.repo" <<REPO
[psito-3d]
name=Psito 3D
baseurl=$URL/rpm
enabled=1
gpgcheck=1
repo_gpgcheck=1
gpgkey=$URL/psito-3d.asc
REPO
