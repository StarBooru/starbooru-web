//! Swiping a gallery tile aside with a finger. A tile with a `.swipe-hint`
//! for a side (`thumb.rs`) can be swiped that way: its picture follows the
//! finger and uncovers what the swipe does; let go far enough, or flicked,
//! the action runs and the picture slides back. A side without an action
//! gives a little and no more. Up and down stay the browser's scroll, and
//! the click that follows a swipe is dropped, as it would open the post.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, MouseEvent, PointerEvent};

use crate::dom::{self, Watch};
use crate::logic::swipe::{Axis, Side, axis, follow, reach, release, speed};

/// The tiles that can be swiped, as `pages/post_menu.rs` finds them.
const TILES: &str = r#"[data-press="post"] [data-post]"#;
/// Positions kept for the speed at the end.
const TRAIL: usize = 8;
/// How long the picture takes to slide back: `--med` in `style/tokens.css`,
/// with some to spare.
const BACK_MS: u64 = 320;

thread_local! {
    /// On while a finger drags across tiles to pick them
    /// (`pages/drag_select.rs`): it isn't swiping any of them.
    static PICKING: Cell<bool> = const { Cell::new(false) };
}

/// Says that a finger is dragging across tiles to pick them, or has stopped.
pub fn picking(on: bool) {
    PICKING.set(on);
}

/// One finger on a tile.
struct Press {
    pointer: i32,
    post: i64,
    tile: HtmlElement,
    media: Option<HtmlElement>,
    left: bool,
    right: bool,
    x0: f64,
    y0: f64,
    dx: f64,
    sideways: bool,
    trail: Vec<(f64, f64)>,
}

impl Press {
    fn can(&self, side: Side) -> bool {
        match side {
            Side::Left => self.left,
            Side::Right => self.right,
        }
    }

    /// The picture `dx` from its place.
    fn drag(&self, dx: f64) {
        let side = Side::of(dx);
        let can = self.can(side);
        let armed = can && dx.abs() >= reach(f64::from(self.tile.client_width()));
        let _ = self.tile.set_attribute("data-swiping", side.as_str());
        let _ = self.tile.toggle_attribute_with_force("data-armed", armed);
        if let Some(media) = &self.media {
            let _ = media
                .style()
                .set_property("translate", &format!("{:.1}px 0", follow(dx, can)));
        }
    }

    /// Back to its place.
    fn settle(&self) {
        let tile = self.tile.clone();
        let _ = tile.set_attribute("data-swiping", "back");
        let _ = tile.remove_attribute("data-armed");
        if let Some(media) = &self.media {
            let _ = media.style().remove_property("translate");
        }
        set_timeout(
            move || {
                if tile.get_attribute("data-swiping").as_deref() == Some("back") {
                    let _ = tile.remove_attribute("data-swiping");
                }
            },
            Duration::from_millis(BACK_MS),
        );
    }
}

/// The tile under a pointer that came down, if it can be swiped.
fn press_at(e: &PointerEvent) -> Option<Press> {
    let tile = e
        .target()?
        .dyn_into::<Element>()
        .ok()?
        .closest(TILES)
        .ok()??
        .dyn_into::<HtmlElement>()
        .ok()?;
    let post = tile.get_attribute("data-post")?.parse().ok()?;
    let has = |side: Side| {
        let selector = format!(r#":scope > .swipe-hint[data-side="{}"]"#, side.as_str());
        tile.query_selector(&selector).ok().flatten().is_some()
    };
    let (left, right) = (has(Side::Left), has(Side::Right));
    if !left && !right {
        return None;
    }
    let media = tile
        .query_selector(":scope > .thumb-media")
        .ok()
        .flatten()
        .and_then(|m| m.dyn_into::<HtmlElement>().ok());
    let (x, y) = (f64::from(e.client_x()), f64::from(e.client_y()));
    Some(Press {
        pointer: e.pointer_id(),
        post,
        tile,
        media,
        left,
        right,
        x0: x,
        y0: y,
        dx: 0.0,
        sideways: false,
        trail: vec![(e.time_stamp(), x)],
    })
}

/// Listens for swipes on tiles until the watches stop. `on_swipe` gets the
/// post and the side of each that went far enough.
pub fn watch(on_swipe: impl Fn(i64, Side) + 'static) -> Vec<Watch> {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return Vec::new();
    };
    let press = Rc::new(RefCell::new(None::<Press>));
    // Until when a click is the end of a swipe, in event time.
    let swallow = Rc::new(Cell::new(f64::NEG_INFINITY));
    let pointer = |e: &web_sys::Event| e.dyn_ref::<PointerEvent>().cloned();

    let down = {
        let (press, swallow) = (Rc::clone(&press), Rc::clone(&swallow));
        move |e: web_sys::Event| {
            let Some(e) = pointer(&e) else { return };
            swallow.set(f64::NEG_INFINITY);
            // A second finger: a pinch or a scroll, not a swipe.
            if let Some(p) = press.borrow_mut().take() {
                if p.sideways {
                    p.settle();
                }
                if !e.is_primary() {
                    return;
                }
            }
            if e.pointer_type() == "mouse" || !e.is_primary() {
                return;
            }
            *press.borrow_mut() = press_at(&e);
        }
    };
    let moved = {
        let press = Rc::clone(&press);
        move |e: web_sys::Event| {
            let Some(e) = pointer(&e) else { return };
            let mut slot = press.borrow_mut();
            let Some(p) = slot.as_mut().filter(|p| p.pointer == e.pointer_id()) else {
                return;
            };
            if PICKING.get() {
                if let Some(p) = slot.take().filter(|p| p.sideways) {
                    p.settle();
                }
                return;
            }
            let x = f64::from(e.client_x());
            let (dx, dy) = (x - p.x0, f64::from(e.client_y()) - p.y0);
            p.trail.push((e.time_stamp(), x));
            if p.trail.len() > TRAIL {
                p.trail.remove(0);
            }
            if !p.sideways {
                match axis(dx, dy) {
                    Axis::Unknown => return,
                    Axis::Scroll => {
                        *slot = None;
                        return;
                    }
                    Axis::Sideways => p.sideways = true,
                }
            }
            p.dx = dx;
            p.drag(dx);
        }
    };
    let up = {
        let (press, swallow) = (Rc::clone(&press), Rc::clone(&swallow));
        move |e: web_sys::Event| {
            let Some(e) = pointer(&e) else { return };
            let taken = press.borrow_mut().take_if(|p| p.pointer == e.pointer_id());
            let Some(p) = taken.filter(|p| p.sideways) else {
                return;
            };
            swallow.set(e.time_stamp() + 500.0);
            let width = f64::from(p.tile.client_width());
            let side = release(p.dx, width, speed(&p.trail), |s| p.can(s));
            p.settle();
            if let Some(side) = side {
                let _ = web_sys::window().map(|w| w.navigator().vibrate_with_duration(8));
                on_swipe(p.post, side);
            }
        }
    };
    let lost = {
        let press = Rc::clone(&press);
        move |e: web_sys::Event| {
            let Some(e) = pointer(&e) else { return };
            let taken = press.borrow_mut().take_if(|p| p.pointer == e.pointer_id());
            if let Some(p) = taken.filter(|p| p.sideways) {
                p.settle();
            }
        }
    };
    let click = move |e: web_sys::Event| {
        let Some(e) = e.dyn_ref::<MouseEvent>() else {
            return;
        };
        if e.is_trusted() && e.time_stamp() < swallow.get() {
            e.prevent_default();
            e.stop_propagation();
            swallow.set(f64::NEG_INFINITY);
        }
    };
    vec![
        dom::listen(&doc, "pointerdown", true, down),
        dom::listen(&doc, "pointermove", true, moved),
        dom::listen(&doc, "pointerup", true, up),
        dom::listen(&doc, "pointercancel", true, lost),
        dom::listen_capture(&doc, "click", click),
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_swipeable_tile_leaves_up_and_down_to_the_browser() {
        let css = include_str!("../../style/pages/gallery.css");
        assert!(css.contains(".thumb:has(> .swipe-hint) { touch-action: pan-y; }"));
        assert!(css.contains(
            r#".swipe-hint[data-action="delete"] { background: var(--danger-container);"#
        ));
        // The hints sit under the picture, which uncovers them.
        let thumb = include_str!("thumb.rs");
        let hint = thumb.find(r#"class="swipe-hint""#);
        let media = thumb.find(r#"class="thumb-media""#);
        assert!(hint.zip(media).is_some_and(|(h, m)| h < m));
        assert!(include_str!("../pages/gallery.rs").contains("swipe=swipe"));
    }
}
