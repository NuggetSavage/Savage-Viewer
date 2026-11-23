//! Savage Viewer's own updates: what a release of the fork is, and whether
//! it is newer than the viewer running.
//!
//! The fork publishes one file a version, `savage-viewer-<version>.exe`,
//! beside its SHA-256 (`.github/workflows/savage-release.yml`). The viewer
//! reads the list of releases, downloads that file, checks it against the
//! hash and replaces its own exe with it: nothing else of the installation
//! folder is touched (a shader injector's files and the older versions live
//! there). Upstream's updater (`aurora-updater`) swaps the whole folder and
//! refuses one holding such files, which is why it is not used.
//!
//! Nothing here opens a connection; the only files touched are those of
//! `install`, the swap of the exe.
//!
//! What the hash is worth: it comes from the same release as the exe, so it
//! catches a file cut short, damaged or swapped on the way, not a release
//! published by someone who took over the repository. The exe is not signed;
//! the trust is in the fork's GitHub account, over HTTPS.

use serde::Deserialize;
use std::path::Path;

/// The exe an update replaces.
pub const EXE: &str = "savage-viewer.exe";
/// What the name of a running exe moved out of the way starts with
/// (`savage-viewer.old-<process id>.exe`); removed at a later start.
pub const OLD_PREFIX: &str = "savage-viewer.old";
/// The folder, beside the exe, of the versions kept for a way back
/// (`savage/savage-build.ps1` keeps its builds there too).
pub const VERSIONS: &str = "versions";

/// The releases of the fork, newest first (GitHub's REST API).
pub const RELEASES: &str = "https://api.github.com/repos/NuggetSavage/Savage-Viewer/releases?per_page=30";

/// Where the files of a release are: nothing is downloaded from another
/// repository.
pub const DOWNLOADS: &str = "https://github.com/NuggetSavage/Savage-Viewer/releases/download/";

/// The most a viewer exe weighs (it is about 180 MiB).
pub const MAX_BYTES: u64 = 1024 * 1024 * 1024;

/// The most the list of releases weighs.
pub const MAX_LIST_BYTES: usize = 4 * 1024 * 1024;

/// A version of the viewer: major, minor, patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(pub u64, pub u64, pub u64);

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

/// The three numbers a version text starts with: "0.4.2", "v0.4.2", and a
/// development build's "0.4.2+3.g1a2b3c4d5" (its base). None for anything
/// else.
pub fn version(text: &str) -> Option<Version> {
    let text = text.strip_prefix('v').unwrap_or(text);
    let base = text.split(['+', '-']).next()?;
    let mut parts = base.split('.');
    let mut number = || {
        let part = parts.next()?;
        (!part.is_empty() && part.len() <= 9 && part.bytes().all(|b| b.is_ascii_digit())).then(|| part.parse().ok())?
    };
    let v = Version(number()?, number()?, number()?);
    parts.next().is_none().then_some(v)
}

/// A published version of the fork.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub version: Version,
    /// `savage-viewer-<version>.exe`.
    pub exe_name: String,
    pub exe_url: String,
    pub sha256_url: String,
    /// The exe's size as GitHub gives it.
    pub size: u64,
}

#[derive(Deserialize)]
struct GithubRelease {
    #[serde(default)]
    tag_name: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    #[serde(default)]
    name: String,
    #[serde(default)]
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

/// Whether a file may be downloaded from `url`: HTTPS, on GitHub's own
/// hosts, with no credentials and no port in it.
pub fn trusted_url(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://") else {
        return false;
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    matches!(
        host,
        "api.github.com" | "github.com" | "release-assets.githubusercontent.com" | "objects.githubusercontent.com"
    ) && !url.chars().any(|c| c.is_control() || c.is_whitespace())
}

/// The newest complete release of GitHub's list: a `v<major>.<minor>.<patch>`
/// tag, neither draft nor pre-release, holding its exe and its hash at
/// addresses that may be downloaded. `Ok(None)`: the fork has published
/// none yet.
pub fn newest(list: &[u8]) -> Result<Option<Release>, String> {
    if list.len() > MAX_LIST_BYTES {
        return Err("the list of releases is too large".into());
    }
    let releases: Vec<GithubRelease> = serde_json::from_slice(list).map_err(|e| format!("the list of releases is not readable: {e}"))?;
    Ok(releases
        .iter()
        .filter(|r| !r.draft && !r.prerelease && r.tag_name.starts_with('v'))
        .filter_map(|r| {
            let v = version(&r.tag_name)?;
            // the tag is the bare version: no build of another kind
            if r.tag_name != format!("v{v}") {
                return None;
            }
            let exe_name = format!("savage-viewer-{v}.exe");
            let exe = r.assets.iter().find(|a| a.name == exe_name)?;
            let sha = r.assets.iter().find(|a| a.name == format!("{exe_name}.sha256"))?;
            let fits = exe.size > 0 && exe.size <= MAX_BYTES;
            let ours = |url: &str| trusted_url(url) && url.starts_with(&format!("{DOWNLOADS}v{v}/"));
            (fits && ours(&exe.browser_download_url) && ours(&sha.browser_download_url)).then(|| Release {
                version: v,
                exe_name,
                exe_url: exe.browser_download_url.clone(),
                sha256_url: sha.browser_download_url.clone(),
                size: exe.size,
            })
        })
        .max_by_key(|r| r.version))
}

/// Whether `release` is offered to a viewer whose version text is
/// `current`: only a higher number. A development build past 0.4.2
/// ("0.4.2+3.g…") is not offered 0.4.2 again; a viewer whose version cannot
/// be read is offered nothing.
pub fn offered(current: &str, release: &Release) -> bool {
    version(current).is_some_and(|now| release.version > now)
}

/// The hash of a `.sha256` file: its first 64 characters, hexadecimal, as
/// `sha256sum` writes them (`<hash>  <file name>`); a byte order mark before
/// them is let through.
pub fn parse_sha256(text: &str) -> Option<[u8; 32]> {
    let text = text.trim_start_matches('\u{feff}').trim_start();
    let hex = text.get(..64)?;
    // the hash ends there: the file name follows a space, or nothing does
    if text[64..].chars().next().is_some_and(|c| !c.is_whitespace()) {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(hex.get(i * 2..i * 2 + 2)?, 16).ok()?;
    }
    Some(out)
}

/// Puts `new` in the place of the running `exe`. The running version is
/// kept in `versions/` first (written under another name, then renamed: a
/// copy cut short is never taken for the kept version). Windows lets a
/// running exe be renamed, so it moves aside to `old` (in the same folder;
/// removed at a later start) and the new one takes its name. If the new one
/// cannot be put there, the old one comes back; an error that says the exe
/// is missing means that did not work either.
pub fn install(exe: &Path, new: &Path, old: &Path, current: &str) -> Result<(), String> {
    let dir = exe.parent().ok_or("the exe has no folder")?;
    let versions = dir.join(VERSIONS);
    std::fs::create_dir_all(&versions).map_err(|e| format!("{}: {e}", versions.display()))?;
    let kept = versions.join(format!("savage-viewer-{current}.exe"));
    if !kept.exists() {
        let part = versions.join(format!("savage-viewer-{current}.exe.part"));
        let copied = std::fs::copy(exe, &part).and_then(|_| std::fs::rename(&part, &kept));
        if let Err(e) = copied {
            let _ = std::fs::remove_file(&part);
            return Err(format!("the running version could not be kept in {}: {e}", versions.display()));
        }
    }
    let _ = std::fs::remove_file(old);
    std::fs::rename(exe, old).map_err(|e| format!("the running exe could not be moved aside: {e}"))?;
    // a scanner may hold a file just written for a moment
    let mut placed = std::fs::rename(new, exe);
    for _ in 0..3 {
        if placed.is_ok() || !new.exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
        placed = std::fs::rename(new, exe);
    }
    placed.map_err(|e| match std::fs::rename(old, exe) {
        Ok(()) => format!("the new exe could not be put in place ({e}); the running one is back"),
        Err(back) => format!(
            "the new exe could not be put in place ({e}) and {} is MISSING: the running one could not be put back ({back}); it is {}",
            exe.display(),
            old.display()
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str, assets: &[(&str, &str, u64)], draft: bool, prerelease: bool) -> String {
        let assets: Vec<String> = assets
            .iter()
            .map(|(name, url, size)| format!(r#"{{"name":"{name}","browser_download_url":"{url}","size":{size}}}"#))
            .collect();
        format!(
            r#"{{"tag_name":"{tag}","draft":{draft},"prerelease":{prerelease},"assets":[{}]}}"#,
            assets.join(",")
        )
    }

    fn full(v: &str) -> String {
        let base = format!("https://github.com/NuggetSavage/Savage-Viewer/releases/download/v{v}/savage-viewer-{v}.exe");
        release(
            &format!("v{v}"),
            &[
                (&format!("savage-viewer-{v}.exe"), &base, 181_000_000),
                (&format!("savage-viewer-{v}.exe.sha256"), &format!("{base}.sha256"), 90),
            ],
            false,
            false,
        )
    }

    #[test]
    fn a_version_is_read_from_its_first_three_numbers() {
        assert_eq!(version("0.4.2"), Some(Version(0, 4, 2)));
        assert_eq!(version("v1.10.0"), Some(Version(1, 10, 0)));
        assert_eq!(version("0.4.2+3.g1a2b3c4d5"), Some(Version(0, 4, 2)), "a development build's base");
        assert_eq!(version("0.4.2-beta.1"), Some(Version(0, 4, 2)));
        for bad in [
            "",
            "0.4",
            "0.4.2.1",
            "a.b.c",
            "0..2",
            "0.4.x",
            "v",
            "0.4.-2",
            "0.4.99999999999999999999",
        ] {
            assert_eq!(version(bad), None, "{bad}");
        }
        assert!(Version(0, 10, 0) > Version(0, 9, 9), "numbers, not text");
    }

    #[test]
    fn the_newest_complete_release_is_the_one_offered() {
        let list = format!("[{},{},{}]", full("0.4.2"), full("0.10.0"), full("0.9.3"));
        let r = newest(list.as_bytes()).expect("a list").expect("a release");
        assert_eq!(r.version, Version(0, 10, 0));
        assert_eq!(r.exe_name, "savage-viewer-0.10.0.exe");
        assert!(r.exe_url.ends_with("/v0.10.0/savage-viewer-0.10.0.exe"));
        assert!(r.sha256_url.ends_with("savage-viewer-0.10.0.exe.sha256"));
        assert_eq!(newest(b"[]"), Ok(None), "nothing published yet");
    }

    #[test]
    fn a_release_that_is_not_a_plain_published_version_is_passed_over() {
        let exe = "https://github.com/NuggetSavage/Savage-Viewer/releases/download/v9.0.0/savage-viewer-9.0.0.exe";
        let assets = [("savage-viewer-9.0.0.exe", exe, 5u64), ("savage-viewer-9.0.0.exe.sha256", exe, 5)];
        let cases = [
            release("v9.0.0", &assets, true, false),
            release("v9.0.0", &assets, false, true),
            release("nightly-9.0.0", &assets, false, false),
            release("v9.0.0-beta.1", &assets, false, false),
            release("media-plugins-9.0.0", &assets, false, false),
            // the hash is missing
            release("v9.0.0", &assets[..1], false, false),
            // the exe is another version's
            release("v9.0.1", &assets, false, false),
            // an empty file, and one past the cap
            release("v9.0.0", &[("savage-viewer-9.0.0.exe", exe, 0), assets[1]], false, false),
            release(
                "v9.0.0",
                &[("savage-viewer-9.0.0.exe", exe, MAX_BYTES + 1), assets[1]],
                false,
                false,
            ),
            // GitHub, another repository
            release(
                "v9.0.0",
                &[
                    (
                        "savage-viewer-9.0.0.exe",
                        "https://github.com/someone/else/releases/download/v9.0.0/savage-viewer-9.0.0.exe",
                        5,
                    ),
                    assets[1],
                ],
                false,
                false,
            ),
            // this repository, another version's folder
            release(
                "v9.0.0",
                &[
                    (
                        "savage-viewer-9.0.0.exe",
                        "https://github.com/NuggetSavage/Savage-Viewer/releases/download/v1.0.0/x.exe",
                        5,
                    ),
                    assets[1],
                ],
                false,
                false,
            ),
            // a number written with a leading zero is not the tag of its version
            release("v9.00.0", &assets, false, false),
            // an address that is not GitHub's
            release(
                "v9.0.0",
                &[
                    ("savage-viewer-9.0.0.exe", "https://example.com/savage-viewer-9.0.0.exe", 5),
                    assets[1],
                ],
                false,
                false,
            ),
        ];
        for case in &cases {
            let list = format!("[{case},{}]", full("0.4.2"));
            let r = newest(list.as_bytes()).expect("a list").expect("the plain one");
            assert_eq!(r.version, Version(0, 4, 2), "{case}");
        }
        assert!(newest(b"{}").is_err(), "not a list");
        assert!(newest(b"not json").is_err());
    }

    #[test]
    fn only_a_higher_version_is_offered() {
        let list = format!("[{}]", full("0.4.2"));
        let r = newest(list.as_bytes()).expect("a list").expect("a release");
        assert!(offered("0.4.1", &r));
        assert!(offered("0.4.1+7.gabcdef123", &r));
        assert!(!offered("0.4.2", &r), "the same version");
        assert!(!offered("0.4.2+3.gabcdef123", &r), "a build past it");
        assert!(!offered("0.5.0", &r), "never back");
        assert!(!offered("unknown", &r), "an unreadable version is offered nothing");
    }

    #[test]
    fn only_githubs_https_addresses_are_downloaded() {
        assert!(trusted_url(
            "https://github.com/NuggetSavage/Savage-Viewer/releases/download/v1.0.0/a.exe"
        ));
        assert!(trusted_url("https://release-assets.githubusercontent.com/x?sig=1"));
        assert!(trusted_url("https://objects.githubusercontent.com/x"));
        assert!(trusted_url(RELEASES));
        for bad in [
            "http://github.com/a.exe",
            "https://github.com.evil.example/a.exe",
            "https://evil.example/github.com/a.exe",
            "https://user:pass@github.com/a.exe",
            "https://github.com:8443/a.exe",
            "https://github.com@evil.example/a.exe",
            "https://github.com /a.exe",
            "file:///C:/a.exe",
            "",
        ] {
            assert!(!trusted_url(bad), "{bad}");
        }
    }

    #[test]
    fn a_hash_file_gives_its_32_bytes() {
        let hex = "00112233445566778899aabbccddeeff00112233445566778899AABBCCDDEEFF";
        let want: [u8; 32] = std::array::from_fn(|i| {
            [
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
            ][i % 16]
        });
        assert_eq!(parse_sha256(&format!("{hex}  savage-viewer-0.4.2.exe\r\n")), Some(want));
        assert_eq!(parse_sha256(hex), Some(want));
        assert_eq!(parse_sha256(&format!("\u{feff}{hex}\n")), Some(want), "a byte order mark");
        assert_eq!(parse_sha256(&hex[..63]), None, "too short");
        assert_eq!(parse_sha256(&format!("{hex}0")), None, "too long to be a SHA-256");
        assert_eq!(parse_sha256(&format!("zz{}", &hex[2..])), None);
        assert_eq!(parse_sha256("<html>Not Found</html>"), None);
        assert_eq!(
            parse_sha256("éééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééé"),
            None,
            "no panic off a character boundary"
        );
    }

    use std::path::PathBuf;

    fn folder(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("savage-update-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a temporary folder");
        dir
    }

    #[test]
    fn an_update_replaces_the_exe_alone_and_keeps_the_old_version() {
        let dir = folder("install");
        let exe = dir.join(EXE);
        std::fs::write(&exe, b"old").expect("write");
        std::fs::write(dir.join("GShade.ini"), b"shader settings").expect("write");
        let new = dir.join("savage-viewer-0.5.0.exe.download");
        std::fs::write(&new, b"new").expect("write");
        let old = dir.join("savage-viewer.old-1.exe");
        std::fs::write(&old, b"left by an earlier update").expect("write");
        install(&exe, &new, &old, "0.4.2+3.gabcdef123").expect("installed");
        assert_eq!(std::fs::read(&exe).expect("read"), b"new");
        assert_eq!(std::fs::read(&old).expect("read"), b"old", "the running exe moved aside");
        let kept = dir.join(VERSIONS).join("savage-viewer-0.4.2+3.gabcdef123.exe");
        assert_eq!(std::fs::read(kept).expect("read"), b"old");
        assert_eq!(std::fs::read_dir(dir.join(VERSIONS)).expect("list").count(), 1, "no part file left");
        assert_eq!(
            std::fs::read(dir.join("GShade.ini")).expect("read"),
            b"shader settings",
            "nothing else is touched"
        );
        assert!(!new.exists(), "the download became the exe");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_version_already_kept_is_not_written_over() {
        let dir = folder("kept");
        let exe = dir.join(EXE);
        std::fs::write(&exe, b"rebuilt 0.4.2").expect("write");
        std::fs::create_dir_all(dir.join(VERSIONS)).expect("folder");
        let kept = dir.join(VERSIONS).join("savage-viewer-0.4.2.exe");
        std::fs::write(&kept, b"first 0.4.2").expect("write");
        let new = dir.join("savage-viewer-0.5.0.exe.download");
        std::fs::write(&new, b"new").expect("write");
        install(&exe, &new, &dir.join("savage-viewer.old-1.exe"), "0.4.2").expect("installed");
        assert_eq!(std::fs::read(&kept).expect("read"), b"first 0.4.2");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_download_that_is_gone_leaves_the_running_exe_in_place() {
        let dir = folder("missing");
        let exe = dir.join(EXE);
        std::fs::write(&exe, b"old").expect("write");
        std::fs::write(dir.join("GShade.ini"), b"shader settings").expect("write");
        let new = dir.join("savage-viewer-0.5.0.exe.download");
        let error = install(&exe, &new, &dir.join("savage-viewer.old-1.exe"), "0.4.2").expect_err("no download");
        assert!(error.contains("the running one is back"), "{error}");
        assert_eq!(std::fs::read(&exe).expect("read"), b"old", "the old exe is back under its name");
        assert_eq!(std::fs::read(dir.join("GShade.ini")).expect("read"), b"shader settings");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
