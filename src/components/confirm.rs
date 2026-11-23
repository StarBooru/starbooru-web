//! A button for something that can't be taken back: the first press asks
//! "Sure?", the second does it. Moving away, a press anywhere else or a few
//! seconds without one forget the first.

use std::time::Duration;

use leptos::ev::MouseEvent;
use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

use super::icons::{Ico, Icon};
use crate::dom;

/// How long the first press waits for the second.
const ARMED_MS: u64 = 4000;

/// An icon button that grows "Sure?" when armed. With `text` it's a text
/// button instead, and `sure` puts other words in place of "Sure?".
#[component]
pub fn ConfirmButton(
    icon: Icon,
    #[prop(into)] title: Signal<String>,
    #[prop(into)] label: Signal<String>,
    #[prop(into)] disabled: Signal<bool>,
    on_confirm: Callback<()>,
    #[prop(optional, into)] text: Option<Signal<String>>,
    #[prop(optional, into)] sure: Option<Signal<String>>,
    /// The smaller text button.
    #[prop(optional)]
    small: bool,
) -> impl IntoView {
    let confirm = RwSignal::new(false);
    let button: NodeRef<html::Button> = NodeRef::new();
    let timer = StoredValue::new(None::<TimeoutHandle>);
    let outside = StoredValue::new(None::<dom::Watch>);
    let disarm = move || {
        if let Some(t) = timer.try_update_value(Option::take).flatten() {
            t.clear();
        }
        // Stopped a moment later: this may run inside that very listener.
        if let Some(watch) = outside.try_update_value(Option::take).flatten() {
            set_timeout(move || watch.stop(), Duration::ZERO);
        }
        confirm.try_set(false);
    };
    let arm = move || {
        confirm.set(true);
        let t = set_timeout_with_handle(disarm, Duration::from_millis(ARMED_MS));
        timer.set_value(t.ok());
        let watch = web_sys::window().and_then(|w| w.document()).map(|doc| {
            dom::listen_capture(&doc, "pointerdown", move |e| {
                let on_it = e
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::Node>().ok())
                    .zip(button.try_get_untracked().flatten())
                    .is_some_and(|(t, b)| b.contains(Some(&t)));
                if !on_it {
                    disarm();
                }
            })
        });
        outside.set_value(watch);
    };
    on_cleanup(disarm);
    // Armed, its name is the question it shows.
    let name = move || match (confirm.get(), sure) {
        (true, Some(sure)) => sure.get(),
        (true, None) => t!("sure-named", action = label.get()),
        (false, _) => label.get(),
    };
    let class = move || match (text.is_some(), confirm.get()) {
        (false, false) => "icon-btn danger".to_owned(),
        (false, true) => "btn danger small".to_owned(),
        (true, armed) => {
            let class = if armed { "btn danger" } else { "btn tonal" };
            if small {
                format!("{class} small")
            } else {
                class.to_owned()
            }
        }
    };
    let words = move || {
        if confirm.get() {
            Some(sure.map_or_else(|| t!("sure"), |s| s.get()))
        } else {
            text.map(|t| t.get())
        }
    };
    view! {
        <button
            node_ref=button
            type="button"
            class=class
            title=move || title.get()
            aria-label=name
            disabled=move || disabled.get()
            on:blur=move |_| disarm()
            on:click=move |e: MouseEvent| {
                if !confirm.get_untracked() {
                    arm();
                    return;
                }
                // The second click of a double click is still the first press.
                if e.detail() > 1 {
                    return;
                }
                disarm();
                on_confirm.run(());
            }
        >
            <Ico icon=icon />
            {words}
        </button>
    }
}
