#!/bin/sh
set -e

# --- Colors ---
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

BIN_NAME="fwaifu"
BIN_PATH="/usr/bin/${BIN_NAME}"
CONFIG_DIR="${HOME}/.config/${BIN_NAME}"
CACHE_DIR="${HOME}/.cache/${BIN_NAME}"
PICTURES_DIR="${HOME}/Pictures/${BIN_NAME}"
FASTFETCH_CACHE="${HOME}/.cache/fastfetch"

# --- Banner ---
printf '%b\n' "${GREEN}========================================${NC}"
printf '%b\n' "${GREEN}          fwaifu Uninstaller${NC}"
printf '%b\n' "${GREEN}========================================${NC}"
printf '\n'

# --- Remove binary ---
printf '%b\n' "${YELLOW}Removing ${BIN_NAME} binary...${NC}"

if ! command -v sudo >/dev/null 2>&1; then
    printf '%b\n' "${RED}Error: sudo is required but not available.${NC}" >&2
    exit 1
fi

sudo rm -f "$BIN_PATH"
printf '%b\n' "${GREEN}Removed ${BIN_PATH}${NC}"
printf '\n'

# --- Interactive directory removal ---
prompt_remove() {
    dir="$1"
    msg="$2"
    if [ -d "$dir" ]; then
        printf '%s' "${YELLOW}${msg} [y/N] ${NC}"
        read -r REPLY
        case "$REPLY" in
            [Yy]|[Yy][Ee][Ss])
                rm -rf "$dir"
                printf '%b\n' "${GREEN}Removed ${dir}${NC}"
                ;;
            *)
                printf '%b\n' "${YELLOW}Kept ${dir}${NC}"
                ;;
        esac
    else
        printf '%b\n' "${YELLOW}${dir} not found (nothing to remove)${NC}"
    fi
    printf '\n'
}

prompt_remove "$CONFIG_DIR"         "Remove config directory?"
prompt_remove "$CACHE_DIR"          "Remove image cache?"
prompt_remove "$PICTURES_DIR"       "Remove saved images?"
prompt_remove "$FASTFETCH_CACHE"    "Remove fastfetch thumbnail cache? (shared with fastfetch)"

# --- Final summary ---
printf '%b\n' "${GREEN}========================================${NC}"
printf '%b\n' "${GREEN}          Uninstall complete!${NC}"
printf '%b\n' "${GREEN}========================================${NC}"
