use super::hub::ConnectionId;

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
}

impl ProjectState {
    pub fn alloc_replica(&mut self) -> u32 {
        self.next_replica_id += 1;
        self.next_replica_id
    }
    pub fn all_conn_ids(&self) -> Vec<ConnectionId> {
        self.collaborators.iter().map(|c| c.conn_id).collect()
    }
}
