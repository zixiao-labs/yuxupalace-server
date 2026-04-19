# syntax=docker/dockerfile:1.7
#
# Multi-stage build: Node 20 builds the SPA, Nginx serves static assets and
# reverse-proxies /api and /rpc to $YUXU_BACKEND_URL at container start.
#
# One image adapts to any deployment: the backend URL, login center URL, and
# listening hostname/IP are all resolved at run time — no rebuild per site.

# ---- Stage 1: build the static bundle ----
FROM node:20-alpine AS builder
WORKDIR /app

# Install deps first with only the manifests copied, so code edits don't
# invalidate the dependency layer.
COPY package.json yarn.lock ./
RUN yarn install --frozen-lockfile

COPY . .

# Override .env.development so the production bundle never hardcodes a backend.
# Empty VITE_API_BASE_URL => SPA uses same-origin fetches; Nginx proxies them.
# VITE_LOGIN_URL defaults to /login (same-origin); Nginx optionally 302s to the
# real login center based on $YUXU_LOGIN_URL.
ENV VITE_API_BASE_URL=""
ENV VITE_LOGIN_URL="/login"

RUN yarn build

# ---- Stage 2: runtime (Nginx + envsubst) ----
FROM nginx:1.29-alpine AS runtime

# wget is in busybox; bash is needed for our entrypoint's validation.
RUN apk add --no-cache bash

# Drop the stock server block so our template is the only one loaded.
RUN rm -f /etc/nginx/conf.d/default.conf

COPY --from=builder /app/dist /usr/share/nginx/html

# /etc/nginx/templates is processed by 20-envsubst-on-templates.sh from the
# official image, which writes to /etc/nginx/conf.d/<name>.
COPY docker/nginx.conf.template /etc/nginx/templates/yuxu.conf.template
# Named 10-* so it runs BEFORE the official 20-envsubst-on-templates.sh and
# gets to normalize env vars (strip trailing slashes, validate required ones)
# before envsubst consumes them.
COPY docker/entrypoint.sh /docker-entrypoint.d/10-yuxu-config.sh
RUN chmod +x /docker-entrypoint.d/10-yuxu-config.sh

# Defaults suitable for a docker compose stack with a service named yuxu-server.
# Override at `docker run -e ...` or via compose.
ENV YUXU_BACKEND_URL="http://yuxu-server:8080"
ENV YUXU_LOGIN_URL=""
# Limit envsubst to YUXU_* so Nginx variables like $host/$remote_addr survive.
ENV NGINX_ENVSUBST_FILTER="^YUXU_.*$"

# 8080 lets the container run as a non-root user in hardened orchestrators.
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --retries=3 \
  CMD wget -qO- http://127.0.0.1:8080/healthz >/dev/null || exit 1
