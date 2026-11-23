//! The links to the privacy policy and the terms, under the sign-in card and
//! in Settings.

use leptos::prelude::*;

#[component]
pub fn LegalLinks() -> impl IntoView {
    view! {
        <nav class="legal-links" aria-label=move || t!("legal-nav")>
            <a href="/privacy">{move || t!("legal-privacy-short")}</a>
            <a href="/terms">{move || t!("legal-terms-short")}</a>
        </nav>
    }
}
