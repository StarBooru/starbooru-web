//! Settings › API keys: long-lived tokens for scripts and other tools. A new
//! key's token shows once, with a button that copies it; the list has each
//! key's start, what it may do, when it was last used and when it expires.

use chrono::Utc;
use leptos::ev::SubmitEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::keys::MAX_API_KEY_NAME_CHARS;
use starbooru_core::{ApiKey, CreatedApiKey};

use super::ConfirmButton;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::icons::{Ico, Icon};
use crate::components::select::{Choice, Select};
use crate::logic::settings::{
    KEY_LIFETIMES, key_error_text, key_expired, key_lifetime_days, key_lifetime_label,
    key_lifetime_value, key_status, new_key,
};
use crate::{api, dom};

#[component]
pub(super) fn ApiKeysCard() -> impl IntoView {
    let list = RwSignal::new(None::<Vec<ApiKey>>);
    let error = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let name = RwSignal::new(String::new());
    let read_only = RwSignal::new(false);
    let lifetime = RwSignal::new(key_lifetime_value(None));
    let created = RwSignal::new(None::<CreatedApiKey>);

    let load = move || {
        error.set(None);
        spawn_local(async move {
            match api::call(|c| async move { c.api_keys().await }).await {
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

    let create = move |e: SubmitEvent| {
        e.prevent_default();
        if busy.get_untracked() {
            return;
        }
        let days = key_lifetime_days(&lifetime.get_untracked());
        let request = match new_key(&name.get_untracked(), read_only.get_untracked(), days) {
            Ok(request) => request,
            Err(e) => {
                error.set(Some(untrack(|| key_error_text(&e))));
                return;
            }
        };
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            match api::call(move |c| async move { c.create_api_key(&request).await }).await {
                Ok(done) => {
                    list.try_update(|l| {
                        if let Some(l) = l {
                            l.insert(0, done.key.clone());
                        }
                    });
                    name.try_set(String::new());
                    created.try_set(Some(done));
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
            busy.try_set(false);
        });
    };
    let remove = Callback::new(move |id: i64| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            match api::call(move |c| async move { c.delete_api_key(id).await }).await {
                Ok(()) => {
                    list.try_update(|l| {
                        if let Some(l) = l {
                            l.retain(|k| k.id != id);
                        }
                    });
                    if created.try_with_untracked(|c| c.as_ref().is_some_and(|c| c.key.id == id))
                        == Some(true)
                    {
                        created.try_set(None);
                    }
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
            busy.try_set(false);
        });
    });

    let fresh = move || {
        created.get().map(|done| {
            let label = done.key.name.clone();
            view! {
                <div class="banner info key-created" role="status">
                    <strong class="break">{move || t!("keys-copy-now", name = label.clone())}</strong>
                    <TokenField token=done.token />
                </div>
            }
        })
    };
    let rows = move || {
        let keys = list.get()?;
        if keys.is_empty() {
            return Some(
                view! { <p class="muted keys-none">{move || t!("keys-none")}</p> }.into_any(),
            );
        }
        let now = Utc::now();
        let rows = keys
            .into_iter()
            .map(|k| {
                let read_only = k.read_only.then(|| {
                    view! { <span class="pill">{move || t!("keys-read-only")}</span> }
                });
                let gone = key_expired(&k, now).then(|| {
                    view! { <span class="pill bad">{move || t!("keys-expired")}</span> }
                });
                let label = {
                    let name = k.name.clone();
                    Signal::derive(move || t!("keys-delete-named", name = name.clone()))
                };
                let id = k.id;
                view! {
                    <li class="session">
                        <div class="session-main">
                            <strong class="break">{k.name.clone()}</strong>
                            <code class="key-prefix">{format!("{}…", k.prefix)}</code>
                            {read_only}
                            {gone}
                            <span class="muted">{key_status(&k, now)}</span>
                        </div>
                        <ConfirmButton
                            icon=Icon::Delete
                            title=Signal::derive(|| t!("keys-delete"))
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
    let access = [false, true]
        .into_iter()
        .map(|value| {
            let on = move || read_only.get() == value;
            view! {
                <button
                    type="button"
                    role="radio"
                    class="seg"
                    class:on=on
                    aria-checked=move || if on() { "true" } else { "false" }
                    on:click=move |_| read_only.set(value)
                >
                    {move || if value { t!("keys-read-only") } else { t!("keys-full") }}
                </button>
            }
        })
        .collect_view();
    let lifetimes = Signal::derive(|| {
        KEY_LIFETIMES
            .into_iter()
            .map(|d| Choice::new(key_lifetime_value(d), key_lifetime_label(d)))
            .collect::<Vec<_>>()
    });

    view! {
        <section class="card">
            <h2 class="card-title">{move || t!("keys-title")}</h2>
            <p class="muted hint">{move || t!("keys-lead")}</p>
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
            {fresh}
            <Show when=waiting>
                <Spinner />
            </Show>
            {rows}
            <form class="key-form" on:submit=create>
                <label class="field">
                    <span>{move || t!("keys-name")}</span>
                    <input
                        placeholder=move || t!("keys-name-placeholder")
                        maxlength=MAX_API_KEY_NAME_CHARS.to_string()
                        autocomplete="off"
                        bind:value=name
                    />
                </label>
                <div class="key-options">
                    <div class="field">
                        <span>{move || t!("keys-access")}</span>
                        <div class="segmented" role="radiogroup" aria-label=move || t!("keys-access")>
                            {access}
                        </div>
                    </div>
                    <div class="field">
                        <span aria-hidden="true">{move || t!("keys-lifetime")}</span>
                        <Select
                            value=lifetime
                            options=lifetimes
                            label=Signal::derive(|| t!("keys-lifetime"))
                            on_change=Callback::new(move |v: String| lifetime.set(v))
                        />
                    </div>
                </div>
                <div class="row-end">
                    <button
                        class="btn"
                        type="submit"
                        disabled=move || busy.get() || name.with(|n| n.trim().is_empty())
                    >
                        <Ico icon=Icon::Add />
                        {move || t!("keys-create")}
                    </button>
                </div>
            </form>
        </section>
    }
}

/// A new key's token, in a field to select, with a button that copies it.
/// Where the clipboard won't take it, the token is selected to copy by hand.
#[component]
fn TokenField(token: String) -> impl IntoView {
    let copied = RwSignal::new(false);
    let failed = RwSignal::new(false);
    let field: NodeRef<html::Input> = NodeRef::new();
    let select = move || {
        if let Some(el) = field.get_untracked() {
            let _ = el.focus();
            el.select();
        }
    };
    let text = token.clone();
    let copy = move |_| {
        let text = text.clone();
        spawn_local(async move {
            let done = dom::copy_text(&text).await;
            copied.try_set(done);
            failed.try_set(!done);
            if !done {
                select();
            }
        });
    };
    view! {
        <div class="key-token">
            <input
                node_ref=field
                readonly=true
                spellcheck="false"
                aria-label=move || t!("keys-token")
                prop:value=token
                on:focus=move |_| {
                    if let Some(el) = field.get_untracked() {
                        el.select();
                    }
                }
            />
            <button class="btn tonal small" type="button" on:click=copy>
                {move || {
                    let icon = if copied.get() { Icon::Check } else { Icon::Copy };
                    view! { <Ico icon=icon size=18 /> }
                }}
                {move || if copied.get() { t!("keys-copied") } else { t!("keys-copy") }}
            </button>
        </div>
        <Show when=move || failed.get()>
            <p class="field-error" role="alert">
                {move || t!("keys-copy-failed")}
            </p>
        </Show>
    }
}
