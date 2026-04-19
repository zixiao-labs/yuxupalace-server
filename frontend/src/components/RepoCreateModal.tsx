import { useState, type FormEvent } from 'react';
import {
  Alert,
  Button,
  Description,
  FieldError,
  Input,
  Label,
  Modal,
  Switch,
  TextArea,
  TextField,
} from '@heroui/react';
import { apiFetch, ApiError } from '../lib/api';
import type { CreateRepositoryRequest, Repository } from '../lib/types';

interface Props {
  onCreated: (repo: Repository) => void;
}

export default function RepoCreateModal({ onCreated }: Props) {
  const [isOpen, setIsOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [isPrivate, setIsPrivate] = useState(false);

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    // Guard against rapid double-submits: `isPending` on the button is
    // visual only, nothing prevents a second onSubmit from firing before
    // the first POST resolves.
    if (loading) return;
    setError(null);
    const form = e.currentTarget;
    const fd = new FormData(form);
    const payload: CreateRepositoryRequest = {
      name: String(fd.get('name') ?? '').trim(),
      description: String(fd.get('description') ?? '').trim(),
      is_private: isPrivate,
    };
    if (!payload.name) {
      setError('请输入仓库名');
      return;
    }
    setLoading(true);
    try {
      const created = await apiFetch<Repository>('/api/repos', {
        method: 'POST',
        body: JSON.stringify(payload),
      });
      onCreated(created);
      setIsOpen(false);
      setIsPrivate(false);
      form.reset();
    } catch (err) {
      setError(err instanceof ApiError ? err.message : '创建仓库失败');
    } finally {
      setLoading(false);
    }
  }

  return (
    <Modal
      isOpen={isOpen}
      onOpenChange={(next) => {
        setIsOpen(next);
        // Clear any stale error whenever the modal opens or closes so a
        // subsequent open shows a fresh form instead of last attempt's Alert.
        setError(null);
      }}
    >
      <Button onPress={() => setIsOpen(true)}>新建仓库</Button>
      <Modal.Backdrop>
        <Modal.Container>
          <Modal.Dialog className="sm:max-w-[480px]">
            <Modal.CloseTrigger />
            <Modal.Header>
              <Modal.Heading>新建仓库</Modal.Heading>
            </Modal.Header>
            <Modal.Body>
              <form
                id="create-repo-form"
                className="flex flex-col gap-4"
                onSubmit={handleSubmit}
              >
                {error ? (
                  <Alert status="danger">
                    <Alert.Indicator />
                    <Alert.Content>
                      <Alert.Title>{error}</Alert.Title>
                    </Alert.Content>
                  </Alert>
                ) : null}
                <TextField name="name" isRequired>
                  <Label>仓库名</Label>
                  <Input placeholder="my-repo" />
                  <Description>将作为 URL 的一部分</Description>
                  <FieldError />
                </TextField>
                <TextField name="description">
                  <Label>描述（可选）</Label>
                  <TextArea placeholder="一句话介绍这个仓库" rows={3} />
                  <FieldError />
                </TextField>
                <Switch isSelected={isPrivate} onChange={setIsPrivate}>
                  <Switch.Control>
                    <Switch.Thumb />
                  </Switch.Control>
                  <Switch.Content>
                    <Label className="text-sm">私有仓库</Label>
                    <Description>仅对仓库成员可见</Description>
                  </Switch.Content>
                </Switch>
              </form>
            </Modal.Body>
            <Modal.Footer>
              <Button
                type="submit"
                form="create-repo-form"
                fullWidth
                isPending={loading}
              >
                {loading ? '创建中…' : '创建'}
              </Button>
            </Modal.Footer>
          </Modal.Dialog>
        </Modal.Container>
      </Modal.Backdrop>
    </Modal>
  );
}
