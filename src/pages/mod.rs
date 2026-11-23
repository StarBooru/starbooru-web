//! The app's pages. `app.rs` says which address shows which.

pub mod gallery;
pub mod login;
pub mod post;
pub mod shell;
pub mod soon;

use leptos::prelude::*;
use leptos_router::hooks::use_location;

/// An address that leads nowhere.
#[component]
pub fn NotFound() -> impl IntoView {
    let path = use_location().pathname;
    view! {
        <div class="solo-page">
            <div class="card empty">
                <h1>{move || t!("notfound-title")}</h1>
                <p class="muted break">{move || t!("notfound-body", path = path.get())}</p>
                <a class="btn" href="/">
                    {move || t!("notfound-back")}
                </a>
            </div>
        </div>
    }
}
