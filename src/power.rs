//! Battery saver: less downloading, decoding and animating while it's on.
//! Automatic, the default, follows the device: on while the battery is low
//! and not charging, or while the browser says to save data.
//!
//! While it's on, `.app` carries `data-saver="on"`: animations are cut short,
//! a video under the mouse doesn't play in its tile, and the viewer doesn't
//! fetch the next post ahead. Videos pause whenever the tab is hidden.

use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::{JsCast, JsValue};

use crate::dom::{self, Watch};
use crate::store::{self, Saver};

thread_local! {
    // Reference counted, so no component's owner can dispose of it.
    static DEVICE_LOW: ArcRwSignal<bool> = ArcRwSignal::new(false);
    // The device's listeners, for as long as the app runs.
    static WATCHES: RefCell<Vec<Watch>> = const { RefCell::new(Vec::new()) };
}

/// The battery level at or under which Automatic turns the saver on.
const LOW_BATTERY: f64 = 0.2;

/// Whether the saver is on, for a setting and the device's state.
pub fn saving(setting: Saver, device_low: bool) -> bool {
    match setting {
        Saver::Always => true,
        Saver::Never => false,
        Saver::Auto => device_low,
    }
}

/// Whether the device wants saving: data saver on, or a battery at
/// `level` (0 to 1) that isn't charging and is low. Either may be unknown.
pub fn low(save_data: bool, battery: Option<(f64, bool)>) -> bool {
    save_data || battery.is_some_and(|(level, charging)| !charging && level <= LOW_BATTERY)
}

/// Whether the saver is on now. Read inside a view, it redraws on a change.
pub fn active() -> bool {
    saving(store::prefs().battery_saver, device_low())
}

/// The same, outside a view.
pub fn active_untracked() -> bool {
    saving(
        store::prefs_untracked().battery_saver,
        DEVICE_LOW.with(|d| d.get_untracked()),
    )
}

/// Whether the device is low right now, for Settings.
pub fn device_low() -> bool {
    DEVICE_LOW.with(|d| d.get())
}

/// A property of a browser object, if it has it.
fn prop(target: &JsValue, name: &str) -> Option<JsValue> {
    js_sys::Reflect::get(target, &JsValue::from_str(name))
        .ok()
        .filter(|v| !v.is_undefined() && !v.is_null())
}

/// What the device says now. The battery API is Chromium's alone, and data
/// saver a phone's: without them, neither counts.
fn read(connection: Option<&JsValue>, battery: Option<&JsValue>) -> bool {
    let save_data = connection
        .and_then(|c| prop(c, "saveData"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let battery = battery.and_then(|b| {
        let level = prop(b, "level")?.as_f64()?;
        let charging = prop(b, "charging")?.as_bool()?;
        Some((level, charging))
    });
    low(save_data, battery)
}

fn keep(watch: Watch) {
    WATCHES.with_borrow_mut(|w| w.push(watch));
}

/// Starts watching the device. Call once, from the root.
pub fn watch() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let navigator = JsValue::from(window.navigator());
    let connection = prop(&navigator, "connection");
    let battery = Rc::new(RefCell::new(None::<JsValue>));
    let update = {
        let connection = connection.clone();
        let battery = Rc::clone(&battery);
        Rc::new(move || {
            let now = read(connection.as_ref(), battery.borrow().as_ref());
            DEVICE_LOW.with(|d| {
                if d.get_untracked() != now {
                    d.set(now);
                }
            });
        })
    };
    update();
    if let Some(target) = connection.and_then(|c| c.dyn_into::<web_sys::EventTarget>().ok()) {
        let update = Rc::clone(&update);
        keep(dom::listen(&target, "change", true, move |_| update()));
    }
    // A video doesn't play on in a tab no one is looking at.
    if let Some(doc) = window.document() {
        let target = doc.clone();
        keep(dom::listen(&target, "visibilitychange", true, move |_| {
            if doc.hidden() {
                pause_videos(&doc);
            }
        }));
    }
    let get_battery =
        prop(&navigator, "getBattery").and_then(|f| f.dyn_into::<js_sys::Function>().ok());
    let Some(get_battery) = get_battery else {
        return;
    };
    spawn_local(async move {
        let Some(promise) = get_battery
            .call0(&navigator)
            .ok()
            .and_then(|p| p.dyn_into::<js_sys::Promise>().ok())
        else {
            return;
        };
        let Ok(found) = wasm_bindgen_futures::JsFuture::from(promise).await else {
            return;
        };
        *battery.borrow_mut() = Some(found.clone());
        update();
        if let Ok(target) = found.dyn_into::<web_sys::EventTarget>() {
            for event in ["levelchange", "chargingchange"] {
                let update = Rc::clone(&update);
                keep(dom::listen(&target, event, true, move |_| update()));
            }
        }
    });
}

fn pause_videos(doc: &web_sys::Document) {
    let Ok(videos) = doc.query_selector_all("video") else {
        return;
    };
    for i in 0..videos.length() {
        if let Some(video) = videos
            .item(i)
            .and_then(|v| v.dyn_into::<web_sys::HtmlMediaElement>().ok())
        {
            let _ = video.pause();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follows_the_device_only_when_automatic() {
        assert!(saving(Saver::Always, false));
        assert!(!saving(Saver::Never, true));
        assert!(saving(Saver::Auto, true));
        assert!(!saving(Saver::Auto, false));
    }

    #[test]
    fn a_low_battery_counts_only_off_the_charger() {
        assert!(!low(false, None));
        assert!(low(true, None));
        assert!(low(false, Some((0.2, false))));
        assert!(low(false, Some((0.05, false))));
        assert!(!low(false, Some((0.05, true))));
        assert!(!low(false, Some((0.21, false))));
        assert!(low(true, Some((1.0, true))));
    }

    #[test]
    fn the_saver_cuts_motion_short_but_keeps_spinners_turning() {
        let css = include_str!("../style/base.css");
        assert!(css.contains(
            r#".app[data-saver="on"] *:not(.spinner), .app[data-saver="on"] *::before, .app[data-saver="on"] *::after {"#
        ));
        let app = include_str!("app.rs");
        assert!(app.contains("data-saver=move || if saver.get() { \"on\" } else { \"off\" }"));
    }
}
