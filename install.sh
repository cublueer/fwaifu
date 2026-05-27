#!/bin/sh
set -e

# --- Colors ---
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# --- Root user check ---
if [ "$(id -u)" -eq 0 ]; then
    printf '%b\n' "${YELLOW}Warning: Running as root is not recommended.${NC}"
    printf '%b\n' "The build step should run as a regular user."
    printf '%b' "${YELLOW}Force run as root? Type 'f' to continue, any other key to exit: ${NC}"
    read -r REPLY
    case "$REPLY" in
        [fF]) ;;
        *)
            printf '%b\n' "${GREEN}Aborted. Please run without sudo.${NC}"
            exit 0
            ;;
    esac
fi

BIN_NAME="fwaifu"
INSTALL_DIR="/usr/bin"
BIN_PATH="${INSTALL_DIR}/${BIN_NAME}"

# Determine source directory (directory containing this script)
SRC_DIR="$(cd "$(dirname "$0")" && pwd)"

# --- 1. Check cargo + Rust toolchain ---
if ! command -v cargo >/dev/null 2>&1; then
    printf '%b\n' "${RED}Error: Rust toolchain not found (cargo command missing)${NC}" >&2
    printf '%b\n' "Install Rust: ${YELLOW}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
    exit 1
fi

# Verify cargo actually works (requires a default toolchain configured).
# This catches the case where cargo exists (via rustup) but no default
# toolchain is set. If you see this while running as a regular user,
# run: rustup default stable
if ! cargo --version >/dev/null 2>&1; then
    printf '%b\n' "${RED}Error: cargo found but no Rust toolchain is configured${NC}" >&2
    printf '%b\n' "  Run: ${YELLOW}rustup default stable${NC} (as your regular user, not as root)"
    printf '%b\n' "  If you ran this script with sudo, try without sudo — only the final install step needs elevated permissions."
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
fi

if [ "$already_installed" -eq 1 ]; then
    printf '%b' "${YELLOW}Reinstall? [y/N] ${NC}"
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

# --- 4. Build from local source ---
if [ ! -f "$SRC_DIR/Cargo.toml" ]; then
    printf '%b\n' "${RED}Error: Cargo.toml not found in ${SRC_DIR}. Is this the project root?${NC}" >&2
    exit 1
fi

printf '%b\n' "${GREEN}Building ${BIN_NAME} from local source at ${SRC_DIR} (this may take a while)...${NC}"
(cd "$SRC_DIR" && cargo build --release) || {
    printf '%b\n' "${RED}Error: Build failed.${NC}" >&2
    exit 1
}

# --- 5. Install binary to /usr/bin/ ---
if ! command -v sudo >/dev/null 2>&1; then
    printf '%b\n' "${RED}Error: sudo is required to install to ${INSTALL_DIR} but is not available.${NC}" >&2
    exit 1
fi

printf '%b\n' "${GREEN}Installing ${BIN_NAME} to ${INSTALL_DIR}/ (sudo may prompt for password)...${NC}"
sudo mkdir -p "$INSTALL_DIR" && sudo cp "$SRC_DIR/target/release/$BIN_NAME" "$BIN_PATH" || {
    printf '%b\n' "${RED}Error: Failed to install to ${INSTALL_DIR}. Permission denied or sudo failed.${NC}" >&2
    exit 1
}

# --- 6. Install shell completions ---
printf '%b\n' "${GREEN}Installing shell completions...${NC}"

# bash completion
if command -v bash >/dev/null 2>&1; then
    BASH_COMPLETION_DIR="${HOME}/.local/share/bash-completion/completions"
    mkdir -p "$BASH_COMPLETION_DIR"
    "$BIN_PATH" --completion bash > "$BASH_COMPLETION_DIR/$BIN_NAME" 2>/dev/null && \
        printf '%b\n' "${GREEN}  bash: installed to ${BASH_COMPLETION_DIR}/${BIN_NAME}${NC}" || \
        printf '%b\n' "${YELLOW}  bash: failed${NC}"
fi

# zsh completion
if command -v zsh >/dev/null 2>&1; then
    ZSH_COMPLETION_DIR="${HOME}/.zsh/completion"
    mkdir -p "$ZSH_COMPLETION_DIR"
    "$BIN_PATH" --completion zsh > "$ZSH_COMPLETION_DIR/_$BIN_NAME" 2>/dev/null && \
        printf '%b\n' "${GREEN}  zsh: installed to ${ZSH_COMPLETION_DIR}/_${BIN_NAME}${NC}" || \
        printf '%b\n' "${YELLOW}  zsh: failed${NC}"
fi

# fish completion
if command -v fish >/dev/null 2>&1; then
    FISH_COMPLETION_DIR="${HOME}/.config/fish/completions"
    mkdir -p "$FISH_COMPLETION_DIR"
    "$BIN_PATH" --completion fish > "$FISH_COMPLETION_DIR/$BIN_NAME.fish" 2>/dev/null && \
        printf '%b\n' "${GREEN}  fish: installed to ${FISH_COMPLETION_DIR}/${BIN_NAME}.fish${NC}" || \
        printf '%b\n' "${YELLOW}  fish: failed${NC}"
fi

printf '\n'

# --- 7. Print usage ---
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
