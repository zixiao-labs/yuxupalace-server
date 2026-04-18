use super::hub::ConnectionId;
use raidian::collab as pb;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProjectCollaborator {
    pub conn_id: ConnectionId,
    pub user_id: String,
    pub replica_id: u32,
    pub is_host: bool,
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct ProjectState {
    pub id: u64,
    pub room_id: u64,
    pub host_conn_id: ConnectionId,
    pub host_user_id: String,
    pub collaborators: Vec<ProjectCollaborator>,
    pub next_replica_id: u32,
    pub worktrees: Vec<pb::WorktreeMetadata>,
    pub is_ssh: bool,
}

impl ProjectState {
    pub fn alloc_replica(&mut self) -> u32 {
        self.next_replica_id += 1;
        self.next_replica_id
    }
    pub fn all_conn_ids(&self) -> Vec<ConnectionId> {
        // Host_conn_id is also recorded in `collaborators` (is_host = true),
        // so the direct iteration already covers everyone; but defend against
        // the list being out of sync by prepending and deduplicating.
        let mut out: Vec<ConnectionId> = std::iter::once(self.host_conn_id)
            .chain(self.collaborators.iter().map(|c| c.conn_id))
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    pub fn has_collaborator(&self, conn_id: ConnectionId) -> bool {
        self.host_conn_id == conn_id || self.collaborators.iter().any(|c| c.conn_id == conn_id)
    }
}
