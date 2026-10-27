//! The theme's options in Settings, its pictures page and the game picture, in the user's language.

use serde_json::{Value, json};
use std::collections::HashMap;

/// The theme's texts in French, Spanish and Polish; English is the key.
fn translations() -> &'static HashMap<String, HashMap<String, String>> {
    static TABLE: std::sync::OnceLock<HashMap<String, HashMap<String, String>>> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| serde_json::from_str(include_str!("i18n.json")).expect("i18n.json is valid"))
}

pub fn tr(language: &str, english: &str) -> String {
    translations().get(language).and_then(|t| t.get(english)).cloned().unwrap_or_else(|| english.to_owned())
}

pub fn manifest(language: &str) -> Value {
    let t = |s: &str| tr(language, s);
    let steps = t("At 1080p. Left/Right steps, Enter to type a value");
    let entry = |key: &str, label: &str| {
        json!({
            "key": format!("entry:{key}"),
            "label": t(label),
            "size": ["list_width", "list_height"],
            "empty": t("None. The tile icon is shown"),
        })
    };
    let mut discover = entry("discover", "Discover");
    discover["section"] = t("Main entries").into();
    discover["section_description"] = t("Pictures for Discover, Add game and Settings in the list, in place of their tile icons").into();
    json!({
        "options": [
            {
                "key": "width", "type": "number", "label": t("Category width"), "description": steps,
                "section": t("Double axis mode"), "section_description": t("Categories side by side, the games of the selected one below it"),
                "min": 80, "max": 600, "step": 10, "default": 220, "unit": "px",
            },
            { "key": "height", "type": "number", "label": t("Category height"), "description": steps, "min": 40, "max": 400, "step": 10, "default": 120, "unit": "px" },
            { "key": "border", "type": "toggle", "label": t("Border around categories"), "description": t("Off shows only the name, centered in the size above"), "default": false },
            { "key": "font", "type": "font", "label": t("Category font"), "description": t("Left/Right switches between imported fonts") },
            {
                "key": "font_size", "type": "number", "label": t("Category font size"), "description": t("The biggest size. Longer names shrink to fit their box"),
                "min": 10, "max": 80, "step": 2, "default": 28, "unit": "px", "disabled_by": "dynamic_font",
            },
            { "key": "dynamic_font", "type": "toggle", "label": t("Dynamic font size"), "description": t("Each name takes the biggest size that fits its box"), "default": false },
            { "key": "text_color", "type": "color", "label": t("Text color"), "description": t("Category and game names. Enter to type a hex value"), "default": "#ffffff" },
            { "key": "glow_color", "type": "color", "label": t("Glow color"), "description": t("Around the selected category and game. Enter to type a hex value"), "default": "#8fc8ff" },
            {
                "key": "glow_intensity", "type": "number", "label": t("Glow intensity"), "description": t("How bright and wide the glow is"),
                "min": 0, "max": 1, "step": 0.1, "default": 0.6, "unit": "%", "display_scale": 100, "zero_label": t("Off"),
            },
            {
                "key": "images", "type": "pictures", "label": t("Category images…"), "description": t("An image shown instead of a category's name"),
                "categories": true, "size": ["width", "height"], "empty": t("None. The name is shown"),
                "slots": [
                    {
                        "key": "builtin:uncategorized", "label": t("Games"), "section": t("Built-in"),
                        "section_description": t("Games holds the games without a category. Main holds Settings, Discover and Add game"),
                    },
                    { "key": "builtin:main", "label": t("Main") },
                    discover,
                    entry("add", "Add game"),
                    entry("settings", "Settings"),
                ],
            },
            {
                "key": "list_width", "type": "number", "label": t("Image width"), "description": t("Of the selected game, at 1080p. Left/Right steps, Enter to type a value"),
                "section": t("Game list"), "section_description": t("The games of the selected category, listed below it"),
                "min": 40, "max": 600, "step": 8, "default": 104, "unit": "px",
            },
            {
                "key": "list_height", "type": "number", "label": t("Image height"), "description": t("Of the selected game, at 1080p. The others are a bit smaller"),
                "min": 40, "max": 600, "step": 8, "default": 104, "unit": "px",
            },
            {
                "key": "list_align", "type": "choice", "label": t("Alignment"), "description": t("Under the selected category's box"), "default": "left",
                "choices": [{ "value": "left", "label": t("Left") }, { "value": "center", "label": t("Center") }, { "value": "right", "label": t("Right") }],
            },
            {
                "key": "title_info", "type": "choice", "label": t("Show game names next to images"), "description": t("A game without an image always shows its name"),
                "default": "selected",
                "choices": [
                    { "value": "selected", "label": t("Selected game only") },
                    { "value": "always", "label": t("Always") },
                    { "value": "never", "label": t("Never") },
                ],
            },
        ],
        "game_picture": {
            "label": t("List image"),
            "description": t("Shown in the game list. Starts as a copy of the cover"),
            "section": t("Double axis"),
            "section_description": t("Without an image the list shows the game's name"),
            "size": ["list_width", "list_height"],
            "seed_from_cover": true,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_text_is_translated() {
        let english = manifest("english").to_string();
        for language in ["french", "spanish", "polish"] {
            let table = &translations()[language];
            assert!(table.values().all(|v| !v.is_empty()), "{language}");
            assert_ne!(manifest(language).to_string(), english, "{language}");
        }
        for key in translations()["polish"].keys() {
            assert!(english.contains(&serde_json::to_string(key).unwrap()), "{key} isn't used");
        }
    }
}
