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

tf() {
    local key="$1"; shift
    eval "printf \"\$MSG_${key}_${CUR_LANG}\\n\" \"\$@\""
}

# --- Colors ---
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# --- Variables ---
BIN_NAME="fwaifu"
INSTALL_DIR="${HOME}/.local/bin"
BIN_PATH="${INSTALL_DIR}/${BIN_NAME}"
SRC_DIR="$(cd "$(dirname "$0")" && pwd)"

# --- Parse arguments ---
SKIP_INSTALLED_CHECK=0
for arg in "$@"; do
    case "$arg" in
        --reinstall) SKIP_INSTALLED_CHECK=1 ;;
    esac
done

# --- Messages ---
MSG_err_no_cargo_zh="${RED}错误: 未找到 Rust 工具链（缺少 cargo 命令）${NC}"
MSG_err_no_cargo_en="${RED}Error: Rust toolchain not found (cargo command missing)${NC}"

MSG_hint_install_rust_zh="安装 Rust: ${YELLOW}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
MSG_hint_install_rust_en="Install Rust: ${YELLOW}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"

MSG_err_no_toolchain_zh="${RED}错误: 找到 cargo 但未配置 Rust 工具链${NC}"
MSG_err_no_toolchain_en="${RED}Error: cargo found but no Rust toolchain is configured${NC}"

MSG_hint_rustup_default_zh="  运行: ${YELLOW}rustup default stable${NC}"
MSG_hint_rustup_default_en="  Run: ${YELLOW}rustup default stable${NC}"

MSG_warn_no_fastfetch_zh="${YELLOW}警告: fastfetch 未安装（必需依赖）${NC}"
MSG_warn_no_fastfetch_en="${YELLOW}Warning: fastfetch is not installed (required dependency)${NC}"

MSG_hint_install_fastfetch_zh="  安装: https://github.com/fastfetch-cli/fastfetch"
MSG_hint_install_fastfetch_en="  Install: https://github.com/fastfetch-cli/fastfetch"

MSG_imagemagick_found_zh="${GREEN}检测到 ImageMagick（必需: 支持图片裁剪）${NC}"
MSG_imagemagick_found_en="${GREEN}ImageMagick detected (required: image cropping support)${NC}"

MSG_err_no_imagemagick_zh="${RED}错误: ImageMagick 未安装（必需依赖）。请安装后再试。${NC}"
MSG_err_no_imagemagick_en="${RED}Error: ImageMagick is not installed (required dependency). Please install it first.${NC}"

MSG_hint_install_imagemagick_zh="  安装: apt install imagemagick  # Debian/Ubuntu"
MSG_hint_install_imagemagick_en="  Install: apt install imagemagick  # Debian/Ubuntu"

MSG_existing_install_zh="${YELLOW}发现已有安装: ${BIN_PATH}${NC}"
MSG_existing_install_en="${YELLOW}Found existing installation at ${BIN_PATH}${NC}"

MSG_prompt_reinstall_zh="重新安装? [y/N] "
MSG_prompt_reinstall_en="Reinstall? [y/N] "

MSG_reinstalling_zh="${GREEN}继续重新安装...${NC}"
MSG_reinstalling_en="${GREEN}Proceeding with reinstall...${NC}"

MSG_skip_install_zh="${GREEN}跳过安装。${NC}"
MSG_skip_install_en="${GREEN}Skipping installation.${NC}"

MSG_err_no_cargo_toml_zh="${RED}错误: 在 ${SRC_DIR} 未找到 Cargo.toml，这是项目根目录吗？${NC}"
MSG_err_no_cargo_toml_en="${RED}Error: Cargo.toml not found in ${SRC_DIR}. Is this the project root?${NC}"

MSG_build_start_zh="${GREEN}正在从 ${SRC_DIR} 编译 ${BIN_NAME}（可能需要一些时间）...${NC}"
MSG_build_start_en="${GREEN}Building ${BIN_NAME} from local source at ${SRC_DIR} (this may take a while)...${NC}"

MSG_err_build_failed_zh="${RED}错误: 编译失败。${NC}"
MSG_err_build_failed_en="${RED}Error: Build failed.${NC}"

MSG_err_no_git_zh="${RED}错误: 未找到 git，无法克隆仓库。请手动安装：${NC}"
MSG_err_no_git_en="${RED}Error: git not found, cannot clone repository. Install manually:${NC}"

MSG_hint_manual_install_zh="  git clone --depth 1 https://github.com/cublueer/fwaifu.git && cd fwaifu && bash install.sh"
MSG_hint_manual_install_en="  git clone --depth 1 https://github.com/cublueer/fwaifu.git && cd fwaifu && bash install.sh"

MSG_cloning_repo_zh="${GREEN}正在克隆仓库...${NC}"
MSG_cloning_repo_en="${GREEN}Cloning repository...${NC}"

MSG_clone_failed_zh="${RED}错误: 克隆仓库失败。请检查网络并重试，或手动安装：${NC}"
MSG_clone_failed_en="${RED}Error: Failed to clone repository. Check your network and try again, or install manually:${NC}"

MSG_cleaning_temp_zh="${GREEN}正在清理临时文件...${NC}"
MSG_cleaning_temp_en="${GREEN}Cleaning up temporary files...${NC}"

MSG_installing_zh="${GREEN}正在安装 ${BIN_NAME} 到 ${INSTALL_DIR}/...${NC}"
MSG_installing_en="${GREEN}Installing ${BIN_NAME} to ${INSTALL_DIR}/...${NC}"

MSG_err_install_failed_zh="${RED}错误: 安装到 ${INSTALL_DIR} 失败。权限不足或磁盘已满。${NC}"
MSG_err_install_failed_en="${RED}Error: Failed to install to ${INSTALL_DIR}. Permission denied or disk full.${NC}"

MSG_warn_path_zh="${YELLOW}注意: ${INSTALL_DIR} 不在 PATH 中。请将以下行添加到 ~/.bashrc 或 ~/.zshrc:${NC}"
MSG_warn_path_en="${YELLOW}Note: ${INSTALL_DIR} is not in PATH. Add this line to your ~/.bashrc or ~/.zshrc:${NC}"

MSG_hint_path_zh="  export PATH=\"${INSTALL_DIR}:\$PATH\""
MSG_hint_path_en="  export PATH=\"${INSTALL_DIR}:\$PATH\""

MSG_installing_completions_zh="${GREEN}正在安装 Shell 补全...${NC}"
MSG_installing_completions_en="${GREEN}Installing shell completions...${NC}"

MSG_shell_installed_zh="  %s: 已安装到 %s"
MSG_shell_installed_en="  %s: installed to %s"

MSG_shell_failed_zh="  %s: 失败"
MSG_shell_failed_en="  %s: failed"

MSG_install_complete_zh="${GREEN}安装完成！${NC}"
MSG_install_complete_en="${GREEN}Installation complete!${NC}"

MSG_usage_zh="${YELLOW}用法:${NC}"
MSG_usage_en="${YELLOW}Usage:${NC}"

MSG_usage_basic_zh="  ${BIN_NAME}              显示随机动漫图片 + 系统信息"
MSG_usage_basic_en="  ${BIN_NAME}              Show a random anime image + system info"

MSG_usage_help_zh="  ${BIN_NAME} --help       显示所有选项"
MSG_usage_help_en="  ${BIN_NAME} --help       Show all options"

MSG_usage_version_zh="  ${BIN_NAME} --version    显示版本"
MSG_usage_version_en="  ${BIN_NAME} --version    Show version"

MSG_dependencies_zh="${YELLOW}依赖项:${NC}"
MSG_dependencies_en="${YELLOW}Dependencies:${NC}"

MSG_ff_required_not_found_zh="  必需: fastfetch ${RED}(未找到)${NC}"
MSG_ff_required_not_found_en="  Required: fastfetch ${RED}(not found)${NC}"

MSG_ff_required_found_zh="  必需: fastfetch ${GREEN}(已找到)${NC}"
MSG_ff_required_found_en="  Required: fastfetch ${GREEN}(found)${NC}"

MSG_im_found_zh="  必需: ImageMagick ${GREEN}(已找到)${NC}"
MSG_im_found_en="  Required: ImageMagick ${GREEN}(found)${NC}"

MSG_config_copied_zh="${GREEN}示例配置已复制到 ~/.config/fwaifu/config.toml${NC}"
MSG_config_copied_en="${GREEN}Example config copied to ~/.config/fwaifu/config.toml${NC}"

MSG_config_exists_zh="${YELLOW}配置文件已存在，跳过复制: ~/.config/fwaifu/config.toml${NC}"
MSG_config_exists_en="${YELLOW}Config file already exists, skipping: ~/.config/fwaifu/config.toml${NC}"

MSG_config_path_zh="配置: ~/.config/fwaifu/config.toml"
MSG_config_path_en="Config: ~/.config/fwaifu/config.toml"

# --- 1. Check cargo + Rust toolchain ---
if ! command -v cargo >/dev/null 2>&1; then
    printf '%b\n' "$(t err_no_cargo)" >&2
    printf '%b\n' "$(t hint_install_rust)"
    exit 1
fi

if ! cargo --version >/dev/null 2>&1; then
    printf '%b\n' "$(t err_no_toolchain)" >&2
    printf '%b\n' "$(t hint_rustup_default)"
    exit 1
fi

# --- 2. Check system dependencies ---
missing_fastfetch=0
if ! command -v fastfetch >/dev/null 2>&1; then
    missing_fastfetch=1
    printf '%b\n' "$(t warn_no_fastfetch)"
    printf '%b\n' "$(t hint_install_fastfetch)"
fi

if ! command -v magick >/dev/null 2>&1 && ! command -v convert >/dev/null 2>&1; then
    printf '%b\n' "$(t err_no_imagemagick)" >&2
    printf '%b\n' "$(t hint_install_imagemagick)"
    exit 1
fi
printf '%b\n' "$(t imagemagick_found)"

# --- 3. Check if already installed ---
already_installed=0
if [ -f "$BIN_PATH" ]; then
    already_installed=1
    printf '%b\n' "$(t existing_install)"
fi

if [ "$already_installed" -eq 1 ]; then
    if [ "$SKIP_INSTALLED_CHECK" -eq 1 ]; then
        printf '%b\n' "$(t reinstalling)"
    else
        printf '%b' "${YELLOW}$(t prompt_reinstall)${NC}"
        read -r REPLY
        case "$REPLY" in
            [Yy]|[Yy][Ee][Ss])
                printf '%b\n' "$(t reinstalling)"
                ;;
            *)
                printf '%b\n' "$(t skip_install)"
                exit 0
                ;;
        esac
    fi
fi

# --- 4. Ensure source code is available (clone if curl-piped) ---
USE_TEMP_DIR=0

if [ ! -f "$SRC_DIR/Cargo.toml" ]; then
    if ! command -v git >/dev/null 2>&1; then
        printf '%b\n' "$(t err_no_git)" >&2
        printf '%b\n' "$(t hint_manual_install)"
        exit 1
    fi

    BUILD_DIR=$(mktemp -d /tmp/fwaifu_build.XXXXXX)
    printf '%b\n' "$(t cloning_repo)"

    git clone --depth 1 https://github.com/cublueer/fwaifu.git "$BUILD_DIR" || {
        printf '%b\n' "$(t err_clone_failed)" >&2
        printf '%b\n' "$(t hint_manual_install)"
        rm -rf "$BUILD_DIR"
        exit 1
    }

    SRC_DIR="$BUILD_DIR"
    USE_TEMP_DIR=1
fi

# --- 5. Build from source ---
printf '%b\n' "$(t build_start)"
(cd "$SRC_DIR" && cargo build --release) || {
    printf '%b\n' "$(t err_build_failed)" >&2
    [ "$USE_TEMP_DIR" -eq 1 ] && rm -rf "$BUILD_DIR"
    exit 1
}

# --- 6. Install binary to ~/.local/bin/ ---
printf '%b\n' "$(t installing)"
mkdir -p "$INSTALL_DIR"
cp "$SRC_DIR/target/release/$BIN_NAME" "$BIN_PATH" || {
    printf '%b\n' "$(t err_install_failed)" >&2
    [ "$USE_TEMP_DIR" -eq 1 ] && rm -rf "$BUILD_DIR"
    exit 1
}

# --- 7. Install shell completions ---
printf '%b\n' "$(t installing_completions)"

# bash completion
if command -v bash >/dev/null 2>&1; then
    BASH_COMPLETION_DIR="${HOME}/.local/share/bash-completion/completions"
    mkdir -p "$BASH_COMPLETION_DIR"
    "$BIN_PATH" --completion bash > "$BASH_COMPLETION_DIR/$BIN_NAME" 2>/dev/null && \
        printf '%b\n' "${GREEN}$(tf shell_installed "bash" "${BASH_COMPLETION_DIR}/${BIN_NAME}")${NC}" || \
        printf '%b\n' "${YELLOW}$(tf shell_failed "bash")${NC}"
fi

# zsh completion
if command -v zsh >/dev/null 2>&1; then
    ZSH_COMPLETION_DIR="${HOME}/.zsh/completion"
    mkdir -p "$ZSH_COMPLETION_DIR"
    "$BIN_PATH" --completion zsh > "$ZSH_COMPLETION_DIR/_$BIN_NAME" 2>/dev/null && \
        printf '%b\n' "${GREEN}$(tf shell_installed "zsh" "${ZSH_COMPLETION_DIR}/_${BIN_NAME}")${NC}" || \
        printf '%b\n' "${YELLOW}$(tf shell_failed "zsh")${NC}"
fi

# fish completion
if command -v fish >/dev/null 2>&1; then
    FISH_COMPLETION_DIR="${HOME}/.config/fish/completions"
    mkdir -p "$FISH_COMPLETION_DIR"
    "$BIN_PATH" --completion fish > "$FISH_COMPLETION_DIR/$BIN_NAME.fish" 2>/dev/null && \
        printf '%b\n' "${GREEN}$(tf shell_installed "fish" "${FISH_COMPLETION_DIR}/${BIN_NAME}.fish")${NC}" || \
        printf '%b\n' "${YELLOW}$(tf shell_failed "fish")${NC}"
fi

printf '\n'

# --- Copy example config ---
CONFIG_DIR="${HOME}/.config/fwaifu"
if [ -f "$CONFIG_DIR/config.toml" ]; then
    printf '%b\n' "$(t config_exists)"
else
    mkdir -p "$CONFIG_DIR"
    cp "$SRC_DIR/config.example.toml" "$CONFIG_DIR/config.toml" && \
        printf '%b\n' "$(t config_copied)"
fi

# --- Cleanup: remove temp directory if cloned from git ---
if [ "$USE_TEMP_DIR" -eq 1 ]; then
    printf '%b\n' "$(t cleaning_temp)"
    rm -rf "$BUILD_DIR"
fi

# --- 8. Print usage ---
printf '\n%b\n' "$(t install_complete)"
printf '\n%b\n' "$(t usage)"
printf '%b\n' "$(t usage_basic)"
printf '%b\n' "$(t usage_help)"
printf '%b\n' "$(t usage_version)"
printf '\n%b\n' "$(t dependencies)"

if [ "$missing_fastfetch" -eq 1 ]; then
    printf '%b\n' "$(t ff_required_not_found)"
else
    printf '%b\n' "$(t ff_required_found)"
fi

printf '%b\n' "$(t im_found)"

printf '\n%b\n' "$(t config_path)"

# --- Check PATH ---
case ":${PATH}:" in
    *:"${INSTALL_DIR}":*) ;;
    *)
        printf '\n%b\n' "$(t warn_path)"
        printf '%b\n' "$(t hint_path)"
        ;;
esac
