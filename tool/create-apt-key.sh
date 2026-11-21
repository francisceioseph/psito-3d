#!/bin/bash
# Creates the key that signs the APT and DNF repositories and stores the private
# part in the GitHub secrets, as APT_SIGNING_KEY. No passphrase, because the CI
# signs. Run it once: a new key would make everyone who installed add it again.
# Usage: tool/create-apt-key.sh
set -e

BACKUP="$HOME/psito-3d-apt-key.asc"
[ ! -e "$BACKUP" ] || {
    echo "$BACKUP already exists: the key was already created." >&2
    exit 1
}

GNUPGHOME="$(mktemp -d)"
export GNUPGHOME
trap 'rm -rf "$GNUPGHOME"' EXIT

gpg --batch --quiet --passphrase '' --quick-generate-key \
    'Psito 3D packages <francisceioseph@users.noreply.github.com>' \
    ed25519 sign never

(umask 077 && gpg --batch --armor --export-secret-keys > "$BACKUP")
gh secret set APT_SIGNING_KEY --repo francisceioseph/psito-3d < "$BACKUP"

echo "Key created and sent to GitHub."
echo "Keep a copy of $BACKUP off this computer."
