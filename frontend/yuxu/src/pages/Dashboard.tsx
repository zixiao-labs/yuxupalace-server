import Box from '@mui/material/Box';
import Grid from '@mui/material/Grid';
import Paper from '@mui/material/Paper';
import Typography from '@mui/material/Typography';
import Chip from '@mui/material/Chip';
import List from '@mui/material/List';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemIcon from '@mui/material/ListItemIcon';
import ListItemText from '@mui/material/ListItemText';
import Avatar from '@mui/material/Avatar';
import AvatarGroup from '@mui/material/AvatarGroup';
import LinearProgress from '@mui/material/LinearProgress';
import FolderIcon from '@mui/icons-material/Folder';
import BugReportIcon from '@mui/icons-material/BugReport';
import MergeIcon from '@mui/icons-material/MergeType';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import ErrorIcon from '@mui/icons-material/Error';
import ScheduleIcon from '@mui/icons-material/Schedule';
import { Link } from 'chen-the-dawnstreak';

interface StatCardProps {
  title: string;
  value: string;
  change: string;
  positive: boolean;
  icon: React.ReactNode;
}

function StatCard({ title, value, change, positive, icon }: StatCardProps) {
  return (
    <Paper sx={{ p: 2.5 }}>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', mb: 2 }}>
        <Typography variant="body2" color="text.secondary">{title}</Typography>
        <Box sx={{ color: 'text.secondary' }}>{icon}</Box>
      </Box>
      <Typography variant="h4" sx={{ mb: 0.5 }}>{value}</Typography>
      <Typography variant="caption" sx={{ color: positive ? 'success.main' : 'text.secondary' }}>
        {change}
      </Typography>
    </Paper>
  );
}

const recentRepos = [
  { name: 'logos', desc: '桌面代码编辑器', lang: 'TypeScript', langColor: '#3178c6', updated: '2小时前' },
  { name: 'chen-the-dawnstreak', desc: 'React 元框架', lang: 'TypeScript', langColor: '#3178c6', updated: '5小时前' },
  { name: 'aefanyl', desc: '协作协议桥接', lang: 'Rust', langColor: '#dea584', updated: '1天前' },
  { name: 'nasti', desc: '前端打包工具', lang: 'Rust', langColor: '#dea584', updated: '2天前' },
];

const recentActivity = [
  { user: 'A', action: '合并了', target: '!142 修复 WebSocket 重连逻辑', repo: 'logos', time: '30分钟前', type: 'merge' as const },
  { user: 'C', action: '创建了议题', target: '#89 CRDT 同步延迟优化', repo: 'aefanyl', time: '1小时前', type: 'issue' as const },
  { user: 'A', action: '推送到', target: 'main', repo: 'chen-the-dawnstreak', time: '2小时前', type: 'push' as const },
  { user: 'D', action: '评审通过', target: '!67 添加 OIDC 认证支持', repo: 'aefanyl', time: '3小时前', type: 'review' as const },
];

const pipelines = [
  { repo: 'logos', branch: 'main', status: 'success' as const, duration: '3m 42s', time: '25分钟前' },
  { repo: 'chen-the-dawnstreak', branch: 'feat/ssr', status: 'running' as const, duration: '1m 15s', time: '进行中' },
  { repo: 'aefanyl', branch: 'fix/reconnect', status: 'failed' as const, duration: '2m 08s', time: '1小时前' },
];

const statusIcon = {
  success: <CheckCircleIcon sx={{ color: 'success.main', fontSize: 18 }} />,
  running: <ScheduleIcon sx={{ color: 'info.main', fontSize: 18 }} />,
  failed: <ErrorIcon sx={{ color: 'error.main', fontSize: 18 }} />,
};

const statusLabel = {
  success: '成功',
  running: '运行中',
  failed: '失败',
};

export default function Dashboard() {
  return (
    <Box>
      <Box sx={{ mb: 3, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Box>
          <Typography variant="h5">工作台</Typography>
          <Typography variant="body2" color="text.secondary">Zixiao Labs DevOps 概览</Typography>
        </Box>
        <AvatarGroup max={4} sx={{ '& .MuiAvatar-root': { width: 28, height: 28, fontSize: 12 } }}>
          <Avatar sx={{ bgcolor: 'primary.main' }}>A</Avatar>
          <Avatar sx={{ bgcolor: 'secondary.main' }}>C</Avatar>
          <Avatar sx={{ bgcolor: 'warning.main' }}>D</Avatar>
        </AvatarGroup>
      </Box>

      <Grid container spacing={2.5} sx={{ mb: 3 }}>
        <Grid size={{ xs: 12, sm: 6, lg: 3 }}>
          <StatCard title="仓库" value="12" change="+2 本月" positive icon={<FolderIcon fontSize="small" />} />
        </Grid>
        <Grid size={{ xs: 12, sm: 6, lg: 3 }}>
          <StatCard title="开放议题" value="34" change="-8 本周" positive icon={<BugReportIcon fontSize="small" />} />
        </Grid>
        <Grid size={{ xs: 12, sm: 6, lg: 3 }}>
          <StatCard title="合并请求" value="7" change="3 待审核" positive={false} icon={<MergeIcon fontSize="small" />} />
        </Grid>
        <Grid size={{ xs: 12, sm: 6, lg: 3 }}>
          <StatCard title="流水线通过率" value="94%" change="+2.1%" positive icon={<CheckCircleIcon fontSize="small" />} />
        </Grid>
      </Grid>

      <Grid container spacing={2.5}>
        {/* Recent repos */}
        <Grid size={{ xs: 12, lg: 6 }}>
          <Paper sx={{ p: 0 }}>
            <Box sx={{ px: 2.5, py: 2, borderBottom: 1, borderColor: 'divider', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <Typography variant="subtitle1" sx={{ fontWeight: 600 }}>最近仓库</Typography>
              <Typography
                component={Link}
                to="/repos"
                variant="caption"
                sx={{ color: 'primary.main', textDecoration: 'none', '&:hover': { textDecoration: 'underline' } }}
              >
                查看全部
              </Typography>
            </Box>
            <List disablePadding>
              {recentRepos.map((repo) => (
                <ListItemButton
                  key={repo.name}
                  component={Link}
                  to={`/repos/zixiao-labs/${repo.name}`}
                  sx={{ mx: 0, borderRadius: 0, borderBottom: 1, borderColor: 'divider', '&:last-child': { borderBottom: 0 } }}
                >
                  <ListItemIcon sx={{ minWidth: 36 }}>
                    <FolderIcon fontSize="small" sx={{ color: 'text.secondary' }} />
                  </ListItemIcon>
                  <ListItemText
                    primary={repo.name}
                    secondary={repo.desc}
                    slotProps={{
                      primary: { sx: { fontSize: 14, fontWeight: 500 } },
                      secondary: { sx: { fontSize: 12 } },
                    }}
                  />
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5 }}>
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
                      <Box sx={{ width: 10, height: 10, borderRadius: '50%', bgcolor: repo.langColor }} />
                      <Typography variant="caption" color="text.secondary">{repo.lang}</Typography>
                    </Box>
                    <Typography variant="caption" color="text.secondary">{repo.updated}</Typography>
                  </Box>
                </ListItemButton>
              ))}
            </List>
          </Paper>
        </Grid>

        {/* Recent activity */}
        <Grid size={{ xs: 12, lg: 6 }}>
          <Paper sx={{ p: 0 }}>
            <Box sx={{ px: 2.5, py: 2, borderBottom: 1, borderColor: 'divider' }}>
              <Typography variant="subtitle1" sx={{ fontWeight: 600 }}>最近动态</Typography>
            </Box>
            <List disablePadding>
              {recentActivity.map((item, i) => (
                <ListItemButton key={i} sx={{ mx: 0, borderRadius: 0, borderBottom: 1, borderColor: 'divider', '&:last-child': { borderBottom: 0 } }}>
                  <ListItemIcon sx={{ minWidth: 36 }}>
                    <Avatar sx={{ width: 24, height: 24, fontSize: 11, bgcolor: 'primary.main' }}>
                      {item.user}
                    </Avatar>
                  </ListItemIcon>
                  <ListItemText
                    primary={
                      <Typography variant="body2" component="span">
                        <strong>{item.user}</strong> {item.action}{' '}
                        <Typography component="span" variant="body2" sx={{ color: 'primary.light' }}>
                          {item.target}
                        </Typography>
                      </Typography>
                    }
                    secondary={`${item.repo} · ${item.time}`}
                    slotProps={{ secondary: { sx: { fontSize: 12 } } }}
                  />
                </ListItemButton>
              ))}
            </List>
          </Paper>
        </Grid>

        {/* CI/CD pipelines */}
        <Grid size={12}>
          <Paper sx={{ p: 0 }}>
            <Box sx={{ px: 2.5, py: 2, borderBottom: 1, borderColor: 'divider', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <Typography variant="subtitle1" sx={{ fontWeight: 600 }}>流水线</Typography>
              <Typography
                component={Link}
                to="/ci"
                variant="caption"
                sx={{ color: 'primary.main', textDecoration: 'none', '&:hover': { textDecoration: 'underline' } }}
              >
                查看全部
              </Typography>
            </Box>
            {pipelines.map((p, i) => (
              <Box
                key={i}
                sx={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 2,
                  px: 2.5,
                  py: 1.5,
                  borderBottom: 1,
                  borderColor: 'divider',
                  '&:last-child': { borderBottom: 0 },
                }}
              >
                {statusIcon[p.status]}
                <Box sx={{ flex: 1 }}>
                  <Typography variant="body2" sx={{ fontWeight: 500 }}>{p.repo}</Typography>
                  <Typography variant="caption" color="text.secondary">{p.branch}</Typography>
                </Box>
                <Chip
                  label={statusLabel[p.status]}
                  size="small"
                  color={p.status === 'success' ? 'success' : p.status === 'failed' ? 'error' : 'info'}
                  variant="outlined"
                  sx={{ fontSize: 11, height: 22 }}
                />
                {p.status === 'running' && (
                  <Box sx={{ width: 80 }}>
                    <LinearProgress color="info" />
                  </Box>
                )}
                <Typography variant="caption" color="text.secondary" sx={{ minWidth: 60, textAlign: 'right' }}>
                  {p.duration}
                </Typography>
                <Typography variant="caption" color="text.secondary" sx={{ minWidth: 60, textAlign: 'right' }}>
                  {p.time}
                </Typography>
              </Box>
            ))}
          </Paper>
        </Grid>
      </Grid>
    </Box>
  );
}
