use bytes::Bytes;
use dashmap::DashMap;
use raidian::collab as pb;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::mpsc;

pub type ConnectionId = u32;

pub struct Connection {
    pub id: ConnectionId,
    pub tx: mpsc::UnboundedSender<Bytes>,
}

#[derive(Default)]
pub struct CollabHub {
    next_conn_id: AtomicU32,
    next_room_id: AtomicU32,
    next_project_id: AtomicU32,
    pub connections: DashMap<ConnectionId, Arc<Connection>>,
    pub conn_users: DashMap<ConnectionId, String>,
    pub user_to_conns: DashMap<String, Vec<ConnectionId>>,
    pub rooms: DashMap<u64, super::room::RoomState>,
    pub projects: DashMap<u64, super::project::ProjectState>,
}

impl CollabHub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alloc_connection_id(&self) -> ConnectionId {
        self.next_conn_id.fetch_add(1, Ordering::Relaxed) + 1
    }
    pub fn alloc_room_id(&self) -> u64 {
        self.next_room_id.fetch_add(1, Ordering::Relaxed) as u64 + 1
    }
    pub fn alloc_project_id(&self) -> u64 {
        self.next_project_id.fetch_add(1, Ordering::Relaxed) as u64 + 1
    }

    pub fn register(&self, conn: Arc<Connection>) {
        self.connections.insert(conn.id, conn);
    }

    pub fn set_user(&self, conn_id: ConnectionId, user_id: String) {
        self.user_to_conns
            .entry(user_id.clone())
            .or_default()
            .push(conn_id);
        self.conn_users.insert(conn_id, user_id);
    }

    pub fn user_of(&self, conn_id: ConnectionId) -> String {
        self.conn_users
            .get(&conn_id)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn deregister(&self, conn_id: ConnectionId) {
        self.connections.remove(&conn_id);
        if let Some((_, user)) = self.conn_users.remove(&conn_id)
            && let Some(mut v) = self.user_to_conns.get_mut(&user)
        {
            v.retain(|id| *id != conn_id);
        }
        self.rooms
            .iter_mut()
            .for_each(|mut r| r.participants.retain(|p| p.conn_id != conn_id));
        self.projects
            .iter_mut()
            .for_each(|mut p| p.collaborators.retain(|c| c.conn_id != conn_id));
    }

    pub fn send_to(&self, conn_id: ConnectionId, env: &pb::Envelope) {
        if let Some(conn) = self.connections.get(&conn_id) {
            let bytes = super::envelope::encode(env);
            let _ = conn.tx.send(bytes);
        }
    }

    pub fn broadcast(&self, conn_ids: &[ConnectionId], env: &pb::Envelope) {
        let bytes = super::envelope::encode(env);
        for id in conn_ids {
            if let Some(conn) = self.connections.get(id) {
                let _ = conn.tx.send(bytes.clone());
            }
        }
    }

    pub fn broadcast_to_project(
        &self,
        project_id: u64,
        env: &pb::Envelope,
        except: Option<ConnectionId>,
    ) {
        if let Some(proj) = self.projects.get(&project_id) {
            let targets: Vec<ConnectionId> = proj
                .all_conn_ids()
                .into_iter()
                .filter(|id| Some(*id) != except)
                .collect();
            self.broadcast(&targets, env);
        }
    }
}

/// Deterministically map an opaque user-id string (typically a UUID) into the
/// `uint64` slot expected by the Zed-style protobuf. A FNV-1a hash keeps the
/// mapping stable across processes and avoids the silent `0` collisions that
/// `str::parse::<u64>().unwrap_or(0)` produces.
pub fn user_id_to_u64(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}
