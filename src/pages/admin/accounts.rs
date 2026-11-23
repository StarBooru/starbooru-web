//! Administration › Accounts: everyone on the server with what they use, and
//! for each account but your own its role, storage limit, a new password,
//! and a way to disable or delete it.

use chrono::{Local, Utc};
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{AdminUser, Role, UserUpdate};

use super::Waiting;
use crate::api;
use crate::components::banner::ErrorBanner;
use crate::components::confirm::ConfirmButton;
use crate::components::icons::{Ico, Icon};
use crate::components::snackbar::notify;
use crate::components::strength::StrengthField;
use crate::logic::admin::{account_parts, quota_change, quota_text};
use crate::logic::settings::{MIN_PASSWORD_CHARS, role_label};
use crate::store;

type List = RwSignal<Option<Vec<AdminUser>>>;

#[component]
pub(super) fn Accounts() -> impl IntoView {
    let list: List = RwSignal::new(None);
    let error = RwSignal::new(None::<String>);
    let open = RwSignal::new(None::<i64>);
    let load = move || {
        error.set(None);
        spawn_local(async move {
            match api::call(|c| async move { c.admin_users().await }).await {
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
    let retry = Callback::new(move |()| load());
    let ids =
        Memo::new(move |_| list.with(|l| l.iter().flatten().map(|u| u.id).collect::<Vec<_>>()));
    let me = store::session_untracked().map(|s| s.user.id);
    let waiting = move || error.with(Option::is_none).then(|| view! { <Waiting /> });

    view! {
        <section class="card">
            <h2 class="card-title">{move || t!("admin-accounts")}</h2>
            <p class="muted hint">{move || t!("admin-accounts-lead")}</p>
            {move || error.get().map(|message| view! { <ErrorBanner message=message on_retry=retry /> })}
            <Show when=move || list.with(Option::is_some) fallback=waiting>
                <ul class="sessions accounts">
                    <For
                        each=move || ids.get()
                        key=|id| *id
                        children=move |id| {
                            view! { <AccountRow id=id list=list open=open mine=me == Some(id) /> }
                        }
                    />
                </ul>
            </Show>
        </section>
    }
}

/// One account. Its row follows the list, so a change made in its tools shows
/// at once while the tools keep what's typed in them.
#[component]
fn AccountRow(id: i64, list: List, open: RwSignal<Option<i64>>, mine: bool) -> impl IntoView {
    let user = Memo::new(move |_| list.with(|l| l.iter().flatten().find(|u| u.id == id).cloned()));
    let has = move |test: fn(&AdminUser) -> bool| user.with(|u| u.as_ref().is_some_and(test));
    let name = Signal::derive(move || {
        user.with(|u| u.as_ref().map(|u| u.username.clone()))
            .unwrap_or_default()
    });
    let expanded = move || open.get() == Some(id);
    let line = move || {
        let parts = user.with(|u| u.as_ref().map(|u| account_parts(u, Utc::now(), &Local)))?;
        Some(
            parts
                .into_iter()
                .enumerate()
                .map(|(i, part)| {
                    view! {
                        {(i > 0).then_some(" · ")}
                        <span>{part}</span>
                    }
                })
                .collect_view(),
        )
    };
    let manage = (!mine).then(|| {
        view! {
            <button
                type="button"
                class="icon-btn"
                class:on=expanded
                title=move || t!("admin-manage")
                aria-label=move || t!("admin-manage-named", name = name.get())
                aria-expanded=move || if expanded() { "true" } else { "false" }
                on:click=move |_| open.set(if expanded() { None } else { Some(id) })
            >
                <Ico icon=Icon::Tune />
            </button>
            <Show when=expanded>
                <AccountTools id=id user=user name=name list=list open=open />
            </Show>
        }
    });
    view! {
        <li class="session account">
            <div class="session-main">
                <strong class="break">{name}</strong>
                <Show when=move || has(|u| u.role == Role::Admin)>
                    <span class="pill">{move || role_label(Role::Admin)}</span>
                </Show>
                <Show when=move || has(|u| u.disabled)>
                    <span class="pill bad">{move || t!("admin-disabled")}</span>
                </Show>
                {mine.then(|| view! { <span class="pill">{move || t!("admin-you")}</span> })}
                <span class="muted account-line">{line}</span>
            </div>
            {manage}
        </li>
    }
}

/// Role, storage limit, password reset, disable and delete for one account.
#[component]
fn AccountTools(
    id: i64,
    user: Memo<Option<AdminUser>>,
    name: Signal<String>,
    list: List,
    open: RwSignal<Option<i64>>,
) -> impl IntoView {
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    // What was done, kept as a function so it follows the language.
    let done = RwSignal::new(None::<fn() -> String>);
    let quota = RwSignal::new(
        user.with_untracked(|u| u.as_ref().and_then(|u| u.quota_bytes))
            .map(quota_text)
            .unwrap_or_default(),
    );
    let password = RwSignal::new(String::new());
    let role = Memo::new(move |_| user.with(|u| u.as_ref().map_or(Role::User, |u| u.role)));
    let disabled = Memo::new(move |_| user.with(|u| u.as_ref().is_some_and(|u| u.disabled)));
    // The limit to send: only one that differs from the account's.
    let limit = Memo::new(move |_| {
        let saved = user.with(|u| u.as_ref().and_then(|u| u.quota_bytes));
        quota.with(|q| quota_change(q, saved))
    });
    let quota_ok = Memo::new(move |_| limit.with(Result::is_ok));
    let quota_new = Memo::new(move |_| limit.with(|l| matches!(l, Ok(Some(_)))));
    let password_ok =
        Memo::new(move |_| password.with(|p| p.chars().count() >= MIN_PASSWORD_CHARS));

    let update = move |change: UserUpdate, message: fn() -> String| {
        if busy.get_untracked() {
            return;
        }
        let new_limit = change.quota_bytes.is_some();
        busy.set(true);
        error.set(None);
        done.set(None);
        spawn_local(async move {
            match api::call(move |c| async move { c.update_user(id, &change).await }).await {
                Ok(changed) => {
                    if new_limit {
                        quota.try_set(changed.quota_bytes.map(quota_text).unwrap_or_default());
                    }
                    list.try_update(|l| {
                        if let Some(slot) = l.iter_mut().flatten().find(|u| u.id == changed.id) {
                            *slot = changed;
                        }
                    });
                    done.try_set(Some(message));
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
            busy.try_set(false);
        });
    };
    let save_quota = move |e: SubmitEvent| {
        e.prevent_default();
        if let Ok(Some(new)) = limit.get_untracked() {
            let change = UserUpdate {
                quota_bytes: Some(new),
                ..UserUpdate::default()
            };
            update(change, || t!("admin-limit-saved"));
        }
    };
    let reset = move |e: SubmitEvent| {
        e.prevent_default();
        if busy.get_untracked() || !password_ok.get_untracked() {
            return;
        }
        let new = password.get_untracked();
        busy.set(true);
        error.set(None);
        done.set(None);
        spawn_local(async move {
            match api::call(move |c| async move { c.reset_user_password(id, &new).await }).await {
                Ok(()) => {
                    password.try_set(String::new());
                    done.try_set(Some(|| t!("admin-password-reset-done")));
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
            busy.try_set(false);
        });
    };
    let switch_off = move |_| {
        let on = disabled.get_untracked();
        let change = UserUpdate {
            disabled: Some(!on),
            ..UserUpdate::default()
        };
        if on {
            update(change, || t!("admin-enabled-done"));
        } else {
            update(change, || t!("admin-disabled-done"));
        }
    };
    let remove = Callback::new(move |()| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        done.set(None);
        let gone = name.get_untracked();
        spawn_local(async move {
            match api::call(move |c| async move { c.delete_user(id).await }).await {
                Ok(()) => {
                    notify(untrack(|| t!("admin-deleted", name = gone)));
                    open.try_set(None);
                    list.try_update(|l| {
                        if let Some(l) = l {
                            l.retain(|u| u.id != id);
                        }
                    });
                }
                Err(message) => {
                    error.try_set(Some(message));
                    busy.try_set(false);
                }
            }
        });
    });

    let roles = [Role::User, Role::Admin]
        .into_iter()
        .map(|r| {
            let on = move || role.get() == r;
            view! {
                <button
                    type="button"
                    role="radio"
                    class="seg"
                    class:on=on
                    aria-checked=move || if on() { "true" } else { "false" }
                    disabled=move || busy.get()
                    on:click=move |_| {
                        if role.get_untracked() != r {
                            let change = UserUpdate {
                                role: Some(r),
                                ..UserUpdate::default()
                            };
                            update(change, || t!("admin-role-saved"));
                        }
                    }
                >
                    {move || role_label(r)}
                </button>
            }
        })
        .collect_view();
    let invalid = |bad: bool| if bad { "true" } else { "false" };
    let quota_id = format!("quota-{id}");
    let password_id = format!("password-{id}");

    view! {
        <div class="account-tools">
            {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
            {move || {
                done.get()
                    .map(|text| {
                        view! {
                            <div class="banner info" role="status">
                                {text}
                            </div>
                        }
                    })
            }}
            <div class="field">
                <span>{move || t!("settings-role")}</span>
                <div class="segmented" role="radiogroup" aria-label=move || t!("settings-role")>
                    {roles}
                </div>
            </div>
            <form class="field" on:submit=save_quota>
                <label for=quota_id.clone()>{move || t!("admin-limit")}</label>
                <div class="inline-input">
                    <input
                        id=quota_id
                        inputmode="decimal"
                        autocomplete="off"
                        placeholder=move || t!("admin-limit-none")
                        aria-invalid=move || invalid(!quota_ok.get())
                        bind:value=quota
                    />
                    <button class="btn tonal" type="submit" disabled=move || busy.get() || !quota_new.get()>
                        {move || t!("admin-limit-save")}
                    </button>
                </div>
                <Show when=move || !quota_ok.get()>
                    <small class="field-error">{move || t!("admin-limit-invalid")}</small>
                </Show>
                <small class="muted field-hint">{move || t!("admin-limit-hint")}</small>
            </form>
            <form on:submit=reset>
                // Whose password this is, for a password manager. Outside
                // the field, whose inputs are full width.
                <input
                    class="visually-hidden"
                    type="text"
                    autocomplete="username"
                    tabindex="-1"
                    aria-hidden="true"
                    readonly=true
                    prop:value=move || name.get()
                />
                <div class="field">
                <label for=password_id.clone()>{move || t!("admin-password-new")}</label>
                <div class="inline-input">
                    <StrengthField value=password username=name>
                        <input
                            id=password_id
                            type="password"
                            autocomplete="new-password"
                            aria-invalid=move || {
                                invalid(password.with(|p| !p.is_empty()) && !password_ok.get())
                            }
                            bind:value=password
                        />
                    </StrengthField>
                    <button class="btn tonal" type="submit" disabled=move || busy.get() || !password_ok.get()>
                        {move || t!("admin-password-reset")}
                    </button>
                </div>
                <Show when=move || password.with(|p| !p.is_empty()) && !password_ok.get()>
                    <small class="field-error">
                        {move || t!("password-too-short", count = MIN_PASSWORD_CHARS)}
                    </small>
                </Show>
                <small class="muted field-hint">{move || t!("admin-password-hint")}</small>
                </div>
            </form>
            <div class="row-end">
                <button class="btn tonal small" type="button" disabled=move || busy.get() on:click=switch_off>
                    {move || if disabled.get() { t!("admin-enable") } else { t!("admin-disable") }}
                </button>
                <ConfirmButton
                    icon=Icon::Delete
                    title=Signal::derive(|| t!("admin-delete"))
                    label=Signal::derive(move || t!("admin-delete-named", name = name.get()))
                    text=Signal::derive(|| t!("admin-delete"))
                    sure=Signal::derive(move || t!("admin-delete-confirm", name = name.get()))
                    small=true
                    disabled=busy
                    on_confirm=remove
                />
            </div>
            <p class="muted field-hint">{move || t!("admin-delete-note")}</p>
        </div>
    }
}
