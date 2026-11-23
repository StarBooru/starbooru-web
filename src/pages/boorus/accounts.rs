//! Accounts on other boorus. They live on the server, which keeps each key
//! sealed and uses it for its site's searches and imports; the browser never
//! holds one past the form it was typed in. What an older app kept on this
//! device moves to the server the first time the browser opens.

use std::cell::RefCell;
use std::collections::HashSet;

use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{BooruAccount, RemoteSite, SaveBooruAccount};

use crate::api;
use crate::components::icons::{Ico, Icon};
use crate::logic::boorus::{
    Site, drop_moved_key, forget_on_device, kept_on_device, key_label, login_label,
    remote_error_text, tidy,
};
use crate::store;

thread_local! {
    // Whose keys from this device were offered to the server in this run.
    static MOVED: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

/// Moves the accounts kept on this device to the server, once for each
/// account signed in, then reads the server's list into `accounts`. A key
/// stays on the device only when the trouble may pass (`drop_moved_key`).
pub(super) fn load_accounts(accounts: RwSignal<Vec<BooruAccount>>) {
    let owner = store::owner();
    let first = MOVED.with_borrow_mut(|moved| moved.insert(owner.clone()));
    spawn_local(async move {
        let moving = if first {
            kept_on_device(&store::prefs_untracked().boorus)
        } else {
            Vec::new()
        };
        for account in moving {
            let base = account.base.clone();
            let answer =
                api::attempt(move |c| async move { c.save_booru_account(&account).await }).await;
            // Signed out, or in as someone else, meanwhile.
            if store::owner() != owner {
                return;
            }
            if drop_moved_key(answer.err().map(|refused| refused.status)) {
                store::update_prefs(|p| forget_on_device(&mut p.boorus, &base));
            }
        }
        if let Ok(list) = api::call(|c| async move { c.booru_accounts().await }).await {
            accounts.try_set(list);
        }
    });
}

/// Forgets the account `id` saved on the server, taking it off `accounts`,
/// or puts why it couldn't in `error`.
pub(super) fn forget_account(
    id: i64,
    accounts: RwSignal<Vec<BooruAccount>>,
    error: RwSignal<Option<String>>,
) {
    spawn_local(async move {
        match api::call(move |c| async move { c.delete_booru_account(id).await }).await {
            Ok(()) => {
                accounts.try_update(|list| list.retain(|a| a.id != id));
            }
            Err(message) => {
                error.try_set(Some(message));
            }
        }
    });
}

/// One site in the sheet: its name, the account saved for it, and the two
/// boxes that take a new one while `open`.
#[component]
pub(super) fn SiteRow(
    site: Site,
    #[prop(into)] account: Signal<Option<BooruAccount>>,
    #[prop(into)] open: Signal<bool>,
    accounts: RwSignal<Vec<BooruAccount>>,
    editing: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
    on_remove: Callback<RemoteSite>,
) -> impl IntoView {
    let api = site.site.api;
    let login = RwSignal::new(String::new());
    let secret = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let save = {
        let site = site.site.clone();
        move |e: SubmitEvent| {
            e.prevent_default();
            if busy.get_untracked() {
                return;
            }
            let typed = SaveBooruAccount {
                api: site.api,
                base: site.base.clone(),
                login: login.get_untracked(),
                api_key: secret.get_untracked(),
            };
            let account = match typed.clean() {
                Ok(account) => account,
                Err(e) => {
                    error.set(Some(untrack(|| remote_error_text(&e))));
                    return;
                }
            };
            busy.set(true);
            error.set(None);
            spawn_local(async move {
                match api::call(move |c| async move { c.save_booru_account(&account).await }).await
                {
                    Ok(kept) => {
                        accounts.try_update(|list| {
                            list.retain(|a| a.id != kept.id && tidy(&a.base) != tidy(&kept.base));
                            list.push(kept);
                        });
                        // The key is the server's from here on.
                        editing.try_set(None);
                        login.try_set(String::new());
                        secret.try_set(String::new());
                    }
                    Err(message) => {
                        error.try_set(Some(message));
                    }
                }
                busy.try_set(false);
            });
        }
    };
    let forget = move |id: i64| forget_account(id, accounts, error);
    let base = site.site.base.clone();
    let edit = move |_| {
        login.set(String::new());
        secret.set(String::new());
        error.set(None);
        let now = (!open.get_untracked()).then(|| base.clone());
        editing.set(now);
    };
    let removed = site.site.clone();
    let name = site.name.clone();
    let remove_label = Signal::derive({
        let name = name.clone();
        move || t!("boorus-site-remove", site = name.clone())
    });
    let kind = site
        .custom
        .then(|| view! { <span class="muted site-kind">{api.name()}</span> });
    let remove = site.custom.then(|| {
        view! {
            <button
                type="button"
                class="icon-btn small danger"
                title=remove_label
                aria-label=remove_label
                on:click=move |_| on_remove.run(removed.clone())
            >
                <Ico icon=Icon::Delete size=20 />
            </button>
        }
    });
    // Every saved account can be forgotten, one that no longer opens too.
    let forget_button = move |id: i64| {
        view! {
            <button type="button" class="btn text small" on:click=move |_| forget(id)>
                {move || t!("boorus-account-forget")}
            </button>
        }
    };
    let account_line = move || match account.get() {
        Some(account) if account.usable => {
            let id = account.id;
            view! {
                <span class="site-login">
                    <Ico icon=Icon::Check size=16 />
                    <span>{account.login}</span>
                </span>
                {forget_button(id)}
            }
            .into_any()
        }
        stale => {
            let id = stale.as_ref().map(|a| a.id);
            view! {
                <span class="site-login">
                    {id.map(|_| {
                        view! {
                            <span class="pill bad" role="status">{move || t!("boorus-account-again")}</span>
                        }
                    })}
                </span>
                <button
                    type="button"
                    class="btn text small"
                    aria-expanded=move || if open.get() { "true" } else { "false" }
                    on:click=edit.clone()
                >
                    {move || t!("boorus-account")}
                </button>
                {id.map(forget_button)}
            }
            .into_any()
        }
    };
    let login_name = Signal::derive(move || login_label(api));
    let key_name = Signal::derive(move || key_label(api));
    view! {
        <li class="site-row">
            <div class="site-line">
                <span class="site-name">{name}</span>
                {kind}
                {account_line}
                {remove}
            </div>
            <Show when=move || open.get()>
                // Another site's account: the browser mustn't fill in the
                // StarBooru sign-in it keeps, which would go to that site.
                <form class="site-account" autocomplete="off" on:submit=save.clone()>
                    <input
                        name="booru-login"
                        placeholder=login_name
                        aria-label=login_name
                        autocomplete="off"
                        spellcheck="false"
                        autocapitalize="none"
                        prop:value=move || login.get()
                        on:input=move |e| login.set(event_target_value(&e))
                    />
                    <input
                        type="password"
                        name="booru-key"
                        placeholder=key_name
                        aria-label=key_name
                        autocomplete="new-password"
                        spellcheck="false"
                        prop:value=move || secret.get()
                        on:input=move |e| secret.set(event_target_value(&e))
                    />
                    <button class="btn" type="submit" disabled=move || busy.get()>
                        {move || t!("save")}
                    </button>
                </form>
            </Show>
        </li>
    }
}
