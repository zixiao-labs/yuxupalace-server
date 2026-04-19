import { Card } from '@heroui/react';

interface Props {
  title?: string;
  description?: string;
}

export default function ComingSoon({
  title = '敬请期待',
  description = '该模块后端接口尚未实现，前端仅预留入口。',
}: Props) {
  return (
    <Card variant="transparent" className="w-full">
      <Card.Header>
        <Card.Title>{title}</Card.Title>
        <Card.Description>{description}</Card.Description>
      </Card.Header>
    </Card>
  );
}
