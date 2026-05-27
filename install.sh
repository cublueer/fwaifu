#!/bin/sh
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

BIN_NAME="fwaifu"
REPO_URL="https://github.com/cublueer/fwaifu"
CARGO_BIN_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"

# --- Check Rust toolchain ---
if ! command -v cargo >/dev/null 2>&1; then
    printf '%b\n' "${RED}Error: Rust toolchain not found (cargo command missing)${NC}" >&2
    printf '%b\n' "Install Rust: ${YELLOW}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
    exit 1
fi

# --- Check system dependencies ---
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

# --- Check if already installed ---
if command -v "$BIN_NAME" >/dev/null 2>&1; then
    existing_path=$(command -v "$BIN_NAME")
    printf '%b\n' "${YELLOW}${BIN_NAME} is already installed at: ${existing_path}${NC}"
    printf '%s' "Reinstall? [y/N] "
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

# --- Install from source ---
printf '%b\n' "${GREEN}Installing ${BIN_NAME} from source via cargo...${NC}"

if cargo install --git "$REPO_URL"; then
    printf '%b\n' "${GREEN}Installed successfully to ${CARGO_BIN_DIR}/${BIN_NAME}${NC}"
else
    # Primary install failed — offer fallback
    printf '%b\n' "${RED}Installation via 'cargo install --git' failed.${NC}"
    printf '%s' "${YELLOW}Try fallback (clone repo and build locally)? [Y/n] ${NC}"
    read -r REPLY
    case "$REPLY" in
        [Nn]|[Nn][Oo])
            printf '%b\n' "${RED}Installation aborted.${NC}" >&2
            exit 1
            ;;
    esac

    TMP_DIR=$(mktemp -d)
    cleanup() { rm -rf "$TMP_DIR"; }
    trap cleanup EXIT

    printf '%b\n' "${GREEN}Cloning ${REPO_URL}...${NC}"
    git clone "$REPO_URL" "$TMP_DIR/fwaifu" || {
        printf '%b\n' "${RED}Failed to clone repository.${NC}" >&2
        exit 1
    }

    printf '%b\n' "${GREEN}Building and installing from local source...${NC}"
    if ! (cd "$TMP_DIR/fwaifu" && cargo install --path .); then
        printf '%b\n' "${RED}Fallback installation also failed.${NC}" >&2
        exit 1
    fi

    printf '%b\n' "${GREEN}Installed successfully to ${CARGO_BIN_DIR}/${BIN_NAME}${NC}"
fi

# --- PATH check ---
case ":$PATH:" in
    *:"$CARGO_BIN_DIR":*) ;;
    *)
        printf '%b\n' "${YELLOW}Warning: ${CARGO_BIN_DIR} is not in your PATH${NC}"
        printf '%b\n' "Add this to your shell config: export PATH=\"${CARGO_BIN_DIR}:\$PATH\""
        ;;
esac

# --- Post-install usage hints ---
printf '\n%b\n' "${GREEN}Installation complete!${NC}"
printf '\n%b\n' "${YELLOW}Usage:${NC}"
printf '%b\n' "  ${BIN_NAME}              Show a random anime image + system info"
printf '%b\n' "  ${BIN_NAME} --help      Show all options"
printf '%b\n' "  ${BIN_NAME} --version   Show version"
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
