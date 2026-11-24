//! The accent the palette is built from, and the few colours worked out from
//! it outside the stylesheet. A colour picked as the accent comes down to its
//! OKLCH hue, chroma and lightness, which `style/tokens.css` builds every role
//! from. The browser paints its own bars (`<meta name="theme-color">`) in the
//! app's canvas, worked out here as sRGB, which every browser reads in that
//! tag.

use starbooru_core::color::Rgb;

use crate::store::{Prefs, Theme};

/// What the palette is built on: `--seed-hue`, `--seed-chroma` and
/// `--seed-lightness` on `.app`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Accent {
    /// OKLCH hue in whole degrees.
    pub hue: u16,
    /// A picked colour's own chroma and lightness. A preset has neither and
    /// takes the palette in `tokens.css` as it stands.
    pub picked: Option<Picked>,
}

/// A picked colour as the palette reads it, in OKLCH.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Picked {
    /// 0 for a grey.
    pub chroma: f64,
    pub lightness: f64,
}

impl Accent {
    pub fn preset(hue: u16) -> Self {
        Self { hue, picked: None }
    }

    /// The accent the preferences hold. A grey keeps the hue it was picked
    /// over, which only the tag categories and the sign-in aurora still show.
    pub fn of(prefs: &Prefs) -> Self {
        Self {
            picked: prefs.accent_color.map(picked),
            ..Self::preset(prefs.accent_hue)
        }
    }

    /// How colourful the accent is, from 0 for a grey to 1 for one at least
    /// as colourful as a preset's swatch: `--k` in `tokens.css`.
    pub fn colourful(self) -> f64 {
        self.picked.map_or(1.0, |p| (p.chroma / SWATCH.1).min(1.0))
    }

    /// The custom properties `.app` carries for it.
    pub fn style(self) -> String {
        match self.picked {
            None => format!("--seed-hue: {}", self.hue),
            Some(p) => format!(
                "--seed-hue: {}; --seed-chroma: {}; --seed-lightness: {}",
                self.hue, p.chroma, p.lightness
            ),
        }
    }
}

/// Below this chroma a colour is a grey: its hue is noise.
const MIN_CHROMA: f64 = 0.02;

fn picked(rgb: Rgb) -> Picked {
    let (l, a, b) = oklab(rgb);
    let chroma = a.hypot(b);
    // Three places are plenty, and the stylesheet gets the same numbers.
    let round = |x: f64| (x * 1000.0).round() / 1000.0;
    Picked {
        chroma: if chroma < MIN_CHROMA {
            0.0
        } else {
            round(chroma)
        },
        lightness: round(l.clamp(0.0, 1.0)),
    }
}

/// `--canvas` as (lightness, chroma) in OKLCH, light and dark. The chroma
/// scales with how colourful the accent is.
const CANVAS_LIGHT: (f64, f64) = (0.985, 0.006);
const CANVAS_DARK: (f64, f64) = (0.165, 0.012);
/// The canvas of the OLED mode.
const BLACK: &str = "#000000";

/// The canvas colours for the light and the dark system setting, in that
/// order. A theme picked in the app uses its colour for both. With `oled`
/// the dark one is black.
pub fn bar_colors(theme: Theme, accent: Accent, oled: bool) -> (String, String) {
    let light = canvas(false, accent);
    let dark = if oled {
        BLACK.to_owned()
    } else {
        canvas(true, accent)
    };
    match theme {
        Theme::System => (light, dark),
        Theme::Light => (light.clone(), light),
        Theme::Dark => (dark.clone(), dark),
    }
}

fn canvas(dark: bool, accent: Accent) -> String {
    let (l, c) = if dark { CANVAS_DARK } else { CANVAS_LIGHT };
    oklch_hex(l, c * accent.colourful(), f64::from(accent.hue))
}

/// An OKLCH colour as `#rrggbb`, clipped to what sRGB can show.
pub fn oklch_hex(l: f64, c: f64, hue_degrees: f64) -> String {
    let (sin, cos) = hue_degrees.to_radians().sin_cos();
    let (a, b) = (c * cos, c * sin);
    // OKLab to linear sRGB (Björn Ottosson's matrices).
    let l_ = l + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
    let m_ = l - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
    let s_ = l - 0.089_484_177_5 * a - 1.291_485_548_0 * b;
    let (l3, m3, s3) = (l_.powi(3), m_.powi(3), s_.powi(3));
    let r = 4.076_741_662_1 * l3 - 3.307_711_591_3 * m3 + 0.230_969_929_2 * s3;
    let g = -1.268_438_004_6 * l3 + 2.609_757_401_1 * m3 - 0.341_319_396_5 * s3;
    let b = -0.004_196_086_3 * l3 - 0.703_418_614_7 * m3 + 1.707_614_701_0 * s3;
    format!("#{:02x}{:02x}{:02x}", channel(r), channel(g), channel(b))
}

/// An 8-bit sRGB channel as a linear one.
fn linear(c: u8) -> f64 {
    let c = f64::from(c) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// An sRGB colour in OKLab: lightness, then the two axes of its hue.
fn oklab(rgb: Rgb) -> (f64, f64, f64) {
    let [r, g, b] = rgb.map(linear);
    // Linear sRGB to OKLab (Björn Ottosson's matrices).
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    (
        0.210_454_255_3 * l + 0.793_617_785 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766 * s,
    )
}

/// The OKLCH hue in whole degrees (0–359) of an sRGB colour, or `None` for
/// a grey.
pub fn hue_of(rgb: Rgb) -> Option<u16> {
    let (_, a, b) = oklab(rgb);
    if a.hypot(b) < MIN_CHROMA {
        return None;
    }
    let degrees = b.atan2(a).to_degrees().rem_euclid(360.0).round();
    // In 0..=360 after rem_euclid and round; 360 wraps to 0.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let hue = (degrees as u16) % 360;
    Some(hue)
}

/// The colour an accent preset of `hue` is painted in: `.accent` in
/// `style/pages/settings.css`, from `--swatch-l` and `--swatch-c`.
pub fn swatch_rgb(hue: u16) -> Rgb {
    let hex = oklch_hex(SWATCH.0, SWATCH.1, f64::from(hue));
    starbooru_core::color::parse_hex_color(&hex).unwrap_or([0x80, 0x80, 0x80])
}

/// `--swatch-l` and `--swatch-c` in `style/tokens.css`. The chroma is also
/// the one from which a picked colour counts as fully colourful.
const SWATCH: (f64, f64) = (0.6, 0.16);

/// The relative luminance of an sRGB colour, as WCAG reckons contrast.
fn luminance(rgb: Rgb) -> f64 {
    let [r, g, b] = rgb.map(linear);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Whether a mark on `rgb` reads better dark than white.
pub fn dark_ink(rgb: Rgb) -> bool {
    // Contrast with black against contrast with white.
    let y = luminance(rgb) + 0.05;
    y * y > 0.05 * 1.05
}

/// A linear channel as an 8-bit sRGB one.
fn channel(linear: f64) -> u8 {
    let x = linear.clamp(0.0, 1.0);
    let encoded = if x <= 0.003_130_8 {
        12.92 * x
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    };
    // In 0..=255 after the clamp, so the cast loses nothing.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let byte = (encoded * 255.0).round().clamp(0.0, 255.0) as u8;
    byte
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use starbooru_core::color::{parse_hex_color, wheel_color};

    use super::*;

    #[test]
    fn known_colours_convert() {
        assert_eq!(oklch_hex(1.0, 0.0, 0.0), "#ffffff");
        assert_eq!(oklch_hex(0.0, 0.0, 0.0), "#000000");
        // sRGB's red, green and blue in OKLCH.
        assert_eq!(oklch_hex(0.627_955, 0.257_683, 29.233_885), "#ff0000");
        assert_eq!(oklch_hex(0.866_440, 0.294_827, 142.495_339), "#00ff00");
        assert_eq!(oklch_hex(0.452_014, 0.313_214, 264.052_021), "#0000ff");
        // Out of gamut clips instead of wrapping round.
        assert_eq!(oklch_hex(1.2, 0.0, 0.0), "#ffffff");
        // And back, lightness first.
        let (l, a, b) = oklab([0xff, 0, 0]);
        assert!((l - 0.627_955).abs() < 1e-4, "{l}");
        assert!((a.hypot(b) - 0.257_683).abs() < 1e-4);
        assert!(oklab([0xff, 0xff, 0xff]).0 > 0.999);
    }

    #[test]
    fn the_bars_follow_the_theme() {
        let violet = Accent::preset(285);
        let (light, dark) = bar_colors(Theme::System, violet, false);
        assert_ne!(light, dark);
        assert_eq!(
            bar_colors(Theme::Light, violet, false),
            (light.clone(), light.clone())
        );
        assert_eq!(
            bar_colors(Theme::Dark, violet, false),
            (dark.clone(), dark.clone())
        );
        // A near-white and a near-black, tinted by the hue.
        assert_eq!(light, "#f9fafe");
        assert_eq!(dark, "#0e0e13");
        assert_ne!(
            bar_colors(Theme::System, Accent::preset(120), false).0,
            light
        );
        // A grey takes the tint away.
        let prefs = Prefs {
            accent_color: Some([0x80, 0x80, 0x80]),
            ..Prefs::default()
        };
        let (light, dark) = bar_colors(Theme::System, Accent::of(&prefs), false);
        assert_eq!((light.as_str(), dark.as_str()), ("#fafafa", "#0e0e0e"));
    }

    #[test]
    fn the_bars_are_black_in_oled() {
        let violet = Accent::preset(285);
        let (light, dark) = bar_colors(Theme::System, violet, true);
        assert_eq!((light.as_str(), dark.as_str()), ("#f9fafe", "#000000"));
        assert_eq!(
            bar_colors(Theme::Dark, violet, true),
            ("#000000".to_owned(), "#000000".to_owned())
        );
        // Light has no black to show.
        assert_eq!(
            bar_colors(Theme::Light, violet, true),
            bar_colors(Theme::Light, violet, false)
        );
    }

    #[test]
    fn finds_the_hue_of_a_colour() {
        let near = |rgb, want: u16| {
            let got = hue_of(rgb).unwrap();
            assert!(got.abs_diff(want) <= 1, "{rgb:?}: {got} vs {want}");
        };
        near([0xff, 0, 0], 29);
        near([0, 0xff, 0], 142);
        near([0, 0, 0xff], 264);
        near([0x67, 0x50, 0xa4], 294);
        // Every hue of a saturated wheel lands in 0..360.
        for i in 0..=255u8 {
            assert!(hue_of([0xff, 0, i]).unwrap() < 360);
        }
    }

    #[test]
    fn greys_have_no_hue_and_no_chroma() {
        for grey in [
            [0, 0, 0],
            [0x80, 0x80, 0x80],
            [0xff, 0xff, 0xff],
            [0x7f, 0x80, 0x81],
        ] {
            assert_eq!(hue_of(grey), None, "{grey:?}");
            assert_eq!(picked(grey).chroma, 0.0, "{grey:?}");
        }
        assert_eq!(picked([0, 0, 0]).lightness, 0.0);
        assert_eq!(picked([0xff, 0xff, 0xff]).lightness, 1.0);
        assert_eq!(picked([0x80, 0x80, 0x80]).lightness, 0.6);
    }

    #[test]
    fn a_picked_colour_sets_how_colourful_the_palette_is() {
        let accent = |rgb| Accent {
            hue: hue_of(rgb).unwrap_or(285),
            picked: Some(picked(rgb)),
        };
        // A preset, and anything as colourful as its swatch, is all colour.
        assert_eq!(Accent::preset(285).colourful(), 1.0);
        assert_eq!(accent([0xff, 0, 0]).colourful(), 1.0);
        assert_eq!(accent([0, 0, 0x80]).colourful(), 1.0);
        // A muted colour is calmer, a grey has none at all.
        let sage = accent([0x7a, 0x8a, 0x6e]).colourful();
        assert!((0.2..0.4).contains(&sage), "{sage}");
        assert_eq!(accent([0x80, 0x80, 0x80]).colourful(), 0.0);
        // What the stylesheet is given.
        assert_eq!(Accent::preset(285).style(), "--seed-hue: 285");
        assert_eq!(
            accent([0xff, 0, 0]).style(),
            "--seed-hue: 29; --seed-chroma: 0.258; --seed-lightness: 0.628"
        );
        assert_eq!(
            accent([0xff, 0xff, 0xff]).style(),
            "--seed-hue: 285; --seed-chroma: 0; --seed-lightness: 1"
        );
    }

    #[test]
    fn a_preset_is_drawn_in_its_own_hue() {
        let css = include_str!("../style/tokens.css");
        let (l, c) = SWATCH;
        assert!(css.contains(&format!("--swatch-l: {l};")));
        assert!(css.contains(&format!("--swatch-c: {c};")));
        // Some fall outside sRGB, and clipping them moves the hue a little.
        for hue in [285, 330, 20, 65, 140, 190, 245, 0] {
            let got = hue_of(swatch_rgb(hue)).unwrap();
            let apart = got.abs_diff(hue).min(360 - got.abs_diff(hue));
            assert!(apart <= 10, "{hue}: {got}");
        }
    }

    #[test]
    fn marks_go_dark_on_pale_colours() {
        assert!(dark_ink([0xff, 0xff, 0xff]));
        assert!(dark_ink([0xff, 0xe0, 0x00]));
        assert!(!dark_ink([0, 0, 0]));
        assert!(!dark_ink([0x20, 0x60, 0xe0]));
    }

    // ---------- the palette, as tokens.css builds it ----------

    /// A role's lightness.
    #[derive(Clone, Copy)]
    enum Light {
        Fixed(f64),
        /// The primary's: its own (`.0`), moved by `--pull` towards a picked
        /// colour's lightness, set between `.1` and `.1 + .2`.
        Pulled(f64, f64, f64),
    }

    /// How a role's chroma follows the accent.
    #[derive(Clone, Copy)]
    enum Tint {
        /// Scaled by how colourful the accent is (`--k`).
        Neutral(f64),
        /// The accent's own chroma where that is less (`--c`).
        Accent(f64),
        /// The same with every accent.
        Fixed(f64),
    }

    #[derive(Clone, Copy)]
    struct Role {
        name: &'static str,
        l: Light,
        tint: Tint,
        /// Degrees from the accent's hue.
        turn: u16,
    }

    const fn neutral(name: &'static str, l: f64, c: f64) -> Role {
        Role {
            name,
            l: Light::Fixed(l),
            tint: Tint::Neutral(c),
            turn: 0,
        }
    }

    const fn accent(name: &'static str, l: f64, c: f64) -> Role {
        Role {
            name,
            l: Light::Fixed(l),
            tint: Tint::Accent(c),
            turn: 0,
        }
    }

    const fn category(name: &'static str, l: f64, c: f64, turn: u16) -> Role {
        Role {
            name,
            l: Light::Fixed(l),
            tint: Tint::Fixed(c),
            turn,
        }
    }

    const fn primary(l: (f64, f64, f64), c: f64) -> Role {
        Role {
            name: "primary",
            l: Light::Pulled(l.0, l.1, l.2),
            tint: Tint::Accent(c),
            turn: 0,
        }
    }

    const LIGHT: [Role; 15] = [
        neutral("canvas", 0.985, 0.006),
        neutral("surface", 0.965, 0.014),
        neutral("surface-2", 0.94, 0.02),
        neutral("surface-3", 0.905, 0.028),
        neutral("on-surface", 0.23, 0.02),
        neutral("on-surface-2", 0.43, 0.025),
        neutral("outline", 0.8, 0.025),
        primary((0.5, 0.2, 0.3), 0.17),
        accent("on-primary", 0.99, 0.01),
        accent("primary-container", 0.9, 0.06),
        accent("on-primary-container", 0.28, 0.1),
        category("cat-artist", 0.5, 0.16, 110),
        category("cat-character", 0.5, 0.14, 215),
        category("cat-copyright", 0.5, 0.16, 40),
        category("cat-meta", 0.5, 0.12, 300),
    ];

    const DARK: [Role; 15] = [
        neutral("canvas", 0.165, 0.012),
        neutral("surface", 0.205, 0.016),
        neutral("surface-2", 0.245, 0.02),
        neutral("surface-3", 0.29, 0.024),
        neutral("on-surface", 0.93, 0.012),
        neutral("on-surface-2", 0.77, 0.02),
        neutral("outline", 0.42, 0.025),
        primary((0.8, 0.68, 0.28), 0.11),
        accent("on-primary", 0.25, 0.09),
        accent("primary-container", 0.37, 0.1),
        accent("on-primary-container", 0.92, 0.05),
        category("cat-artist", 0.8, 0.12, 110),
        category("cat-character", 0.8, 0.11, 215),
        category("cat-copyright", 0.8, 0.12, 40),
        category("cat-meta", 0.8, 0.09, 300),
    ];

    /// What the OLED mode changes of the dark roles.
    const OLED: [Role; 4] = [
        category("canvas", 0.0, 0.0, 0),
        neutral("surface", 0.17, 0.014),
        neutral("surface-2", 0.21, 0.018),
        neutral("surface-3", 0.26, 0.022),
    ];

    /// A role as `tokens.css` writes it.
    fn css(role: &Role) -> String {
        let l = match role.l {
            Light::Fixed(l) => l.to_string(),
            Light::Pulled(own, low, span) => {
                format!("calc({own} + var(--pull) * ({low} + {span} * var(--l) - {own}))")
            }
        };
        let c = match role.tint {
            Tint::Neutral(c) => format!("calc({c} * var(--k))"),
            Tint::Accent(c) => format!("min({c}, var(--c))"),
            Tint::Fixed(c) => c.to_string(),
        };
        let h = match (role.turn, role.tint) {
            (0, Tint::Fixed(c)) if c <= 0.0 => "0".to_owned(),
            (0, _) => "var(--h)".to_owned(),
            (turn, _) => format!("calc(var(--h) + {turn})"),
        };
        format!("oklch({l} {c} {h})")
    }

    /// A role in an accent, as the browser works it out.
    fn paint(role: &Role, accent: Accent) -> Rgb {
        let k = accent.colourful();
        // The stylesheet's defaults, for a preset.
        let (own_c, own_l) = accent
            .picked
            .map_or((1.0, 0.5), |p| (p.chroma, p.lightness));
        let l = match role.l {
            Light::Fixed(l) => l,
            Light::Pulled(own, low, span) => own + (1.0 - k) * (low + span * own_l - own),
        };
        let c = match role.tint {
            Tint::Neutral(c) => c * k,
            Tint::Accent(c) => c.min(own_c),
            Tint::Fixed(c) => c,
        };
        let hue = f64::from(accent.hue) + f64::from(role.turn);
        parse_hex_color(&oklch_hex(l, c, hue)).unwrap()
    }

    #[derive(Clone, Copy, Debug)]
    enum Look {
        Light,
        Dark,
        Oled,
    }

    fn palette(look: Look, accent: Accent) -> HashMap<&'static str, Rgb> {
        let roles: Vec<Role> = match look {
            Look::Light => LIGHT.to_vec(),
            Look::Dark => DARK.to_vec(),
            Look::Oled => DARK.iter().chain(&OLED).copied().collect(),
        };
        roles.iter().map(|r| (r.name, paint(r, accent))).collect()
    }

    /// WCAG's contrast ratio.
    fn contrast(a: Rgb, b: Rgb) -> f64 {
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// The value of `--name` in the first rule of `css` that opens with
    /// `selector {`.
    fn declared<'a>(css: &'a str, selector: &str, name: &str) -> &'a str {
        let (_, body) = css
            .split_once(&format!("{selector} {{"))
            .unwrap_or_else(|| panic!("{selector}"));
        let body = body.split_once('}').map_or(body, |(b, _)| b);
        let (_, value) = body
            .split_once(&format!(" --{name}: "))
            .unwrap_or_else(|| panic!("--{name} in {selector}"));
        value.split_once(';').unwrap().0
    }

    #[test]
    fn the_palette_matches_the_stylesheet() {
        let tokens = include_str!("../style/tokens.css");
        // The light and dark roles are also those of the theme's pictures in
        // Settings.
        let light = "\n.app, .scheme-light";
        let dark = r#".app[data-theme="dark"], .scheme-dark"#;
        let blocks: [(&str, &[Role]); 5] = [
            (light, &LIGHT),
            (dark, &DARK),
            (r#".app[data-theme="system"]"#, &DARK),
            (
                r#".app[data-theme="dark"][data-oled], .scheme-dark.oled"#,
                &OLED,
            ),
            (r#".app[data-theme="system"][data-oled]"#, &OLED),
        ];
        for (selector, roles) in blocks {
            for role in roles {
                assert_eq!(
                    declared(tokens, selector, role.name),
                    css(role),
                    "--{} in {selector}",
                    role.name
                );
            }
        }
        // The accent's own numbers, with what a preset leaves them at.
        let base = "\n.app";
        assert_eq!(declared(tokens, base, "c"), "var(--seed-chroma, 1)");
        assert_eq!(declared(tokens, base, "l"), "var(--seed-lightness, 0.5)");
        assert_eq!(
            declared(tokens, base, "k"),
            format!("min(1, var(--c) / {})", SWATCH.1)
        );
        assert_eq!(declared(tokens, base, "pull"), "calc(1 - var(--k))");
        // The canvas the bars are painted in.
        let canvas = |l: f64, c: f64| format!("oklch({l} calc({c} * var(--k)) var(--h))");
        assert_eq!(
            declared(tokens, light, "canvas"),
            canvas(CANVAS_LIGHT.0, CANVAS_LIGHT.1)
        );
        assert_eq!(
            declared(tokens, dark, "canvas"),
            canvas(CANVAS_DARK.0, CANVAS_DARK.1)
        );
    }

    #[test]
    fn presets_look_as_they_always_have() {
        // Before picked colours had a say, every role was its own lightness
        // and chroma in the preset's hue.
        let fixed = |role: &Role| {
            let l = match role.l {
                Light::Fixed(l) | Light::Pulled(l, _, _) => l,
            };
            let (Tint::Neutral(c) | Tint::Accent(c) | Tint::Fixed(c)) = role.tint;
            (l, c)
        };
        for hue in (0..360).step_by(5) {
            let preset = Accent::preset(hue);
            for role in LIGHT.iter().chain(&DARK) {
                let (l, c) = fixed(role);
                let was = oklch_hex(l, c, f64::from(hue) + f64::from(role.turn));
                assert_eq!(paint(role, preset), parse_hex_color(&was).unwrap());
            }
        }
    }

    #[test]
    fn a_grey_makes_a_palette_without_a_tint() {
        for rgb in [[0, 0, 0], [0x80, 0x80, 0x80], [0xff, 0xff, 0xff]] {
            let grey = Accent {
                hue: 285,
                picked: Some(picked(rgb)),
            };
            for look in [Look::Light, Look::Dark, Look::Oled] {
                for (name, [r, g, b]) in palette(look, grey) {
                    if name.starts_with("cat-") {
                        continue;
                    }
                    let spread = r.max(g).max(b) - r.min(g).min(b);
                    assert!(spread <= 1, "{rgb:?} {look:?} --{name}: {r} {g} {b}");
                }
            }
        }
    }

    #[test]
    fn white_and_black_are_told_apart_and_stay_readable() {
        let pick = |rgb| Accent {
            hue: 285,
            picked: Some(picked(rgb)),
        };
        let primary = |look, rgb| palette(look, pick(rgb))["primary"];
        // Light can't show a white primary on its white canvas: white is the
        // lightest grey that white text still reads on, black is near black.
        assert_eq!(primary(Look::Light, [0xff; 3]), [0x63, 0x63, 0x63]);
        assert_eq!(primary(Look::Light, [0x80; 3]), [0x42, 0x42, 0x42]);
        assert_eq!(primary(Look::Light, [0; 3]), [0x16, 0x16, 0x16]);
        // In dark, the other way round: black is the darkest grey that dark
        // text reads on, and white is nearly white.
        assert_eq!(primary(Look::Dark, [0; 3]), [0x98, 0x98, 0x98]);
        assert_eq!(primary(Look::Dark, [0x80; 3]), [0xcd, 0xcd, 0xcd]);
        assert_eq!(primary(Look::Dark, [0xff; 3]), [0xf2, 0xf2, 0xf2]);
    }

    /// Every preset, and picks from every corner of what the picker makes:
    /// greys, the wheel's edge and halfway in, and a grid of hex colours.
    fn accents() -> Vec<Accent> {
        let mut all: Vec<Accent> = (0..360).step_by(5).map(Accent::preset).collect();
        let mut rgbs: Vec<Rgb> = (0..=255u8).step_by(17).map(|v| [v, v, v]).collect();
        for step in 0..36 {
            let angle = f64::from(step) * 10f64.to_radians();
            for radius in [1.0, 0.5, 0.2] {
                rgbs.push(wheel_color(radius * angle.sin(), -radius * angle.cos()));
            }
        }
        let levels = [0, 0x40, 0x80, 0xc0, 0xff];
        for r in levels {
            for g in levels {
                for b in levels {
                    rgbs.push([r, g, b]);
                }
            }
        }
        // Muted, pastel and dark ones.
        rgbs.extend([
            [0x7a, 0x8a, 0x6e],
            [0x8a, 0x7f, 0x99],
            [0xff, 0xc0, 0xc0],
            [0xc0, 0xe0, 0xff],
            [0x40, 0x00, 0x00],
            [0x00, 0x00, 0x80],
            [0x00, 0xa0, 0xa0],
        ]);
        all.extend(rgbs.into_iter().map(|rgb| Accent {
            hue: hue_of(rgb).unwrap_or(285),
            picked: Some(picked(rgb)),
        }));
        all
    }

    #[test]
    fn every_accent_stays_readable() {
        // (what, on, under, least contrast)
        let pairs = [
            ("text on the primary", "on-primary", "primary", 4.5),
            ("the primary as text", "primary", "canvas", 4.5),
            ("a focus ring on a card", "primary", "surface", 3.0),
            ("a switch's track", "primary", "surface-3", 3.0),
            ("a chosen chip's edge", "primary", "primary-container", 3.0),
            (
                "a chosen chip",
                "on-primary-container",
                "primary-container",
                4.5,
            ),
            ("a chip", "on-surface", "surface-2", 4.5),
            ("a switch that's off", "on-surface-2", "surface-3", 3.0),
            // The presets' own least, in light.
            ("an artist's chip", "cat-artist", "surface-2", 4.0),
            ("a character's chip", "cat-character", "surface-2", 4.0),
            ("a copyright's chip", "cat-copyright", "surface-2", 4.0),
            ("a meta tag's chip", "cat-meta", "surface-2", 4.0),
        ];
        for accent in accents() {
            for look in [Look::Light, Look::Dark, Look::Oled] {
                let colours = palette(look, accent);
                for (what, on, under, least) in pairs {
                    let ratio = contrast(colours[on], colours[under]);
                    assert!(
                        ratio >= least,
                        "{what} in {look:?}, {accent:?}: {ratio:.2} ({:?} on {:?})",
                        colours[on],
                        colours[under]
                    );
                }
            }
        }
    }

    #[test]
    fn oled_is_black_with_surfaces_that_stand_apart() {
        let lightness = |rgb| oklab(rgb).0;
        for accent in accents() {
            let oled = palette(Look::Oled, accent);
            assert_eq!(oled["canvas"], [0, 0, 0]);
            // A card stands out from the black, and a chip or a track from
            // its card, by as much as the dark theme's do.
            let dark = palette(Look::Dark, accent);
            let step = |p: &HashMap<_, _>, a, b| lightness(p[a]) - lightness(p[b]);
            assert!(step(&oled, "surface", "canvas") >= step(&dark, "surface", "canvas"));
            assert!(step(&oled, "surface-2", "surface") >= 0.035);
            assert!(step(&oled, "surface-3", "surface-2") >= 0.035);
            // Edges show on the cards as well as in dark.
            assert!(
                contrast(oled["outline"], oled["surface"])
                    >= contrast(dark["outline"], dark["surface"])
            );
        }
    }
}
