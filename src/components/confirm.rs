//! A button for something that can't be taken back: the first press asks
//! "Sure?", the second does it, and moving away forgets the first.

use leptos::prelude::*;

use super::icons::{Ico, Icon};

#[component]
pub fn ConfirmButton(
    icon: Icon,
    #[prop(into)] title: Signal<String>,
    #[prop(into)] label: Signal<String>,
    #[prop(into)] disabled: Signal<bool>,
    on_confirm: Callback<()>,
) -> impl IntoView {
    let confirm = RwSignal::new(false);
    view! {
        <button
            type="button"
            class=move || if confirm.get() { "btn danger small" } else { "icon-btn danger" }
            title=move || title.get()
            aria-label=move || label.get()
            disabled=move || disabled.get()
            on:blur=move |_| confirm.set(false)
            on:click=move |_| {
                if confirm.get_untracked() {
                    confirm.set(false);
                    on_confirm.run(());
                } else {
                    confirm.set(true);
                }
            }
        >
            <Ico icon=icon />
            <Show when=move || confirm.get()>{move || t!("sure")}</Show>
        </button>
    }
}
