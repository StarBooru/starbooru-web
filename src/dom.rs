//! The few things the pages ask of the browser directly: watching the window
//! scroll, an element resize or come near the screen, and scrolling. Each
//! watcher comes back as a [`Watch`] to stop when its page goes.

use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Element, IntersectionObserverEntry, ScrollBehavior, ScrollToOptions};

/// A watcher to stop. It can go into `on_cleanup`, which wants `Send`, as the
/// app only ever runs on the browser's one thread.
pub struct Watch(Option<SendWrapper<Box<dyn FnOnce()>>>);

impl Watch {
    fn new(stop: impl FnOnce() + 'static) -> Self {
        Self(Some(SendWrapper::new(Box::new(stop))))
    }

    /// For a watcher the browser wouldn't set up: nothing to stop.
    fn none() -> Self {
        Self(None)
    }

    pub fn stop(mut self) {
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

/// The bottom edge of the top bar, in pixels from the top of the window:
/// what's above it is hidden behind the search.
pub fn bar_bottom() -> f64 {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.query_selector(".topbar").ok().flatten())
        .map_or(0.0, |bar| bar.get_bounding_client_rect().bottom())
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

/// The browser puts a page back where it was on Back by itself, which would
/// fight the gallery putting its tile back. The app does it instead.
pub fn manual_scroll_restoration() {
    if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
        let _ = history.set_scroll_restoration(web_sys::ScrollRestoration::Manual);
    }
}
