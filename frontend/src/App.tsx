import { ChenRouter, Route, Routes } from 'chen-the-dawnstreak';
import Layout from './pages/_layout';
import Dashboard from './pages/index';
import NotFound from './pages/_404';
import ReposIndex from './pages/repos/index';
import RepoDetail from './pages/repos/[...fullName]';
import SettingsIndex from './pages/settings/index';
import Login from './pages/login';
import Register from './pages/register';
import OauthCallback from './pages/oauth/callback';

export default function App() {
  return (
    <ChenRouter>
      <Routes>
        {/* Auth surfaces live outside the RequireAuth-wrapped Layout. */}
        <Route path="login" element={<Login />} />
        <Route path="register" element={<Register />} />
        <Route path="oauth/callback" element={<OauthCallback />} />
        <Route element={<Layout />}>
          <Route index element={<Dashboard />} />
          <Route path="repos" element={<ReposIndex />} />
          <Route path="repos/*" element={<RepoDetail />} />
          <Route path="settings" element={<SettingsIndex />} />
        </Route>
        <Route path="*" element={<NotFound />} />
      </Routes>
    </ChenRouter>
  );
}
