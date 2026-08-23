#!/usr/bin/env bash
# SpeedyColibri Installer for Linux and macOS
# Downloads the appropriate pre-compiled coli binary and installs it to ~/.local/bin or /usr/local/bin

set -euo pipefail

REPO="GriffinPilz/SpeedyColibri"

echo "=================================================="
echo " Installing SpeedyColibri (coli) for Unix/Linux/macOS"
echo "=================================================="

# 1. Detect OS and Architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux)
    if [[ "$ARCH" == "x86_64" ]]; then
      TARGET="x86_64-unknown-linux-gnu"
    elif [[ "$ARCH" == "aarch64" || "$ARCH" == "arm64" ]]; then
      TARGET="aarch64-unknown-linux-gnu"
    else
      echo "Error: Unsupported architecture $ARCH on Linux." >&2
      exit 1
    fi
    ;;
  darwin)
    if [[ "$ARCH" == "x86_64" ]]; then
      TARGET="x86_64-apple-darwin"
    elif [[ "$ARCH" == "arm64" || "$ARCH" == "aarch64" ]]; then
      TARGET="aarch64-apple-darwin"
    else
      echo "Error: Unsupported architecture $ARCH on macOS." >&2
      exit 1
    fi
    ;;
  *)
    echo "Error: Unsupported OS $OS." >&2
    exit 1
    ;;
esac

ASSET_NAME="coli-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"

# 2. Determine installation directory
if [[ -w "/usr/local/bin" ]]; then
  INSTALL_DIR="/usr/local/bin"
else
  INSTALL_DIR="${HOME}/.local/bin"
  mkdir -p "$INSTALL_DIR"
fi

# 3. Download and extract
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "[1/3] Downloading SpeedyColibri release package (${ASSET_NAME})..."
curl -sSL "$DOWNLOAD_URL" -o "${TMP_DIR}/${ASSET_NAME}"

echo "[2/3] Extracting binary..."
tar -xzf "${TMP_DIR}/${ASSET_NAME}" -C "$TMP_DIR"

echo "[3/3] Installing coli binary to ${INSTALL_DIR}..."
cp "${TMP_DIR}/coli" "${INSTALL_DIR}/coli"
chmod +x "${INSTALL_DIR}/coli"

# 4. PATH check
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
  echo ""
  echo "Note: ${INSTALL_DIR} is not in your current PATH."
  echo "Add it to your PATH by adding this line to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
  echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
fi

echo ""
echo "=================================================="
echo " SpeedyColibri installation complete!"
echo " Run 'coli --help' or 'coli serve <model>' to start."
echo "=================================================="
