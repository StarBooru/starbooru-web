//! The menu a right-clicked tag or tile opens (`tag_menu.rs`, the pages'
//! `post_menu.rs`): what can be done with it, growing out of the point it was
//! opened at, over a backdrop that closes it.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use leptos::prelude::*;
use send_wrapper::SendWrapper;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, HtmlElement, KeyboardEvent, MouseEvent};

use super::icons::{Ico, Icon};
use crate::dom;

/// Where a menu was asked for, and the size of the window then, in CSS
/// pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuAt {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// A menu's width, unless the window is narrower.
pub const MENU_W: f64 = 248.0;
/// Space kept between a menu and the window's edges.
const EDGE: f64 = 8.0;

impl MenuAt {
    /// The menu's place: its corner at the point, on the side of it with more
    /// room, and never past the window's edges. Its animation grows from the
    /// point.
    pub fn style(self) -> String {
        let sane = |v: f64| if v.is_finite() { v.max(0.0) } else { 0.0 };
        let (vw, vh) = (sane(self.width), sane(self.height));
        let (x, y) = (sane(self.x).min(vw), sane(self.y).min(vh));
        let w = MENU_W.min(vw - 2.0 * EDGE).max(0.0);
        let right_room = vw - x - EDGE;
        let left = if right_room >= w { x } else { x - w };
        let left = left.min(vw - w - EDGE).max(EDGE.min(vw - w).max(0.0));
        let origin_x = x - left;
        if y > vh / 2.0 {
            let bottom = vh - y;
            let room = (y - EDGE).max(0.0);
            format!(
                "left: {left:.0}px; bottom: {bottom:.0}px; width: {w:.0}px; max-height: {room:.0}px; transform-origin: {origin_x:.0}px 100%"
            )
        } else {
            let room = (vh - y - EDGE).max(0.0);
            format!(
                "left: {left:.0}px; top: {y:.0}px; width: {w:.0}px; max-height: {room:.0}px; transform-origin: {origin_x:.0}px 0"
            )
        }
    }

    /// For a control from `top` to `bottom`, starting at `left`: above it in
    /// the lower half of the window, below it in the upper half.
    pub fn beside(left: f64, top: f64, bottom: f64, width: f64, height: f64) -> Self {
        let y = if top > height / 2.0 { top } else { bottom };
        MenuAt {
            x: left,
            y,
            width,
            height,
        }
    }
}

/// Where the menu goes for a button that opens one: beside it.
pub fn beside(el: &Element) -> MenuAt {
    let r = el.get_bounding_client_rect();
    let (width, height) = dom::window_size();
    MenuAt::beside(r.left(), r.top(), r.bottom(), width, height)
}

/// The element matching `selector` a right click was on, and where the menu
/// goes for it.
pub fn pressed(e: &MouseEvent, selector: &str) -> Option<(Element, MenuAt)> {
    let el = e
        .target()?
        .dyn_into::<Element>()
        .ok()?
        .closest(selector)
        .ok()??;
    let window = web_sys::window()?;
    let size = |v: Result<JsValue, JsValue>| v.ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
    let (mut x, mut y) = (f64::from(e.client_x()), f64::from(e.client_y()));
    // The menu key has no pointer: under the element, then.
    if x == 0.0 && y == 0.0 {
        let r = el.get_bounding_client_rect();
        (x, y) = (r.left(), r.bottom());
    }
    let at = MenuAt {
        x,
        y,
        width: size(window.inner_width()),
        height: size(window.inner_height()),
    };
    Some((el, at))
}

/// How long after opening a right click on the backdrop still belongs to the
/// press that opened the menu: Android sends one more as the finger lifts.
const SETTLE_MS: f64 = 800.0;

/// Moves the focus along the menu's items by `by`, round from either end.
/// From outside the menu, down goes to the first item and up to the last.
fn step_focus(by: i32) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Ok(found) = doc.query_selector_all(".press-menu .menu-item") else {
        return;
    };
    let items: Vec<HtmlElement> = (0..found.length())
        .filter_map(|i| found.item(i)?.dyn_into::<HtmlElement>().ok())
        .collect();
    let n = items.len();
    if n == 0 {
        return;
    }
    let active: Option<Element> = doc.active_element();
    let at = items.iter().position(|el| {
        active
            .as_ref()
            .is_some_and(|a| a == AsRef::<Element>::as_ref(el))
    });
    let next = match (by, at) {
        (0, _) | (1.., None) => 0,
        (_, None) => n - 1,
        (1.., Some(i)) => (i + 1) % n,
        (_, Some(i)) => (i + n - 1) % n,
    };
    if let Some(el) = items.get(next) {
        let _ = el.focus();
    }
}

/// The element with the focus, unless it's nowhere in particular: the page
/// itself, or something no longer on it.
fn focused() -> Option<HtmlElement> {
    let doc = web_sys::window()?.document()?;
    let el = doc.active_element()?.dyn_into::<HtmlElement>().ok()?;
    let body = doc
        .body()
        .is_some_and(|b| AsRef::<Element>::as_ref(&b) == AsRef::<Element>::as_ref(&el));
    (!body && el.is_connected()).then_some(el)
}

/// A menu at `at` over a backdrop that closes it. Escape and Tab close it
/// too, and the arrows go through its items, wherever the focus is. As it
/// closes, the focus goes back where it was.
#[component]
pub fn ActionMenu(
    at: MenuAt,
    title: String,
    on_close: Callback<()>,
    children: Children,
) -> impl IntoView {
    let opened = js_sys::Date::now();
    // Where the focus was, to go back to as the menu goes.
    let opener = SendWrapper::new(focused());
    // Off as soon as the menu goes; the listener itself goes a moment later.
    let open = Arc::new(AtomicBool::new(true));
    let listening = Arc::clone(&open);
    let keys = web_sys::window().and_then(|w| w.document()).map(|doc| {
        // On the document, so it comes before the page's own keys on the
        // window: Escape closes the menu and not the post behind it.
        dom::listen(&doc, "keydown", false, move |e| {
            if !listening.load(Ordering::Relaxed) {
                return;
            }
            let Some(key) = e.dyn_ref::<KeyboardEvent>().map(KeyboardEvent::key) else {
                return;
            };
            match key.as_str() {
                "Escape" | "Tab" => on_close.run(()),
                "ArrowDown" => step_focus(1),
                "ArrowUp" => step_focus(-1),
                _ => return,
            }
            e.prevent_default();
            e.stop_propagation();
        })
    });
    on_cleanup(move || {
        open.store(false, Ordering::Relaxed);
        // Escape closes the menu from inside this very listener, which can't
        // be dropped while it runs. By then the menu has left the page.
        set_timeout(
            move || {
                if let Some(keys) = keys {
                    keys.stop();
                }
                // Unless what the menu did put the focus somewhere of its own.
                if let Some(el) = opener.take()
                    && el.is_connected()
                    && focused().is_none()
                {
                    let _ = el.focus();
                }
            },
            Duration::ZERO,
        );
    });
    let menu: NodeRef<leptos::html::Div> = NodeRef::new();
    menu.on_load(|_| request_animation_frame(|| step_focus(0)));
    let label = title.clone();
    view! {
        <div
            class="press-backdrop"
            on:click=move |_| on_close.run(())
            on:wheel=move |_| on_close.run(())
            on:contextmenu=move |e| {
                e.prevent_default();
                if js_sys::Date::now() - opened > SETTLE_MS {
                    on_close.run(());
                }
            }
        ></div>
        <div class="press-menu" role="menu" aria-label=label style=at.style() node_ref=menu>
            <div class="press-menu-title" aria-hidden="true">
                {title}
            </div>
            {children()}
        </div>
    }
}

/// One of the menu's items. `danger` is for the one that removes something;
/// `hint`, a line under the label, says what it will do when that isn't
/// plain from the label.
#[component]
pub fn MenuItem(
    icon: Icon,
    #[prop(into)] label: Signal<String>,
    #[prop(optional)] danger: bool,
    #[prop(optional, into)] hint: Option<Signal<String>>,
    on_click: Callback<()>,
) -> impl IntoView {
    let text = match hint {
        None => view! { <span>{move || label.get()}</span> }.into_any(),
        Some(hint) => view! {
            <span class="menu-text">
                <span>{move || label.get()}</span>
                <small class="menu-hint">{move || hint.get()}</small>
            </span>
        }
        .into_any(),
    };
    view! {
        <button
            type="button"
            role="menuitem"
            class="menu-item"
            class:danger=danger
            on:click=move |_| on_click.run(())
        >
            <Ico icon=icon size=20 />
            {text}
        </button>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn px(style: &str, name: &str) -> f64 {
        style
            .split("; ")
            .find_map(|part| part.strip_prefix(&format!("{name}: ")))
            .and_then(|v| v.strip_suffix("px"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(f64::NAN)
    }

    fn at(x: f64, y: f64) -> MenuAt {
        MenuAt {
            x,
            y,
            width: 400.0,
            height: 800.0,
        }
    }

    #[test]
    fn a_menu_opens_beside_the_point_on_the_side_with_room() {
        // Top left: down and to the right, from the point.
        let s = at(20.0, 100.0).style();
        assert_eq!((px(&s, "left"), px(&s, "top")), (20.0, 100.0));
        assert!(s.contains("transform-origin: 0px 0"));
        // Near the right edge: to the left of the point.
        let s = at(380.0, 100.0).style();
        assert_eq!(px(&s, "left"), 380.0 - MENU_W);
        assert!(s.contains(&format!("transform-origin: {MENU_W:.0}px 0")));
        // Low down: upwards, its bottom at the point.
        let s = at(20.0, 700.0).style();
        assert_eq!(px(&s, "bottom"), 100.0);
        assert_eq!(px(&s, "max-height"), 692.0);
        assert!(s.contains("100%"));
    }

    #[test]
    fn a_button_s_menu_opens_away_from_the_nearer_edge() {
        // A bar low on a phone: the menu's bottom at the button's top.
        let low = MenuAt::beside(300.0, 600.0, 640.0, 390.0, 844.0);
        assert_eq!(low.y, 600.0);
        assert_eq!(px(&low.style(), "bottom"), 244.0);
        // High up: under the button.
        let high = MenuAt::beside(300.0, 100.0, 140.0, 390.0, 844.0);
        assert_eq!(px(&high.style(), "top"), 140.0);
    }

    #[test]
    fn a_menu_stays_inside_the_window() {
        // Narrower than the menu: as wide as the window allows.
        let narrow = MenuAt {
            x: 150.0,
            y: 10.0,
            width: 200.0,
            height: 400.0,
        };
        let s = narrow.style();
        assert_eq!((px(&s, "left"), px(&s, "width")), (8.0, 184.0));
        // A phone: a tile at the right edge, low down, opens up and leftwards
        // and keeps its margin from both edges.
        let phone = MenuAt {
            x: 380.0,
            y: 700.0,
            width: 390.0,
            height: 844.0,
        };
        let s = phone.style();
        assert_eq!(px(&s, "left"), 380.0 - MENU_W);
        assert_eq!(px(&s, "bottom"), 144.0);
        assert!(px(&s, "left") + px(&s, "width") <= 390.0 - 8.0);
        // Off the window, or nonsense: at an edge, not off the page.
        for odd in [
            at(-50.0, -50.0),
            at(f64::NAN, f64::INFINITY),
            at(1e12, 1e12),
            MenuAt {
                x: 1.0,
                y: 1.0,
                width: f64::NAN,
                height: -1.0,
            },
        ] {
            let s = odd.style();
            for name in ["left", "width", "max-height"] {
                let v = px(&s, name);
                assert!(v.is_finite() && v >= 0.0, "{name} in {s}");
            }
            let left = px(&s, "left");
            assert!(
                left + px(&s, "width") <= odd.width.max(0.0).max(MENU_W),
                "{s}"
            );
        }
    }

    #[test]
    fn the_menu_is_not_glass_and_grows_from_the_point() {
        let css = include_str!("../../style/base.css");
        let (_, menu) = css.split_once("\n.press-menu {").unwrap();
        let (menu, _) = menu.split_once('}').unwrap();
        assert!(menu.contains("background: var(--surface)"));
        assert!(menu.contains("animation: menu-in"));
        // The item that removes something says so in its colour.
        assert!(css.contains(".menu-item.danger { color: var(--danger); }"));
    }
}
