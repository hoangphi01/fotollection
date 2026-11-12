// SPDX-License-Identifier: GPL-3.0-only
//! Render settings shared by preview and export, persisted in `settings.json` (DATABASE.md §6).

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::print_size::PrintSize;

/// Interface language (FR-020). English is the default; text printed on photos never follows it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Language {
    #[default]
    En,
    Vi,
    /// Magyar (FR-123), translated from English; see docs/LOCALIZATION.md.
    Hu,
}

impl Language {
    /// The language of an operating system's locale (`hu-HU`, `vi_VN`, `en-GB`…), English
    /// when it is none of ours: the first launch speaks the machine's language (FR-124).
    #[must_use]
    pub fn from_locale(tag: &str) -> Self {
        let primary = tag
            .split(['-', '_', '.', '@'])
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        match primary.as_str() {
            "hu" => Self::Hu,
            "vi" => Self::Vi,
            _ => Self::En,
        }
    }
}

/// Light or dark interface (FR-052). `System` follows the operating system.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

/// How much of the app is on screen (FR-048): every control, two photos per print, or the
/// keep-or-drop loop.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum AppMode {
    Simple,
    Collage,
    #[default]
    Professional,
}

/// Per-machine interface preferences (FR-020, FR-052). Never part of a project file.
// The per-machine switches of settings.json, each its own key as the file stores it: not a
// state machine that an enum would describe better.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UiSettings {
    pub language: Language,
    /// Absent before settings v6: follow the system.
    #[serde(default)]
    pub theme: ThemePreference,
    /// Absent before settings v10: Professional, the app as it was (FR-048).
    #[serde(default)]
    pub mode: AppMode,
    /// Where an export puts its files (FR-009). Absent before settings v12: one folder.
    #[serde(default)]
    pub export_folders: ExportFolders,
    /// The share of the machine's RAM the previews may keep, in percent (FR-091, FR-086).
    /// Absent before settings v15: 25.
    #[serde(default = "default_preview_memory_percent")]
    pub preview_memory_percent: u8,
    /// The disk cache's limit in GB, one of [`CACHE_LIMITS_GB`] (FR-091, FR-086). Absent
    /// before settings v15: 5.
    #[serde(default = "default_cache_limit_gb")]
    pub cache_limit_gb: u8,
    /// Whether every photo's standard preview is made in the background (FR-091, FR-086).
    /// Absent before settings v15: on.
    #[serde(default = "default_prepare_previews")]
    pub prepare_previews: bool,
    /// Whether Pro mode's preview shows the filmstrip under the photo (FR-090). Absent
    /// before it: shown.
    #[serde(default = "default_filmstrip")]
    pub filmstrip: bool,
    /// How Pro mode's photo preview opens (FR-095). Absent before it: a dialog.
    #[serde(default)]
    pub preview_layout: PreviewLayout,
    /// The JPEG quality of an export, 60–100 (FR-092, FR-008). Absent before settings v16:
    /// 90, below a visible difference on a print at a third less than 95 (RESULT §30, owner
    /// 2026-11-06).
    #[serde(default = "default_export_quality")]
    pub export_quality: u8,
    /// Whether *Export* opens the export dialog first (FR-093); false once *Don't ask again*
    /// is ticked. Absent before settings v16: ask.
    #[serde(default = "default_export_ask")]
    pub export_ask: bool,
    /// Several source folders exported each to a subfolder of its own (FR-056). Absent
    /// before settings v16: together.
    #[serde(default)]
    pub export_separate: bool,
    /// The folder exports go to, kept from one launch to the next (FR-093); `None` asks.
    /// Absent before settings v16.
    #[serde(default)]
    pub export_destination: Option<String>,
    /// A subfolder of the destination the export goes into (FR-009, Lightroom's *Put in
    /// subfolder*), one level, cleaned; `None` writes into the destination. Absent before
    /// settings v16.
    #[serde(default)]
    pub export_subfolder: Option<String>,
    /// How exported files are named (FR-104). Absent before settings v16: the photo's name.
    #[serde(default)]
    pub export_naming: crate::naming::ExportNaming,
    /// What an export does with a file of the same name (FR-009). Absent before settings
    /// v16: add a number, as exports always did.
    #[serde(default)]
    pub export_existing: ExistingFiles,
    /// What the exported files say about the photo (FR-010); GPS never. Absent before
    /// settings v16: the camera details, as exports always wrote.
    #[serde(default)]
    pub export_metadata: ExportMetadata,
    /// The largest file an export writes, in KB of 1,000 bytes (FR-105); `None` for no limit.
    /// Absent before settings v16.
    #[serde(default)]
    pub export_limit_kb: Option<u32>,
    /// Whether the app looks for a newer version when it starts (FR-103). Absent before
    /// settings v16: on, as the owner asked (2026-11-06), until the user turns it off.
    #[serde(default = "default_check_updates")]
    pub check_updates_at_start: bool,
    /// The welcome screen was seen on this machine (FR-109). Absent before settings v17: not
    /// yet, so it shows once, after the install or the update.
    #[serde(default)]
    pub welcome_seen: bool,
    /// The welcome screen shows at every start, its last page's tick (FR-109). Absent before
    /// settings v17: off.
    #[serde(default)]
    pub welcome_every_start: bool,
    /// How much the interface moves (FR-100). Absent before settings v18: follow the system.
    #[serde(default)]
    pub motion: MotionPreference,
    /// A small "fotollection.org" on each print (FR-121). Absent before settings v19: on, as
    /// the owner decided (2026-11-10), until the user turns it off.
    #[serde(default = "default_print_credit")]
    pub print_credit: bool,
}

/// What an export does when a file of the same name is already there (FR-009, Lightroom's
/// *Existing files*).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ExistingFiles {
    /// `name_1.jpg`, `name_2.jpg`: nothing is overwritten.
    #[default]
    AddNumber,
    /// The existing file stays and the photo is not written.
    Skip,
    /// The existing file is replaced, once the new one is complete.
    Replace,
}

/// What an exported file says about the photo (FR-010). GPS is never written.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ExportMetadata {
    /// Camera, lens, exposure, capture time and Fotollection's name (`exif_out`'s allowlist).
    #[default]
    Camera,
    /// The capture time and Fotollection's name only.
    DateOnly,
    /// No EXIF at all.
    Nothing,
}

/// File-size limits one can set, in KB (FR-105).
pub const EXPORT_LIMIT_KB_RANGE: std::ops::RangeInclusive<u32> = 100..=50_000;

/// Export qualities one can choose (FR-092).
pub const EXPORT_QUALITY_RANGE: std::ops::RangeInclusive<u8> = 60..=100;
/// The longest destination path kept, in bytes; a longer one is dropped (asked again).
const MAX_DESTINATION: usize = 4096;

/// How much the interface moves (FR-100, `docs/TRANS_ANIMATION.md`): `System` follows the
/// operating system's *Reduce motion*; `Reduce` turns slides and zooms into short fades
/// whatever the system says.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum MotionPreference {
    #[default]
    System,
    Reduce,
}

/// How the photo preview opens (FR-095): as a dialog over the gallery, or filling the app
/// window with the Navigator and History on the left.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PreviewLayout {
    #[default]
    Dialog,
    Window,
}

/// The cache limits one can choose, in GB (FR-086).
pub const CACHE_LIMITS_GB: [u8; 4] = [2, 5, 10, 20];
/// The memory shares one can choose for previews, in percent (FR-086); a saved value
/// between them goes to the nearest, so the window always shows one selected (audit
/// 2026-11-06).
pub const PREVIEW_MEMORY_CHOICES: [u8; 3] = [15, 25, 40];
/// One GB as the window shows it and disks are sold: 10⁹ bytes. The limit, *Auto* and the
/// window use it alike (audit 2026-11-06: *Auto* compared in GiB).
const GB: u64 = 1_000_000_000;

const fn default_preview_memory_percent() -> u8 {
    25
}
const fn default_cache_limit_gb() -> u8 {
    5
}
const fn default_prepare_previews() -> bool {
    true
}
const fn default_filmstrip() -> bool {
    true
}
const fn default_export_quality() -> u8 {
    crate::export::EXPORT_QUALITY
}
const fn default_check_updates() -> bool {
    true
}
const fn default_export_ask() -> bool {
    true
}
const fn default_print_credit() -> bool {
    true
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            language: Language::default(),
            theme: ThemePreference::default(),
            mode: AppMode::default(),
            export_folders: ExportFolders::default(),
            preview_memory_percent: default_preview_memory_percent(),
            cache_limit_gb: default_cache_limit_gb(),
            prepare_previews: default_prepare_previews(),
            filmstrip: default_filmstrip(),
            preview_layout: PreviewLayout::default(),
            export_quality: default_export_quality(),
            export_ask: default_export_ask(),
            export_separate: false,
            export_destination: None,
            export_subfolder: None,
            export_naming: crate::naming::ExportNaming::default(),
            export_existing: ExistingFiles::default(),
            export_metadata: ExportMetadata::default(),
            export_limit_kb: None,
            check_updates_at_start: default_check_updates(),
            welcome_seen: false,
            welcome_every_start: false,
            motion: MotionPreference::default(),
            print_credit: default_print_credit(),
        }
    }
}

impl UiSettings {
    /// Every value within what the app allows, from a file or over IPC: the memory share
    /// and the cache limit each the nearest one offered.
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        self.preview_memory_percent = PREVIEW_MEMORY_CHOICES
            .into_iter()
            .min_by_key(|p| p.abs_diff(self.preview_memory_percent))
            .unwrap_or(25);
        self.cache_limit_gb = CACHE_LIMITS_GB
            .into_iter()
            .min_by_key(|g| g.abs_diff(self.cache_limit_gb))
            .unwrap_or(5);
        self.export_quality = self
            .export_quality
            .clamp(*EXPORT_QUALITY_RANGE.start(), *EXPORT_QUALITY_RANGE.end());
        // A path with a NUL or longer than any system's is not a folder: asked again.
        if self
            .export_destination
            .as_ref()
            .is_some_and(|d| d.is_empty() || d.len() > MAX_DESTINATION || d.contains('\0'))
        {
            self.export_destination = None;
        }
        // Names from the webview never leave the destination (SECURITY.md §2).
        self.export_subfolder = self
            .export_subfolder
            .and_then(|f| crate::naming::sanitize_component(&f));
        self.export_naming = self.export_naming.sanitized();
        self.export_limit_kb = self
            .export_limit_kb
            .map(|kb| kb.clamp(*EXPORT_LIMIT_KB_RANGE.start(), *EXPORT_LIMIT_KB_RANGE.end()));
        self
    }

    /// The disk cache's limit in bytes, in GB of 10⁹ bytes as disks are sold and Finder and
    /// Explorer show them, so "5 GB" reads 5.0 GB in the window.
    pub fn cache_limit_bytes(&self) -> u64 {
        u64::from(self.cache_limit_gb) * GB
    }

    /// The memory and cache settings that suit a machine with `total_ram` bytes of RAM and
    /// `free_disk` bytes free on the cache's disk (FR-086, the *Auto* button): 40 % of RAM
    /// from 32 GB, 25 % from 12 GB, else 15 %; the largest cache limit within a tenth of the
    /// free disk, at least 2 GB; previews prepared unless under 5 GB are free. In GB of 10⁹
    /// bytes, as the limit is.
    #[must_use]
    pub fn auto_for(self, total_ram: u64, free_disk: u64) -> Self {
        let percent = if total_ram >= 32 * GB {
            40
        } else if total_ram >= 12 * GB {
            25
        } else {
            15
        };
        let limit = CACHE_LIMITS_GB
            .into_iter()
            .filter(|g| u64::from(*g) * GB <= free_disk / 10)
            .max()
            .unwrap_or(2);
        Self {
            preview_memory_percent: percent,
            cache_limit_gb: limit,
            prepare_previews: free_disk >= 5 * GB,
            ..self
        }
    }

    /// The memory and cache settings as they come (FR-086, the *Default* button).
    #[must_use]
    pub fn memory_defaults(self) -> Self {
        let d = Self::default();
        Self {
            preview_memory_percent: d.preview_memory_percent,
            cache_limit_gb: d.cache_limit_gb,
            prepare_previews: d.prepare_previews,
            ..self
        }
    }
}

/// Where an export puts its files (FR-009): all in the chosen folder, or a `YYYY-MM-DD`
/// subfolder for each day.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ExportFolders {
    #[default]
    OneFolder,
    ByDay,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RenderSettings {
    /// A built-in `PrintSize.id` (e.g. `"10x15"`) or a `CustomPrintSize.id` (FR-006, FR-043).
    pub print_size: String,
    pub stamp: StampSettings,
    /// User-defined sizes such as 4 × 6 in (FR-043).
    #[serde(default)]
    pub custom_sizes: Vec<CustomPrintSize>,
    /// How place names are written, in the stamp and in the info frame (FR-005).
    #[serde(default)]
    pub place_format: crate::geocode::PlaceFormat,
    /// Prints or a lighter file for sharing (FR-053). Absent before settings v8: prints.
    #[serde(default)]
    pub export_target: crate::share::ExportTarget,
    /// A 3 mm border inside the print size, in every mode and frame style (FR-067). Absent
    /// before settings v10, where FR-065's `frame.border` held it for Crop to fill only.
    #[serde(default)]
    pub border: bool,
    /// The colour of the 3 mm border and of a collage sheet's paper round its photos (FR-067,
    /// FR-035). An info frame keeps its own colour. Absent: white.
    #[serde(default)]
    pub border_color: BorderColor,
    /// Two photos on one print, the Collage mode (FR-035, FR-048). Absent before settings
    /// v10: the defaults.
    #[serde(default)]
    pub collage: CollageSettings,
    /// Shared photos fitted whole into a ratio, the rest filled (FR-075). Absent before
    /// settings v12: off.
    #[serde(default)]
    pub share_fill: ShareFill,
    /// The colour look of every photo that has none of its own (FR-079). Absent before
    /// settings v14: none.
    #[serde(default)]
    pub look: crate::looks::LookSettings,
}

/// A shared photo fitted whole into a frame of a set ratio, the rest filled with one colour,
/// as for a square feed (FR-075). Share exports only; prints keep their print size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ShareFill {
    #[serde(default)]
    pub enabled: bool,
    /// The frame's width and height ratio, `[1, 1]` for a square.
    #[serde(default = "default_fill_ratio")]
    pub ratio: [u16; 2],
    /// The fill's colour, sRGB.
    #[serde(default = "default_fill_color")]
    pub color: [u8; 3],
}

const fn default_fill_ratio() -> [u16; 2] {
    [1, 1]
}

const fn default_fill_color() -> [u8; 3] {
    [255, 255, 255]
}

/// Each side of a fill ratio, and the most one side may be of the other.
pub const FILL_RATIO_SIDE: std::ops::RangeInclusive<u16> = 1..=100;
pub const FILL_RATIO_MAX: u16 = 4;

impl Default for ShareFill {
    fn default() -> Self {
        Self {
            enabled: false,
            ratio: default_fill_ratio(),
            color: default_fill_color(),
        }
    }
}

impl ShareFill {
    /// The fill with its ratio kept to whole sides of 1–100, at most 4:1 either way (files
    /// and IPC are not trusted).
    #[must_use]
    pub fn sanitized(self) -> Self {
        let side = |v: u16| v.clamp(*FILL_RATIO_SIDE.start(), *FILL_RATIO_SIDE.end());
        let [mut w, mut h] = self.ratio.map(side);
        w = w.min(h * FILL_RATIO_MAX);
        h = h.min(w * FILL_RATIO_MAX);
        Self {
            ratio: [w, h],
            ..self
        }
    }
}

/// The colour of the paper left round a photo: its 3 mm border, a sheet's margins (FR-067).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum BorderColor {
    #[default]
    White,
    Black,
}

impl BorderColor {
    /// The paper's pixel colour, the same white and black as the info frame's.
    pub const fn rgb(self) -> [u8; 3] {
        match self {
            Self::White => crate::plan::MAT_WHITE,
            Self::Black => [13, 13, 13],
        }
    }
}

/// The ratio of the photo area in a collage cell (FR-035).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum CellRatio {
    /// Each photo's own ratio.
    #[default]
    Photo,
    /// The sheet's ratio: 2:3 on a 10 × 15.
    Sheet,
    FourThree,
    Square,
}

/// A photo size typed by the user for collage cells, in whole millimetres.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CellSize {
    pub short_mm: u32,
    pub long_mm: u32,
}

/// How two photos share one print (FR-035).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CollageSettings {
    #[serde(default)]
    pub ratio: CellRatio,
    /// White kept round the photo in its cell, on every side.
    #[serde(default = "default_collage_margin")]
    pub margin_mm: u32,
    /// The size typed by the user; `None` uses the suggested size.
    #[serde(default)]
    pub size: Option<CellSize>,
    /// With the ratio locked, a typed size keeps the chosen ratio: only its short edge counts.
    #[serde(default = "yes")]
    pub lock_ratio: bool,
    /// A thin line between the cells, to cut the print in two.
    #[serde(default = "yes")]
    pub cut_line: bool,
    /// Each photo fills its whole cell, with no white: the two meet edge to edge, and the
    /// user chooses what each keeps with its crop. Ratio, size and margin do not apply.
    #[serde(default)]
    pub fill: bool,
    /// One photo and a postcard back instead of two photos (FR-073).
    #[serde(default)]
    pub postcard: PostcardSettings,
}

/// Which way a postcard sheet is held: landscape puts the photo on the left half, portrait
/// on the bottom half; the stamp box is always at the top right (FR-073).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PostcardOrientation {
    #[default]
    Landscape,
    Portrait,
}

/// The bundled faces a postcard's text can be typed in (FR-073, FR-076).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PostcardFace {
    /// Xanh Mono, with its own italic.
    #[default]
    Typewriter,
    /// Patrick Hand; italic is an oblique.
    Handwriting,
}

/// The ink of a postcard's typed text (FR-073). `Auto` is near-black on white paper and
/// near-white on black; the others print the same on both.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PostcardInk {
    #[default]
    Auto,
    /// The blue of a ballpoint pen.
    Blue,
    Sepia,
    Red,
}

impl PostcardInk {
    /// The colour on this paper.
    pub fn rgb(self, paper: BorderColor) -> [u8; 3] {
        match (self, paper) {
            (Self::Auto, BorderColor::White) => [35, 35, 35],
            (Self::Auto, BorderColor::Black) => [235, 235, 235],
            (Self::Blue, _) => [28, 62, 150],
            (Self::Sepia, _) => [112, 66, 20],
            (Self::Red, _) => [176, 32, 32],
        }
    }
}

/// Bold, italic and underline for one part of a postcard's text (FR-073).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TextStyle {
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub underline: bool,
}

/// The postcard half of a collage sheet (FR-073). The text on each card is the photo's own
/// (`PhotoEdits::postcard`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PostcardSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub orientation: PostcardOrientation,
    /// Writing lines in the text box, `POSTCARD_LINES`.
    #[serde(default = "default_postcard_lines")]
    pub lines: u8,
    /// A user font for the typed text (FR-042); `None` uses the bundled face `face`.
    #[serde(default)]
    pub font: Option<u32>,
    /// The bundled face when no user font is chosen: the typewriter or the handwriting one
    /// (FR-076). Absent before settings v13: the typewriter.
    #[serde(default)]
    pub face: PostcardFace,
    /// A small line under the text box on every card ("Photo © …"); empty prints nothing.
    #[serde(default)]
    pub credit: String,
    /// The typed text's ink.
    #[serde(default)]
    pub ink: PostcardInk,
    /// The typed text's size, the date stamp's three steps (`postcard_text_mm`).
    #[serde(default)]
    pub text_size: StampSize,
    /// How the message and the address are set.
    #[serde(default)]
    pub message_style: TextStyle,
    #[serde(default)]
    pub address_style: TextStyle,
    /// Made to be posted (FR-073, owner 2026-11-12): the post's coding zone is kept free on
    /// the whole card, the photo too: 15 mm at the foot of a landscape card, along the right
    /// edge of a portrait one, marked by a thin line and "postal use" in seven languages.
    /// Absent before settings v19: on.
    #[serde(default = "default_postal_use")]
    pub postal_use: bool,
}

const fn default_postal_use() -> bool {
    true
}

/// The typed text's em in millimetres for each size, about 10, 12 and 15 pt: what a
/// typewriter or a hand writes on a card. A size the lines cannot hold is made smaller.
pub fn postcard_text_mm(size: StampSize) -> f32 {
    match size {
        StampSize::Small => 3.4,
        StampSize::Medium => 4.2,
        StampSize::Large => 5.2,
    }
}

/// How many writing lines a postcard may have.
pub const POSTCARD_LINES: std::ops::RangeInclusive<u8> = 3..=4;

fn default_postcard_lines() -> u8 {
    4
}

impl Default for PostcardSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            orientation: PostcardOrientation::Landscape,
            lines: default_postcard_lines(),
            font: None,
            face: PostcardFace::Typewriter,
            credit: String::new(),
            ink: PostcardInk::Auto,
            text_size: StampSize::Medium,
            message_style: TextStyle::default(),
            address_style: TextStyle::default(),
            postal_use: true,
        }
    }
}

/// The largest collage margin, in millimetres.
pub const MAX_COLLAGE_MARGIN_MM: u32 = 20;
/// Edges a typed collage size may have, in millimetres.
pub const COLLAGE_EDGE_MM: std::ops::RangeInclusive<u32> = 10..=1000;

fn default_collage_margin() -> u32 {
    5
}

fn yes() -> bool {
    true
}

impl Default for CollageSettings {
    fn default() -> Self {
        Self {
            ratio: CellRatio::Photo,
            margin_mm: default_collage_margin(),
            size: None,
            lock_ratio: true,
            cut_line: true,
            fill: false,
            postcard: PostcardSettings::default(),
        }
    }
}

impl CollageSettings {
    /// Brings values from the webview or a file within range (SECURITY.md §2).
    #[must_use]
    pub fn sanitized(self) -> Self {
        let edge = |v: u32| v.clamp(*COLLAGE_EDGE_MM.start(), *COLLAGE_EDGE_MM.end());
        Self {
            margin_mm: self.margin_mm.min(MAX_COLLAGE_MARGIN_MM),
            postcard: PostcardSettings {
                lines: self
                    .postcard
                    .lines
                    .clamp(*POSTCARD_LINES.start(), *POSTCARD_LINES.end()),
                credit: sanitize_caption(&self.postcard.credit),
                ..self.postcard.clone()
            },
            size: self.size.map(|s| {
                let (a, b) = (edge(s.short_mm), edge(s.long_mm));
                CellSize {
                    short_mm: a.min(b),
                    long_mm: a.max(b),
                }
            }),
            ..self
        }
    }
}

/// A saved custom print size (FR-043). Stored in millimetres, short edge first.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CustomPrintSize {
    /// `custom-<n>`, unique.
    pub id: String,
    pub name: String,
    pub short_mm: u32,
    pub long_mm: u32,
    /// How the user entered it, so labels show `4 × 6 in` rather than `10.2 × 15.2 cm`.
    #[serde(default)]
    pub unit: LengthUnit,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LengthUnit {
    #[default]
    Cm,
    In,
}

pub const MAX_CUSTOM_SIZES: usize = 30;
/// Allowed edge range for custom sizes, in mm (2 cm – 1 m).
pub const CUSTOM_EDGE_MM: std::ops::RangeInclusive<u32> = 20..=1000;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StampSettings {
    pub enabled: bool,
    pub format: DateFormat,
    pub corner: Corner,
    pub color: StampColor,
    /// The place on the same line as the date (FR-005). The stamp is always set in the
    /// bundled LCD font; user fonts are for the info frame only (FR-042). Settings v4 and
    /// older had a `font` here, which is ignored.
    #[serde(default)]
    pub place: bool,
    /// A line of the user's own text under the date (FR-036), shown in plain upper-case
    /// letters; empty for none.
    #[serde(default)]
    pub caption: String,
    /// Small, medium or large text (FR-004). Absent before settings v10 and project v9:
    /// medium, the size every stamp had.
    #[serde(default)]
    pub size: StampSize,
}

/// The date stamp's text size on the print (FR-004).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum StampSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl StampSize {
    /// The text's em height as a share of the print's short edge: 2.8, 3.6 or 4.8 mm on a
    /// 10 cm print, more on a larger one, half on each photo of a collage sheet (FR-004).
    pub fn of_short_edge(self) -> f32 {
        match self {
            Self::Small => 0.028,
            Self::Medium => 0.036,
            Self::Large => 0.048,
        }
    }
}

/// Longest stamp caption, in characters.
pub const MAX_CAPTION_CHARS: usize = 80;

/// A caption within limits: no control characters, 80 characters.
pub fn sanitize_caption(caption: &str) -> String {
    caption
        .chars()
        .filter(|c| !c.is_control())
        .take(MAX_CAPTION_CHARS)
        .collect()
}

/// The two parts of a postcard's text (FR-073).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PostcardPart {
    Message,
    Address,
}

impl PostcardPart {
    /// The most characters and lines this part takes: the card has 3 or 4 lines for the
    /// message and 3 for the address, 4 when it needs them; more is kept but cut on the
    /// print, and the card shows a warning.
    pub fn limits(self) -> (usize, usize) {
        match self {
            Self::Message => (400, 8),
            Self::Address => (200, 4),
        }
    }
}

/// A postcard part within limits: line breaks kept, other control characters dropped, at
/// most its characters and lines.
pub fn sanitize_postcard(part: PostcardPart, text: &str) -> String {
    let (chars, lines) = part.limits();
    let kept: String = text
        .chars()
        .filter(|c| *c == '\n' || !c.is_control())
        .take(chars)
        .collect();
    kept.split('\n').take(lines).collect::<Vec<_>>().join("\n")
}

/// Checks a postcard part coming over IPC (SECURITY.md §2).
pub fn validate_postcard(part: PostcardPart, text: &str) -> bool {
    let (chars, lines) = part.limits();
    text.chars().count() <= chars
        && text.split('\n').count() <= lines
        && !text.chars().any(|c| c != '\n' && c.is_control())
}

/// How many addresses a machine remembers for the next postcards (FR-077).
pub const MAX_RECENT_ADDRESSES: usize = 8;

/// Puts `address` first in `recent`, once, and keeps the newest few: the addresses typed on
/// this machine's postcards, offered on the next card (FR-077). Blank or invalid ones are
/// not kept, nor the start of it already kept: a draft saved while it was typed.
pub fn remember_address(recent: &mut Vec<String>, address: &str) {
    let address = address.trim();
    if address.is_empty() || !validate_postcard(PostcardPart::Address, address) {
        return;
    }
    recent.retain(|a| a != address && !address.starts_with(a.as_str()));
    recent.insert(0, address.to_owned());
    recent.truncate(MAX_RECENT_ADDRESSES);
}

/// The remembered addresses as read from a file: valid, distinct and at most the limit.
pub fn sanitized_addresses(recent: &[String]) -> Vec<String> {
    let mut kept = Vec::new();
    for address in recent.iter().rev() {
        remember_address(&mut kept, address);
    }
    kept
}

/// Checks a caption coming over IPC (SECURITY.md §2).
pub fn validate_caption(caption: &str) -> Result<(), SettingsError> {
    if caption.chars().count() > MAX_CAPTION_CHARS || caption.chars().any(char::is_control) {
        return Err(SettingsError::InvalidCaption);
    }
    Ok(())
}

impl StampSettings {
    /// Brings a caption from a file within limits: no control characters, 80 characters.
    pub fn sanitize(&mut self) {
        self.caption = sanitize_caption(&self.caption);
    }

    /// Checks a stamp coming over IPC (SECURITY.md §2).
    pub fn validate(&self) -> Result<(), SettingsError> {
        validate_caption(&self.caption)
    }
}

impl RenderSettings {
    /// The settings for the chosen export target: a photo for social media is not a print,
    /// so it keeps its own ratio and has no print border (FR-053, FR-067). Export and
    /// preview both use this, so the preview shows what is exported.
    pub fn for_target(&self) -> std::borrow::Cow<'_, Self> {
        match self.export_target.sanitized() {
            crate::share::ExportTarget::Share(_) => std::borrow::Cow::Owned(Self {
                print_size: crate::PrintSize::ORIGINAL_ID.to_owned(),
                border: false,
                ..self.clone()
            }),
            crate::share::ExportTarget::Print => std::borrow::Cow::Borrowed(self),
        }
    }

    /// The settings one photo is rendered with: its own stamp when it has one (FR-044), and
    /// its own line under the date over either (FR-036).
    pub fn for_photo(
        &self,
        stamp: Option<&StampSettings>,
        caption: Option<&str>,
        look: Option<&crate::looks::LookSettings>,
    ) -> std::borrow::Cow<'_, Self> {
        if stamp.is_none() && caption.is_none() && look.is_none() {
            return std::borrow::Cow::Borrowed(self);
        }
        let mut own = self.clone();
        if let Some(stamp) = stamp {
            own.stamp = stamp.clone();
        }
        if let Some(look) = look {
            own.look = look.clone();
        }
        if let Some(caption) = caption {
            caption.clone_into(&mut own.stamp.caption);
        }
        std::borrow::Cow::Owned(own)
    }
}

/// Date stamp formats (FR-004).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum DateFormat {
    /// `2026-10-23`
    Iso,
    /// `23.10.2026`
    Dmy,
    /// `'26 10 23`, as on 1990s film cameras
    Retro,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Corner {
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum StampColor {
    White,
    RetroOrange,
}

impl StampColor {
    pub fn rgb(self) -> [u8; 3] {
        match self {
            Self::White => [255, 255, 255],
            Self::RetroOrange => [255, 138, 31],
        }
    }
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            print_size: "10x15".to_owned(),
            stamp: StampSettings {
                enabled: true,
                format: DateFormat::Iso,
                corner: Corner::BottomRight,
                color: StampColor::White,
                place: false,
                caption: String::new(),
                size: StampSize::Medium,
            },
            custom_sizes: Vec::new(),
            place_format: crate::geocode::PlaceFormat::default(),
            export_target: crate::share::ExportTarget::default(),
            border: false,
            border_color: BorderColor::White,
            collage: CollageSettings::default(),
            share_fill: ShareFill::default(),
            look: crate::looks::LookSettings::default(),
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SettingsError {
    #[error("unknown print size: {0}")]
    UnknownPrintSize(String),
    #[error("invalid custom size: {0}")]
    InvalidCustomSize(String),
    #[error("the caption has control characters or more than {MAX_CAPTION_CHARS} characters")]
    InvalidCaption,
}

impl RenderSettings {
    /// Validates input coming over IPC (SECURITY.md §2).
    /// Checks the settings and resolves the selected size. For Original (FR-045) this is a
    /// placeholder: use [`Self::print_for`] wherever the photo is known.
    pub fn validate(&self) -> Result<PrintSize, SettingsError> {
        self.validate_custom_sizes()?;
        self.stamp.validate()?;
        if self.print_size == PrintSize::ORIGINAL_ID {
            return Ok(PrintSize::original(3, 2));
        }
        if let Some(size) = PrintSize::by_id(&self.print_size) {
            return Ok(size);
        }
        self.custom_sizes
            .iter()
            .find(|c| c.id == self.print_size)
            .map(|c| PrintSize::custom(c.short_mm, c.long_mm))
            .ok_or_else(|| SettingsError::UnknownPrintSize(self.print_size.clone()))
    }

    /// The print size for an upright `width` × `height` photo: the selected size, or the
    /// photo's own ratio when Original is selected (FR-045).
    pub fn print_for(&self, width: u32, height: u32) -> Result<PrintSize, SettingsError> {
        let size = self.validate()?;
        Ok(if size.is_original() {
            PrintSize::original(width, height)
        } else {
            size
        })
    }

    /// Display unit of the selected size: inches for custom sizes entered in inches.
    pub fn unit(&self) -> LengthUnit {
        self.custom_sizes
            .iter()
            .find(|c| c.id == self.print_size)
            .map_or(LengthUnit::Cm, |c| c.unit)
    }

    fn validate_custom_sizes(&self) -> Result<(), SettingsError> {
        let bad = |why: &str| Err(SettingsError::InvalidCustomSize(why.to_owned()));
        if self.custom_sizes.len() > MAX_CUSTOM_SIZES {
            return bad("too many custom sizes");
        }
        let mut ids = std::collections::HashSet::new();
        for c in &self.custom_sizes {
            if !c.id.starts_with("custom-") || !ids.insert(c.id.as_str()) {
                return bad("duplicate or malformed id");
            }
            if c.name.chars().count() > 40 || c.name.chars().any(char::is_control) {
                return bad("name");
            }
            if !CUSTOM_EDGE_MM.contains(&c.short_mm)
                || !CUSTOM_EDGE_MM.contains(&c.long_mm)
                || c.short_mm > c.long_mm
            {
                return bad("edges must be 2–100 cm, short edge first");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    /// FR-124: the machine's locale picks the first launch's language, English otherwise.
    #[test]
    fn fr_124_the_machines_language() {
        use super::Language;
        assert_eq!(Language::from_locale("hu-HU"), Language::Hu);
        assert_eq!(Language::from_locale("hu"), Language::Hu);
        assert_eq!(Language::from_locale("vi_VN.UTF-8"), Language::Vi);
        assert_eq!(Language::from_locale("VI"), Language::Vi);
        assert_eq!(Language::from_locale("en-GB"), Language::En);
        assert_eq!(Language::from_locale("cs-CZ"), Language::En);
        assert_eq!(Language::from_locale(""), Language::En);
    }

    /// FR-086, the *Auto* button: the memory share by RAM (40 % from 32 GB, 25 % from
    /// 12 GB, else 15 %), the largest cache within a tenth of the free disk, previews off
    /// under 5 GB free; other settings untouched.
    #[test]
    fn fr_086_auto_follows_the_machine() {
        const GB: u64 = 1024 * 1024 * 1024;
        let base = UiSettings {
            language: Language::Vi,
            ..UiSettings::default()
        };
        let pick = |ram, disk| {
            let a = base.clone().auto_for(ram, disk);
            (
                a.preview_memory_percent,
                a.cache_limit_gb,
                a.prepare_previews,
                a.language,
            )
        };
        assert_eq!(pick(8 * GB, 30 * GB), (15, 2, true, Language::Vi));
        assert_eq!(pick(16 * GB, 60 * GB), (25, 5, true, Language::Vi));
        assert_eq!(pick(16 * GB, 120 * GB), (25, 10, true, Language::Vi));
        assert_eq!(pick(32 * GB, 500 * GB), (40, 20, true, Language::Vi));
        assert_eq!(pick(12 * GB, 4 * GB), (25, 2, false, Language::Vi));
        let d = base.clone().auto_for(32 * GB, 500 * GB).memory_defaults();
        assert_eq!(
            (
                d.preview_memory_percent,
                d.cache_limit_gb,
                d.prepare_previews
            ),
            (25, 5, true)
        );
    }

    /// Audit 2026-11-06: *Auto* counts in GB of 10⁹ bytes, the unit of the limit and the
    /// window, so its thresholds sit where the window says (in GiB, 50 GB free gave 2 GB, not
    /// 5, and 12 GB of RAM gave 15 %).
    #[test]
    fn fr_086_auto_counts_in_the_windows_gb() {
        const GB: u64 = 1_000_000_000;
        let auto = |ram, disk| {
            let a = UiSettings::default().auto_for(ram, disk);
            (
                a.preview_memory_percent,
                a.cache_limit_gb,
                a.prepare_previews,
            )
        };
        assert_eq!(auto(12 * GB, 50 * GB), (25, 5, true));
        assert_eq!(auto(12 * GB - 1, 50 * GB - 1), (15, 2, true));
        assert_eq!(auto(32 * GB, 200 * GB), (40, 20, true));
        assert_eq!(auto(32 * GB - 1, 100 * GB), (25, 10, true));
        assert_eq!(auto(8 * GB, 5 * GB), (15, 2, true));
        assert_eq!(auto(8 * GB, 5 * GB - 1), (15, 2, false));
    }

    /// Audit 2026-11-06: a memory share saved outside the window's choices goes to the
    /// nearest, so one segment is always selected.
    #[test]
    fn fr_086_a_memory_share_snaps_to_a_choice() {
        let share = |p| {
            UiSettings {
                preview_memory_percent: p,
                ..UiSettings::default()
            }
            .sanitized()
            .preview_memory_percent
        };
        let got: Vec<u8> = [0, 15, 19, 21, 30, 33, 40, 50, 255].map(share).to_vec();
        assert_eq!(got, vec![15, 15, 15, 25, 25, 40, 40, 40, 40]);
    }
    use super::*;

    #[test]
    fn fr_036_a_photos_own_line_wins_and_the_rest_follows_the_settings() {
        let mut settings = RenderSettings::default();
        settings.stamp.caption = "TRIP".into();
        settings.stamp.color = StampColor::RetroOrange;
        // No own stamp, no own line: the settings themselves.
        assert!(matches!(
            settings.for_photo(None, None, None),
            std::borrow::Cow::Borrowed(_)
        ));
        // Its own line only: the line changes, the colour still follows the settings.
        let own = settings.for_photo(None, Some("GRANDMA"), None);
        assert_eq!(own.stamp.caption, "GRANDMA");
        assert_eq!(own.stamp.color, StampColor::RetroOrange);
        // Over its own stamp too.
        let stamp = StampSettings {
            caption: "OWN".into(),
            ..settings.stamp.clone()
        };
        assert_eq!(
            settings
                .for_photo(Some(&stamp), Some("LINE"), None)
                .stamp
                .caption,
            "LINE"
        );
    }

    fn with_custom(sizes: Vec<CustomPrintSize>, selected: &str) -> RenderSettings {
        RenderSettings {
            print_size: selected.to_owned(),
            custom_sizes: sizes,
            ..RenderSettings::default()
        }
    }

    fn size(id: &str, short_mm: u32, long_mm: u32, unit: LengthUnit) -> CustomPrintSize {
        CustomPrintSize {
            id: id.to_owned(),
            name: "Mine".to_owned(),
            short_mm,
            long_mm,
            unit,
        }
    }

    #[test]
    fn fr_043_custom_size_resolves_and_validates() {
        // 4 × 6 in, stored in mm.
        let four_by_six = size("custom-1", 102, 152, LengthUnit::In);
        let s = with_custom(vec![four_by_six.clone()], "custom-1");
        let print = s.validate();
        assert_eq!(print.map(|p| (p.short_mm, p.long_mm)), Ok((102, 152)));
        assert_eq!(s.unit(), LengthUnit::In);
        // Built-ins still resolve, and use cm.
        let builtin = with_custom(vec![four_by_six.clone()], "10x15");
        assert!(builtin.validate().is_ok());
        assert_eq!(builtin.unit(), LengthUnit::Cm);

        for bad in [
            vec![size("custom-1", 10, 152, LengthUnit::Cm)], // edge below 2 cm
            vec![size("custom-1", 152, 102, LengthUnit::Cm)], // long edge first
            vec![size("x-1", 102, 152, LengthUnit::Cm)],     // malformed id
            vec![four_by_six.clone(), four_by_six.clone()],  // duplicate id
        ] {
            assert!(with_custom(bad, "10x15").validate().is_err());
        }
        assert!(
            with_custom(vec![], "custom-9").validate().is_err(),
            "unknown id"
        );
    }
    /// FR-077: the newest address comes first, once, the list keeps eight, and blank or
    /// invalid ones are never kept, from a file either.
    #[test]
    fn fr_077_recent_addresses_are_newest_first_distinct_and_few() {
        let mut recent = Vec::new();
        for n in 0..10 {
            remember_address(&mut recent, &format!("Street {n}\nPraha"));
        }
        remember_address(&mut recent, "Street 3\nPraha");
        remember_address(&mut recent, "   ");
        remember_address(&mut recent, &"x\n".repeat(9));
        assert_eq!(recent.len(), MAX_RECENT_ADDRESSES);
        assert_eq!(recent[0], "Street 3\nPraha");
        assert_eq!(recent[1], "Street 9\nPraha");
        assert_eq!(recent.iter().filter(|a| *a == "Street 3\nPraha").count(), 1);
        let from_file =
            sanitized_addresses(&["A".into(), "A".into(), "bad\u{7}".into(), "B".into()]);
        assert_eq!(from_file, ["A", "B"]);
    }

    /// FR-077 (owner, 2026-11-07: the list held the same address five times): the start of an
    /// address, a draft saved while it was typed, gives way to the whole one, and a list read
    /// from a file loses such drafts too.
    #[test]
    fn fr_077_a_draft_of_an_address_gives_way_to_the_whole_one() {
        let mut recent = Vec::new();
        for draft in ["THAI", "THAI DUONG", "THAI DUONG PHAM\nUnit 2"] {
            remember_address(&mut recent, draft);
        }
        assert_eq!(recent, ["THAI DUONG PHAM\nUnit 2"]);
        remember_address(&mut recent, "Other\nPraha");
        assert_eq!(recent.len(), 2, "a different address stays");
        let from_file = sanitized_addresses(&["A\nB\nC".into(), "A\nB".into(), "A".into()]);
        assert_eq!(from_file, ["A\nB\nC"]);
    }
}
