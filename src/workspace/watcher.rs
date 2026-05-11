//! Filesystem watcher for the current workspace.
//!
//! Wraps `notify` with a crossbeam channel so the app can drain events
//! during its normal update tick without blocking.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};
use crossbeam_channel::{Receiver, Sender};
use notify::{
    event::{ModifyKind, RenameMode},
    EventKind, RecursiveMode, Watcher as NotifyWatcher,
};

#[derive(Debug, Clone)]
pub struct Change {
    pub path: PathBuf,
    pub kind: ChangeKind,
    /// Useful for debouncing or ordering if the app ever wants it.
    #[allow(dead_code)]
    pub at: SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Modified,
    Created,
    Removed,
    Renamed,
    Other,
}

pub struct WorkspaceWatcher {
    _watcher: notify::RecommendedWatcher,
    rx: Receiver<Change>,
}

impl WorkspaceWatcher {
    pub fn watch(root: &Path) -> Result<Self> {
        let (tx, rx) = crossbeam_channel::unbounded::<Change>();
        let mut w = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                let kind = match event.kind {
                    EventKind::Create(_) => ChangeKind::Created,
                    EventKind::Remove(_) => ChangeKind::Removed,
                    EventKind::Modify(ModifyKind::Name(RenameMode::Any))
                    | EventKind::Modify(ModifyKind::Name(RenameMode::From))
                    | EventKind::Modify(ModifyKind::Name(RenameMode::To))
                    | EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                        ChangeKind::Renamed
                    }
                    EventKind::Modify(_) => ChangeKind::Modified,
                    _ => ChangeKind::Other,
                };
                for p in event.paths {
                    let _ = send_filtered(&tx, p, kind);
                }
            }
        })
        .context("create notify watcher")?;
        w.watch(root, RecursiveMode::Recursive)
            .with_context(|| format!("watch {}", root.display()))?;
        Ok(Self { _watcher: w, rx })
    }

    /// Drain all pending events into a Vec. Cheap; returns empty when idle.
    pub fn drain(&self) -> Vec<Change> {
        self.rx.try_iter().collect()
    }
}

fn send_filtered(tx: &Sender<Change>, path: PathBuf, kind: ChangeKind) -> Result<()> {
    // Filter the noisiest dirs at the source so the app doesn't have to.
    // These are essentially always noise for an editor.
    if path
        .components()
        .any(|c| matches!(c.as_os_str().to_str(), Some(".git" | "target" | "node_modules")))
    {
        return Ok(());
    }
    tx.send(Change {
        path,
        kind,
        at: SystemTime::now(),
    })
    .ok();
    Ok(())
}
