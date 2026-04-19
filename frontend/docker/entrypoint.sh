#!/usr/bin/env bash
# Normalizes and validates YUXU_* env vars before Nginx's built-in
# 20-envsubst-on-templates.sh consumes them. Failing here (instead of
# letting envsubst write a broken conf) makes misconfig loud and early.
set -euo pipefail

if [[ -z "${YUXU_BACKEND_URL:-}" ]]; then
  echo "[yuxu-config] ERROR: YUXU_BACKEND_URL must be set (e.g. http://yuxu-server:8080)." >&2
  exit 1
fi

# Strip trailing slash so `proxy_pass ${YUXU_BACKEND_URL}` combined with the
# /api/ or /rpc locations doesn't produce a double slash.
export YUXU_BACKEND_URL="${YUXU_BACKEND_URL%/}"

if [[ -n "${YUXU_LOGIN_URL:-}" ]]; then
  export YUXU_LOGIN_URL="${YUXU_LOGIN_URL%/}"
fi

echo "[yuxu-config] YUXU_BACKEND_URL=${YUXU_BACKEND_URL}"
if [[ -n "${YUXU_LOGIN_URL:-}" ]]; then
  echo "[yuxu-config] YUXU_LOGIN_URL=${YUXU_LOGIN_URL}"
else
  echo "[yuxu-config] YUXU_LOGIN_URL=<unset, /login falls through to SPA>"
fi
