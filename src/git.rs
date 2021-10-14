use chrono::{DateTime, FixedOffset, Utc};
use git2::{Commit, Repository};
use std::{
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct GitInfo {
    path: PathBuf,
}

impl GitInfo {
    pub fn new(path: &Path) -> Self {
        Self { path: path.into() }
    }

    fn render_commit<W>(&self, write: &mut W, commit: &Commit) -> anyhow::Result<()>
    where
        W: Write,
    {
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
}

impl<W> super::Addon<W> for GitInfo
where
    W: Write,
{
    fn render(&self, write: &mut W) -> anyhow::Result<()> {
        let repo = Repository::open(&self.path)?;

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
