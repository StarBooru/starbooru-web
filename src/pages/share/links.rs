//! Settings › Shared links: every link the account made, newest first, with
//! its cover, how often it was opened and when it expires, whether chats
//! show its picture, the link to copy, and a way to stop it working.

use chrono::Utc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::Share;

use super::LinkField;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::confirm::ConfirmButton;
use crate::components::icons::{Ico, Icon};
use crate::logic::share::{expired, flip_preview, previewed, share_name, share_status, share_url};
use crate::{api, instance};

/// Whether chats show the link's picture, which a press turns on or off.
/// It follows the account's choice until pressed into another, and follows
/// it again once pressed back. Only while the server allows previews.
fn preview_chip(
    share: &Share,
    account: RwSignal<Option<bool>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView + use<> {
    let id = share.id;
    let own = RwSignal::new(share.preview);
    let told = share.preview_on;
    let busy = RwSignal::new(false);
    let on =
        Memo::new(move |_| previewed(instance::link_previews(), account.get(), own.get(), told));
    let flip = move |_| {
        let Some(mine) = account.get_untracked() else {
            return;
        };
        if busy.get_untracked() {
            return;
        }
        let update = flip_preview(mine, on.get_untracked());
        let was = own.get_untracked();
        if let Some(next) = update.preview {
            own.set(next);
        }
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            match api::call(move |c| async move { c.update_share(id, &update).await }).await {
                Ok(kept) => {
                    own.try_set(kept.preview);
                }
                Err(message) => {
                    own.try_set(was);
                    error.try_set(Some(message));
                }
            }
            busy.try_set(false);
        });
    };
    move || {
        instance::link_previews().then(|| {
            view! {
                <button
                    type="button"
                    class="chip preview-chip"
                    class:on=move || on.get()
                    aria-pressed=move || if on.get() { "true" } else { "false" }
                    disabled=move || busy.get() || account.get().is_none()
                    on:click=flip
                >
                    {move || {
                        let icon = if on.get() { Icon::Gallery } else { Icon::Link };
                        view! { <Ico icon=icon size=18 /> }
                    }}
                    <span>{move || t!("share-preview")}</span>
                </button>
            }
        })
    }
}

/// `account` is the account's choice of previews, once read.
#[component]
pub fn SharesCard(account: RwSignal<Option<bool>>) -> impl IntoView {
    let list = RwSignal::new(None::<Vec<Share>>);
    let error = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);

    let load = move || {
        error.set(None);
        spawn_local(async move {
            match api::call(|c| async move { c.shares().await }).await {
                Ok(found) => {
                    list.try_set(Some(found));
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    };
    load();

    let remove = Callback::new(move |id: i64| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            match api::call(move |c| async move { c.delete_share(id).await }).await {
                Ok(()) => {
                    list.try_update(|l| {
                        if let Some(l) = l {
                            l.retain(|s| s.id != id);
                        }
                    });
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
            busy.try_set(false);
        });
    });

    let rows = move || {
        // The list failed to load: the banner says so and offers another go.
        let shares = list.get()?;
        if shares.is_empty() {
            return Some(view! { <p class="muted">{move || t!("shares-none")}</p> }.into_any());
        }
        let client = api::client();
        let now = Utc::now();
        let rows = shares
            .into_iter()
            .map(|s| {
                let cover = match (s.cover_url.as_deref(), client.as_ref()) {
                    (Some(cover), Some(c)) => view! {
                        <img src=c.absolute(cover) alt="" loading="lazy" decoding="async" />
                    }
                    .into_any(),
                    _ => view! { <Ico icon=Icon::Link /> }.into_any(),
                };
                let url = client.as_ref().map(|c| share_url(c, &s.token));
                let name = share_name(&s);
                let gone = expired(&s, now).then(|| {
                    view! { <span class="pill bad">{move || t!("shares-expired")}</span> }
                });
                let posts = s.posts;
                let status = share_status(&s, now);
                let chip = preview_chip(&s, account, error);
                let id = s.id;
                let label = {
                    let name = name.clone();
                    Signal::derive(move || t!("shares-delete-named", name = name.clone()))
                };
                view! {
                    <li class="session share">
                        <div class="share-cover">{cover}</div>
                        <div class="session-main">
                            <strong class="break">{name}</strong>
                            <span class="pill">{move || t!("share-posts", count = posts)}</span>
                            {gone}
                            <span class="muted">{status}</span>
                            {chip}
                            {url.map(|url| view! { <LinkField url=url /> })}
                        </div>
                        <ConfirmButton
                            icon=Icon::Delete
                            title=Signal::derive(|| t!("shares-delete"))
                            label=label
                            disabled=busy
                            on_confirm=Callback::new(move |()| remove.run(id))
                        />
                    </li>
                }
            })
            .collect_view();
        Some(view! { <ul class="sessions">{rows}</ul> }.into_any())
    };
    let waiting = move || list.with(Option::is_none) && error.with(Option::is_none);

    view! {
        <section class="card shares" id="links">
            <h3 class="card-title">{move || t!("shares-title")}</h3>
            <p class="muted hint">{move || t!("shares-lead")}</p>
            {move || {
                error
                    .get()
                    .map(|message| {
                        let retry = list
                            .with(Option::is_none)
                            .then(|| Callback::new(move |()| load()));
                        match retry {
                            Some(retry) => view! { <ErrorBanner message=message on_retry=retry /> }.into_any(),
                            None => view! { <ErrorBanner message=message /> }.into_any(),
                        }
                    })
            }}
            <Show when=waiting>
                <Spinner />
            </Show>
            {rows}
        </section>
    }
}
