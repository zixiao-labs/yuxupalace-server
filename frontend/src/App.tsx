import { ChenRouter, Route, Routes } from 'chen-the-dawnstreak';
import Layout from './pages/_layout';
import Dashboard from './pages/index';
import NotFound from './pages/_404';
import ReposIndex from './pages/repos/index';
import RepoDetail from './pages/repos/[...fullName]';
import SettingsIndex from './pages/settings/index';

export default function App() {
  return (
    <ChenRouter>
      <Routes>
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
