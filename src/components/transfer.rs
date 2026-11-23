//! A bar for an upload or a download under way, with how much is done and
//! how fast it goes.

use leptos::prelude::*;
use starbooru_core::Transfer;

use crate::logic::upload::transfer_label;

/// Without `progress`, or without a size to measure against, the bar only
/// shows that something is moving. `name` says what is moving, for a screen
/// reader.
#[component]
pub fn TransferBar(
    #[prop(into)] progress: Signal<Option<Transfer>>,
    #[prop(into)] name: Signal<String>,
) -> impl IntoView {
    let percent = move || progress.get().and_then(|p| p.percent());
    let label = move || {
        progress
            .get()
            .as_ref()
            .map(transfer_label)
            .unwrap_or_default()
    };
    view! {
        <div class="transfer">
            <span
                class="meter"
                class:unknown=move || percent().is_none()
                role="progressbar"
                aria-label=move || name.get()
                aria-valuemin="0"
                aria-valuemax="100"
                aria-valuenow=move || percent().map(|p| p.to_string())
                aria-valuetext=label
            >
                <span style=move || percent().map(|p| format!("width: {p}%"))></span>
            </span>
            <Show when=move || !label().is_empty()>
                <span class="transfer-text">{label}</span>
            </Show>
        </div>
    }
}
