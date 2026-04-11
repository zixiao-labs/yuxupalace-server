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
import List from '@mui/material/List';
import ListItemButton from '@mui/material/ListItemButton';
import SearchIcon from '@mui/icons-material/Search';
import AddIcon from '@mui/icons-material/Add';
import CircleIcon from '@mui/icons-material/Circle';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import ChatBubbleOutlineIcon from '@mui/icons-material/ChatBubbleOutlined';

interface Issue {
  id: number;
  title: string;
  repo: string;
  labels: { name: string; color: string }[];
  author: string;
  assignee: string | null;
  comments: number;
  created: string;
  open: boolean;
}

const issues: Issue[] = [
  {
    id: 89, title: 'CRDT 同步延迟优化', repo: 'aefanyl',
    labels: [{ name: '性能', color: '#d29922' }, { name: '优先', color: '#f85149' }],
    author: 'C', assignee: 'A', comments: 5, created: '1小时前', open: true,
  },
  {
    id: 234, title: 'Monaco Editor 中文输入法候选词位置偏移', repo: 'logos',
    labels: [{ name: 'bug', color: '#f85149' }],
    author: 'D', assignee: 'C', comments: 3, created: '3小时前', open: true,
  },
  {
    id: 88, title: 'WebSocket 断线重连后状态丢失', repo: 'aefanyl',
    labels: [{ name: 'bug', color: '#f85149' }, { name: '协作', color: '#7c4dff' }],
    author: 'A', assignee: 'A', comments: 8, created: '5小时前', open: true,
  },
  {
    id: 45, title: '支持文件路由的 catch-all 模式', repo: 'chen-the-dawnstreak',
    labels: [{ name: '功能', color: '#3fb950' }],
    author: 'A', assignee: null, comments: 2, created: '1天前', open: true,
  },
  {
    id: 12, title: 'Tree-shaking 在 re-export 场景下失效', repo: 'nasti',
    labels: [{ name: 'bug', color: '#f85149' }],
    author: 'C', assignee: 'A', comments: 6, created: '2天前', open: true,
  },
  {
    id: 233, title: 'SSH 远程连接超时处理', repo: 'logos',
    labels: [{ name: '功能', color: '#3fb950' }],
    author: 'A', assignee: null, comments: 1, created: '3天前', open: true,
  },
  {
    id: 87, title: '添加 OIDC 认证支持', repo: 'aefanyl',
    labels: [{ name: '功能', color: '#3fb950' }, { name: '协作', color: '#7c4dff' }],
    author: 'D', assignee: 'D', comments: 12, created: '1周前', open: false,
  },
  {
    id: 44, title: 'SSR hydration mismatch 问题', repo: 'chen-the-dawnstreak',
    labels: [{ name: 'bug', color: '#f85149' }],
    author: 'C', assignee: 'A', comments: 4, created: '1周前', open: false,
  },
];

export default function Issues() {
  const [search, setSearch] = useState('');
  const [tab, setTab] = useState(0);

  const openCount = issues.filter((i) => i.open).length;
  const closedCount = issues.filter((i) => !i.open).length;

  const filtered = issues.filter((i) => {
    const matchSearch = i.title.toLowerCase().includes(search.toLowerCase()) ||
      i.repo.toLowerCase().includes(search.toLowerCase());
    if (tab === 0) return matchSearch && i.open;
    if (tab === 1) return matchSearch && !i.open;
    return matchSearch;
  });

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Typography variant="h5">议题</Typography>
        <Button variant="contained" startIcon={<AddIcon />} size="small">
          新建议题
        </Button>
      </Box>

      <Box sx={{ display: 'flex', gap: 2, mb: 2, flexWrap: 'wrap', alignItems: 'center' }}>
        <TextField
          size="small"
          placeholder="搜索议题..."
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
            icon={<CircleIcon sx={{ fontSize: 10, color: 'success.main' }} />}
            iconPosition="start"
            label={`开放 (${openCount})`}
          />
          <Tab
            icon={<CheckCircleIcon sx={{ fontSize: 14, color: 'text.secondary' }} />}
            iconPosition="start"
            label={`已关闭 (${closedCount})`}
          />
        </Tabs>
      </Box>

      <Paper sx={{ p: 0 }}>
        <List disablePadding>
          {filtered.map((issue) => (
            <ListItemButton
              key={`${issue.repo}-${issue.id}`}
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
              {issue.open ? (
                <CircleIcon sx={{ fontSize: 14, color: 'success.main', mt: 0.5 }} />
              ) : (
                <CheckCircleIcon sx={{ fontSize: 16, color: 'secondary.main', mt: 0.3 }} />
              )}

              <Box sx={{ flex: 1, minWidth: 0 }}>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, flexWrap: 'wrap' }}>
                  <Typography variant="body2" sx={{ fontWeight: 600, '&:hover': { color: 'primary.light' } }}>
                    {issue.title}
                  </Typography>
                  {issue.labels.map((l) => (
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
                </Box>
                <Typography variant="caption" color="text.secondary">
                  {issue.repo}#{issue.id} · {issue.author} 创建于 {issue.created}
                  {issue.assignee && ` · 指派给 ${issue.assignee}`}
                </Typography>
              </Box>

              {issue.comments > 0 && (
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5, color: 'text.secondary' }}>
                  <ChatBubbleOutlineIcon sx={{ fontSize: 14 }} />
                  <Typography variant="caption">{issue.comments}</Typography>
                </Box>
              )}
            </ListItemButton>
          ))}
        </List>
      </Paper>
    </Box>
  );
}
