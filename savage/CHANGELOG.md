# Savage Viewer: what changed, version by version

The newest version is at the top. The number is shown in the viewer's title
bar and in About. Every version is tagged on GitHub (`v0.2.0`, ...), and
the build keeps a copy of each one in `C:\dev\savage-viewer-app\versions`.

## 0.2.0 (2026-11-16)

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
