import { useEffect, useState } from 'react';
import { Link, useSearchParams } from 'chen-the-dawnstreak';
import { Alert, Button, Card, Spinner } from '@heroui/react';
import { apiFetch, ApiError } from '../../lib/api';
import { setSession } from '../../lib/session-store';
import type {
  AuthResponse,
  GithubOauthRequest,
  ZixiaoOauthRequest,
} from '../../lib/types';

const GITHUB_STATE_KEY = 'yuxu_github_oauth_state';
const ZIXIAO_STATE_KEY = 'yuxu_zixiao_oauth_state';
const PROVIDER_KEY = 'yuxu_oauth_provider';
const RETURN_KEY = 'yuxu_oauth_return';

/**
 * Shared OAuth callback page. The provider is determined by the marker
 * we stashed in sessionStorage before the redirect; we can't rely on
 * query params alone because GitHub and Zixiao Cloud both send us back
 * with `?code=...&state=...`.
 */
export default function OauthCallback() {
  const [params] = useSearchParams();
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const code = params.get('code');
    const returnedState = params.get('state');
    const providerError = params.get('error_description') || params.get('error');

    const provider = sessionStorage.getItem(PROVIDER_KEY);
    const returnTo = sessionStorage.getItem(RETURN_KEY) || '/';
    sessionStorage.removeItem(PROVIDER_KEY);
    sessionStorage.removeItem(RETURN_KEY);

    if (providerError) {
      setError(`登录被拒绝：${providerError}`);
      return;
    }
    if (!code || !returnedState || !provider) {
      setError('缺少 code/state 或未找到登录上下文，请重新发起登录');
      return;
    }

    const stateKey = provider === 'github' ? GITHUB_STATE_KEY : ZIXIAO_STATE_KEY;
    const savedState = sessionStorage.getItem(stateKey);
    sessionStorage.removeItem(stateKey);
    if (!savedState || savedState !== returnedState) {
      setError('state 校验失败，请重新发起登录');
      return;
    }

    const redirectUri = `${window.location.origin}/oauth/callback`;

    const promise =
      provider === 'github'
        ? apiFetch<AuthResponse>('/api/auth/github/callback', {
            method: 'POST',
            body: JSON.stringify({
              code,
              state: returnedState,
            } satisfies GithubOauthRequest),
          })
        : apiFetch<AuthResponse>('/api/auth/zixiao/callback', {
            method: 'POST',
            body: JSON.stringify({
              code,
              state: returnedState,
              redirect_uri: redirectUri,
            } satisfies ZixiaoOauthRequest),
          });

    promise
      .then((res) => {
        setSession(res.token, res.user);
        window.location.href = returnTo;
      })
      .catch((err) => {
        setError(err instanceof ApiError ? err.message : '登录失败，请稍后重试');
      });
  }, [params]);

  if (error) {
    return (
      <div className="mx-auto mt-24 w-full max-w-md px-4">
        <Card>
          <Card.Header>
            <Card.Title>登录失败</Card.Title>
          </Card.Header>
          <Card.Content>
            <Alert status="danger">
              <Alert.Indicator />
              <Alert.Content>
                <Alert.Title>{error}</Alert.Title>
              </Alert.Content>
            </Alert>
          </Card.Content>
          <Card.Footer className="mt-4">
            <Link to="/login">
              <Button fullWidth variant="tertiary">
                返回登录
              </Button>
            </Link>
          </Card.Footer>
        </Card>
      </div>
    );
  }

  return (
    <div className="flex min-h-screen flex-col items-center justify-center gap-4">
      <Spinner />
      <p style={{ color: 'var(--muted)' }}>正在完成登录…</p>
    </div>
  );
}
