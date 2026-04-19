# 玉虚宫自托管部署指南

本文面向想要自建玉虚宫 DevOps 平台的运维或个人开发者，说明如何用
Docker 起前后端，并在**自定义域名**或**裸 IP**下提供访问，可选叠加 TLS。

构建/发布镜像的操作见 [docker-hub-setup.md](./docker-hub-setup.md)。
本文假设镜像已经发布到 Docker Hub 或 GHCR，直接 `pull` 即可。

---

## 1. 架构

```
                 ┌──────────────────────────────┐
  user ──TLS──▶  │  edge proxy (Caddy / Nginx)  │  (optional: TLS termination)
                 │  yourdomain.com / <ip>:443   │
                 └───────────────┬──────────────┘
                                 │  http
                                 ▼
                 ┌──────────────────────────────┐
                 │  yuxu-frontend  :8080        │  Nginx serving static SPA
                 │  (Docker image)              │  + reverse-proxying /api,/rpc
                 └───────────────┬──────────────┘
                                 │  http
                                 ▼
                 ┌──────────────────────────────┐
                 │  yuxu-server    :8080        │  Axum + SQLx
                 │  (Docker image)              │  SQLite or PostgreSQL
                 └──────────────────────────────┘
```

前端镜像（`yuxu-frontend`）内置 Nginx：
- 静态 SPA 在 `/`、`/repos/...` 等路径下走 SPA fallback；
- `/api/*` 反代到后端 HTTP；
- `/rpc` 反代到后端 WebSocket（用于实时协作）；
- 容器内健康检查在 `/healthz`（不走后端）。

后端镜像（`yuxu-server`）默认使用 SQLite。如果选 PostgreSQL，参见下文第 5 节。

---

## 2. 先决条件

| 工具 | 最低版本 | 说明 |
|---|---|---|
| Docker Engine | 24.0 | `buildx` 随 engine 自带 |
| Docker Compose | v2 | 用 `docker compose`，不是 legacy `docker-compose` |
| 域名或固定 IP | 任意 | 镜像的 Nginx 走 `server_name _`，不校验 Host |

Linux / macOS / WSL2 均可。1 GB 内存起步；带 PostgreSQL 则建议 2 GB 起。

---

## 3. 最小部署：Docker Compose

### 3.1 建工作目录

```bash
mkdir -p ~/yuxu && cd ~/yuxu
mkdir data           # sqlite 数据持久化
```

### 3.2 生成 JWT 密钥

```bash
# 要求 >= 32 字节。保存到 .env，不要入 git。
echo "YUXU_JWT_SECRET=$(openssl rand -base64 48)" > .env
```

### 3.3 `compose.yml`

```yaml
services:
  yuxu-server:
    # 两个源二选一：
    image: ghcr.io/zixiao-labs/yuxu-server:latest
    # image: zixiaolabs/yuxu-server:latest
    container_name: yuxu-server
    restart: unless-stopped
    env_file: [.env]
    environment:
      YUXU_BIND: "0.0.0.0:8080"
      DATABASE_URL: "sqlite:///data/yuxu.db?mode=rwc"
      YUXU_JWT_TTL_SECS: "86400"
      # 仅当前端通过别的源访问后端时才需要开 CORS：
      # YUXU_CORS_ORIGINS: "https://yourdomain.com"
      RUST_LOG: "yuxu_server=info,tower_http=info"
    volumes:
      - ./data:/data
    healthcheck:
      test: ["CMD", "curl", "-fsS", "http://127.0.0.1:8080/health"]
      interval: 30s
      timeout: 5s
      retries: 3

  yuxu-frontend:
    image: ghcr.io/zixiao-labs/yuxu-frontend:latest
    # image: zixiaolabs/yuxu-frontend:latest
    container_name: yuxu-frontend
    restart: unless-stopped
    depends_on:
      yuxu-server:
        condition: service_healthy
    environment:
      # 关键：Nginx 启动时把 /api 和 /rpc 反代到这里
      YUXU_BACKEND_URL: "http://yuxu-server:8080"
      # 可选：如果你另外部署了独立登录中心
      # YUXU_LOGIN_URL: "https://login.yourdomain.com/login"
    ports:
      - "8080:8080"     # 宿主 8080 → 容器 8080
```

### 3.4 启动

```bash
docker compose pull
docker compose up -d
docker compose logs -f
```

浏览器访问 `http://localhost:8080`，应看到玉虚宫主控台。后端健康检查：

```bash
curl http://localhost:8080/api/dashboard      # 401 未授权是预期
curl http://localhost:8080/healthz            # "ok"（前端自检）
```

---

## 4. 暴露到自定义域名或 IP

前端镜像的 Nginx 写的是 `server_name _`，**任何 Host** 都会命中，所以你
可以直接用域名或裸 IP 访问，不用重建镜像。

### 4.1 方案 A：裸 HTTP（内网 / 测试）

直接暴露宿主的 8080 端口：

```yaml
    ports:
      - "80:8080"       # 前端容器 8080 → 宿主 80
```

然后访问：

- `http://192.168.1.50/` （LAN 固定 IP）
- `http://yuxu.home.arpa/` （自建 DNS / /etc/hosts 条目）
- `http://your-public-ip/` （公网 IP 直连）

### 4.2 方案 B：Caddy 做 TLS 自动证书

最简 —— Caddy 会自动从 Let's Encrypt 申请证书，零配置 HTTPS。

`~/yuxu/Caddyfile`：

```caddyfile
yourdomain.com {
    reverse_proxy yuxu-frontend:8080
}
```

`compose.yml` 里加一段：

```yaml
  caddy:
    image: caddy:2
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy-data:/data
      - caddy-config:/config
    depends_on: [yuxu-frontend]

volumes:
  caddy-data:
  caddy-config:
```

把前端的 `ports:` 去掉（不再直接对外暴露 8080），只让 Caddy 对外：

```yaml
  yuxu-frontend:
    # ports 移除
    expose:
      - "8080"
```

然后 `docker compose up -d`。Caddy 启动后会自动申请证书，
几分钟后 `https://yourdomain.com` 可用。

### 4.3 方案 C：自行管理的 Nginx 做 edge TLS

如果你已经有一个 Nginx 在 443 上终结 TLS：

```nginx
# /etc/nginx/sites-enabled/yuxu.conf
map $http_upgrade $connection_upgrade {
    default upgrade;
    ''      close;
}

server {
    listen 443 ssl http2;
    server_name yourdomain.com;

    ssl_certificate     /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;

    # WebSocket 升级需要手动透传
    location /rpc {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade     $http_upgrade;
        proxy_set_header Connection  $connection_upgrade;
        proxy_set_header Host        $host;
        proxy_read_timeout 3600s;
    }

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host              $host;
        proxy_set_header X-Real-IP         $remote_addr;
        proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

server {
    listen 80;
    server_name yourdomain.com;
    return 301 https://$host$request_uri;
}
```

---

## 5. 可选：PostgreSQL 代替 SQLite

官方发布的镜像默认走 SQLite。需要 PostgreSQL 的话用下面两种方式之一：

### 5.1 自己重建 postgres 变体

```bash
git clone https://github.com/zixiao-labs/yuxupalace-server.git
cd yuxupalace-server
docker build -f crates/yuxu-server/Dockerfile \
  --build-arg FEATURES=postgres \
  -t yuxu-server:latest-postgres .
```

### 5.2 Compose 中加 postgres 服务

```yaml
  postgres:
    image: postgres:16-alpine
    restart: unless-stopped
    environment:
      POSTGRES_PASSWORD: "change-me"
      POSTGRES_DB: yuxu
    volumes:
      - pg-data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres -d yuxu"]
      interval: 10s

  yuxu-server:
    image: yuxu-server:latest-postgres   # 上一步自建的
    environment:
      DATABASE_URL: "postgres://postgres:change-me@postgres:5432/yuxu"
    depends_on:
      postgres:
        condition: service_healthy

volumes:
  pg-data:
```

---

## 6. CORS

如果前端和后端**同源**（通过前端镜像的 Nginx 反代到后端），不需要设置
`YUXU_CORS_ORIGINS`。

只有当前端 SPA 跨源调用后端（例如前端部署在 `https://app.example.com`、
后端部署在 `https://api.example.com`）时，才在 `yuxu-server` 环境变量里加：

```yaml
    environment:
      YUXU_CORS_ORIGINS: "https://app.example.com"
```

多个源用逗号分隔；`*` 只在本地开发用。

---

## 7. 备份 & 升级

### 备份

```bash
# SQLite
docker compose stop yuxu-server
cp data/yuxu.db data/yuxu.db.$(date +%Y%m%d).bak
docker compose start yuxu-server

# PostgreSQL
docker compose exec postgres pg_dump -U postgres yuxu > yuxu-$(date +%Y%m%d).sql
```

### 升级

```bash
docker compose pull          # 拉新 :latest
docker compose up -d         # 滚动重启
docker compose logs -f yuxu-server
```

Tag 版本固定：把 compose 里的 `:latest` 换成 `:1.2.3`，这样升级可控。

---

## 8. 故障排查

| 症状 | 可能原因 | 排查 |
|---|---|---|
| 前端页面 502 Bad Gateway | `YUXU_BACKEND_URL` 不通 | `docker compose exec yuxu-frontend wget -qO- $YUXU_BACKEND_URL/health` |
| 登录后立刻 401 | JWT 密钥每次启动变了 | 检查 `.env` 是否挂载；`YUXU_JWT_SECRET` 是否 >=32 字节 |
| 403 CORS error | 前后端跨源，未配置 CORS | 后端加 `YUXU_CORS_ORIGINS` |
| WebSocket 连不上 | edge 反代丢了 Upgrade 头 | 确认 edge 配置包含 `proxy_set_header Upgrade $http_upgrade` 和足够长的 `proxy_read_timeout` |
| 前端加载慢 | 浏览器 Service Worker 缓存 | 打开 DevTools → Application → Unregister SW 后刷新 |
| 后端 healthcheck 反复失败 | 迁移卡住 / 数据库不可达 | `docker compose logs yuxu-server` |
| `sqlite` 镜像启动报错找不到目录 | `/data` 没挂 volume | 确认 `volumes: - ./data:/data`，且目录由 uid 10001 可写 |

常用排错命令：

```bash
docker compose logs --tail=200 yuxu-frontend
docker compose logs --tail=200 yuxu-server
docker compose exec yuxu-frontend nginx -T | less   # 查看最终 Nginx 配置
docker compose exec yuxu-server env | grep YUXU     # 检查环境变量
```

---

## 9. 安全清单

- [ ] `YUXU_JWT_SECRET` 是 ≥32 字节的随机值，且只在宿主 `.env` 中保存；
- [ ] `.env` 的权限是 600（`chmod 600 .env`）；
- [ ] 公网部署时，仅暴露 edge proxy（80/443），不直接对外暴露 8080；
- [ ] 如果用 PostgreSQL，`POSTGRES_PASSWORD` 独立于 JWT 密钥；
- [ ] 镜像 tag 固定到具体版本而不是 `latest`（便于回滚）；
- [ ] 定期 `docker compose pull` + 查看 [release notes](https://github.com/zixiao-labs/yuxupalace-server/releases) 更新基础镜像；
- [ ] SQLite 数据库所在宿主目录 (`./data`) 有独立备份计划。

---

## 10. 进一步

- 发布自定义镜像 / 修改 Docker Hub 配置：见 [docker-hub-setup.md](./docker-hub-setup.md)
- 报告问题：<https://github.com/zixiao-labs/yuxupalace-server/issues>
