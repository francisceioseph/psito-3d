//! Scene codes: a whole scene in one short line of text, to paste in a chat
//! so someone else opens the same world.
//!
//! A code is `p3d1.` followed by the scene's compact JSON, compressed with
//! deflate and written in URL-safe base64. Reading one goes through the same
//! checks as a scene file, with limits on the size, so a code from a
//! stranger cannot do more than a hand-edited file could.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

use crate::{Scene, SceneError};

/// The start of every code; the digit is the code format.
pub const CODE_PREFIX: &str = "p3d1.";
/// The longest code accepted, in characters.
const MAX_CODE_LEN: usize = 256 * 1024;
/// The most a code may expand to, in bytes.
const MAX_JSON_LEN: usize = 4 * 1024 * 1024;

impl Scene {
    /// The scene as a code. Picture terrains keep their path, which only
    /// works on the same computer.
    pub fn to_code(&self) -> String {
        let json = serde_json::to_vec(self).expect("a scene is always serialisable");
        let packed = miniz_oxide::deflate::compress_to_vec(&json, 9);
        format!("{CODE_PREFIX}{}", URL_SAFE_NO_PAD.encode(packed))
    }

    /// Reads a code. Spaces and line breaks (from a chat that wrapped the
    /// line) are ignored.
    pub fn from_code(code: &str) -> Result<Self, SceneError> {
        let code: String = code.chars().filter(|c| !c.is_whitespace()).collect();
        let body = code.strip_prefix(CODE_PREFIX).ok_or(SceneError::BadCode("it does not start with p3d1."))?;
        if body.len() > MAX_CODE_LEN {
            return Err(SceneError::BadCode("it is too long"));
        }
        let packed =
            URL_SAFE_NO_PAD.decode(body).map_err(|_| SceneError::BadCode("it has characters that do not belong"))?;
        let json = miniz_oxide::inflate::decompress_to_vec_with_limit(&packed, MAX_JSON_LEN)
            .map_err(|_| SceneError::BadCode("it is cut short or damaged"))?;
        let text = std::str::from_utf8(&json).map_err(|_| SceneError::BadCode("it is damaged"))?;
        Scene::from_json(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let scene = Scene::starter();
        let code = scene.to_code();
        assert!(code.starts_with(CODE_PREFIX));
        assert_eq!(Scene::from_code(&code).unwrap(), scene);
    }

    #[test]
    fn short_enough_to_paste() {
        // The starter scene, terrain recipe and a chrome ball included.
        assert!(Scene::starter().to_code().len() < 1200, "{}", Scene::starter().to_code().len());
    }

    #[test]
    fn wrapped_codes_still_open() {
        let code = Scene::starter().to_code();
        let wrapped: String =
            code.chars().enumerate().flat_map(|(i, c)| if i % 40 == 39 { vec![c, '\n'] } else { vec![c] }).collect();
        assert_eq!(Scene::from_code(&format!("  {wrapped}  ")).unwrap(), Scene::starter());
    }

    #[test]
    fn bad_codes_are_refused() {
        for bad in ["", "hello", "p3d1.", "p3d1.!!!!", "p3d1.AAAA", "p3d2.abc"] {
            assert!(Scene::from_code(bad).is_err(), "{bad:?}");
        }
        let code = Scene::starter().to_code();
        assert!(Scene::from_code(&code[..code.len() / 2]).is_err(), "a code cut in half");
    }

    #[test]
    fn a_compression_bomb_is_refused() {
        // 50 MB of spaces compress to a few kilobytes.
        let bomb = miniz_oxide::deflate::compress_to_vec(&vec![b' '; 50 * 1024 * 1024], 9);
        let code = format!("{CODE_PREFIX}{}", URL_SAFE_NO_PAD.encode(bomb));
        assert!(matches!(Scene::from_code(&code), Err(SceneError::BadCode(_))));
    }

    #[test]
    fn codes_are_checked_like_files() {
        let mut scene = Scene::starter();
        scene.sea.level = 0.3;
        let json = serde_json::to_string(&scene).unwrap().replace("\"level\":0.3", "\"level\":9.0");
        let code = format!(
            "{CODE_PREFIX}{}",
            URL_SAFE_NO_PAD.encode(miniz_oxide::deflate::compress_to_vec(json.as_bytes(), 9))
        );
        assert_eq!(Scene::from_code(&code).unwrap().sea.level, 1.0, "clamped, as in a file");
    }
}
