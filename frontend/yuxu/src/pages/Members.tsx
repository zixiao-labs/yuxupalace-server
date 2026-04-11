import Box from '@mui/material/Box';
import Paper from '@mui/material/Paper';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import Avatar from '@mui/material/Avatar';
import Chip from '@mui/material/Chip';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableContainer from '@mui/material/TableContainer';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';
import PersonAddIcon from '@mui/icons-material/PersonAdd';

const members = [
  { name: 'Amiya167', role: '所有者', email: 'amiya@zixiao.dev', avatar: 'A', joined: '2024-01-15', repos: 12, color: '#7c4dff' },
  { name: 'Chen', role: '维护者', email: 'chen@zixiao.dev', avatar: 'C', joined: '2024-03-22', repos: 8, color: '#00e5ff' },
  { name: 'Doctor', role: '开发者', email: 'doctor@zixiao.dev', avatar: 'D', joined: '2024-06-10', repos: 5, color: '#d29922' },
];

const roleColor = {
  '所有者': 'primary' as const,
  '维护者': 'secondary' as const,
  '开发者': 'default' as const,
};

export default function Members() {
  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Box>
          <Typography variant="h5">成员</Typography>
          <Typography variant="body2" color="text.secondary">管理组织成员和权限</Typography>
        </Box>
        <Button variant="contained" startIcon={<PersonAddIcon />} size="small">
          邀请成员
        </Button>
      </Box>

      <TableContainer component={Paper}>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell sx={{ fontWeight: 600, fontSize: 12 }}>成员</TableCell>
              <TableCell sx={{ fontWeight: 600, fontSize: 12 }}>角色</TableCell>
              <TableCell sx={{ fontWeight: 600, fontSize: 12 }}>仓库访问</TableCell>
              <TableCell sx={{ fontWeight: 600, fontSize: 12 }}>加入时间</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {members.map((m) => (
              <TableRow key={m.name} hover>
                <TableCell>
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5 }}>
                    <Avatar sx={{ width: 32, height: 32, fontSize: 14, bgcolor: m.color }}>{m.avatar}</Avatar>
                    <Box>
                      <Typography variant="body2" sx={{ fontWeight: 500 }}>{m.name}</Typography>
                      <Typography variant="caption" color="text.secondary">{m.email}</Typography>
                    </Box>
                  </Box>
                </TableCell>
                <TableCell>
                  <Chip
                    label={m.role}
                    size="small"
                    color={roleColor[m.role as keyof typeof roleColor]}
                    variant="outlined"
                    sx={{ height: 22, fontSize: 11 }}
                  />
                </TableCell>
                <TableCell>
                  <Typography variant="body2" sx={{ fontSize: 13 }}>{m.repos} 个仓库</Typography>
                </TableCell>
                <TableCell>
                  <Typography variant="caption" color="text.secondary">{m.joined}</Typography>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
    </Box>
  );
}
