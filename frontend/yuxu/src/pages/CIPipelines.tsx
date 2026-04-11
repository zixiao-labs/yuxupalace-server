import Box from '@mui/material/Box';
import Paper from '@mui/material/Paper';
import Typography from '@mui/material/Typography';
import Chip from '@mui/material/Chip';
import LinearProgress from '@mui/material/LinearProgress';
import Avatar from '@mui/material/Avatar';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableContainer from '@mui/material/TableContainer';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import ErrorIcon from '@mui/icons-material/Error';
import ScheduleIcon from '@mui/icons-material/Schedule';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import ReplayIcon from '@mui/icons-material/Replay';
import IconButton from '@mui/material/IconButton';

interface Pipeline {
  id: number;
  repo: string;
  branch: string;
  commit: string;
  commitMsg: string;
  author: string;
  status: 'success' | 'running' | 'failed' | 'pending';
  stages: { name: string; status: 'success' | 'running' | 'failed' | 'pending' }[];
  duration: string;
  time: string;
}

const pipelines: Pipeline[] = [
  {
    id: 1024, repo: 'logos', branch: 'main', commit: 'a3f7c2d',
    commitMsg: '修复 WebSocket 重连逻辑', author: 'A',
    status: 'success',
    stages: [
      { name: '构建', status: 'success' },
      { name: '测试', status: 'success' },
      { name: '部署', status: 'success' },
    ],
    duration: '3m 42s', time: '25分钟前',
  },
  {
    id: 512, repo: 'chen-the-dawnstreak', branch: 'feat/ssr', commit: 'b8e1f3a',
    commitMsg: 'SSR 流式渲染支持', author: 'A',
    status: 'running',
    stages: [
      { name: '构建', status: 'success' },
      { name: '测试', status: 'running' },
      { name: '部署', status: 'pending' },
    ],
    duration: '1m 15s', time: '进行中',
  },
  {
    id: 256, repo: 'aefanyl', branch: 'fix/reconnect', commit: 'c4d2e9b',
    commitMsg: '优化断线重连处理', author: 'C',
    status: 'failed',
    stages: [
      { name: '构建', status: 'success' },
      { name: '测试', status: 'failed' },
      { name: '部署', status: 'pending' },
    ],
    duration: '2m 08s', time: '1小时前',
  },
  {
    id: 1023, repo: 'logos', branch: 'chore/upgrade-monaco', commit: 'f1a3b7c',
    commitMsg: '升级 Monaco Editor 到 v0.52', author: 'C',
    status: 'success',
    stages: [
      { name: '构建', status: 'success' },
      { name: '测试', status: 'success' },
      { name: '部署', status: 'success' },
    ],
    duration: '4m 11s', time: '3小时前',
  },
  {
    id: 255, repo: 'aefanyl', branch: 'feat/oidc', commit: 'e7f8a1d',
    commitMsg: '添加 OIDC 认证支持', author: 'D',
    status: 'success',
    stages: [
      { name: '构建', status: 'success' },
      { name: '测试', status: 'success' },
      { name: '部署', status: 'success' },
    ],
    duration: '2m 55s', time: '4小时前',
  },
  {
    id: 13, repo: 'nasti', branch: 'main', commit: 'd3c4e5f',
    commitMsg: '修复 tree-shaking 在 re-export 场景下的问题', author: 'A',
    status: 'success',
    stages: [
      { name: '构建', status: 'success' },
      { name: '测试', status: 'success' },
    ],
    duration: '1m 30s', time: '1天前',
  },
];

const statusConfig = {
  success: { icon: <CheckCircleIcon sx={{ fontSize: 18 }} />, color: 'success.main', label: '成功' },
  running: { icon: <ScheduleIcon sx={{ fontSize: 18 }} />, color: 'info.main', label: '运行中' },
  failed: { icon: <ErrorIcon sx={{ fontSize: 18 }} />, color: 'error.main', label: '失败' },
  pending: { icon: <ScheduleIcon sx={{ fontSize: 18 }} />, color: 'text.secondary', label: '等待' },
};

const stageColors = {
  success: '#3fb950',
  running: '#58a6ff',
  failed: '#f85149',
  pending: '#30363d',
};

export default function CIPipelines() {
  return (
    <Box>
      <Box sx={{ mb: 3 }}>
        <Typography variant="h5">CI/CD 流水线</Typography>
        <Typography variant="body2" color="text.secondary">所有仓库的持续集成与部署流水线</Typography>
      </Box>

      {/* Stats */}
      <Box sx={{ display: 'flex', gap: 2, mb: 3, flexWrap: 'wrap' }}>
        {(['success', 'running', 'failed'] as const).map((s) => {
          const count = pipelines.filter((p) => p.status === s).length;
          const cfg = statusConfig[s];
          return (
            <Paper key={s} sx={{ px: 2.5, py: 1.5, display: 'flex', alignItems: 'center', gap: 1.5 }}>
              <Box sx={{ color: cfg.color }}>{cfg.icon}</Box>
              <Box>
                <Typography variant="h6" sx={{ lineHeight: 1 }}>{count}</Typography>
                <Typography variant="caption" color="text.secondary">{cfg.label}</Typography>
              </Box>
            </Paper>
          );
        })}
      </Box>

      {/* Pipeline table */}
      <TableContainer component={Paper}>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell sx={{ fontWeight: 600, fontSize: 12 }}>状态</TableCell>
              <TableCell sx={{ fontWeight: 600, fontSize: 12 }}>流水线</TableCell>
              <TableCell sx={{ fontWeight: 600, fontSize: 12 }}>阶段</TableCell>
              <TableCell sx={{ fontWeight: 600, fontSize: 12 }}>耗时</TableCell>
              <TableCell sx={{ fontWeight: 600, fontSize: 12 }}>时间</TableCell>
              <TableCell sx={{ fontWeight: 600, fontSize: 12 }} align="right">操作</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {pipelines.map((p) => {
              const cfg = statusConfig[p.status];
              return (
                <TableRow key={p.id} hover>
                  <TableCell>
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                      <Box sx={{ color: cfg.color }}>{cfg.icon}</Box>
                      <Chip
                        label={cfg.label}
                        size="small"
                        sx={{ height: 20, fontSize: 11, color: cfg.color, borderColor: cfg.color }}
                        variant="outlined"
                      />
                    </Box>
                  </TableCell>
                  <TableCell>
                    <Box>
                      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 0.25 }}>
                        <Typography variant="body2" sx={{ fontWeight: 500, fontSize: 13 }}>
                          #{p.id}
                        </Typography>
                        <Chip label={p.branch} size="small" variant="outlined" sx={{ height: 18, fontSize: 10, fontFamily: 'monospace' }} />
                      </Box>
                      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                        <Avatar sx={{ width: 18, height: 18, fontSize: 9, bgcolor: 'primary.main' }}>{p.author}</Avatar>
                        <Typography variant="caption" color="text.secondary">
                          {p.repo} · {p.commitMsg}
                        </Typography>
                        <Typography variant="caption" sx={{ fontFamily: 'monospace', color: 'text.secondary' }}>
                          {p.commit}
                        </Typography>
                      </Box>
                    </Box>
                  </TableCell>
                  <TableCell>
                    <Box sx={{ display: 'flex', gap: 0.5, alignItems: 'center' }}>
                      {p.stages.map((stage, i) => (
                        <Box key={i} sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
                          <Box
                            sx={{
                              width: 48,
                              height: 6,
                              borderRadius: 1,
                              bgcolor: stageColors[stage.status],
                              position: 'relative',
                              overflow: 'hidden',
                            }}
                          >
                            {stage.status === 'running' && (
                              <LinearProgress
                                sx={{
                                  position: 'absolute',
                                  top: 0,
                                  left: 0,
                                  right: 0,
                                  bottom: 0,
                                  height: '100%',
                                  bgcolor: 'transparent',
                                }}
                                color="info"
                              />
                            )}
                          </Box>
                          {i < p.stages.length - 1 && (
                            <Box sx={{ width: 4, height: 1, bgcolor: 'divider' }} />
                          )}
                        </Box>
                      ))}
                    </Box>
                    <Typography variant="caption" color="text.secondary" sx={{ fontSize: 10 }}>
                      {p.stages.map((s) => s.name).join(' → ')}
                    </Typography>
                  </TableCell>
                  <TableCell>
                    <Typography variant="body2" sx={{ fontSize: 13 }}>{p.duration}</Typography>
                  </TableCell>
                  <TableCell>
                    <Typography variant="caption" color="text.secondary">{p.time}</Typography>
                  </TableCell>
                  <TableCell align="right">
                    {p.status === 'failed' && (
                      <IconButton size="small" title="重试">
                        <ReplayIcon sx={{ fontSize: 16 }} />
                      </IconButton>
                    )}
                    {p.status === 'pending' && (
                      <IconButton size="small" title="运行">
                        <PlayArrowIcon sx={{ fontSize: 16 }} />
                      </IconButton>
                    )}
                  </TableCell>
                </TableRow>
              );
            })}
          </TableBody>
        </Table>
      </TableContainer>
    </Box>
  );
}
