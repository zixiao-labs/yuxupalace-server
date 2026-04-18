use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepoRole {
    Owner,
    Maintainer,
    Developer,
    Reporter,
    Guest,
}

impl RepoRole {
    pub fn rank(self) -> u8 {
        match self {
            RepoRole::Owner => 5,
            RepoRole::Maintainer => 4,
            RepoRole::Developer => 3,
            RepoRole::Reporter => 2,
            RepoRole::Guest => 1,
        }
    }
    pub fn at_least(self, other: RepoRole) -> bool {
        self.rank() >= other.rank()
    }
    pub fn as_str(self) -> &'static str {
        match self {
            RepoRole::Owner => "owner",
            RepoRole::Maintainer => "maintainer",
            RepoRole::Developer => "developer",
            RepoRole::Reporter => "reporter",
            RepoRole::Guest => "guest",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "owner" => Some(Self::Owner),
            "maintainer" => Some(Self::Maintainer),
            "developer" => Some(Self::Developer),
            "reporter" => Some(Self::Reporter),
            "guest" => Some(Self::Guest),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectRole {
    Admin,
    Member,
    Guest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelRole {
    Admin,
    Member,
    Talker,
    Guest,
    Banned,
}
