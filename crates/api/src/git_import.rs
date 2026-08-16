use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{ApiError, ApiResult};

#[derive(Debug, Clone)]
pub struct GitCommitFile {
    pub commit: String,
    pub short: String,
    pub subject: String,
    pub bytes: Vec<u8>,
}

pub fn resolve_repo(repos_dir: &Path, repo: &str) -> ApiResult<PathBuf> {
    if repo.is_empty()
        || repo.contains("..")
        || repo.contains('/')
        || repo.contains('\\')
        || Path::new(repo).is_absolute()
    {
        return Err(ApiError::bad_request(
            "repo must be a single allowlisted directory name under data/repos",
        ));
    }
    let root = repos_dir
        .canonicalize()
        .map_err(|_| ApiError::bad_request("git repos directory is not available"))?;
    let candidate = root.join(repo);
    let resolved = candidate
        .canonicalize()
        .map_err(|_| ApiError::not_found(format!("git repo not found: {repo}")))?;
    if !resolved.starts_with(&root) {
        return Err(ApiError::bad_request("repo escapes allowlisted git root"));
    }
    if !resolved.join(".git").exists() && !resolved.join("HEAD").exists() {
        return Err(ApiError::bad_request(format!(
            "{repo} is not a git repository"
        )));
    }
    Ok(resolved)
}

pub fn validate_tracked_path(path: &str) -> ApiResult<()> {
    if path.is_empty() || path.contains('\0') || Path::new(path).is_absolute() {
        return Err(ApiError::bad_request("invalid file path"));
    }
    if path.split(['/', '\\']).any(|part| part == "..") {
        return Err(ApiError::bad_request("path must not contain .."));
    }
    Ok(())
}

pub fn read_file_history(
    repo: &Path,
    path: &str,
    max_commits: usize,
) -> ApiResult<Vec<GitCommitFile>> {
    validate_tracked_path(path)?;
    let max_commits = max_commits.clamp(1, 10);
    let output = Command::new("git")
        .current_dir(repo)
        .args([
            "log",
            "--follow",
            "--reverse",
            &format!("--max-count={max_commits}"),
            "--format=%H%x00%s",
            "--",
            path,
        ])
        .output()
        .map_err(|err| ApiError::internal(format!("git log failed: {err}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ApiError::bad_request(format!(
            "git log failed: {}",
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(2, '\0');
        let commit = parts.next().unwrap_or_default().trim();
        let subject = parts.next().unwrap_or_default().trim();
        if commit.is_empty() {
            continue;
        }
        let short = commit.chars().take(7).collect::<String>();
        let show = Command::new("git")
            .current_dir(repo)
            .args(["show", &format!("{commit}:{path}")])
            .output()
            .map_err(|err| ApiError::internal(format!("git show failed: {err}")))?;
        if !show.status.success() {
            continue;
        }
        if show.stdout.is_empty() {
            continue;
        }
        commits.push(GitCommitFile {
            commit: commit.to_string(),
            short,
            subject: if subject.is_empty() {
                short_label(&commit)
            } else {
                subject.to_string()
            },
            bytes: show.stdout,
        });
    }

    if commits.is_empty() {
        return Err(ApiError::not_found(format!(
            "no commits found for path {path}"
        )));
    }
    Ok(commits)
}

fn short_label(commit: &str) -> String {
    commit.chars().take(7).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_traversal_repo_names() {
        let err = resolve_repo(Path::new("/tmp"), "../etc").unwrap_err();
        assert_eq!(err.code, "bad_request");
    }

    #[test]
    fn rejects_dotdot_in_file_path() {
        let err = validate_tracked_path("../secret").unwrap_err();
        assert_eq!(err.code, "bad_request");
    }
}
