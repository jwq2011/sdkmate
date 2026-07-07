# 配置部署

本文档详述 `sdkm deploy` 命令的使用方法，用于将配置文件部署到远程服务器。服务器管理见 [server-management.md](./server-management.md)，预配功能见 [provisioning.md](./provisioning.md)。

命令总览：

| 命令 | 别名 | 作用 |
|:---|:---|:---|
| [`sdkm deploy prompt`](#sdkm-deploy-prompt) | — | 部署 Shell 提示符配置 |
| [`sdkm deploy git`](#sdkm-deploy-git) | — | 部署 Git 配置 |
| [`sdkm deploy bash`](#sdkm-deploy-bash) | — | 部署 Bash 配置 |

---

## sdkm deploy prompt

部署 Shell 提示符配置到远程服务器。

```bash
sdkm deploy prompt <targets...> [OPTIONS]
```

- `<targets...>`：目标服务器，可以是别名或 `user@host` 格式，支持多个。

选项：

| 选项 | 短选项 | 说明 | 默认值 |
|:---|:---|:---|:---|
| `--dry-run` | `-d` | 预览模式，不实际执行 | false |
| `--parallel LIMIT` | `-p` | 并行部署数量限制 | 5 |

示例：

```bash
# 部署到单台服务器
sdkm deploy prompt wsl2

# 部署到多台服务器
sdkm deploy prompt wsl2 dev-remote

# 使用 user@host 格式
sdkm deploy prompt root@192.168.1.100

# 预览模式
sdkm deploy prompt wsl2 --dry-run

# 限制并行数量
sdkm deploy prompt server1 server2 server3 --parallel 2
```

部署内容：
- 源文件：`modules/prompt/my_prompt_rc`
- 目标路径：`~/.my_prompt_rc`
- 占位符：`{{SERVER_LABEL}}`

---

## sdkm deploy git

部署 Git 配置到远程服务器。

```bash
sdkm deploy git <targets...> [OPTIONS]
```

- `<targets...>`：目标服务器，可以是别名或 `user@host` 格式，支持多个。

选项：

| 选项 | 短选项 | 说明 | 默认值 |
|:---|:---|:---|:---|
| `--dry-run` | `-d` | 预览模式，不实际执行 | false |
| `--parallel LIMIT` | `-p` | 并行部署数量限制 | 5 |

示例：

```bash
# 部署到单台服务器
sdkm deploy git wsl2

# 部署到多台服务器
sdkm deploy git wsl2 dev-remote prod-web-01

# 预览模式
sdkm deploy git wsl2 --dry-run
```

部署内容：
- 源文件：`modules/git/.gitconfig`
- 目标路径：`~/.gitconfig`
- 占位符：`{{GIT_NAME}}`, `{{GIT_EMAIL}}`

---

## sdkm deploy bash

部署 Bash 配置到远程服务器。

```bash
sdkm deploy bash <targets...> [OPTIONS]
```

- `<targets...>`：目标服务器，可以是别名或 `user@host` 格式，支持多个。

选项：

| 选项 | 短选项 | 说明 | 默认值 |
|:---|:---|:---|:---|
| `--dry-run` | `-d` | 预览模式，不实际执行 | false |
| `--parallel LIMIT` | `-p` | 并行部署数量限制 | 5 |

示例：

```bash
# 部署到单台服务器
sdkm deploy bash wsl2

# 部署到多台服务器
sdkm deploy bash wsl2 dev-remote

# 预览模式
sdkm deploy bash wsl2 --dry-run
```

部署内容：
- 源文件：`modules/bash/.bashrc`
- 目标路径：`~/.bashrc`
- 占位符：无

---

## 目标服务器格式

支持两种目标服务器格式：

### 1. 服务器别名

使用 `sdkm server add` 添加的服务器别名：

```bash
sdkm deploy prompt wsl2
sdkm deploy prompt prod-web-01 prod-web-02
```

### 2. user@host 格式

直接指定 SSH 登录信息：

```bash
sdkm deploy prompt root@192.168.1.100
sdkm deploy prompt user@host:2222  # 自定义端口（暂不支持）
```

> **注意**：使用 `user@host` 格式时，SSH 端口默认为 22，密钥使用 `~/.ssh/id_rsa`。

### 3. 混合使用

可以同时使用别名和 `user@host` 格式：

```bash
sdkm deploy prompt wsl2 root@192.168.1.100 dev-remote
```

---

## 模板渲染

部署过程中会自动替换配置文件中的占位符。

### 占位符语法

使用 `{{变量名}}` 格式：

```bash
# 原始内容
export PS1="\u@{{SERVER_LABEL}}:\w\$ "

# 渲染后
export PS1="\u@WSL2 开发环境:\w\$ "
```

### 内置变量

| 变量 | 说明 | 示例值 |
|:---|:---|:---|
| `SERVER_LABEL` | 服务器标签 | `WSL2 开发环境` |
| `GIT_NAME` | Git 用户名 | `Your Name` |
| `GIT_EMAIL` | Git 邮箱 | `your@email.com` |

### 自定义变量

可以通过环境变量传递自定义变量：

```bash
export MY_VAR="custom value"
sdkm deploy prompt wsl2
```

---

## 并行部署

当部署到多台服务器时，默认并行执行以提高效率。

### 并行限制

默认并行数量为 5，可以通过 `--parallel` 选项调整：

```bash
# 限制为 2 个并行
sdkm deploy prompt server1 server2 server3 --parallel 2

# 串行执行（用于调试）
sdkm deploy prompt server1 server2 server3 --parallel 1
```

### 并行部署流程

1. 解析目标服务器列表
2. 为每台服务器创建独立的 SSH 连接
3. 并行执行模板渲染和文件上传
4. 收集所有服务器的执行结果
5. 输出汇总报告

---

## 预览模式

使用 `--dry-run` 选项可以预览部署操作，不实际执行：

```bash
sdkm deploy prompt wsl2 --dry-run
```

输出示例：

```
Deploying module 'prompt' to 1 server(s)...
[DRY RUN] Would deploy prompt to wsl2 (WSL2 开发环境)
```

预览模式下会显示：
- 目标服务器列表
- 部署模块名称
- 源文件路径
- 目标路径

---

## 部署流程

实际部署过程包括以下步骤：

1. **解析目标服务器**：从配置文件加载服务器信息
2. **建立 SSH 连接**：使用配置的认证方式连接服务器
3. **读取源文件**：从本地读取配置模板
4. **渲染模板**：替换占位符为实际值
5. **创建远程目录**：确保目标目录存在
6. **上传文件**：通过 SCP 传输渲染后的文件
7. **验证结果**：检查文件是否成功写入

---

## 使用场景

### 场景 1：初始化新服务器

```bash
# 添加服务器
sdkm server add new-server root@10.0.0.1 --tags "dev"

# 部署所有配置
sdkm deploy prompt new-server
sdkm deploy git new-server
sdkm deploy bash new-server
```

### 场景 2：批量更新配置

```bash
# 更新所有生产服务器的 prompt
sdkm deploy prompt --filter prod

# 更新所有开发服务器的 git 配置
sdkm deploy git --filter dev
```

### 场景 3：配置迁移

```bash
# 从旧服务器迁移到新服务器
sdkm deploy prompt old-server --dry-run  # 预览
sdkm deploy prompt new-server            # 执行
```

---

## 故障排查

### 问题：SSH 连接失败

原因：密钥路径错误、服务器不可达或认证失败。

解决：

```bash
# 检查服务器配置
sdkm server list

# 测试 SSH 连接
ssh -i ~/.ssh/id_rsa -p 22 user@host

# 更新服务器配置
sdkm server remove <alias>
sdkm server add <alias> <user@host> --key /correct/key/path
```

### 问题：模板渲染失败

原因：源文件不存在或占位符语法错误。

解决：

```bash
# 检查源文件是否存在
ls -la modules/prompt/my_prompt_rc

# 检查占位符语法
grep "{{" modules/prompt/my_prompt_rc
```

### 问题：文件上传失败

原因：目标目录权限不足或磁盘空间不足。

解决：

```bash
# 检查目标目录权限
ssh user@host "ls -la ~"

# 检查磁盘空间
ssh user@host "df -h"
```

---

## 最佳实践

1. **使用预览模式**：首次部署前先用 `--dry-run` 预览
2. **批量操作**：使用标签进行批量部署，提高效率
3. **并行限制**：网络不稳定时降低并行数量
4. **配置备份**：重要配置部署前先备份远程文件
5. **版本控制**：将配置模板纳入版本控制

---

## 与其他命令的配合

### 与 server 命令配合

```bash
# 先添加服务器
sdkm server add wsl2 root@192.168.1.100 --tags "dev"

# 再部署配置
sdkm deploy prompt wsl2
```

### 与 provision 命令配合

```bash
# 先部署配置
sdkm deploy prompt wsl2
sdkm deploy git wsl2

# 再预配软件包
sdkm provision wsl2
```
