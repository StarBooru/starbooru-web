//! Installing StarBooru as an app.
//!
//! The service worker (`assets/sw.js`) is what lets a browser install the app
//! and start it without a network; [`register_worker`] registers it once the
//! app is running. Chromium also offers installing with an event, which
//! Settings › About shows as a button for as long as the offer stands. Safari
//! and Firefox make no such offer, so they show nothing.

use std::cell::RefCell;

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

use crate::components::snackbar::notify;
use crate::dom::{self, Watch};

/// The worker's address: at the root, so every page is in its scope.
const WORKER: &str = "/sw.js";
/// Where `index.html` keeps an offer made before the app started.
const EARLY_OFFER: &str = "starbooruInstall";

/// Where the browser's offer to install stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Offer {
    /// None made: a browser without the offer, or the app is installed.
    #[default]
    None,
    /// Made, and the button can take it up.
    Ready,
    /// The browser's own dialog is open.
    Asking,
    /// Installed while this page was open.
    Installed,
}

/// Whether the Install button shows: while the offer stands, and not in the
/// app's own window.
pub fn shows(offer: Offer, standalone: bool) -> bool {
    !standalone && matches!(offer, Offer::Ready | Offer::Asking)
}

/// The offer once the browser's dialog closes. One turned down is spent: the
/// browser makes another later, or doesn't.
pub fn after_choice(outcome: Option<&str>) -> Offer {
    match outcome {
        Some("accepted") => Offer::Installed,
        _ => Offer::None,
    }
}

thread_local! {
    // Reference counted, so no component's owner can dispose of them.
    static OFFER: ArcRwSignal<Offer> = ArcRwSignal::new(Offer::None);
    static STANDALONE: ArcRwSignal<bool> = ArcRwSignal::new(false);
    /// The browser's event, which asks when it's prompted, once.
    static EVENT: RefCell<Option<JsValue>> = const { RefCell::new(None) };
    static WATCHES: RefCell<Vec<Watch>> = const { RefCell::new(Vec::new()) };
}

/// Whether Settings offers installing. Read inside a view, it redraws on a
/// change.
pub fn offered() -> bool {
    shows(OFFER.with(|o| o.get()), STANDALONE.with(|s| s.get()))
}

/// Whether the browser's dialog is open.
pub fn asking() -> bool {
    OFFER.with(|o| o.get()) == Offer::Asking
}

fn set(offer: Offer) {
    OFFER.with(|o| {
        if o.get_untracked() != offer {
            o.set(offer);
        }
    });
}

fn take(event: JsValue) {
    EVENT.with_borrow_mut(|e| *e = Some(event));
    set(Offer::Ready);
}

fn prop(target: &JsValue, name: &str) -> Option<JsValue> {
    js_sys::Reflect::get(target, &JsValue::from_str(name))
        .ok()
        .filter(|v| !v.is_undefined() && !v.is_null())
}

/// Starts listening for the browser's offer. Call once, from the root.
pub fn watch() {
    let Some(window) = web_sys::window() else {
        return;
    };
    if let Some(early) = prop(&window, EARLY_OFFER) {
        let _ = js_sys::Reflect::delete_property(&window, &JsValue::from_str(EARLY_OFFER));
        take(early);
    }
    let keep = |watch: Watch| WATCHES.with_borrow_mut(|w| w.push(watch));
    // Not passive: holding the browser's own banner back cancels the event.
    keep(dom::listen(&window, "beforeinstallprompt", false, |e| {
        e.prevent_default();
        take(e.into());
    }));
    // From the button, or from the browser's own menu.
    keep(dom::listen(&window, "appinstalled", true, |_| {
        EVENT.with_borrow_mut(Option::take);
        set(Offer::Installed);
        notify(t!("install-done"));
    }));
    if let Ok(Some(query)) = window.match_media("(display-mode: standalone)") {
        STANDALONE.with(|s| s.set(query.matches()));
        let target = query.clone();
        keep(dom::listen(&target, "change", true, move |_| {
            STANDALONE.with(|s| s.set(query.matches()));
        }));
    }
}

/// Opens the browser's install dialog. Called from a click: the browser opens
/// it only for one.
pub fn ask() {
    let Some(event) = EVENT.with_borrow_mut(Option::take) else {
        set(Offer::None);
        return;
    };
    let shown = prop(&event, "prompt")
        .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
        .is_some_and(|prompt| prompt.call0(&event).is_ok());
    let choice = prop(&event, "userChoice").and_then(|p| p.dyn_into::<js_sys::Promise>().ok());
    let Some(choice) = choice.filter(|_| shown) else {
        set(Offer::None);
        return;
    };
    set(Offer::Asking);
    spawn_local(async move {
        let choice = wasm_bindgen_futures::JsFuture::from(choice).await.ok();
        let outcome = choice
            .and_then(|c| prop(&c, "outcome"))
            .and_then(|o| o.as_string());
        // `appinstalled` may have come first.
        if OFFER.with(|o| o.get_untracked()) == Offer::Asking {
            set(after_choice(outcome.as_deref()));
        }
    });
}

/// Registers the service worker once the page has loaded, so it never holds
/// up the start. Only a secure origin (https, or this computer) has them.
pub fn register_worker() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let navigator = window.navigator();
    if prop(&navigator, "serviceWorker").is_none() {
        return;
    }
    let register = move || {
        let registered = navigator.service_worker().register(WORKER);
        // An app without the worker still works, online.
        spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(registered).await;
        });
    };
    let loaded = window
        .document()
        .is_some_and(|d| d.ready_state() == "complete");
    if loaded {
        register();
        return;
    }
    let options = web_sys::AddEventListenerOptions::new();
    options.set_once(true);
    let on_load = Closure::once_into_js(register);
    let _ = window.add_event_listener_with_callback_and_add_event_listener_options(
        "load",
        on_load.unchecked_ref(),
        &options,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_button_shows_while_the_offer_stands_outside_the_apps_window() {
        assert!(shows(Offer::Ready, false));
        assert!(shows(Offer::Asking, false));
        assert!(!shows(Offer::None, false));
        assert!(!shows(Offer::Installed, false));
        for offer in [Offer::None, Offer::Ready, Offer::Asking, Offer::Installed] {
            assert!(!shows(offer, true), "{offer:?}");
        }
    }

    #[test]
    fn a_dialog_turned_down_spends_the_offer() {
        assert_eq!(after_choice(Some("accepted")), Offer::Installed);
        assert_eq!(after_choice(Some("dismissed")), Offer::None);
        assert_eq!(after_choice(None), Offer::None);
    }

    #[test]
    fn the_page_links_the_manifest_and_copies_the_worker() {
        let page = include_str!("../index.html");
        for line in [
            r#"<link rel="manifest" href="/manifest.webmanifest">"#,
            r#"<link data-trunk rel="copy-file" href="assets/manifest.webmanifest">"#,
            r#"<link data-trunk rel="copy-file" href="assets/sw.js">"#,
            r#"<meta name="mobile-web-app-capable" content="yes">"#,
            r#"<meta name="apple-mobile-web-app-title" content="StarBooru">"#,
        ] {
            assert!(page.contains(line), "{line}");
        }
        assert!(page.contains(&format!("window.{EARLY_OFFER} = event;")));
        assert_eq!(WORKER, "/sw.js");
    }

    #[test]
    fn the_manifest_starts_in_the_default_canvas_with_icons_that_exist() {
        let manifest: serde_json::Value =
            serde_json::from_str(include_str!("../assets/manifest.webmanifest")).unwrap();
        let (light, _) = crate::theme::bar_colors(
            crate::store::Theme::System,
            crate::theme::Accent::preset(285),
            false,
        );
        assert_eq!(manifest["theme_color"], light.as_str());
        assert_eq!(manifest["background_color"], light.as_str());
        assert_eq!(manifest["start_url"], "/");
        assert_eq!(manifest["scope"], "/");
        assert_eq!(manifest["display"], "standalone");
        let page = include_str!("../index.html");
        let icons = manifest["icons"].as_array().unwrap();
        assert!(icons.iter().any(|i| i["purpose"] == "maskable"));
        for icon in icons {
            let src = icon["src"].as_str().unwrap();
            let name = src.trim_start_matches('/');
            // Copied to the root by Trunk.
            let copied = format!(r#"<link data-trunk rel="copy-file" href="assets/brand/{name}">"#);
            assert!(page.contains(&copied), "{src}");
        }
    }

    #[test]
    fn the_worker_leaves_the_servers_paths_alone() {
        let worker = include_str!("../assets/sw.js");
        assert!(worker.contains(r"const SERVER = /^\/(api|media|remote|backups|healthz)(\/|$)/;"));
        assert!(worker.contains(r#"if (request.method !== "GET") return;"#));
        assert!(worker.contains(
            "if (url.origin !== self.location.origin || SERVER.test(url.pathname)) return;"
        ));
    }
}
