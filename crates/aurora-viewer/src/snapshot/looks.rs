//! The photo's look (issue #969), in three families that the interface
//! shows as tabs: the Legacy Viewer's photo filters (LLImageFiltersManager,
//! the Filter list of LLFloaterSnapshot, indra/newview, originally LGPL
//! 2.1), Aurora's own looks in sections, and the user's 3D look-up tables
//! (.cube files, `luts`). All are drawn on the GPU by `aurora_render::looks`
//! on the live view of the photo mode and on the photo, at a strength, with
//! a before / after line on the live view.
//!
//! Looks stack: each layer of `Settings::looks` is drawn over the ones
//! before at its own strength, and can be switched off without losing its
//! place. Aurora's looks have settings (`knobs`), kept per look.
//!
//! A look is saved by its id: the Legacy Viewer's file name (`Sepia`),
//! `aurora:<id>` or `lut:<file name>`.

use super::knobs;
use super::luts::LutLibrary;
use aurora_render::looks::{AuroraSection, LookLayer, LookOp, Lut, PhotoLook, aurora_look, aurora_looks, legacy_look, legacy_looks};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Layers of the stack at most.
pub const MAX_LAYERS: usize = 6;

/// One look of the stack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Layer {
    pub id: String,
    /// In percent.
    pub strength: u8,
    /// Drawn (unticked: kept in the stack, not drawn).
    pub on: bool,
}

impl Default for Layer {
    fn default() -> Self {
        Layer {
            id: String::new(),
            strength: 100,
            on: true,
        }
    }
}

impl Layer {
    pub fn new(id: impl Into<String>) -> Layer {
        Layer {
            id: id.into(),
            ..Default::default()
        }
    }
}

/// How a click in the list changes the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// The chosen layer takes the look.
    #[default]
    Replace,
    /// The look goes on top as a new layer.
    Add,
}

/// The stack once `pointed` (Some(None): "No filter") is tried in `mode`
/// over layer `selected`.
pub fn tried(stack: &[Layer], pointed: Option<Option<&str>>, mode: Mode, selected: usize) -> Vec<Layer> {
    let mut stack = stack.to_vec();
    match pointed {
        None => {}
        Some(None) => stack.clear(),
        Some(Some(id)) => {
            choose(&mut stack, id, mode, selected);
        }
    }
    stack
}

/// A look chosen in the list: the chosen layer takes it, or it goes on
/// top; returns the layer it is in.
pub fn choose(stack: &mut Vec<Layer>, id: &str, mode: Mode, selected: usize) -> usize {
    if mode == Mode::Add || stack.is_empty() {
        if stack.len() >= MAX_LAYERS {
            stack.pop();
        }
        stack.push(Layer::new(id));
        stack.len() - 1
    } else {
        let at = selected.min(stack.len() - 1);
        stack[at].id = id.to_owned();
        stack[at].on = true;
        at
    }
}

pub const AURORA: &str = "aurora:";
pub const LUT: &str = "lut:";

/// The tabs of the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Legacy,
    Aurora,
    Luts,
}

impl Tab {
    pub const ALL: [Tab; 3] = [Tab::Legacy, Tab::Aurora, Tab::Luts];

    pub fn label(self) -> &'static str {
        match self {
            Tab::Legacy => aurora_i18n::tr!("photo-looks-legacy"),
            Tab::Aurora => aurora_i18n::tr!("photo-looks-aurora"),
            Tab::Luts => aurora_i18n::tr!("photo-looks-luts"),
        }
    }
}

/// The tab a look is in.
pub fn tab_of(id: &str) -> Tab {
    if id.starts_with(AURORA) {
        Tab::Aurora
    } else if id.starts_with(LUT) {
        Tab::Luts
    } else {
        Tab::Legacy
    }
}

/// The translated name of a Legacy Viewer filter (its strings.xml).
fn legacy_name(file: &str) -> Option<&'static str> {
    Some(match file {
        "Antique" => aurora_i18n::tr!("photo-look-antique"),
        "Autocontrast" => aurora_i18n::tr!("photo-look-autocontrast"),
        "Badtrip" => aurora_i18n::tr!("photo-look-badtrip"),
        "BlackAndWhite" => aurora_i18n::tr!("photo-look-black-and-white"),
        "Blownhighlights" => aurora_i18n::tr!("photo-look-blown-highlights"),
        "Brighten" => aurora_i18n::tr!("photo-look-brighten"),
        "Cartoon" => aurora_i18n::tr!("photo-look-cartoon"),
        "Colors1970" => aurora_i18n::tr!("photo-look-colors-1970"),
        "Darken" => aurora_i18n::tr!("photo-look-darken"),
        "Edges" => aurora_i18n::tr!("photo-look-edges"),
        "Focus" => aurora_i18n::tr!("photo-look-focus"),
        "Heatwave" => aurora_i18n::tr!("photo-look-heatwave"),
        "Intense" => aurora_i18n::tr!("photo-look-intense"),
        "Julesverne" => aurora_i18n::tr!("photo-look-jules-verne"),
        "LensFlare" => aurora_i18n::tr!("photo-look-lens-flare"),
        "Lightleak" => aurora_i18n::tr!("photo-look-light-leak"),
        "Linearize" => aurora_i18n::tr!("photo-look-linearize"),
        "Miniature" => aurora_i18n::tr!("photo-look-miniature"),
        "Negative" => aurora_i18n::tr!("photo-look-negative"),
        "Newspaper" => aurora_i18n::tr!("photo-look-newspaper"),
        "Overcast" => aurora_i18n::tr!("photo-look-overcast"),
        "Posterize" => aurora_i18n::tr!("photo-look-posterize"),
        "Rotatecolors180" => aurora_i18n::tr!("photo-look-color-shift"),
        "Sepia" => aurora_i18n::tr!("photo-look-sepia"),
        "Sharpen" => aurora_i18n::tr!("photo-look-sharpen"),
        "Softfocus" => aurora_i18n::tr!("photo-look-soft-focus"),
        "Spotlight" => aurora_i18n::tr!("photo-look-spotlight"),
        "Thematrix" => aurora_i18n::tr!("photo-look-the-matrix"),
        "Toycamera" => aurora_i18n::tr!("photo-look-toy-camera"),
        "Video" => aurora_i18n::tr!("photo-look-video"),
        _ => return None,
    })
}

/// The name and the description of one of Aurora's looks.
fn aurora_text(id: &str) -> Option<(&'static str, &'static str)> {
    use aurora_i18n::tr;
    Some(match id {
        "fine-grain" => (tr!("photo-look-a-fine-grain"), tr!("photo-look-a-fine-grain-tip")),
        "silver-grain" => (tr!("photo-look-a-silver-grain"), tr!("photo-look-a-silver-grain-tip")),
        "teal-orange" => (tr!("photo-look-a-teal-orange"), tr!("photo-look-a-teal-orange-tip")),
        "bleach-bypass" => (tr!("photo-look-a-bleach-bypass"), tr!("photo-look-a-bleach-bypass-tip")),
        "cross-process" => (tr!("photo-look-a-cross-process"), tr!("photo-look-a-cross-process-tip")),
        "instant" => (tr!("photo-look-a-instant"), tr!("photo-look-a-instant-tip")),
        "chrome" => (tr!("photo-look-a-chrome"), tr!("photo-look-a-chrome-tip")),
        "halation" => (tr!("photo-look-a-halation"), tr!("photo-look-a-halation-tip")),
        "cyanotype" => (tr!("photo-look-a-cyanotype"), tr!("photo-look-a-cyanotype-tip")),
        "infrared" => (tr!("photo-look-a-infrared"), tr!("photo-look-a-infrared-tip")),
        "vivid" => (tr!("photo-look-a-vivid"), tr!("photo-look-a-vivid-tip")),
        "faded" => (tr!("photo-look-a-faded"), tr!("photo-look-a-faded-tip")),
        "keep-red" => (tr!("photo-look-a-keep-red"), tr!("photo-look-a-keep-tip")),
        "keep-yellow" => (tr!("photo-look-a-keep-yellow"), tr!("photo-look-a-keep-tip")),
        "keep-green" => (tr!("photo-look-a-keep-green"), tr!("photo-look-a-keep-tip")),
        "keep-blue" => (tr!("photo-look-a-keep-blue"), tr!("photo-look-a-keep-tip")),
        "duotone" => (tr!("photo-look-a-duotone"), tr!("photo-look-a-duotone-tip")),
        "sunset" => (tr!("photo-look-a-sunset"), tr!("photo-look-a-sunset-tip")),
        "night-vision" => (tr!("photo-look-a-night-vision"), tr!("photo-look-a-night-vision-tip")),
        "chromatic" => (tr!("photo-look-a-chromatic"), tr!("photo-look-a-chromatic-tip")),
        "fisheye" => (tr!("photo-look-a-fisheye"), tr!("photo-look-a-fisheye-tip")),
        "old-lens" => (tr!("photo-look-a-old-lens"), tr!("photo-look-a-old-lens-tip")),
        "scope" => (tr!("photo-look-a-scope"), tr!("photo-look-a-bars-tip")),
        "widescreen" => (tr!("photo-look-a-widescreen"), tr!("photo-look-a-bars-tip")),
        "dreamy" => (tr!("photo-look-a-dreamy"), tr!("photo-look-a-dreamy-tip")),
        "pixel" => (tr!("photo-look-a-pixel"), tr!("photo-look-a-pixel-tip")),
        "vhs" => (tr!("photo-look-a-vhs"), tr!("photo-look-a-vhs-tip")),
        "glitch" => (tr!("photo-look-a-glitch"), tr!("photo-look-a-glitch-tip")),
        "oil-paint" => (tr!("photo-look-a-oil-paint"), tr!("photo-look-a-oil-paint-tip")),
        "sketch" => (tr!("photo-look-a-sketch"), tr!("photo-look-a-sketch-tip")),
        "clarity" => (tr!("photo-look-a-clarity"), tr!("photo-look-a-clarity-tip")),
        "haze" => (tr!("photo-look-a-haze"), tr!("photo-look-a-haze-tip")),
        "golden-haze" => (tr!("photo-look-a-golden-haze"), tr!("photo-look-a-haze-tip")),
        "miniature-3d" => (tr!("photo-look-a-miniature-3d"), tr!("photo-look-a-miniature-3d-tip")),
        "background-blur" => (tr!("photo-look-a-background-blur"), tr!("photo-look-a-background-blur-tip")),
        "subject" => (tr!("photo-look-a-subject"), tr!("photo-look-a-subject-tip")),
        _ => return None,
    })
}

pub fn section_name(section: AuroraSection) -> &'static str {
    match section {
        AuroraSection::Film => aurora_i18n::tr!("photo-looks-film"),
        AuroraSection::Color => aurora_i18n::tr!("photo-looks-color"),
        AuroraSection::Lens => aurora_i18n::tr!("photo-looks-lens"),
        AuroraSection::Creative => aurora_i18n::tr!("photo-looks-creative"),
        AuroraSection::Depth => aurora_i18n::tr!("photo-looks-depth"),
    }
}

/// The name of a look: translated, or a table's file name.
pub fn name(id: &str) -> Option<Cow<'static, str>> {
    if let Some(aurora) = id.strip_prefix(AURORA) {
        aurora_text(aurora).map(|(name, _)| Cow::Borrowed(name))
    } else if let Some(lut) = id.strip_prefix(LUT) {
        Some(Cow::Owned(lut_display(lut).to_owned()))
    } else {
        legacy_name(id).map(Cow::Borrowed)
    }
}

/// A table's name without its extension.
pub fn lut_display(file: &str) -> &str {
    file.strip_suffix(".cube").or_else(|| file.strip_suffix(".CUBE")).unwrap_or(file)
}

/// The name of the chosen look, "No filter" without one or with an unknown
/// one.
pub fn label(id: Option<&str>) -> Cow<'static, str> {
    id.and_then(name).unwrap_or(Cow::Borrowed(aurora_i18n::tr!("photo-look-none")))
}

/// The key a name sorts and is searched by: letters without their
/// accents, any case (the Legacy Viewer's map sorts the bytes, which puts É
/// after Z).
pub fn fold(name: &str) -> String {
    name.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'à' | 'â' | 'ä' => 'a',
            'ç' => 'c',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            other => other,
        })
        .collect()
}

/// One entry of a list: its id, name and description.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub id: String,
    pub name: Cow<'static, str>,
    pub tip: Option<&'static str>,
}

/// The Legacy Viewer's filters, in the order of their names.
pub fn legacy_list() -> Vec<Entry> {
    let mut looks: Vec<Entry> = legacy_looks()
        .iter()
        .filter_map(|(id, _)| {
            Some(Entry {
                id: (*id).to_owned(),
                name: Cow::Borrowed(legacy_name(id)?),
                tip: None,
            })
        })
        .collect();
    looks.sort_by_cached_key(|e| fold(&e.name));
    looks
}

/// Aurora's looks by section.
pub fn aurora_list() -> Vec<(AuroraSection, Vec<Entry>)> {
    let mut sections: Vec<(AuroraSection, Vec<Entry>)> = Vec::new();
    for look in aurora_looks() {
        let Some((name, tip)) = aurora_text(look.id) else { continue };
        let entry = Entry {
            id: format!("{AURORA}{}", look.id),
            name: Cow::Borrowed(name),
            tip: Some(tip),
        };
        match sections.last_mut() {
            Some((section, entries)) if *section == look.section => entries.push(entry),
            _ => sections.push((look.section, vec![entry])),
        }
    }
    sections
}

/// The user's tables.
pub fn lut_list(names: &[String]) -> Vec<Entry> {
    names
        .iter()
        .map(|file| Entry {
            id: format!("{LUT}{file}"),
            name: Cow::Owned(lut_display(file).to_owned()),
            tip: None,
        })
        .collect()
}

/// The entries of every tab whose name or description holds `query`.
pub fn search(query: &str, lut_names: &[String]) -> Vec<Entry> {
    let query = fold(query.trim());
    legacy_list()
        .into_iter()
        .chain(aurora_list().into_iter().flat_map(|(_, entries)| entries))
        .chain(lut_list(lut_names))
        .filter(|e| fold(&e.name).contains(&query) || e.tip.is_some_and(|t| fold(t).contains(&query)))
        .collect()
}

/// What a look draws: its operations, and its table.
#[derive(Clone)]
pub struct Resolved {
    pub ops: Arc<[LookOp]>,
    pub lut: Option<Arc<Lut>>,
}

/// The settings of one of Aurora's looks by id.
pub type Settings = BTreeMap<String, Vec<f32>>;

/// The operations of a look with its settings; a table not read yet gives
/// None (it is being read, on a worker).
pub fn resolve(id: &str, settings: &Settings, luts: &mut LutLibrary) -> Option<Resolved> {
    if let Some(aurora) = id.strip_prefix(AURORA) {
        let look = aurora_look(aurora)?;
        let ops = match settings.get(id) {
            Some(values) => knobs::apply(knobs::knobs(aurora), &look.ops, values).into(),
            None => look.ops.clone(),
        };
        return Some(Resolved { ops, lut: None });
    }
    if let Some(file) = id.strip_prefix(LUT) {
        let lut = luts.table(file)?;
        return Some(Resolved {
            ops: Arc::from([LookOp::Lut]),
            lut: Some(lut),
        });
    }
    Some(Resolved {
        ops: legacy_look(id)?,
        lut: None,
    })
}

/// Where the photo is in a frame drawn and how big it is there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// The photo's top left corner, in pixels of the frame.
    pub origin: [f32; 2],
    /// The photo's size, in its own pixels.
    pub photo: (u32, u32),
    /// Pixels of the frame per pixel of the photo.
    pub scale: f32,
}

/// Which live view draws the looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Live {
    /// The photo window's: the photo's frame, the look pointed at in the
    /// list, the before / after line.
    Photo,
    /// The world's, the photo window closed: the looks kept as they are,
    /// over the whole window (`Settings::looks_always`).
    World,
}

/// The live view that draws the looks this frame. The photo window, open,
/// keeps its own view as it always was; closed, the world takes the looks
/// when `always` is on and the world is shown (`in_world`: not the login
/// screen nor the loading one).
///
/// `env_locked`: RLVa holds the environment (@setenv). The world then goes
/// without its looks: one that brightens or equalises the image would
/// partly undo a forced dark sky for as long as it stays on. Stricter than
/// upstream, whose looks only show while the photo window is open.
pub fn live(panel_open: bool, always: bool, in_world: bool, env_locked: bool) -> Option<Live> {
    if panel_open {
        Some(Live::Photo)
    } else if always && in_world && !env_locked {
        Some(Live::World)
    } else {
        None
    }
}

/// Pixels the world is drawn on per pixel of the window, as the renderer
/// sizes its images (`Renderer::make_targets_now`): RLVa's reduced
/// resolution first (`divisor` above 1, the paused scene's too), then
/// FSR, whose output has the window's size, then the render scale
/// (`render_percent`, kept between 50 and 100).
pub fn drawn_scale(fsr: bool, divisor: u32, render_percent: u32) -> f32 {
    if divisor > 1 {
        1.0 / divisor as f32
    } else if fsr {
        1.0
    } else {
        render_percent.clamp(50, 100) as f32 / 100.0
    }
}

/// The world's look laid over the whole `window` (pixels), as a photo that
/// fills it.
///
/// `density`: pixels of the photo per pixel of the window (2 for a 4K
/// photo seen in a 1080p window), so that what is sized in the photo's
/// pixels (a blur, a grain, scan lines) is on screen what it is on the
/// photo taken; anything that is no number of pixels counts as 1.
/// `drawn`: pixels the world is drawn on per pixel of the window
/// (`drawn_scale`): the look's passes run on that image, and a vignette
/// or a bar placed for the window's size would otherwise sit off centre.
/// A window without a size (it is minimised) still gives a photo of one
/// pixel.
pub fn world_placement(window: (u32, u32), drawn: f32, density: f32) -> Placement {
    let density = if density.is_finite() && density > 0.0 { density } else { 1.0 };
    let side = |pixels: u32| ((pixels as f32 * density).round() as u32).max(1);
    Placement {
        origin: [0.0, 0.0],
        photo: (side(window.0), side(window.1)),
        scale: drawn / density,
    }
}

/// The layers switched on, resolved, with their strength (a table not
/// read yet is left out this frame).
pub fn resolve_stack(stack: &[Layer], settings: &Settings, luts: &mut LutLibrary) -> Vec<(Resolved, u8)> {
    stack
        .iter()
        .filter(|l| l.on)
        .filter_map(|l| Some((resolve(&l.id, settings, luts)?, l.strength)))
        .collect()
}

/// The looks of a frame; `focus`: the focus distance of the depth
/// operations (the photo's depth of field), None for the scene at the
/// middle of the photo. None without a layer.
pub fn frame_look(layers: &[(Resolved, u8)], place: Placement, compare: Option<f32>, hold: bool, focus: Option<f32>) -> Option<PhotoLook> {
    if layers.is_empty() {
        return None;
    }
    Some(PhotoLook {
        layers: layers
            .iter()
            .map(|(look, strength)| LookLayer {
                ops: look.ops.clone(),
                intensity: f32::from((*strength).min(100)) / 100.0,
                lut: look.lut.clone(),
            })
            .collect(),
        compare: compare.map(|c| c.clamp(0.0, 1.0)),
        image: [place.origin[0], place.origin[1], place.photo.0 as f32, place.photo.1 as f32],
        scale: place.scale,
        hold,
        focus,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_photo_window_wins_over_the_world_look() {
        assert_eq!(live(true, false, true, false), Some(Live::Photo));
        assert_eq!(live(true, true, true, false), Some(Live::Photo));
        // the photo window's live view is the same before login (the
        // offline demo, the login screen)
        assert_eq!(live(true, true, false, false), Some(Live::Photo));
    }

    #[test]
    fn the_world_look_needs_the_switch_and_the_world() {
        assert_eq!(live(false, true, true, false), Some(Live::World));
        assert_eq!(live(false, false, true, false), None);
        assert_eq!(live(false, true, false, false), None);
    }

    #[test]
    fn the_world_look_waits_while_rlva_locks_the_environment() {
        assert_eq!(live(false, true, true, true), None);
        // the photo window keeps its live view, as upstream
        assert_eq!(live(true, true, true, true), Some(Live::Photo));
    }

    #[test]
    fn the_world_look_covers_the_window() {
        let place = world_placement((1920, 1080), 1.0, 1.0);
        assert_eq!(
            place,
            Placement {
                origin: [0.0, 0.0],
                photo: (1920, 1080),
                scale: 1.0
            }
        );
        // a minimised window has no size: never a photo of zero pixels
        assert_eq!(world_placement((0, 0), 1.0, 1.0).photo, (1, 1));
    }

    #[test]
    fn the_world_look_follows_the_size_the_world_is_drawn_at() {
        // the 3D at half the window: the same image, on half the pixels
        let place = world_placement((1920, 1080), 0.5, 1.0);
        assert_eq!(place.photo, (1920, 1080));
        assert_eq!(place.scale, 0.5);
        assert_eq!(drawn_scale(false, 1, 50), 0.5);
        assert_eq!(drawn_scale(false, 1, 100), 1.0);
        // the setting is kept between 50 and 100
        assert_eq!(drawn_scale(false, 1, 10), 0.5);
        // FSR draws its output at the window's size
        assert_eq!(drawn_scale(true, 1, 50), 1.0);
        // RLVa's reduced resolution comes before both
        assert_eq!(drawn_scale(true, 4, 100), 0.25);
    }

    #[test]
    fn the_world_look_works_in_the_photos_pixels() {
        // a 4K photo seen in a 1080p window: two photo pixels per pixel
        // of the window, so a blur is on screen what it is on the photo
        let place = world_placement((1920, 1080), 1.0, 2.0);
        assert_eq!(place.photo, (3840, 2160));
        assert_eq!(place.scale, 0.5);
        // a density that is no number of pixels: the window's own
        assert_eq!(world_placement((1920, 1080), 1.0, 0.0).photo, (1920, 1080));
        assert_eq!(world_placement((1920, 1080), 1.0, f32::NAN).photo, (1920, 1080));
    }

    #[test]
    fn every_look_has_a_name() {
        // a missing message gives its key back (the French catalogue has
        // every English key, aurora-i18n's tests)
        let all = legacy_list().into_iter().chain(aurora_list().into_iter().flat_map(|(_, e)| e));
        let mut count = 0;
        for entry in all {
            assert!(!entry.name.starts_with("photo-look"), "{}: {}", entry.id, entry.name);
            assert!(entry.tip.is_none_or(|t| !t.starts_with("photo-look")), "{}", entry.id);
            count += 1;
        }
        assert_eq!(count, 30 + 36);
        assert_eq!(aurora_list().len(), 5);
    }

    #[test]
    fn ids_name_their_tab() {
        assert_eq!(tab_of("Sepia"), Tab::Legacy);
        assert_eq!(tab_of("aurora:vhs"), Tab::Aurora);
        assert_eq!(tab_of("lut:Warm.cube"), Tab::Luts);
        assert_eq!(name("lut:Warm.cube").as_deref(), Some("Warm"));
        assert_eq!(label(Some("Nothing")), aurora_i18n::tr!("photo-look-none"));
    }

    #[test]
    fn names_sort_and_search_without_their_accents() {
        assert!(fold("Éclaircir") < fold("Fuite"));
        assert!(fold("Négatif") > fold("Matrix"));
        let found = search("SEPIA", &["Sépia doux.cube".to_owned()]);
        assert!(found.iter().any(|e| e.id == "Sepia"), "{found:?}");
        assert!(found.iter().any(|e| e.id == "lut:Sépia doux.cube"), "{found:?}");
    }

    #[test]
    fn a_frame_look_takes_its_strength_and_place() {
        let place = Placement {
            origin: [10.0, -4.0],
            photo: (3840, 2160),
            scale: 0.5,
        };
        let mut luts = LutLibrary::new(std::env::temp_dir().join("aurora-no-luts"));
        let settings = Settings::new();
        let mut stack = vec![Layer::new("Sepia"), Layer::new("aurora:vhs"), Layer::new("Nothing")];
        stack[0].strength = 140;
        stack[1].on = false;
        let layers = resolve_stack(&stack, &settings, &mut luts);
        assert_eq!(layers.len(), 1);
        let look = frame_look(&layers, place, Some(1.4), true, Some(3.0)).expect("a look");
        assert_eq!(look.layers[0].intensity, 1.0);
        assert_eq!(look.compare, Some(1.0));
        assert_eq!(look.image, [10.0, -4.0, 3840.0, 2160.0]);
        assert!(look.hold);
        assert_eq!(look.focus, Some(3.0));
        assert!(frame_look(&[], place, None, false, None).is_none());
        assert!(resolve("aurora:nothing", &settings, &mut luts).is_none());
    }

    #[test]
    fn settings_change_an_aurora_look() {
        let mut luts = LutLibrary::new(std::env::temp_dir().join("aurora-no-luts"));
        let mut settings = Settings::new();
        let plain = resolve("aurora:glitch", &settings, &mut luts).expect("glitch");
        settings.insert("aurora:glitch".into(), vec![0.05, 0.02, 9.0]);
        let set = resolve("aurora:glitch", &settings, &mut luts).expect("glitch");
        assert_ne!(plain.ops, set.ops);
        assert_eq!(set.ops[0].param(0), Some(0.05));
    }

    #[test]
    fn clicks_replace_the_chosen_layer_or_add_one() {
        let mut stack = Vec::new();
        assert_eq!(choose(&mut stack, "Sepia", Mode::Replace, 0), 0);
        assert_eq!(choose(&mut stack, "aurora:vhs", Mode::Add, 0), 1);
        assert_eq!(choose(&mut stack, "Negative", Mode::Replace, 0), 0);
        let ids: Vec<&str> = stack.iter().map(|l| l.id.as_str()).collect();
        assert_eq!(ids, ["Negative", "aurora:vhs"]);
        // trying leaves the stack as it is
        let tried = tried(&stack, Some(Some("Sepia")), Mode::Add, 0);
        assert_eq!(tried.len(), 3);
        assert_eq!(stack.len(), 2);
        assert!(super::tried(&stack, Some(None), Mode::Add, 0).is_empty());
        // the stack has a ceiling: the top layer gives way
        for _ in 0..10 {
            choose(&mut stack, "aurora:pixel", Mode::Add, 0);
        }
        assert_eq!(stack.len(), MAX_LAYERS);
    }
}
