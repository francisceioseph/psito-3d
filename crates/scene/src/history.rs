//! Undo and redo.

use crate::Scene;

/// The undo history, kept as snapshots of the whole scene (the Memento
/// pattern).
///
/// A scene is a few kilobytes, so a snapshot per change costs nothing, and
/// unlike a list of commands it can never get out of step with the scene.
/// The editor takes a [`History::checkpoint`] *before* each change; a slider
/// drag takes one when the drag starts, not on every frame.
#[derive(Debug, Clone)]
pub struct History {
    undo: Vec<Scene>,
    redo: Vec<Scene>,
    limit: usize,
}

impl Default for History {
    fn default() -> Self {
        Self::new(200)
    }
}

impl History {
    /// A history that keeps up to `limit` steps back.
    pub fn new(limit: usize) -> Self {
        Self { undo: Vec::new(), redo: Vec::new(), limit: limit.max(1) }
    }

    /// Remembers `scene` as it is before a change. A checkpoint equal to the
    /// last one is ignored, and any redo steps are dropped.
    pub fn checkpoint(&mut self, scene: &Scene) {
        if self.undo.last() == Some(scene) {
            return;
        }
        self.undo.push(scene.clone());
        if self.undo.len() > self.limit {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Goes one step back; returns whether there was a step.
    pub fn undo(&mut self, scene: &mut Scene) -> bool {
        // Skip snapshots equal to the scene, left by a change that changed nothing.
        while let Some(previous) = self.undo.pop() {
            if &previous != scene {
                self.redo.push(std::mem::replace(scene, previous));
                return true;
            }
        }
        false
    }

    /// Goes one step forward again; returns whether there was a step.
    pub fn redo(&mut self, scene: &mut Scene) -> bool {
        match self.redo.pop() {
            Some(next) => {
                self.undo.push(std::mem::replace(scene, next));
                true
            }
            None => false,
        }
    }

    /// Whether there is anything to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether there is anything to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_haze(h: f32) -> Scene {
        let mut s = Scene::default();
        s.sky.haze = h;
        s
    }

    #[test]
    fn undo_and_redo() {
        let mut h = History::default();
        let mut scene = with_haze(0.1);
        h.checkpoint(&scene);
        scene.sky.haze = 0.5;
        assert!(h.undo(&mut scene));
        assert_eq!(scene.sky.haze, 0.1);
        assert!(h.redo(&mut scene));
        assert_eq!(scene.sky.haze, 0.5);
        assert!(!h.redo(&mut scene));
    }

    #[test]
    fn a_new_change_drops_redo() {
        let mut h = History::default();
        let mut scene = with_haze(0.1);
        h.checkpoint(&scene);
        scene.sky.haze = 0.5;
        h.undo(&mut scene);
        h.checkpoint(&scene);
        scene.sky.haze = 0.9;
        assert!(!h.can_redo());
    }

    #[test]
    fn empty_changes_are_skipped() {
        let mut h = History::default();
        let mut scene = with_haze(0.1);
        h.checkpoint(&scene);
        h.checkpoint(&scene);
        // A checkpoint whose change did nothing: undo has nothing to do.
        assert!(!h.undo(&mut scene));
    }

    #[test]
    fn limit_drops_the_oldest() {
        let mut h = History::new(2);
        let mut scene = with_haze(0.0);
        for i in 1..=3 {
            h.checkpoint(&scene);
            scene.sky.haze = i as f32 / 10.0;
        }
        assert!(h.undo(&mut scene) && h.undo(&mut scene));
        assert!(!h.undo(&mut scene));
        assert!((scene.sky.haze - 0.1).abs() < 1e-6);
    }
}
