# Yuxupalace-Server

~~Zixiao Labs DevOps Service（代号：玉虚宫）~~ 不想重新起名了，直接叫玉虚宫吧

开发者的一站式工作台，集成Git仓库，Issue，合并请求，CI/CD，基于CRDT的实时协作

支持[Logos](https://github.com/zixiao-labs/logos)、~~[Zed](https://github.com/amiya167/zed)~~ 由于官方暂时Out Of Scope，~~Zed支持将通过扩展程序提供~~ 等不到视觉API了，还是用Zed同款的GPUI手搓一个GUI Git Client吧、CLI客户端，并会推出手机客户端

## 技术栈

前端React+自研chen-the-dawnstreak+MUI+自研Nasti打包

后端Rust (Cargo Workspace, Axum, sqlx, PostgreSQL, git2, ~~yrs CRDT~~ 手搓CRDT)

## 仓库结构

```
devops-service/
├── Cargo.toml                  # Cargo Workspace root
├── migrations/                 # sqlx database migrations (PostgreSQL)
├── docs/
│   └── logos-integration.md   # Logos IDE integration guide
├── crates/
│   ├── raidian/               # Shared Protobuf crate (publishable)
│   │   └── proto/             # 8 .proto files
│   ├── yuxu-core/             # Shared business logic (auth, ACL, git ops)
│   ├── yuxu-server/           # Axum HTTP + WebSocket server
│   └── yuxu-cli/              # Command-line client
└── frontend/yuxu/             # React frontend (WIP)
```

## 客户端

| 客户端 | 状态 | 说明 |
|--------|------|------|
| [yuxu-cli](crates/yuxu-cli) | ✅ MVP | Rust CLI，支持所有核心功能 |
| [Zed 面板](https://github.com/amiya167/zed) | ✅ MVP | Zed 编辑器内嵌 DevOps 面板（无Auth） |
| [Logos IDE](https://github.com/zixiao-labs/logos) | 🚧 计划中 | 第一方 IDE 客户端，见 [集成文档](docs/logos-integration.md) |
| 手机客户端 | 🔜 计划中 | iOS/Android（React Native+chen-the-dawnstreak） |

## 核心功能

- **Git 仓库管理** — 创建/浏览/管理 bare 仓库，分支、树、提交记录
- **Issue 追踪** — 创建、评论、标签、关闭
- **合并请求** — 全生命周期管理，代码审查，三种合并策略（merge/squash/rebase）
- **分支保护** — 要求 CI 通过 + 审批数量才能合并
- **CI/CD 流水线** — 触发、状态追踪、阶段日志
- **实时协作** — 基于自研 CRDT 协同编辑，WebSocket 传输
- **ACL 访问控制** — owner/maintainer/developer/reporter/guest 五级角色
- **Protobuf API** — `raidian` crate 提供所有消息类型，可直接作为依赖使用

## 快速开始

```bash
# 启动 PostgreSQL 并运行迁移
sqlx migrate run

# 启动服务器
cargo run -p yuxu-server

# 使用 CLI
cargo run -p yuxu-cli -- auth login --username admin --password secret
cargo run -p yuxu-cli -- repo list
```

## 配置

服务器通过环境变量配置：

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `DATABASE_URL` | — | PostgreSQL 连接字符串 |
| `JWT_SECRET` | — | JWT 签名密钥 |
| `GIT_ROOT` | `./repos` | 裸仓库存储路径 |
| `HOST` | `0.0.0.0` | 监听地址 |
| `PORT` | `3000` | 监听端口 |

## Protobuf / Raidian

`raidian` crate 包含所有消息类型的 `.proto` 定义，通过 `prost` 编译，并附加 `serde` 派生。

现已上架crates.io

```bash
cargo add raidian
```


或者通过 git 依赖使用：

```toml
[dependencies]
raidian = { git = "https://github.com/zixiao-labs/yuxupalace-server", package = "raidian" }
```


## 实时协作协议

WebSocket 端点：`/api/v1/collab/ws`

二进制帧格式（1字节类型前缀）：

| 类型字节 | 消息类型 | 说明 |
|---------|---------|------|
| `0x01` | Join | 加入协作房间 |
| `0x02` | JoinAck | 加入确认 + 当前状态向量 |
| `0x03` | SyncUpdate | CRDT 增量更新 |
| `0x04` | Awareness | 光标/选区 awareness 数据 |
| `0x05` | ParticipantJoined | 新参与者加入通知 |
| `0x06` | ParticipantLeft | 参与者离开通知 |

详细集成说明见 [docs/logos-integration.md](docs/logos-integration.md)。

## 关于名字

玉虚宫，元始天尊的道场（Claude帮忙起的~~文化输出~~，其实一开始就是~~一个能在OpenWRT路由器上跑的GitLab~~）

这并不是像Github那样冰冷的工具，这是一个开发者的一站式工作台+欢迎所有人的社区
