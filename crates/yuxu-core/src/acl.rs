use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Guest,
    Reporter,
    Developer,
    Maintainer,
    Owner,
}

impl Role {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "guest" => Some(Role::Guest),
            "reporter" => Some(Role::Reporter),
            "developer" => Some(Role::Developer),
            "maintainer" => Some(Role::Maintainer),
            "owner" => Some(Role::Owner),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Guest => "guest",
            Role::Reporter => "reporter",
            Role::Developer => "developer",
            Role::Maintainer => "maintainer",
            Role::Owner => "owner",
        }
    }

    pub fn can(&self, action: Action) -> bool {
        match action {
            Action::ViewRepo => true,
            Action::CreateIssue => *self >= Role::Reporter,
            Action::Comment => *self >= Role::Reporter,
            Action::CreateBranch => *self >= Role::Developer,
            Action::PushBranch => *self >= Role::Developer,
            Action::CreateMergeRequest => *self >= Role::Developer,
            Action::ReviewMergeRequest => *self >= Role::Developer,
            Action::TriggerPipeline => *self >= Role::Developer,
            Action::MergeMergeRequest => *self >= Role::Maintainer,
            Action::ManageLabels => *self >= Role::Maintainer,
            Action::ManageMembers => *self >= Role::Maintainer,
            Action::ManageBranchProtection => *self >= Role::Maintainer,
            Action::DeleteRepo => *self >= Role::Owner,
            Action::TransferOwnership => *self >= Role::Owner,
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    ViewRepo,
    CreateIssue,
    Comment,
    CreateBranch,
    PushBranch,
    CreateMergeRequest,
    ReviewMergeRequest,
    MergeMergeRequest,
    ManageLabels,
    ManageMembers,
    ManageBranchProtection,
    DeleteRepo,
    TransferOwnership,
    TriggerPipeline,
}
