# Savage Viewer: what differs from upstream

This fork tracks Aurora Viewer. `main` is an untouched copy of upstream;
every change of ours is on `savage` and is listed here, newest first, so a
merge of upstream knows what to keep. Upstream's `TASKS.md` is left alone:
it is rewritten daily there and any line of ours would conflict.

## 2026-11-09: the name shown is Savage Viewer

- **What:** the window title, the login, loading and About screens, the
  quit and exit prompts, the error titles, the auto-response texts sent to
  other residents, Windows' notification name, the "open with" name and
  the program file's details (ProductName, FileDescription) say Savage
  Viewer. The About line reads "a third-party viewer based on Aurora
  Viewer, not provided or supported by Linden Lab".
- **Files:** the three catalogues of `crates/aurora-i18n/locales/`,
  `ui/about.rs`, `ui/login.rs`, `ui/login/carousel.rs`, `ui/loading.rs`,
  `system_notify.rs`, `url_scheme.rs`, `crates/aurora-viewer/build.rs`.
- **Guard:** `the_viewer_is_named_savage_viewer`
  (`crates/aurora-i18n/tests/catalogs.rs`) fails if a message names
  upstream's viewer outside the list kept on purpose.
- **Still upstream's, on purpose:** the offline demo, the crash and bug
  reports and the updates (upstream's services), the team name tag, the
  wiki and privacy links, `NOTICE.md` and the licence. Feature names that
  say "Aurora" alone (the Aurora bridge, the Aurora style) are unchanged.
- **Not changed:** the program file's name (`aurora-viewer.exe`: the
  owner's build script and her GShade setup point at it), the logo and the
  icon, the settings folder, the photo folder, the names of the saved
  logins in the system's credential store (renaming them would drop the
  saved logins), the log's first line, the two user agents.
- **Not verified here:** type-checked and linted with the whole viewer;
  the catalogue tests pass (10 of 10); not run, not seen on screen, and
  not reviewed by a second reader.

## 2026-11-09: the looks kept on the world view

- **What:** a switch, "Keep on in the world" (photo window > Filters, and
  Preferences > Graphics > Photo). On, the active looks (filters and LUTs)
  stay drawn over the world view once the photo window is closed. Off by
  default.
- **Why it differs:** upstream draws the looks on the photo window's live
  view only (`App::live_look` returned `None` with the window closed).
- **Files:** `crates/aurora-viewer/src/snapshot/looks.rs` (`Live`, `live`,
  `world_placement`), `snapshot/mod.rs` (`Settings::looks_always`),
  `app/snapshot.rs` (`live_look`), `ui/looks_editor.rs`,
  `ui/options/pages/graphics.rs`, the three catalogues of
  `crates/aurora-i18n/locales/` (`photo-looks-always`,
  `photo-looks-always-hint`).
- **Rules kept:** the photo window, open, behaves as before; nothing is
  drawn on the login or loading screens; an empty stack costs nothing; the
  renderer crate is unchanged.
- **Stricter than upstream:** the world goes without its looks while RLVa
  locks the environment (@setenv, `world.eep.rlva_locked`), so a look
  cannot lighten a forced dark sky for as long as it stays on. The photo
  window's own live view is as upstream.
- **Placement:** the look is laid as a photo filling the window, in the
  pixels of the photo that would be taken now (`world_placement`'s
  `density`), on the image the world is really drawn on (`drawn_scale`:
  RLVa's or the paused scene's divisor, FSR, the render scale). Upstream's
  photo-window path still places its look in window pixels whatever the
  render scale; that is upstream's and is left alone.
- **Known limits (independent review, 2026-11-09):**
  - translucent ("glass") window backgrounds show the filtered world behind
    them; text, icons, name tags and worn HUDs are not filtered;
  - a depth look with the depth of field off focuses on what is at the
    centre of the view each frame, so it can pulse as the camera pans;
  - a broken or deleted .cube file is dropped from the world look without
    a message until the photo window is next opened;
  - more than four different LUT layers upload a table every frame;
  - VR was not looked at.
- **Not verified here:** the viewer was not run (it needs Windows and a
  graphics card), and its test binary cannot be built on the 8 GB
  workspace used. The rule functions' tests were run on their source
  extracted by script; `looks_stay_in_the_photo_window_by_default` and
  `theme_guard` have not been run anywhere. The owner's test on the grid
  is the validation.

## 2026-11-09: the viewer's own channel

- **What:** `aurora_net::login::CHANNEL` is `Savage Viewer` (upstream:
  `Aurora Viewer`). It is sent at login, with currency quotes and in the
  viewer statistics.
- **Why it differs:** the Policy on Third-Party Viewers wants one
  identifier per viewer, never shared with another.
- **Files:** `crates/aurora-net/src/login.rs`, `currency.rs` (its test).
- **Not changed yet:** the window title, the logo, the icon, the settings
  folder and the credential store's name still say Aurora Viewer, as do
  the HTTP user agent (`aurora_version::user_agent`) and the media user
  agent (`media/mod.rs`).
- **Version:** the fork has no `v*` tag, so a build reports
  `0.0.0+<commits>.g<commit>`. Tags are for the branding step.

## 2026-11-09: three libraries updated

- `ctutils` 0.4.8, `pulldown-cmark` 0.13.6, `rustls-webpki` 0.103.16
  (`Cargo.lock` only), for RUSTSEC-2026-0385, -0433 and -0382.

## 2026-11-09: the owner's setup and build scripts

- `savage/Savage-Setup.cmd`, `savage/savage-setup.ps1`,
  `savage/Savage-Update-and-Build.cmd`, `savage/savage-build.ps1`.
