# fwaifu

基于 fastfetch 的终端看板娘 / Terminal anime girl viewer powered by fastfetch

> ⚠️ 个人向工具。本项目含有 LLM 生成内容。
> Personal-use tool. This project contains LLM-generated content.

## 功能 / Features

- 从 [Nekos.moe](https://nekos.moe) 随机获取 SFW/NSFW 二次元图片，配合 fastfetch 展示系统信息
- 持续轮播模式，适合挂在副屏当作动态看板娘
- 本地缓存 + 后台守护进程自动补货
- 图片裁剪（需 ImageMagick）
- HTTP(S) 代理支持
- 多语言（中文 / English，根据 `LANG` 环境变量自动切换）
- 登录 Nekos.moe 后解锁 NSFW 内容
- `-s / --save` 保存上一次展示的图片
- `--clean` 清除缓存
- `--update` 检查并提示更新
- `--completion` 生成 Shell 补全脚本 (bash/zsh/fish)

## 依赖 / Requirements

- [fastfetch](https://github.com/fastfetch-cli/fastfetch)（必需）
- [ImageMagick](https://imagemagick.org)（可选，用于裁剪）

## 安装 / Installation

```bash
# 一键安装
curl -fsSL https://raw.githubusercontent.com/cublueer/fwaifu/main/install.sh | bash

# rust编译安装
cargo install fwaifu
```

安装后即可使用，无需额外配置 PATH（二进制位于 `/usr/bin/fwaifu`）。Shell 补全（bash/zsh/fish）会在安装时自动配置。

## 卸载 / Uninstallation

```bash
# 一键卸载
curl -fsSL https://raw.githubusercontent.com/cublueer/fwaifu/main/uninstall.sh | bash

# 或手动执行
sudo rm /usr/bin/fwaifu
```

### Shell 补全

安装脚本会自动配置补全。以下为手动安装方式：

```bash
# bash
fwaifu --completion bash > ~/.local/share/bash-completion/completions/fwaifu

# zsh
mkdir -p ~/.zsh/completion
fwaifu --completion zsh > ~/.zsh/completion/_fwaifu
# 在 ~/.zshrc 中添加: fpath=(~/.zsh/completion $fpath); autoload -Uz compinit; compinit

# fish
fwaifu --completion fish > ~/.config/fish/completions/fwaifu.fish
```

## 使用 / Usage

### 基本用法

```bash
fwaifu                    # 显示随机 SFW 图片 + 系统信息
fwaifu -n                 # NSFW 模式（需先 --login）
fwaifu -w                 # 持续轮播模式
fwaifu -w -n              # NSFW 轮播
```

### 账户

```bash
fwaifu --login            # 登录 Nekos.moe
fwaifu --logout           # 登出
fwaifu --status           # 查看登录状态
```

### 保存

```bash
fwaifu -s                 # 保存上一张图片到默认目录
fwaifu -s /path/to/dir    # 保存到指定目录
fwaifu --save /tmp/waifu  # 同 -s
```

默认保存路径为 `~/Pictures/fwaifu/`，可通过配置文件分别指定 SFW 和 NSFW 的保存目录。

### 清理缓存

```bash
fwaifu --clean sfw        # 清除 SFW 缓存
fwaifu --clean nsfw       # 清除 NSFW 缓存
```

### 更新

```bash
fwaifu --update           # 检查并提示更新
```

### 全部选项

**模式选项:**
  -n, --nsfw               启用 NSFW 模式
  -w, --watch              启用轮播模式
      --watch-interval <秒> 轮播间隔（默认 5）

**网络选项:**
  -p, --proxy <URL>        代理地址

**显示选项:**
      --no-crop            禁用裁剪
      --crop-width <宽>    裁剪宽度（默认 600）
      --crop-height <高>   裁剪高度（默认 800）
      --logo-width <宽>    fastfetch 图片宽度（默认 40）

**账户选项:**
      --login              登录 Nekos.moe
      --logout             登出
      --status             查看登录状态

**缓存与保存:**
      --clean [sfw|nsfw]   清除缓存
  -s, --save [PATH]        保存上一张图片

**其他:**
  -h, --help               帮助
      --update             检查更新
      --version            版本
      --completion <SHELL>  生成 Shell 补全

所有选项之后可直接附加 fastfetch 原生参数，如：
fwaifu --logo-width 30 --pipe false

## 配置 / Configuration

配置文件：`~/.config/fwaifu/config.toml`

完整示例见项目根目录的 [config.example.toml](config.example.toml)。

```toml
# 代理
# proxy = "http://127.0.0.1:7890"

# 裁剪
# crop = true
# crop_width = 600
# crop_height = 800

# 轮播间隔
# watch_interval = 5

# fastfetch 图片宽度
# logo_width = 40

[download]
# batch_size = 10              # 每次补货下载数量

[cache]
# max_limit = 100              # 最大库存
# min_trigger = 60             # 触发补货阈值
# max_used = 50                # 已用图片保留上限
# clean_cache = true           # 是否清理 fastfetch 缩略图缓存

# SFW / NSFW 分别指定保存路径（不设则默认 ~/Pictures/fwaifu）
# save_path_sfw = "~/Pictures/fwaifu"
# save_path_nsfw = "~/Pictures/fwaifu"
```

优先级：CLI 参数 > 环境变量 `FWAIFU_PROXY` > 配置文件

## 图片源 / Image Source

[Nekos.moe](https://nekos.moe)

## 许可 / License

MIT
