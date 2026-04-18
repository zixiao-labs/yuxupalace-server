use super::envelope;
use super::hub::{CollabHub, Connection, ConnectionId};
use super::project::{ProjectCollaborator, ProjectState};
use super::room::{RoomParticipant, RoomState};
use crate::app_state::AppState;
use axum::extract::{
    State, WebSocketUpgrade,
    ws::{Message, WebSocket},
};
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use raidian::collab as pb;
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn handler(
    State(state): State<AppState>,
    upgrade: WebSocketUpgrade,
) -> impl IntoResponse {
    upgrade.on_upgrade(move |socket| run(state, socket))
}

async fn run(state: AppState, socket: WebSocket) {
    let hub = state.hub.clone();
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<bytes::Bytes>();

    let conn_id = hub.alloc_connection_id();
    let conn = Arc::new(Connection {
        id: conn_id,
        tx: out_tx,
    });
    hub.register(conn);

    let writer = tokio::spawn(async move {
        while let Some(bytes) = out_rx.recv().await {
            if ws_tx.send(Message::Binary(bytes)).await.is_err() {
                break;
            }
        }
    });

    while let Some(msg) = ws_rx.next().await {
        let Ok(msg) = msg else { break };
        match msg {
            Message::Binary(data) => {
                let Ok(env) = envelope::decode(&data) else {
                    hub.send_to(
                        conn_id,
                        &envelope::error(0, pb::error::Code::Internal, "bad envelope"),
                    );
                    continue;
                };
                handle_envelope(&state, conn_id, env).await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    hub.deregister(conn_id);
    writer.abort();
}

async fn handle_envelope(state: &AppState, conn_id: ConnectionId, env: pb::Envelope) {
    use pb::envelope::Payload;
    let hub = state.hub.clone();
    let req_id = env.id;
    let Some(payload) = env.payload else {
        return;
    };

    match payload {
        Payload::Hello(h) => {
            if let Some(token) = h.metadata.get("token")
                && let Ok(claims) = state.jwt.verify(token)
            {
                hub.set_user(conn_id, claims.sub);
            }
            hub.send_to(
                conn_id,
                &envelope::respond_with(req_id, Payload::Ack(pb::Ack {})),
            );
        }
        Payload::Ping(_) => {
            hub.send_to(
                conn_id,
                &envelope::respond_with(req_id, Payload::Pong(pb::Pong {})),
            );
        }
        Payload::CreateRoom(_) => {
            let user_id = hub.user_of(conn_id);
            let room_id = hub.alloc_room_id();
            let live_kit_room = format!("yuxu-room-{room_id}");
            let state_room = RoomState {
                id: room_id,
                host_user_id: user_id.clone(),
                live_kit_room: live_kit_room.clone(),
                participants: vec![RoomParticipant {
                    conn_id,
                    user_id: user_id.clone(),
                    peer_id: pb::PeerId {
                        owner_id: 1,
                        id: conn_id,
                    },
                }],
                shared_project_ids: Vec::new(),
            };
            let room_pb = state_room.to_pb();
            hub.rooms.insert(room_id, state_room);
            let (lk_token, lk_url) = livekit_info(state, &user_id, &live_kit_room);
            hub.send_to(
                conn_id,
                &envelope::respond_with(
                    req_id,
                    Payload::CreateRoomResponse(pb::CreateRoomResponse {
                        room: Some(room_pb),
                        live_kit_connection_info_token: lk_token,
                        live_kit_connection_info_server_url: lk_url,
                    }),
                ),
            );
        }
        Payload::JoinRoom(j) => {
            let user_id = hub.user_of(conn_id);
            let (room_pb, live_kit_room) = {
                let Some(mut room) = hub.rooms.get_mut(&j.id) else {
                    hub.send_to(
                        conn_id,
                        &envelope::error(req_id, pb::error::Code::NotFound, "room not found"),
                    );
                    return;
                };
                room.participants.push(RoomParticipant {
                    conn_id,
                    user_id: user_id.clone(),
                    peer_id: pb::PeerId {
                        owner_id: 1,
                        id: conn_id,
                    },
                });
                (room.to_pb(), room.live_kit_room.clone())
            };
            hub.send_to(
                conn_id,
                &envelope::respond_with(
                    req_id,
                    Payload::JoinRoomResponse({
                        let (lk_token, lk_url) = livekit_info(state, &user_id, &live_kit_room);
                        pb::JoinRoomResponse {
                            room: Some(room_pb.clone()),
                            live_kit_connection_info_token: lk_token,
                            live_kit_connection_info_server_url: lk_url,
                            channel_id: None,
                        }
                    }),
                ),
            );
            let targets: Vec<ConnectionId> = room_pb
                .participants
                .iter()
                .filter_map(|p| p.peer_id.as_ref().map(|pid| pid.id))
                .filter(|id| *id != conn_id)
                .collect();
            hub.broadcast(
                &targets,
                &envelope::unsolicited(Payload::RoomUpdated(pb::RoomUpdated {
                    room: Some(room_pb),
                })),
            );
        }
        Payload::LeaveRoom(_) => {
            hub.rooms
                .iter_mut()
                .for_each(|mut r| r.participants.retain(|p| p.conn_id != conn_id));
            hub.send_to(
                conn_id,
                &envelope::respond_with(req_id, Payload::Ack(pb::Ack {})),
            );
        }
        Payload::ShareProject(sp) => {
            let user_id = hub.user_of(conn_id);
            let project_id = hub.alloc_project_id();
            let proj = ProjectState {
                id: project_id,
                room_id: sp.room_id,
                host_conn_id: conn_id,
                host_user_id: user_id.clone(),
                collaborators: vec![ProjectCollaborator {
                    conn_id,
                    user_id: user_id.clone(),
                    replica_id: 0,
                    is_host: true,
                }],
                next_replica_id: 0,
            };
            let _ = (sp.worktrees, sp.is_ssh_project);
            hub.projects.insert(project_id, proj);
            if let Some(mut room) = hub.rooms.get_mut(&sp.room_id) {
                room.shared_project_ids.push(project_id);
            }
            hub.send_to(
                conn_id,
                &envelope::respond_with(
                    req_id,
                    Payload::ShareProjectResponse(pb::ShareProjectResponse { project_id }),
                ),
            );
        }
        Payload::UnshareProject(u) => {
            if let Some((_, proj)) = hub.projects.remove(&u.project_id) {
                if let Some(mut room) = hub.rooms.get_mut(&proj.room_id) {
                    room.shared_project_ids.retain(|id| *id != proj.id);
                }
                let targets = proj
                    .collaborators
                    .iter()
                    .filter(|c| !c.is_host)
                    .map(|c| c.conn_id)
                    .collect::<Vec<_>>();
                hub.broadcast(&targets, &envelope::unsolicited(Payload::UnshareProject(u)));
            }
            hub.send_to(
                conn_id,
                &envelope::respond_with(req_id, Payload::Ack(pb::Ack {})),
            );
        }
        Payload::JoinProject(j) => {
            let user_id = hub.user_of(conn_id);
            let (replica_id, host_conn, existing) = {
                let Some(mut proj) = hub.projects.get_mut(&j.project_id) else {
                    hub.send_to(
                        conn_id,
                        &envelope::error(req_id, pb::error::Code::NotFound, "project not found"),
                    );
                    return;
                };
                let replica_id = proj.alloc_replica();
                let existing: Vec<pb::Collaborator> = proj
                    .collaborators
                    .iter()
                    .map(|c| pb::Collaborator {
                        peer_id: Some(pb::PeerId {
                            owner_id: 1,
                            id: c.conn_id,
                        }),
                        replica_id: c.replica_id,
                        user_id: super::hub::user_id_to_u64(&c.user_id),
                        is_host: c.is_host,
                        committer_name: None,
                        committer_email: None,
                    })
                    .collect();
                proj.collaborators.push(ProjectCollaborator {
                    conn_id,
                    user_id: user_id.clone(),
                    replica_id,
                    is_host: false,
                });
                (replica_id, proj.host_conn_id, existing)
            };
            hub.send_to(
                conn_id,
                &envelope::respond_with(
                    req_id,
                    Payload::JoinProjectResponse(pb::JoinProjectResponse {
                        replica_id,
                        worktrees: Vec::new(),
                        collaborators: existing,
                        language_servers: Vec::new(),
                        repositories: Vec::new(),
                        role: pb::RoleType::RoleMember as i32,
                    }),
                ),
            );
            hub.send_to(
                host_conn,
                &envelope::unsolicited(Payload::AddProjectCollaborator(
                    pb::AddProjectCollaborator {
                        project_id: j.project_id,
                        collaborator: Some(pb::Collaborator {
                            peer_id: Some(pb::PeerId {
                                owner_id: 1,
                                id: conn_id,
                            }),
                            replica_id,
                            user_id: super::hub::user_id_to_u64(&user_id),
                            is_host: false,
                            committer_name: None,
                            committer_email: None,
                        }),
                    },
                )),
            );
        }
        Payload::LeaveProject(l) => {
            if let Some(mut proj) = hub.projects.get_mut(&l.project_id) {
                proj.collaborators.retain(|c| c.conn_id != conn_id);
                hub.send_to(
                    proj.host_conn_id,
                    &envelope::unsolicited(Payload::RemoveProjectCollaborator(
                        pb::RemoveProjectCollaborator {
                            project_id: l.project_id,
                            peer_id: Some(pb::PeerId {
                                owner_id: 1,
                                id: conn_id,
                            }),
                        },
                    )),
                );
            }
            hub.send_to(
                conn_id,
                &envelope::respond_with(req_id, Payload::Ack(pb::Ack {})),
            );
        }

        // CRDT / state broadcasts.
        Payload::UpdateBuffer(u) => {
            let pid = u.project_id;
            hub.broadcast_to_project(
                pid,
                &envelope::unsolicited(Payload::UpdateBuffer(u)),
                Some(conn_id),
            );
        }
        Payload::UpdateWorktree(u) => {
            let pid = u.project_id;
            hub.broadcast_to_project(
                pid,
                &envelope::unsolicited(Payload::UpdateWorktree(u)),
                Some(conn_id),
            );
        }
        Payload::UpdateRepository(u) => {
            let pid = u.project_id;
            hub.broadcast_to_project(
                pid,
                &envelope::unsolicited(Payload::UpdateRepository(u)),
                Some(conn_id),
            );
        }
        Payload::UpdateDiagnosticSummary(u) => {
            let pid = u.project_id;
            hub.broadcast_to_project(
                pid,
                &envelope::unsolicited(Payload::UpdateDiagnosticSummary(u)),
                Some(conn_id),
            );
        }
        Payload::UpdateLanguageServer(u) => {
            let pid = u.project_id;
            hub.broadcast_to_project(
                pid,
                &envelope::unsolicited(Payload::UpdateLanguageServer(u)),
                Some(conn_id),
            );
        }
        Payload::UpdateParticipantLocation(u) => {
            if let Some(room) = hub.rooms.get(&u.room_id) {
                let targets: Vec<ConnectionId> = room
                    .participants
                    .iter()
                    .map(|p| p.conn_id)
                    .filter(|id| *id != conn_id)
                    .collect();
                hub.broadcast(
                    &targets,
                    &envelope::unsolicited(Payload::UpdateParticipantLocation(u)),
                );
            }
        }

        // Host-bound forwards.
        Payload::SaveBuffer(r) => {
            forward_to_host(&hub, conn_id, req_id, r.project_id, Payload::SaveBuffer(r))
        }
        Payload::ReloadBuffers(r) => forward_to_host(
            &hub,
            conn_id,
            req_id,
            r.project_id,
            Payload::ReloadBuffers(r),
        ),
        Payload::OpenBufferById(r) => forward_to_host(
            &hub,
            conn_id,
            req_id,
            r.project_id,
            Payload::OpenBufferById(r),
        ),
        Payload::OpenBufferByPath(r) => forward_to_host(
            &hub,
            conn_id,
            req_id,
            r.project_id,
            Payload::OpenBufferByPath(r),
        ),
        Payload::LspRequest(r) => {
            forward_to_host(&hub, conn_id, req_id, r.project_id, Payload::LspRequest(r))
        }
        Payload::GitStage(r) => {
            forward_to_host(&hub, conn_id, req_id, r.project_id, Payload::GitStage(r))
        }
        Payload::GitUnstage(r) => {
            forward_to_host(&hub, conn_id, req_id, r.project_id, Payload::GitUnstage(r))
        }
        Payload::GitCommit(r) => {
            forward_to_host(&hub, conn_id, req_id, r.project_id, Payload::GitCommit(r))
        }
        Payload::GitBranches(r) => {
            forward_to_host(&hub, conn_id, req_id, r.project_id, Payload::GitBranches(r))
        }

        other => {
            let _ = other;
            hub.send_to(
                conn_id,
                &envelope::respond_with(req_id, Payload::Ack(pb::Ack {})),
            );
        }
    }
}

fn forward_to_host(
    hub: &CollabHub,
    sender: ConnectionId,
    req_id: u32,
    project_id: u64,
    payload: pb::envelope::Payload,
) {
    let Some(proj) = hub.projects.get(&project_id) else {
        hub.send_to(
            sender,
            &envelope::error(req_id, pb::error::Code::NotFound, "project not found"),
        );
        return;
    };
    let host = proj.host_conn_id;
    let env = pb::Envelope {
        id: req_id,
        responding_to: None,
        original_sender_id: Some(pb::PeerId {
            owner_id: 1,
            id: sender,
        }),
        payload: Some(payload),
    };
    hub.send_to(host, &env);
}

/// Best-effort LiveKit connection info. Until real token signing is wired up,
/// `issue_token` fails — we log and surface empty strings so clients can detect
/// that audio/video isn't available rather than connecting with a junk token.
fn livekit_info(state: &AppState, user_id: &str, room: &str) -> (String, String) {
    match super::livekit::issue_token(user_id, room) {
        Ok(tok) => (tok, super::livekit::server_url(&state.config.live_kit_url)),
        Err(e) => {
            tracing::warn!(error = %e, room, "livekit token unavailable");
            (String::new(), String::new())
        }
    }
}
