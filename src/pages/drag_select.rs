//! Picking several of the gallery's tiles at once by dragging across them,
//! as in Google Photos. A mouse or a pen drags from the tile it pressed on
//! as soon as it moves. A finger only drags in selection mode, once it has
//! rested on a tile, since moving straight away scrolls the page or swipes
//! the tile; there, its long press picks instead of opening the post's menu.
//! The range runs in list order from the first tile to the one under the
//! pointer, found from the layout, so tiles scrolled off the page count too.
//! Near the top and the bottom the page scrolls by itself. Escape puts back
//! the selection from before the drag.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use leptos::html;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use starbooru_core::Post;
use wasm_bindgen::JsCast;
use web_sys::{Element, KeyboardEvent, PointerEvent};

use super::gallery::Grid;
use crate::components::swipe;
use crate::dom::{self, Watch};
use crate::logic::bulk::Selection;
use crate::logic::drag_select::{
    DRAG_SLOP, Drag, HOLD_MS, Start, edge_speed, moved, scroll_step, start,
};
use crate::logic::layout::Layout;
use crate::logic::swipe::SLOP;

/// A tile, as `components/thumb.rs` marks it.
const TILE: &str = "[data-post]";
/// On `<html>` while a drag lasts (`style/pages/gallery.css`).
const DRAGGING: &str = "sb-drag-select";
/// How long after a drag the click its release makes is dropped, in
/// milliseconds.
const CLICK_MS: f64 = 250.0;
/// How long after a finger's drag a right click still belongs to it: Android
/// sends one more as the finger lifts.
const MENU_MS: f64 = 800.0;

/// The pointer on a tile.
enum Press {
    Idle,
    /// Down on post `post` at `x`, `y`, not dragging yet. `hold` tells a
    /// finger's timer whether this is still its press.
    Waiting {
        pointer: i32,
        post: i64,
        x: f64,
        y: f64,
        touch: bool,
        hold: u32,
    },
    /// Dragging, with the pointer last at `x`, `y`.
    Dragging {
        pointer: i32,
        drag: Drag,
        x: f64,
        y: f64,
        touch: bool,
        was_selecting: bool,
    },
    /// Escape called the drag off: the rest of the press does nothing.
    Spent {
        pointer: i32,
        touch: bool,
    },
}

impl Press {
    fn pointer(&self) -> Option<i32> {
        match self {
            Press::Idle => None,
            Press::Waiting { pointer, .. }
            | Press::Dragging { pointer, .. }
            | Press::Spent { pointer, .. } => Some(*pointer),
        }
    }

    /// Whether a finger is dragging, or was until Escape.
    fn finger_down(&self) -> bool {
        matches!(
            self,
            Press::Dragging { touch: true, .. } | Press::Spent { touch: true, .. }
        )
    }
}

/// What a pointer moving means for the press.
enum Next {
    Nothing,
    Forget,
    Begin,
    Follow,
    Finish,
}

/// What a drag reads and changes in the gallery.
#[derive(Clone, Copy)]
struct Gallery {
    masonry: NodeRef<html::Div>,
    posts: RwSignal<Vec<Arc<Post>>>,
    tiles: Memo<Layout>,
    grid: Memo<Grid>,
    selecting: RwSignal<bool>,
    selection: RwSignal<Selection>,
    pinned: RwSignal<Option<usize>>,
}

impl Gallery {
    fn element(&self) -> Option<web_sys::HtmlDivElement> {
        self.masonry.try_get_untracked().flatten()
    }

    /// An event's target, if it's in the grid.
    fn inside(&self, target: Option<web_sys::EventTarget>) -> Option<Element> {
        let el = target?.dyn_into::<Element>().ok()?;
        self.element().filter(|grid| grid.contains(Some(&*el)))?;
        Some(el)
    }

    /// The post of the tile an event is on, if it's one of the grid's.
    fn post_of(&self, target: Option<web_sys::EventTarget>) -> Option<i64> {
        let tile = self.inside(target)?.closest(TILE).ok()??;
        tile.get_attribute("data-post")?.parse().ok()
    }

    /// The post under a point of the window, from the layout rather than the
    /// tiles on the page.
    fn post_at(&self, x: f64, y: f64) -> Option<i64> {
        let rect = self.element()?.get_bounding_client_rect();
        let Grid { tile, gap, .. } = self.grid.try_get_untracked()?;
        let index = self.tiles.try_with_untracked(|t| {
            t.at(tile, gap, rect.width(), x - rect.left(), y - rect.top())
        })??;
        self.posts
            .try_with_untracked(|p| p.get(index).map(|p| p.id))?
    }

    fn index_of(&self, id: i64) -> Option<usize> {
        self.posts
            .try_with_untracked(|p| p.iter().position(|p| p.id == id))?
    }

    /// Shows what `drag` picks.
    fn show(&self, drag: &Drag) {
        let loaded = self
            .posts
            .try_with_untracked(|p| p.iter().map(|p| p.id).collect::<Vec<_>>());
        let Some(now) = loaded.and_then(|l| drag.selection(&l)) else {
            return;
        };
        if self.selection.try_with_untracked(|s| *s != now) == Some(true) {
            self.selection.try_set(now);
        }
    }
}

/// The part of the window tiles show in, as its top and bottom: under the
/// top bar, and above the navigation at the bottom of a phone and the
/// selection's bar.
fn in_sight() -> (f64, f64) {
    let bar = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.query_selector(".bulk-bar").ok().flatten())
        .map(|bar| bar.get_bounding_client_rect().top());
    let dock = dom::dock_top();
    (dom::bar_bottom(), bar.map_or(dock, |bar| bar.min(dock)))
}

struct Sweeper {
    gallery: Gallery,
    press: RefCell<Press>,
    /// Until when a click is the end of a drag, in event time.
    swallow: Cell<f64>,
    /// Until when a right click belongs to a finger's drag, in event time.
    quiet: Cell<f64>,
    /// Counts presses, for a finger's timer to know its own.
    holds: Cell<u32>,
    /// Whether frames are scrolling the page, when the last one was, and the
    /// part of a pixel it left to scroll.
    scrolling: Cell<bool>,
    last_frame: Cell<f64>,
    carry: Cell<f64>,
    alive: Cell<bool>,
}

impl Sweeper {
    fn down(self: &Rc<Self>, e: &PointerEvent) {
        let pointer = e.pointer_id();
        let busy = match &*self.press.borrow() {
            Press::Idle => None,
            // A second finger: a pinch, not a long press.
            Press::Waiting { .. } => Some(false),
            // The same pointer down again missed its way up, out of the window.
            Press::Dragging { .. } | Press::Spent { .. } => Some(true),
        };
        match busy {
            Some(true) if self.press.borrow().pointer() != Some(pointer) => return,
            Some(true) => self.reset(),
            Some(false) => *self.press.borrow_mut() = Press::Idle,
            None => {}
        }
        if !e.is_primary() {
            return;
        }
        let Some(post) = self.gallery.post_of(e.target()) else {
            return;
        };
        let modifiers = e.shift_key() || e.ctrl_key() || e.meta_key() || e.alt_key();
        let selecting = self.gallery.selecting.try_get_untracked() == Some(true);
        let Some(how) = start(&e.pointer_type(), e.button(), modifiers, selecting) else {
            return;
        };
        let hold = self.holds.get().wrapping_add(1);
        self.holds.set(hold);
        *self.press.borrow_mut() = Press::Waiting {
            pointer,
            post,
            x: f64::from(e.client_x()),
            y: f64::from(e.client_y()),
            touch: how == Start::OnHold,
            hold,
        };
        if how == Start::OnHold {
            let me = Rc::clone(self);
            set_timeout(move || me.held(hold), Duration::from_millis(HOLD_MS));
        }
    }

    /// A finger's timer: it has rested on its tile long enough.
    fn held(self: &Rc<Self>, hold: u32) {
        let mine = matches!(
            *self.press.borrow(),
            Press::Waiting { hold: h, touch: true, .. } if h == hold
        );
        if mine && self.alive.get() {
            self.begin(None);
        }
    }

    /// Turns the press into a drag, with the pointer moved to `to`, or still
    /// where it came down.
    fn begin(self: &Rc<Self>, to: Option<(f64, f64)>) {
        let Press::Waiting {
            pointer,
            post,
            x,
            y,
            touch,
            ..
        } = *self.press.borrow()
        else {
            return;
        };
        let (x, y) = to.unwrap_or((x, y));
        let g = self.gallery;
        let Some(before) = g.selection.try_get_untracked() else {
            return;
        };
        let was_selecting = g.selecting.try_get_untracked() == Some(true);
        let drag = Drag::new(before, post);
        if !was_selecting {
            g.selecting.try_set(true);
        }
        // A finger's events go to the tile it came down on, and only reach
        // the page while that tile is on it.
        g.pinned.try_set(g.index_of(post));
        dom::root_class(DRAGGING, true);
        if let Some(el) = g.element() {
            let _ = el.set_pointer_capture(pointer);
        }
        if touch {
            swipe::picking(true);
            let _ = web_sys::window().map(|w| w.navigator().vibrate_with_duration(8));
        }
        g.show(&drag);
        *self.press.borrow_mut() = Press::Dragging {
            pointer,
            drag,
            x,
            y,
            touch,
            was_selecting,
        };
        self.reach();
        // A finger held still at an edge doesn't set the page off yet.
        if to.is_some() {
            self.edge();
        }
    }

    fn moved(self: &Rc<Self>, e: &PointerEvent) {
        let (x, y) = (f64::from(e.client_x()), f64::from(e.client_y()));
        // A mouse let go out of the window, where the page heard nothing.
        let let_go = e.buttons() == 0 && e.pointer_type() != "touch";
        let next = {
            let mut press = self.press.borrow_mut();
            if press.pointer() != Some(e.pointer_id()) {
                return;
            }
            match &mut *press {
                // Before the long press: a scroll or a swipe.
                Press::Waiting {
                    x: x0,
                    y: y0,
                    touch: true,
                    ..
                } => {
                    if moved(x - *x0, y - *y0, SLOP) {
                        Next::Forget
                    } else {
                        Next::Nothing
                    }
                }
                Press::Waiting { .. } if let_go => Next::Forget,
                Press::Waiting { x: x0, y: y0, .. } => {
                    if moved(x - *x0, y - *y0, DRAG_SLOP) {
                        Next::Begin
                    } else {
                        Next::Nothing
                    }
                }
                Press::Dragging { .. } | Press::Spent { .. } if let_go => Next::Finish,
                Press::Dragging { x: px, y: py, .. } => {
                    *px = x;
                    *py = y;
                    Next::Follow
                }
                Press::Idle | Press::Spent { .. } => Next::Nothing,
            }
        };
        match next {
            Next::Nothing => {}
            Next::Forget => *self.press.borrow_mut() = Press::Idle,
            Next::Begin => self.begin(Some((x, y))),
            Next::Follow => self.follow(),
            Next::Finish => self.finish(e.time_stamp()),
        }
    }

    /// Takes the range to the tile under the pointer, and scrolls the page
    /// when the pointer is near an edge.
    fn follow(self: &Rc<Self>) {
        self.reach();
        self.edge();
    }

    fn reach(&self) {
        let Press::Dragging { x, y, .. } = *self.press.borrow() else {
            return;
        };
        // In a gap between tiles the range stays as it is.
        let Some(id) = self.gallery.post_at(x, y) else {
            return;
        };
        let reached = match &mut *self.press.borrow_mut() {
            Press::Dragging { drag, .. } => drag.reach(id).then(|| drag.clone()),
            _ => None,
        };
        if let Some(drag) = reached {
            self.gallery.show(&drag);
        }
    }

    fn edge(self: &Rc<Self>) {
        if self.scrolling.get() || self.speed() == 0.0 {
            return;
        }
        self.scrolling.set(true);
        self.last_frame.set(js_sys::Date::now());
        self.carry.set(0.0);
        let me = Rc::clone(self);
        request_animation_frame(move || me.frame());
    }

    /// How fast the page scrolls under the pointer now.
    fn speed(&self) -> f64 {
        let Press::Dragging { y, .. } = *self.press.borrow() else {
            return 0.0;
        };
        let (top, bottom) = in_sight();
        edge_speed(y, top, bottom)
    }

    /// One frame of scrolling at an edge: whole pixels, with the rest kept
    /// for the next frame, and no gliding, as it follows the pointer.
    fn frame(self: &Rc<Self>) {
        let speed = if self.alive.get() { self.speed() } else { 0.0 };
        if speed == 0.0 {
            self.scrolling.set(false);
            return;
        }
        let now = js_sys::Date::now();
        let step = scroll_step(speed, now - self.last_frame.get()) + self.carry.get();
        self.last_frame.set(now);
        let whole = step.trunc();
        self.carry.set(step - whole);
        if whole != 0.0 {
            dom::scroll_by(whole);
        }
        self.reach();
        let me = Rc::clone(self);
        request_animation_frame(move || me.frame());
    }

    fn up(&self, e: &PointerEvent) {
        if self.press.borrow().pointer() == Some(e.pointer_id()) {
            self.finish(e.time_stamp());
        }
    }

    /// The press is over, at event time `at`. What it picked stays.
    fn finish(&self, at: f64) {
        let was = std::mem::replace(&mut *self.press.borrow_mut(), Press::Idle);
        let touch = match was {
            Press::Idle | Press::Waiting { .. } => return,
            Press::Dragging { touch, .. } | Press::Spent { touch, .. } => touch,
        };
        // The release clicks the tile under it, which would open or pick it.
        self.swallow.set(at + CLICK_MS);
        if touch {
            self.quiet.set(at + MENU_MS);
        }
        self.reset();
    }

    /// Back to no press at all.
    fn reset(&self) {
        *self.press.borrow_mut() = Press::Idle;
        swipe::picking(false);
        dom::root_class(DRAGGING, false);
        self.gallery.pinned.try_set(None);
    }

    /// Escape: the selection goes back to what it was before the drag, and so
    /// does selection mode.
    fn escape(&self) -> bool {
        let was = std::mem::replace(&mut *self.press.borrow_mut(), Press::Idle);
        match was {
            Press::Dragging {
                pointer,
                drag,
                touch,
                was_selecting,
                ..
            } => {
                let g = self.gallery;
                g.selection.try_set(drag.before().clone());
                if !was_selecting {
                    g.selecting.try_set(false);
                }
                dom::root_class(DRAGGING, false);
                g.pinned.try_set(None);
                *self.press.borrow_mut() = Press::Spent { pointer, touch };
                true
            }
            other => {
                *self.press.borrow_mut() = other;
                false
            }
        }
    }

    /// A right click, or a phone's long press: in selection mode a finger's
    /// long press picks, and opens no menu.
    fn menu(self: &Rc<Self>, e: &web_sys::Event) {
        let waiting = matches!(*self.press.borrow(), Press::Waiting { touch: true, .. });
        if waiting {
            self.begin(None);
        }
        if waiting || self.press.borrow().finger_down() || e.time_stamp() < self.quiet.get() {
            e.prevent_default();
            e.stop_propagation();
        }
    }
}

/// Lets the gallery's tiles be picked by dragging across them while it's on
/// the page. `pinned` is the tile the gallery keeps on the page during a
/// drag.
#[component]
pub(super) fn DragSelect(
    masonry: NodeRef<html::Div>,
    posts: RwSignal<Vec<Arc<Post>>>,
    tiles: Memo<Layout>,
    grid: Memo<Grid>,
    selecting: RwSignal<bool>,
    selection: RwSignal<Selection>,
    pinned: RwSignal<Option<usize>>,
) -> impl IntoView {
    let me = Rc::new(Sweeper {
        gallery: Gallery {
            masonry,
            posts,
            tiles,
            grid,
            selecting,
            selection,
            pinned,
        },
        press: RefCell::new(Press::Idle),
        swallow: Cell::new(f64::NEG_INFINITY),
        quiet: Cell::new(f64::NEG_INFINITY),
        holds: Cell::new(0),
        scrolling: Cell::new(false),
        last_frame: Cell::new(0.0),
        carry: Cell::new(0.0),
        alive: Cell::new(true),
    });
    let on = |me: &Rc<Sweeper>, f: fn(&Rc<Sweeper>, &PointerEvent)| {
        let me = Rc::clone(me);
        move |e: web_sys::Event| {
            if let Some(e) = e.dyn_ref::<PointerEvent>() {
                f(&me, e);
            }
        }
    };
    let mut watches = Vec::new();
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        watches.push(dom::listen(
            &doc,
            "pointerdown",
            true,
            on(&me, Sweeper::down),
        ));
        watches.push(dom::listen(
            &doc,
            "pointermove",
            true,
            on(&me, Sweeper::moved),
        ));
        watches.push(dom::listen(
            &doc,
            "pointerup",
            true,
            on(&me, |me, e| me.up(e)),
        ));
        watches.push(dom::listen(
            &doc,
            "pointercancel",
            true,
            on(&me, |me, e| me.up(e)),
        ));
        watches.push(dom::listen_capture(&doc, "contextmenu", {
            let me = Rc::clone(&me);
            move |e| me.menu(&e)
        }));
        // Before the selection's bar hears it and ends selection mode.
        watches.push(dom::listen_capture(&doc, "keydown", {
            let me = Rc::clone(&me);
            move |e| {
                let escape = e
                    .dyn_ref::<KeyboardEvent>()
                    .is_some_and(|k| k.key() == "Escape");
                if escape && me.escape() {
                    e.prevent_default();
                    e.stop_propagation();
                }
            }
        }));
        watches.push(dom::listen_capture(&doc, "click", {
            let me = Rc::clone(&me);
            move |e| {
                if e.is_trusted() && e.time_stamp() < me.swallow.get() {
                    e.prevent_default();
                    e.stop_propagation();
                    me.swallow.set(f64::NEG_INFINITY);
                }
            }
        }));
        // The browser's own drag of a link or a picture would take the pointer.
        watches.push(dom::listen_capture(&doc, "dragstart", {
            let me = Rc::clone(&me);
            move |e| {
                if me.gallery.inside(e.target()).is_some() {
                    e.prevent_default();
                }
            }
        }));
    }
    // The wheel moves the tiles under a pointer that stays put.
    watches.push(dom::on_window("scroll", {
        let me = Rc::clone(&me);
        move || me.reach()
    }));
    // A finger's drag keeps the page from scrolling under it. Only in
    // selection mode, where one can start: a listener that can stop a scroll
    // makes the browser wait for it on every touch.
    let still = StoredValue::new(None::<Watch>);
    Effect::new({
        let me = Rc::clone(&me);
        move |_| {
            let watch = masonry.get().filter(|_| selecting.get()).map(|el| {
                let me = Rc::clone(&me);
                dom::listen(&el, "touchmove", false, move |e| {
                    if me.press.borrow().finger_down() && e.cancelable() {
                        e.prevent_default();
                    }
                })
            });
            still.set_value(watch);
        }
    });
    let me = SendWrapper::new(me);
    let watches = SendWrapper::new(watches);
    on_cleanup(move || {
        me.alive.set(false);
        me.reset();
        for watch in watches.take() {
            watch.stop();
        }
        if let Some(Some(watch)) = still.try_update_value(Option::take) {
            watch.stop();
        }
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_browser_keeps_its_own_drags_out_of_the_grid() {
        let css = include_str!("../../style/pages/gallery.css");
        assert!(css.contains(".gallery .masonry { user-select: none;"));
        assert!(css.contains(":is(.thumb, img) { -webkit-user-drag: none; }"));
        assert!(css.contains(".masonry-box.selecting .thumb { -webkit-touch-callout: none; }"));
        assert!(css.contains(&format!("html.{} {{ user-select: none;", super::DRAGGING)));
    }
}
