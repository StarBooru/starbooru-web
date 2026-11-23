//! What a page outside the shell shows until it's built: its title and a
//! short note. A page that lands replaces its `SoonAlone` in `app.rs`.

use leptos::prelude::*;

/// `title` is the page's name, translated.
#[component]
pub fn SoonAlone(#[prop(into)] title: TextProp) -> impl IntoView {
    view! {
        <div class="solo-page">
            <div class="card empty">
                <h1 class="page-title break">{move || title.get().to_string()}</h1>
                <p class="muted">{move || t!("soon-body")}</p>
                <a class="btn" href="/">
                    {move || t!("notfound-back")}
                </a>
            </div>
        </div>
    }
}
