//! Savage Viewer: the graphics presets folder.
//!
//! `<settings folder>/graphics-presets` holds one file a saved graphics
//! profile (`<name>.json`, the profile as the settings hold it): Save and
//! Save as… write it, deleting the profile removes it, and a file copied
//! there (a friend's, a backup) shows in the list and becomes a profile when
//! chosen. A profile's file changed outside the viewer (a newer copy put
//! there) is listed as changed, and loads into the profile when chosen:
//! never on its own. The settings keep their own copy, which is what an
//! older build reads and what a profile applies.
//!
//! The folder also takes the preset files of other viewers, `*.xml`: Black
//! Dragon's (`user_settings/presets/graphic`) and the Legacy Viewer's
//! (Firestorm's presets). Choosing one applies what this viewer has a
//! control for (`savage_presets`, `saved::apply_foreign`). The viewer only
//! reads those: it never writes, renames or deletes one.
//!
//! The Legacy Viewer keeps its own presets in `user_settings/presets/
//! graphic` too (LLPresetsManager, indra/newview/llpresetsmanager.cpp).

use std::path::{Path, PathBuf};

/// The most files the list shows: a Black Dragon folder holds under a
/// hundred.
const MAX_FILES: usize = 512;

/// The presets folder (it may not exist yet).
#[cfg(not(test))]
pub fn dir() -> PathBuf {
    crate::settings::config_dir().join("graphics-presets")
}

/// In the tests, a folder of its own for each test (each runs on its own
/// thread): the settings folder is one for all of them, and a file one
/// test wrote would be another's profile.
#[cfg(test)]
pub fn dir() -> PathBuf {
    std::env::temp_dir().join(format!(
        "savage-presets-test-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ))
}

/// Whose file a preset file is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A profile of this viewer (`.json`).
    Own,
    /// Another viewer's preset (`.xml`).
    Other,
}

/// A preset file of the folder.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The name shown: the file's, decoded (`savage_presets::display_name`).
    pub name: String,
    pub path: PathBuf,
    pub kind: Kind,
}

/// The file of the profile called `name`.
pub fn profile_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{}.json", savage_presets::file_stem(name)))
}

/// Writes a profile's file (the folder is made if needed): to a temporary
/// file first, so that a failed write leaves the old file whole.
pub fn write_profile(dir: &Path, name: &str, profile: &super::saved::Saved) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let path = profile_path(dir, name);
    let text = serde_json::to_string_pretty(profile).map_err(std::io::Error::other)?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, &path).inspect_err(|_| {
        // the file is open elsewhere or read-only: no half file is left
        let _ = std::fs::remove_file(&tmp);
    })
}

/// `write_profile`, a failure going to the log: the settings hold the
/// profile too, and the file left behind is listed as changed, never read
/// over the profile on its own.
pub fn save_profile(name: &str, profile: &super::saved::Saved) {
    if let Err(e) = write_profile(&dir(), name, profile) {
        log::warn!("graphics profile {name:?} not written to the presets folder: {e}");
    }
}

/// Reads a profile's file. None when it is missing, too large or no
/// profile; the values are cleaned when the profile is applied
/// (`Saved::apply`), like those of the settings.
pub fn read_profile(path: &Path) -> Option<super::saved::Saved> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() > savage_presets::MAX_BYTES as u64 {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    // every field of a profile has a default, so any JSON object would read
    // as one: a profile's file holds its quality values
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    if !value.get("quality").is_some_and(serde_json::Value::is_object) {
        log::warn!("graphics profile {}: no profile in the file", path.display());
        return None;
    }
    match serde_json::from_value(value) {
        Ok(profile) => Some(profile),
        Err(e) => {
            log::warn!("graphics profile {}: {e}", path.display());
            None
        }
    }
}

/// Removes the file of a profile the user deleted in the viewer.
pub fn remove_profile(name: &str) {
    let path = profile_path(&dir(), name);
    if let Err(e) = std::fs::remove_file(&path)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        log::warn!("graphics profile file {}: {e}", path.display());
    }
}

/// Writes the file of every profile that has none: the profiles saved
/// before the folder existed. Once a run, so that a file the user removes
/// by hand does not come back while the viewer is open.
pub fn export_missing(profiles: &std::collections::BTreeMap<String, super::saved::Saved>) {
    let dir = dir();
    for (name, profile) in profiles {
        if !profile_path(&dir, name).exists()
            && let Err(e) = write_profile(&dir, name, profile)
        {
            log::warn!("graphics profile {name:?} not written to the presets folder: {e}");
        }
    }
}

/// The preset files of `dir`, this viewer's and the others', by name. Empty
/// when the folder is missing or cannot be read.
pub fn list(dir: &Path) -> Vec<Entry> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<Entry> = read
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter_map(|path| {
            let ext = path.extension()?;
            let kind = if ext.eq_ignore_ascii_case("json") {
                Kind::Own
            } else if ext.eq_ignore_ascii_case("xml") {
                Kind::Other
            } else {
                return None;
            };
            if !path.is_file() {
                return None;
            }
            let name = savage_presets::display_name(path.file_stem()?.to_str()?);
            Some(Entry { name, path, kind })
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
