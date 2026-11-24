//! Settings › Signed-in devices: where this account is signed in, this
//! browser first, with a way to sign out any other, or all of them.

use chrono::Utc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::SessionInfo;

use crate::api;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::icons::{Ico, Icon};
use crate::logic::settings::{others, session_name, session_times};

/// `version` changes when the list should be read again, as after a new
/// password signed the other devices out.
#[component]
pub(super) fn SessionsCard(version: RwSignal<u32>) -> impl IntoView {
    let list = RwSignal::new(None::<Vec<SessionInfo>>);
    let error = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);

    let load = move || {
        error.set(None);
        spawn_local(async move {
            match api::call(|c| async move { c.sessions().await }).await {
                Ok(found) => {
                    list.try_set(Some(found));
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    };
    Effect::watch(move || version.get(), move |_, _, _| load(), true);

    let revoke = move |id: i64| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            match api::call(move |c| async move { c.revoke_session(id).await }).await {
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
    };
    let revoke_others = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            match api::call(|c| async move { c.revoke_other_sessions().await }).await {
                Ok(_) => {
                    list.try_update(|l| {
                        if let Some(l) = l {
                            l.retain(|s| s.current);
                        }
                    });
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
            busy.try_set(false);
        });
    };

    let rows = move || {
        let sessions = list.get()?;
        let now = Utc::now();
        let rows = sessions
            .into_iter()
            .map(|s| {
                let id = s.id;
                let here = s.current.then(|| {
                    view! { <span class="pill">{move || t!("sessions-this-device")}</span> }
                });
                let out = (!s.current).then(|| {
                    view! {
                        <button
                            type="button"
                            class="icon-btn danger"
                            title=move || t!("sessions-revoke")
                            aria-label=move || t!("sessions-revoke")
                            disabled=move || busy.get()
                            on:click=move |_| revoke(id)
                        >
                            <Ico icon=Icon::Logout />
                        </button>
                    }
                });
                view! {
                    <li class="session">
                        <div class="session-main">
                            <strong class="break">{session_name(&s)}</strong>
                            {here}
                            <span class="muted">{session_times(&s, now)}</span>
                        </div>
                        {out}
                    </li>
                }
            })
            .collect_view();
        Some(view! { <ul class="sessions">{rows}</ul> })
    };
    let waiting = move || list.with(Option::is_none) && error.with(Option::is_none);
    let any_other = move || list.with(|l| l.as_deref().is_some_and(others));

    view! {
        <section class="card" id="devices">
            <h3 class="card-title">{move || t!("sessions-title")}</h3>
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
            <Show when=any_other>
                <div class="row-end">
                    <button class="btn tonal" type="button" disabled=move || busy.get() on:click=revoke_others>
                        <Ico icon=Icon::Logout />
                        {move || t!("sessions-revoke-others")}
                    </button>
                </div>
            </Show>
        </section>
    }
}
