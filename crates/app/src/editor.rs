//! The editor's state, with no window and no GPU: the scene, the undo
//! history, the selection and the file.

use std::path::{Path, PathBuf};

use psito3d_scene::{History, ObjectId, Scene, SceneError};

/// What the editor is editing.
///
/// Changes are made straight to [`EditorState::scene`] by the panels. After
/// each frame [`EditorState::settle`] compares it with the last settled
/// scene and, once the person has let go of the mouse, records the change
/// in the history. One drag of a slider is therefore one undo step.
#[derive(Debug)]
pub struct EditorState {
    /// The scene being edited.
    pub scene: Scene,
    /// The object whose settings are shown.
    pub selected: Option<ObjectId>,
    /// The file the scene was opened from or saved to.
    pub file: Option<PathBuf>,
    /// The terrain brush in hand.
    pub brush: psito3d_ui::organisms::BrushSettings,
    history: History,
    settled: Scene,
    saved: Scene,
}

/// What can go wrong opening or saving.
#[derive(Debug)]
pub enum FileError {
    /// Reading or writing failed.
    Io(std::io::Error),
    /// The file is not a scene.
    Scene(SceneError),
}

impl std::error::Error for FileError {}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => e.fmt(f),
            Self::Scene(e) => e.fmt(f),
        }
    }
}

impl EditorState {
    /// An editor for `scene`, with an empty history.
    pub fn new(scene: Scene) -> Self {
        Self {
            settled: scene.clone(),
            saved: scene.clone(),
            scene,
            selected: None,
            file: None,
            brush: Default::default(),
            history: History::default(),
        }
    }

    /// Records the change since the last call in the history, unless the
    /// person is still dragging (`pointer_down`). Returns whether it did.
    pub fn settle(&mut self, pointer_down: bool) -> bool {
        if pointer_down {
            return false;
        }
        if self.scene.same_world(&self.settled) {
            // Only the camera moved: remember it, but it is not an undo step.
            self.settled.camera = self.scene.camera;
            return false;
        }
        self.history.checkpoint(&self.settled);
        self.settled = self.scene.clone();
        self.drop_stale_selection();
        true
    }

    /// Goes one step back.
    pub fn undo(&mut self) -> bool {
        // A change still in progress is settled first, so it is the one undone.
        self.settle(false);
        let camera = self.scene.camera;
        let done = self.history.undo(&mut self.scene);
        // Undo brings the world back, not the view.
        self.scene.camera = camera;
        self.settled = self.scene.clone();
        self.drop_stale_selection();
        done
    }

    /// Goes one step forward again.
    pub fn redo(&mut self) -> bool {
        let camera = self.scene.camera;
        let done = self.history.redo(&mut self.scene);
        self.scene.camera = camera;
        self.settled = self.scene.clone();
        self.drop_stale_selection();
        done
    }

    /// Whether there is anything to undo, counting a change not yet settled.
    pub fn can_undo(&self) -> bool {
        self.history.can_undo() || !self.scene.same_world(&self.settled)
    }

    /// Whether there is anything to redo.
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Whether the scene differs from the file.
    pub fn is_dirty(&self) -> bool {
        self.scene != self.saved
    }

    fn drop_stale_selection(&mut self) {
        if self.selected.is_some_and(|id| self.scene.object(id).is_none()) {
            self.selected = None;
        }
    }

    /// The folder of the scene file, against which pictures are found.
    pub fn base_dir(&self) -> Option<PathBuf> {
        self.file.as_deref().and_then(Path::parent).map(Path::to_owned)
    }

    /// Opens a scene file. The scene it replaces can be brought back with
    /// undo.
    pub fn open(&mut self, path: &Path) -> Result<(), FileError> {
        let text = std::fs::read_to_string(path).map_err(FileError::Io)?;
        let scene = Scene::from_json(&text).map_err(FileError::Scene)?;
        self.settle(false);
        self.scene = scene;
        self.settle(false);
        self.saved = self.scene.clone();
        self.file = Some(path.to_owned());
        self.selected = None;
        Ok(())
    }

    /// Saves the scene to `path`. Picture paths inside the scene's folder are
    /// stored relative to it, so the folder can be moved as a whole.
    pub fn save(&mut self, path: &Path) -> Result<(), FileError> {
        let mut scene = self.scene.clone();
        if let psito3d_scene::TerrainSource::Image { path: picture } = &mut scene.terrain.source
            && let Some(dir) = path.parent()
        {
            let absolute = if picture.is_relative() {
                self.base_dir().unwrap_or_default().join(&*picture)
            } else {
                picture.clone()
            };
            if let Ok(rel) = absolute.strip_prefix(dir) {
                *picture = rel.to_owned();
            }
        }
        std::fs::write(path, scene.to_json()).map_err(FileError::Io)?;
        self.scene = scene;
        self.settle(false);
        self.saved = self.scene.clone();
        self.file = Some(path.to_owned());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use psito3d_scene::{Material, Shape, TerrainSource};

    #[test]
    fn a_drag_is_one_undo_step() {
        let mut e = EditorState::new(Scene::default());
        for haze in [0.2, 0.4, 0.6] {
            e.scene.sky.haze = haze;
            assert!(!e.settle(true), "still dragging");
        }
        assert!(e.settle(false));
        assert!(e.undo());
        assert_eq!(e.scene.sky.haze, Scene::default().sky.haze);
        assert!(e.redo());
        assert_eq!(e.scene.sky.haze, 0.6);
    }

    #[test]
    fn moving_the_camera_is_not_an_undo_step() {
        let mut e = EditorState::new(Scene::default());
        e.scene.sky.haze = 0.9;
        e.settle(false);
        for _ in 0..500 {
            e.scene.camera.orbit(1.0, 0.0);
            assert!(!e.settle(false));
        }
        let view = e.scene.camera;
        assert!(e.undo(), "the haze change is still there to undo");
        assert_eq!(e.scene.sky.haze, Scene::default().sky.haze);
        assert_eq!(e.scene.camera, view, "and the view stays where it is");
        assert!(!e.can_undo());
    }

    #[test]
    fn undo_settles_a_change_in_progress_first() {
        let mut e = EditorState::new(Scene::default());
        e.scene.sea.level = 0.9;
        assert!(e.can_undo());
        assert!(e.undo());
        assert_eq!(e.scene.sea.level, Scene::default().sea.level);
    }

    #[test]
    fn undoing_an_add_drops_the_selection() {
        let mut e = EditorState::new(Scene::default());
        let id = e.scene.add_object(Shape::Sphere, Material::chrome()).unwrap();
        e.selected = Some(id);
        e.settle(false);
        e.undo();
        assert!(e.scene.objects.is_empty());
        assert_eq!(e.selected, None);
    }

    #[test]
    fn save_open_and_dirty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("island.psito3d");
        let mut e = EditorState::new(Scene::starter());
        e.scene.sky.haze = 0.77;
        assert!(e.is_dirty());
        e.save(&path).unwrap();
        assert!(!e.is_dirty());

        let mut other = EditorState::new(Scene::default());
        other.open(&path).unwrap();
        assert_eq!(other.scene.sky.haze, 0.77);
        assert_eq!(other.file.as_deref(), Some(path.as_path()));
        // Opening can be undone.
        assert!(other.undo());
        assert_eq!(other.scene, Scene::default());
    }

    #[test]
    fn pictures_next_to_the_scene_are_saved_relative() {
        let dir = tempfile::tempdir().unwrap();
        let mut e = EditorState::new(Scene::default());
        e.scene.terrain.source = TerrainSource::Image { path: dir.path().join("pics/doodle.png") };
        e.save(&dir.path().join("a.psito3d")).unwrap();
        let TerrainSource::Image { path } = &e.scene.terrain.source else { panic!() };
        assert_eq!(path, Path::new("pics/doodle.png"));
        assert_eq!(e.base_dir().as_deref(), Some(dir.path()));
    }

    #[test]
    fn opening_garbage_fails_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.psito3d");
        std::fs::write(&path, "nope").unwrap();
        let mut e = EditorState::new(Scene::default());
        assert!(matches!(e.open(&path), Err(FileError::Scene(_))));
        assert!(matches!(e.open(&dir.path().join("missing")), Err(FileError::Io(_))));
        assert_eq!(e.file, None);
    }
}
