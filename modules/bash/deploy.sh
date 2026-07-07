#!/bin/bash
# Bash 配置模块部署脚本
#
# 使用 secrets.env 管理敏感配置
# 部署时自动替换 {{VARIABLE}} 占位符

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOTFILES_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"
MODULE_DIR="$SCRIPT_DIR"

# 引入 secrets 管理库
source "$DOTFILES_DIR/scripts/secrets.sh"

CONFIG_FILE="$MODULE_DIR/bashrc"
CONFIG_LOCAL="$MODULE_DIR/bashrc.local"
INPUTRC="$MODULE_DIR/inputrc"
DOTFILES_SH="$MODULE_DIR/dotfiles.sh"
DOTFILES_COMP="$MODULE_DIR/dotfiles_completion.sh"
BIN_DIR="$MODULE_DIR/bin"
TEMP_DIR="/tmp/dotfiles_bash_$$"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

usage() {
    echo "用法: $0 user@host1 [user@host2 ...]"
    echo "示例: $0 \"user@host1:标签\""
    echo ""
    echo "可选变量 (在 secrets.env 中配置):"
    echo "  - PROJECT_PATH - 项目根路径"
    exit 1
}

if [ $# -eq 0 ]; then
    usage
fi

parse_target() {
    local target_label="$1"
    if [[ "$target_label" == *:* ]]; then
        target="${target_label%%:*}"
    else
        target="$target_label"
    fi
    echo "$target"
}

# 加载 secrets
load_module_secrets() {
    local secrets_file

    secrets_file="$(find_secrets "$SCRIPT_DIR")"

    if [ -n "$secrets_file" ] && [ -f "$secrets_file" ]; then
        echo -e "${GREEN}加载配置: $secrets_file${NC}"
        load_secrets "$secrets_file"
        return 0
    else
        echo -e "${YELLOW}警告: 未找到 secrets.env，使用环境变量或默认值${NC}"
        return 1
    fi
}

# 部署配置到远程
deploy_bash_config() {
    local target="$1"
    local temp_config="$TEMP_DIR/bashrc.dotfiles"

    # 创建临时目录
    mkdir -p "$TEMP_DIR"

    # 替换变量
    if replace_variables "$CONFIG_FILE" "$temp_config"; then
        echo -e "${GREEN}✓ 变量替换成功${NC}"
    else
        echo -e "${RED}✗ 变量替换失败，跳过${NC}"
        return 1
    fi

    # 1. 部署 bashrc（追加到远程 .bashrc）
    echo -e "${GREEN}部署 .bashrc 配置${NC}"
    ssh "$target" bash <<'ENDSSH'
        # 创建备份
        if [ -f ~/.bashrc ]; then
            cp ~/.bashrc ~/.bashrc.backup.$(date +%Y%m%d_%H%M%S)
        fi

        # 追加配置（如果标记不存在）
        MARKER="# DOTFILES_BASHRC"
        if ! grep -qF "$MARKER" ~/.bashrc; then
            echo "" >> ~/.bashrc
            echo "$MARKER" >> ~/.bashrc
            echo "source ~/.bashrc.dotfiles" >> ~/.bashrc
            echo "已追加到 .bashrc"
        else
            echo ".bashrc 配置已存在"
        fi
ENDSSH

    # 2. 上传 bashrc 配置（已替换变量）
    scp "$temp_config" "${target}:~/.bashrc.dotfiles" || {
        echo -e "${RED}错误：无法上传 bashrc 到 $target${NC}"
        return 1
    }

    # 3. 部署 inputrc
    echo -e "${GREEN}部署 .inputrc${NC}"
    scp "$INPUTRC" "${target}:~/.inputrc" || {
        echo -e "${YELLOW}警告：无法上传 inputrc${NC}"
    }

    # 4. 部署 dotfiles 快捷命令脚本
    echo -e "${GREEN}部署 dotfiles 快捷命令${NC}"
    scp "$DOTFILES_SH" "${target}:~/.dotfiles.sh" || {
        echo -e "${YELLOW}警告：无法上传 dotfiles.sh${NC}"
    }
    scp "$DOTFILES_COMP" "${target}:~/.dotfiles_completion.sh" || {
        echo -e "${YELLOW}警告：无法上传 dotfiles_completion.sh${NC}"
    }

    # 5. 部署 bin/ 目录
    echo -e "${GREEN}部署 ~/bin/ 快捷脚本${NC}"
    ssh "$target" "mkdir -p ~/bin"
    scp -r "$BIN_DIR"/* "${target}:~/bin/" || {
        echo -e "${YELLOW}警告：无法上传 bin/ 脚本${NC}"
    }
    ssh "$target" "chmod + ~/bin/*" 2>/dev/null || true

    return 0
}

# 清理临时文件
cleanup() {
    rm -rf "$TEMP_DIR" 2>/dev/null || true
}
trap cleanup EXIT

# 主流程
echo -e "${GREEN}========== Bash 配置部署 ==========${NC}"
echo ""

# 加载 secrets
load_module_secrets

# 处理每个目标
for target_label in "$@"; do
    target=$(parse_target "$target_label")
    echo -e "${GREEN}>>> 处理 $target${NC}"
    echo ""

    if deploy_bash_config "$target"; then
        echo -e "${GREEN}✓ $target Bash 配置完成${NC}"
    else
        echo -e "${RED}✗ $target Bash 配置失败${NC}"
    fi
    echo ""
done

echo -e "${GREEN}全部完成！${NC}"