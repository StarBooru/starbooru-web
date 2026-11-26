//! Settings › Sharing › Link previews: whether the account's links show
//! their first picture in chats, unless a link says otherwise. Only while
//! the server allows previews; otherwise the card says so, and leads an
//! administrator to the switch.

use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{Role, SharingSettings};

use crate::components::banner::ErrorBanner;
use crate::components::icons::{Ico, Icon};
use crate::components::link_card::LinkCardPicture;
use crate::logic::admin::Tab;
use crate::{api, instance, store};

/// `account` is the account's setting once read, which the list of links
/// follows too.
#[component]
pub fn PreviewsCard(account: RwSignal<Option<bool>>) -> impl IntoView {
    let error = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let load = move || {
        error.set(None);
        spawn_local(async move {
            match api::call(|c| async move { c.sharing().await }).await {
                Ok(s) => {
                    account.try_set(Some(s.previews));
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    };
    load();

    let server = Memo::new(|_| instance::link_previews());
    let on = Memo::new(move |_| server.get() && account.get().unwrap_or(false));
    let admin = Memo::new(|_| store::session().is_some_and(|s| s.user.role == Role::Admin));
    let flip = move |_| {
        let Some(was) = account.get_untracked() else {
            return;
        };
        if busy.get_untracked() || !server.get_untracked() {
            return;
        }
        // The switch moves at once and goes back if the server says no.
        account.set(Some(!was));
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            let wanted = SharingSettings { previews: !was };
            match api::call(move |c| async move { c.set_sharing(&wanted).await }).await {
                Ok(kept) => {
                    account.try_set(Some(kept.previews));
                }
                Err(message) => {
                    account.try_set(Some(was));
                    error.try_set(Some(message));
                }
            }
            busy.try_set(false);
        });
    };
    let off_here = move || {
        (!server.get()).then(|| {
            let way = admin.get().then(|| {
                view! {
                    <a class="btn tonal small" href=format!("{}#previews", Tab::Server.href())>
                        <Ico icon=Icon::Tune size=18 />
                        {move || t!("previews-server-off-admin")}
                    </a>
                }
            });
            view! {
                <div class="banner info previews-off" role="status">
                    <span>{move || t!("previews-server-off")}</span>
                    {way}
                </div>
            }
        })
    };

    view! {
        <section class="card previews-card" id="previews">
            <h3 class="card-title">{move || t!("previews-title")}</h3>
            <div class="previews-body">
                <LinkCardPicture on=on />
                <div class="previews-side">
                    <button
                        type="button"
                        class="list-link switch-row"
                        role="switch"
                        aria-checked=move || if on.get() { "true" } else { "false" }
                        aria-labelledby="previews-mine-name"
                        aria-describedby="previews-mine-hint"
                        disabled=move || !server.get() || account.get().is_none()
                        on:click=flip
                    >
                        <span id="previews-mine-name">{move || t!("previews-mine")}</span>
                        <span id="previews-mine-hint" class="muted">
                            {move || t!("previews-mine-hint")}
                        </span>
                        <span class="switch" class:on=move || on.get() aria-hidden="true">
                            <span class="switch-knob">
                                {move || {
                                    let icon = if on.get() { Icon::Check } else { Icon::Close };
                                    view! { <Ico icon=icon size=16 /> }
                                }}
                            </span>
                        </span>
                    </button>
                    <p class="muted fine">{move || t!("previews-copies")}</p>
                </div>
            </div>
            {off_here}
            {move || {
                error
                    .get()
                    .map(|message| {
                        let retry = account
                            .with(Option::is_none)
                            .then(|| Callback::new(move |()| load()));
                        match retry {
                            Some(retry) => view! { <ErrorBanner message=message on_retry=retry /> }.into_any(),
                            None => view! { <ErrorBanner message=message /> }.into_any(),
                        }
                    })
            }}
        </section>
    }
}
