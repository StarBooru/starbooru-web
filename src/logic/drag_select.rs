//! Picking gallery tiles by dragging across them, without a browser: which
//! presses can start a drag, what a drag picks or lets go of, and how fast
//! the page scrolls under a pointer near its edges.

use super::bulk::Selection;

/// How far a mouse or a pen moves on a tile before the press is a drag
/// rather than a click, in pixels.
pub const DRAG_SLOP: f64 = 6.0;
/// How long a finger rests on a tile before moving it drags, in
/// milliseconds: about a phone's long press.
pub const HOLD_MS: u64 = 450;
/// The fastest the page scrolls under a pointer at an edge, in pixels a
/// second.
const EDGE_SPEED: f64 = 1600.0;
/// A frame longer than this, after a stall, scrolls no further than one this
/// long would, so the page doesn't jump.
const LONGEST_FRAME_MS: f64 = 50.0;

/// How a press on a tile turns into a drag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    /// As soon as it moves: a mouse or a pen.
    OnMove,
    /// Once it has rested on the tile: a finger, whose moving at once
    /// scrolls the page or swipes the tile.
    OnHold,
}

/// How a press with `pointer` (a pointer event's `pointerType`) and `button`
/// can start a drag. A finger only can in selection mode, where its long press
/// no longer opens the post's menu. A modifier key already means something to
/// a click.
pub fn start(pointer: &str, button: i16, modifiers: bool, selecting: bool) -> Option<Start> {
    if modifiers || button != 0 {
        return None;
    }
    match pointer {
        "touch" => selecting.then_some(Start::OnHold),
        // A mouse, a pen, or a browser that doesn't say.
        _ => Some(Start::OnMove),
    }
}

/// Whether a pointer `dx`, `dy` from where it came down has gone `slop` or
/// further.
pub fn moved(dx: f64, dy: f64, slop: f64) -> bool {
    dx.hypot(dy) >= slop
}

/// A drag across the tiles: the post it started on, the post under the
/// pointer now, whether it picks the posts between or lets them go, and the
/// selection before it.
#[derive(Clone, Debug, PartialEq)]
pub struct Drag {
    from: i64,
    to: i64,
    adding: bool,
    before: Selection,
}

impl Drag {
    /// A drag from post `id`. It lets go of posts when that one is picked
    /// already, and picks them otherwise.
    pub fn new(before: Selection, id: i64) -> Self {
        Self {
            from: id,
            to: id,
            adding: !before.contains(id),
            before,
        }
    }

    pub fn before(&self) -> &Selection {
        &self.before
    }

    /// The pointer is over post `id` now. Whether that moves the range's end.
    pub fn reach(&mut self, id: i64) -> bool {
        let moved = self.to != id;
        self.to = id;
        moved
    }

    /// The selection as the drag leaves it, of the posts `loaded` in the
    /// gallery's order: the one before, with every post from the first to
    /// the one under the pointer picked or let go, as a Shift-click would.
    /// Posts the range has moved off are as they were. `None` once the post it
    /// started on has left the list.
    pub fn selection(&self, loaded: &[i64]) -> Option<Selection> {
        let from = loaded.iter().position(|p| *p == self.from)?;
        let to = loaded.iter().position(|p| *p == self.to).unwrap_or(from);
        let mut picked = self.before.clone();
        picked.set_range(loaded, from, to, self.adding);
        Some(picked)
    }
}

/// The band along the top and the bottom of a part of the window `tall`
/// pixels high where a drag scrolls the page.
fn band(tall: f64) -> f64 {
    (tall * 0.12).clamp(32.0, 96.0).min(tall / 2.0)
}

/// How fast the page scrolls under a pointer at `y` during a drag, with
/// tiles in sight from `top` to `bottom` of the window, in pixels a second:
/// up (below zero) near the top and down near the bottom, faster the closer
/// to the edge, and fastest at it or past it.
pub fn edge_speed(y: f64, top: f64, bottom: f64) -> f64 {
    if !(y.is_finite() && top.is_finite() && bottom.is_finite()) {
        return 0.0;
    }
    let band = band(bottom - top);
    if band <= 0.0 {
        return 0.0;
    }
    let up = (top + band - y) / band;
    let down = (y - (bottom - band)) / band;
    let pace = |depth: f64| EDGE_SPEED * depth.min(1.0).powi(2);
    if up > 0.0 && up >= down {
        -pace(up)
    } else if down > 0.0 {
        pace(down)
    } else {
        0.0
    }
}

/// How far the page scrolls at `speed` in a frame `ms` long, in pixels.
pub fn scroll_step(speed: f64, ms: f64) -> f64 {
    if !(speed.is_finite() && ms.is_finite()) {
        return 0.0;
    }
    speed * ms.clamp(0.0, LONGEST_FRAME_MS) / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picked(s: &Selection) -> Vec<i64> {
        s.ids().iter().copied().collect()
    }

    /// Posts 1 to `n`, in the gallery's order.
    fn list(n: i64) -> Vec<i64> {
        (1..=n).collect()
    }

    #[test]
    fn a_mouse_or_pen_drags_at_once_and_a_finger_only_while_selecting() {
        assert_eq!(start("mouse", 0, false, false), Some(Start::OnMove));
        assert_eq!(start("pen", 0, false, true), Some(Start::OnMove));
        assert_eq!(start("", 0, false, false), Some(Start::OnMove));
        assert_eq!(start("touch", 0, false, true), Some(Start::OnHold));
        // Outside selection mode a finger scrolls, swipes and opens the menu.
        assert_eq!(start("touch", 0, false, false), None);
        // Another button, or a key that changes what a click does.
        assert_eq!(start("mouse", 2, false, true), None);
        assert_eq!(start("mouse", 1, false, false), None);
        assert_eq!(start("mouse", 0, true, true), None);
    }

    #[test]
    fn a_press_has_to_go_somewhere_to_drag() {
        assert!(!moved(3.0, 4.0, DRAG_SLOP));
        assert!(moved(6.0, 0.0, DRAG_SLOP));
        assert!(moved(-4.0, -5.0, DRAG_SLOP));
        assert!(!moved(f64::NAN, 40.0, DRAG_SLOP));
    }

    #[test]
    fn a_drag_picks_from_its_first_tile_to_the_one_under_the_pointer() {
        let loaded = list(10);
        let mut drag = Drag::new(Selection::default(), 2);
        assert!(drag.adding);
        // Just pressed: the tile it started on.
        assert_eq!(picked(&drag.selection(&loaded).unwrap()), [2]);
        assert!(drag.reach(7));
        assert_eq!(
            picked(&drag.selection(&loaded).unwrap()),
            [2, 3, 4, 5, 6, 7]
        );
        // Back again: the range shrinks.
        assert!(drag.reach(4));
        assert_eq!(picked(&drag.selection(&loaded).unwrap()), [2, 3, 4]);
        assert!(!drag.reach(4));
        // Past where it started, the other way.
        drag.reach(1);
        assert_eq!(picked(&drag.selection(&loaded).unwrap()), [1, 2]);
        assert_eq!(drag.from, 2);
    }

    #[test]
    fn a_drag_from_a_picked_tile_lets_go_and_leaves_the_rest_as_it_was() {
        let loaded = list(10);
        let mut all = Selection::default();
        all.toggle_all(&loaded);
        let mut drag = Drag::new(all.clone(), 3);
        assert!(!drag.adding);
        drag.reach(6);
        assert_eq!(
            picked(&drag.selection(&loaded).unwrap()),
            [1, 2, 7, 8, 9, 10]
        );
        drag.reach(4);
        assert_eq!(
            picked(&drag.selection(&loaded).unwrap()),
            [1, 2, 5, 6, 7, 8, 9, 10]
        );
        assert_eq!(drag.before(), &all);
    }

    #[test]
    fn posts_picked_before_stay_picked_as_the_range_moves_off_them() {
        let loaded = list(10);
        let mut before = Selection::default();
        before.toggle(&loaded, 4, false);
        before.toggle(&loaded, 9, false);
        let mut drag = Drag::new(before, 2);
        drag.reach(6);
        assert_eq!(
            picked(&drag.selection(&loaded).unwrap()),
            [2, 3, 4, 5, 6, 9]
        );
        drag.reach(3);
        assert_eq!(picked(&drag.selection(&loaded).unwrap()), [2, 3, 4, 9]);
    }

    #[test]
    fn shift_goes_on_from_where_a_drag_ended() {
        let loaded = list(10);
        let mut drag = Drag::new(Selection::default(), 2);
        drag.reach(4);
        let mut after = drag.selection(&loaded).unwrap();
        after.toggle(&loaded, 6, true);
        assert_eq!(picked(&after), [2, 3, 4, 5, 6]);
    }

    #[test]
    fn a_drag_picks_no_more_than_one_edit_takes_nearest_its_start() {
        let loaded: Vec<i64> = (0..600).collect();
        let mut drag = Drag::new(Selection::default(), 599);
        drag.reach(0);
        let s = drag.selection(&loaded).unwrap();
        assert_eq!(s.len(), starbooru_core::MAX_BULK_POSTS);
        assert!(s.contains(599) && s.contains(100) && !s.contains(99));
    }

    #[test]
    fn a_drag_whose_posts_left_the_list() {
        let mut drag = Drag::new(Selection::default(), 2);
        drag.reach(5);
        assert_eq!(drag.selection(&[1, 3, 4, 5]), None);
        // The end went: back to the start alone.
        assert_eq!(picked(&drag.selection(&[1, 2, 3]).unwrap()), [2]);
    }

    #[test]
    fn the_page_scrolls_near_its_edges_faster_closer_to_them() {
        // Tiles in sight from 64 to 764: a band of 84 px at each end.
        let (top, bottom) = (64.0, 764.0);
        assert_eq!(edge_speed(400.0, top, bottom), 0.0);
        assert_eq!(edge_speed(top + 84.0, top, bottom), 0.0);
        assert_eq!(edge_speed(bottom - 84.0, top, bottom), 0.0);
        let slow = edge_speed(bottom - 60.0, top, bottom);
        let fast = edge_speed(bottom - 10.0, top, bottom);
        assert!(0.0 < slow && slow < fast && fast < EDGE_SPEED);
        assert!(edge_speed(top + 10.0, top, bottom) < edge_speed(top + 60.0, top, bottom));
        assert!(edge_speed(top + 60.0, top, bottom) < 0.0);
        // At the edge and past it, over the bars or out of the window.
        assert_eq!(edge_speed(bottom, top, bottom), EDGE_SPEED);
        assert_eq!(edge_speed(2000.0, top, bottom), EDGE_SPEED);
        assert_eq!(edge_speed(top, top, bottom), -EDGE_SPEED);
        assert_eq!(edge_speed(-50.0, top, bottom), -EDGE_SPEED);
    }

    #[test]
    fn a_short_view_still_scrolls_and_a_senseless_one_doesnt() {
        // 50 px: the bands meet in the middle, each going its own way.
        assert!(edge_speed(20.0, 0.0, 50.0) < 0.0);
        assert!(edge_speed(30.0, 0.0, 50.0) > 0.0);
        assert_eq!(edge_speed(25.0, 0.0, 50.0), 0.0);
        assert_eq!(edge_speed(50.0, 100.0, 0.0), 0.0);
        assert_eq!(edge_speed(f64::NAN, 0.0, 800.0), 0.0);
        assert_eq!(edge_speed(10.0, 0.0, f64::INFINITY), 0.0);
    }

    #[test]
    fn a_frame_scrolls_its_share_of_a_second_and_a_stall_no_more() {
        assert_eq!(scroll_step(1000.0, 16.0), 16.0);
        assert_eq!(scroll_step(-1000.0, 10.0), -10.0);
        assert_eq!(scroll_step(1000.0, 500.0), 50.0);
        assert_eq!(scroll_step(1000.0, -5.0), 0.0);
        assert_eq!(scroll_step(f64::NAN, 16.0), 0.0);
    }
}
