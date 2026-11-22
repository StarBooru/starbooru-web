//! The root of the app.

use leptos::prelude::*;

use crate::i18n;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <div class="app" lang=move || i18n::lang().code()>
            <main class="placeholder">
                <h1>{move || t!("app-name")}</h1>
                <p class="muted">{move || t!("placeholder-soon")}</p>
            </main>
        </div>
    }
}
