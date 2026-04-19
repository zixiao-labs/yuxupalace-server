import { useEffect, useState, type ReactNode } from 'react';
import { Spinner } from '@heroui/react';
import { apiFetch, ApiError } from '../lib/api';
import { loadSession, clearSession } from '../lib/auth';
import { setSession } from '../lib/session-store';
import type { UserProfile } from '../lib/types';

const LOGIN_URL = import.meta.env.VITE_LOGIN_URL || 'http://localhost:5173/login';

interface Props {
  children: ReactNode;
}

/** Gate: validate the stored JWT against /api/auth/me; redirect to uni-login on 401. */
export default function RequireAuth({ children }: Props) {
  const [ready, setReady] = useState(false);

  useEffect(() => {
    const session = loadSession();
    if (!session) {
      redirectToLogin();
      return;
    }
    apiFetch<UserProfile>('/api/auth/me')
      .then((user) => {
        setSession(session.token, user);
        setReady(true);
      })
      .catch((err) => {
        if (err instanceof ApiError && err.status === 401) {
          clearSession();
          redirectToLogin();
        } else {
          // Network/unknown — still render and let downstream pages show
          // their own error states rather than hanging on a splash screen.
          setReady(true);
        }
      });
  }, []);

  if (!ready) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <Spinner />
      </div>
    );
  }

  return <>{children}</>;
}

function redirectToLogin() {
  const ret = encodeURIComponent(window.location.href);
  window.location.href = `${LOGIN_URL}?return=${ret}`;
}
