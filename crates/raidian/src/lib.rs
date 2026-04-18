/// Re-export of `prost::Message` for encoding / decoding protobuf messages.
pub use prost::Message;

/// Generated Protobuf message types for the YuXu DevOps platform API.
///
/// This module contains the raw structs emitted by `prost-build`. Consumers
/// can also use the organized submodules below for cleaner imports.
#[rustfmt::skip]
#[allow(clippy::all)]
mod generated;
pub use generated::*;

/// Authentication and user profile messages.
///
/// # Example
/// ```
/// use raidian::auth::LoginRequest;
/// let req = LoginRequest {
///     username_or_email: "alice@example.com".into(),
///     password: "secret".into(),
/// };
/// ```
pub mod auth {
    pub use crate::generated::{
        AuthResponse, LoginRequest, RegisterRequest, UpdateProfileRequest, UserProfile,
    };
}

/// Repository and git-related messages.
pub mod repository {
    pub use crate::generated::{
        BranchProtectionRule, CreateBranchProtectionRequest, CreateRepositoryRequest,
        GitBlobContent, GitCommitInfo, GitTreeEntry, Repository,
    };
}

/// Issue tracking messages.
pub mod issue {
    pub use crate::generated::{
        CreateCommentRequest, CreateIssueRequest, CreateLabelRequest, Issue, IssueComment, Label,
        ListCommentsResponse, ListIssuesRequest, ListIssuesResponse, UpdateIssueRequest,
    };
}

/// Merge request and code review messages.
pub mod merge_request {
    pub use crate::generated::{
        CreateMergeRequestRequest, CreateMrCommentRequest, ListMergeRequestsRequest,
        ListMergeRequestsResponse, MergeMergeRequestRequest, MergeRequest, MrComment, Review,
        SubmitReviewRequest, UpdateMergeRequestRequest,
    };
}

/// Repository member and ACL messages.
pub mod member {
    pub use crate::generated::{
        AddMemberRequest, ListMembersResponse, RepositoryMember, UpdateMemberRoleRequest,
    };
}

/// Real-time collaboration messages (yrs/CRDT over WebSocket).
///
/// These types are used by the YuXu binary WebSocket protocol for
/// multi-player editor sessions in downstream consumers such as Zed
/// and the Logos IDE.
pub mod collaboration {
    pub use crate::generated::{
        CollabAwareness, CollabJoinRequest, CollabJoinResponse, CollabParticipant,
        CollabParticipantJoined, CollabParticipantLeft, CollabSessionInfo, CollabUpdate,
        ListCollabSessionsResponse,
    };
}

/// CI/CD pipeline messages.
pub mod pipeline {
    pub use crate::generated::{
        ListPipelinesRequest, ListPipelinesResponse, Pipeline, PipelineStage,
        TriggerPipelineRequest,
    };
}

/// Dashboard and analytics messages.
pub mod dashboard {
    pub use crate::generated::{DashboardStats, RecentActivity};
}

/// Zed-style real-time collaboration protocol (package `raidian.collab`).
///
/// Covers the binary WebSocket envelope, CRDT buffer operations, room/project
/// state, and LSP/Git forwarding messages used by the `/rpc` endpoint.
#[rustfmt::skip]
#[allow(clippy::all)]
pub mod collab {
    include!("generated_collab.rs");
}
