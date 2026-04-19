import { Outlet } from 'chen-the-dawnstreak';
import RequireAuth from '../components/RequireAuth';
import Sidebar from '../components/Sidebar';
import Header from '../components/Header';

export default function Layout() {
  return (
    <RequireAuth>
      <div
        className="flex min-h-screen"
        style={{ background: 'var(--background)', color: 'var(--foreground)' }}
      >
        <Sidebar />
        <div className="flex min-w-0 flex-1 flex-col">
          <Header />
          <main className="flex-1 overflow-y-auto p-6">
            <Outlet />
          </main>
        </div>
      </div>
    </RequireAuth>
  );
}
