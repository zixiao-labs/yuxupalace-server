import { NavLink } from 'chen-the-dawnstreak';

const items: Array<{ to: string; label: string; icon: string }> = [
  { to: '/', label: '仪表盘', icon: '🏠' },
  { to: '/repos', label: '仓库', icon: '📦' },
  { to: '/settings', label: '设置', icon: '⚙️' },
];

export default function Sidebar() {
  return (
    <aside
      className="flex flex-col gap-1 p-4"
      style={{
        width: 224,
        background: 'var(--surface-secondary)',
        borderRight: '1px solid var(--separator)',
      }}
    >
      <div className="mb-4 flex items-center gap-2">
        <div
          className="flex h-8 w-8 items-center justify-center rounded-lg text-sm font-bold"
          style={{ background: 'var(--accent)', color: 'var(--accent-foreground)' }}
        >
          玉
        </div>
        <div className="flex flex-col leading-tight">
          <span className="text-sm font-semibold" style={{ color: 'var(--foreground)' }}>
            玉虚宫
          </span>
          <span className="text-xs" style={{ color: 'var(--muted)' }}>
            DevOps Console
          </span>
        </div>
      </div>
      <nav className="flex flex-col gap-1">
        {items.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.to === '/'}
            style={({ isActive }) => ({
              display: 'flex',
              alignItems: 'center',
              gap: 10,
              padding: '8px 12px',
              borderRadius: 10,
              textDecoration: 'none',
              fontSize: 14,
              color: isActive ? 'var(--accent-foreground)' : 'var(--foreground)',
              background: isActive ? 'var(--accent)' : 'transparent',
            })}
          >
            <span>{item.icon}</span>
            <span>{item.label}</span>
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}
