# Docker Hub 注册与发布配置

本文面向**要发布自建镜像**的维护者，讲解如何从零开始：

1. 注册 Docker Hub 账号并建仓库；
2. 生成 Access Token；
3. 在 GitHub 仓库里配置 Secrets / Variables；
4. 配置 Environment 作为人工审批门。

配完这些，`.github/workflows/docker-publish.yml` 才能成功发布。
想直接**使用**已发布镜像的自托管用户，读 [self-hosted.md](./self-hosted.md) 就够了。

---

## 0. 为什么是两个 Registry

本仓库的发布工作流默认同时推 **Docker Hub** 和 **GitHub Container Registry (GHCR)**：

| 项目 | Docker Hub | GHCR (ghcr.io) |
|---|---|---|
| 公共 pull 限流 | 未登录 100 次 / 6 小时 | 基本无限制（GitHub 账号登录有更高配额） |
| 私有镜像 | 个人免费 1 个 | 组织可免费多个 |
| 要额外账号吗？ | 是 | 否，用 `GITHUB_TOKEN` 即可 |
| 用户体验 | `docker pull foo/bar` | `docker pull ghcr.io/org/bar` |

建议同时推两边：Docker Hub 更符合大众习惯，GHCR 免限流且与仓库权限打通。
如果你只想推 GHCR，触发工作流时把 `push_to_dockerhub` 设成 false 即可，
本节也可以跳过。

---

## 1. 注册 Docker Hub

1. 打开 <https://hub.docker.com/signup>。
2. 用 **组织能长期持有** 的邮箱（不要用个人邮箱，除非你就是唯一维护者）。
3. 选用户名 —— 这就是未来镜像的 namespace 前缀。示例：`zixiaolabs`。
   - 建议风格：lowercase、无 dash 或 underscore（Docker Hub 用户名不允许 dash）。
   - 如果你代表组织 zixiao-labs，可以注册 `zixiaolabs` 或 `zxlabs`。
4. 验证邮箱。

### 升级为组织（可选，推荐长期维护者）

个人账号的所有权绑定在一个人身上，不适合团队。升级：

- Docker Hub 顶部 → **Organizations** → **Create Organization**
- 免费计划允许 **无限公共仓库** + **1 个私有仓库**。
- 把自己加成 Owner；之后邀请协作者进组织。

后续所有 repo 都建在这个组织 namespace 下。

---

## 2. 在 Docker Hub 建 Repository

对每个要发布的镜像建一个仓库：

1. Docker Hub → **Repositories** → **Create Repository**。
2. Namespace：选你的用户名或组织名（例如 `zixiaolabs`）。
3. Name：
   - `yuxu-frontend` — 前端 Nginx 镜像
   - `yuxu-server` — 后端 Rust 镜像
4. 公开性：一般选 **Public**（否则自托管用户 pull 需要登录）。
5. 简介里填上项目仓库链接。

**注意：** 镜像仓库是懒创建的 —— 如果不先建，第一次 push 会因权限提示失败，
所以一定要先建好。

---

## 3. 生成 Access Token

**不要直接用你的 Docker Hub 密码推镜像**（密码泄漏影响整个账号）。

1. Docker Hub 顶部 → 头像 → **Account Settings**。
2. 左栏 **Security** → **New Access Token**。
3. Token 描述：`yuxupalace-ci`（或任何能识别用途的名字）。
4. 权限：选 **Read, Write, Delete**。
   - Read + Write 是最小推送权限；Delete 用于覆盖 tag（CI 里 `:latest` 会反复覆盖）。
5. 点 **Generate**。
6. **马上复制 token** —— 关掉窗口之后就再也看不到了。

Token 格式类似 `dckr_pat_xxxxxxxxxxxxxxxxxxxxxxxxxxxx`。

### 何时轮换

- 默认策略：每 90 天换一次；
- CI 日志泄漏 / 团队成员离职 → 立刻吊销旧 token，生成新的；
- 吊销：Account Settings → Security → 对应 token → Delete。

---

## 4. 在 GitHub 仓库配置

把 token 和用户名放到 GitHub，工作流才能用。

### 4.1 打开配置页

<https://github.com/zixiao-labs/yuxupalace-server/settings/secrets/actions>

（路径：仓库 → **Settings** → **Secrets and variables** → **Actions**）

### 4.2 新增 Repository Variables

进 **Variables** tab（不是 Secrets，因为这不是机密）：

| Name | Value | 说明 |
|---|---|---|
| `DOCKERHUB_USERNAME` | `zixiaolabs` | 你的 Docker Hub 登录名（个人或组织名） |
| `DOCKERHUB_NAMESPACE` | `zixiaolabs` | 镜像前缀。通常等于 USERNAME；如果用组织就写组织名 |

### 4.3 新增 Environment Secret（重要）

Secrets 放在 **Environment** 里而不是 Repository 级别，这样可以配合审批 gate。
详见下一节的 Environment 配置。

---

## 5. 配置审批门（Environment）

这是「发布需要批准」的核心 —— GitHub Environments 支持 required reviewers。

### 5.1 新建 Environment

1. 仓库 → **Settings** → **Environments** → **New environment**。
2. Name：**`docker-publish`** —— 必须和 `docker-publish.yml` 里的
   `environment: docker-publish` 完全一致。
3. Configure environment。

### 5.2 启用 Required reviewers

- 勾 **Required reviewers**。
- 最多加 6 人。加至少 2 位能审批发布的维护者（防止单点）。
- 每次工作流触发后会停在 **"Waiting for approval"** 状态，审批人可以在
  Actions 页面点 **Approve and deploy** 或 **Reject**。

### 5.3 可选：Wait timer

- Wait timer 可以设一个冷静时间（如 5 分钟）——
  审批后不会立刻发布，给人机会在最后一刻叫停。

### 5.4 可选：限制触发分支

- **Deployment branches** → 选 **Protected branches only** 或自定义。
- 推荐：`main` 和 `v*` tag —— 禁止从随便的 feature branch 发布。

### 5.5 放 Secret

- 同一个 Environment 配置页下滑到 **Environment secrets** → **Add secret**：
  - Name：`DOCKERHUB_TOKEN`
  - Value：步骤 3 复制的 `dckr_pat_...` token
- **删掉** Repository 级别同名 secret（如果之前加过）——
  放在 environment 里可以限定只有通过审批的 job 能读到，泄漏面更小。

---

## 6. 首次触发验证

1. 仓库 → **Actions** → 左栏选 **Docker Publish**。
2. 右上 **Run workflow**：
   - Branch: `main`
   - Tag: `latest`
   - Platforms: `linux/amd64`（第一次先单架构，多架构见下文）
   - `push_to_dockerhub` / `push_to_ghcr`：按需勾选
3. 工作流启动后会停在 **Waiting for approval**。
4. 去 Environments → `docker-publish` 点 **Approve and deploy**。
5. 等 build & push 结束（首次冷构后端大约 8–15 分钟）。
6. 验证：

   ```bash
   docker pull zixiaolabs/yuxu-frontend:latest
   docker pull zixiaolabs/yuxu-server:latest
   # 以及（需要先登录 GitHub）
   docker pull ghcr.io/zixiao-labs/yuxu-frontend:latest
   docker pull ghcr.io/zixiao-labs/yuxu-server:latest
   ```

---

## 7. GHCR 特殊事项

### 7.1 首次推送后镜像默认私有

GHCR 新镜像默认 private。想让自托管用户无认证 pull，需要改 public：

1. 打开 <https://github.com/orgs/zixiao-labs/packages>。
2. 点 `yuxu-frontend` 或 `yuxu-server`。
3. 右下 **Package settings** → **Change visibility** → **Public**。

### 7.2 把 Package 绑到仓库

绑定后 package 会继承仓库的权限和页面，而且会自动出现在仓库主页的
Packages 面板里：

- Package settings → **Manage Actions access** → Add repository
  `zixiao-labs/yuxupalace-server`。

### 7.3 GHCR 拉镜像时如果还是私有

```bash
# 生成一个 PAT：https://github.com/settings/tokens?scopes=read:packages
echo $GHCR_PAT | docker login ghcr.io -u <your-github-username> --password-stdin
docker pull ghcr.io/zixiao-labs/yuxu-server:latest
```

---

## 8. 多架构构建（arm64）

默认工作流用 `linux/amd64`。要支持 arm64（Apple Silicon、树莓派）：

- 触发工作流时选 `linux/amd64,linux/arm64`。
- **警告**：Rust 在 QEMU 下编译 arm64 需要 10–20 分钟。
- 生产用法：建议在 Release tag（`v1.2.3` push）时才启用 arm64，
  日常 `latest` 保持单架构。

长期优化思路（不在本 PR 范围）：用 self-hosted arm64 runner 或
`docker/build-push-action` 的 matrix 并行两个架构再 manifest 合并。

---

## 9. 轮换与吊销清单

- [ ] Docker Hub access token 每 90 天轮换
- [ ] 成员离职 24 小时内吊销该成员创建的 token
- [ ] 检查 GitHub Actions 日志里没有意外打印 token
- [ ] 如果仓库曾经有过 Repository-level 的 `DOCKERHUB_TOKEN`（非 environment），
      现在应该只存在于 `docker-publish` Environment 下
- [ ] Docker Hub Security 页面定期查看 **Last Used** 时间，删除长期未用的 token

---

## 10. 常见错误速查

| 错误 | 原因 | 处理 |
|---|---|---|
| `denied: requested access to the resource is denied` (Docker Hub) | 仓库不存在 / token 权限不够 / USERNAME 拼错 | 确认 Docker Hub 里已建 `<ns>/yuxu-frontend`；Token 权限含 Write；`vars.DOCKERHUB_NAMESPACE` 正确 |
| `denied: installation not allowed to Create organization package` (GHCR) | workflow 没有 `packages: write` 权限 | `docker-publish.yml` 顶部 `permissions:` 已经配置；如果 fork 后用自己的 org，组织级设置里要允许 Actions 写 packages |
| Workflow 卡在 "Waiting for approval" 不动 | 没人是 reviewer | Environment 设置里确认自己在 Required reviewers 列表 |
| `Error: buildx failed with: ERROR: ...` (arm64) | QEMU 编译阶段超时 | 只构 amd64，或用 self-hosted arm64 runner |
| tag 是空字符串 | `inputs.tag` 没填 | workflow_dispatch 时手填；tag push 时自动用 semver |

---

## 11. 延伸阅读

- Docker Hub 官方文档：<https://docs.docker.com/docker-hub/access-tokens/>
- GHCR 官方文档：<https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry>
- GitHub Environments：<https://docs.github.com/en/actions/deployment/targeting-different-environments/using-environments-for-deployment>
