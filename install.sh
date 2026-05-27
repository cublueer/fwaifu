#!/bin/sh
set -e

# --- Colors ---
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

BIN_NAME="fwaifu"
REPO_URL="https://github.com/cublueer/fwaifu"
INSTALL_DIR="/opt/fwaifu"
LOCAL_BIN_DIR="${HOME}/.local/bin"
BIN_PATH="${INSTALL_DIR}/${BIN_NAME}"
SYMLINK_PATH="${LOCAL_BIN_DIR}/${BIN_NAME}"

# --- 1. Check cargo exists ---
if ! command -v cargo >/dev/null 2>&1; then
    printf '%b\n' "${RED}Error: Rust toolchain not found (cargo command missing)${NC}" >&2
    printf '%b\n' "Install Rust: ${YELLOW}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
    exit 1
fi

# --- 2. Check system dependencies ---
missing_fastfetch=0
if ! command -v fastfetch >/dev/null 2>&1; then
    missing_fastfetch=1
    printf '%b\n' "${YELLOW}Warning: fastfetch is not installed (required dependency)${NC}"
    printf '%b\n' "  Install: https://github.com/fastfetch-cli/fastfetch"
fi

has_imagemagick=0
if command -v magick >/dev/null 2>&1 || command -v convert >/dev/null 2>&1; then
    has_imagemagick=1
    printf '%b\n' "${GREEN}ImageMagick detected (optional: image cropping support)${NC}"
fi

# --- 3. Check if already installed ---
already_installed=0
if [ -f "$BIN_PATH" ]; then
    already_installed=1
    printf '%b\n' "${YELLOW}Found existing installation at ${BIN_PATH}${NC}"
elif [ -h "$SYMLINK_PATH" ] && [ "$(readlink "$SYMLINK_PATH")" = "$BIN_PATH" ]; then
    already_installed=1
    printf '%b\n' "${YELLOW}Found existing symlink at ${SYMLINK_PATH} -> ${BIN_PATH}${NC}"
fi

if [ "$already_installed" -eq 1 ]; then
    printf '%s' "${YELLOW}Reinstall? [y/N] ${NC}"
    read -r REPLY
    case "$REPLY" in
        [Yy]|[Yy][Ee][Ss])
            printf '%b\n' "${GREEN}Proceeding with reinstall...${NC}"
            ;;
        *)
            printf '%b\n' "${GREEN}Skipping installation.${NC}"
            exit 0
            ;;
    esac
fi

# --- 4. Clone and build ---
TMP_DIR=$(mktemp -d)
cleanup() {
    [ -n "$TMP_DIR" ] && [ -d "$TMP_DIR" ] && rm -rf "$TMP_DIR"
}
trap cleanup EXIT

printf '%b\n' "${GREEN}Cloning ${REPO_URL}...${NC}"
git clone "$REPO_URL" "$TMP_DIR/fwaifu" || {
    printf '%b\n' "${RED}Error: Failed to clone repository.${NC}" >&2
    exit 1
}

printf '%b\n' "${GREEN}Building ${BIN_NAME} (this may take a while)...${NC}"
(cd "$TMP_DIR/fwaifu" && cargo build --release) || {
    printf '%b\n' "${RED}Error: Build failed.${NC}" >&2
    exit 1
}

# --- 5. Install binary to /opt/fwaifu/ ---
if ! command -v sudo >/dev/null 2>&1; then
    printf '%b\n' "${RED}Error: sudo is required to install to ${INSTALL_DIR} but is not available.${NC}" >&2
    exit 1
fi

printf '%b\n' "${GREEN}Installing ${BIN_NAME} to ${INSTALL_DIR}/ (sudo may prompt for password)...${NC}"
sudo mkdir -p "$INSTALL_DIR" && sudo cp "$TMP_DIR/fwaifu/target/release/$BIN_NAME" "$BIN_PATH" || {
    printf '%b\n' "${RED}Error: Failed to install to ${INSTALL_DIR}. Permission denied or sudo failed.${NC}" >&2
    exit 1
}

# --- 6. Create symlink in ~/.local/bin/ ---
mkdir -p "$LOCAL_BIN_DIR"
ln -sf "$BIN_PATH" "$SYMLINK_PATH"
printf '%b\n' "${GREEN}Symlink created: ${SYMLINK_PATH} -> ${BIN_PATH}${NC}"

# --- 7. PATH check ---
case ":$PATH:" in
    *:"$LOCAL_BIN_DIR":*)
        ;;
    *)
        printf '%b\n' "${YELLOW}Warning: ${LOCAL_BIN_DIR} is not in your PATH${NC}"
        printf '%b\n' "Add to your shell config: export PATH=\"${LOCAL_BIN_DIR}:\$PATH\""
        ;;
esac

# --- 8. Print usage ---
printf '\n%b\n' "${GREEN}Installation complete!${NC}"
printf '\n%b\n' "${YELLOW}Usage:${NC}"
printf '%b\n' "  ${BIN_NAME}              Show a random anime image + system info"
printf '%b\n' "  ${BIN_NAME} --help       Show all options"
printf '%b\n' "  ${BIN_NAME} --version    Show version"
printf '\n%b\n' "${YELLOW}Dependencies:${NC}"

if [ "$missing_fastfetch" -eq 1 ]; then
    printf '%b\n' "  Required: fastfetch ${RED}(not found)${NC}"
else
    printf '%b\n' "  Required: fastfetch ${GREEN}(found)${NC}"
fi

if [ "$has_imagemagick" -eq 1 ]; then
    printf '%b\n' "  Optional: ImageMagick ${GREEN}(found)${NC}"
else
    printf '%b\n' "  Optional: ImageMagick (not installed — image cropping unavailable)"
fi

printf '\n%b\n' "Config: ~/.config/fwaifu/config.toml"
