# 服务器预配

本文档详述 `sdkm provision` 命令的使用方法，用于在远程服务器上安装软件包和同步工具。服务器管理见 [server-management.md](./server-management.md)，部署功能见 [deployment.md](./deployment.md)。

命令总览：

| 命令 | 别名 | 作用 |
|:---|:---|:---|
| [`sdkm provision`](#sdkm-provision) | — | 预配服务器（安装软件包 + 同步工具） |

---

## sdkm provision

预配远程服务器，包括安装系统软件包和同步自定义工具。

```bash
sdkm provision <targets...> [OPTIONS]
```

- `<targets...>`：目标服务器，可以是别名或 `user@host` 格式，支持多个。

选项：

| 选项 | 短选项 | 说明 | 默认值 |
|:---|:---|:---|:---|
| `--dry-run` | `-d` | 预览模式，不实际执行 | false |
| `--only-pkg` | — | 只安装系统软件包 | false |
| `--only-bin` | — | 只同步自定义工具 | false |
| `--skip-update` | — | 跳过包管理器缓存更新 | false |
| `--packages-conf` | — | 自定义 packages.conf 路径 | 自动检测 |

示例：

```bash
# 完整预配
sdkm provision wsl2

# 预览模式
sdkm provision wsl2 --dry-run

# 只安装系统软件包
sdkm provision wsl2 --only-pkg

# 只同步自定义工具
sdkm provision wsl2 --only-bin

# 使用自定义配置文件
sdkm provision wsl2 --packages-conf ./my-packages.conf

# 批量预配
sdkm provision wsl2 dev-remote prod-web-01
```

---

## 软件清单

软件清单定义了要安装的软件包和工具，使用 pipe 分隔格式。

### 配置文件位置

默认按以下顺序查找 `packages.conf`：

1. `~/.sdkmate/packages.conf`
2. `~/dotfiles/config/packages.conf`
3. `./config/packages.conf`
4. `./packages.conf`

### 配置文件格式

```ini
# 软件清单
# 格式: 类型|包名|描述[:apt名|yum名|apk名]

[essential]
pkg|vim|文本编辑器
pkg|git|版本控制
pkg|curl|HTTP 客户端和数据传输工具
pkg|wget|文件下载工具
pkg|openssh-client|SSH 客户端:openssh-clients|
pkg|ca-certificates|CA 根证书

[development]
pkg|build-essential|C/C++ 编译工具链:gcc,make,cmake|@development tools|build-base
pkg|gdb|GNU 调试器
pkg|strace|系统调用跟踪工具
pkg|dos2unix|换行符格式转换（Windows/Linux 互转）
pkg|patch|补丁应用工具

[tools]
pkg|tmux|终端复用器（分屏、会话保持）
pkg|tig|Git 文本模式界面（浏览提交历史、差异）
pkg|htop|交互式进程监控器
pkg|tree|目录树结构显示
pkg|jq|JSON 命令行处理器
pkg|unzip|ZIP 解压缩工具
pkg|less|分页器（大文件浏览）
pkg|file|文件类型检测工具
pkg|lsof|查看打开文件和网络连接
pkg|net-tools|网络工具集（ifconfig、netstat 等）
pkg|rsync|文件增量同步工具
pkg|ncdu|磁盘使用分析器（交互式）

[custom]
bin|tig|Git 浏览器（静态编译二进制，无需依赖）
bin|bsp_qnxlog_analysis.sh|QNX BSP 日志分析工具（display/camera/system/ethernet）
```

### 配置项说明

| 字段 | 说明 | 示例 |
|:---|:---|:---|
| 类型 | `pkg`（系统包）或 `bin`（自定义工具） | `pkg` |
| 包名 | 软件包名称 | `vim` |
| 描述 | 软件包描述 | `文本编辑器` |
| apt 名 | Debian/Ubuntu 包名（可选） | `openssh-clients` |
| yum 名 | CentOS/RHEL 包名（可选） | `@development tools` |
| apk 名 | Alpine 包名（可选） | `build-base` |

### 包名覆盖机制

不同 Linux 发行版的包名可能不同，支持跨发行版包名映射：

```ini
# 格式: 类型|包名|描述:apt名|yum名|apk名
pkg|openssh-client|SSH 客户端:openssh-clients|
```

- 如果指定的包名在当前发行版不适用，会自动使用对应的包名
- 留空表示使用默认包名
- 以 `@` 开头表示 yum 组名（如 `@development tools`）

---

## 包管理器支持

sdkm 自动检测当前系统的包管理器，并使用对应的命令安装软件包。

### 支持的包管理器

| 包管理器 | 发行版 | 安装命令 |
|:---|:---|:---|
| apt | Debian, Ubuntu | `apt-get install -y` |
| yum | CentOS, RHEL, Fedora | `yum install -y` |
| apk | Alpine | `apk add` |

### 自动检测逻辑

1. 读取 `/etc/os-release` 文件
2. 解析 `ID` 字段确定发行版
3. 选择对应的包管理器
4. 如果检测失败，尝试通过 `which` 命令查找

### 包管理器抽象

sdkm 使用 `PackageManager` trait 抽象包管理器操作：

```rust
#[async_trait]
pub trait PackageManager: Send + Sync {
    async fn install(&self, package: &str) -> Result<()>;
    async fn uninstall(&self, package: &str) -> Result<()>;
    async fn is_installed(&self, package: &str) -> Result<bool>;
    async fn update(&self) -> Result<()>;
    fn name(&self) -> &str;
}
```

---

## 自定义工具同步

除了系统软件包，sdkm 还支持同步自定义工具。

### 工具清单

工具清单定义在 `bin-manifest.json` 中：

```json
{
  "version": 1,
  "tools": {
    "tig": {
      "hash": "abc123...",
      "description": "Git 浏览器（静态编译二进制，无需依赖）",
      "type": "script",
      "remote_path": "~/bin/tig"
    }
  }
}
```

### 工具类型

| 类型 | 说明 | 示例 |
|:---|:---|:---|
| `script` | Shell 脚本 | `tig` |
| `binary` | 编译后的二进制文件 | `jq` |
| `config` | 配置文件 | `.vimrc` |

### 同步模式

支持三种同步模式：

1. **check**：检查本地和远程的差异
2. **pull**：从远程拉取工具到本地
3. **push**：推送本地工具到远程

### 同步状态

| 状态 | 符号 | 说明 |
|:---|:---|:---|
| 同步 | `✓` | 本地和远程版本一致 |
| 冲突 | `≠` | 本地和远程版本不同 |
| 仅远程 | `←` | 只在远程存在 |
| 仅本地 | `→` | 只在本地存在 |

---

## 预配流程

实际预配过程包括以下步骤：

1. **解析目标服务器**：从配置文件加载服务器信息
2. **加载软件清单**：读取 `packages.conf` 配置
3. **过滤软件包**：根据 `--only-pkg` 或 `--only-bin` 过滤
4. **检测包管理器**：确定当前系统的包管理器
5. **更新缓存**：执行 `apt update` 或 `yum makecache`（除非 `--skip-update`）
6. **安装软件包**：逐个安装系统软件包
7. **同步工具**：同步自定义工具到远程服务器
8. **验证结果**：检查安装和同步是否成功

---

## 预览模式

使用 `--dry-run` 选项可以预览预配操作，不实际执行：

```bash
sdkm provision wsl2 --dry-run
```

输出示例：

```
Provisioning 1 server(s) with 15 package(s)...

[DRY RUN] Would install:
  - vim: 文本编辑器 (system package)
  - git: 版本控制 (system package)
  - curl: HTTP 客户端 (system package)
  - tmux: 终端复用器 (system package)
  - htop: 进程监控器 (system package)
  - tig: Git 浏览器 (custom tool)

To servers:
  - wsl2 (root@192.168.1.100:22)
```

预览模式下会显示：
- 目标服务器列表
- 要安装的软件包列表
- 软件包类型（系统包或自定义工具）

---

## 使用场景

### 场景 1：初始化新服务器

```bash
# 添加服务器
sdkm server add new-server root@10.0.0.1 --tags "dev"

# 预配服务器
sdkm provision new-server
```

### 场景 2：批量预配

```bash
# 预配所有开发服务器
sdkm provision --filter dev

# 预配所有生产服务器
sdkm provision --filter prod
```

### 场景 3：选择性预配

```bash
# 只安装系统软件包
sdkm provision wsl2 --only-pkg

# 只同步自定义工具
sdkm provision wsl2 --only-bin
```

### 场景 4：跳过缓存更新

```bash
# 网络较慢时跳过缓存更新
sdkm provision wsl2 --skip-update
```

---

## 配置文件示例

### 最小配置

```ini
[essential]
pkg|vim|文本编辑器
pkg|git|版本控制
```

### 完整配置

```ini
# 软件清单
# 格式: 类型|包名|描述[:apt名|yum名|apk名]

[essential]
pkg|vim|文本编辑器
pkg|git|版本控制
pkg|curl|HTTP 客户端和数据传输工具
pkg|wget|文件下载工具
pkg|openssh-client|SSH 客户端:openssh-clients|
pkg|ca-certificates|CA 根证书

[development]
pkg|build-essential|C/C++ 编译工具链:gcc,make,cmake|@development tools|build-base
pkg|gdb|GNU 调试器
pkg|strace|系统调用跟踪工具
pkg|dos2unix|换行符格式转换（Windows/Linux 互转）
pkg|patch|补丁应用工具

[tools]
pkg|tmux|终端复用器（分屏、会话保持）
pkg|tig|Git 文本模式界面（浏览提交历史、差异）
pkg|htop|交互式进程监控器
pkg|tree|目录树结构显示
pkg|jq|JSON 命令行处理器
pkg|unzip|ZIP 解压缩工具
pkg|less|分页器（大文件浏览）
pkg|file|文件类型检测工具
pkg|lsof|查看打开文件和网络连接
pkg|net-tools|网络工具集（ifconfig、netstat 等）
pkg|rsync|文件增量同步工具
pkg|ncdu|磁盘使用分析器（交互式）

[custom]
bin|tig|Git 浏览器（静态编译二进制，无需依赖）
bin|bsp_qnxlog_analysis.sh|QNX BSP 日志分析工具（display/camera/system/ethernet）
```

---

## 故障排查

### 问题：找不到 packages.conf

原因：配置文件不在默认路径。

解决：

```bash
# 使用 --packages-conf 指定路径
sdkm provision wsl2 --packages-conf /path/to/packages.conf

# 或创建默认配置文件
mkdir -p ~/.sdkmate
cp packages.conf ~/.sdkmate/
```

### 问题：包管理器检测失败

原因：不支持的 Linux 发行版。

解决：

```bash
# 检查发行版信息
cat /etc/os-release

# 手动指定包管理器（暂不支持，需修改配置）
```

### 问题：软件包安装失败

原因：包名错误、网络问题或权限不足。

解决：

```bash
# 检查包名是否正确
apt search vim  # Debian/Ubuntu
yum search vim  # CentOS/RHEL

# 检查网络连接
ping -c 3 archive.ubuntu.com

# 检查权限
sudo apt-get install vim
```

### 问题：自定义工具同步失败

原因：工具清单文件损坏或工具文件不存在。

解决：

```bash
# 检查工具清单
cat bin-manifest.json

# 检查工具文件
ls -la bin/
```

---

## 最佳实践

1. **使用预览模式**：首次预配前先用 `--dry-run` 预览
2. **分组管理**：按环境或用途分组软件包
3. **包名映射**：为不同发行版配置对应的包名
4. **版本控制**：将 `packages.conf` 纳入版本控制
5. **增量预配**：使用 `--only-pkg` 或 `--only-bin` 进行增量更新

---

## 与其他命令的配合

### 与 server 命令配合

```bash
# 先添加服务器
sdkm server add wsl2 root@192.168.1.100 --tags "dev"

# 再预配服务器
sdkm provision wsl2
```

### 与 deploy 命令配合

```bash
# 先部署配置
sdkm deploy prompt wsl2
sdkm deploy git wsl2

# 再预配软件包
sdkm provision wsl2
```

### 完整初始化流程

```bash
# 1. 添加服务器
sdkm server add new-server root@10.0.0.1 --tags "dev"

# 2. 部署配置
sdkm deploy prompt new-server
sdkm deploy git new-server
sdkm deploy bash new-server

# 3. 预配软件包
sdkm provision new-server
```
