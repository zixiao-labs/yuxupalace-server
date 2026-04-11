import Box from '@mui/material/Box';
import Paper from '@mui/material/Paper';
import Typography from '@mui/material/Typography';
import TextField from '@mui/material/TextField';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import Switch from '@mui/material/Switch';
import FormControlLabel from '@mui/material/FormControlLabel';

export default function Settings() {
  return (
    <Box>
      <Box sx={{ mb: 3 }}>
        <Typography variant="h5">设置</Typography>
        <Typography variant="body2" color="text.secondary">管理组织和平台设置</Typography>
      </Box>

      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3, maxWidth: 640 }}>
        {/* General */}
        <Paper sx={{ p: 3 }}>
          <Typography variant="subtitle1" sx={{ fontWeight: 600 }} gutterBottom>基本信息</Typography>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, mt: 2 }}>
            <TextField label="组织名称" defaultValue="Zixiao Labs" size="small" fullWidth />
            <TextField label="描述" defaultValue="紫霄实验室 - 开发者工具与基础设施" size="small" fullWidth multiline rows={2} />
            <TextField label="域名" defaultValue="zixiao.dev" size="small" fullWidth />
            <Button variant="contained" size="small" sx={{ alignSelf: 'flex-start' }}>保存</Button>
          </Box>
        </Paper>

        {/* Features */}
        <Paper sx={{ p: 3 }}>
          <Typography variant="subtitle1" sx={{ fontWeight: 600 }} gutterBottom>功能开关</Typography>
          <Box sx={{ mt: 1 }}>
            <FormControlLabel
              control={<Switch defaultChecked size="small" />}
              label={<Typography variant="body2">启用 CI/CD 流水线</Typography>}
            />
            <FormControlLabel
              control={<Switch defaultChecked size="small" />}
              label={<Typography variant="body2">启用合并请求审核</Typography>}
            />
            <FormControlLabel
              control={<Switch defaultChecked size="small" />}
              label={<Typography variant="body2">启用实时协作 (CRDT)</Typography>}
            />
            <FormControlLabel
              control={<Switch size="small" />}
              label={<Typography variant="body2">启用容器注册表</Typography>}
            />
          </Box>
        </Paper>

        {/* Danger zone */}
        <Paper sx={{ p: 3, borderColor: 'error.main', border: 1 }}>
          <Typography variant="subtitle1" sx={{ fontWeight: 600 }} color="error" gutterBottom>危险区域</Typography>
          <Divider sx={{ my: 1.5 }} />
          <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
            <Box>
              <Typography variant="body2" sx={{ fontWeight: 500 }}>转让组织</Typography>
              <Typography variant="caption" color="text.secondary">将此组织转让给其他用户</Typography>
            </Box>
            <Button variant="outlined" color="error" size="small">转让</Button>
          </Box>
          <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <Box>
              <Typography variant="body2" sx={{ fontWeight: 500 }}>删除组织</Typography>
              <Typography variant="caption" color="text.secondary">永久删除此组织及所有数据</Typography>
            </Box>
            <Button variant="outlined" color="error" size="small">删除</Button>
          </Box>
        </Paper>
      </Box>
    </Box>
  );
}
