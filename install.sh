#!/bin/sh
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

REPO="cublueer/fwaifu"
BIN_NAME="fwaifu"
DEFAULT_VERSION="1.0.0"
VERSION="${1:-v${DEFAULT_VERSION}}"
VERSION_NUM="${VERSION#v}"

# --- Architecture detection ---
ARCH=$(uname -m)
case "$ARCH" in
    x86_64|amd64)  TARGET="x86_64-unknown-linux-gnu" ;;
    aarch64|arm64) TARGET="aarch64-unknown-linux-gnu" ;;
    *)
        printf '%b\n' "${RED}Error: Unsupported architecture: $ARCH${NC}" >&2
        exit 1
        ;;
esac

# --- Musl detection (Alpine Linux) ---
if ldd --version 2>&1 | grep -qi musl 2>/dev/null; then
    TARGET="${TARGET%-gnu}-musl"
fi

printf '%b\n' "${GREEN}Installing ${BIN_NAME} ${VERSION} for ${TARGET}...${NC}"

# --- Download URL ---
URL="https://github.com/${REPO}/releases/download/${VERSION}/fwaifu-${VERSION_NUM}-${TARGET}.tar.gz"
TMP_DIR=$(mktemp -d)
TAR_FILE="${TMP_DIR}/fwaifu.tar.gz"

cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# --- Download ---
printf '%b\n' "Downloading from ${URL}..."
if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$URL" -o "$TAR_FILE"
elif command -v wget >/dev/null 2>&1; then
    wget -q "$URL" -O "$TAR_FILE"
else
    printf '%b\n' "${RED}Error: curl or wget required${NC}" >&2
    exit 1
fi

# --- Extract ---
printf '%b\n' "Extracting..."
tar xzf "$TAR_FILE" -C "$TMP_DIR"

# --- Locate binary (handles nested archive layouts) ---
BIN_PATH=$(find "$TMP_DIR" -name "$BIN_NAME" -type f | head -1)
if [ -z "$BIN_PATH" ]; then
    printf '%b\n' "${RED}Error: binary '${BIN_NAME}' not found in archive${NC}" >&2
    exit 1
fi

# --- Install ---
INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "$INSTALL_DIR"
cp -f "$BIN_PATH" "${INSTALL_DIR}/${BIN_NAME}"
chmod +x "${INSTALL_DIR}/${BIN_NAME}"

printf '%b\n' "${GREEN}Installed to ${INSTALL_DIR}/${BIN_NAME}${NC}"

# --- PATH check ---
case ":$PATH:" in
    *:"$INSTALL_DIR":*) ;;
    *)
        printf '%b\n' "${YELLOW}Warning: ${INSTALL_DIR} is not in your PATH${NC}"
        printf '%b\n' "Add this to your shell config: export PATH=\"\$HOME/.local/bin:\$PATH\""
        ;;
esac

# --- Post-install ---
cat <<EOF

${GREEN}Installation complete!${NC}

${YELLOW}Dependencies:${NC}
  Required: fastfetch (https://github.com/fastfetch-cli/fastfetch)
  Optional: ImageMagick (for image cropping)

Verify installation: ${BIN_NAME} --version
EOF
