//! Swiping a gallery tile aside, without a browser: which way a finger is
//! going, how far the picture follows it, and whether letting go does what
//! Settings › Gestures says for that side.

use crate::store::SwipeAction;

/// How far a finger moves before it counts as going somewhere, in pixels.
pub const SLOP: f64 = 10.0;
/// How much more sideways than up or down a move must be to be a swipe.
const SIDEWAYS: f64 = 1.2;
/// A side with nothing to do gives this share of the way, and no more.
const STIFF: f64 = 0.15;
/// A flick this fast, in pixels a millisecond, does it however short.
const FLING_SPEED: f64 = 0.5;
/// …as long as it went at least this far.
const FLING_MIN: f64 = 24.0;
/// The time a flick's speed is measured over, in milliseconds.
const SPEED_WINDOW: f64 = 100.0;

/// The side a tile is swiped to: `Left` when the finger goes left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn of(dx: f64) -> Side {
        if dx < 0.0 { Side::Left } else { Side::Right }
    }

    /// As `.swipe-hint`'s `data-side` and the tile's `data-swiping` say it.
    pub fn as_str(self) -> &'static str {
        match self {
            Side::Left => "left",
            Side::Right => "right",
        }
    }
}

/// What a finger that has moved `dx`, `dy` from where it came down is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    /// Not far enough to tell yet.
    Unknown,
    /// Up or down: the browser's scroll.
    Scroll,
    Sideways,
}

pub fn axis(dx: f64, dy: f64) -> Axis {
    if dx.hypot(dy) < SLOP || !dx.is_finite() || !dy.is_finite() {
        Axis::Unknown
    } else if dx.abs() >= dy.abs() * SIDEWAYS {
        Axis::Sideways
    } else {
        Axis::Scroll
    }
}

/// How far a swipe must go to do its action, for a tile `width` wide.
pub fn reach(width: f64) -> f64 {
    (width * 0.4).clamp(56.0, 120.0)
}

/// How far the picture follows a finger `dx` along, where `can` says whether
/// that side does something.
pub fn follow(dx: f64, can: bool) -> f64 {
    if can { dx } else { dx * STIFF }
}

/// The finger's speed in pixels a millisecond as it let go, from its last
/// positions as `(time, x)`.
pub fn speed(trail: &[(f64, f64)]) -> f64 {
    let Some(&(t1, x1)) = trail.last() else {
        return 0.0;
    };
    let (t0, x0) = trail
        .iter()
        .find(|(t, _)| t1 - t <= SPEED_WINDOW)
        .copied()
        .unwrap_or((t1, x1));
    if t1 > t0 { (x1 - x0) / (t1 - t0) } else { 0.0 }
}

/// The side whose action runs as the finger lets go `dx` along a tile
/// `width` wide at `speed`: far enough, or flicked that way.
pub fn release(dx: f64, width: f64, speed: f64, can: impl Fn(Side) -> bool) -> Option<Side> {
    let side = Side::of(dx);
    let flung = speed.abs() > FLING_SPEED && speed.signum() == dx.signum() && dx.abs() > FLING_MIN;
    (can(side) && (dx.abs() >= reach(width) || flung)).then_some(side)
}

/// What swiping to `side` does, of the two chosen.
pub fn action(side: Side, (left, right): (SwipeAction, SwipeAction)) -> SwipeAction {
    match side {
        Side::Left => left,
        Side::Right => right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_finger_has_to_go_somewhere_and_mostly_sideways() {
        assert_eq!(axis(3.0, 4.0), Axis::Unknown);
        assert_eq!(axis(20.0, 4.0), Axis::Sideways);
        assert_eq!(axis(-20.0, 10.0), Axis::Sideways);
        // About as far down as along: the page's scroll.
        assert_eq!(axis(12.0, 11.0), Axis::Scroll);
        assert_eq!(axis(0.0, -30.0), Axis::Scroll);
        assert_eq!(axis(f64::NAN, 40.0), Axis::Unknown);
    }

    #[test]
    fn a_side_with_nothing_to_do_hardly_gives() {
        assert_eq!(follow(-80.0, true), -80.0);
        assert!((follow(-80.0, false) + 12.0).abs() < 1e-9);
    }

    #[test]
    fn letting_go_far_enough_or_with_a_flick_does_it() {
        let both = |_| true;
        let left_only = |s| s == Side::Left;
        // A 200 px tile needs 80 px; a narrow one 56, a wide one 120.
        assert_eq!(reach(200.0), 80.0);
        assert_eq!(reach(90.0), 56.0);
        assert_eq!(reach(900.0), 120.0);
        assert_eq!(release(-85.0, 200.0, 0.0, both), Some(Side::Left));
        assert_eq!(release(85.0, 200.0, 0.0, both), Some(Side::Right));
        assert_eq!(release(-60.0, 200.0, 0.0, both), None);
        // A flick that way, past a little.
        assert_eq!(release(-30.0, 200.0, -0.8, both), Some(Side::Left));
        assert_eq!(release(-20.0, 200.0, -0.8, both), None);
        // A flick back the other way isn't one.
        assert_eq!(release(-30.0, 200.0, 0.8, both), None);
        // Nothing chosen for that side.
        assert_eq!(release(85.0, 200.0, 0.0, left_only), None);
    }

    #[test]
    fn the_speed_is_the_last_tenth_of_a_second() {
        assert_eq!(speed(&[]), 0.0);
        assert_eq!(speed(&[(5.0, 10.0)]), 0.0);
        // A slow start doesn't count against a fast end.
        let trail = [(0.0, 0.0), (300.0, 10.0), (350.0, 30.0), (400.0, 80.0)];
        assert!((speed(&trail) - 70.0 / 100.0).abs() < 1e-9);
    }

    #[test]
    fn each_side_does_its_own_action() {
        let chosen = (SwipeAction::Delete, SwipeAction::Like);
        assert_eq!(action(Side::Left, chosen), SwipeAction::Delete);
        assert_eq!(action(Side::Right, chosen), SwipeAction::Like);
        assert_eq!(Side::Left.as_str(), "left");
        assert_eq!(Side::Right.as_str(), "right");
        assert_eq!(Side::of(-0.5), Side::Left);
        assert_eq!(Side::of(0.0), Side::Right);
    }
}
