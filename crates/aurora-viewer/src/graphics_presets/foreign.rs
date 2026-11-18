//! Savage Viewer: the graphics presets folder.
//!
//! `<settings folder>/graphics-presets` holds the preset files of other
//! viewers the user drops there: Black Dragon's (`user_settings/presets/
//! graphic/*.xml`) and the Legacy Viewer's (Firestorm's presets). They show
//! in Preferences > Graphics with the saved profiles; choosing one applies
//! what this viewer has a control for (`savage_presets`, `saved::
//! apply_foreign`). The viewer only reads these files: it never writes,
//! renames or deletes one.
//!
//! The Legacy Viewer keeps its own presets in `user_settings/presets/
//! graphic` too (LLPresetsManager, indra/newview/llpresetsmanager.cpp).

use std::path::{Path, PathBuf};

/// The most files the list shows: a Black Dragon folder holds under a
/// hundred.
const MAX_FILES: usize = 512;

/// The presets folder (it may not exist yet).
pub fn dir() -> PathBuf {
    crate::settings::config_dir().join("graphics-presets")
}

/// A preset file of the folder.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The name shown: the file's, decoded as Black Dragon wrote it.
    pub name: String,
    pub path: PathBuf,
}

/// The preset files of `dir`, by name. Empty when the folder is missing or
/// cannot be read.
pub fn list(dir: &Path) -> Vec<Entry> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<Entry> = read
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|path| path.extension().is_some_and(|x| x.eq_ignore_ascii_case("xml")) && path.is_file())
        .filter_map(|path| {
            let name = savage_presets::display_name(path.file_stem()?.to_str()?);
            Some(Entry { name, path })
        })
        .collect();
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()).then_with(|| a.path.cmp(&b.path)));
    out.truncate(MAX_FILES);
    out
}

/// Reads a preset file. None when it cannot be read or is no preset; the
/// reason goes to the log.
pub fn load(path: &Path) -> Option<savage_presets::Preset> {
    let too_large = std::fs::metadata(path).is_ok_and(|m| m.len() > savage_presets::MAX_BYTES as u64);
    if too_large {
        log::warn!("graphics preset {} is too large to be one", path.display());
        return None;
    }
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            log::warn!("graphics preset {}: {e}", path.display());
            return None;
        }
    };
    match savage_presets::read(&String::from_utf8_lossy(&bytes)) {
        Ok(preset) => Some(preset),
        Err(e) => {
            log::warn!("graphics preset {}: {e}", path.display());
            None
        }
    }
}

/// Makes the folder if needed and shows it in the file manager.
pub fn open_dir() {
    let dir = dir();
    if let Err(e) = std::fs::create_dir_all(&dir).and_then(|()| crate::logging::open_directory(&dir)) {
        log::warn!("graphics presets folder {}: {e}", dir.display());
    }
}
