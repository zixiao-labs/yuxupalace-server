use crate::error::AppError;
use git2::{
    BranchType, Commit, ObjectType, Oid, Repository, Signature, Sort, TreeWalkMode, TreeWalkResult,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub head_sha: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeEntry {
    pub name: String,
    pub entry_type: String,
    pub oid: String,
    pub mode: i32,
    pub size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobContent {
    pub content: String,
    pub is_binary: bool,
    pub size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub sha: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: i64,
    pub parent_shas: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MergeStrategy {
    Merge,
    Squash,
    Rebase,
}

impl MergeStrategy {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "merge" => Some(MergeStrategy::Merge),
            "squash" => Some(MergeStrategy::Squash),
            "rebase" => Some(MergeStrategy::Rebase),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffStats {
    pub diff: String,
    pub additions: i32,
    pub deletions: i32,
}

pub fn init_bare_repo(path: &Path) -> Result<Repository, AppError> {
    if path.exists() {
        return Err(AppError::Conflict(format!(
            "repository already exists at {}",
            path.display()
        )));
    }
    std::fs::create_dir_all(path)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("failed to create repo dir: {}", e)))?;
    let repo = Repository::init_bare(path)?;

    // Create an initial empty commit on the default branch
    let sig = Signature::now("YuXu System", "system@yuxu.dev")?;
    let tree_id = repo.treebuilder(None)?.write()?;
    let tree = repo.find_tree(tree_id)?;
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    Ok(repo)
}

pub fn open_repo(path: &Path) -> Result<Repository, AppError> {
    Repository::open_bare(path).map_err(|e| {
        if e.code() == git2::ErrorCode::NotFound {
            AppError::NotFound(format!(
                "repository not found at {}: {}",
                path.display(),
                e
            ))
        } else {
            AppError::Internal(anyhow::anyhow!(
                "failed to open repository at {}: {}",
                path.display(),
                e
            ))
        }
    })
}

pub fn list_branches(repo: &Repository, default_branch: &str) -> Result<Vec<BranchInfo>, AppError> {
    let mut branches = Vec::new();
    for branch_result in repo.branches(Some(BranchType::Local))? {
        let (branch, _) = branch_result?;
        let name = branch
            .name()?
            .unwrap_or("unknown")
            .to_string();
        let head_sha = branch
            .get()
            .peel_to_commit()?
            .id()
            .to_string();
        branches.push(BranchInfo {
            is_default: name == default_branch,
            name,
            head_sha,
        });
    }
    Ok(branches)
}

pub fn get_tree(
    repo: &Repository,
    ref_name: &str,
    path: &str,
) -> Result<Vec<TreeEntry>, AppError> {
    let obj = repo
        .revparse_single(ref_name)
        .map_err(|_| AppError::NotFound(format!("ref '{}' not found", ref_name)))?;
    let commit = obj
        .peel_to_commit()
        .map_err(|_| AppError::NotFound("could not resolve to commit".into()))?;
    let tree = commit.tree()?;

    let target_tree = if path.is_empty() || path == "/" {
        tree
    } else {
        let entry = tree
            .get_path(Path::new(path))
            .map_err(|_| AppError::NotFound(format!("path '{}' not found", path)))?;
        let obj = entry.to_object(repo)?;
        obj.peel_to_tree()
            .map_err(|_| AppError::Validation(format!("'{}' is not a directory", path)))?
    };

    let mut entries = Vec::new();
    for entry in target_tree.iter() {
        let name = entry.name().unwrap_or("").to_string();
        let entry_type = match entry.kind() {
            Some(ObjectType::Blob) => "blob",
            Some(ObjectType::Tree) => "tree",
            _ => "unknown",
        }
        .to_string();
        let oid = entry.id().to_string();
        let mode = entry.filemode();
        let size = if entry_type == "blob" {
            entry
                .to_object(repo)
                .ok()
                .and_then(|o| o.as_blob().map(|b| b.size() as i64))
        } else {
            None
        };
        entries.push(TreeEntry {
            name,
            entry_type,
            oid,
            mode,
            size,
        });
    }
    Ok(entries)
}

pub fn get_blob(
    repo: &Repository,
    ref_name: &str,
    path: &str,
) -> Result<BlobContent, AppError> {
    let obj = repo
        .revparse_single(ref_name)
        .map_err(|_| AppError::NotFound(format!("ref '{}' not found", ref_name)))?;
    let commit = obj.peel_to_commit()?;
    let tree = commit.tree()?;
    let entry = tree
        .get_path(Path::new(path))
        .map_err(|_| AppError::NotFound(format!("path '{}' not found", path)))?;
    let blob_obj = entry.to_object(repo)?;
    let blob = blob_obj
        .as_blob()
        .ok_or_else(|| AppError::Validation(format!("'{}' is not a file", path)))?;

    let is_binary = blob.is_binary();
    let size = blob.size() as i64;
    let content = if is_binary {
        String::new()
    } else {
        String::from_utf8_lossy(blob.content()).to_string()
    };

    Ok(BlobContent {
        content,
        is_binary,
        size,
    })
}

pub fn list_commits(
    repo: &Repository,
    ref_name: &str,
    limit: usize,
    offset: usize,
) -> Result<Vec<CommitInfo>, AppError> {
    let obj = repo
        .revparse_single(ref_name)
        .map_err(|_| AppError::NotFound(format!("ref '{}' not found", ref_name)))?;
    let commit = obj.peel_to_commit()?;

    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(Sort::TIME)?;
    revwalk.push(commit.id())?;

    let mut commits = Vec::new();
    for (idx, oid_result) in revwalk.enumerate() {
        if idx < offset {
            continue;
        }
        if commits.len() >= limit {
            break;
        }
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;
        let parent_shas = commit.parent_ids().map(|id| id.to_string()).collect();
        commits.push(CommitInfo {
            sha: oid.to_string(),
            message: commit.message().unwrap_or("").to_string(),
            author_name: commit.author().name().unwrap_or("").to_string(),
            author_email: commit.author().email().unwrap_or("").to_string(),
            timestamp: commit.time().seconds(),
            parent_shas,
        });
    }
    Ok(commits)
}

pub fn create_branch(
    repo: &Repository,
    branch_name: &str,
    from_ref: &str,
) -> Result<(), AppError> {
    let obj = repo
        .revparse_single(from_ref)
        .map_err(|_| AppError::NotFound(format!("ref '{}' not found", from_ref)))?;
    let commit = obj.peel_to_commit()?;
    repo.branch(branch_name, &commit, false)?;
    Ok(())
}

pub fn delete_branch(repo: &Repository, branch_name: &str) -> Result<(), AppError> {
    let mut branch = repo
        .find_branch(branch_name, BranchType::Local)
        .map_err(|_| AppError::NotFound(format!("branch '{}' not found", branch_name)))?;
    branch.delete()?;
    Ok(())
}

pub fn get_diff(
    repo: &Repository,
    from_ref: &str,
    to_ref: &str,
) -> Result<DiffStats, AppError> {
    let from_obj = repo
        .revparse_single(from_ref)
        .map_err(|_| AppError::NotFound(format!("ref '{}' not found", from_ref)))?;
    let to_obj = repo
        .revparse_single(to_ref)
        .map_err(|_| AppError::NotFound(format!("ref '{}' not found", to_ref)))?;

    let from_tree = from_obj.peel_to_commit()?.tree()?;
    let to_tree = to_obj.peel_to_commit()?.tree()?;

    let diff = repo.diff_tree_to_tree(Some(&from_tree), Some(&to_tree), None)?;
    let stats = diff.stats()?;

    let mut diff_text = Vec::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        diff_text.extend_from_slice(line.content());
        true
    })?;

    Ok(DiffStats {
        diff: String::from_utf8_lossy(&diff_text).to_string(),
        additions: stats.insertions() as i32,
        deletions: stats.deletions() as i32,
    })
}

pub fn merge_branches(
    repo: &Repository,
    source_ref: &str,
    target_ref: &str,
    strategy: MergeStrategy,
    committer_name: &str,
    committer_email: &str,
) -> Result<String, AppError> {
    // Rebase is not implemented
    if matches!(strategy, MergeStrategy::Rebase) {
        return Err(AppError::Internal(anyhow::anyhow!(
            "rebase strategy not implemented"
        )));
    }

    let source_obj = repo
        .revparse_single(source_ref)
        .map_err(|_| AppError::NotFound(format!("source ref '{}' not found", source_ref)))?;
    let target_obj = repo
        .revparse_single(target_ref)
        .map_err(|_| AppError::NotFound(format!("target ref '{}' not found", target_ref)))?;

    let source_commit = source_obj.peel_to_commit()?;
    let target_commit = target_obj.peel_to_commit()?;

    let source_tree = source_commit.tree()?;
    let target_tree = target_commit.tree()?;

    let ancestor = repo.merge_base(source_commit.id(), target_commit.id())?;
    let ancestor_commit = repo.find_commit(ancestor)?;
    let ancestor_tree = ancestor_commit.tree()?;

    let mut merge_index = repo.merge_trees(&ancestor_tree, &target_tree, &source_tree, None)?;

    if merge_index.has_conflicts() {
        return Err(AppError::Conflict(
            "merge has conflicts that must be resolved".to_string(),
        ));
    }

    let merged_tree_oid = merge_index.write_tree_to(repo)?;
    let merged_tree = repo.find_tree(merged_tree_oid)?;

    let sig = Signature::now(committer_name, committer_email)?;
    let message = match strategy {
        MergeStrategy::Merge => format!(
            "Merge branch '{}' into '{}'",
            source_ref, target_ref
        ),
        MergeStrategy::Squash => format!(
            "Squashed commit from branch '{}'",
            source_ref
        ),
        MergeStrategy::Rebase => format!(
            "Rebased branch '{}' onto '{}'",
            source_ref, target_ref
        ),
    };

    let parents: Vec<&Commit> = match strategy {
        MergeStrategy::Merge => vec![&target_commit, &source_commit],
        MergeStrategy::Squash | MergeStrategy::Rebase => vec![&target_commit],
    };

    let commit_oid = repo.commit(
        Some(&format!("refs/heads/{}", target_ref)),
        &sig,
        &sig,
        &message,
        &merged_tree,
        &parents,
    )?;

    Ok(commit_oid.to_string())
}