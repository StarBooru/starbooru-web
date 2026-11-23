//! What a page shows while it waits, and when a request failed.

use leptos::prelude::*;

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
