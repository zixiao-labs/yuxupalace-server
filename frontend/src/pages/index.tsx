import { Card } from '@heroui/react';
import ComingSoon from '../components/ComingSoon';
import { useSession } from '../lib/session-store';

export default function Dashboard() {
  const { session } = useSession();
  const user = session?.user;

  return (
    <div className="flex flex-col gap-6">
      <Card>
        <Card.Header>
          <Card.Title>仪表盘</Card.Title>
          <Card.Description>
            {user ? `你好，${user.display_name || user.username}` : ''}
          </Card.Description>
        </Card.Header>
        <Card.Content>
          <p style={{ color: 'var(--muted)' }}>
            快速浏览你的仓库、Issue、合并请求和 CI 状态。完整的仪表盘数据接口正在开发中。
          </p>
        </Card.Content>
      </Card>

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <ComingSoon title="待办 Issue" description="assigned_to_me 的 Issue 列表" />
        <ComingSoon title="待审 MR" description="需要我审批的合并请求" />
        <ComingSoon title="CI 状态" description="最近的流水线运行" />
      </div>
    </div>
  );
}
