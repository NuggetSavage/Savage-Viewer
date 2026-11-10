# Viewer ID and Always-On Looks Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the fork its own viewer ID, then let the photo looks (filters and LUTs) stay on in the world view after the photo window closes.

**Architecture:** The renderer already draws a frame's `look` whenever one is set (`aurora-render/src/renderer.rs`, around line 9137). Only the viewer's `live_look` withholds it while the photo window is closed. A new setting lets `live_look` hand over the saved stack placed on the whole window. No renderer change.

**Tech Stack:** Rust (stable, `rust-toolchain.toml`), egui, Fluent catalogues (`crates/aurora-i18n/locales/{en,fr,ja}.ftl`).

**Spec:** `claude/Savage_Viewer_Feature_1_Always_On_Looks.md` in the owner's project (approved 2026-11-09, steps 0 and 1).

## Global Constraints

- Branch `savage` only. `main` stays an untouched copy of upstream.
- Viewer ID string: exactly `Savage Viewer`.
- Setting name: `snapshot.looks_always`, a `bool`, default `false`.
- Label keys: `photo-looks-always`, `photo-looks-always-hint`, present in `en.ftl`, `fr.ftl` and `ja.ftl` (the catalogue parity test fails otherwise).
- Interface code uses `widgets::switch_labeled` and the options `cx.toggle`, never stock egui widgets or literal colours (`ui/theme_guard.rs`).
- No `unwrap()` / `expect()` on external data. `cargo fmt --check` clean.
- Our notes go in `savage/CHANGES.md`, not `TASKS.md`: upstream rewrites `TASKS.md` daily and every edit there would conflict at each merge.
- No change to the renderer crate, the window title, the logo or the icon.

## Review Focus

- Photo window open with the switch on: the photo's own live view (hovered look, before / after line, photo frame) must behave exactly as before.
- Login screen and loading screen: no look is drawn before the avatar is in the world.
- Empty stack with the switch on: no look, no cost (`frame_look` returns `None`).
- A window of zero size (minimised): no division by zero, no zero-sized photo.
- A settings file written before this change (no `looks_always` key) still loads, with the switch off.

---

### Task 1: Viewer ID

**Files:**
- Modify: `crates/aurora-net/src/login.rs:13`
- Modify: `crates/aurora-net/src/currency.rs:213` (test)

**Interfaces:**
- Produces: `aurora_net::login::CHANNEL == "Savage Viewer"` (already read by `login_params`, `currency::quote_params` and `app/viewer_stats.rs`).

- [ ] **Step 1: Change the test first.** In `quote_request_follows_the_viewer_fields`, assert `p["viewerChannel"].as_str() == "Savage Viewer"`. Add to `login.rs`'s tests:

```rust
#[test]
fn the_channel_is_this_viewers_own() {
    assert_eq!(CHANNEL, "Savage Viewer");
    assert_ne!(CHANNEL, "Aurora Viewer");
}
```

- [ ] **Step 2: Run** `cargo test -p aurora-net channel viewer_fields` and see both fail.
- [ ] **Step 3: Set** `pub const CHANNEL: &str = "Savage Viewer";` and update its doc comment to say why it differs from upstream (TPV policy: one identifier per viewer).
- [ ] **Step 4: Run** `cargo test -p aurora-net`. Expected: all pass.
- [ ] **Step 5: Commit** `Give the fork its own viewer channel`.

### Task 2: The setting and the rule

**Files:**
- Modify: `crates/aurora-viewer/src/snapshot/mod.rs` (`Settings`, its `Default`)
- Modify: `crates/aurora-viewer/src/snapshot/looks.rs` (rule, placement, tests)
- Modify: `crates/aurora-viewer/src/app/snapshot.rs:1178` (`live_look`)

**Interfaces:**
- Produces, in `snapshot::looks`:
  - `pub enum Live { Photo, World }`
  - `pub fn live(panel_open: bool, always: bool, in_world: bool) -> Option<Live>`
  - `pub fn world_placement(window: (u32, u32)) -> Placement`
- Produces: `snapshot::Settings::looks_always: bool`.

- [ ] **Step 1: Write the failing tests** in `looks.rs`:

```rust
#[test]
fn the_photo_window_wins_over_the_world_look() {
    assert_eq!(live(true, false, true), Some(Live::Photo));
    assert_eq!(live(true, true, true), Some(Live::Photo));
    // the photo window's live view is the same before login (demo, login screen)
    assert_eq!(live(true, true, false), Some(Live::Photo));
}

#[test]
fn the_world_look_needs_the_switch_and_the_world() {
    assert_eq!(live(false, true, true), Some(Live::World));
    assert_eq!(live(false, false, true), None);
    assert_eq!(live(false, true, false), None);
}

#[test]
fn the_world_look_covers_the_window() {
    let place = world_placement((1920, 1080));
    assert_eq!(place, Placement { origin: [0.0, 0.0], photo: (1920, 1080), scale: 1.0 });
    // a minimised window has no size: never a photo of zero pixels
    assert_eq!(world_placement((0, 0)).photo, (1, 1));
}
```

and in `snapshot/tests.rs` (or the module's existing settings test):

```rust
#[test]
fn looks_stay_in_the_photo_window_by_default() {
    assert!(!Settings::default().looks_always);
    // a file from before the switch existed
    let old: Settings = serde_json::from_str("{}").expect("an empty settings object loads");
    assert!(!old.looks_always);
}
```

(Use the serialisation format the neighbouring settings tests use if it is not JSON.)

- [ ] **Step 2: Run** `cargo test -p aurora-viewer looks` and see them fail to compile.
- [ ] **Step 3: Implement.** Add the field (doc comment: the looks stay on the world view once the photo window is closed; Savage Viewer's own, no Legacy Viewer setting) and `looks_always: false` in `Default`. Add `Live`, `live` and `world_placement` to `looks.rs`.
- [ ] **Step 4: Wire `live_look`.** Replace the early `return None` with a match on `looks::live(self.panels.snapshot, self.settings.snapshot.looks_always, self.in_world())`:
  - `None` returns `None`.
  - `Some(Live::Photo)` keeps today's body unchanged.
  - `Some(Live::World)` resolves `self.settings.snapshot.looks` as it is (no hovered look), places it with `looks::world_placement(window)`, passes `compare: None`, `hold: false` and the same `focus`.
- [ ] **Step 5: Run** `cargo test -p aurora-viewer looks`. Expected: pass.
- [ ] **Step 6: Commit** `Keep the photo looks on the world view when asked`.

### Task 3: The switch and its labels

**Files:**
- Modify: `crates/aurora-viewer/src/ui/looks_editor.rs` (after the before / after switch, around line 97)
- Modify: `crates/aurora-viewer/src/ui/options/pages/graphics.rs` (beside the `photo-quality` toggle, line 543)
- Modify: `crates/aurora-i18n/locales/en.ftl`, `fr.ftl`, `ja.ftl` (beside `photo-look-compare-tip`)

**Interfaces:**
- Consumes: `snapshot::Settings::looks_always`.

- [ ] **Step 1: Add the labels.**

| Key | en | fr | ja |
|---|---|---|---|
| `photo-looks-always` | Keep on in the world | Garder dans le monde | ワールド表示でも適用 |
| `photo-looks-always-hint` | Your active looks stay on the world view once the photo window is closed. Each look costs some speed. | Vos filtres actifs restent appliqués à la vue du monde une fois la fenêtre photo fermée. Chaque filtre coûte un peu de fluidité. | 写真ウィンドウを閉じても、有効なフィルタをワールド表示に適用し続けます。フィルタごとに描画が少し重くなります。 |

- [ ] **Step 2: Run** `cargo test -p aurora-i18n`. Expected: pass (parity of the three catalogues).
- [ ] **Step 3: Looks editor.** Below the before / after switch, outside the `if opts.compare` block so every host of the editor shows it: `widgets::switch_labeled(ui, p, &mut s.looks_always, tr!("photo-looks-always"))` with the hint as hover text, and `keep_keys(&resp)` like its neighbour. The surrounding `changed(s, before, out)` already saves any difference in `Settings`.
- [ ] **Step 4: Preferences.** `cx.toggle(tr!("photo-looks-always"), tr!("photo-looks-always-hint"), &field!(snapshot.looks_always));` directly after the `photo-quality` rows.
- [ ] **Step 5: Run** `cargo fmt --check`, then `cargo clippy -p aurora-viewer -- -D warnings` and `cargo test -p aurora-viewer theme_guard looks`. Expected: clean. If the viewer crate cannot be built on this Linux workspace, record that in `savage/CHANGES.md` and in the hand-off: the owner's Windows build is then the first compile.
- [ ] **Step 6: Commit** `Add the world looks switch to the photo window and Preferences`.

### Task 4: Record and hand off

**Files:**
- Create: `savage/CHANGES.md`

- [ ] **Step 1: Write** `savage/CHANGES.md`: one dated entry per change above, each with the files touched, why it differs from upstream, and what was and was not verified.
- [ ] **Step 2: Commit and push** `savage`.
- [ ] **Step 3: Independent review** of `git diff origin/main...savage -- crates` by a fresh reviewer against this plan and its Review Focus. Fix what it finds, push again.
- [ ] **Step 4: Owner's test** (grid, her test account): rebuild with `C:\dev\Savage-Update-and-Build.cmd`, log in, then the five checks in the spec's "Done when".
