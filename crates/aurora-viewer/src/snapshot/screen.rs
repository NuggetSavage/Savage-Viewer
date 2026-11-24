//! Savage Viewer: a photo of what is on the screen.
//!
//! A shader injector (GShade, ReShade) draws its effects on the image the
//! viewer presents, after the viewer has finished with it: the viewer's own
//! photo, drawn apart at its size, never has them, and a presented image
//! cannot be read back. This reads the window's picture from the desktop
//! instead, once the interface has been left out of a few frames
//! (`App::photo_screen_grab`): what the screen shows, effects included, at the
//! window's size and no larger.
//!
//! No equivalent in the Legacy Viewer, whose snapshot reads its own
//! framebuffer (LLViewerWindow::rawSnapshot, indra/newview/llviewerwindow.cpp).

/// The window's client area as the desktop shows it: its size and RGBA
/// pixels, top row first.
#[cfg(windows)]
pub fn grab(window: &winit::window::Window) -> Result<((u32, u32), Vec<u8>), String> {
    use windows_sys::Win32::Foundation::{HWND, POINT, RECT};
    use windows_sys::Win32::Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, ClientToScreen, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS, DeleteDC,
        DeleteObject, GdiFlush, GetDC, ReleaseDC, SRCCOPY, SelectObject,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::GetClientRect;
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let handle = window.window_handle().map_err(|e| e.to_string())?;
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return Err("not a Windows window".into());
    };
    let hwnd = handle.hwnd.get() as HWND;
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut origin = POINT { x: 0, y: 0 };
    // SAFETY: `hwnd` is this process's live window (the handle winit gives
    // while the window exists); both calls only write the two structures.
    if unsafe { GetClientRect(hwnd, &mut rect) } == 0 || unsafe { ClientToScreen(hwnd, &mut origin) } == 0 {
        return Err("the window's place on the screen is unknown".into());
    }
    let (width, height) = (rect.right - rect.left, rect.bottom - rect.top);
    if width <= 0 || height <= 0 || width > 16384 || height > 16384 {
        return Err(format!("a window of {width} x {height}"));
    }
    let mut info = BITMAPINFO::default();
    info.bmiHeader = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width,
        // negative: the first row is the top one
        biHeight: -height,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB,
        biSizeImage: 0,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
        biClrUsed: 0,
        biClrImportant: 0,
    };
    let bytes = width as usize * height as usize * 4;
    // SAFETY: the screen's device context is taken and released here; the
    // memory context and the bitmap are created, used and deleted here, on
    // every path. `bits` points to `bytes` bytes owned by the bitmap (32
    // bits a pixel, `width` x `height`) and is read before it is deleted.
    unsafe {
        let screen = GetDC(std::ptr::null_mut());
        if screen.is_null() {
            return Err("the screen cannot be read".into());
        }
        let memory = CreateCompatibleDC(screen);
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        let bitmap = CreateDIBSection(screen, &info, DIB_RGB_COLORS, &mut bits, std::ptr::null_mut(), 0);
        let mut result = Err("the screen cannot be copied".to_owned());
        if !memory.is_null() && !bitmap.is_null() && !bits.is_null() {
            let before = SelectObject(memory, bitmap);
            if BitBlt(memory, 0, 0, width, height, screen, origin.x, origin.y, SRCCOPY) != 0 {
                GdiFlush();
                let bgra = std::slice::from_raw_parts(bits.cast::<u8>(), bytes);
                result = Ok(((width as u32, height as u32), rgba_of(bgra)));
            }
            SelectObject(memory, before);
        }
        if !bitmap.is_null() {
            DeleteObject(bitmap);
        }
        if !memory.is_null() {
            DeleteDC(memory);
        }
        ReleaseDC(std::ptr::null_mut(), screen);
        result
    }
}

#[cfg(not(windows))]
pub fn grab(_window: &winit::window::Window) -> Result<((u32, u32), Vec<u8>), String> {
    Err("only on Windows".into())
}

/// Whether the screen can be photographed here.
pub fn supported() -> bool {
    cfg!(windows)
}

/// The desktop's pixels (blue, green, red, unused) as opaque RGBA.
#[cfg_attr(not(any(windows, test)), allow(dead_code))]
fn rgba_of(bgra: &[u8]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(bgra.len());
    for px in bgra.chunks_exact(4) {
        rgba.extend_from_slice(&[px[2], px[1], px[0], 255]);
    }
    rgba
}

/// The largest centered part of a `size` picture with the proportions of
/// `wanted` (the photo's): the window shows the photo's frame in its middle.
/// Returns the part's size and pixels; the whole picture when the
/// proportions already match (to the pixel) or nothing fits.
pub fn crop_to(size: (u32, u32), rgba: Vec<u8>, wanted: (u32, u32)) -> ((u32, u32), Vec<u8>) {
    let (w, h) = (u64::from(size.0), u64::from(size.1));
    let (pw, ph) = (u64::from(wanted.0), u64::from(wanted.1));
    let bytes = w.checked_mul(h).and_then(|n| n.checked_mul(4));
    if w == 0 || h == 0 || pw == 0 || ph == 0 || bytes != Some(rgba.len() as u64) {
        return (size, rgba);
    }
    // the part: full height and a narrower width, or full width and a
    // lower height
    let (cw, ch) = if w * ph > h * pw {
        ((h * pw / ph).max(1), h)
    } else {
        (w, (w * ph / pw).max(1))
    };
    if (cw, ch) == (w, h) {
        return (size, rgba);
    }
    let (x0, y0) = ((w - cw) / 2, (h - ch) / 2);
    let mut out = Vec::with_capacity((cw * ch * 4) as usize);
    for y in y0..y0 + ch {
        let start = ((y * w + x0) * 4) as usize;
        out.extend_from_slice(&rgba[start..start + (cw * 4) as usize]);
    }
    ((cw as u32, ch as u32), out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_desktops_pixels_become_opaque_rgba() {
        assert_eq!(rgba_of(&[1, 2, 3, 0, 10, 20, 30, 99]), vec![3, 2, 1, 255, 30, 20, 10, 255]);
        assert!(rgba_of(&[]).is_empty());
    }

    #[test]
    fn the_photos_proportions_are_cut_from_the_middle() {
        // 4 x 2, each pixel its column in red and its row in green
        let px = |x: u8, y: u8| [x, y, 0, 255];
        let wide: Vec<u8> = (0..2).flat_map(|y| (0..4).flat_map(move |x| px(x, y))).collect();
        // a square photo: the two middle columns
        let ((w, h), cut) = crop_to((4, 2), wide.clone(), (100, 100));
        assert_eq!((w, h), (2, 2));
        assert_eq!(cut, [px(1, 0), px(2, 0), px(1, 1), px(2, 1)].concat());
        // the same proportions, at another size: untouched
        assert_eq!(crop_to((4, 2), wide.clone(), (1920, 960)), ((4, 2), wide.clone()));
        // a wider photo than the window: the middle rows
        let tall: Vec<u8> = (0..4).flat_map(|y| (0..2).flat_map(move |x| px(x, y))).collect();
        let ((w, h), cut) = crop_to((2, 4), tall, (2, 2));
        assert_eq!((w, h), (2, 2));
        assert_eq!(cut, [px(0, 1), px(1, 1), px(0, 2), px(1, 2)].concat());
        // nothing to go by, or pixels that do not match the size: untouched
        assert_eq!(crop_to((4, 2), wide.clone(), (0, 5)), ((4, 2), wide.clone()));
        assert_eq!(crop_to((3, 3), wide.clone(), (1, 1)), ((3, 3), wide.clone()));
        assert_eq!(crop_to((u32::MAX, u32::MAX), wide.clone(), (1, 1)), ((u32::MAX, u32::MAX), wide));
    }
}
