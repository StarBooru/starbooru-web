//! Settings › Shared links: every link the account made, newest first, with
//! its cover, how often it was opened and when it expires, the link to copy,
//! and a way to stop it working.

use chrono::Utc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::Share;

use super::LinkField;
use crate::api;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::icons::{Ico, Icon};
use crate::logic::share::{expired, share_name, share_status, share_url};

#[component]
pub fn SharesCard() -> impl IntoView {
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
                view! {
                    <li class="session share">
                        <div class="share-cover">{cover}</div>
                        <div class="session-main">
                            <strong class="break">{name.clone()}</strong>
                            <span class="pill">{move || t!("share-posts", count = posts)}</span>
                            {gone}
                            <span class="muted">{status}</span>
                            {url.map(|url| view! { <LinkField url=url /> })}
                        </div>
                        <ShareDelete id=s.id name=name busy=busy on_delete=remove />
                    </li>
                }
            })
            .collect_view();
        Some(view! { <ul class="sessions">{rows}</ul> }.into_any())
    };
    let waiting = move || list.with(Option::is_none) && error.with(Option::is_none);

    view! {
        <section class="card shares">
            <h2 class="card-title">{move || t!("shares-title")}</h2>
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

/// Removes one link, after a second press that says "Sure?".
#[component]
fn ShareDelete(
    id: i64,
    name: String,
    busy: RwSignal<bool>,
    on_delete: Callback<i64>,
) -> impl IntoView {
    let confirm = RwSignal::new(false);
    view! {
        <button
            type="button"
            class=move || if confirm.get() { "btn danger small" } else { "icon-btn danger" }
            title=move || t!("shares-delete")
            aria-label=move || t!("shares-delete-named", name = name.clone())
            disabled=move || busy.get()
            on:blur=move |_| confirm.set(false)
            on:click=move |_| {
                if confirm.get_untracked() {
                    confirm.set(false);
                    on_delete.run(id);
                } else {
                    confirm.set(true);
                }
            }
        >
            <Ico icon=Icon::Delete />
            <Show when=move || confirm.get()>{move || t!("sure")}</Show>
        </button>
    }
}
