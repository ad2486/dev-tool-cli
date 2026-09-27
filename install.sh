#!/bin/sh
set -eu

REPO="ad2486/dev-tool-cli"
INSTALL_DIR="${DEV_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${DEV_VERSION:-latest}"

fail() {
    echo "error: $1" >&2
    exit 1
}

case "$(uname -s)" in
    Darwin) os="apple-darwin" ;;
    Linux) os="unknown-linux-musl" ;;
    *) fail "unsupported system $(uname -s); dev supports macOS and Linux" ;;
esac

case "$(uname -m)" in
    x86_64 | amd64) arch="x86_64" ;;
    arm64 | aarch64) arch="aarch64" ;;
    *) fail "unsupported processor $(uname -m)" ;;
esac

command -v curl >/dev/null || fail "curl is required"

archive="dev-$arch-$os.tar.gz"
if [ "$VERSION" = "latest" ]; then
    url="https://github.com/$REPO/releases/latest/download/$archive"
else
    url="https://github.com/$REPO/releases/download/$VERSION/$archive"
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "downloading $archive ($VERSION)"
curl -fsSL "$url" -o "$tmp/$archive" || fail "could not download $url"
curl -fsSL "$url.sha256" -o "$tmp/$archive.sha256" || fail "could not download $url.sha256"

cd "$tmp"
if command -v sha256sum >/dev/null; then
    sha256sum -c "$archive.sha256" >/dev/null || fail "checksum mismatch for $archive"
else
    shasum -a 256 -c "$archive.sha256" >/dev/null || fail "checksum mismatch for $archive"
fi

tar -xzf "$archive"
mkdir -p "$INSTALL_DIR"
mv dev "$INSTALL_DIR/dev"
chmod +x "$INSTALL_DIR/dev"

echo "installed dev to $INSTALL_DIR/dev"
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) echo "add $INSTALL_DIR to your PATH, e.g.: export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac
