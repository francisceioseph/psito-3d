//! Builds terrains on a background thread.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use psito3d_scene::TerrainSettings;
use psito3d_terrain::{CancelToken, Heightfield, TerrainError, generate_terrain};

use crate::catalog::Generators;

struct Job {
    cancel: CancelToken,
    rx: Receiver<Result<Heightfield, TerrainError>>,
}

/// Runs one terrain build at a time. A new request cancels the one running,
/// so dragging a slider never queues up stale terrains.
pub struct TerrainWorker {
    generators: Generators,
    job: Option<Job>,
    last: Option<(TerrainSettings, Option<PathBuf>)>,
}

impl TerrainWorker {
    /// A worker that builds with `generators`.
    pub fn new(generators: Generators) -> Self {
        Self { generators, job: None, last: None }
    }

    /// Builds `settings` unless it is what was asked last. `on_done` runs
    /// on the worker thread when the result is ready (to wake the UI).
    pub fn request(
        &mut self,
        settings: &TerrainSettings,
        base_dir: Option<PathBuf>,
        on_done: impl FnOnce() + Send + 'static,
    ) {
        let key = (settings.clone(), base_dir.clone());
        if self.last.as_ref() == Some(&key) {
            return;
        }
        self.last = Some(key);
        if let Some(job) = self.job.take() {
            job.cancel.cancel();
        }
        let (tx, rx) = channel();
        let cancel = CancelToken::new();
        let generators = Generators::clone(&self.generators);
        let settings = settings.clone();
        let token = cancel.clone();
        std::thread::Builder::new()
            .name("terrain".into())
            .spawn(move || {
                let find = |id: &str| generators.iter().find(|g| g.info().id == id).map(|g| g.as_ref());
                let result = generate_terrain(&settings, find, base_dir.as_deref(), &token);
                // The receiver is gone when a newer job replaced this one.
                let _ = tx.send(result);
                on_done();
            })
            .expect("can start a thread");
        self.job = Some(Job { cancel, rx });
    }

    /// Forgets the last request, so the next one runs even if it is equal
    /// (after the generators changed, say).
    pub fn invalidate(&mut self) {
        self.last = None;
    }

    /// The result of the running job, once it is ready.
    pub fn poll(&mut self) -> Option<Result<Heightfield, TerrainError>> {
        let job = self.job.as_ref()?;
        match job.rx.try_recv() {
            Ok(result) => {
                self.job = None;
                Some(result)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.job = None;
                None
            }
        }
    }

    /// Whether a build is running.
    pub fn is_busy(&self) -> bool {
        self.job.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    fn wait(worker: &mut TerrainWorker) -> Result<Heightfield, TerrainError> {
        let start = Instant::now();
        loop {
            if let Some(r) = worker.poll() {
                return r;
            }
            assert!(start.elapsed() < Duration::from_secs(20), "the terrain never came");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn worker() -> TerrainWorker {
        TerrainWorker::new(Arc::new(psito3d_terrain::builtin_generators()))
    }

    #[test]
    fn builds_and_reports() {
        let mut w = worker();
        let settings = TerrainSettings { resolution: 64, ..Default::default() };
        w.request(&settings, None, || {});
        assert!(w.is_busy());
        assert_eq!(wait(&mut w).unwrap().size(), 64);
        assert!(!w.is_busy());
    }

    #[test]
    fn the_same_request_is_not_repeated() {
        let mut w = worker();
        let settings = TerrainSettings { resolution: 32, ..Default::default() };
        w.request(&settings, None, || {});
        wait(&mut w).unwrap();
        w.request(&settings, None, || {});
        assert!(!w.is_busy());
        w.invalidate();
        w.request(&settings, None, || {});
        assert!(w.is_busy());
    }

    #[test]
    fn a_newer_request_wins() {
        let mut w = worker();
        w.request(&TerrainSettings { resolution: 1024, ..Default::default() }, None, || {});
        w.request(&TerrainSettings { resolution: 48, ..Default::default() }, None, || {});
        assert_eq!(wait(&mut w).unwrap().size(), 48);
    }
}
