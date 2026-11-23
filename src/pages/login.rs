//! The sign-in screen, which the shell shows while no one is signed in, and
//! `/oidc`, where a sign-in through the identity provider comes back to.

use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_location, use_navigate};
use starbooru_client::{Client, ClientError};
use starbooru_core::{Credentials, ServerInfo, password_is_weak};

use crate::api;
use crate::components::brand::{Mark, Wordmark};
use crate::components::icons::{Ico, Icon};
use crate::components::legal::LegalLinks;
use crate::components::strength::StrengthField;
use crate::logic::address::normalize_server;
use crate::oidc::{self, Back};
use crate::store;

#[component]
pub fn Login() -> impl IntoView {
    let server = RwSignal::new(store::default_server());
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let info = RwSignal::new(None::<ServerInfo>);
    let creating = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    // Why this screen is showing, such as a session that expired.
    let note = RwSignal::new(store::take_sign_in_note());
    // Only the answer for the latest address counts.
    let probes = StoredValue::new(0u32);

    // What the server at that address offers.
    let probe = move || {
        let url = server.get_untracked();
        if url.is_empty() {
            return;
        }
        let n = probes.get_value().wrapping_add(1);
        probes.set_value(n);
        spawn_local(async move {
            let result = match Client::new(&url) {
                Ok(client) => client.info().await,
                Err(e) => Err(e),
            };
            // Also gone when the screen is.
            if probes.try_get_value() != Some(n) {
                return;
            }
            match result {
                Ok(i) => {
                    creating.set(!i.has_users);
                    info.set(Some(i));
                    error.set(None);
                }
                Err(e) => {
                    info.set(None);
                    error.set(Some(api::error_text(&e, &url)));
                }
            }
        });
    };
    probe();

    // The password a "too easy to guess" refusal was about. The refusal goes
    // once that password changes, so it never sits beside a strong one.
    let refused = StoredValue::new(None::<String>);
    Effect::new(move |_| {
        let typed = password.get();
        if refused.get_value().is_some_and(|p| p != typed) {
            refused.set_value(None);
            error.set(None);
        }
    });

    let submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        if busy.get_untracked() {
            return;
        }
        let url = normalize_server(&server.get_untracked());
        server.set(url.clone());
        let creds = Credentials {
            username: username.get_untracked().trim().to_owned(),
            password: password.get_untracked(),
        };
        let register = creating.get_untracked();
        note.set(None);
        // The server would refuse it too; this says so before sending.
        if register && password_is_weak(&creds.password, &creds.username) {
            error.set(Some(t!("login-weak-password")));
            refused.set_value(Some(creds.password));
            return;
        }
        refused.set_value(None);
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            let result = match Client::new(&url) {
                Ok(client) if register => client.register(&creds).await,
                Ok(client) => client.login(&creds).await,
                Err(e) => Err(e),
            };
            busy.set(false);
            match result {
                Ok(auth) => store::sign_in(url, auth),
                Err(e) => error.set(Some(sign_in_error(&e, &url))),
            }
        });
    };

    let with_provider = move |_| {
        if busy.get_untracked() {
            return;
        }
        let url = normalize_server(&server.get_untracked());
        server.set(url.clone());
        note.set(None);
        error.set(None);
        busy.set(true);
        spawn_local(async move {
            // Done, the page is on its way to the provider and stays busy.
            if let Err(e) = oidc::begin(url).await {
                busy.set(false);
                error.set(Some(e));
            }
        });
    };
    // Back from the provider without signing in, the browser may bring the
    // page back as it left it, busy.
    let shown = window_event_listener(leptos::ev::pageshow, move |e| {
        if e.persisted() {
            busy.set(false);
        }
    });
    on_cleanup(move || shown.remove());

    let first_user = move || info.with(|i| i.as_ref().is_some_and(|i| !i.has_users));
    let can_register = move || {
        info.with(|i| {
            i.as_ref()
                .is_some_and(|i| i.registration_open && i.has_users)
        })
    };
    let provider = move || info.with(|i| i.as_ref().and_then(|i| i.oidc.clone()));
    let banner = move || error.get().or_else(|| note.get());
    let normalize = move |_| {
        server.update(|s| {
            let tidy = normalize_server(s);
            *s = tidy;
        });
        probe();
    };

    view! {
        <div class="login-page">
            <div class="aurora" aria-hidden="true"></div>
            <form class="login-card glass" on:submit=submit>
                <div class="login-brand">
                    <span class="logo">
                        <Mark height=40 />
                    </span>
                    <h1 aria-label=move || t!("app-name")>
                        <Wordmark height=27 />
                    </h1>
                    <p class="muted break">
                        {move || match info.get() {
                            Some(i) => t!("login-server-version", name = i.name, version = i.version),
                            None => t!("login-tagline"),
                        }}
                    </p>
                </div>
                <Show when=first_user>
                    <div class="banner info">{move || t!("login-first-user")}</div>
                </Show>
                <label class="field">
                    <span>{move || t!("login-server")}</span>
                    <input
                        type="text"
                        inputmode="url"
                        autocomplete="url"
                        autocapitalize="off"
                        spellcheck="false"
                        required
                        placeholder=move || t!("login-server-placeholder")
                        bind:value=server
                        on:change=normalize
                    />
                </label>
                <label class="field">
                    <span>{move || t!("login-username")}</span>
                    <input
                        autocomplete="username"
                        autocapitalize="off"
                        spellcheck="false"
                        required
                        bind:value=username
                    />
                </label>
                <label class="field">
                    <span>{move || t!("login-password")}</span>
                    {move || {
                        if creating.get() {
                            view! {
                                <StrengthField value=password username=username>
                                    <input
                                        type="password"
                                        autocomplete="new-password"
                                        required
                                        minlength="8"
                                        bind:value=password
                                    />
                                </StrengthField>
                            }
                                .into_any()
                        } else {
                            view! {
                                <input
                                    type="password"
                                    autocomplete="current-password"
                                    required
                                    bind:value=password
                                />
                            }
                                .into_any()
                        }
                    }}
                </label>
                {move || {
                    banner()
                        .map(|text| {
                            view! {
                                <div class="banner error" role="alert">
                                    {text}
                                </div>
                            }
                        })
                }}
                <button class="btn wide" type="submit" disabled=move || busy.get()>
                    {move || {
                        if busy.get() {
                            t!("login-wait")
                        } else if first_user() {
                            t!("login-create-admin")
                        } else if creating.get() {
                            t!("login-create")
                        } else {
                            t!("login-sign-in")
                        }
                    }}
                </button>
                {move || {
                    provider()
                        .map(|name| {
                            view! {
                                <div class="or">
                                    <span>{move || t!("login-or")}</span>
                                </div>
                                <button
                                    class="btn tonal wide"
                                    type="button"
                                    disabled=move || busy.get()
                                    on:click=with_provider
                                >
                                    <Ico icon=Icon::Shield />
                                    {move || t!("login-with", provider = name.clone())}
                                </button>
                            }
                        })
                }}
                <Show when=can_register>
                    <button
                        class="btn text wide"
                        type="button"
                        on:click=move |_| creating.update(|c| *c = !*c)
                    >
                        {move || {
                            if creating.get() {
                                t!("login-have-account")
                            } else {
                                t!("login-want-account")
                            }
                        }}
                    </button>
                </Show>
                <LegalLinks />
            </form>
        </div>
    }
}

/// A failed sign-in or sign-up, in words.
fn sign_in_error(e: &ClientError, server: &str) -> String {
    if e.is_unauthorized() {
        t!("login-wrong")
    } else if e.code() == Some("weak_password") {
        t!("login-weak-password")
    } else {
        api::error_text(e, server)
    }
}

/// `/oidc`: the browser back from the identity provider. It finishes the
/// sign-in and goes to the gallery, or to the sign-in screen saying why not.
#[component]
pub fn OidcReturn() -> impl IntoView {
    let query = use_location().search.get_untracked();
    let navigate = use_navigate();
    let home = move || {
        navigate(
            "/",
            NavigateOptions {
                replace: true,
                ..NavigateOptions::default()
            },
        );
    };
    // Once this page is drawn, as the router takes no new address mid-draw.
    // Untracked, so it runs once: the handle can be taken only once.
    Effect::new(move |_| {
        untrack(|| match oidc::back(&query, oidc::take()) {
            Back::Nothing => home(),
            Back::Failed(why) => {
                store::set_sign_in_note(why.text());
                home();
            }
            Back::Finish { server, finish } => {
                let home = home.clone();
                spawn_local(async move {
                    let result = match Client::new(&server) {
                        Ok(client) => client.oidc_finish(&finish).await,
                        Err(e) => Err(e),
                    };
                    match result {
                        Ok(auth) => store::sign_in(server, auth),
                        Err(e) => store::set_sign_in_note(oidc::finish_error(&e, &server)),
                    }
                    home();
                });
            }
        });
    });
    view! {
        <div class="login-page">
            <div class="aurora" aria-hidden="true"></div>
            <div class="login-card glass">
                <p class="muted status-line" role="status">
                    {move || t!("oidc-finishing")}
                </p>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use starbooru_core::ApiError;

    use super::*;

    fn api(status: u16, code: &str) -> ClientError {
        ClientError::Api {
            status,
            error: ApiError {
                code: code.into(),
                message: "server words".into(),
            },
        }
    }

    #[test]
    fn a_failed_sign_in_says_what_to_do() {
        let server = "https://booru.example";
        assert_eq!(
            sign_in_error(&api(401, "unauthorized"), server),
            "Wrong username or password."
        );
        assert!(sign_in_error(&api(400, "weak_password"), server).starts_with("This password"));
        assert_eq!(
            sign_in_error(
                &ClientError::RateLimited {
                    retry_after: Some(120)
                },
                server
            ),
            "Too many attempts. Try again in 2 minutes."
        );
        assert_eq!(
            sign_in_error(&api(403, "forbidden"), server),
            "server words"
        );
    }
}
