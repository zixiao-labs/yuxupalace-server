import { useState } from 'react';
import Box from '@mui/material/Box';
import Paper from '@mui/material/Paper';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';
import InputAdornment from '@mui/material/InputAdornment';
import Tabs from '@mui/material/Tabs';
import Tab from '@mui/material/Tab';
import Chip from '@mui/material/Chip';
import Avatar from '@mui/material/Avatar';
import List from '@mui/material/List';
import ListItemButton from '@mui/material/ListItemButton';
import SearchIcon from '@mui/icons-material/Search';
import AddIcon from '@mui/icons-material/Add';
import MergeIcon from '@mui/icons-material/MergeType';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import ChatBubbleOutlineIcon from '@mui/icons-material/ChatBubbleOutlined';
import ThumbUpAltOutlinedIcon from '@mui/icons-material/ThumbUpAltOutlined';

interface MR {
  id: number;
  title: string;
  repo: string;
  sourceBranch: string;
  targetBranch: string;
  author: string;
  reviewers: string[];
  approvals: number;
  comments: number;
  created: string;
  status: 'open' | 'merged' | 'closed';
  ciStatus: 'success' | 'running' | 'failed';
  labels: { name: string; color: string }[];
}

const mergeRequests: MR[] = [
  {
    id: 142, title: '修复 WebSocket 重连逻辑', repo: 'logos',
    sourceBranch: 'fix/ws-reconnect', targetBranch: 'main',
    author: 'A', reviewers: ['C', 'D'], approvals: 2, comments: 7,
    created: '30分钟前', status: 'merged', ciStatus: 'success',
    labels: [{ name: 'bug', color: '#f85149' }],
  },
  {
    id: 67, title: '添加 OIDC 认证支持', repo: 'aefanyl',
    sourceBranch: 'feat/oidc', targetBranch: 'main',
    author: 'D', reviewers: ['A'], approvals: 1, comments: 14,
    created: '3小时前', status: 'open', ciStatus: 'success',
    labels: [{ name: '功能', color: '#3fb950' }, { name: '协作', color: '#7c4dff' }],
  },
  {
    id: 46, title: 'SSR 流式渲染支持', repo: 'chen-the-dawnstreak',
    sourceBranch: 'feat/streaming-ssr', targetBranch: 'main',
    author: 'A', reviewers: ['C'], approvals: 0, comments: 3,
    created: '1天前', status: 'open', ciStatus: 'running',
    labels: [{ name: '功能', color: '#3fb950' }],
  },
  {
    id: 66, title: '优化 Protobuf 消息序列化性能', repo: 'aefanyl',
    sourceBranch: 'perf/protobuf', targetBranch: 'main',
    author: 'C', reviewers: ['A', 'D'], approvals: 1, comments: 5,
    created: '2天前', status: 'open', ciStatus: 'failed',
    labels: [{ name: '性能', color: '#d29922' }],
  },
  {
    id: 141, title: '升级 Monaco Editor 到 v0.52', repo: 'logos',
    sourceBranch: 'chore/upgrade-monaco', targetBranch: 'main',
    author: 'C', reviewers: ['A'], approvals: 1, comments: 2,
    created: '3天前', status: 'merged', ciStatus: 'success',
    labels: [{ name: '依赖', color: '#8b949e' }],
  },
];

const ciStatusColor = {
  success: 'success' as const,
  running: 'info' as const,
  failed: 'error' as const,
};

const ciStatusLabel = {
  success: '通过',
  running: '运行中',
  failed: '失败',
};

const statusIcon = {
  open: <MergeIcon sx={{ fontSize: 16, color: 'success.main' }} />,
  merged: <MergeIcon sx={{ fontSize: 16, color: 'secondary.main' }} />,
  closed: <MergeIcon sx={{ fontSize: 16, color: 'error.main' }} />,
};

export default function MergeRequests() {
  const [search, setSearch] = useState('');
  const [tab, setTab] = useState(0);

  const openCount = mergeRequests.filter((m) => m.status === 'open').length;
  const mergedCount = mergeRequests.filter((m) => m.status === 'merged').length;

  const filtered = mergeRequests.filter((m) => {
    const matchSearch = m.title.toLowerCase().includes(search.toLowerCase()) ||
      m.repo.toLowerCase().includes(search.toLowerCase());
    if (tab === 0) return matchSearch && m.status === 'open';
    if (tab === 1) return matchSearch && m.status === 'merged';
    return matchSearch;
  });

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Typography variant="h5">合并请求</Typography>
        <Button variant="contained" startIcon={<AddIcon />} size="small">
          新建合并请求
        </Button>
      </Box>

      <Box sx={{ display: 'flex', gap: 2, mb: 2, flexWrap: 'wrap', alignItems: 'center' }}>
        <TextField
          size="small"
          placeholder="搜索合并请求..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          sx={{ minWidth: 280 }}
          slotProps={{
            input: {
              startAdornment: (
                <InputAdornment position="start">
                  <SearchIcon fontSize="small" sx={{ color: 'text.secondary' }} />
                </InputAdornment>
              ),
            },
          }}
        />
        <Tabs
          value={tab}
          onChange={(_, v) => setTab(v)}
          sx={{ minHeight: 36, '& .MuiTab-root': { minHeight: 36, py: 0, fontSize: 13 } }}
        >
          <Tab
            icon={<MergeIcon sx={{ fontSize: 14, color: 'success.main' }} />}
            iconPosition="start"
            label={`开放 (${openCount})`}
          />
          <Tab
            icon={<CheckCircleIcon sx={{ fontSize: 14, color: 'secondary.main' }} />}
            iconPosition="start"
            label={`已合并 (${mergedCount})`}
          />
        </Tabs>
      </Box>

      <Paper sx={{ p: 0 }}>
        <List disablePadding>
          {filtered.map((mr) => (
            <ListItemButton
              key={`${mr.repo}-${mr.id}`}
              sx={{
                borderRadius: 0,
                borderBottom: 1,
                borderColor: 'divider',
                '&:last-child': { borderBottom: 0 },
                py: 1.5,
                px: 2.5,
                display: 'flex',
                alignItems: 'flex-start',
                gap: 1.5,
              }}
            >
              {statusIcon[mr.status]}

              <Box sx={{ flex: 1, minWidth: 0 }}>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, flexWrap: 'wrap', mb: 0.5 }}>
                  <Typography variant="body2" sx={{ fontWeight: 600, '&:hover': { color: 'primary.light' } }}>
                    {mr.title}
                  </Typography>
                  {mr.labels.map((l) => (
                    <Chip
                      key={l.name}
                      label={l.name}
                      size="small"
                      sx={{
                        height: 20,
                        fontSize: 11,
                        bgcolor: `${l.color}20`,
                        color: l.color,
                        borderColor: `${l.color}40`,
                        border: 1,
                      }}
                    />
                  ))}
                  <Chip
                    label={ciStatusLabel[mr.ciStatus]}
                    size="small"
                    color={ciStatusColor[mr.ciStatus]}
                    variant="outlined"
                    sx={{ height: 20, fontSize: 11 }}
                  />
                </Box>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <Typography variant="caption" color="text.secondary">
                    {mr.repo}!{mr.id} · {mr.author} 创建于 {mr.created}
                  </Typography>
                  <Chip
                    label={`${mr.sourceBranch} → ${mr.targetBranch}`}
                    size="small"
                    variant="outlined"
                    sx={{ height: 18, fontSize: 10, fontFamily: 'monospace' }}
                  />
                </Box>
              </Box>

              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5 }}>
                {mr.approvals > 0 && (
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.3, color: 'success.main' }}>
                    <ThumbUpAltOutlinedIcon sx={{ fontSize: 14 }} />
                    <Typography variant="caption">{mr.approvals}</Typography>
                  </Box>
                )}
                {mr.comments > 0 && (
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.3, color: 'text.secondary' }}>
                    <ChatBubbleOutlineIcon sx={{ fontSize: 14 }} />
                    <Typography variant="caption">{mr.comments}</Typography>
                  </Box>
                )}
                <Box sx={{ display: 'flex', gap: -0.5 }}>
                  {mr.reviewers.map((r) => (
                    <Avatar key={r} sx={{ width: 22, height: 22, fontSize: 10, bgcolor: 'primary.main', border: '2px solid', borderColor: 'background.paper' }}>
                      {r}
                    </Avatar>
                  ))}
                </Box>
              </Box>
            </ListItemButton>
          ))}
        </List>
      </Paper>
    </Box>
  );
}
