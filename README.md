# Double axis theme

![Rust Percentage](https://img.shields.io/github/languages/top/naisyrkdev/YasaffDoubleAxisTheme?logo=rust&logoColor=white)
[![License](https://img.shields.io/github/license/naisyrkdev/YasaffDoubleAxisTheme)](LICENSE)
![Status](https://img.shields.io/badge/status-alpha-red)
![Status](https://img.shields.io/badge/status-WIP-yellow)

A custom theme for [YASAFF](../YASAFF/), and the example of making one. The categories run along the top, and the games of the selected one are listed below it, as on a PlayStation 3. The last category, **Main**, holds Settings, Discover and Add game.

![Double axis theme](previews/1.png)

YASAFF knows nothing about this layout. The theme brings it as code: `theme.wasm`, built from `src/`, draws the home screen, moves the selection, and declares the options YASAFF shows for it in Settings. YASAFF doesn't ship it. Install it like any theme.

## Install it

1. `./package.sh` builds `theme.wasm` and writes `dist/double_axis_theme.zip`.
2. In YASAFF, open **Settings → Theme → Import theme…** and pick the `.zip`.

The theme is added as a new theme and selected. Importing it twice gives two copies.

To list it on a YASAFF Discover server, submit this repository. Its root is the theme folder Discover expects: `themeSettings.json`, `theme.wasm`, `images/` and `previews/`. The source in `src/` and the build files are allowed next to them. Commit `theme.wasm` after each `./package.sh`, since Discover serves what is in the repository.

Building needs Rust and its WebAssembly target:

```sh
rustup target add wasm32-unknown-unknown
```

## What it does

- Left/Right picks a category, and Up/Down picks a game in it. Each category remembers the game last selected in it. Clicking a category opens it, and clicking a game selects it, or starts it when it is already selected.
- Games without a category are under **Games**. Empty categories are left out.
- A category shows its name, fitted into its box (a long name shrinks, centered and justified), or a picture set in Settings.
- A game shows its picture: a copy of its square cover at first, which can be changed in its **Appearance**. Without a picture, it shows its name. Next to pictures, names show for the selected game only, always, or never. The selected game also shows its status line.
- The selected category and game glow. The halo follows the outline of transparent pictures.
- A green dot marks a running game.

## Options

The theme declares these in `src/describe.rs`, and YASAFF shows them in **Settings → Home screen**:

| Key | Default | Meaning |
|-----|---------|---------|
| `width`, `height` | `220`, `120` | A category's box at 1080p (80–600 and 40–400) |
| `border` | `false` | An outline around each category's box |
| `font` | none | An imported font for category names; none uses the game title font |
| `font_size` | `28` | The largest size of a category's name (10–80); off while `dynamic_font` is on |
| `dynamic_font` | `false` | Each name takes the biggest size that fits its box |
| `text_color` | `#ffffff` | Category and game names |
| `glow_color`, `glow_intensity` | `#8fc8ff`, `0.6` | The glow around the selected category and game (0–1) |
| `images` | | The pictures page: one picture per category, and for Games, Main, Discover, Add game and Settings |
| `list_width`, `list_height` | `104`, `104` | The selected game's picture at 1080p (40–600); the others are a bit smaller |
| `list_align` | `left` | The list under the category's box: `left`, `center` or `right` |
| `title_info` | `selected` | Next to pictures, names show for the `selected` game, `always` or `never` |

The user's values are saved in the theme's `themeSettings.json` under `custom.values`. Pictures are saved in `custom.images`, keyed `category:<name>` (so they fit any profile with a category of that name), `builtin:uncategorized`, `builtin:main`, and `entry:discover`, `entry:add` and `entry:settings`. A package can ship defaults for both, for example:

```json
{
  "name": "Double axis",
  "kind": "custom",
  "custom": {
    "values": { "glow_color": "#ffcc66", "list_align": "center" },
    "images": { "builtin:main": "main.png" }
  }
}
```

with `images/main.png` next to it. The rest of `themeSettings.json` is the look YASAFF draws around the theme: the background, the bars, the buttons and the swap sound.

## The code

| File | Holds |
|------|-------|
| `src/yasaff.rs` | Talks to YASAFF: the input and output types, and the exports `theme.wasm` must have. Copy it unchanged into a new theme. |
| `src/describe.rs` | The options, the pictures page and the game picture, in English, French, Spanish and Polish (`src/i18n.json`) |
| `src/lib.rs` | The layout: the columns, navigation, animation and drawing |

The layout is a type implementing `yasaff::Theme`, exported with `export_theme!`. Each frame, YASAFF sends the screen, the library, the options and what was pressed or clicked. The theme answers with shapes to draw (rectangles, glows, images, text), click areas, and the tile to select or click. [YASAFF's docs/THEMES.md](../YASAFF/docs/THEMES.md) describes the whole protocol.

`cargo test` runs the layout on the host with JSON input, without YASAFF.

## Make your own

1. Copy this repository and rename the crate in `Cargo.toml` and the `.wasm` name in `package.sh`.
2. Keep `src/yasaff.rs`. Replace `src/lib.rs` and `src/describe.rs` with your layout and options.
3. Set `name` in `themeSettings.json`, keep `"kind": "custom"`, and put pictures your code draws with `Source::File` in `images/`.
4. `./package.sh`, then import `dist/*.zip` in YASAFF.
