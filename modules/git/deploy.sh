#!/bin/bash
# Git 配置模块部署脚本
#
# 使用 secrets.env 管理敏感配置
# 部署时自动替换 {{VARIABLE}} 占位符

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOTFILES_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"
MODULE_DIR="$SCRIPT_DIR"

# 引入 secrets 管理库
source "$DOTFILES_DIR/scripts/secrets.sh"

CONFIG_FILE="$MODULE_DIR/gitconfig"
TEMPLATE_FILE="$MODULE_DIR/git-commit.template"
IGNORE_FILE="$MODULE_DIR/gitignore"
TEMP_DIR="/tmp/dotfiles_git_$$"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

usage() {
    echo "用法: $0 user@host1 [user@host2 ...]"
    echo "示例: $0 \"user@host1:标签\""
    echo ""
    echo "必填变量 (在 secrets.env 中配置):"
    echo "  - GIT_EMAIL  - 你的邮箱"
    echo "  - GIT_NAME   - 你的名字"
    echo ""
    echo "可选变量:"
    echo "  - AREA       - 区域代码（默认 wh）"
    echo ""
    echo "首次使用请先创建 secrets.env:"
    echo "  cp secrets.env.example secrets.env"
    echo "  vim secrets.env"
    exit 1
}

if [ $# -eq 0 ]; then
    usage
fi

# 解析 host:label 格式
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

    # 查找 secrets.env
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
deploy_git_config() {
    local target="$1"
    local temp_config="$TEMP_DIR/gitconfig"

    # 创建临时目录
    mkdir -p "$TEMP_DIR"

    # 替换变量
    if replace_variables "$CONFIG_FILE" "$temp_config"; then
        echo -e "${GREEN}✓ 变量替换成功${NC}"
    else
        echo -e "${RED}✗ 变量替换失败，跳过${NC}"
        return 1
    fi

    # 1. 部署 gitconfig
    echo -e "${GREEN}部署 .gitconfig${NC}"
    scp "$temp_config" "${target}:~/.gitconfig" || {
        echo -e "${RED}错误：无法上传 .gitconfig 到 $target${NC}"
        return 1
    }

    # 2. 部署 commit template（不涉及变量替换）
    echo -e "${GREEN}部署 .git-commit.template${NC}"
    scp "$TEMPLATE_FILE" "${target}:~/.git-commit.template" || {
        echo -e "${YELLOW}警告：无法上传 .git-commit.template${NC}"
    }

    # 3. 部署全局忽略文件
    echo -e "${GREEN}部署 .gitignore_global${NC}"
    scp "$IGNORE_FILE" "${target}:~/.gitignore_global" || {
        echo -e "${YELLOW}警告：无法上传 .gitignore_global${NC}"
    }

    # 4. 配置 git 使用全局忽略文件
    echo -e "${GREEN}配置全局忽略文件${NC}"
    ssh "$target" "git config --global core.excludesfile ~/.gitignore_global" 2>/dev/null || true

    return 0
}

# 清理临时文件
cleanup() {
    rm -rf "$TEMP_DIR" 2>/dev/null || true
}
trap cleanup EXIT

# 主流程
echo -e "${GREEN}========== Git 配置部署 ==========${NC}"
echo ""

# 加载 secrets
load_module_secrets

# 处理每个目标
for target_label in "$@"; do
    target=$(parse_target "$target_label")
    echo -e "${GREEN}>>> 处理 $target${NC}"
    echo ""

    if deploy_git_config "$target"; then
        echo -e "${GREEN}✓ $target Git 配置完成${NC}"
    else
        echo -e "${RED}✗ $target Git 配置失败${NC}"
    fi
    echo ""
done

echo -e "${GREEN}全部完成！${NC}"