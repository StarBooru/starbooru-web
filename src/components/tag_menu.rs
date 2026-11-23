//! The menu a right-clicked tag opens, on any page that lists tags as links
//! with `data-tag` under `data-press="tag"`: search for it, add it to the
//! search or leave it out, open its page, copy its name. A long press on a
//! phone opens it too where the browser reports one as a right click. It
//! grows out of the point it was opened at.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
use send_wrapper::SendWrapper;
use starbooru_core::normalize_tag;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, HtmlElement, KeyboardEvent, MouseEvent};

use super::icons::{Ico, Icon};
use super::snackbar::notify;
use crate::dom;
use crate::logic::tags::{exclude, include, words};
use crate::route;

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
}

/// The tag a right click was on, and where.
fn pressed(e: &MouseEvent) -> Option<(String, MenuAt)> {
    let link = e
        .target()?
        .dyn_into::<Element>()
        .ok()?
        .closest(r#"[data-press="tag"] [data-tag]"#)
        .ok()??;
    let name = normalize_tag(&link.get_attribute("data-tag")?).ok()?;
    let window = web_sys::window()?;
    let size = |v: Result<JsValue, JsValue>| v.ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
    let (mut x, mut y) = (f64::from(e.client_x()), f64::from(e.client_y()));
    // The menu key has no pointer: under the tag, then.
    if x == 0.0 && y == 0.0 {
        let r = link.get_bounding_client_rect();
        (x, y) = (r.left(), r.bottom());
    }
    Some((
        name,
        MenuAt {
            x,
            y,
            width: size(window.inner_width()),
            height: size(window.inner_height()),
        },
    ))
}

/// Listens for right clicks on tags while it's on the page, and shows their
/// menu. `q` is the search the page belongs to, which a tag can narrow.
#[component]
pub fn TagPress(#[prop(into)] q: Signal<String>) -> impl IntoView {
    let menu = RwSignal::new(None::<(String, MenuAt)>);
    let watch = web_sys::window().and_then(|w| w.document()).map(|doc| {
        dom::listen(&doc, "contextmenu", false, move |e| {
            let Some(found) = e.dyn_ref::<MouseEvent>().and_then(pressed) else {
                return;
            };
            e.prevent_default();
            menu.set(Some(found));
        })
    });
    on_cleanup(move || {
        if let Some(watch) = watch {
            watch.stop();
        }
    });

    let navigate = SendWrapper::new(use_navigate());
    let go = Callback::new(move |to: String| {
        menu.set(None);
        (*navigate)(&to, NavigateOptions::default());
    });
    let close = Callback::new(move |()| menu.set(None));
    move || {
        let (name, at) = menu.get()?;
        let q = q.get_untracked();
        let searching = !q.trim().is_empty();
        let has = words(&q).contains(&name.as_str());
        let search = route::gallery(&name);
        let narrowed = (searching && !has).then(|| route::gallery(&include(&q, &name)));
        let without = route::gallery(&exclude(if searching { &q } else { "" }, &name));
        let details = route::tag(&name);
        let copied = name.clone();
        Some(view! {
            <ActionMenu at=at title=name on_close=close>
                <MenuItem
                    icon=Icon::Search
                    label=move || t!("tag-menu-search")
                    on_click=Callback::new(move |()| go.run(search.clone()))
                />
                {narrowed
                    .map(|to| {
                        view! {
                            <MenuItem
                                icon=Icon::Add
                                label=move || t!("tag-menu-add")
                                on_click=Callback::new(move |()| go.run(to.clone()))
                            />
                        }
                    })}
                <MenuItem
                    icon=Icon::Remove
                    label=move || t!("tag-menu-exclude")
                    on_click=Callback::new(move |()| go.run(without.clone()))
                />
                <MenuItem
                    icon=Icon::Label
                    label=move || t!("tag-menu-details")
                    on_click=Callback::new(move |()| go.run(details.clone()))
                />
                <MenuItem
                    icon=Icon::Copy
                    label=move || t!("tag-menu-copy")
                    on_click=Callback::new(move |()| {
                        menu.set(None);
                        copy_name(copied.clone());
                    })
                />
            </ActionMenu>
        })
    }
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

/// A menu at `at` over a backdrop that closes it. Escape and Tab close it
/// too, and the arrows go through its items, wherever the focus is.
#[component]
fn ActionMenu(
    at: MenuAt,
    title: String,
    on_close: Callback<()>,
    children: Children,
) -> impl IntoView {
    let opened = js_sys::Date::now();
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
        // be dropped while it runs.
        set_timeout(
            move || {
                if let Some(keys) = keys {
                    keys.stop();
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

#[component]
fn MenuItem(
    icon: Icon,
    #[prop(into)] label: Signal<String>,
    on_click: Callback<()>,
) -> impl IntoView {
    view! {
        <button type="button" role="menuitem" class="menu-item" on:click=move |_| on_click.run(())>
            <Ico icon=icon size=20 />
            <span>{move || label.get()}</span>
        </button>
    }
}

/// Copies `name` and says so.
fn copy_name(name: String) {
    spawn_local(async move {
        let done = copy_text(&name).await;
        notify(if done {
            t!("tag-copied")
        } else {
            t!("tag-copy-failed")
        });
    });
}

/// Puts `text` on the clipboard. A page on plain http, other than on this
/// computer, has no clipboard API, and the old way still works there.
async fn copy_text(text: &str) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let navigator = window.navigator();
    let api = js_sys::Reflect::get(&navigator, &JsValue::from_str("clipboard"))
        .is_ok_and(|c| !c.is_undefined() && !c.is_null());
    if api {
        let promise = navigator.clipboard().write_text(text);
        if wasm_bindgen_futures::JsFuture::from(promise).await.is_ok() {
            return true;
        }
    }
    copy_by_selection(text).unwrap_or(false)
}

fn copy_by_selection(text: &str) -> Option<bool> {
    let doc = web_sys::window()?.document()?;
    let field = doc
        .create_element("textarea")
        .ok()?
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .ok()?;
    field.set_value(text);
    let _ = field.set_attribute("style", "position: fixed; opacity: 0");
    doc.body()?.append_child(&field).ok()?;
    field.select();
    let done = doc
        .dyn_ref::<web_sys::HtmlDocument>()
        .and_then(|d| d.exec_command("copy").ok());
    field.remove();
    done
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
        let source = include_str!("tag_menu.rs");
        assert!(source.contains(r#"[data-press="tag"] [data-tag]"#));
    }
}
