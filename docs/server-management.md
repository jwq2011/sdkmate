# 服务器管理

本文档详述 `sdkm server` 命令的使用方法，用于管理远程服务器配置。部署功能见 [deployment.md](./deployment.md)，预配功能见 [provisioning.md](./provisioning.md)。

命令总览：

| 命令 | 别名 | 作用 |
|:---|:---|:---|
| [`sdkm server add`](#sdkm-server-add) | — | 添加服务器配置 |
| [`sdkm server remove`](#sdkm-server-remove) | `rm` | 移除服务器配置 |
| [`sdkm server list`](#sdkm-server-list) | `ls` | 列出所有服务器 |
| [`sdkm server status`](#sdkm-server-status) | `st` | 查看服务器状态 |

---

## sdkm server add

添加新的服务器配置到 `~/.sdkmate/servers.toml`。

```bash
sdkm server add <alias> <user@host> [OPTIONS]
```

- `<alias>`：服务器别名，用于后续命令引用。
- `<user@host>`：SSH 登录格式，例如 `root@192.168.1.100`。

选项：

| 选项 | 短选项 | 说明 | 默认值 |
|:---|:---|:---|:---|
| `--port PORT` | `-p` | SSH 端口 | 22 |
| `--label LABEL` | `-l` | 服务器标签（可选） | 使用别名 |
| `--key PATH` | — | SSH 私钥路径 | `~/.ssh/id_rsa` |
| `--tags TAGS` | — | 逗号分隔的标签列表 | 空 |

示例：

```bash
# 基本添加
sdkm server add wsl2 root@192.168.1.100

# 完整参数
sdkm server add prod-db admin@10.0.0.50 \
  --port 2222 \
  --label "生产数据库" \
  --key ~/.ssh/prod_key \
  --tags "prod,database"

# 添加开发服务器
sdkm server add dev will@192.168.1.101 --tags "dev,local"
```

> **注意**：如果别名已存在，会报错提示。需要先 `sdkm server remove` 再重新添加。

---

## sdkm server remove

移除服务器配置。

```bash
sdkm server remove <alias>
```

- `<alias>`：要移除的服务器别名。

示例：

```bash
sdkm server remove wsl2
```

> **注意**：此操作仅删除配置，不会影响远程服务器上的任何文件或服务。

---

## sdkm server list

列出所有已配置的服务器。

```bash
sdkm server list [--filter TAG]
```

- `--filter TAG`：按标签过滤服务器。

示例：

```bash
# 列出所有服务器
sdkm server list

# 只显示生产环境服务器
sdkm server list --filter prod

# 只显示本地开发服务器
sdkm server list --filter local
```

输出格式：

```
ALIAS           HOST                      PORT     LABEL                TAGS
--------------------------------------------------------------------------------
wsl2            root@192.168.1.100        22       WSL2 开发环境         dev,local
prod-db         admin@10.0.0.50           2222     生产数据库            prod,database
dev             will@192.168.1.101        22       dev                  dev,local
```

如果没有任何服务器配置，会提示：

```
No servers configured.
Use 'sdkm server add <alias> <user@host>' to add a server.
```

---

## sdkm server status

查看服务器状态（需要 SSH 连接）。

```bash
sdkm server status [--filter TAG] [--json]
```

- `--filter TAG`：按标签过滤服务器。
- `--json`：以 JSON 格式输出。

示例：

```bash
# 查看所有服务器状态
sdkm server status

# 只查看生产环境状态
sdkm server status --filter prod

# JSON 格式输出（便于脚本处理）
sdkm server status --json
```

输出信息包括：
- 操作系统版本
- 内核版本
- 磁盘使用情况
- 内存使用情况
- 已安装的 SDK 版本

---

## 配置文件

服务器配置存储在 `~/.sdkmate/servers.toml`，使用 TOML 格式。

配置文件示例：

```toml
[servers.wsl2]
host = "192.168.1.100"
user = "will"
port = 22
tags = ["dev", "local"]
label = "WSL2 开发环境"

[servers.wsl2.auth]
KeyPath = "/home/will/.ssh/id_rsa"

[servers.prod-db]
host = "10.0.0.50"
user = "admin"
port = 2222
tags = ["prod", "database"]
label = "生产数据库"

[servers.prod-db.auth]
Password = "your-password-here"
```

### 配置项说明

| 字段 | 类型 | 必填 | 说明 |
|:---|:---|:---|:---|
| `host` | string | ✅ | 服务器 IP 或域名 |
| `user` | string | ✅ | SSH 用户名 |
| `port` | integer | ❌ | SSH 端口，默认 22 |
| `tags` | array | ❌ | 标签列表，用于过滤 |
| `label` | string | ❌ | 服务器标签，默认使用别名 |
| `auth` | object | ✅ | 认证方式 |

### 认证方式

支持两种认证方式：

#### 密钥认证（推荐）

```toml
[servers.myserver.auth]
KeyPath = "/path/to/private/key"
```

#### 密码认证

```toml
[servers.myserver.auth]
Password = "your-password"
```

> **安全提示**：建议使用密钥认证，避免在配置文件中存储明文密码。后续版本将支持加密存储。

---

## 标签系统

标签用于对服务器进行分组管理，支持以下场景：

1. **过滤服务器**：`sdkm server list --filter prod`
2. **批量操作**：`sdkm deploy prompt --filter prod`
3. **分类管理**：按环境（dev/prod）、用途（web/db）、位置（local/cloud）等维度组织

标签命名建议：

| 维度 | 示例标签 | 用途 |
|:---|:---|:---|
| 环境 | `dev`, `staging`, `prod` | 区分开发、测试、生产环境 |
| 用途 | `web`, `db`, `cache`, `mq` | 按服务类型分类 |
| 位置 | `local`, `cloud`, `aws`, `aliyun` | 按部署位置分类 |
| 团队 | `frontend`, `backend`, `ops` | 按团队分类 |

示例：

```bash
# 添加带多标签的服务器
sdkm server add web-01 root@10.0.0.1 --tags "prod,web,aws"
sdkm server add db-01 root@10.0.0.2 --tags "prod,db,aws"
sdkm server add dev-fe will@192.168.1.100 --tags "dev,frontend,local"

# 按标签过滤
sdkm server list --filter prod      # 所有生产环境服务器
sdkm server list --filter web       # 所有 Web 服务器
sdkm server list --filter aws       # 所有 AWS 服务器
```

---

## 使用场景

### 场景 1：管理开发环境

```bash
# 添加本地 WSL2 开发服务器
sdkm server add wsl2 root@192.168.1.100 --tags "dev,local" --label "WSL2 开发环境"

# 添加远程开发服务器
sdkm server add dev-remote user@dev.example.com --tags "dev,remote"

# 查看所有开发服务器
sdkm server list --filter dev
```

### 场景 2：管理生产环境

```bash
# 添加生产服务器
sdkm server add prod-web-01 root@10.0.0.1 --tags "prod,web" --port 2222
sdkm server add prod-web-02 root@10.0.0.2 --tags "prod,web" --port 2222
sdkm server add prod-db root@10.0.0.3 --tags "prod,db" --port 2222

# 查看生产环境状态
sdkm server status --filter prod
```

### 场景 3：批量部署配置

```bash
# 部署 prompt 到所有开发服务器
sdkm deploy prompt --filter dev

# 部署到特定服务器
sdkm deploy prompt wsl2 dev-remote
```

---

## 故障排查

### 问题：添加服务器时报错 "Server already exists"

原因：别名已被占用。

解决：

```bash
# 先移除旧配置
sdkm server remove <alias>

# 再重新添加
sdkm server add <alias> <user@host> [OPTIONS]
```

### 问题：配置文件损坏

原因：手动编辑 `servers.toml` 导致 TOML 语法错误。

解决：

1. 用编辑器打开 `~/.sdkmate/servers.toml`
2. 检查 TOML 语法（括号匹配、引号闭合等）
3. 或删除配置文件重新添加服务器

### 问题：SSH 连接失败

原因：密钥路径错误或服务器不可达。

解决：

```bash
# 检查密钥文件是否存在
ls -la ~/.ssh/id_rsa

# 测试 SSH 连接
ssh -i ~/.ssh/id_rsa -p 22 user@host

# 更新服务器配置
sdkm server remove <alias>
sdkm server add <alias> <user@host> --key /correct/key/path
```
