import { Avatar, Button, Tooltip } from '@heroui/react';
import { useSession, logout } from '../lib/session-store';

const LOGIN_URL = import.meta.env.VITE_LOGIN_URL || 'http://localhost:5173/login';

export default function Header() {
  const state = useSession();
  const user = state.session?.user;

  function handleLogout() {
    logout();
    window.location.href = LOGIN_URL;
  }

  return (
    <header
      className="flex items-center justify-between px-6 py-3"
      style={{
        background: 'var(--surface)',
        borderBottom: '1px solid var(--separator)',
      }}
    >
      <div className="text-sm" style={{ color: 'var(--muted)' }}>
        {user ? `欢迎回来，${user.display_name || user.username}` : ''}
      </div>
      <div className="flex items-center gap-3">
        {user ? (
          <>
            <Tooltip>
              <Tooltip.Trigger>
                <Avatar className="size-8" aria-label={user.username}>
                  {user.avatar_url ? <Avatar.Image src={user.avatar_url} alt={user.username} /> : null}
                  <Avatar.Fallback>{initials(user.username)}</Avatar.Fallback>
                </Avatar>
              </Tooltip.Trigger>
              <Tooltip.Content>
                {user.username}
                {user.is_admin ? ' · admin' : ''}
              </Tooltip.Content>
            </Tooltip>
            <Button size="sm" variant="ghost" onPress={handleLogout}>
              登出
            </Button>
          </>
        ) : null}
      </div>
    </header>
  );
}

function initials(name: string): string {
  return name.slice(0, 2).toUpperCase();
}
