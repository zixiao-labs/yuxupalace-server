import { useEffect, useState, type FormEvent } from 'react';
import { Link, useSearchParams } from 'chen-the-dawnstreak';
import {
  Alert,
  Button,
  Card,
  FieldError,
  Form,
  Input,
  Label,
  Spinner,
  TextField,
} from '@heroui/react';
import { apiFetch, ApiError } from '../lib/api';
import { setSession } from '../lib/session-store';
import type {
  AuthConfigResponse,
  AuthResponse,
  LoginRequest,
} from '../lib/types';

const GITHUB_STATE_KEY = 'yuxu_github_oauth_state';
const ZIXIAO_STATE_KEY = 'yuxu_zixiao_oauth_state';
const PROVIDER_KEY = 'yuxu_oauth_provider';
const RETURN_KEY = 'yuxu_oauth_return';

/**
 * Own-login page for the yuxu console. Talks to /api/auth/config to decide
 * which providers to render, so the SaaS build doesn't need to embed any
 * knowledge of uni-login's frontend. In self-hosted mode the local
 * username/password form still appears alongside the OAuth buttons.
 */
export default function Login() {
  const [params] = useSearchParams();
  const returnTo = params.get('return') || '/';

  const [cfg, setCfg] = useState<AuthConfigResponse | null>(null);
  const [cfgError, setCfgError] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    apiFetch<AuthConfigResponse>('/api/auth/config')
      .then(setCfg)
      .catch((err) => {
        setCfgError(
          err instanceof ApiError
            ? err.message
            : '无法读取后端登录配置，请检查服务状态',
        );
      });
  }, []);

  async function handleLocalSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setFormError(null);
    const fd = new FormData(e.currentTarget);
    const payload: LoginRequest = {
      username_or_email: String(fd.get('username_or_email') ?? '').trim(),
      password: String(fd.get('password') ?? ''),
    };
    if (!payload.username_or_email || !payload.password) {
      setFormError('请填写用户名和密码');
      return;
    }
    setLoading(true);
    try {
      const res = await apiFetch<AuthResponse>('/api/auth/login', {
        method: 'POST',
        body: JSON.stringify(payload),
      });
      setSession(res.token, res.user);
      window.location.href = returnTo;
    } catch (err) {
      setFormError(err instanceof ApiError ? err.message : '登录失败，请稍后重试');
    } finally {
      setLoading(false);
    }
  }

  function beginGithubOauth() {
    if (!cfg?.providers.github) return;
    const state = crypto.randomUUID();
    sessionStorage.setItem(GITHUB_STATE_KEY, state);
    sessionStorage.setItem(PROVIDER_KEY, 'github');
    sessionStorage.setItem(RETURN_KEY, returnTo);
    const redirectUri = `${window.location.origin}/oauth/callback`;
    // We intentionally do NOT pass the GitHub client_id from anywhere public —
    // we expect the yuxu backend to have it in YUXU_GITHUB_CLIENT_ID and will
    // hand the code back to /api/auth/github/callback. So the browser needs
    // the public id too; read it from the provider config endpoint if the
    // server exposed it. For now we fall back to a dedicated env var.
    const githubClientId = import.meta.env.VITE_GITHUB_CLIENT_ID;
    if (!githubClientId) {
      setFormError(
        'GitHub 登录未配置 VITE_GITHUB_CLIENT_ID（前端构建时需要）',
      );
      return;
    }
    const q = new URLSearchParams({
      client_id: githubClientId,
      scope: 'read:user user:email',
      redirect_uri: redirectUri,
      state,
    });
    window.location.href = `https://github.com/login/oauth/authorize?${q.toString()}`;
  }

  function beginZixiaoOauth() {
    const zx = cfg?.providers.zixiao_cloud;
    if (!zx) return;
    const state = crypto.randomUUID();
    sessionStorage.setItem(ZIXIAO_STATE_KEY, state);
    sessionStorage.setItem(PROVIDER_KEY, 'zixiao');
    sessionStorage.setItem(RETURN_KEY, returnTo);
    const redirectUri = `${window.location.origin}/oauth/callback`;
    const q = new URLSearchParams({
      client_id: zx.client_id,
      response_type: 'code',
      redirect_uri: redirectUri,
      state,
      scope: 'openid profile email',
    });
    window.location.href = `${zx.base_url}/oauth/authorize?${q.toString()}`;
  }

  if (cfgError) {
    return (
      <div className="mx-auto mt-24 w-full max-w-md px-4">
        <Card>
          <Card.Header>
            <Card.Title>无法加载登录页</Card.Title>
          </Card.Header>
          <Card.Content>
            <Alert status="danger">
              <Alert.Indicator />
              <Alert.Content>
                <Alert.Title>{cfgError}</Alert.Title>
              </Alert.Content>
            </Alert>
          </Card.Content>
        </Card>
      </div>
    );
  }

  if (!cfg) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <Spinner />
      </div>
    );
  }

  const { local, github, zixiao_cloud } = cfg.providers;

  return (
    <div className="mx-auto mt-16 w-full max-w-md px-4">
      <div className="mb-6 text-center">
        <h1
          className="text-xl font-semibold"
          style={{ color: 'var(--foreground)' }}
        >
          登录玉虚宫
        </h1>
        <p className="mt-1 text-xs" style={{ color: 'var(--muted)' }}>
          YuXuPalace DevOps ·{' '}
          {cfg.deployment_mode === 'saas' ? 'SaaS' : 'Self-hosted'}
        </p>
      </div>
      <Card>
        <Card.Content className="flex flex-col gap-4">
          {formError ? (
            <Alert status="danger">
              <Alert.Indicator />
              <Alert.Content>
                <Alert.Title>{formError}</Alert.Title>
              </Alert.Content>
            </Alert>
          ) : null}

          {local ? (
            <Form onSubmit={handleLocalSubmit}>
              <div className="flex flex-col gap-4">
                <TextField name="username_or_email" isRequired autoComplete="username">
                  <Label>用户名或邮箱</Label>
                  <Input placeholder="admin 或 admin@example.com" />
                  <FieldError />
                </TextField>
                <TextField
                  name="password"
                  type="password"
                  isRequired
                  autoComplete="current-password"
                >
                  <Label>密码</Label>
                  <Input placeholder="••••••••" />
                  <FieldError />
                </TextField>
                <Button type="submit" fullWidth isPending={loading}>
                  {loading ? '登录中…' : '登录'}
                </Button>
              </div>
            </Form>
          ) : (
            <div
              className="rounded-md border p-3 text-sm"
              style={{
                borderColor: 'var(--border)',
                color: 'var(--muted)',
                background: 'var(--card)',
              }}
            >
              当前为 SaaS 部署，密码登录已关闭；请使用下方的云账号或 GitHub 登录。
            </div>
          )}

          {(local && (github || zixiao_cloud)) || (github && zixiao_cloud) ? (
            <div className="relative my-1 flex items-center justify-center">
              <div
                className="absolute inset-x-0 top-1/2 border-t"
                style={{ borderColor: 'var(--border)' }}
              />
              <span
                className="relative px-2 text-xs"
                style={{
                  color: 'var(--muted)',
                  background: 'var(--card)',
                }}
              >
                或使用
              </span>
            </div>
          ) : null}

          {zixiao_cloud ? (
            <Button
              type="button"
              fullWidth
              variant="secondary"
              onPress={beginZixiaoOauth}
            >
              使用 紫霄实验室云账号 登录
            </Button>
          ) : null}

          {github ? (
            <Button type="button" fullWidth variant="tertiary" onPress={beginGithubOauth}>
              使用 GitHub 登录
            </Button>
          ) : null}
        </Card.Content>
        {local ? (
          <Card.Footer className="mt-4">
            <div
              className="w-full text-center text-sm"
              style={{ color: 'var(--muted)' }}
            >
              还没有账号？<Link to="/register">立即注册</Link>
            </div>
          </Card.Footer>
        ) : null}
      </Card>
    </div>
  );
}
