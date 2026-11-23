//! The colour the browser paints its own bars in (`<meta name="theme-color">`):
//! the app's canvas, `--canvas` in `style/tokens.css`. Worked out here as
//! sRGB, which every browser reads in that tag.

use crate::store::Theme;

/// `--canvas` as (lightness, chroma) in OKLCH, light and dark.
const CANVAS_LIGHT: (f64, f64) = (0.985, 0.006);
const CANVAS_DARK: (f64, f64) = (0.165, 0.012);

/// The canvas colours for the light and the dark system setting, in that
/// order. A theme picked in the app uses its colour for both.
pub fn bar_colors(theme: Theme, hue: u16) -> (String, String) {
    let light = canvas(false, hue);
    let dark = canvas(true, hue);
    match theme {
        Theme::System => (light, dark),
        Theme::Light => (light.clone(), light),
        Theme::Dark => (dark.clone(), dark),
    }
}

fn canvas(dark: bool, hue: u16) -> String {
    let (l, c) = if dark { CANVAS_DARK } else { CANVAS_LIGHT };
    oklch_hex(l, c, f64::from(hue))
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
    }

    #[test]
    fn the_bars_follow_the_theme() {
        let (light, dark) = bar_colors(Theme::System, 285);
        assert_ne!(light, dark);
        assert_eq!(bar_colors(Theme::Light, 285), (light.clone(), light));
        assert_eq!(bar_colors(Theme::Dark, 285), (dark.clone(), dark));
        // A near-white and a near-black, tinted by the hue.
        let (light, dark) = bar_colors(Theme::System, 285);
        assert_eq!(light, "#f9fafe");
        assert_eq!(dark, "#0e0e13");
        assert_ne!(bar_colors(Theme::System, 120).0, light);
    }

    #[test]
    fn the_canvas_matches_the_stylesheet() {
        let css = include_str!("../style/tokens.css");
        let (l, c) = CANVAS_LIGHT;
        assert!(css.contains(&format!("--canvas: oklch({l} {c} var(--h));")));
        let (l, c) = CANVAS_DARK;
        assert!(css.contains(&format!("--canvas: oklch({l} {c} var(--h));")));
    }
}
