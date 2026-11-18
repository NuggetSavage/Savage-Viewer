//! Graphics presets written by other viewers, read for Savage Viewer.
//!
//! Two files are understood, both as found in a Black Dragon
//! `user_settings/presets/graphic` folder:
//!
//! - Black Dragon's own: one `<llsd>` document per setting, each a map of
//!   `name` and `value`, one after the other in the file;
//! - the Legacy Viewer's (Firestorm's presets, the viewer's `settings.xml`):
//!   one `<llsd>` map of setting name to a map holding its `Value`.
//!
//! The setting names are the Legacy Viewer's debug settings, which Savage
//! Viewer's own values follow (`graphics_presets::Values` names the same
//! ones). Only the settings Savage Viewer has a control for are taken, in
//! their own unit; every other one is listed as left out, never guessed.
//! Black Dragon's RenderGlobalLightStrength is left out on purpose: its
//! presets pair it with the exposure and tone curve of their own colour
//! pipeline (a strength of 10 beside an exposure of 0.07), which this
//! viewer does not read, so the strength alone would burn the picture out.
//! Nothing here touches a file or a setting of the viewer: text in,
//! [`Preset`] out.

use aurora_llsd::Llsd;

/// What a file gave for the controls Savage Viewer has. `None`: the file
/// says nothing of it, and the viewer keeps its own value.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mapped {
    /// RenderFarClip, metres.
    pub draw_distance: Option<f32>,
    /// RenderVolumeLODFactor.
    pub lod_factor: Option<f32>,
    /// RenderTreeLODFactor.
    pub tree_lod_factor: Option<f32>,
    /// RenderAvatarMaxNonImpostors; a file's 0 (no limit there) is left out.
    pub max_avatars: Option<u32>,
    /// RenderAvatarMaxComplexity, 0 = no limit in both viewers.
    pub max_complexity: Option<u32>,
    /// RenderMaxPartCount.
    pub max_particles: Option<u32>,
    /// RenderShadowDetail above 0.
    pub shadows: Option<bool>,
    /// RenderShadowDetail at 2: the projectors cast shadows too.
    pub projector_shadows: Option<bool>,
    /// RenderShadowResolution, the first of its four maps, in texels
    /// (Black Dragon).
    pub shadow_resolution: Option<u32>,
    /// RenderShadowFarClip, metres (Black Dragon).
    pub shadow_distance: Option<f32>,
    /// RenderDeferredSSAO.
    pub ambient_occlusion: Option<bool>,
    /// RenderScreenSpaceReflections.
    pub ssr: Option<bool>,
    /// RenderGlow.
    pub glow: Option<bool>,
    /// RenderDepthOfField.
    pub depth_of_field: Option<bool>,
    /// CameraFNumber.
    pub f_number: Option<f32>,
    /// CameraFocalLength, millimetres.
    pub focal_length: Option<f32>,
    /// CameraFieldOfView, degrees.
    pub reference_fov: Option<f32>,
    /// RenderAnisotropic.
    pub anisotropic: Option<bool>,
    /// AvatarPhysics.
    pub avatar_physics: Option<bool>,
    /// RenderAvatarPhysicsLODFactor.
    pub avatar_physics_detail: Option<f32>,
}

/// A preset read from a file.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Preset {
    pub mapped: Mapped,
    /// The names of the settings taken: in the file's order for a Black
    /// Dragon preset, by name for a Legacy Viewer one.
    pub taken: Vec<String>,
    /// The names of the settings the file holds and Savage Viewer has no
    /// control for (or whose value could not be read), in the same order.
    pub left_out: Vec<String>,
}

/// Why a file is no preset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Not LLSD XML at all.
    NotXml,
    /// LLSD, with no setting in either shape.
    NoSettings,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Error::NotXml => "not a preset file (no LLSD XML)",
            Error::NoSettings => "no graphics setting in the file",
        })
    }
}

impl std::error::Error for Error {}

/// The most a preset file weighs: Black Dragon's are 12 KiB, a whole
/// `settings.xml` under 1 MiB.
pub const MAX_BYTES: usize = 2 * 1024 * 1024;

/// The settings of a file, by name, in its order: both shapes.
pub fn settings(text: &str) -> Result<Vec<(String, Llsd)>, Error> {
    if text.len() > MAX_BYTES {
        return Err(Error::NotXml);
    }
    let mut out = Vec::new();
    let mut any_doc = false;
    for doc in text.split_inclusive("</llsd>") {
        let Some(start) = doc.find("<llsd") else { continue };
        let Ok(llsd) = aurora_llsd::from_xml(&doc.as_bytes()[start..]) else {
            continue;
        };
        any_doc = true;
        let Some(map) = llsd.as_map() else { continue };
        let name = map.get("name").map(Llsd::as_str).unwrap_or_default();
        if !name.is_empty() && map.contains_key("value") && map.len() == 2 {
            // Black Dragon: one document a setting
            out.push((name.to_owned(), map["value"].clone()));
            continue;
        }
        // the Legacy Viewer: name -> { Comment, Persist, Type, Value }
        for (name, entry) in map {
            if let Some(value) = entry.as_map().and_then(|m| m.get("Value")) {
                out.push((name.clone(), value.clone()));
            }
        }
    }
    match (any_doc, out.is_empty()) {
        (false, _) => Err(Error::NotXml),
        (true, true) => Err(Error::NoSettings),
        (true, false) => Ok(out),
    }
}

/// A number, whatever the file wrote it as: these files hold reals,
/// integers, booleans and numbers in strings ("10.0") for the same kind of
/// setting. None for anything else, and for a number that is not finite.
fn number(v: &Llsd) -> Option<f64> {
    let n = match v {
        Llsd::Real(r) => *r,
        Llsd::Integer(i) => f64::from(*i),
        Llsd::Boolean(b) => f64::from(u8::from(*b)),
        Llsd::String(s) => s.trim().parse().ok()?,
        _ => return None,
    };
    n.is_finite().then_some(n)
}

fn switch(v: &Llsd) -> Option<bool> {
    number(v).map(|n| n != 0.0)
}

fn count(v: &Llsd) -> Option<u32> {
    let n = number(v)?;
    (0.0..=f64::from(u32::MAX)).contains(&n).then(|| n.round() as u32)
}

fn real(v: &Llsd) -> Option<f32> {
    number(v).map(|n| n as f32)
}

/// Reads a preset file's text.
pub fn read(text: &str) -> Result<Preset, Error> {
    let mut p = Preset::default();
    for (name, v) in settings(text)? {
        let m = &mut p.mapped;
        let took = match name.as_str() {
            "RenderFarClip" => set(&mut m.draw_distance, real(&v)),
            "RenderVolumeLODFactor" => set(&mut m.lod_factor, real(&v)),
            "RenderTreeLODFactor" => set(&mut m.tree_lod_factor, real(&v)),
            // 0 is "no limit" there and "no avatar" here: the viewer's own stays
            "RenderAvatarMaxNonImpostors" => set(&mut m.max_avatars, count(&v).filter(|n| *n > 0)),
            "RenderAvatarMaxComplexity" => set(&mut m.max_complexity, count(&v)),
            "RenderMaxPartCount" => set(&mut m.max_particles, count(&v)),
            "RenderShadowDetail" => {
                let detail = count(&v);
                set(&mut m.projector_shadows, detail.map(|d| d >= 2));
                set(&mut m.shadows, detail.map(|d| d >= 1))
            }
            "RenderShadowResolution" => set(&mut m.shadow_resolution, v.as_array().first().and_then(count).filter(|n| *n > 0)),
            "RenderShadowFarClip" => set(&mut m.shadow_distance, real(&v).filter(|d| *d > 0.0)),
            "RenderDeferredSSAO" => set(&mut m.ambient_occlusion, switch(&v)),
            "RenderScreenSpaceReflections" => set(&mut m.ssr, switch(&v)),
            "RenderGlow" => set(&mut m.glow, switch(&v)),
            "RenderDepthOfField" => set(&mut m.depth_of_field, switch(&v)),
            "CameraFNumber" => set(&mut m.f_number, real(&v).filter(|n| *n > 0.0)),
            "CameraFocalLength" => set(&mut m.focal_length, real(&v).filter(|n| *n > 0.0)),
            "CameraFieldOfView" => set(&mut m.reference_fov, real(&v).filter(|n| *n > 0.0)),
            "RenderAnisotropic" => set(&mut m.anisotropic, switch(&v)),
            "AvatarPhysics" => set(&mut m.avatar_physics, switch(&v)),
            "RenderAvatarPhysicsLODFactor" => set(&mut m.avatar_physics_detail, real(&v)),
            _ => false,
        };
        if took { p.taken.push(name) } else { p.left_out.push(name) }
    }
    Ok(p)
}

/// A preset's name from its file's name without the extension: Black
/// Dragon writes the name percent-encoded ("My%20%2D%20Ultra%20Photo").
/// A `%` that starts no valid byte stays as it is.
pub fn display_name(stem: &str) -> String {
    let bytes = stem.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = |b: u8| char::from(b).to_digit(16).map(|d| d as u8);
        match (
            bytes[i],
            bytes.get(i + 1).copied().and_then(hex),
            bytes.get(i + 2).copied().and_then(hex),
        ) {
            (b'%', Some(hi), Some(lo)) => {
                out.push(hi * 16 + lo);
                i += 3;
            }
            (b, ..) => {
                out.push(b);
                i += 1;
            }
        }
    }
    let name = String::from_utf8(out).unwrap_or_else(|_| stem.to_owned());
    let name: String = name.chars().filter(|c| !c.is_control()).collect();
    let name = name.trim();
    if name.is_empty() { stem.to_owned() } else { name.to_owned() }
}

/// The file name, without its extension, of a preset called `name`: the
/// name itself, with every character a file name cannot hold everywhere
/// (and the `%` itself) percent-encoded, the way Black Dragon names its
/// own. [`display_name`] gives the name back. A leading or trailing space
/// or dot is encoded too (Windows drops them), and a name Windows keeps for
/// a device ("CON", "NUL", "COM1"…) has its first letter encoded.
pub fn file_stem(name: &str) -> String {
    const DEVICES: [&str; 22] = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4",
        "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let device = DEVICES
        .iter()
        .any(|d| name.split('.').next().is_some_and(|base| base.trim().eq_ignore_ascii_case(d)));
    let last = name.chars().count().saturating_sub(1);
    let mut out = String::with_capacity(name.len());
    for (i, c) in name.chars().enumerate() {
        let edge = (i == 0 || i == last) && matches!(c, ' ' | '.');
        let unsafe_char = c.is_control() || matches!(c, '%' | '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*');
        if edge || unsafe_char || (device && i == 0) {
            let mut buf = [0u8; 4];
            for b in c.encode_utf8(&mut buf).bytes() {
                out.push_str(&format!("%{b:02X}"));
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn set<T>(slot: &mut Option<T>, value: Option<T>) -> bool {
    match value {
        Some(v) => {
            *slot = Some(v);
            true
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cut of a preset as Black Dragon wrote it: one
    /// document a setting, the numbers under four types.
    const BLACK_DRAGON: &str = concat!(
        "<llsd><map><key>name</key><string>AvatarPhysics</string><key>value</key><integer>1</integer></map></llsd>\n",
        "<llsd><map><key>name</key><string>RenderFarClip</string><key>value</key><real>200</real></map></llsd>\n",
        "<llsd><map><key>name</key><string>RenderVolumeLODFactor</string><key>value</key><string>4</string></map></llsd>\n",
        "<llsd><map><key>name</key><string>RenderAnisotropic</string><key>value</key><boolean>1</boolean></map></llsd>\n",
        "<llsd><map><key>name</key><string>RenderAvatarMaxNonImpostors</string><key>value</key><integer>0</integer></map></llsd>\n",
        "<llsd><map><key>name</key><string>RenderAvatarMaxComplexity</string><key>value</key><integer>0</integer></map></llsd>\n",
        "<llsd><map><key>name</key><string>RenderShadowDetail</string><key>value</key><integer>2</integer></map></llsd>\n",
        "<llsd><map><key>name</key><string>RenderShadowResolution</string><key>value</key><array><real>6144</real><real>6144</real><real>6144</real><real>6144</real></array></map></llsd>\n",
        "<llsd><map><key>name</key><string>RenderShadowFarClip</string><key>value</key><real>208</real></map></llsd>\n",
        "<llsd><map><key>name</key><string>RenderGlobalLightStrength</string><key>value</key><real>10</real></map></llsd>\n",
        "<llsd><map><key>name</key><string>RenderGlowWarmthAmount</string><key>value</key><real>16.3</real></map></llsd>\n",
        "<llsd><map><key>name</key><string>RenderDepthOfField</string><key>value</key><integer>0</integer></map></llsd>\n",
    );

    #[test]
    fn a_black_dragon_preset_gives_its_values_in_savage_units() {
        let p = read(BLACK_DRAGON).expect("a preset");
        let m = &p.mapped;
        assert_eq!(m.avatar_physics, Some(true));
        assert_eq!(m.draw_distance, Some(200.0));
        assert_eq!(m.lod_factor, Some(4.0), "a number in a string is a number");
        assert_eq!(m.anisotropic, Some(true));
        assert_eq!(m.max_complexity, Some(0), "0 is no limit in both viewers");
        assert_eq!(m.shadows, Some(true));
        assert_eq!(m.projector_shadows, Some(true));
        assert_eq!(m.shadow_resolution, Some(6144), "the first of the four maps");
        assert_eq!(m.shadow_distance, Some(208.0));
        assert_eq!(m.depth_of_field, Some(false));
    }

    #[test]
    fn a_setting_with_no_control_is_listed_and_never_guessed() {
        let p = read(BLACK_DRAGON).expect("a preset");
        assert_eq!(
            p.left_out,
            ["RenderAvatarMaxNonImpostors", "RenderGlobalLightStrength", "RenderGlowWarmthAmount"],
            "the light strength goes with an exposure this viewer does not read"
        );
        assert_eq!(p.mapped.max_avatars, None, "no limit there is not zero avatars here");
        assert_eq!(p.taken.len(), 9);
        assert!(p.taken.contains(&"RenderShadowDetail".to_owned()));
    }

    #[test]
    fn a_legacy_viewer_preset_is_read_from_its_value_keys() {
        let text = "<llsd>\n<map>\n<key>RenderFarClip</key>\n<map>\n<key>Comment</key><string>Distance of far clip plane</string>\n\
            <key>Persist</key><integer>1</integer>\n<key>Type</key><string>F32</string>\n<key>Value</key><real>128</real>\n</map>\n\
            <key>RenderShadowDetail</key><map><key>Type</key><string>S32</string><key>Value</key><integer>0</integer></map>\n\
            <key>CameraFNumber</key><map><key>Type</key><string>F32</string><key>Value</key><real>2.8</real></map>\n\
            <key>TextureMemory</key><map><key>Type</key><string>S32</string><key>Value</key><integer>2048</integer></map>\n\
            </map>\n</llsd>\n";
        let p = read(text).expect("a preset");
        assert_eq!(p.mapped.draw_distance, Some(128.0));
        assert_eq!(p.mapped.shadows, Some(false));
        assert_eq!(p.mapped.projector_shadows, Some(false));
        assert_eq!(p.mapped.f_number, Some(2.8));
        assert_eq!(p.left_out, ["TextureMemory"]);
    }

    #[test]
    fn a_file_that_is_no_preset_says_so() {
        assert_eq!(read("hello"), Err(Error::NotXml));
        assert_eq!(read(""), Err(Error::NotXml));
        assert_eq!(
            read("<llsd><map><key>a</key><integer>1</integer></map></llsd>"),
            Err(Error::NoSettings)
        );
        assert_eq!(read("<llsd><array><integer>1</integer></array></llsd>"), Err(Error::NoSettings));
    }

    #[test]
    fn a_file_name_gives_the_name_black_dragon_shows() {
        assert_eq!(display_name("My%20%2D%20Ultra%20Photo"), "My - Ultra Photo");
        assert_eq!(display_name("%2F%2F3005B"), "//3005B");
        assert_eq!(display_name("Dera%27s%20Favorite"), "Dera's Favorite");
        assert_eq!(display_name("SVG-3005B"), "SVG-3005B");
        assert_eq!(display_name("100%"), "100%", "a stray percent stays");
        assert_eq!(display_name("50%zz"), "50%zz");
        assert_eq!(display_name("%ff%fe"), "%ff%fe", "bytes that are no text stay as written");
        assert_eq!(display_name("a%0Ab"), "ab", "no line break in a name");
        assert_eq!(display_name("%20"), "%20", "an empty name falls back to the file's");
    }

    #[test]
    fn a_preset_name_makes_a_file_name_that_gives_it_back() {
        assert_eq!(file_stem("Night shoot"), "Night shoot");
        assert_eq!(file_stem("My - Ultra (v2)"), "My - Ultra (v2)");
        assert_eq!(file_stem("//3005B"), "%2F%2F3005B");
        assert_eq!(file_stem("50% glow: soft?"), "50%25 glow%3A soft%3F");
        assert_eq!(file_stem(" padded. "), "%20padded.%20");
        assert_eq!(file_stem("..."), "%2E.%2E");
        assert_eq!(file_stem("con"), "%63on");
        assert_eq!(file_stem("NUL.backup"), "%4EUL.backup");
        assert_eq!(file_stem("Console"), "Console", "only the device names themselves");
        for name in [
            "Night shoot",
            "//3005B",
            "50% glow: soft?",
            "Été à 100%",
            "a\\b|c*d\"e<f>g",
            "con",
            "日本語",
        ] {
            assert_eq!(display_name(&file_stem(name)), name, "{name}");
        }
    }

    #[test]
    fn a_value_that_is_no_number_is_left_out() {
        let text = "<llsd><map><key>name</key><string>RenderFarClip</string><key>value</key><string>far</string></map></llsd>\
            <llsd><map><key>name</key><string>RenderShadowResolution</string><key>value</key><array /></map></llsd>\
            <llsd><map><key>name</key><string>RenderShadowFarClip</string><key>value</key><real>nan</real></map></llsd>\
            <llsd><map><key>name</key><string>RenderMaxPartCount</string><key>value</key><integer>-5</integer></map></llsd>";
        let p = read(text).expect("a preset");
        assert_eq!(p.mapped, Mapped::default());
        assert_eq!(p.left_out.len(), 4);
    }
}
