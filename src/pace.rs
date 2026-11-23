//! Effects the device can afford.
//!
//! The glass of the bars, the entrance of new thumbnails and the glow behind
//! a picture cost little on a desktop and too much on a small board or an old
//! phone, where they make scrolling stutter. Nothing a page can read says
//! which kind of device it is on, so the app times the frames of real
//! scrolling instead. A device that keeps losing frames gets the reduced
//! effects from then on; the setting in Settings › Appearance overrides the
//! measurement either way. While they are reduced, `.app` carries
//! `data-effects="lite"` and the stylesheets do the rest.

use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;

use crate::dom;
use crate::store::{self, Effects};

/// One stretch of scrolling as the page timed it: how many frames followed
/// its first, how long they took, and the time between two frames once the
/// page had come to rest, which is what the screen can show.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Burst {
    pub frames: f64,
    pub ms: f64,
    pub frame_ms: f64,
}

/// A shorter stretch than this many frames says too little.
const MIN_FRAMES: f64 = 20.0;
/// The share of frames a stretch can lose and still count as smooth.
const ROUGH: f64 = 0.25;
/// Rough stretches, less the smooth ones in between, before the effects go.
/// One bad stretch can be a page of results arriving or another program.
const STRIKES: u8 = 3;
/// A gap shorter than this is two callbacks of one frame: no screen
/// refreshes faster than 250 times a second.
const SHORTEST_FRAME_MS: f64 = 4.0;
/// Scrolling is over once this long passes without a scroll event.
const SCROLL_GAP_MS: f64 = 150.0;
/// Frames timed at rest after a stretch, for the screen's own pace.
const REST_FRAMES: u32 = 8;

/// Whether a stretch lost too many of the frames its screen could have
/// shown. `None` if it was too short to tell, or the numbers make no sense.
pub fn rough(b: Burst) -> Option<bool> {
    let sane = b.frames.is_finite() && b.ms.is_finite() && b.frame_ms.is_finite();
    if !sane || b.frames < 0.0 || b.frame_ms < SHORTEST_FRAME_MS {
        return None;
    }
    let possible = b.ms / b.frame_ms;
    (possible >= MIN_FRAMES).then(|| 1.0 - b.frames / possible > ROUGH)
}

/// The stretches seen so far.
#[derive(Debug, Default)]
pub struct Pace {
    strikes: u8,
}

impl Pace {
    /// Takes one stretch. True once the device has shown it can't keep up.
    pub fn take(&mut self, b: Burst) -> bool {
        match rough(b) {
            Some(true) => self.strikes = self.strikes.saturating_add(1),
            Some(false) => self.strikes = self.strikes.saturating_sub(1),
            None => {}
        }
        self.strikes >= STRIKES
    }
}

/// Whether the effects are reduced, for a setting and what was measured.
pub fn reduced(setting: Effects, measured_slow: bool) -> bool {
    match setting {
        Effects::Lite => true,
        Effects::Full => false,
        Effects::Auto => measured_slow,
    }
}

/// Whether the effects are reduced now. Read inside a view, it redraws when
/// that changes.
pub fn lite() -> bool {
    let prefs = store::prefs();
    reduced(prefs.effects, prefs.effects_reduced)
}

/// One stretch being timed, frame by frame: from the first scroll event until
/// scrolling stops, then [`REST_FRAMES`] frames of the page at rest. Those
/// say how fast the screen is right now, which a number kept from earlier
/// wouldn't after a change of monitor or of power profile. Scrolling again
/// meanwhile carries the stretch on.
#[derive(Debug)]
struct Stretch {
    count: u32,
    rest: u32,
    first: f64,
    end: f64,
    last: f64,
    quickest: f64,
}

impl Default for Stretch {
    fn default() -> Self {
        Self {
            count: 0,
            rest: 0,
            first: 0.0,
            end: 0.0,
            last: 0.0,
            quickest: f64::INFINITY,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Step {
    Next,
    /// Over, with what it measured if it measured anything.
    Done(Option<Burst>),
}

impl Stretch {
    fn frame(&mut self, now: f64, scrolling: bool) -> Step {
        if scrolling {
            // The frames at rest so far were part of it.
            if self.count == 0 {
                self.first = now;
            }
            self.count = self.count.saturating_add(1).saturating_add(self.rest);
            self.rest = 0;
            self.quickest = f64::INFINITY;
            self.end = now;
            self.last = now;
            return Step::Next;
        }
        self.rest += 1;
        // The first gap at rest still ends the scrolling.
        if self.rest > 1 {
            self.quickest = self.quickest.min(now - self.last);
        }
        self.last = now;
        if self.rest <= REST_FRAMES {
            return Step::Next;
        }
        let burst = (self.count > 1 && self.quickest.is_finite()).then(|| Burst {
            frames: f64::from(self.count - 1),
            ms: self.end - self.first,
            frame_ms: self.quickest,
        });
        Step::Done(burst)
    }
}

/// What the watcher keeps between events.
#[derive(Default)]
struct Watcher {
    /// When scrolling counts as stopped, unless another event comes.
    until: f64,
    running: bool,
    stretch: Stretch,
    pace: Pace,
}

fn now() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map_or(0.0, |p| p.now())
}

/// Starts timing. Call once, from the root.
pub fn watch() {
    // Only Automatic measures, and only until it has its answer.
    let measuring = Memo::new(|_| {
        let prefs = store::prefs();
        prefs.effects == Effects::Auto && !prefs.effects_reduced
    });
    let Some(window) = web_sys::window() else {
        return;
    };
    let state = Rc::new(RefCell::new(Watcher::default()));
    // In the capture phase, so any scroller counts, not just the page.
    let watch = dom::listen_capture(&window, "scroll", move |_| {
        let mut w = state.borrow_mut();
        w.until = now() + SCROLL_GAP_MS;
        if w.running || !measuring.get_untracked() {
            return;
        }
        w.running = true;
        w.stretch = Stretch::default();
        drop(w);
        next_frame(Rc::clone(&state), measuring);
    });
    on_cleanup(move || watch.stop());
}

fn next_frame(state: Rc<RefCell<Watcher>>, measuring: Memo<bool>) {
    request_animation_frame(move || {
        let t = now();
        let step = {
            let mut w = state.borrow_mut();
            let scrolling = t < w.until;
            w.stretch.frame(t, scrolling)
        };
        let Step::Done(burst) = step else {
            next_frame(state, measuring);
            return;
        };
        let slow = {
            let mut w = state.borrow_mut();
            w.running = false;
            if measuring.get_untracked() {
                burst.is_some_and(|b| w.pace.take(b))
            } else {
                w.pace = Pace::default();
                false
            }
        };
        if slow {
            state.borrow_mut().pace = Pace::default();
            store::update_prefs(|p| p.effects_reduced = true);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn burst(frames: f64, ms: f64, frame_ms: f64) -> Burst {
        Burst {
            frames,
            ms,
            frame_ms,
        }
    }

    /// A second of scrolling on a 60 Hz screen that drew `frames` frames.
    fn second(frames: f64) -> Burst {
        burst(frames, 1000.0, 1000.0 / 60.0)
    }

    #[test]
    fn a_stretch_is_rough_past_a_quarter_of_its_frames_lost() {
        assert_eq!(rough(burst(60.0, 1000.0, 16.7)), Some(false));
        assert_eq!(rough(burst(50.0, 1000.0, 16.7)), Some(false));
        assert_eq!(rough(burst(40.0, 1000.0, 16.7)), Some(true));
        // A 30 Hz screen at full speed.
        assert_eq!(rough(burst(30.0, 1000.0, 33.3)), Some(false));
        // Too short to say.
        assert_eq!(rough(burst(2.0, 200.0, 16.7)), None);
    }

    #[test]
    fn hostile_numbers_say_nothing() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(rough(burst(bad, 1000.0, 16.7)), None);
            assert_eq!(rough(burst(30.0, bad, 16.7)), None);
            assert_eq!(rough(burst(30.0, 1000.0, bad)), None);
        }
        assert_eq!(rough(burst(-5.0, 1000.0, 16.7)), None);
        assert_eq!(rough(burst(30.0, 1000.0, 0.0)), None);
        let mut pace = Pace::default();
        for _ in 0..50 {
            assert!(!pace.take(burst(f64::NAN, f64::INFINITY, -1.0)));
            assert!(!pace.take(burst(1.0, 1e300, 5.0e-300)));
        }
    }

    #[test]
    fn three_rough_stretches_reduce_the_effects_and_smooth_ones_forgive() {
        let mut pace = Pace::default();
        assert!(!pace.take(second(35.0)));
        assert!(!pace.take(second(35.0)));
        assert!(pace.take(second(35.0)));
        let mut pace = Pace::default();
        for _ in 0..50 {
            assert!(!pace.take(second(30.0)));
            assert!(!pace.take(second(60.0)));
        }
    }

    #[test]
    fn the_setting_overrides_the_measurement() {
        assert!(reduced(Effects::Lite, false));
        assert!(!reduced(Effects::Full, true));
        assert!(reduced(Effects::Auto, true));
        assert!(!reduced(Effects::Auto, false));
    }

    #[test]
    fn a_stretch_counts_its_frames_and_the_screens_pace_at_rest() {
        let mut s = Stretch::default();
        // Ten frames of scrolling, 20 ms apart, then the page at rest on a
        // 60 Hz screen.
        let mut t = 0.0;
        for _ in 0..10 {
            assert_eq!(s.frame(t, true), Step::Next);
            t += 20.0;
        }
        let mut step = Step::Next;
        for _ in 0..=REST_FRAMES {
            step = s.frame(t, false);
            t += 16.0;
        }
        assert_eq!(
            step,
            Step::Done(Some(Burst {
                frames: 9.0,
                ms: 180.0,
                frame_ms: 16.0,
            }))
        );
        // A single scroll event measures nothing.
        let mut s = Stretch::default();
        assert_eq!(s.frame(0.0, true), Step::Next);
        let mut step = Step::Next;
        for i in 1..=REST_FRAMES + 1 {
            step = s.frame(f64::from(i) * 16.0, false);
        }
        assert_eq!(step, Step::Done(None));
    }

    #[test]
    fn scrolling_again_at_rest_carries_the_stretch_on() {
        let mut s = Stretch::default();
        s.frame(0.0, true);
        s.frame(16.0, true);
        s.frame(32.0, false);
        s.frame(48.0, false);
        s.frame(64.0, true);
        assert_eq!(s.count, 5);
        assert_eq!(s.rest, 0);
        assert!(s.quickest.is_infinite());
    }

    #[test]
    fn reduced_effects_leave_out_what_costs_most() {
        let tokens = include_str!("../style/tokens.css");
        assert!(tokens.contains(
            r#".app[data-effects="lite"] .glass { background: oklch(from var(--glass) l c h / 1); backdrop-filter: none; }"#
        ));
        let gallery = include_str!("../style/pages/gallery.css");
        assert!(
            gallery
                .contains(r#".thumb.still, .app[data-effects="lite"] .thumb { animation: none; }"#)
        );
        let post = include_str!("../style/pages/post.css");
        assert!(post.contains(r#".app[data-effects="lite"] .post-glow { display: none; }"#));
        let app = include_str!("app.rs");
        assert!(app.contains("data-effects=move || if lite.get() { \"lite\" } else { \"full\" }"));
    }
}
