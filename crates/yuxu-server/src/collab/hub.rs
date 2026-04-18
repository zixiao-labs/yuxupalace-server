use bytes::Bytes;
use dashmap::DashMap;
use raidian::collab as pb;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::mpsc;

pub type ConnectionId = u32;

/// Per-connection outbound queue depth. A slow client that can't drain frames
/// will have messages dropped and the connection torn down rather than growing
/// memory without bound.
pub const OUTBOUND_CHANNEL_CAPACITY: usize = 128;

pub struct Connection {
    pub id: ConnectionId,
    pub tx: mpsc::Sender<Bytes>,
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

/// Side effects produced by `deregister` that the caller should broadcast.
#[derive(Default)]
pub struct DisconnectEffects {
    pub removed_project_ids: Vec<u64>,
    pub remaining_guest_conns: Vec<ConnectionId>,
    pub affected_rooms: Vec<u64>,
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

    /// Remove the connection and every piece of shared state that referenced
    /// it. Returns a summary so the caller can notify remaining peers.
    ///
    /// Projects whose host disappeared are unshared entirely — the server
    /// cannot keep serving LSP/Git/file requests for them. Rooms with a
    /// vanished host stay alive with the remaining participants; a future
    /// improvement could promote a new host.
    pub fn deregister(&self, conn_id: ConnectionId) -> DisconnectEffects {
        let mut effects = DisconnectEffects::default();

        self.connections.remove(&conn_id);
        if let Some((_, user)) = self.conn_users.remove(&conn_id)
            && let Some(mut v) = self.user_to_conns.get_mut(&user)
        {
            v.retain(|id| *id != conn_id);
        }

        // Remove from every room's participant list; record which rooms were touched.
        for mut room in self.rooms.iter_mut() {
            let before = room.participants.len();
            room.participants.retain(|p| p.conn_id != conn_id);
            if room.participants.len() != before {
                effects.affected_rooms.push(*room.key());
            }
        }

        // Gather every project hosted by or joined by this connection.
        let mut projects_to_drop: Vec<u64> = Vec::new();
        for entry in self.projects.iter() {
            if entry.value().host_conn_id == conn_id {
                projects_to_drop.push(*entry.key());
            }
        }
        // Drop hosted projects entirely and collect the guest ids to notify.
        for pid in &projects_to_drop {
            if let Some((_, proj)) = self.projects.remove(pid) {
                if let Some(mut room) = self.rooms.get_mut(&proj.room_id) {
                    room.shared_project_ids.retain(|id| id != pid);
                    if !effects.affected_rooms.contains(&proj.room_id) {
                        effects.affected_rooms.push(proj.room_id);
                    }
                }
                for c in &proj.collaborators {
                    if c.conn_id != conn_id && !effects.remaining_guest_conns.contains(&c.conn_id) {
                        effects.remaining_guest_conns.push(c.conn_id);
                    }
                }
                effects.removed_project_ids.push(*pid);
            }
        }
        // For projects we *didn't* host, just remove the departing collaborator.
        for mut proj in self.projects.iter_mut() {
            proj.collaborators.retain(|c| c.conn_id != conn_id);
        }

        effects
    }

    /// Enqueue an envelope to a single connection. If the per-connection queue
    /// is full (slow/stalled peer) the connection is torn down rather than
    /// letting the outbound backlog grow without bound.
    pub fn send_to(&self, conn_id: ConnectionId, env: &pb::Envelope) {
        let Some(conn) = self.connections.get(&conn_id).map(|c| c.clone()) else {
            return;
        };
        let bytes = super::envelope::encode(env);
        if let Err(mpsc::error::TrySendError::Full(_)) = conn.tx.try_send(bytes) {
            tracing::warn!(conn_id, "outbound queue full; disconnecting slow client");
            drop(conn);
            self.deregister(conn_id);
        }
    }

    pub fn broadcast(&self, conn_ids: &[ConnectionId], env: &pb::Envelope) {
        let bytes = super::envelope::encode(env);
        let mut full: Vec<ConnectionId> = Vec::new();
        for id in conn_ids {
            let Some(conn) = self.connections.get(id).map(|c| c.clone()) else {
                continue;
            };
            if let Err(mpsc::error::TrySendError::Full(_)) = conn.tx.try_send(bytes.clone()) {
                full.push(*id);
            }
        }
        for id in full {
            tracing::warn!(
                conn_id = id,
                "outbound queue full; disconnecting slow client"
            );
            self.deregister(id);
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
            drop(proj);
            self.broadcast(&targets, env);
        }
    }

    /// True iff the given connection is registered as a collaborator (host or
    /// guest) on the project. Used to gate every project-scoped RPC.
    pub fn is_project_collaborator(&self, project_id: u64, conn_id: ConnectionId) -> bool {
        self.projects
            .get(&project_id)
            .is_some_and(|p| p.has_collaborator(conn_id))
    }

    /// True iff the connection is currently a participant in the room.
    pub fn is_room_participant(&self, room_id: u64, conn_id: ConnectionId) -> bool {
        self.rooms
            .get(&room_id)
            .is_some_and(|r| r.participants.iter().any(|p| p.conn_id == conn_id))
    }

    pub fn connection_alive(&self, conn_id: ConnectionId) -> bool {
        self.connections.contains_key(&conn_id)
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
