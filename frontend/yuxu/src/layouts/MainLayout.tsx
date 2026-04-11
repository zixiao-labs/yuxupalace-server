import { useState } from 'react';
import { Outlet, NavLink, useLocation } from 'chen-the-dawnstreak';
import Box from '@mui/material/Box';
import Drawer from '@mui/material/Drawer';
import AppBar from '@mui/material/AppBar';
import Toolbar from '@mui/material/Toolbar';
import Typography from '@mui/material/Typography';
import List from '@mui/material/List';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemIcon from '@mui/material/ListItemIcon';
import ListItemText from '@mui/material/ListItemText';
import ListSubheader from '@mui/material/ListSubheader';
import IconButton from '@mui/material/IconButton';
import InputBase from '@mui/material/InputBase';
import Avatar from '@mui/material/Avatar';
import Chip from '@mui/material/Chip';
import MenuIcon from '@mui/icons-material/Menu';
import DashboardIcon from '@mui/icons-material/Dashboard';
import FolderIcon from '@mui/icons-material/Folder';
import BugReportIcon from '@mui/icons-material/BugReport';
import MergeIcon from '@mui/icons-material/MergeType';
import RocketLaunchIcon from '@mui/icons-material/RocketLaunch';
import GroupIcon from '@mui/icons-material/Group';
import SettingsIcon from '@mui/icons-material/Settings';
import SearchIcon from '@mui/icons-material/Search';
import NotificationsNoneIcon from '@mui/icons-material/NotificationsNone';
import AddIcon from '@mui/icons-material/Add';

const DRAWER_WIDTH = 260;

const navItems = [
  { label: '工作台', path: '/', icon: <DashboardIcon fontSize="small" /> },
  { label: '仓库', path: '/repos', icon: <FolderIcon fontSize="small" /> },
  { label: '议题', path: '/issues', icon: <BugReportIcon fontSize="small" /> },
  { label: '合并请求', path: '/merge-requests', icon: <MergeIcon fontSize="small" /> },
  { label: 'CI/CD', path: '/ci', icon: <RocketLaunchIcon fontSize="small" /> },
];

const adminItems = [
  { label: '成员', path: '/members', icon: <GroupIcon fontSize="small" /> },
  { label: '设置', path: '/settings', icon: <SettingsIcon fontSize="small" /> },
];

function isActive(currentPath: string, itemPath: string) {
  if (itemPath === '/') return currentPath === '/';
  return currentPath.startsWith(itemPath);
}

export default function MainLayout() {
  const [mobileOpen, setMobileOpen] = useState(false);
  const location = useLocation();

  const drawer = (
    <Box sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <Toolbar sx={{ gap: 1.5, px: 2 }}>
        <Box
          sx={{
            width: 32,
            height: 32,
            borderRadius: 1,
            background: 'linear-gradient(135deg, #7c4dff 0%, #00e5ff 100%)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontWeight: 700,
            fontSize: 14,
            color: '#fff',
          }}
        >
          YX
        </Box>
        <Box>
          <Typography variant="subtitle2" sx={{ fontWeight: 700, lineHeight: 1.2 }}>
            玉虚宫
          </Typography>
          <Typography variant="caption" sx={{ color: 'text.secondary', lineHeight: 1 }}>
            Zixiao Labs
          </Typography>
        </Box>
      </Toolbar>

      <List sx={{ flex: 1, pt: 1 }}>
        {navItems.map((item) => (
          <ListItemButton
            key={item.path}
            component={NavLink}
            to={item.path}
            selected={isActive(location.pathname, item.path)}
            sx={{ mb: 0.5, py: 0.75 }}
          >
            <ListItemIcon sx={{ minWidth: 36, color: 'inherit' }}>
              {item.icon}
            </ListItemIcon>
            <ListItemText
              primary={item.label}
              slotProps={{ primary: { sx: { fontSize: 14 } } }}
            />
          </ListItemButton>
        ))}
      </List>

      <List
        subheader={
          <ListSubheader
            sx={{
              bgcolor: 'transparent',
              fontSize: 11,
              fontWeight: 600,
              textTransform: 'uppercase',
              letterSpacing: 1,
              color: 'text.secondary',
              lineHeight: '32px',
            }}
          >
            管理
          </ListSubheader>
        }
        sx={{ pb: 2 }}
      >
        {adminItems.map((item) => (
          <ListItemButton
            key={item.path}
            component={NavLink}
            to={item.path}
            selected={isActive(location.pathname, item.path)}
            sx={{ mb: 0.5, py: 0.75 }}
          >
            <ListItemIcon sx={{ minWidth: 36, color: 'inherit' }}>
              {item.icon}
            </ListItemIcon>
            <ListItemText
              primary={item.label}
              slotProps={{ primary: { sx: { fontSize: 14 } } }}
            />
          </ListItemButton>
        ))}
      </List>
    </Box>
  );

  return (
    <Box sx={{ display: 'flex', minHeight: '100vh' }}>
      {/* Sidebar */}
      <Drawer
        variant="temporary"
        open={mobileOpen}
        onClose={() => setMobileOpen(false)}
        sx={{
          display: { xs: 'block', md: 'none' },
          '& .MuiDrawer-paper': { width: DRAWER_WIDTH },
        }}
      >
        {drawer}
      </Drawer>
      <Drawer
        variant="permanent"
        sx={{
          display: { xs: 'none', md: 'block' },
          '& .MuiDrawer-paper': { width: DRAWER_WIDTH, boxSizing: 'border-box' },
        }}
        open
      >
        {drawer}
      </Drawer>

      {/* Main content area */}
      <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column', ml: { md: `${DRAWER_WIDTH}px` } }}>
        <AppBar
          position="sticky"
          elevation={0}
          sx={{
            bgcolor: 'background.default',
            borderBottom: 1,
            borderColor: 'divider',
          }}
        >
          <Toolbar sx={{ gap: 1 }}>
            <IconButton
              edge="start"
              sx={{ display: { md: 'none' } }}
              onClick={() => setMobileOpen(true)}
            >
              <MenuIcon />
            </IconButton>

            <Box
              sx={{
                display: 'flex',
                alignItems: 'center',
                bgcolor: 'background.paper',
                borderRadius: 1,
                border: 1,
                borderColor: 'divider',
                px: 1.5,
                py: 0.5,
                flex: 1,
                maxWidth: 480,
              }}
            >
              <SearchIcon sx={{ color: 'text.secondary', mr: 1, fontSize: 20 }} />
              <InputBase
                placeholder="搜索仓库、议题、合并请求..."
                sx={{ flex: 1, fontSize: 14, color: 'text.primary' }}
              />
              <Chip label="/" size="small" variant="outlined" sx={{ height: 22, fontSize: 11 }} />
            </Box>

            <Box sx={{ flex: 1 }} />

            <IconButton size="small" sx={{ color: 'text.secondary' }}>
              <AddIcon />
            </IconButton>
            <IconButton size="small" sx={{ color: 'text.secondary' }}>
              <NotificationsNoneIcon />
            </IconButton>
            <Avatar sx={{ width: 30, height: 30, bgcolor: 'primary.main', fontSize: 13 }}>
              A
            </Avatar>
          </Toolbar>
        </AppBar>

        <Box component="main" sx={{ flex: 1, p: 3 }}>
          <Outlet />
        </Box>
      </Box>
    </Box>
  );
}
