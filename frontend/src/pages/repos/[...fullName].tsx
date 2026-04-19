import { useEffect, useState } from 'react';
import { Link, useParams } from 'chen-the-dawnstreak';
import { Alert, Breadcrumbs, Card, Chip, Skeleton, Tabs } from '@heroui/react';
import { apiFetch, ApiError } from '../../lib/api';
import ComingSoon from '../../components/ComingSoon';
import type { Repository } from '../../lib/types';

export default function RepoDetail() {
  const params = useParams();
  // From Chen file routing [...fullName].tsx, react-router exposes the rest
  // parameter as `*`. It arrives already URL-decoded (contains a literal `/`).
  const fullName = (params['*'] || params.fullName || '') as string;

  const [repo, setRepo] = useState<Repository | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!fullName) return;
    setError(null);
    apiFetch<Repository>(`/api/repos/${fullName}`)
      .then(setRepo)
      .catch((err) => {
        setError(err instanceof ApiError ? err.message : '加载仓库失败');
      });
  }, [fullName]);

  if (!fullName) {
    return (
      <Alert status="danger">
        <Alert.Indicator />
        <Alert.Content>
          <Alert.Title>路径缺少仓库名</Alert.Title>
        </Alert.Content>
      </Alert>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col gap-4">
        <Breadcrumbs>
          <Link to="/repos">仓库</Link>
          <span>{fullName}</span>
        </Breadcrumbs>
        <Alert status="danger">
          <Alert.Indicator />
          <Alert.Content>
            <Alert.Title>{error}</Alert.Title>
          </Alert.Content>
        </Alert>
      </div>
    );
  }

  if (!repo) {
    return (
      <div className="flex flex-col gap-4">
        <Skeleton className="h-6 w-48 rounded-md" />
        <Skeleton className="h-24 w-full rounded-2xl" />
        <Skeleton className="h-64 w-full rounded-2xl" />
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6">
      <Breadcrumbs>
        <Link to="/repos">仓库</Link>
        <span>{repo.full_name}</span>
      </Breadcrumbs>

      <Card>
        <Card.Header>
          <Card.Title className="flex items-center gap-2">
            <span>{repo.full_name}</span>
            {repo.is_private ? (
              <Chip size="sm" color="warning">
                私有
              </Chip>
            ) : (
              <Chip size="sm">公开</Chip>
            )}
          </Card.Title>
          <Card.Description>
            {repo.description || '（无描述）'} · 默认分支 {repo.default_branch}
          </Card.Description>
        </Card.Header>
      </Card>

      <Tabs defaultSelectedKey="overview">
        <Tabs.ListContainer>
          <Tabs.List aria-label="仓库视图">
            <Tabs.Tab id="overview">
              概览
              <Tabs.Indicator />
            </Tabs.Tab>
            <Tabs.Tab id="issues">
              Issues
              <Tabs.Indicator />
            </Tabs.Tab>
            <Tabs.Tab id="mr">
              合并请求
              <Tabs.Indicator />
            </Tabs.Tab>
            <Tabs.Tab id="pipelines">
              流水线
              <Tabs.Indicator />
            </Tabs.Tab>
            <Tabs.Tab id="members">
              成员
              <Tabs.Indicator />
            </Tabs.Tab>
          </Tabs.List>
        </Tabs.ListContainer>

        <Tabs.Panel id="overview" className="pt-4">
          <Card variant="transparent">
            <Card.Header>
              <Card.Title>克隆</Card.Title>
            </Card.Header>
            <Card.Content>
              <code
                className="block rounded-xl px-4 py-3 text-sm"
                style={{
                  background: 'var(--surface-secondary)',
                  color: 'var(--surface-secondary-foreground)',
                }}
              >
                yuxu repo clone {repo.full_name}
              </code>
            </Card.Content>
          </Card>
        </Tabs.Panel>

        <Tabs.Panel id="issues" className="pt-4">
          <ComingSoon title="Issues" description="Issue 列表、评论、标签、关闭状态追踪。" />
        </Tabs.Panel>

        <Tabs.Panel id="mr" className="pt-4">
          <ComingSoon
            title="合并请求"
            description="MR 生命周期、审核、三种合并策略（merge / squash / rebase）。"
          />
        </Tabs.Panel>

        <Tabs.Panel id="pipelines" className="pt-4">
          <ComingSoon title="流水线" description="CI/CD 触发、阶段日志、状态追踪。" />
        </Tabs.Panel>

        <Tabs.Panel id="members" className="pt-4">
          <ComingSoon
            title="成员"
            description="ACL 五级角色（owner / maintainer / developer / reporter / guest）。"
          />
        </Tabs.Panel>
      </Tabs>
    </div>
  );
}
