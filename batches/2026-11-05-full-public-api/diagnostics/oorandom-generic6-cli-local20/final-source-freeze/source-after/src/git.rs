use anyhow::{ensure, Context, Result};
use std::{path::Path, process::Command};

pub fn run(dir: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .context("Run git")?;
    ensure!(
        out.status.success(),
        "git {} failed: {}",
        args.first().unwrap_or(&""),
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(String::from_utf8(out.stdout)?.trim().into())
}

/// Resolve a public, immutable source before verification. No generated evidence needs a push.
pub fn published_source(root: &Path, remote: &str) -> Result<serde_json::Value> {
    ensure!(
        !remote.starts_with('-') && !remote.contains('/'),
        "Use a Git remote name"
    );
    let raw = run(root, &["remote", "get-url", remote])?;
    let url = raw
        .strip_prefix("git@github.com:")
        .map(|s| format!("https://github.com/{s}"))
        .unwrap_or(raw);
    let url = url.trim_end_matches('/').trim_end_matches(".git");
    let path = url
        .strip_prefix("https://github.com/")
        .context("Source remote must be a GitHub repository")?;
    let parts: Vec<_> = path.split('/').collect();
    ensure!(
        parts.len() == 2
            && parts.iter().all(|s| !s.is_empty()
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))),
        "Invalid GitHub repository URL"
    );
    let commit = run(root, &["rev-parse", "HEAD"])?;
    ensure!(
        run(root, &["status", "--porcelain"])?.is_empty(),
        "Commit and push source changes before cargo proofs run"
    );
    run(root, &["fetch", "--prune", remote])?;
    ensure!(
        !run(
            root,
            &[
                "for-each-ref",
                "--format=%(refname)",
                "--contains",
                &commit,
                &format!("refs/remotes/{remote}/")
            ]
        )?
        .is_empty(),
        "Push the source commit before cargo proofs run"
    );
    Ok(serde_json::json!({"repository":url,"commit":commit}))
}

/// A detached, disposable checkout. Drop always removes the Git worktree registration.
pub struct Worktree {
    root: std::path::PathBuf,
    pub path: std::path::PathBuf,
    _temp: tempfile::TempDir,
}
impl Worktree {
    pub fn create(root: &Path, commit: &str) -> Result<Self> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().canonicalize()?.join("source");
        run(
            root,
            &[
                "worktree",
                "add",
                "--detach",
                path.to_str().context("Non-UTF8 worktree path")?,
                commit,
            ],
        )?;
        Ok(Self {
            root: root.to_owned(),
            path,
            _temp: temp,
        })
    }
    pub fn unchanged(&self) -> Result<()> {
        ensure!(
            run(
                &self.path,
                &["status", "--porcelain", "--untracked-files=no"]
            )?
            .is_empty(),
            "Tracked inputs changed during verification; run again"
        );
        Ok(())
    }
}
impl Drop for Worktree {
    fn drop(&mut self) {
        if let Some(path) = self.path.to_str() {
            let _ = run(&self.root, &["worktree", "remove", "--force", path]);
        }
    }
}
