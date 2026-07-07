#!/bin/bash
# 使用方法: ./modules/prompt/deploy.sh "user@host1" "user@host2" ...
# 示例: ./modules/prompt/deploy.sh "root@192.168.1.10" "root@192.168.1.11"
#
# 注意：需要先配置 secrets.env 中的 SERVER_LABEL

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOTFILES_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"
CONFIG_FILE="$SCRIPT_DIR/my_prompt_rc"
SECRETS_FILE="$SCRIPT_DIR/secrets.env"
REMOTE_PROMPT=".my_prompt_rc"
REMOTE_SECRETS=".dotfiles_secrets"

# 加载 secrets
load_secrets() {
    if [ -f "$SECRETS_FILE" ]; then
        set -a
        source "$SECRETS_FILE"
        set +a
    fi
}

for target in "$@"; do
    echo ">>> 处理 $target"

    # 1. 推送配置文件
    scp "$CONFIG_FILE" "${target}:~/${REMOTE_PROMPT}" || {
        echo "  错误：无法上传配置"
        continue
    }
    echo "  ✓ 上传 my_prompt_rc"

    # 2. 推送 secrets（如果存在）
    if [ -f "$SECRETS_FILE" ]; then
        scp "$SECRETS_FILE" "${target}:~/${REMOTE_SECRETS}" || {
            echo "  警告：无法上传 secrets"
        }
        echo "  ✓ 上传 secrets"
    fi

    # 3. 在远程 .bashrc 追加引用（如果标记不存在）
    ssh "$target" bash <<ENDSSH
PROMPT_RC=".my_prompt_rc"
SECRETS=".dotfiles_secrets"
MARKER="# CUSTOM_PROMPT_RC_V1"

# 加载 secrets
if [ -f ~/\$SECRETS ]; then
    set -a
    source ~/\$SECRETS
    set +a
fi

# 追加到 .bashrc
if ! grep -qF "\$MARKER" ~/.bashrc 2>/dev/null; then
    echo "" >> ~/.bashrc
    echo "\$MARKER" >> ~/.bashrc
    echo "[ -f ~/\$PROMPT_RC ] && source ~/\$PROMPT_RC" >> ~/.bashrc
    echo "已追加到 .bashrc"
else
    echo ".bashrc 配置已存在"
fi
ENDSSH

    echo "  ✓ $target 完成"
done
echo ""
echo "完成！"