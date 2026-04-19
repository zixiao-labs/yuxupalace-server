import { Card, Description, Input, Label, TextField } from '@heroui/react';
import { useSession } from '../../lib/session-store';

export default function SettingsIndex() {
  const { session } = useSession();
  const user = session?.user;
  if (!user) return null;

  return (
    <Card className="max-w-xl">
      <Card.Header>
        <Card.Title>个人资料</Card.Title>
        <Card.Description>只读视图，编辑功能正在开发中</Card.Description>
      </Card.Header>
      <Card.Content className="flex flex-col gap-4">
        <TextField isReadOnly>
          <Label>用户名</Label>
          <Input value={user.username} readOnly />
          <Description>登录使用</Description>
        </TextField>
        <TextField isReadOnly>
          <Label>邮箱</Label>
          <Input value={user.email} readOnly />
        </TextField>
        <TextField isReadOnly>
          <Label>显示名</Label>
          <Input value={user.display_name} readOnly />
        </TextField>
        <TextField isReadOnly>
          <Label>角色</Label>
          <Input value={user.is_admin ? '管理员' : '普通用户'} readOnly />
        </TextField>
      </Card.Content>
    </Card>
  );
}
