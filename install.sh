#!/bin/sh
set -eu

REPOSITORY='horizzon3507/handoff'
ASSET='handoff-linux-x86_64'

if [ "$(uname -s)" != 'Linux' ] || [ "$(uname -m)" != 'x86_64' ]; then
    printf '%s\n' 'The prebuilt installer currently supports Linux x86_64.' >&2
    exit 1
fi

if ! command -v curl >/dev/null 2>&1 || ! command -v sha256sum >/dev/null 2>&1; then
    printf '%s\n' 'Install curl and sha256sum, then run this installer again.' >&2
    exit 1
fi

INSTALL_DIR=${HANDOFF_INSTALL_DIR:-"$HOME/.local/bin"}
DOWNLOAD_DIR=$(mktemp -d)
trap 'rm -rf "$DOWNLOAD_DIR"' EXIT HUP INT TERM

BASE_URL="https://github.com/$REPOSITORY/releases/latest/download"
curl --fail --location --silent --show-error "$BASE_URL/$ASSET" -o "$DOWNLOAD_DIR/$ASSET"
curl --fail --location --silent --show-error "$BASE_URL/$ASSET.sha256" -o "$DOWNLOAD_DIR/$ASSET.sha256"

(
    cd "$DOWNLOAD_DIR"
    sha256sum --check "$ASSET.sha256"
)

mkdir -p "$INSTALL_DIR"
install -m 0755 "$DOWNLOAD_DIR/$ASSET" "$INSTALL_DIR/handoff"
printf 'Installed Handoff to %s/handoff\n' "$INSTALL_DIR"
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) printf 'Add %s to PATH, then run: handoff\n' "$INSTALL_DIR" ;;
esac
