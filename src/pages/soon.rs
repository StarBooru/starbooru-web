//! What a page shows until it's built: its title and a short note. A page
//! that lands replaces its `Soon` in `app.rs`.

use leptos::prelude::*;

use super::library::LibraryImportCard;
use super::share::SharesCard;
use crate::api;
use crate::components::brand::Mark;
use crate::components::icons::{Ico, Icon};
use crate::route;
use crate::store;

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

/// `/settings` until it's built: the account, with a way to sign out, as the
/// reference's Settings had at its foot, the ways to the recycle bin and the
/// duplicate finder from its Library card, and under that the import of a
/// whole library from another booru, unless the server is set never to reach
/// other sites. The shared links come after the account.
#[component]
pub fn SettingsSoon() -> impl IntoView {
    let info = LocalResource::new(api::server_info);
    let library_import = move || {
        let reaches_out = info.get().flatten().is_some_and(|i| i.url_import);
        (store::signed_in() && reaches_out).then(|| view! { <LibraryImportCard /> })
    };
    let account = move || {
        store::session().map(|s| {
            view! {
                <section class="card">
                    <h2 class="card-title">{move || t!("settings-account")}</h2>
                    <dl class="meta">
                        <dt>{move || t!("settings-signed-in-as")}</dt>
                        <dd class="break">{s.user.username}</dd>
                        <dt>{move || t!("login-server")}</dt>
                        <dd class="break">{s.server}</dd>
                        {move || {
                            info.get()
                                .flatten()
                                .map(|i| {
                                    view! {
                                        <dt>{move || t!("settings-server-version")}</dt>
                                        <dd class="break">
                                            {move || {
                                                t!(
                                                    "login-server-version", name = i.name.clone(), version = i.version.clone()
                                                )
                                            }}
                                        </dd>
                                    }
                                })
                        }}
                    </dl>
                    <div class="row-end">
                        <button class="btn tonal" type="button" on:click=move |_| store::sign_out()>
                            <Ico icon=Icon::Logout />
                            {move || t!("settings-sign-out")}
                        </button>
                    </div>
                </section>
            }
        })
    };
    // Where the reference's Settings had them, after the account.
    let shares = move || store::signed_in().then(|| view! { <SharesCard /> });
    view! {
        <div class="page narrow">
            <h1 class="page-title break">{move || t!("nav-settings")}</h1>
            <section class="card">
                <h2 class="card-title">{move || t!("settings-library")}</h2>
                <a class="list-link" href=route::TRASH>
                    <Ico icon=Icon::Delete />
                    <span>{move || t!("trash-title")}</span>
                    <span class="muted">{move || t!("settings-trash-note")}</span>
                </a>
                <a class="list-link" href=route::DUPLICATES>
                    <Ico icon=Icon::Copy />
                    <span>{move || t!("dup-title")}</span>
                    <span class="muted">{move || t!("settings-dup-note")}</span>
                </a>
            </section>
            {library_import}
            {account}
            {shares}
            <SoonCard />
        </div>
    }
}
