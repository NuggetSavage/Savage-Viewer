# Savage Viewer: what differs from upstream

This fork tracks Aurora Viewer. `main` is an untouched copy of upstream;
every change of ours is on `savage` and is listed here, newest first, so a
merge of upstream knows what to keep. Upstream's `TASKS.md` is left alone:
it is rewritten daily there and any line of ours would conflict.

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
- **Not verified here:** the viewer was not run (it needs Windows and a
  graphics card). The owner's test on the grid is the validation.

## 2026-11-09: the viewer's own channel

- **What:** `aurora_net::login::CHANNEL` is `Savage Viewer` (upstream:
  `Aurora Viewer`). It is sent at login, with currency quotes and in the
  viewer statistics.
- **Why it differs:** the Policy on Third-Party Viewers wants one
  identifier per viewer, never shared with another.
- **Files:** `crates/aurora-net/src/login.rs`, `currency.rs` (its test).
- **Not changed yet:** the window title, the logo, the icon, the settings
  folder and the credential store's name still say Aurora Viewer.

## 2026-11-09: three libraries updated

- `ctutils` 0.4.8, `pulldown-cmark` 0.13.6, `rustls-webpki` 0.103.16
  (`Cargo.lock` only), for RUSTSEC-2026-0385, -0433 and -0382.

## 2026-11-09: the owner's setup and build scripts

- `savage/Savage-Setup.cmd`, `savage/savage-setup.ps1`,
  `savage/Savage-Update-and-Build.cmd`, `savage/savage-build.ps1`.
