//! Talks to YASAFF. Copy this file unchanged into your own theme; the layout lives elsewhere.
//!
//! YASAFF runs `theme.wasm` sandboxed and calls three functions, all exported here by
//! [`export_theme!`]: `yasaff_alloc` for room to write its input, `yasaff_describe` for the
//! theme's options, and `yasaff_frame` once per frame. Input and output are JSON. The module may
//! import nothing, so build for `wasm32-unknown-unknown` and don't use WASI, the clock or files:
//! everything a theme needs comes in the [`Frame`].

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A theme: says what options it has, and draws frames.
pub trait Theme: Default {
    /// The options shown in Settings › Home screen, the pictures page and the game picture, as
    /// YASAFF's `Manifest` reads them. `language` is `english`, `french`, `spanish` or `polish`.
    fn describe(language: &str) -> Value;

    /// Draws one frame. The theme keeps whatever it wants between frames in `self`.
    fn frame(&mut self, frame: Frame) -> Output;
}

/// What YASAFF sends each frame.
#[derive(Deserialize)]
pub struct Frame {
    /// The window: `[x, y, width, height]`.
    pub screen: [f32; 4],
    /// The space between the top and bottom bars.
    pub area: [f32; 4],
    /// The window's height over 1080: multiply sizes given at 1080p by it.
    pub scale: f32,
    /// Seconds since the theme was loaded, and since the last frame.
    pub time: f64,
    pub dt: f32,
    /// Off while a menu or Settings covers the home screen.
    pub interactive: bool,
    /// The selected tile, by its index in `Library::tiles`.
    pub selected: usize,
    /// Sent when it changes; [`Cache`] keeps the last one.
    pub library: Option<Library>,
    /// Sent when they change; [`Cache`] keeps the last ones.
    pub options: Option<Map<String, Value>>,
    pub events: Vec<Event>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default)]
pub struct Library {
    /// `english`, `french`, `spanish` or `polish`.
    pub language: String,
    /// The user's categories, in their order.
    pub categories: Vec<String>,
    /// The keys of the theme's pictures that are set.
    pub pictures: Vec<String>,
    /// The games, then Discover, Add game and Settings (those not moved to a bar).
    pub tiles: Vec<Tile>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default)]
pub struct Tile {
    /// `game`, `discover`, `add` or `settings`.
    pub kind: String,
    pub name: String,
    /// A line about it: last played, running, an update, or what a built-in tile does.
    pub status: String,
    pub categories: Vec<String>,
    pub main_category: Option<String>,
    /// A game has its picture (the manifest's `game_picture`).
    pub picture: bool,
    /// A built-in tile has an icon.
    pub icon: bool,
    pub running: bool,
}

impl Tile {
    pub fn is_game(&self) -> bool {
        self.kind == "game"
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Event {
    /// `left`, `right`, `up` or `down`.
    Nav(String),
    /// The id of a clicked [`Output::hit`] area.
    Click(u64),
}

/// The library and options, kept from the frame that last sent them.
#[derive(Default)]
pub struct Cache {
    pub library: Library,
    pub options: Map<String, Value>,
}

impl Cache {
    pub fn update(&mut self, frame: &mut Frame) {
        if let Some(library) = frame.library.take() {
            self.library = library;
        }
        if let Some(options) = frame.options.take() {
            self.options = options;
        }
    }

    pub fn number(&self, key: &str) -> f32 {
        self.options.get(key).and_then(Value::as_f64).unwrap_or(0.0) as f32
    }

    pub fn flag(&self, key: &str) -> bool {
        self.options.get(key).and_then(Value::as_bool).unwrap_or(false)
    }

    pub fn text(&self, key: &str) -> Option<&str> {
        self.options.get(key).and_then(Value::as_str)
    }

    pub fn has_picture(&self, key: &str) -> bool {
        self.library.pictures.iter().any(|p| p == key)
    }
}

/// What the theme gives back.
#[derive(Serialize, Default)]
pub struct Output {
    pub draw: Vec<Shape>,
    pub hits: Vec<Hit>,
    /// Selects this tile; YASAFF plays the swap sound.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select: Option<usize>,
    /// Clicks this tile: selects it, or starts it when it is already selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click: Option<usize>,
}

impl Output {
    pub fn draw(&mut self, shape: Shape) {
        self.draw.push(shape);
    }

    /// Reports a click on `rect` as [`Event::Click`] with `id` on the next frame.
    pub fn hit(&mut self, rect: Rect, id: u64) {
        self.hits.push(Hit { rect, id });
    }
}

pub type Rect = [f32; 4];

#[derive(Serialize)]
pub struct Hit {
    pub rect: Rect,
    pub id: u64,
}

/// Colors are `#rrggbb` or `#rrggbbaa`.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Shape {
    Rect {
        rect: Rect,
        radius: f32,
        #[serde(skip_serializing_if = "Option::is_none")]
        fill: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        stroke: Option<Stroke>,
    },
    /// A soft halo around a box.
    Glow { rect: Rect, radius: f32, color: String, strength: f32 },
    Circle { center: [f32; 2], radius: f32, fill: String },
    Image {
        source: Source,
        rect: Rect,
        alpha: f32,
        radius: f32,
        /// A halo that follows the picture's outline.
        #[serde(skip_serializing_if = "Option::is_none")]
        glow: Option<Glow>,
    },
    /// Text anchored at `pos`: `align` is horizontal (`left`, `center`, `right`) then vertical
    /// (`top`, `center`, `bottom`).
    Text {
        text: String,
        pos: [f32; 2],
        align: [&'static str; 2],
        size: f32,
        font: Font,
        color: String,
        /// A halo around the letters; without one the text has a drop shadow.
        #[serde(skip_serializing_if = "Option::is_none")]
        glow: Option<Glow>,
        shadow: bool,
    },
    /// Text centered in `rect` at the biggest size up to `max_size` where it fits without breaking a word.
    FitText {
        text: String,
        rect: Rect,
        max_size: f32,
        font: Font,
        color: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        glow: Option<Glow>,
    },
}

#[derive(Serialize)]
pub struct Stroke {
    pub width: f32,
    pub color: String,
}

#[derive(Serialize, Clone)]
pub struct Glow {
    pub color: String,
    pub strength: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    /// A game's picture.
    Tile(usize),
    /// A built-in tile's icon.
    Icon(usize),
    /// One of the theme's pictures, by key.
    Picture(String),
    /// A file in the theme's `images/` folder.
    File(String),
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Font {
    /// The game title font from Settings › Fonts.
    Title,
    /// YASAFF's interface font.
    Body,
    /// An imported font file, e.g. a `font` option's value.
    File(String),
    /// A game's own title font.
    Tile(usize),
}

/// `color` with its alpha multiplied by `alpha`.
pub fn fade(color: &str, alpha: f32) -> String {
    let hex = color.trim_start_matches('#');
    let base = u8::from_str_radix(hex.get(6..8).unwrap_or("ff"), 16).unwrap_or(255) as f32;
    let a = (base * alpha.clamp(0.0, 1.0)).round() as u8;
    format!("#{}{a:02x}", hex.get(..6).unwrap_or("ffffff"))
}

/// A value that moves to its target over a fixed time, at an even speed.
#[derive(Default)]
pub struct Animated {
    from: f32,
    to: f32,
    since: f64,
    started: bool,
}

impl Animated {
    pub fn get(&mut self, target: f32, now: f64, seconds: f64) -> f32 {
        let current = self.value(now, seconds);
        if !self.started {
            (self.from, self.to, self.since, self.started) = (target, target, now, true);
        } else if target != self.to {
            (self.from, self.to, self.since) = (current, target, now);
        }
        self.value(now, seconds)
    }

    fn value(&self, now: f64, seconds: f64) -> f32 {
        let t = if seconds <= 0.0 { 1.0 } else { ((now - self.since) / seconds).clamp(0.0, 1.0) as f32 };
        self.from + (self.to - self.from) * t
    }
}

// The ABI. One buffer for YASAFF's input and one for the theme's output; wasm is single-threaded.
thread_local! {
    static INPUT: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
    static OUTPUT: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
}

#[doc(hidden)]
pub fn alloc(len: i32) -> i32 {
    INPUT.with_borrow_mut(|input| {
        input.clear();
        input.resize(len.max(0) as usize, 0);
        input.as_mut_ptr() as i32
    })
}

/// Parses the input YASAFF wrote, runs `f`, and hands its JSON back packed as YASAFF reads it.
#[doc(hidden)]
pub fn respond<I: for<'de> Deserialize<'de>, O: Serialize>(f: impl FnOnce(I) -> O) -> i64 {
    let input = INPUT.with_borrow(|input| serde_json::from_slice(input));
    let json = match input {
        Ok(input) => serde_json::to_vec(&f(input)),
        Err(e) => panic!("YASAFF sent input this theme can't read: {e}"),
    }
    .unwrap_or_default();
    OUTPUT.with_borrow_mut(|out| {
        *out = json;
        ((out.as_ptr() as i64) << 32) | out.len() as i64
    })
}

#[derive(Deserialize)]
#[doc(hidden)]
pub struct DescribeIn {
    #[serde(default)]
    pub language: String,
}

/// Exports `yasaff_alloc`, `yasaff_describe` and `yasaff_frame` for the theme type `$theme`.
#[macro_export]
macro_rules! export_theme {
    ($theme:ty) => {
        thread_local! {
            static THEME: std::cell::RefCell<$theme> = std::cell::RefCell::new(<$theme>::default());
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn yasaff_alloc(len: i32) -> i32 {
            $crate::yasaff::alloc(len)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn yasaff_describe(_ptr: i32, _len: i32) -> i64 {
            $crate::yasaff::respond(|input: $crate::yasaff::DescribeIn| <$theme as $crate::yasaff::Theme>::describe(&input.language))
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn yasaff_frame(_ptr: i32, _len: i32) -> i64 {
            $crate::yasaff::respond(|frame: $crate::yasaff::Frame| THEME.with_borrow_mut(|theme| $crate::yasaff::Theme::frame(theme, frame)))
        }
    };
}
