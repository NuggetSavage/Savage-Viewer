# Savage Viewer: what differs from upstream

This fork tracks Aurora Viewer. `main` is an untouched copy of upstream;
every change of ours is on `savage` and is listed here, newest first, so a
merge of upstream knows what to keep. Upstream's `TASKS.md` is left alone:
it is rewritten daily there and any line of ours would conflict.

## 2026-11-13: profiles hold the photo filters, a clean hidden display, the sun dome

- **Saved graphics profiles hold the look:** a profile now keeps the photo
  filters with it (`graphics_presets::saved::Look`: the stack of looks,
  the settings of the looks in it, and "Keep on in the world"), next to
  what upstream already kept (quality, exposure, tone mapping, depth of
  field, water effects, FSR). A profile saved before this holds no look:
  loading it leaves the filters alone and they never mark it modified
  (`Saved::same_as`). LUT layers are held by file name, as in the
  settings; the sky is not part of a profile.
- **Photo display, key I** (`ui/viewfinder.rs`), different from upstream on
  purpose: Essential also puts the lens and aperture controls away;
  Hidden draws nothing at all, neither the darkened margins outside the
  photo's proportions, nor the frame, nor the "Show the interface" button.
  A message says for 2.5 s that I brings it back.
- **Personal lighting** has the sky editor's dome to place the sun and the
  moon by hand, above their azimuth and elevation sliders
  (`ui/environment.rs`, `env_editor::sky_dome`).
- **Not verified here:** linted with the whole viewer; the viewer's tests
  (the new profile test included) were not run, nothing was seen on
  screen.

## 2026-11-10: Savage in every label, the SVG icon, the Black Dragon controls

- **Name:** the messages of the three catalogues say Savage where they
  said Aurora (feature names included: the Savage bridge, the Savage
  style, the Savage page of Preferences, the updater, crash and bug
  texts, the offline demo), and so do the crash window, the memory
  monitor, the Dev Toolbox report header and the log's first line. Kept
  on purpose, because they name real things: the About line that credits
  Aurora Viewer (the licence asks for it), the "Aurora Team" name tag
  (upstream's developers), the inventory folders `#Aurora` and
  `Aurora Library`, the Blender add-on "Aurora LiveMesh" and its panel,
  the bundled model "Aurora 3D", links, `aurora.log`, the `aurora-viewer`
  command in the help and the `AURORA_*` variables.
- **Guard:** `the_viewer_is_named_savage_viewer` fails on any "Aurora" in
  a message outside those keys and real names.
- **Known mismatch:** the update, wiki and privacy texts say Savage while
  the feed, the wiki and the policy they open are upstream's.
- **Reports:** "Report a bug", the crash window and the GitHub button go to
  this fork (`NuggetSavage/savage-viewer`), never to upstream
  (`links.rs`, `bug_report/github.rs`, `crash/github.rs`). The in-viewer
  direct send still uses upstream's GitHub App identifier and is expected
  to fail on this repository; the browser route works once the fork has
  its issues turned on (they are off: the links lead nowhere until then).
- **Icon:** `assets/branding/savage.ico`, `savage-logo.png`,
  `savage-logo.svg`, `savage-logo-white.svg`, `savage-logo-black.svg`
  (an original "SVG" letter mark, letters from Poppins Bold, OFL). The
  program icon (`build.rs`), the window and notification logo
  (`app.rs`, `LOGO_PNG`) and the top bar's brand mark (`ui/icons.rs`)
  use them. Upstream's files stay in the folder, unused by the viewer.
- **Program file:** `savage/savage-build.ps1` copies the build as
  `savage-viewer.exe` and removes the previous copy named
  `aurora-viewer.exe` from the viewer folder. Cargo's package and the
  build output keep upstream's name. The link-handler repair knows both
  file names (`url_scheme.rs`).
- **Light strength:** `Settings::light_strength` (0.25 to 10, 1 by
  default; Preferences > Graphics > Light > Exposure and glow) multiplies
  the sun or moon colour a frame is lit with (`app.rs`, `sun_color`).
  After Black Dragon's RenderGlobalLightStrength; saved with the graphics
  profiles (`graphics_presets::Effects::light_strength`, 1 for a profile
  written before it). Like the glow it belongs to no level: choosing a
  level keeps it, and it never turns a level into Custom. Held at 1 or
  less while RLVa locks the environment (`frame_light_strength`).
- **Black Dragon look:** a button (Preferences > Graphics > Quality) that
  turns on the finest sun and projector shadows, the contact shadows,
  SSR, HDR, glow, bloom, sharpening at 0.4 and a light strength of 2
  (`graphics_presets::black_dragon_look`). The 2 is a starting value,
  not a measured match.
- **Not done:** a shadow softness slider (the penumbra is a constant of
  `shaders/common.wgsl`), glow width and warmth, an import of Black
  Dragon's preset files, the settings folder and credential names, the
  inventory folder's name, Cargo's crate names.
- **Not verified here:** type-checked and linted with the whole viewer;
  the catalogue tests pass; the viewer's own tests were not run (this
  workspace cannot build them) and nothing was seen on screen.

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
