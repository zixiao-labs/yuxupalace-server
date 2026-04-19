import { useCallback, useEffect, useState } from 'react';
import { Link } from 'chen-the-dawnstreak';
import { Alert, Card, Chip, Skeleton } from '@heroui/react';
import { apiFetch, ApiError } from '../../lib/api';
import type { Repository } from '../../lib/types';
import RepoCreateModal from '../../components/RepoCreateModal';

export default function ReposIndex() {
  const [repos, setRepos] = useState<Repository[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => {
    setError(null);
    apiFetch<Repository[]>('/api/repos')
      .then(setRepos)
      .catch((err) => {
        setError(err instanceof ApiError ? err.message : '加载仓库列表失败');
        setRepos([]);
      });
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold" style={{ color: 'var(--foreground)' }}>
            我的仓库
          </h1>
          <p className="text-sm" style={{ color: 'var(--muted)' }}>
            创建和管理你拥有的 Git 仓库
          </p>
        </div>
        <RepoCreateModal onCreated={load} />
      </div>

      {error ? (
        <Alert status="danger">
          <Alert.Indicator />
          <Alert.Content>
            <Alert.Title>{error}</Alert.Title>
          </Alert.Content>
        </Alert>
      ) : null}

      {repos === null ? (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {[0, 1, 2].map((i) => (
            <Skeleton key={i} className="h-32 w-full rounded-2xl" />
          ))}
        </div>
      ) : repos.length === 0 ? (
        <Card variant="transparent">
          <Card.Header>
            <Card.Title>还没有仓库</Card.Title>
            <Card.Description>点击右上角「新建仓库」开始</Card.Description>
          </Card.Header>
        </Card>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {repos.map((repo) => (
            <Link
              key={repo.id}
              to={`/repos/${repo.full_name}`}
              style={{ textDecoration: 'none' }}
            >
              <Card>
                <Card.Header>
                  <Card.Title className="flex items-center gap-2">
                    <span>{repo.name}</span>
                    {repo.is_private ? (
                      <Chip size="sm" color="warning">
                        私有
                      </Chip>
                    ) : (
                      <Chip size="sm">公开</Chip>
                    )}
                  </Card.Title>
                  <Card.Description>{repo.full_name}</Card.Description>
                </Card.Header>
                <Card.Content>
                  <p
                    className="text-sm"
                    style={{ color: repo.description ? 'var(--foreground)' : 'var(--muted)' }}
                  >
                    {repo.description || '（无描述）'}
                  </p>
                </Card.Content>
                <Card.Footer className="text-xs" style={{ color: 'var(--muted)' }}>
                  默认分支：{repo.default_branch}
                </Card.Footer>
              </Card>
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}
