#!/bin/bash
# Install touchless-mac (the ready-made build, hand models included) on an Apple Silicon Mac.
#
#   curl -fsSL https://raw.githubusercontent.com/qualixar/touchless-mac/main/install.sh | bash
#
# What it does, in order:
#   1. finds the latest release on GitHub (or the one in TOUCHLESS_VERSION, for example v0.1.1);
#   2. downloads it and checks it against the release's SHA-256 file;
#   3. unpacks it into ~/.local/share/touchless-mac/<version> and points "current" at it;
#   4. links ~/.local/bin/touchless-mac to it.
# Nothing needs sudo. To uninstall:  rm -rf ~/.local/share/touchless-mac ~/.local/bin/touchless-mac
set -euo pipefail

REPO="qualixar/touchless-mac"
HOME_DIR="${TOUCHLESS_HOME:-$HOME/.local/share/touchless-mac}"
BIN_DIR="${TOUCHLESS_BIN:-$HOME/.local/bin}"
# For testing a release before it is published: a folder URL that holds the archive and its .sha256.
BASE="${TOUCHLESS_DOWNLOAD_BASE:-}"

say() { printf '%s\n' "$*"; }
fail() { printf 'touchless-mac install: %s\n' "$*" >&2; exit 1; }

[ "$(uname -s)" = "Darwin" ] || fail "this app runs on macOS only."
if [ "$(uname -m)" != "arm64" ]; then
    fail "the ready-made build is for Apple Silicon (M1 or newer). On an Intel Mac, build from source:
  https://github.com/$REPO#quick-start"
fi
command -v curl >/dev/null || fail "curl is missing."
command -v shasum >/dev/null || fail "shasum is missing."

version="${TOUCHLESS_VERSION:-}"
if [ -z "$version" ]; then
    version="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
        | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1)"
    [ -n "$version" ] || fail "could not find the latest release. Set TOUCHLESS_VERSION=v0.1.1 and try again."
fi

name="touchless-mac-${version}-macos-arm64"
url="${BASE:-https://github.com/$REPO/releases/download/${version}}/${name}.tar.gz"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

say "Downloading touchless-mac $version ..."
curl -fsSL -o "$work/$name.tar.gz" "$url" || fail "download failed: $url"
curl -fsSL -o "$work/$name.tar.gz.sha256" "$url.sha256" || fail "checksum download failed."

say "Checking the SHA-256 fingerprint ..."
( cd "$work" && shasum -a 256 -c "$name.tar.gz.sha256" >/dev/null ) \
    || fail "the download does not match its fingerprint. Nothing was installed."

tar -xzf "$work/$name.tar.gz" -C "$work"
[ -x "$work/$name/touchless-mac" ] || fail "the archive has no touchless-mac program."
[ -f "$work/$name/models/palm.onnx" ] && [ -f "$work/$name/models/hand.onnx" ] \
    || fail "the archive has no hand models."

mkdir -p "$HOME_DIR" "$BIN_DIR"
rm -rf "${HOME_DIR:?}/$version"
mv "$work/$name" "$HOME_DIR/$version"
# Files fetched by a browser carry a quarantine flag; curl does not add one, but clear it to be sure.
xattr -dr com.apple.quarantine "$HOME_DIR/$version" 2>/dev/null || true
ln -sfn "$HOME_DIR/$version" "$HOME_DIR/current"
ln -sf "$HOME_DIR/current/touchless-mac" "$BIN_DIR/touchless-mac"

say ""
say "Installed touchless-mac $version"
say "  program: $BIN_DIR/touchless-mac"
say "  models:  $HOME_DIR/current/models"
case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *)
        say ""
        say "$BIN_DIR is not on your PATH yet. Add it with:"
        say "  echo 'export PATH=\"$BIN_DIR:\$PATH\"' >> ~/.zshrc && source ~/.zshrc"
        ;;
esac
say ""
say "Next:"
say "  touchless-mac doctor    checks the models, camera and permissions"
say "  touchless-mac           starts it (the first run asks you to point at four corners)"
