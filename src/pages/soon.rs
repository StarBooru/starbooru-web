//! What a page shows until it's built: its title and a short note. A page
//! that lands replaces its `Soon` in `app.rs`.

use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::components::brand::Mark;

/// `title` is the page's name, translated.
#[component]
pub fn Soon(#[prop(into)] title: TextProp) -> impl IntoView {
    view! {
        <div class="page narrow">
            <h1 class="page-title break">{move || title.get().to_string()}</h1>
            <SoonCard />
        </div>
    }
}

/// The same for a page outside the shell, with no bars around it.
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

#[component]
fn SoonCard() -> impl IntoView {
    view! {
        <div class="card empty">
            <span class="soon-mark">
                <Mark height=56 />
            </span>
            <p class="muted">{move || t!("soon-body")}</p>
        </div>
    }
}

/// `/post/:id`.
#[component]
pub fn PostSoon() -> impl IntoView {
    let params = use_params_map();
    let title = move || {
        let id = params.with(|p| p.get("id").unwrap_or_default());
        t!("post-number", id = id)
    };
    view! { <Soon title=title /> }
}

/// `/tag?name=…`: the tag's name, or Tags without one.
#[component]
pub fn TagSoon() -> impl IntoView {
    let query = use_query_map();
    let title = move || {
        query
            .with(|q| q.get("name"))
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| t!("nav-tags"))
    };
    view! { <Soon title=title /> }
}
