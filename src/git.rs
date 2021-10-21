use chrono::{DateTime, FixedOffset, Utc};
use git2::{Commit, Repository};
use std::{
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct GitInfo {
    /// Path to the repository.
    path: PathBuf,
    /// If the operation is required. If not, it will fail silently.
    required: bool,
}

impl GitInfo {
    pub fn new(path: &Path, required: bool) -> Self {
        Self {
            path: path.into(),
            required,
        }
    }

    fn render_commit(&self, write: &mut dyn Write, commit: &Commit) -> anyhow::Result<()> {
        let tz = FixedOffset::west(commit.time().offset_minutes() * 60);
        let time =
            DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(commit.time().seconds() as u64))
                .with_timezone(&tz);

        writeln!(write, "    Commit: {}", commit.id())?;
        writeln!(write, "    Author: {}", commit.author())?;
        writeln!(write, "    Date: {}", time.to_rfc2822())?;

        writeln!(write)?;

        for line in String::from_utf8_lossy(commit.message_bytes()).lines() {
            writeln!(write, "        {}", line)?;
        }

        Ok(())
    }

    fn render_git(&self, write: &mut dyn Write) -> anyhow::Result<()> {
        let repo = Repository::open(&self.path)?;

        let remote = repo.find_remote("origin")?;
        writeln!(
            write,
            "**Git:** `{repo}` @ `{ref}`",
            repo = remote.url().unwrap_or("<unknown>"),
            ref = repo.head()?.name().unwrap_or("<unknown>")
        )?;
        writeln!(write)?;

        let commit = repo
            .head()?
            .target()
            .map(|id| repo.find_commit(id))
            .transpose()?;

        if let Some(commit) = commit {
            self.render_commit(write, &commit)?;
        }

        Ok(())
    }
}

impl super::Addon for GitInfo {
    fn render(&self, write: &mut dyn Write) -> anyhow::Result<()> {
        match self.render_git(write) {
            Err(err) if self.required => Err(err),
            _ => Ok(()),
        }
    }
}
