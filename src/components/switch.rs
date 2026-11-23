//! An on/off switch with its icon on the knob, so it isn't taken for a
//! button that does something once.

use leptos::prelude::*;

use super::icons::{Ico, Icon};

#[component]
pub fn Switch(
    icon: Icon,
    #[prop(into)] label: Signal<String>,
    #[prop(into)] on: Signal<bool>,
    on_change: Callback<bool>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class="switch"
            class:on=move || on.get()
            role="switch"
            aria-checked=move || if on.get() { "true" } else { "false" }
            aria-label=move || label.get()
            title=move || label.get()
            on:click=move |_| on_change.run(!on.get_untracked())
        >
            <span class="switch-knob">
                <Ico icon=icon size=16 />
            </span>
        </button>
    }
}
