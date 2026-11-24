//! Settings › Account: who is signed in, how much they've uploaded, where,
//! and a way out; the ways to sign in, with the server's identity provider
//! to link; and the password.

use chrono::Utc;
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{ServerInfo, SignInMethods};

use crate::components::banner::ErrorBanner;
use crate::components::confirm::ConfirmButton;
use crate::components::icons::{Ico, Icon};
use crate::components::strength::StrengthField;
use crate::logic::settings::{
    MIN_PASSWORD_CHARS, check_password, identity_name, identity_times, initial, last_way_in,
    role_label, usage_text, wrong_current,
};
use crate::{api, oidc, store};

#[component]
pub(super) fn AccountCard(info: LocalResource<Option<ServerInfo>>) -> impl IntoView {
    let usage =
        LocalResource::new(
            || async move { api::call(|c| async move { c.usage().await }).await.ok() },
        );
    let usage = move || {
        usage.get().flatten().map(|u| {
            let meter = u.fraction().map(|f| {
                let percent = f * 100.0;
                let full = f >= 0.9;
                view! {
                    <span
                        class="meter"
                        class:full=full
                        role="meter"
                        aria-valuemin="0"
                        aria-valuemax="100"
                        aria-valuenow=format!("{percent:.0}")
                        aria-label=move || t!("settings-storage")
                    >
                        <span style=format!("width: {percent:.1}%")></span>
                    </span>
                }
            });
            view! {
                <dt>{move || t!("settings-storage")}</dt>
                <dd class="usage">
                    <span>{move || usage_text(&u)}</span>
                    {meter}
                </dd>
            }
        })
    };
    let server_version = move || {
        info.get().flatten().map(|i| {
            view! {
                <dt>{move || t!("settings-server-version")}</dt>
                <dd class="break">
                    {move || t!("login-server-version", name = i.name.clone(), version = i.version.clone())}
                </dd>
            }
        })
    };
    move || {
        store::session().map(|s| {
            let role = s.user.role;
            let badge = initial(&s.user.username);
            view! {
                <section class="card" id="account">
                    // The badge and the name say it.
                    <h3 class="visually-hidden">{move || t!("settings-account")}</h3>
                    <div class="profile">
                        <span class="avatar big" aria-hidden="true">{badge}</span>
                        <span class="profile-name">
                            <span class="visually-hidden">{move || t!("settings-signed-in-as")}</span>
                            <strong class="break">{s.user.username}</strong>
                            <span class="pill">
                                <span class="visually-hidden">{move || t!("settings-role")} " "</span>
                                {move || role_label(role)}
                            </span>
                        </span>
                    </div>
                    <dl class="meta">
                        {usage}
                        <dt>{move || t!("login-server")}</dt>
                        <dd class="break">{s.server}</dd>
                        {server_version}
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
    }
}

/// The account's password and its provider accounts, with linking and
/// unlinking. The last way in can't be removed.
#[component]
pub(super) fn SignInCard(
    methods: RwSignal<Option<SignInMethods>>,
    provider: Signal<Option<String>>,
) -> impl IntoView {
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let link = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            // Done, the page is on its way to the provider and stays busy.
            if let Err(message) = oidc::begin_link().await {
                busy.try_set(false);
                error.try_set(Some(message));
            }
        });
    };
    // Back from the provider without linking, the browser may bring the page
    // back as it left it, busy.
    let shown = window_event_listener(leptos::ev::pageshow, move |e| {
        if e.persisted() {
            busy.set(false);
        }
    });
    on_cleanup(move || shown.remove());
    let unlink = Callback::new(move |id: i64| {
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            match api::call(move |c| async move { c.unlink_identity(id).await }).await {
                Ok(()) => {
                    methods.try_update(|m| {
                        if let Some(m) = m {
                            m.identities.retain(|i| i.id != id);
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
        let m = methods.get()?;
        let now = Utc::now();
        let last = last_way_in(&m);
        let provider = provider.get();
        let identities = m
            .identities
            .iter()
            .map(|identity| {
                let id = identity.id;
                let name = identity_name(identity, provider.as_deref());
                let times = identity_times(identity, now);
                let label = {
                    let name = name.clone();
                    Signal::derive(move || t!("signin-unlink-named", name = name.clone()))
                };
                view! {
                    <li class="session">
                        <div class="session-main">
                            <strong class="break">{name}</strong>
                            <span class="muted">{times}</span>
                        </div>
                        <ConfirmButton
                            icon=Icon::Close
                            title=Signal::derive(|| t!("signin-unlink"))
                            label=label
                            disabled=Signal::derive(move || busy.get() || last)
                            on_confirm=Callback::new(move |()| unlink.run(id))
                        />
                    </li>
                }
            })
            .collect_view();
        let password = m.password;
        Some(view! {
            <ul class="sessions">
                <li class="session">
                    <div class="session-main">
                        <strong>{move || t!("login-password")}</strong>
                        <span class="muted">
                            {move || {
                                if password { t!("signin-password-set") } else { t!("signin-password-none") }
                            }}
                        </span>
                    </div>
                </li>
                {identities}
            </ul>
            <Show when=move || last>
                <p class="muted hint">{move || t!("signin-last")}</p>
            </Show>
        })
    };
    let link_button = move || {
        provider.get().map(|name| {
            view! {
                <div class="row-end">
                    <button class="btn tonal" type="button" disabled=move || busy.get() on:click=link>
                        <Ico icon=Icon::Link />
                        {move || t!("signin-link", provider = name.clone())}
                    </button>
                </div>
            }
        })
    };
    view! {
        <section class="card" id="sign-in">
            <h3 class="card-title">{move || t!("signin-title")}</h3>
            {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
            {rows}
            {link_button}
        </section>
    }
}

/// Changes the password, or sets the first one for an account a provider
/// made. The server signs the other devices out, so `on_changed` hears of it.
#[component]
pub(super) fn PasswordCard(
    methods: RwSignal<Option<SignInMethods>>,
    on_changed: Callback<()>,
) -> impl IntoView {
    let current = RwSignal::new(String::new());
    let new = RwSignal::new(String::new());
    let repeat = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let done = RwSignal::new(false);
    // Until the server says, as most accounts have one.
    let has_password = Memo::new(move |_| methods.with(|m| m.as_ref().is_none_or(|m| m.password)));
    let username = Signal::derive(|| {
        store::session()
            .map(|s| s.user.username)
            .unwrap_or_default()
    });
    let check = Memo::new(move |_| {
        check_password(
            has_password.get(),
            &current.get(),
            &new.get(),
            &repeat.get(),
        )
    });

    let submit = move |e: SubmitEvent| {
        e.prevent_default();
        if busy.get_untracked() || !check.get_untracked().ready {
            return;
        }
        let (old, fresh) = (current.get_untracked(), new.get_untracked());
        let had_password = has_password.get_untracked();
        busy.set(true);
        error.set(None);
        done.set(false);
        spawn_local(async move {
            let result =
                api::attempt(move |c| async move { c.change_password(&old, &fresh).await }).await;
            match result {
                Ok(()) => {
                    current.try_set(String::new());
                    new.try_set(String::new());
                    repeat.try_set(String::new());
                    done.try_set(true);
                    methods.try_update(|m| {
                        if let Some(m) = m {
                            m.password = true;
                        }
                    });
                    on_changed.try_run(());
                }
                Err(e) if wrong_current(had_password, e.status, e.code.as_deref(), &e.message) => {
                    error.try_set(Some(untrack(|| t!("password-wrong-current"))));
                }
                // Anything else in the server's words.
                Err(e) => {
                    error.try_set(Some(e.message));
                }
            }
            busy.try_set(false);
        });
    };

    let invalid = |on: bool| if on { "true" } else { "false" };
    view! {
        <section class="card" id="password">
            <h3 class="card-title">{move || t!("login-password")}</h3>
            <Show when=move || !has_password.get()>
                <p class="muted hint">{move || t!("password-none")}</p>
            </Show>
            <form on:submit=submit>
                // A hidden username, so a password manager knows whose it is.
                <input
                    class="visually-hidden"
                    type="text"
                    autocomplete="username"
                    tabindex="-1"
                    aria-hidden="true"
                    readonly=true
                    prop:value=move || username.get()
                />
                <Show when=move || has_password.get()>
                    <label class="field">
                        <span>{move || t!("password-current")}</span>
                        <input type="password" autocomplete="current-password" bind:value=current />
                    </label>
                </Show>
                <label class="field">
                    <span>{move || t!("password-new")}</span>
                    <StrengthField value=new username=username>
                        <input
                            type="password"
                            autocomplete="new-password"
                            aria-invalid=move || invalid(check.get().too_short)
                            bind:value=new
                        />
                    </StrengthField>
                    <Show when=move || check.get().too_short>
                        <small class="field-error">
                            {move || t!("password-too-short", count = MIN_PASSWORD_CHARS)}
                        </small>
                    </Show>
                </label>
                <label class="field">
                    <span>{move || t!("password-repeat")}</span>
                    <input
                        type="password"
                        autocomplete="new-password"
                        aria-invalid=move || invalid(check.get().mismatch)
                        bind:value=repeat
                    />
                    <Show when=move || check.get().mismatch>
                        <small class="field-error">{move || t!("password-mismatch")}</small>
                    </Show>
                </label>
                {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
                <Show when=move || done.get()>
                    <div class="banner info" role="status">
                        {move || t!("password-changed")}
                    </div>
                </Show>
                <div class="row-end">
                    <button class="btn" type="submit" disabled=move || !check.get().ready || busy.get()>
                        {move || {
                            if busy.get() {
                                t!("password-changing")
                            } else if has_password.get() {
                                t!("password-change")
                            } else {
                                t!("password-set")
                            }
                        }}
                    </button>
                </div>
            </form>
        </section>
    }
}
