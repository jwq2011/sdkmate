#!/bin/bash
# dotfiles 快捷命令加载脚本
# source 此文件来加载所有快捷命令
# 使用说明: source ~/.dotfiles.sh

# 添加 bin 目录到 PATH
export PATH="$HOME/bin:$PATH"

# ==================== 定义快捷命令 ====================

# repo 相关
repo-sync() {
    JOBS=${1:-24}
    echo ">>> repo sync -j$JOBS"
    repo sync -c --no-tags --force-remove-dirty -j$JOBS --force-sync --quiet
    echo ">>> 完成"
}

repo-status() {
    repo status
}

# ==================== dotfiles 主命令 ====================

dotfiles() {
    if [ -z "$1" ]; then
        echo ""
        echo " dotfiles 快捷命令"
        echo ""
        echo " 用法: dotfiles <命令>"
        echo ""
        echo " 可用命令:"
        echo "   repo-sync [n]  - repo sync (默认 j24)"
        echo "   repo-status    - repo status"
        echo ""
        echo " 示例:"
        echo "   dotfiles repo-sync     # 使用默认 j24"
        echo "   dotfiles repo-sync 16  # 使用 j16"
        echo ""
        return
    fi

    case "$1" in
        repo-sync)
            shift
            repo-sync "$@"
            ;;
        repo-status|rs)
            repo status
            ;;
        *)
            echo "未知命令: $1"
            echo "运行 'dotfiles' 查看可用命令"
            return 1
            ;;
    esac
}