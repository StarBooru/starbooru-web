//! The app's pages. `app.rs` says which address shows which.

pub mod gallery;
pub mod help;
pub mod login;
pub mod post;
mod search_demo;
pub mod shell;
pub mod soon;

use leptos::prelude::*;
use leptos_router::hooks::use_location;
use starbooru_core::Rating;

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

/// A rating's name in the current language.
pub(crate) fn rating_label(rating: Rating) -> String {
    match rating {
        Rating::Safe => t!("rating-safe"),
        Rating::Sensitive => t!("rating-sensitive"),
        Rating::Questionable => t!("rating-questionable"),
        Rating::Explicit => t!("rating-explicit"),
    }
}
