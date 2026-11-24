//! What stands in for a page this visitor can't use: for a guest, why it
//! needs an account and the way to one, which comes back here signed in;
//! on a public server, the other boorus' pages, which it doesn't have.

use leptos::prelude::*;

use crate::components::banner::Spinner;
use crate::components::icons::{Ico, Icon};
use crate::{instance, route};

/// Sign in, and Create an account where the server takes new ones. Both open
/// the sign-in screen over the page, which stays where it is.
#[component]
pub fn SignInButtons() -> impl IntoView {
    view! {
        <div class="invite-actions">
            <button type="button" class="btn" on:click=|_| instance::ask_sign_in(None)>
                <Ico icon=Icon::Login />
                {move || t!("guest-sign-in")}
            </button>
            <Show when=instance::registration_open>
                <button type="button" class="btn tonal" on:click=|_| instance::ask_sign_up()>
                    {move || t!("guest-sign-up")}
                </button>
            </Show>
        </div>
    }
}

/// Why something needs an account, with the way to one.
#[component]
pub fn SignInCard(#[prop(into)] why: Signal<String>) -> impl IntoView {
    view! {
        <div class="card empty invite">
            <span class="invite-mark" aria-hidden="true">
                <Ico icon=Icon::User size=40 />
            </span>
            <h2>{move || t!("guest-invite-title")}</h2>
            <p class="muted">{move || why.get()}</p>
            <SignInButtons />
        </div>
    }
}

/// A page that needs an account, or for a guest the invitation to one.
#[component]
pub fn MembersOnly(children: ChildrenFn) -> impl IntoView {
    view! {
        <Show
            when=move || !instance::guest()
            fallback=|| {
                view! {
                    <div class="center-page">
                        <SignInCard why=Signal::derive(|| t!("guest-page-needs-account")) />
                    </div>
                }
            }
        >
            {children()}
        </Show>
    }
}

/// The other boorus' pages, which a public server doesn't have: a short
/// note in their place, and the way back to Explore. Nothing until the
/// server has said which it is, so a public one isn't asked for boorus.
#[component]
pub fn PrivateOnly(children: ChildrenFn) -> impl IntoView {
    let shown = Memo::new(|_| instance::known().then(instance::public));
    move || match shown.get() {
        None => view! {
            <div class="sentinel">
                <Spinner />
            </div>
        }
        .into_any(),
        Some(false) => children().into_any(),
        Some(true) => view! {
            <div class="center-page">
                <div class="card empty">
                    <Ico icon=Icon::Explore size=48 />
                    <h2>{move || t!("boorus-not-here")}</h2>
                    <p class="muted">{move || t!("boorus-not-here-body")}</p>
                    <a class="btn" href=route::EXPLORE>
                        {move || t!("boorus-not-here-back")}
                    </a>
                </div>
            </div>
        }
        .into_any(),
    }
}
