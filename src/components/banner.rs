//! What a page shows while it waits, when a request failed, and when the
//! server corrected a search.

use leptos::prelude::*;
use starbooru_core::TagCorrection;

use crate::logic::search::corrections;

/// Which unknown tags a search was corrected to, with a link to the same
/// search as typed (`exact`, from `route::exact`).
#[component]
pub fn CorrectionBanner(
    #[prop(into)] corrected: Signal<Vec<TagCorrection>>,
    #[prop(into)] exact: Signal<String>,
) -> impl IntoView {
    view! {
        <div class="banner info" role="status">
            <span class="break">
                {move || corrected.with(|c| t!("search-corrected", tags = corrections(c)))}
            </span>
            <a class="btn text" href=move || exact.get()>
                {move || t!("search-exact")}
            </a>
        </div>
    }
}

/// A failed request, in words the user can act on, with a way to try again.
#[component]
pub fn ErrorBanner(
    #[prop(into)] message: Signal<String>,
    #[prop(optional)] on_retry: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="banner error" role="alert">
            <span class="break">{move || message.get()}</span>
            {on_retry
                .map(|retry| {
                    view! {
                        <button type="button" class="btn text" on:click=move |_| retry.run(())>
                            {move || t!("retry")}
                        </button>
                    }
                })}
        </div>
    }
}

#[component]
pub fn Spinner() -> impl IntoView {
    view! { <span class="spinner" role="status" aria-label=move || t!("loading")></span> }
}
