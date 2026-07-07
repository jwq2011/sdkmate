#!/bin/bash
# 软件预配模块 - 一键安装软件到远程服务器
#
# 用法: ./modules/provision/deploy.sh [选项] target1 [target2 ...]
#   --dry-run       只显示将安装的包，不实际安装
#   --skip-update   跳过包管理器缓存更新
#   --only-pkg      只安装系统包
#   --only-bin      只同步自定义工具

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOTFILES_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"
SCRIPTS_DIR="$DOTFILES_DIR/scripts"
PACKAGES_CONF="$DOTFILES_DIR/config/packages.conf"
MANIFEST="$DOTFILES_DIR/config/bin-manifest.json"
LOCAL_BIN="$DOTFILES_DIR/bin"

source "$SCRIPTS_DIR/lib/platform.sh"
source "$SCRIPTS_DIR/lib/pkgmgr.sh"
source "$SCRIPTS_DIR/servers.sh"

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# 选项
DRY_RUN=false
SKIP_UPDATE=false
ONLY_PKG=false
ONLY_BIN=false

usage() {
    echo "用法: $0 [选项] target1 [target2 ...]"
    echo ""
    echo "选项:"
    echo "  --dry-run       只显示将安装的包，不实际安装"
    echo "  --skip-update   跳过包管理器缓存更新"
    echo "  --only-pkg      只安装系统包"
    echo "  --only-bin      只同步自定义工具"
    echo "  --help          显示帮助"
    echo ""
    echo "目标格式: user@host 或 servers.conf 中的别名"
    echo ""
    echo "示例:"
    echo "  $0 user@host                  # 完整预配"
    echo "  $0 --dry-run user@host        # 只预览"
    echo "  $0 --only-pkg user@host       # 只装系统包"
    echo "  $0 --only-bin user@host       # 只同步工具"
    exit 1
}

# 解析 packages.conf 中的条目
# 用法: parse_packages_conf <配置文件> [过滤类型: pkg|bin]
# 输出: 类型|包名|描述|apt覆盖|yum覆盖|apk覆盖|分组
parse_packages_conf() {
    local config_file="$1"
    local filter_type="${2:-}"
    local current_group=""

    while IFS= read -r line || [ -n "$line" ]; do
        # 跳过空行和注释
        [[ -z "$line" || "$line" =~ ^[[:space:]]*# ]] && continue

        # 分组头
        [[ "$line" =~ ^\[.*\]$ ]] && {
            current_group="${line//[\[\]]/}"
            continue
        }

        # 解析: 类型|包名|描述[:apt名|yum名|apk名]
        IFS='|' read -r type name desc_rest <<< "$line"
        type="$(echo "$type" | xargs)"
        name="$(echo "$name" | xargs)"

        # 过滤类型
        [ -n "$filter_type" ] && [ "$type" != "$filter_type" ] && continue

        # 解析描述和包名覆盖
        local desc="$desc_rest"
        local apt_name="" yum_name="" apk_name=""

        if [[ "$desc_rest" == *:* ]]; then
            IFS=':' read -r desc apt_name yum_name apk_name <<< "$desc_rest"
        fi

        # 去除空白
        desc="$(echo "$desc" | xargs)"
        apt_name="$(echo "$apt_name" | xargs)"
        yum_name="$(echo "$yum_name" | xargs)"
        apk_name="$(echo "$apk_name" | xargs)"

        echo "${type}|${name}|${desc}|${apt_name}|${yum_name}|${apk_name}|${current_group}"
    done < "$config_file"
}

# 安装系统包
install_system_packages() {
    local target="$1"
    local full="$2"
    local pkg_mgr="$3"

    echo -e "  ${BLUE}检测到包管理器: ${pkg_mgr}${NC}"

    if [ "$pkg_mgr" = "unknown" ]; then
        echo -e "  ${YELLOW}跳过: 无法识别包管理器${NC}"
        return 0
    fi

    local packages=()
    local descs=()

    while IFS='|' read -r type name desc apt_name yum_name apk_name group; do
        local resolved
        resolved=$(get_package_name "$name" "$apt_name" "$yum_name" "$apk_name" "$pkg_mgr")

        if $DRY_RUN; then
            echo -e "  ${CYAN}[dry-run]${NC} $name ($desc) -> $resolved"
        else
            packages+=("$resolved")
            descs+=("$desc")
        fi
    done < <(parse_packages_conf "$PACKAGES_CONF" "pkg")

    if [ ${#packages[@]} -eq 0 ]; then
        echo -e "  ${YELLOW}无系统包需要安装${NC}"
        return 0
    fi

    echo -e "  ${BLUE}安装 ${#packages[@]} 个系统包...${NC}"

    # 先更新缓存
    if ! $SKIP_UPDATE; then
        echo -ne "  更新包缓存... "
        case "$pkg_mgr" in
            apt) ssh -o BatchMode=yes "$full" "sudo apt-get update -qq" &>/dev/null ;;
            yum) ssh -o BatchMode=yes "$full" "sudo yum makecache -q" &>/dev/null ;;
            apk) ssh -o BatchMode=yes "$full" "sudo apk update -q" &>/dev/null ;;
        esac
        echo -e "${GREEN}完成${NC}"
    fi

    # 批量安装
    echo -ne "  安装中... "
    if install_packages_batch "$full" "$pkg_mgr" "${packages[@]}" &>/dev/null; then
        echo -e "${GREEN}完成${NC}"
    else
        echo -e "${RED}部分失败${NC}"
        return 1
    fi
}

# 同步自定义工具
sync_bin_tools() {
    local target="$1"
    local full="$2"

    echo -e "  ${BLUE}同步自定义工具...${NC}"

    local synced=0
    local skipped=0

    while IFS='|' read -r type name desc apt_name yum_name apk_name group; do
        # 检查 bin-manifest.json
        if ! grep -q "\"$name\"" "$MANIFEST" 2>/dev/null; then
            echo -e "    ${YELLOW}跳过 $name: 不在 bin-manifest.json 中${NC}"
            ((skipped++))
            continue
        fi

        # 检查本地文件
        if [ ! -f "$LOCAL_BIN/$name" ]; then
            echo -e "    ${YELLOW}跳过 $name: 本地文件不存在${NC}"
            ((skipped++))
            continue
        fi

        if $DRY_RUN; then
            echo -e "    ${CYAN}[dry-run]${NC} $name ($desc)"
        else
            ssh -o BatchMode=yes "$full" "mkdir -p ~/bin" 2>/dev/null
            cat "$LOCAL_BIN/$name" | ssh -o BatchMode=yes "$full" "cat > ~/bin/$name && chmod +x ~/bin/$name" 2>/dev/null
            echo -e "    ${GREEN}✓${NC} $name ($desc)"
            ((synced++))
        fi
    done < <(parse_packages_conf "$PACKAGES_CONF" "bin")

    echo -e "  ${GREEN}同步完成: ${synced} 个工具, ${skipped} 个跳过${NC}"
}

# 预配单台服务器
provision_server() {
    local target="$1"
    local host="${target%%:*}"
    local label="${target#*:}"
    [ "$label" = "$host" ] && label=""

    echo -e "${GREEN}========== ${target} ==========${NC}"

    # 检测包管理器（跳过 SSH 连接测试，直接获取）
    local pkg_mgr
    pkg_mgr=$(detect_pkg_manager "$host")

    if [ "$pkg_mgr" = "unknown" ] && ! $ONLY_BIN; then
        echo -e "  ${RED}无法连接或识别包管理器${NC}"
        return 1
    fi

    # 安装系统包
    if ! $ONLY_BIN; then
        install_system_packages "$target" "$host" "$pkg_mgr"
    fi

    # 同步自定义工具
    if ! $ONLY_PKG; then
        sync_bin_tools "$target" "$host"
    fi

    echo ""
}

# 主流程
TARGETS=()

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)      DRY_RUN=true; shift ;;
        --skip-update)  SKIP_UPDATE=true; shift ;;
        --only-pkg)     ONLY_PKG=true; shift ;;
        --only-bin)     ONLY_BIN=true; shift ;;
        --help)         usage ;;
        *)              TARGETS+=("$1"); shift ;;
    esac
done

if [ ${#TARGETS[@]} -eq 0 ]; then
    echo -e "${RED}错误: 请指定目标服务器${NC}"
    echo "用法: $0 [选项] user@host [user@host2 ...]"
    exit 1
fi

if [ ! -f "$PACKAGES_CONF" ]; then
    echo -e "${RED}错误: 软件清单不存在: $PACKAGES_CONF${NC}"
    exit 1
fi

echo -e "${GREEN}软件预配${NC}"
$DRY_RUN && echo -e "${YELLOW}[dry-run 模式]${NC}"
echo ""

for target in "${TARGETS[@]}"; do
    provision_server "$target"
done

echo -e "${GREEN}预配完成!${NC}"
