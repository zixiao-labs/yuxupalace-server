import { useState } from 'react';
import { useParams, Link } from 'chen-the-dawnstreak';
import Box from '@mui/material/Box';
import Paper from '@mui/material/Paper';
import Typography from '@mui/material/Typography';
import Breadcrumbs from '@mui/material/Breadcrumbs';
import Tabs from '@mui/material/Tabs';
import Tab from '@mui/material/Tab';
import Button from '@mui/material/Button';
import Chip from '@mui/material/Chip';
import IconButton from '@mui/material/IconButton';
import Avatar from '@mui/material/Avatar';
import AvatarGroup from '@mui/material/AvatarGroup';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableContainer from '@mui/material/TableContainer';
import TableRow from '@mui/material/TableRow';
import CodeIcon from '@mui/icons-material/Code';
import BugReportIcon from '@mui/icons-material/BugReport';
import MergeIcon from '@mui/icons-material/MergeType';
import RocketLaunchIcon from '@mui/icons-material/RocketLaunch';
import SettingsIcon from '@mui/icons-material/Settings';
import StarBorderIcon from '@mui/icons-material/StarBorder';
import ForkRightIcon from '@mui/icons-material/ForkRight';
import ContentCopyIcon from '@mui/icons-material/ContentCopy';
import FolderIcon from '@mui/icons-material/Folder';
import InsertDriveFileIcon from '@mui/icons-material/InsertDriveFile';
import CommitIcon from '@mui/icons-material/Commit';

const fileTree = [
  { name: 'src', type: 'dir' as const, lastCommit: '重构路由模块', time: '2小时前' },
  { name: 'tests', type: 'dir' as const, lastCommit: '添加集成测试', time: '1天前' },
  { name: 'docs', type: 'dir' as const, lastCommit: '更新 API 文档', time: '3天前' },
  { name: '.gitignore', type: 'file' as const, lastCommit: '初始化项目', time: '2周前' },
  { name: 'Cargo.toml', type: 'file' as const, lastCommit: '升级依赖版本', time: '5天前' },
  { name: 'LICENSE', type: 'file' as const, lastCommit: '初始化项目', time: '2周前' },
  { name: 'README.md', type: 'file' as const, lastCommit: '更新 README', time: '1天前' },
];

const readmeContent = `# ${'{name}'}

开发者的一站式工作台，集成 Git 仓库，Issue，合并请求，CI/CD，基于 CRDT 的实时协作。

## 快速开始

\`\`\`bash
git clone https://git.zixiao.dev/zixiao-labs/{name}.git
cd {name}
cargo build
\`\`\`

## 技术栈

- **前端**: React + chen-the-dawnstreak + MUI + Nasti
- **后端**: Rust
- **协作**: CRDT (Lamport timestamps + vector clocks)

## 贡献

欢迎提交 Issue 和 Merge Request。

## 许可

MIT License`;

export default function RepositoryDetail() {
  const { owner, name } = useParams<{ owner: string; name: string }>();
  const [tab, setTab] = useState(0);

  return (
    <Box>
      {/* Breadcrumb */}
      <Breadcrumbs sx={{ mb: 2 }}>
        <Typography
          component={Link}
          to="/repos"
          variant="body2"
          sx={{ color: 'primary.light', textDecoration: 'none', '&:hover': { textDecoration: 'underline' } }}
        >
          仓库
        </Typography>
        <Typography
          component={Link}
          to={`/repos/${owner}/${name}`}
          variant="body2"
          sx={{ color: 'primary.light', textDecoration: 'none', '&:hover': { textDecoration: 'underline' } }}
        >
          {owner}
        </Typography>
        <Typography variant="body2" sx={{ color: 'text.primary', fontWeight: 600 }}>
          {name}
        </Typography>
      </Breadcrumbs>

      {/* Header */}
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', mb: 2, flexWrap: 'wrap', gap: 1 }}>
        <Box>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 0.5 }}>
            <Typography variant="h5">{name}</Typography>
            <Chip label="公开" size="small" variant="outlined" sx={{ height: 20, fontSize: 11 }} />
          </Box>
          <Typography variant="body2" color="text.secondary">
            跨编辑器协作协议桥接实现
          </Typography>
        </Box>
        <Box sx={{ display: 'flex', gap: 1 }}>
          <Button variant="outlined" size="small" startIcon={<StarBorderIcon />}>
            Star <Chip label="89" size="small" sx={{ ml: 0.5, height: 18, fontSize: 11 }} />
          </Button>
          <Button variant="outlined" size="small" startIcon={<ForkRightIcon />}>
            Fork <Chip label="5" size="small" sx={{ ml: 0.5, height: 18, fontSize: 11 }} />
          </Button>
        </Box>
      </Box>

      {/* Tabs */}
      <Tabs
        value={tab}
        onChange={(_, v) => setTab(v)}
        sx={{ mb: 2.5, borderBottom: 1, borderColor: 'divider', '& .MuiTab-root': { minHeight: 42, fontSize: 13 } }}
      >
        <Tab icon={<CodeIcon sx={{ fontSize: 16 }} />} iconPosition="start" label="代码" />
        <Tab icon={<BugReportIcon sx={{ fontSize: 16 }} />} iconPosition="start" label="议题 (12)" />
        <Tab icon={<MergeIcon sx={{ fontSize: 16 }} />} iconPosition="start" label="合并请求 (3)" />
        <Tab icon={<RocketLaunchIcon sx={{ fontSize: 16 }} />} iconPosition="start" label="CI/CD" />
        <Tab icon={<SettingsIcon sx={{ fontSize: 16 }} />} iconPosition="start" label="设置" />
      </Tabs>

      {tab === 0 && (
        <Box>
          {/* Branch selector + clone */}
          <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2, flexWrap: 'wrap', gap: 1 }}>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5 }}>
              <Chip label="main" variant="outlined" size="small" />
              <Typography variant="caption" color="text.secondary">4 个分支</Typography>
              <Typography variant="caption" color="text.secondary">12 个标签</Typography>
            </Box>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
              <Paper
                variant="outlined"
                sx={{
                  display: 'flex',
                  alignItems: 'center',
                  px: 1.5,
                  py: 0.5,
                  bgcolor: 'background.default',
                  gap: 1,
                }}
              >
                <Typography variant="caption" sx={{ fontFamily: 'monospace', color: 'text.secondary' }}>
                  https://git.zixiao.dev/{owner}/{name}.git
                </Typography>
                <IconButton size="small">
                  <ContentCopyIcon sx={{ fontSize: 14 }} />
                </IconButton>
              </Paper>
            </Box>
          </Box>

          {/* Last commit info */}
          <Paper variant="outlined" sx={{ display: 'flex', alignItems: 'center', gap: 1.5, px: 2, py: 1.25, mb: 0 , borderBottomLeftRadius: 0, borderBottomRightRadius: 0 }}>
            <Avatar sx={{ width: 22, height: 22, fontSize: 10, bgcolor: 'primary.main' }}>A</Avatar>
            <Typography variant="body2" sx={{ fontWeight: 500, fontSize: 13 }}>Amiya167</Typography>
            <Typography variant="body2" color="text.secondary" sx={{ fontSize: 13, flex: 1 }}>
              重构路由模块
            </Typography>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
              <CommitIcon sx={{ fontSize: 14, color: 'text.secondary' }} />
              <Typography variant="caption" sx={{ fontFamily: 'monospace', color: 'text.secondary' }}>a3f7c2d</Typography>
            </Box>
            <Typography variant="caption" color="text.secondary">2小时前</Typography>
            <AvatarGroup max={3} sx={{ '& .MuiAvatar-root': { width: 20, height: 20, fontSize: 9 } }}>
              <Avatar sx={{ bgcolor: 'primary.main' }}>A</Avatar>
              <Avatar sx={{ bgcolor: 'secondary.main' }}>C</Avatar>
            </AvatarGroup>
            <Typography variant="caption" color="text.secondary">128 commits</Typography>
          </Paper>

          {/* File tree */}
          <TableContainer component={Paper} variant="outlined" sx={{ borderTop: 0, borderTopLeftRadius: 0, borderTopRightRadius: 0 }}>
            <Table size="small">
              <TableBody>
                {fileTree.map((f) => (
                  <TableRow key={f.name} hover sx={{ cursor: 'pointer', '& td': { py: 1 } }}>
                    <TableCell sx={{ width: 36, pr: 0 }}>
                      {f.type === 'dir' ? (
                        <FolderIcon sx={{ fontSize: 18, color: 'info.main' }} />
                      ) : (
                        <InsertDriveFileIcon sx={{ fontSize: 18, color: 'text.secondary' }} />
                      )}
                    </TableCell>
                    <TableCell>
                      <Typography variant="body2" sx={{ fontSize: 13, fontWeight: f.type === 'dir' ? 500 : 400 }}>
                        {f.name}
                      </Typography>
                    </TableCell>
                    <TableCell>
                      <Typography variant="body2" color="text.secondary" sx={{ fontSize: 13 }}>
                        {f.lastCommit}
                      </Typography>
                    </TableCell>
                    <TableCell align="right">
                      <Typography variant="caption" color="text.secondary">
                        {f.time}
                      </Typography>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>

          {/* README */}
          <Paper variant="outlined" sx={{ mt: 2.5, overflow: 'hidden' }}>
            <Box sx={{ px: 2.5, py: 1.5, borderBottom: 1, borderColor: 'divider', display: 'flex', alignItems: 'center', gap: 1 }}>
              <InsertDriveFileIcon sx={{ fontSize: 16, color: 'text.secondary' }} />
              <Typography variant="body2" sx={{ fontWeight: 500, fontSize: 13 }}>README.md</Typography>
            </Box>
            <Box sx={{ p: 2.5 }}>
              <Typography
                component="pre"
                sx={{
                  fontFamily: 'monospace',
                  fontSize: 13,
                  lineHeight: 1.7,
                  whiteSpace: 'pre-wrap',
                  wordBreak: 'break-word',
                  color: 'text.secondary',
                  m: 0,
                }}
              >
                {readmeContent.replaceAll('{name}', name || 'repo')}
              </Typography>
            </Box>
          </Paper>
        </Box>
      )}

      {tab === 1 && (
        <Paper sx={{ p: 3, textAlign: 'center' }}>
          <BugReportIcon sx={{ fontSize: 48, color: 'text.secondary', mb: 1 }} />
          <Typography variant="h6" gutterBottom>议题</Typography>
          <Typography variant="body2" color="text.secondary">
            此仓库有 12 个开放议题，4 个已关闭
          </Typography>
        </Paper>
      )}

      {tab === 2 && (
        <Paper sx={{ p: 3, textAlign: 'center' }}>
          <MergeIcon sx={{ fontSize: 48, color: 'text.secondary', mb: 1 }} />
          <Typography variant="h6" gutterBottom>合并请求</Typography>
          <Typography variant="body2" color="text.secondary">
            3 个开放的合并请求等待审核
          </Typography>
        </Paper>
      )}

      {tab === 3 && (
        <Paper sx={{ p: 3, textAlign: 'center' }}>
          <RocketLaunchIcon sx={{ fontSize: 48, color: 'text.secondary', mb: 1 }} />
          <Typography variant="h6" gutterBottom>CI/CD 流水线</Typography>
          <Typography variant="body2" color="text.secondary">
            查看此仓库的持续集成与部署流水线
          </Typography>
        </Paper>
      )}

      {tab === 4 && (
        <Paper sx={{ p: 3, textAlign: 'center' }}>
          <SettingsIcon sx={{ fontSize: 48, color: 'text.secondary', mb: 1 }} />
          <Typography variant="h6" gutterBottom>仓库设置</Typography>
          <Typography variant="body2" color="text.secondary">
            管理仓库名称、可见性、Webhook、部署密钥等
          </Typography>
        </Paper>
      )}
    </Box>
  );
}
