//! Project links shown on the login screen. Empty = not configured yet: the
//! button is shown disabled and nothing is fetched. The README header carries
//! the same Discord and website links: change them there too.

/// Source code repository (GitHub page).
pub const GITHUB: &str = "https://github.com/NuggetSavage/savage-viewer";
/// Savage Viewer's own: whether the interface offers "Report a bug" (the
/// top bar's button and its switch, the Help menu's line, the login
/// screen's button). Off in the viewer: the fork keeps no issue tracker.
/// On in the tests, which still cover upstream's button.
pub const BUG_REPORTS: bool = cfg!(test);
/// Issue template chooser, opened in the browser by "Report a bug" (login
/// screen, top bar, Help menu) and by the crash window.
pub const NEW_ISSUE: &str = "https://github.com/NuggetSavage/savage-viewer/issues/new/choose";
/// The project's wiki, generated from the guides of `docs/`
/// (`scripts/build-wiki.ps1`): Help › Aurora Viewer wiki.
pub const WIKI: &str = "https://github.com/Aurora-Viewer/Aurora-Viewer/wiki";
/// Community Discord invite.
pub const DISCORD: &str = "https://discord.gg/TK4nbMGmKt";
/// Project website.
pub const WEBSITE: &str = "https://aurora-viewer.org";
/// The privacy policy on the website (Preferences › Privacy › Your data).
pub const PRIVACY_POLICY: &str = "https://aurora-viewer.org/privacy/";
/// News feed: this folder of `main`, hand-edited Markdown files, one
/// `<language>.md` per UI language (`news/` at the repository root). The
/// viewer keeps what it read in its cache as a JSON array of
/// `{"date", "title", "summary", "url"}`.
pub const NEWS_FEED: &str = "https://raw.githubusercontent.com/Aurora-Viewer/Aurora-Viewer/main/news/";
/// Files of the source repository as GitHub serves them unrendered; a branch
/// name and a path follow.
pub const RAW_FILES: &str = "https://raw.githubusercontent.com/Aurora-Viewer/Aurora-Viewer/";
