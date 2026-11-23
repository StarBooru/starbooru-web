//! The colour the browser paints its own bars in (`<meta name="theme-color">`):
//! the app's canvas, `--canvas` in `style/tokens.css`. Worked out here as
//! sRGB, which every browser reads in that tag. Also the way back, from a
//! colour picked as the accent to the hue the palette is built on.

use starbooru_core::color::Rgb;

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

/// Below this chroma a colour is a grey, with no hue to build a palette on.
const MIN_CHROMA: f64 = 0.02;

/// The OKLCH hue in whole degrees (0–359) of an sRGB colour, or `None` for
/// a grey. The palette is built from a hue, so this is what a colour picked
/// as the accent comes down to.
pub fn hue_of(rgb: Rgb) -> Option<u16> {
    let [r, g, b] = rgb.map(|c| {
        let c = f64::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    });
    // Linear sRGB to OKLab (Björn Ottosson's matrices).
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    let a = 1.977_998_495_1 * l - 2.428_592_205 * m + 0.450_593_709_9 * s;
    let b = 0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766 * s;
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

/// `--swatch-l` and `--swatch-c` in `style/tokens.css`.
const SWATCH: (f64, f64) = (0.6, 0.16);

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
    fn greys_have_no_hue() {
        for grey in [
            [0, 0, 0],
            [0x80, 0x80, 0x80],
            [0xff, 0xff, 0xff],
            [0x7f, 0x80, 0x81],
        ] {
            assert_eq!(hue_of(grey), None, "{grey:?}");
        }
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
    fn the_canvas_matches_the_stylesheet() {
        let css = include_str!("../style/tokens.css");
        let (l, c) = CANVAS_LIGHT;
        assert!(css.contains(&format!("--canvas: oklch({l} {c} var(--h));")));
        let (l, c) = CANVAS_DARK;
        assert!(css.contains(&format!("--canvas: oklch({l} {c} var(--h));")));
    }
}
