import { Link } from 'chen-the-dawnstreak';
import { Button, Card } from '@heroui/react';

export default function NotFound() {
  return (
    <div
      className="flex min-h-screen items-center justify-center p-6"
      style={{ background: 'var(--background)' }}
    >
      <Card className="w-full max-w-md">
        <Card.Header>
          <Card.Title>页面不存在</Card.Title>
          <Card.Description>你访问的页面已下线或从未存在。</Card.Description>
        </Card.Header>
        <Card.Footer>
          <Link to="/">
            <Button>返回首页</Button>
          </Link>
        </Card.Footer>
      </Card>
    </div>
  );
}
