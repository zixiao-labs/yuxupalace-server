use super::hub::{ConnectionId, user_id_to_u64};
use raidian::collab as pb;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RoomParticipant {
    pub conn_id: ConnectionId,
    pub user_id: String,
    pub peer_id: pb::PeerId,
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct RoomState {
    pub id: u64,
    pub host_user_id: String,
    pub live_kit_room: String,
    pub participants: Vec<RoomParticipant>,
    pub shared_project_ids: Vec<u64>,
}

impl RoomState {
    pub fn to_pb(&self) -> pb::Room {
        pb::Room {
            id: self.id,
            participants: self
                .participants
                .iter()
                .map(|p| pb::Participant {
                    user_id: user_id_to_u64(&p.user_id),
                    peer_id: Some(p.peer_id),
                    projects: Vec::new(),
                    location: None,
                    participant_index: 0,
                    role: pb::ParticipantRole::ParticipantMember as i32,
                    muted_reason: None,
                })
                .collect(),
            pending_participants: Vec::new(),
            shared_projects: self
                .shared_project_ids
                .iter()
                .map(|id| pb::ParticipantProject {
                    id: *id,
                    worktree_root_names: Vec::new(),
                })
                .collect(),
            live_kit_room: self.live_kit_room.clone(),
            channel_id: None,
        }
    }
}
