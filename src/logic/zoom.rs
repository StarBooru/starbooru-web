//! The viewer's zoom and pan, and how far one turn of the wheel zooms. Pure
//! maths, ported from the reference with its tests.

pub const MIN_SCALE: f64 = 1.0;
pub const MAX_SCALE: f64 = 8.0;
/// The scale a double click or tap zooms to.
pub const TAP_SCALE: f64 = 2.5;
/// One zoom-in or zoom-out step, for the buttons and the keys.
pub const STEP: f64 = 1.5;
/// One notch of a mouse wheel.
const WHEEL_NOTCH: f64 = 1.1;
/// A pixel delta this big is a whole notch, whatever its size. On Linux,
/// Chromium sends 120 for one and WebKitGTK 80, at any display scale.
const NOTCH_MIN_PIXELS: f64 = 40.0;
/// Smaller pixel deltas (touchpads, smooth wheels) add up to a notch at this.
const PIXELS_PER_NOTCH: f64 = 100.0;
/// Firefox and its forks send lines: 3 for one notch by default, 6 in Zen.
const LINES_PER_NOTCH: f64 = 3.0;

/// A wheel event's vertical travel, in the unit its `deltaMode` says.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WheelDelta {
    Pixels(f64),
    Lines(f64),
    Pages(f64),
}

impl WheelDelta {
    /// From `WheelEvent.deltaMode` (0 pixels, 1 lines, 2 pages) and `deltaY`.
    pub fn new(mode: u32, y: f64) -> Self {
        match mode {
            1 => WheelDelta::Lines(y),
            2 => WheelDelta::Pages(y),
            _ => WheelDelta::Pixels(y),
        }
    }
}

/// The zoom factor of one wheel event, the same for a notch on every browser,
/// platform and display scale. An event counts as one notch at most, so a long
/// swipe or a big delta can't make the zoom jump.
pub fn wheel_factor(delta: WheelDelta) -> f64 {
    let notches = match delta {
        WheelDelta::Pixels(y) if y.abs() >= NOTCH_MIN_PIXELS => y.signum(),
        WheelDelta::Pixels(y) => y / PIXELS_PER_NOTCH,
        WheelDelta::Lines(y) => (y / LINES_PER_NOTCH).clamp(-1.0, 1.0),
        WheelDelta::Pages(y) => y.clamp(-1.0, 1.0),
    };
    // Scrolling down (positive) zooms out.
    WHEEL_NOTCH.powf(-notches)
}

/// The picture's scale and offset, in CSS pixels from the stage's centre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Zoom {
    pub scale: f64,
    pub x: f64,
    pub y: f64,
}

impl Default for Zoom {
    fn default() -> Self {
        Zoom {
            scale: MIN_SCALE,
            x: 0.0,
            y: 0.0,
        }
    }
}

impl Zoom {
    pub fn is_zoomed(&self) -> bool {
        self.scale > MIN_SCALE + 0.001
    }

    /// Scales by `factor`, keeping the point `at` (from the stage's centre)
    /// under the pointer. `bounds` is the picture's size at fit.
    pub fn zoom_at(self, factor: f64, at: (f64, f64), bounds: (f64, f64)) -> Self {
        if !factor.is_finite() || factor <= 0.0 || !at.0.is_finite() || !at.1.is_finite() {
            return self;
        }
        let scale = (self.scale * factor).clamp(MIN_SCALE, MAX_SCALE);
        let k = scale / self.scale;
        Zoom {
            scale,
            x: at.0 - (at.0 - self.x) * k,
            y: at.1 - (at.1 - self.y) * k,
        }
        .clamped(bounds)
    }

    /// Zoomed in: back to fit. At fit: in on `at`.
    pub fn toggle_at(self, at: (f64, f64), bounds: (f64, f64)) -> Self {
        if self.is_zoomed() {
            Zoom::default()
        } else {
            self.zoom_at(TAP_SCALE / self.scale, at, bounds)
        }
    }

    pub fn pan(self, dx: f64, dy: f64, bounds: (f64, f64)) -> Self {
        if !dx.is_finite() || !dy.is_finite() {
            return self;
        }
        Zoom {
            x: self.x + dx,
            y: self.y + dy,
            ..self
        }
        .clamped(bounds)
    }

    /// Keeps the picture over its own place: the offset is at most half of
    /// what the zoom added to its size.
    fn clamped(self, (w, h): (f64, f64)) -> Self {
        if !self.is_zoomed() {
            return Zoom::default();
        }
        let over = self.scale - MIN_SCALE;
        // `max` also turns a NaN size into 0.
        let mx = w.max(0.0) * over / 2.0;
        let my = h.max(0.0) * over / 2.0;
        Zoom {
            x: self.x.clamp(-mx, mx),
            y: self.y.clamp(-my, my),
            ..self
        }
    }

    /// The picture's `style`: the transform, and `--z` for its corners.
    pub fn css(&self) -> String {
        format!(
            "transform: translate({:.1}px, {:.1}px) scale({:.3}); --z: {:.3}",
            self.x, self.y, self.scale, self.scale
        )
    }

    pub fn percent(&self) -> u32 {
        // The scale is clamped to 1..=8, so this always fits.
        (self.scale * 100.0).round() as u32
    }
}

/// Whether a turn of the wheel over the stage zooms the picture rather than
/// scrolling the page. Rolling down, or up from further down the page, moves
/// the page, so what's under the stage can be reached; a picture already
/// zoomed in, or in fullscreen, takes the wheel. Beside the picture the
/// wheel only zooms one that has grown or is in fullscreen. A held Ctrl is a
/// touchpad's pinch.
pub fn wheel_zooms(
    on_picture: bool,
    full: bool,
    zoomed: bool,
    ctrl: bool,
    down: bool,
    page_scrolled: bool,
) -> bool {
    let scrolls = !ctrl && !full && !zoomed && (down || page_scrolled);
    !scrolls && (on_picture || full || zoomed)
}

/// Wheel events this far apart, in milliseconds, are two turns.
const TURN_GAP_MS: f64 = 300.0;

/// One turn of the wheel stays one thing to its end, so a touchpad coasting
/// up to the top of the page doesn't turn a scroll into a zoom.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WheelTurn {
    at: Option<f64>,
    zooms: bool,
}

impl WheelTurn {
    /// Whether the event at `time` (ms) zooms: as the turn it is part of
    /// does, or as `decide` says for a new turn.
    pub fn zooms(&mut self, time: f64, decide: impl FnOnce() -> bool) -> bool {
        let new = self.at.is_none_or(|at| {
            let since = time - at;
            !since.is_finite() || since >= TURN_GAP_MS
        });
        self.at = Some(time);
        if new {
            self.zooms = decide();
        }
        self.zooms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const B: (f64, f64) = (800.0, 600.0);

    #[test]
    fn zooms_around_the_cursor() {
        let z = Zoom::default().zoom_at(2.0, (100.0, 50.0), B);
        assert_eq!(z.scale, 2.0);
        // The point under the cursor stays put: at = t + (at - t0) * k.
        assert_eq!((z.x, z.y), (-100.0, -50.0));
        let centre = Zoom::default().zoom_at(2.0, (0.0, 0.0), B);
        assert_eq!((centre.x, centre.y), (0.0, 0.0));
    }

    #[test]
    fn clamps_scale_and_offset() {
        let z = Zoom::default().zoom_at(100.0, (0.0, 0.0), B);
        assert_eq!(z.scale, MAX_SCALE);
        let out = Zoom::default().zoom_at(0.1, (10.0, 10.0), B);
        assert_eq!(out, Zoom::default());
        let z = Zoom::default()
            .zoom_at(2.0, (0.0, 0.0), B)
            .pan(10_000.0, -10_000.0, B);
        assert_eq!((z.x, z.y), (400.0, -300.0));
        // Not zoomed: panning does nothing.
        assert_eq!(Zoom::default().pan(50.0, 50.0, B), Zoom::default());
    }

    #[test]
    fn toggles() {
        let z = Zoom::default().toggle_at((0.0, 0.0), B);
        assert_eq!(z.scale, TAP_SCALE);
        assert_eq!(z.toggle_at((5.0, 5.0), B), Zoom::default());
    }

    #[test]
    fn ignores_hostile_numbers() {
        let z = Zoom::default().zoom_at(2.0, (0.0, 0.0), B);
        assert_eq!(z.zoom_at(f64::NAN, (0.0, 0.0), B), z);
        assert_eq!(z.zoom_at(-1.0, (0.0, 0.0), B), z);
        assert_eq!(z.zoom_at(2.0, (f64::INFINITY, 0.0), B), z);
        assert_eq!(z.pan(f64::NAN, 1.0, B), z);
        let nan_bounds = z.pan(5.0, 5.0, (f64::NAN, f64::NAN));
        assert_eq!((nan_bounds.x, nan_bounds.y), (0.0, 0.0));
        assert_eq!(z.percent(), 200);
    }

    #[test]
    fn the_style_carries_the_scale_for_the_corners() {
        assert_eq!(
            Zoom::default().css(),
            "transform: translate(0.0px, 0.0px) scale(1.000); --z: 1.000"
        );
        let z = Zoom::default().zoom_at(2.0, (100.0, 50.0), B);
        assert_eq!(
            z.css(),
            "transform: translate(-100.0px, -50.0px) scale(2.000); --z: 2.000"
        );
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn a_wheel_notch_is_the_same_everywhere() {
        // One notch, in the units and sizes engines send.
        for notch in [
            WheelDelta::Pixels(120.0),
            WheelDelta::Pixels(100.0),
            WheelDelta::Pixels(150.0),
            WheelDelta::Pixels(80.0),
            WheelDelta::Lines(3.0),
            WheelDelta::Lines(6.0),
            WheelDelta::Pages(1.0),
        ] {
            assert!(close(wheel_factor(notch), 1.0 / WHEEL_NOTCH), "{notch:?}");
        }
        for notch in [
            WheelDelta::Pixels(-120.0),
            WheelDelta::Pixels(-80.0),
            WheelDelta::Lines(-6.0),
        ] {
            assert!(close(wheel_factor(notch), WHEEL_NOTCH), "{notch:?}");
        }
        assert_eq!(WheelDelta::new(0, 3.0), WheelDelta::Pixels(3.0));
        assert_eq!(WheelDelta::new(1, 3.0), WheelDelta::Lines(3.0));
        assert_eq!(WheelDelta::new(2, 3.0), WheelDelta::Pages(3.0));
        assert_eq!(WheelDelta::new(9, 3.0), WheelDelta::Pixels(3.0));
    }

    #[test]
    fn small_wheel_deltas_add_up_to_a_notch() {
        // A touchpad or smooth wheel: ten events of 10 px make one notch.
        let event = wheel_factor(WheelDelta::Pixels(10.0));
        assert!(close(event.powi(10), 1.0 / WHEEL_NOTCH));
        let event = wheel_factor(WheelDelta::Lines(0.75));
        assert!(close(event.powi(4), 1.0 / WHEEL_NOTCH));
    }

    #[test]
    fn a_wheel_event_never_jumps() {
        assert!(close(
            wheel_factor(WheelDelta::Pixels(-10_000.0)),
            WHEEL_NOTCH
        ));
        assert!(close(wheel_factor(WheelDelta::Lines(-500.0)), WHEEL_NOTCH));
        assert!(close(wheel_factor(WheelDelta::Pages(-9.0)), WHEEL_NOTCH));
    }

    #[test]
    fn a_wheel_event_without_vertical_travel_does_nothing() {
        assert_eq!(wheel_factor(WheelDelta::Pixels(0.0)), 1.0);
        assert_eq!(wheel_factor(WheelDelta::Lines(0.0)), 1.0);
    }

    #[test]
    fn hostile_wheel_deltas_leave_the_zoom_alone() {
        let z = Zoom::default().zoom_at(2.0, (0.0, 0.0), B);
        for delta in [
            WheelDelta::Pixels(f64::NAN),
            WheelDelta::Lines(f64::NAN),
            WheelDelta::Pages(f64::NAN),
        ] {
            assert_eq!(z.zoom_at(wheel_factor(delta), (0.0, 0.0), B), z);
        }
        let huge = wheel_factor(WheelDelta::Pixels(f64::INFINITY));
        assert!(close(huge, 1.0 / WHEEL_NOTCH));
    }

    #[test]
    fn the_wheel_scrolls_the_page_until_the_picture_takes_it() {
        // At the top of the page, rolling up over the picture zooms in;
        // rolling down scrolls on to the details.
        assert!(wheel_zooms(true, false, false, false, false, false));
        assert!(!wheel_zooms(true, false, false, false, true, false));
        // Further down the page, both ways scroll.
        assert!(!wheel_zooms(true, false, false, false, false, true));
        // Zoomed in or in fullscreen, the picture has it either way.
        assert!(wheel_zooms(true, false, true, false, true, true));
        assert!(wheel_zooms(false, true, false, false, true, false));
        // Beside a picture at fit, the page scrolls, or the browser zooms.
        assert!(!wheel_zooms(false, false, false, false, false, false));
        assert!(!wheel_zooms(false, false, false, true, false, false));
        // A pinch on a touchpad over the picture.
        assert!(wheel_zooms(true, false, false, true, true, true));
    }

    #[test]
    fn a_turn_of_the_wheel_stays_what_it_started_as() {
        let mut turn = WheelTurn::default();
        assert!(!turn.zooms(0.0, || false));
        // Still coasting: the page has reached its top, but this turn scrolls.
        assert!(!turn.zooms(120.0, || true));
        assert!(!turn.zooms(400.0, || true));
        // A pause, and the next turn decides afresh.
        assert!(turn.zooms(800.0, || true));
        assert!(turn.zooms(900.0, || false));
        // A clock that went back, or nonsense, starts a turn.
        assert!(!turn.zooms(f64::NAN, || false));
        assert!(turn.zooms(1000.0, || true));
    }
}
