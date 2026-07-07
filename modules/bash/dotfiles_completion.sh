#!/bin/bash
# dotfiles 命令补全
# source 此文件来启用 tab 补全

_dotfiles_completion() {
    local cur prev
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    # dotfiles 子命令
    commands="repo-sync repo-status"

    case "${prev}" in
        dotfiles)
            COMPREPLY=($(compgen -W "${commands}" -- ${cur}))
            return 0
            ;;
        repo-sync)
            # repo-sync 接受数字参数
            COMPREPLY=($(compgen -W "8 16 24 32" -- ${cur}))
            return 0
            ;;
    esac
}

# 注册补全（仅 bash）
if [ -n "$BASH_VERSION" ]; then
    complete -F _dotfiles_completion dotfiles
fi