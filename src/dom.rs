//! The few things the pages ask of the browser directly: watching the window
//! scroll, an element resize or come near the screen, and scrolling. Each
//! watcher comes back as a [`Watch`] to stop when its page goes.

use send_wrapper::SendWrapper;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, IntersectionObserverEntry, ScrollBehavior, ScrollToOptions};

/// A watcher, which stops when dropped. It can go into `on_cleanup`, which
/// wants `Send`, as the app only ever runs on the browser's one thread.
pub struct Watch(Option<SendWrapper<Box<dyn FnOnce()>>>);

impl Watch {
    fn new(stop: impl FnOnce() + 'static) -> Self {
        Self(Some(SendWrapper::new(Box::new(stop))))
    }

    /// For a watcher the browser wouldn't set up: nothing to stop.
    fn none() -> Self {
        Self(None)
    }

    pub fn stop(self) {
        drop(self);
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        if let Some(stop) = self.0.take() {
            (stop.take())();
        }
    }
}

/// Calls `cb` on every `event` (`scroll`, `resize`) of the window. Passive,
/// so the browser never waits for it to scroll.
pub fn on_window(event: &'static str, cb: impl FnMut() + 'static) -> Watch {
    let Some(window) = web_sys::window() else {
        return Watch::none();
    };
    let closure = Closure::<dyn FnMut()>::new(cb);
    let options = web_sys::AddEventListenerOptions::new();
    options.set_passive(true);
    let added = window.add_event_listener_with_callback_and_add_event_listener_options(
        event,
        closure.as_ref().unchecked_ref(),
        &options,
    );
    if added.is_err() {
        return Watch::none();
    }
    Watch::new(move || {
        let _ = window.remove_event_listener_with_callback(event, closure.as_ref().unchecked_ref());
        drop(closure);
    })
}

/// Calls `cb` with every `event` on `target`. A passive listener can't
/// cancel the event, and lets the browser scroll without waiting for it.
pub fn listen(
    target: &web_sys::EventTarget,
    event: &'static str,
    passive: bool,
    cb: impl FnMut(web_sys::Event) + 'static,
) -> Watch {
    let closure = Closure::<dyn FnMut(web_sys::Event)>::new(cb);
    let options = web_sys::AddEventListenerOptions::new();
    options.set_passive(passive);
    let added = target.add_event_listener_with_callback_and_add_event_listener_options(
        event,
        closure.as_ref().unchecked_ref(),
        &options,
    );
    if added.is_err() {
        return Watch::none();
    }
    let target = target.clone();
    Watch::new(move || {
        let _ = target.remove_event_listener_with_callback(event, closure.as_ref().unchecked_ref());
        drop(closure);
    })
}

/// [`listen`] in the capture phase, so `cb` sees the event before the
/// element it is for, and can stop it.
pub fn listen_capture(
    target: &web_sys::EventTarget,
    event: &'static str,
    cb: impl FnMut(web_sys::Event) + 'static,
) -> Watch {
    let closure = Closure::<dyn FnMut(web_sys::Event)>::new(cb);
    let options = web_sys::AddEventListenerOptions::new();
    options.set_capture(true);
    let added = target.add_event_listener_with_callback_and_add_event_listener_options(
        event,
        closure.as_ref().unchecked_ref(),
        &options,
    );
    if added.is_err() {
        return Watch::none();
    }
    let target = target.clone();
    Watch::new(move || {
        let _ = target.remove_event_listener_with_callback_and_bool(
            event,
            closure.as_ref().unchecked_ref(),
            true,
        );
        drop(closure);
    })
}

/// Waits `ms` milliseconds.
pub async fn sleep(ms: i32) {
    let promise = js_sys::Promise::new(&mut |done, _| {
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&done, ms);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

/// A box with a caret: a one-line field or a text area.
pub trait Caret {
    /// The caret's place, in UTF-16 units.
    fn caret(&self) -> Option<u32>;
}

impl Caret for web_sys::HtmlInputElement {
    fn caret(&self) -> Option<u32> {
        self.selection_start().ok().flatten()
    }
}

impl Caret for web_sys::HtmlTextAreaElement {
    fn caret(&self) -> Option<u32> {
        self.selection_start().ok().flatten()
    }
}

/// Where the caret is in `el`, in bytes into `value`, its text. At the end
/// when the browser doesn't say.
pub fn caret_of(el: &impl Caret, value: &str) -> usize {
    el.caret()
        .and_then(|c| usize::try_from(c).ok())
        .map_or(value.len(), |c| crate::logic::search::byte_index(value, c))
}

/// Whether an event comes from a place the user types in, where keys are
/// text and not commands.
pub fn typing(target: Option<web_sys::EventTarget>) -> bool {
    let Some(el) = target.and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok()) else {
        return false;
    };
    el.is_content_editable() || matches!(el.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT")
}

/// Calls `cb` with the exact width of `el` now and whenever it changes. A
/// rounded width adds up to a visible error far down a grid.
pub fn on_width(el: &Element, mut cb: impl FnMut(f64) + 'static) -> Watch {
    let target = el.clone();
    let closure = Closure::<dyn FnMut()>::new(move || {
        if target.is_connected() {
            cb(target.get_bounding_client_rect().width());
        }
    });
    let Ok(observer) = web_sys::ResizeObserver::new(closure.as_ref().unchecked_ref()) else {
        return Watch::none();
    };
    observer.observe(el);
    Watch::new(move || {
        observer.disconnect();
        drop(closure);
    })
}

/// Calls `cb` with the width and height of `el`'s content box now and
/// whenever they change.
pub fn on_size(el: &web_sys::HtmlElement, mut cb: impl FnMut(f64, f64) + 'static) -> Watch {
    let target = el.clone();
    let closure = Closure::<dyn FnMut()>::new(move || {
        if target.is_connected() {
            cb(
                f64::from(target.client_width()),
                f64::from(target.client_height()),
            );
        }
    });
    let Ok(observer) = web_sys::ResizeObserver::new(closure.as_ref().unchecked_ref()) else {
        return Watch::none();
    };
    observer.observe(el);
    Watch::new(move || {
        observer.disconnect();
        drop(closure);
    })
}

/// The window's inner width and height, in CSS pixels.
pub fn window_size() -> (f64, f64) {
    let Some(window) = web_sys::window() else {
        return (0.0, 0.0);
    };
    let size = |v: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>| {
        v.ok().and_then(|v| v.as_f64()).unwrap_or(0.0)
    };
    (size(window.inner_width()), size(window.inner_height()))
}

/// Calls `cb` when `el` comes within `margin` (CSS lengths, as for
/// `IntersectionObserver`) of the screen.
pub fn on_near(el: &Element, margin: &str, mut cb: impl FnMut() + 'static) -> Watch {
    let closure = Closure::<dyn FnMut(js_sys::Array)>::new(move |entries: js_sys::Array| {
        let near = entries.iter().any(|e| {
            e.dyn_into::<IntersectionObserverEntry>()
                .is_ok_and(|e| e.is_intersecting())
        });
        if near {
            cb();
        }
    });
    let init = web_sys::IntersectionObserverInit::new();
    init.set_root_margin(margin);
    let Ok(observer) =
        web_sys::IntersectionObserver::new_with_options(closure.as_ref().unchecked_ref(), &init)
    else {
        return Watch::none();
    };
    observer.observe(el);
    Watch::new(move || {
        observer.disconnect();
        drop(closure);
    })
}

/// Whether the user asked for less motion.
pub fn reduced_motion() -> bool {
    web_sys::window()
        .and_then(|w| {
            w.match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .is_some_and(|m| m.matches())
}

/// How far down the page is scrolled, and how tall the window is.
pub fn scroll() -> (f64, f64) {
    let Some(window) = web_sys::window() else {
        return (0.0, 0.0);
    };
    let y = window.scroll_y().unwrap_or(0.0);
    let tall = window
        .inner_height()
        .ok()
        .and_then(|h| h.as_f64())
        .unwrap_or(0.0);
    (y, tall)
}

/// Scrolls the page to `top`, gliding there unless `smooth` is off or the user
/// asked for less motion.
pub fn scroll_to(top: f64, smooth: bool) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let options = ScrollToOptions::new();
    options.set_top(top.max(0.0));
    options.set_behavior(if smooth && !reduced_motion() {
        ScrollBehavior::Smooth
    } else {
        ScrollBehavior::Instant
    });
    window.scroll_to_with_scroll_to_options(&options);
}

/// Scrolls the page `dy` pixels down, or up when below zero, at once: it's
/// for a step of a scroll that goes on frame by frame.
pub fn scroll_by(dy: f64) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let options = ScrollToOptions::new();
    options.set_top(dy);
    options.set_behavior(ScrollBehavior::Instant);
    window.scroll_by_with_scroll_to_options(&options);
}

/// The bottom edge of the top bar, in pixels from the top of the window:
/// what's above it is hidden behind the search.
pub fn bar_bottom() -> f64 {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.query_selector(".topbar").ok().flatten())
        .map_or(0.0, |bar| bar.get_bounding_client_rect().bottom())
}

/// The top edge of the navigation at the bottom of a phone, or the bottom of
/// the window where there's none: what's below it is hidden behind it.
pub fn dock_top() -> f64 {
    let (_, tall) = scroll();
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.query_selector(".nav-bottom").ok().flatten())
        .map(|nav| nav.get_bounding_client_rect())
        // At the side, as a rail, it takes no height from the window.
        .filter(|r| r.width() > r.height() && r.top() > 0.0)
        .map_or(tall, |r| r.top())
}

/// How far the middle of the first element matching `selector` that is
/// drawn lies from the middle of the window, for something to grow out of it.
pub fn offset_from_middle(selector: &str) -> Option<(f64, f64)> {
    let window = web_sys::window()?;
    let list = window.document()?.query_selector_all(selector).ok()?;
    let wide = window.inner_width().ok()?.as_f64()?;
    let tall = window.inner_height().ok()?.as_f64()?;
    (0..list.length())
        .filter_map(|i| list.item(i)?.dyn_into::<Element>().ok())
        .map(|el| el.get_bounding_client_rect())
        .find(|r| r.width() > 0.0 && r.height() > 0.0)
        .map(|r| {
            (
                r.left() + r.width() / 2.0 - wide / 2.0,
                r.top() + r.height() / 2.0 - tall / 2.0,
            )
        })
}

/// Where the first element matching `selector` sits, as its distance from
/// each edge of the window (top, right, bottom, left) and how round its
/// corners are. One out of sight stands in as a small square in the middle.
pub fn edges_of(selector: &str) -> Option<[f64; 5]> {
    let window = web_sys::window()?;
    let wide = window.inner_width().ok()?.as_f64()?;
    let tall = window.inner_height().ok()?.as_f64()?;
    let middle = || {
        let side = 80.0;
        let (across, down) = ((wide / 2.0 - side).max(0.0), (tall / 2.0 - side).max(0.0));
        [down, across, down, across, 24.0]
    };
    let Some(el) = window.document()?.query_selector(selector).ok()? else {
        return Some(middle());
    };
    let r = el.get_bounding_client_rect();
    if r.width() == 0.0 || r.bottom() <= 0.0 || r.top() >= tall {
        return Some(middle());
    }
    let radius = window
        .get_computed_style(&el)
        .ok()
        .flatten()
        .and_then(|s| s.get_property_value("border-top-left-radius").ok())
        .and_then(|v| v.trim().trim_end_matches("px").parse().ok())
        .unwrap_or(0.0);
    Some([
        r.top().max(0.0),
        (wide - r.right()).max(0.0),
        (tall - r.bottom()).max(0.0),
        r.left().max(0.0),
        radius,
    ])
}

/// Puts class `name` on `<html>`, or takes it off.
pub fn root_class(name: &str, on: bool) {
    if let Some(root) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let _ = root.class_list().toggle_with_force(name, on);
    }
}

/// Puts `text` on the clipboard. A page on plain http, other than on this
/// computer, has no clipboard API, and the old way still works there.
pub async fn copy_text(text: &str) -> bool {
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

/// The browser's own share sheet (`navigator.share`), where it has one: on
/// phones and some desktops, and only on https.
fn share_api() -> Option<(web_sys::Navigator, js_sys::Function)> {
    let navigator = web_sys::window()?.navigator();
    let share = js_sys::Reflect::get(&navigator, &JsValue::from_str("share")).ok()?;
    let share = share.dyn_into::<js_sys::Function>().ok()?;
    Some((navigator, share))
}

/// Whether the browser has a share sheet of its own.
pub fn can_share() -> bool {
    share_api().is_some()
}

/// Opens the browser's share sheet for `url`. Closing it without sharing
/// rejects the promise, which is ignored like any other refusal.
pub async fn share(title: &str, url: &str) {
    let Some((navigator, share)) = share_api() else {
        return;
    };
    let data = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&data, &JsValue::from_str("url"), &JsValue::from_str(url));
    if !title.is_empty() {
        let _ = js_sys::Reflect::set(
            &data,
            &JsValue::from_str("title"),
            &JsValue::from_str(title),
        );
    }
    if let Ok(answer) = share.call1(&navigator, &data)
        && let Ok(promise) = answer.dyn_into::<js_sys::Promise>()
    {
        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
    }
}

/// The page's own origin, `https://host[:port]`.
pub fn origin() -> Option<String> {
    web_sys::window()?.location().origin().ok()
}

/// The browser puts a page back where it was on Back by itself, which would
/// fight the gallery putting its tile back. The app does it instead.
pub fn manual_scroll_restoration() {
    if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
        let _ = history.set_scroll_restoration(web_sys::ScrollRestoration::Manual);
    }
}
