//! The Double axis theme: categories side by side along the top, the games of the selected one
//! listed below it, as on a PlayStation 3. The last category, Main, holds Settings, Discover and
//! Add game.
//!
//! YASAFF calls [`DoubleAxis::frame`] every frame with the library, the options and the input; the
//! theme answers with what to draw. Everything it remembers lives in [`DoubleAxis`].

mod describe;
pub mod yasaff;

use describe::tr;
use serde_json::Value;
use std::collections::HashMap;
use yasaff::{Animated, Cache, Event, Font, Frame, Glow, Output, Rect, Shape, Source, Stroke, Theme, fade};

export_theme!(DoubleAxis);

/// Hit ids of the game list start here; below it are the columns.
const ITEM_HITS: u64 = 1 << 32;
/// The dot beside a running game.
const RUNNING: &str = "#5cb83c";

#[derive(Default)]
pub struct DoubleAxis {
    cache: Cache,
    /// The column last moved to, by its key, and the tile last selected in each column.
    column: Option<String>,
    rows: HashMap<String, usize>,
    column_scroll: Animated,
    row_scrolls: HashMap<String, Animated>,
}

/// One column: a category and the tiles in it.
struct Column {
    /// The key its picture is saved under: `category:<name>`, `builtin:uncategorized` or `builtin:main`.
    key: String,
    name: String,
    items: Vec<usize>,
}

/// The options, read once a frame.
struct Options {
    size: [f32; 2],
    border: bool,
    font: Font,
    font_size: f32,
    dynamic_font: bool,
    text_color: String,
    glow_color: String,
    glow: f32,
    title_info: String,
    list_size: [f32; 2],
    list_align: String,
}

impl Options {
    fn read(cache: &Cache) -> Self {
        Self {
            size: [cache.number("width"), cache.number("height")],
            border: cache.flag("border"),
            font: cache.text("font").map_or(Font::Title, |f| Font::File(f.to_owned())),
            font_size: cache.number("font_size"),
            dynamic_font: cache.flag("dynamic_font"),
            text_color: cache.text("text_color").unwrap_or("#ffffff").to_owned(),
            glow_color: cache.text("glow_color").unwrap_or("#8fc8ff").to_owned(),
            glow: cache.number("glow_intensity"),
            title_info: cache.text("title_info").unwrap_or("selected").to_owned(),
            list_size: [cache.number("list_width"), cache.number("list_height")],
            list_align: cache.text("list_align").unwrap_or("left").to_owned(),
        }
    }

    fn glow(&self, strength: f32) -> Option<Glow> {
        (strength > 0.01).then(|| Glow { color: self.glow_color.clone(), strength })
    }
}

impl Theme for DoubleAxis {
    fn describe(language: &str) -> Value {
        describe::manifest(language)
    }

    fn frame(&mut self, mut frame: Frame) -> Output {
        self.cache.update(&mut frame);
        let mut out = Output::default();
        let columns = self.columns();
        let mut selected = frame.selected;
        for event in &frame.events {
            match event {
                Event::Nav(dir) => self.navigate(&columns, &mut selected, dir),
                Event::Click(id) if *id >= ITEM_HITS => {
                    let current = self.current(&columns, selected);
                    self.column = Some(columns[current].key.clone());
                    out.click = Some((id - ITEM_HITS) as usize);
                }
                Event::Click(id) => {
                    let current = self.current(&columns, selected);
                    if (*id as usize) != current {
                        self.focus(&columns, &mut selected, current, *id as usize);
                    }
                }
            }
        }
        if selected != frame.selected {
            out.select = Some(selected);
        }
        if !columns.is_empty() {
            self.draw(&frame, &columns, selected, &mut out);
        }
        out
    }
}

impl DoubleAxis {
    /// The columns with something in them, Main last.
    fn columns(&self) -> Vec<Column> {
        let library = &self.cache.library;
        let tiles = &library.tiles;
        let mut columns: Vec<Column> = library
            .categories
            .iter()
            .map(|name| Column {
                key: format!("category:{name}"),
                name: name.clone(),
                items: (0..tiles.len()).filter(|&i| tiles[i].is_game() && tiles[i].categories.contains(name)).collect(),
            })
            .collect();
        columns.push(Column {
            key: "builtin:uncategorized".to_owned(),
            name: tr(&library.language, "Games"),
            items: (0..tiles.len()).filter(|&i| tiles[i].is_game() && tiles[i].categories.is_empty()).collect(),
        });
        columns.retain(|c| !c.items.is_empty());
        columns.push(Column {
            key: "builtin:main".to_owned(),
            name: tr(&library.language, "Main"),
            items: (0..tiles.len()).filter(|&i| !tiles[i].is_game()).collect(),
        });
        columns
    }

    /// The column shown: the one last moved to while it holds the selected tile, else the first that does.
    fn current(&self, columns: &[Column], selected: usize) -> usize {
        let holds = |c: &Column| c.items.contains(&selected);
        let remembered = self.column.as_ref().and_then(|key| columns.iter().position(|c| c.key == *key && holds(c)));
        remembered.or_else(|| columns.iter().position(holds)).unwrap_or(0)
    }

    fn focus(&mut self, columns: &[Column], selected: &mut usize, from: usize, to: usize) {
        if let Some(col) = columns.get(from) {
            self.rows.insert(col.key.clone(), *selected);
        }
        let Some(col) = columns.get(to) else { return };
        let remembered = self.rows.get(&col.key).copied().filter(|i| col.items.contains(i));
        if let Some(i) = remembered.or(col.items.first().copied()) {
            *selected = i;
        }
        self.column = Some(col.key.clone());
    }

    fn navigate(&mut self, columns: &[Column], selected: &mut usize, dir: &str) {
        let current = self.current(columns, *selected);
        match dir {
            "left" if current > 0 => self.focus(columns, selected, current, current - 1),
            "right" if current + 1 < columns.len() => self.focus(columns, selected, current, current + 1),
            "up" | "down" => {
                let items = &columns[current].items;
                if items.is_empty() {
                    return;
                }
                let row = items.iter().position(|&i| i == *selected).unwrap_or(0);
                let row = if dir == "up" { row.saturating_sub(1) } else { (row + 1).min(items.len() - 1) };
                *selected = items[row];
                self.column = Some(columns[current].key.clone());
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &Frame, columns: &[Column], selected: usize, out: &mut Output) {
        let o = Options::read(&self.cache);
        let s = frame.scale;
        let [sx, _, sw, _] = frame.screen;
        let current = self.current(columns, selected);
        let [w, h] = o.size.map(|v| v * s);
        let gap = 36.0 * s;
        let radius = 8.0 * s;
        let top = frame.area[1] + 40.0 * s;
        let anchor = sx + (sw * 0.18).max(110.0 * s);
        let scroll = self.column_scroll.get(current as f32, frame.time, 0.2);

        for (j, col) in columns.iter().enumerate() {
            let offset = j as f32 - scroll;
            let focus = (1.0 - offset.abs()).max(0.0);
            let scale = 0.75 + 0.25 * focus;
            let (cx, cy) = (anchor + offset * (w + gap) + w / 2.0, top + h / 2.0);
            let rect: Rect = [cx - w * scale / 2.0, cy - h * scale / 2.0, w * scale, h * scale];
            if rect[0] + rect[2] < sx || rect[0] > sx + sw {
                continue;
            }
            // Only the selected category glows; the others are just smaller, never faded.
            let glow = o.glow * focus;
            let has_image = self.cache.has_picture(&col.key);
            if has_image {
                out.draw(Shape::Image { source: Source::Picture(col.key.clone()), rect, alpha: 1.0, radius, glow: o.glow(glow) });
            } else if o.border && glow > 0.01 {
                out.draw(Shape::Glow { rect, radius, color: o.glow_color.clone(), strength: glow });
            }
            if o.border {
                let stroke = Stroke { width: 2.0 * s, color: o.text_color.clone() };
                out.draw(Shape::Rect { rect, radius, fill: None, stroke: Some(stroke) });
            }
            if !has_image {
                let pad = 10.0 * s * scale;
                let inner = [rect[0] + pad, rect[1] + pad, rect[2] - 2.0 * pad, rect[3] - 2.0 * pad];
                let max_size = if o.dynamic_font { inner[3] } else { o.font_size * s * scale };
                let glow = if o.border { None } else { o.glow(glow) };
                out.draw(Shape::FitText { text: col.name.clone(), rect: inner, max_size, font: o.font.clone(), color: o.text_color.clone(), glow });
            }
            if frame.interactive {
                out.hit(rect, j as u64);
            }
        }

        self.draw_items(frame, &columns[current], selected, [anchor, top], [w, h], &o, out);
    }

    /// The tiles of the selected column: the selected one just below the categories, the ones
    /// before it above them.
    #[allow(clippy::too_many_arguments)]
    fn draw_items(&mut self, frame: &Frame, col: &Column, selected: usize, origin: [f32; 2], size: [f32; 2], o: &Options, out: &mut Output) {
        let s = frame.scale;
        let [sx, sy, sw, sh] = frame.screen;
        let row = col.items.iter().position(|&i| i == selected).unwrap_or(0);
        let scroll = self.row_scrolls.entry(col.key.clone()).or_default().get(row as f32, frame.time, 0.18);
        let [list_w, list_h] = o.list_size;
        let aspect = list_w / list_h.max(1.0);
        let big = list_h * s;
        let small = big * 0.62;
        let step = small + 18.0 * s;
        // Pictures line up with the selected category's box; names go beside them, away from it.
        let (box_left, box_right) = (origin[0], origin[0] + size[0]);
        let below = origin[1] + size[1] + 30.0 * s;
        let above = origin[1] - 24.0 * s - small;
        let tiles = &self.cache.library.tiles;
        for (k, &i) in col.items.iter().enumerate() {
            let Some(tile) = tiles.get(i) else { continue };
            let d = k as f32 - scroll;
            let focus = (1.0 - d.abs()).max(0.0);
            let is_selected = k == row;
            let y = if d >= 0.0 {
                below + d * step + (big - small) * d.min(1.0)
            } else if d > -1.0 {
                above - step + (below - above + step) * (d + 1.0)
            } else {
                above + (d + 1.0) * step
            };
            let item_h = small + (big - small) * focus;
            if y > sy + sh || y + item_h < sy {
                continue;
            }
            let alpha = if d < -0.5 { 0.45 } else { 0.7 + 0.3 * focus };
            let glow = o.glow * focus;
            let item_w = item_h * aspect;
            let left = match o.list_align.as_str() {
                "center" => (box_left + box_right - item_w) / 2.0,
                "right" => box_right - item_w,
                _ => box_left,
            };
            let picture: Rect = [left, y, item_w, item_h];
            // A Main entry shows the theme's picture for it, else its tile icon.
            let entry = format!("entry:{}", tile.kind);
            let source = if tile.is_game() {
                tile.picture.then_some(Source::Tile(i))
            } else {
                self.cache.has_picture(&entry).then_some(Source::Picture(entry))
            };
            let icon = source.is_none() && !tile.is_game() && tile.icon;
            let has_picture = source.is_some() || icon;
            if let Some(source) = source {
                out.draw(Shape::Image { source, rect: picture, alpha, radius: 6.0 * s, glow: o.glow(glow) });
            } else if icon {
                let side = item_h * 0.64;
                let shown = [left + (item_w - side) / 2.0, y + (item_h - side) / 2.0, side, side];
                out.draw(Shape::Image { source: Source::Icon(i), rect: shown, alpha, radius: 0.0, glow: o.glow(glow) });
            }
            if tile.running {
                let x = if o.list_align == "right" { left + item_w + 18.0 * s } else { left - 18.0 * s };
                out.draw(Shape::Circle { center: [x, y + item_h / 2.0], radius: 6.0 * s, fill: RUNNING.to_owned() });
            }
            // Without a picture the name is all there is, so it always shows.
            let show_title = !has_picture
                || match o.title_info.as_str() {
                    "always" => true,
                    "never" => false,
                    _ => is_selected,
                };
            if show_title {
                // Where the name starts or ends, and which way it runs from there.
                let (x, align) = match (has_picture, o.list_align.as_str()) {
                    (true, "right") => (left - 20.0 * s, "right"),
                    (true, _) => (left + item_w + 20.0 * s, "left"),
                    (false, "center") => ((box_left + box_right) / 2.0, "center"),
                    (false, "right") => (box_right, "right"),
                    (false, _) => (box_left, "left"),
                };
                let font_size = 26.0 * s + 14.0 * s * focus;
                let font = if tile.is_game() { Font::Tile(i) } else { Font::Body };
                let color = fade(&o.text_color, alpha);
                let with_status = is_selected && o.title_info != "never";
                let text_y = if with_status { y + item_h * 0.42 } else { y + item_h / 2.0 };
                let glow = if has_picture { None } else { o.glow(glow) };
                out.draw(Shape::Text { text: tile.name.clone(), pos: [x, text_y], align: [align, "center"], size: font_size, font, color, glow, shadow: true });
                if with_status {
                    let sub_color = fade(&o.text_color, 0.75 * ((focus - 0.5) * 2.0).max(0.0));
                    let pos = [x, y + item_h * 0.42 + font_size * 0.75];
                    out.draw(Shape::Text { text: tile.status.clone(), pos, align: [align, "top"], size: 20.0 * s, font: Font::Body, color: sub_color, glow: None, shadow: true });
                }
            }
            if frame.interactive {
                out.hit([sx + 60.0 * s, y, sw - 120.0 * s, item_h], ITEM_HITS + i as u64);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn frame(events: serde_json::Value, selected: usize, library: bool) -> Frame {
        let mut input = json!({
            "screen": [0, 0, 1920, 1080], "area": [0, 88, 1920, 892], "scale": 1.0, "time": 0.0, "dt": 0.016,
            "interactive": true, "selected": selected, "events": events,
        });
        if library {
            input["library"] = json!({
                "language": "english",
                "categories": ["Racing"],
                "pictures": [],
                "tiles": [
                    { "kind": "game", "name": "A", "categories": ["Racing"] },
                    { "kind": "game", "name": "B" },
                    { "kind": "game", "name": "C" },
                    { "kind": "discover", "name": "Discover", "icon": true },
                    { "kind": "settings", "name": "Settings", "icon": true },
                ],
            });
            input["options"] = json!({ "width": 220, "height": 120, "glow_intensity": 0.6, "list_width": 104, "list_height": 104,
                                       "text_color": "#ffffff", "glow_color": "#8fc8ff", "title_info": "selected", "list_align": "left" });
        }
        serde_json::from_value(input).unwrap()
    }

    #[test]
    fn moves_between_columns_and_remembers_the_row() {
        let mut theme = DoubleAxis::default();
        assert_eq!(theme.frame(frame(json!([{ "nav": "right" }]), 0, true)).select, Some(1));
        assert_eq!(theme.frame(frame(json!([{ "nav": "down" }]), 1, false)).select, Some(2));
        assert_eq!(theme.frame(frame(json!([{ "nav": "right" }]), 2, false)).select, Some(3));
        assert_eq!(theme.frame(frame(json!([{ "nav": "left" }]), 3, false)).select, Some(2));
        assert_eq!(theme.frame(frame(json!([{ "nav": "left" }]), 2, false)).select, Some(0));
        assert_eq!(theme.frame(frame(json!([{ "nav": "left" }]), 0, false)).select, None);
    }

    #[test]
    fn clicks_pick_a_column_or_a_tile() {
        let mut theme = DoubleAxis::default();
        let out = theme.frame(frame(json!([{ "click": 2 }]), 0, true));
        assert_eq!(out.select, Some(3));
        let out = theme.frame(frame(json!([{ "click": ITEM_HITS + 4 }]), 3, false));
        assert_eq!((out.select, out.click), (None, Some(4)));
        assert!(!out.draw.is_empty() && !out.hits.is_empty());
    }

    #[test]
    fn fades_colors() {
        assert_eq!(fade("#ffffff", 0.5), "#ffffff80");
        assert_eq!(fade("#8fc8ff80", 0.5), "#8fc8ff40");
    }
}
