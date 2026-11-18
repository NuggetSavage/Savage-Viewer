# Savage Viewer: what changed, version by version

The newest version is at the top. The number is shown in the viewer's title
bar and in About. Each version below names its commit on GitHub (branch
`savage`), which is the exact code of that version, and the build keeps a
copy of each viewer in `C:\dev\savage-viewer-app\versions`.

## 0.3.0 (2026-11-17)

Commit: the one titled "Version 0.3.0" on GitHub.

Added
- Black Dragon and Firestorm graphics presets load in Savage Viewer.
  Preferences > Graphics > Saved graphics profiles has a folder button: it
  opens the presets folder. Put preset files (.xml) there and they show in
  the profiles list under "Preset files from other viewers". Choosing one
  applies every setting Savage Viewer has a control for and says how many
  it applied and how many it has no control for yet (hover the line for
  their names). Savage Viewer only reads these files. "Save as…" keeps the
  result as your own profile.
- What a preset sets today: draw distance, object and tree detail, avatar
  limit and complexity, particle count, shadows on/off, projector shadows,
  shadow resolution and distance, ambient occlusion on/off, screen
  reflections, glow on/off, depth of field on/off and its lens values,
  texture filtering, avatar physics.
- Light strength is not taken from a preset yet, on purpose: Black Dragon
  presets pair a high light strength with their own exposure setting, which
  Savage Viewer cannot read yet. Taking one without the other would burn the
  picture out. Set Light strength by hand for now.
- Not set yet (no control in Savage Viewer): glow strength, warmth and
  width, ambient occlusion size and strength, reflection resolution and
  brightness, shadow softness, tone curve and colour grading, vignette,
  godrays, motion blur, lens flare and the other fine settings.

## 0.2.2 (2026-11-17)

Commit: the one titled "Version 0.2.2" on GitHub.

Added
- Photo mode: Shift + E rolls the camera the opposite way from Shift + Q.
  It works even though E alone is the Jump key.

Changed
- Photo mode: the camera pad (gamepad button on the right bar) now stays
  on screen in the Essential display. The Hidden display still hides it.
- Preferences: a setting's description now shows when the pointer rests
  anywhere on its row, including the slider or switch, not only on the label.

## 0.2.1 (2026-11-16)

Commit: the one titled "Version 0.2.1" on GitHub.

Changed
- Shadow resolution: the shadows now follow the slider while you drag it,
  like Exposure, instead of waiting until you let go. To keep the drag
  smooth, the shadow map is rebuilt about 8 times a second while dragging.

## 0.2.0 (2026-11-16)

Commit: the one titled "Version 0.2.0" on GitHub (it sets `savage/VERSION`).

New
- Shadow resolution (1024 to 6144) and Shadow distance (16 to 256 m)
  sliders, under Shadow quality (Preferences > Graphics > Light). They
  fine tune what the quality step sets, and they are saved with graphics
  profiles. Resolution applies when you let the slider go.
- Photo mode: Q rolls the camera left, Shift + Q rolls it right, 0 puts it
  straight. Hold the key to keep rolling.
- Personal lighting: "Load a sky..." and "Edit and save..." buttons. The
  second opens the sky editor on the sky you see, where Save as asks for a
  name, and closes Personal lighting. The buttons are not shown when
  Personal lighting is opened inside photo mode.
- The title bar shows a version number (0.2.0) instead of a build code.
  A build made from later, not yet numbered changes reads 0.2.0+N.g<code>.
- The build keeps a copy of every version, to go back without rebuilding.

Changed
- Photo mode: Q no longer opens the photo settings menu. Tab does.
- Choosing a Shadow quality step or a graphics level resets the two new
  shadow sliders to that step's values.

Removed
- The "Report a bug" buttons (top bar, Help menu, login screen). The
  crash window still has its own report link, which leads nowhere.
- The "Black Dragon look" button. It was not a true match for Black
  Dragon. Light strength stays.
- The check for Aurora's updates, which this viewer must not follow.

Known limits
- E does not roll the camera, because E is Jump. It will if you unbind it.
- Not seen on screen before release: built and reviewed from the code only.

## 0.1.0 (code of 2026-11-13, built 2026-11-15)

Commit: `b700889`. Its title bar reads 0.0.0+1564.gb700889b2: it was built
before versions had numbers.

- Saved graphics profiles also keep your photo filters and the "Keep on
  in the world" switch.
- Photo mode, key I: the third press gives a clean screen (no dark side
  bands, no frame). The "Show the interface" button appears only when the
  pointer is in the bottom right corner.
- Personal lighting: a dome to place the sun and the moon by hand.

## Earlier builds (no version number)

- 2026-11-10: every menu and label says Savage instead of Aurora; the SVG
  icon; the program file is `savage-viewer.exe`; Light strength slider.
- 2026-11-09: the viewer identifies itself to Second Life as "Savage
  Viewer"; "Keep on in the world" for photo filters; the visible name is
  Savage Viewer.
