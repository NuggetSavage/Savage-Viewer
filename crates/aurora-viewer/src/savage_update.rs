//! Savage Viewer: the Update button of the login screen.
//!
//! The fork publishes each version as one file, `savage-viewer-<version>.exe`
//! with its SHA-256 (`.github/workflows/savage-release.yml`; the rules of
//! what a release is are in the `savage-update` crate). At start the viewer
//! reads the list of releases; a higher version shows in the Version panel
//! with an Update button. The button downloads the file beside the exe,
//! checks it against the published hash, keeps the running version in
//! `versions/`, puts the new exe in the place of the running one and starts
//! it. Only the exe changes: the folder also holds what a shader injector
//! put there and the older versions, which is why upstream's updater
//! (`crate::updater`, off in this fork: `UPSTREAM_FEED`) is not used; it
//! swaps the whole folder and refuses one holding other files.
//!
//! Nothing is installed without the click, and a version that is not the
//! newest still signs in. The Legacy Viewer's updater downloads an installer
//! and runs it (LLUpdaterService, indra/viewer_components/updater); this one
//! has no installer to run.
//!
//! Only on Windows, in a release build started as `savage-viewer.exe`: the
//! copy `savage/savage-build.ps1` installs (a build made on the PC, which
//! the next published version replaces like any other) or an earlier
//! update. A build run from `target/`, a development profile or an exe
//! under another name is never replaced.
//!
//! The update is asked for on the login screen and signing in waits for
//! its end (`busy`): a viewer in the world is never closed by it. The new
//! version is started at the very end of `App::shutdown`, once the settings
//! are written, as a restart into VR is.

use crate::ui::news::{LoginInfo, SavageUpdate, VersionState};
use crossbeam_channel::{Receiver, Sender};
use savage_update::{EXE, OLD_PREFIX, Release};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// A download left in the folder is another viewer's, still running, until
/// it is this old.
const STALE_DOWNLOAD: Duration = Duration::from_secs(3600);

/// Where the Version panel leaves the Update button's click for `frame`.
pub fn click_id() -> egui::Id {
    egui::Id::unique("savage_update_click")
}

enum State {
    Idle,
    Checking,
    /// Nothing newer is published.
    Current,
    Offer(Release),
    Downloading {
        release: Release,
        done: Arc<AtomicU64>,
    },
    /// The new exe is in place and this viewer still runs the old one: it
    /// could not close for it. Nothing more is offered.
    Installed(Release),
    /// The check failed (no release), or a download or the installation did
    /// (the release, to try again).
    Failed(Option<Release>),
}

enum Msg {
    Checked(Result<Option<Release>, String>),
    /// The exe was downloaded, checked and put in place, or not.
    Installed(Result<(), String>),
}

pub struct Updater {
    /// The exe to replace; None where the viewer does not update itself.
    exe: Option<PathBuf>,
    state: State,
    started: bool,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
}

/// The running exe when it is one an update may replace.
fn installed_exe() -> Option<PathBuf> {
    if !cfg!(windows) || cfg!(test) || std::env::var_os("AURORA_DEMO").is_some() || env!("AURORA_BUILD_PROFILE") != "release" {
        return None;
    }
    let exe = std::env::current_exe().ok()?;
    exe.file_name()?.to_str()?.eq_ignore_ascii_case(EXE).then_some(exe)
}

/// Removes what earlier updates left in the folder: the exes moved aside
/// (one still running refuses, and stays for a later start) and the
/// downloads that did not end. A recent download is another viewer's at
/// work: it stays.
fn sweep(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let moved_aside = name.starts_with(OLD_PREFIX) && name.ends_with(".exe");
        let download = name.starts_with("savage-viewer-") && name.ends_with(".download");
        let stale = || {
            entry
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|at| at.elapsed().ok())
                .is_some_and(|age| age > STALE_DOWNLOAD)
        };
        if moved_aside || (download && stale()) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

impl Updater {
    pub fn new() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        let exe = installed_exe();
        if let Some(dir) = exe.as_deref().and_then(Path::parent) {
            sweep(dir);
        }
        Self {
            exe,
            state: State::Idle,
            started: false,
            tx,
            rx,
        }
    }

    /// Whether this viewer updates itself (the login screen's check button
    /// is then this module's).
    pub fn enabled(&self) -> bool {
        self.exe.is_some()
    }

    /// Whether an update is being downloaded and installed: signing in
    /// waits, the viewer restarts at its end.
    pub fn busy(&self) -> bool {
        matches!(self.state, State::Downloading { .. })
    }

    /// Asks the fork for its releases.
    pub fn check(&mut self, rt: &tokio::runtime::Handle) {
        if self.exe.is_none() || matches!(self.state, State::Checking | State::Downloading { .. } | State::Installed(_)) {
            return;
        }
        self.state = State::Checking;
        let tx = self.tx.clone();
        rt.spawn(async move {
            let _ = tx.send(Msg::Checked(newest().await));
        });
    }

    fn download(&mut self, rt: &tokio::runtime::Handle, release: Release, exe: PathBuf) {
        let done = Arc::new(AtomicU64::new(0));
        self.state = State::Downloading {
            release: release.clone(),
            done: done.clone(),
        };
        let tx = self.tx.clone();
        rt.spawn(async move {
            let result = fetch_and_install(&release, &exe, &done).await;
            let _ = tx.send(Msg::Installed(result));
        });
    }

    /// Once a frame: takes the answers of the background work and the
    /// button's click, and fills the login screen's Version panel.
    /// `on_login_screen`: neither in the world nor signing in; the click
    /// only counts there. Returns the exe to start once the viewer has
    /// closed: the new version is in place.
    pub fn frame(
        &mut self,
        info: &mut LoginInfo,
        ctx: &egui::Context,
        rt: &tokio::runtime::Handle,
        on_login_screen: bool,
    ) -> Option<PathBuf> {
        let exe = self.exe.clone()?;
        if !self.started {
            self.started = true;
            self.check(rt);
        }
        let mut restart = None;
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Msg::Checked(_) if !matches!(self.state, State::Checking) => {}
                Msg::Checked(Ok(Some(release))) if savage_update::offered(aurora_version::version(), &release) => {
                    log::info!("update: version {} is published", release.version);
                    self.state = State::Offer(release);
                }
                Msg::Checked(Ok(_)) => self.state = State::Current,
                Msg::Checked(Err(e)) => {
                    log::warn!("update: the releases could not be read: {e}");
                    self.state = State::Failed(None);
                }
                Msg::Installed(result) => {
                    let State::Downloading { release, .. } = &self.state else {
                        continue;
                    };
                    let release = release.clone();
                    match result {
                        Ok(()) if on_login_screen => {
                            log::info!("update: version {} is in place; closing to start it", release.version);
                            self.state = State::Installed(release);
                            restart = Some(exe.clone());
                        }
                        Ok(()) => {
                            // signing in waits for a download, so this is
                            // not expected: the session is left alone
                            log::warn!("update: version {} is in place; it starts with the next launch", release.version);
                            self.state = State::Installed(release);
                        }
                        Err(e) => {
                            log::error!("update: version {} not installed: {e}", release.version);
                            self.state = State::Failed(Some(release));
                        }
                    }
                }
            }
        }
        let clicked = ctx.data_mut(|d| d.remove_temp::<bool>(click_id())).unwrap_or(false);
        if clicked
            && on_login_screen
            && let State::Offer(release) | State::Failed(Some(release)) = &self.state
        {
            self.download(rt, release.clone(), exe);
        }
        info.can_check_updates = !matches!(self.state, State::Checking | State::Downloading { .. } | State::Installed(_));
        (info.version, info.savage) = match &self.state {
            State::Idle | State::Checking => (VersionState::Checking, None),
            State::Current => (VersionState::UpToDate, None),
            State::Failed(None) => (VersionState::Failed, None),
            // no address: the panel's own button installs, no page to open
            State::Offer(r) => (available(r), Some(SavageUpdate::Offer)),
            State::Failed(Some(r)) => (available(r), Some(SavageUpdate::Failed)),
            State::Installed(r) => (available(r), Some(SavageUpdate::Installed)),
            State::Downloading { release, done } => {
                ctx.request_repaint_after(Duration::from_millis(100));
                let part = done.load(Ordering::Relaxed) as f32 / release.size.max(1) as f32;
                (available(release), Some(SavageUpdate::Downloading(part.clamp(0.0, 1.0))))
            }
        };
        restart
    }
}

/// Starts the updated viewer, at the end of `App::shutdown`: the settings
/// of the closing one are written, the new one reads them. Port of the
/// restart into VR (`restart_with_vr`, app.rs), with the same arguments
/// carried over.
pub fn start(exe: &Path) {
    let cli = crate::cli::get();
    let mut command = std::process::Command::new(exe);
    if let Some(title) = &cli.title {
        command.arg("--title").arg(title);
    }
    if let Some(fps) = cli.fps_limit {
        command.arg("--fps-limit").arg(fps.to_string());
    }
    if let Some(language) = &cli.language {
        command.arg("--language").arg(language);
    }
    match command.spawn() {
        Ok(_) => log::info!("update: the new version is started"),
        Err(e) => log::warn!("update: the new version is in place, not started ({e}); it starts with the next launch"),
    }
}

fn available(release: &Release) -> VersionState {
    VersionState::Available {
        version: release.version.to_string(),
        url: String::new(),
    }
}

fn client() -> Result<reqwest::Client, String> {
    // (through the proxy the user set, as the viewer's other clients)
    aurora_proxy::http_client(|builder| {
        builder
            .user_agent(aurora_version::user_agent())
            .https_only(true)
            .connect_timeout(Duration::from_secs(15))
            // a connection that sends nothing for a minute is given up: no
            // request waits for ever
            .read_timeout(Duration::from_secs(60))
    })
    .map_err(|e| e.to_string())
}

/// A response's body, at most `max` bytes of it.
async fn body(mut response: reqwest::Response, max: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        if bytes.len() + chunk.len() > max {
            return Err(format!("more than {max} bytes"));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

/// A GET of one of GitHub's addresses, the redirections it follows ending
/// on one of them too (each hop is HTTPS: `https_only`).
async fn get(http: &reqwest::Client, url: &str) -> Result<reqwest::Response, String> {
    if !savage_update::trusted_url(url) {
        return Err(format!("not an address of GitHub: {url}"));
    }
    let request = http.get(url).header(
        reqwest::header::ACCEPT,
        "application/vnd.github+json, application/octet-stream;q=0.9, */*;q=0.8",
    );
    let response = request.send().await.map_err(|e| e.to_string())?;
    if !savage_update::trusted_url(response.url().as_str()) {
        return Err(format!("redirected outside GitHub: {}", response.url()));
    }
    response.error_for_status().map_err(|e| e.to_string())
}

async fn newest() -> Result<Option<Release>, String> {
    let http = client()?;
    let list = tokio::time::timeout(Duration::from_secs(30), async {
        body(get(&http, savage_update::RELEASES).await?, savage_update::MAX_LIST_BYTES).await
    })
    .await
    .map_err(|_| "no answer in 30 s".to_owned())??;
    savage_update::newest(&list)
}

/// Downloads the release's exe beside the running one, checks it against
/// the published size and hash, and puts it in the place of `exe`
/// (`savage_update::install`). The download carries this process's id in
/// its name: two viewers updating at once do not write the same file. It
/// is removed on any failure.
async fn fetch_and_install(release: &Release, exe: &Path, done: &AtomicU64) -> Result<(), String> {
    let dir = exe.parent().ok_or("the exe has no folder")?;
    let http = client()?;
    let sha = body(get(&http, &release.sha256_url).await?, 4096).await?;
    let want = savage_update::parse_sha256(&String::from_utf8_lossy(&sha)).ok_or("the published hash is not readable")?;
    let pid = std::process::id();
    let new = dir.join(format!("{}.{pid}.download", release.exe_name));
    let old = dir.join(format!("{OLD_PREFIX}-{pid}.exe"));
    let mut result = write_checked(&http, release, &new, &want, done).await;
    if result.is_ok() {
        // the copy of the running exe to `versions/` takes a moment: off the
        // network threads
        let (exe, new) = (exe.to_path_buf(), new.clone());
        result = tokio::task::spawn_blocking(move || savage_update::install(&exe, &new, &old, aurora_version::version()))
            .await
            .map_err(|e| e.to_string())
            .and_then(|r| r);
    }
    if result.is_err() {
        let _ = std::fs::remove_file(&new);
    }
    result
}

async fn write_checked(http: &reqwest::Client, release: &Release, path: &Path, want: &[u8; 32], done: &AtomicU64) -> Result<(), String> {
    use std::io::Write;
    let mut response = get(http, &release.exe_url).await?;
    let mut file = std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut total = 0u64;
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        total += chunk.len() as u64;
        if total > release.size {
            return Err("the file is larger than published".into());
        }
        hasher.update(&chunk);
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        done.store(total, Ordering::Relaxed);
    }
    file.sync_all().map_err(|e| e.to_string())?;
    if total != release.size {
        return Err(format!("{total} bytes received, {} published", release.size));
    }
    if hasher.finalize().as_slice() != want {
        return Err("the file does not match its published hash".into());
    }
    Ok(())
}
