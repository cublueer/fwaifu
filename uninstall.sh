#!/bin/sh
set -e

# --- Language detection (same logic as src/i18n.rs) ---
detect_lang() {
    case "${LANG:-}" in
        zh_*) echo "zh" ;;
        *)    echo "en" ;;
    esac
}
CUR_LANG=$(detect_lang)

t() {
    eval "printf '%b' \"\$MSG_${1}_${CUR_LANG}\""
}

# --- Colors ---
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# --- Variables ---
BIN_NAME="fwaifu"
BIN_PATH="/usr/bin/${BIN_NAME}"
CONFIG_DIR="${HOME}/.config/${BIN_NAME}"
CACHE_DIR="${HOME}/.cache/${BIN_NAME}"
PICTURES_DIR="${HOME}/Pictures/${BIN_NAME}"
FASTFETCH_CACHE="${HOME}/.cache/fastfetch"

# --- Messages ---
MSG_uninstaller_zh="          fwaifu 卸载程序"
MSG_uninstaller_en="          fwaifu Uninstaller"

MSG_removing_binary_zh="${YELLOW}正在删除 ${BIN_NAME} 二进制文件...${NC}"
MSG_removing_binary_en="${YELLOW}Removing ${BIN_NAME} binary...${NC}"

MSG_err_sudo_required_zh="${RED}错误: 需要 sudo 但 sudo 不可用。${NC}"
MSG_err_sudo_required_en="${RED}Error: sudo is required but not available.${NC}"

MSG_removed_binary_zh="${GREEN}已删除 ${BIN_PATH}${NC}"
MSG_removed_binary_en="${GREEN}Removed ${BIN_PATH}${NC}"

MSG_removed_completions_zh="${YELLOW}已删除 Shell 补全（如有）${NC}"
MSG_removed_completions_en="${YELLOW}Removed shell completions (if any)${NC}"

MSG_prompt_config_zh="删除配置目录？"
MSG_prompt_config_en="Remove config directory?"

MSG_prompt_cache_zh="删除图片缓存？"
MSG_prompt_cache_en="Remove image cache?"

MSG_prompt_pictures_zh="删除已保存的图片？"
MSG_prompt_pictures_en="Remove saved images?"

MSG_prompt_ff_cache_zh="删除 fastfetch 缩略图缓存？（与 fastfetch 共享）"
MSG_prompt_ff_cache_en="Remove fastfetch thumbnail cache? (shared with fastfetch)"

MSG_removed_zh="已删除"
MSG_removed_en="Removed"

MSG_kept_zh="已保留"
MSG_kept_en="Kept"

MSG_not_found_zh="未找到（无需删除）"
MSG_not_found_en="not found (nothing to remove)"

MSG_uninstall_complete_zh="          卸载完成！"
MSG_uninstall_complete_en="          Uninstall complete!"

# --- Banner ---
printf '%b\n' "${GREEN}========================================${NC}"
printf '%b\n' "${GREEN}$(t uninstaller)${NC}"
printf '%b\n' "${GREEN}========================================${NC}"
printf '\n'

# --- Remove binary ---
printf '%b\n' "$(t removing_binary)"

if ! command -v sudo >/dev/null 2>&1; then
    printf '%b\n' "$(t err_sudo_required)" >&2
    exit 1
fi

sudo rm -f "$BIN_PATH"
printf '%b\n' "$(t removed_binary)"
printf '\n'

# --- Remove shell completions ---
rm -f "${HOME}/.local/share/bash-completion/completions/${BIN_NAME}"
rm -f "${HOME}/.zsh/completion/_${BIN_NAME}"
rm -f "${HOME}/.config/fish/completions/${BIN_NAME}.fish"
printf '%b\n' "$(t removed_completions)"
printf '\n'

# --- Interactive directory removal ---
prompt_remove() {
    dir="$1"
    msg_key="$2"
    if [ -d "$dir" ]; then
        printf '%s' "${YELLOW}$(t ${msg_key}) [y/N] ${NC}"
        read -r REPLY
        case "$REPLY" in
            [Yy]|[Yy][Ee][Ss])
                rm -rf "$dir"
                printf '%b\n' "${GREEN}$(t removed) ${dir}${NC}"
                ;;
            *)
                printf '%b\n' "${YELLOW}$(t kept) ${dir}${NC}"
                ;;
        esac
    else
        printf '%b\n' "${YELLOW}${dir} $(t not_found)${NC}"
    fi
    printf '\n'
}

prompt_remove "$CONFIG_DIR"         "prompt_config"
prompt_remove "$CACHE_DIR"          "prompt_cache"
prompt_remove "$PICTURES_DIR"       "prompt_pictures"
prompt_remove "$FASTFETCH_CACHE"    "prompt_ff_cache"

# --- Final summary ---
printf '%b\n' "${GREEN}========================================${NC}"
printf '%b\n' "${GREEN}$(t uninstall_complete)${NC}"
printf '%b\n' "${GREEN}========================================${NC}"
