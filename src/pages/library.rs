//! Settings › Import from another booru: moves a whole library over. The
//! server does the work, so it goes on when the tab is closed; this card
//! starts, follows and mends those imports.

use chrono::Utc;
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{
    LibraryCheck, LibraryFailure, LibraryImport, LibraryRetry, LibrarySite, LibrarySource, Rating,
    clean_credentials,
};

use super::rating_label;
use crate::api;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::icons::{Ico, Icon};
use crate::components::select::{Choice, Select};
use crate::dom;
use crate::logic::library::{
    IDLE_ROUNDS, POLL_MS, ago, counts, form_error, full, given, moving, source_hint, state_class,
    state_label,
};

#[component]
pub fn LibraryImportCard() -> impl IntoView {
    let list = RwSignal::new(None::<Vec<LibraryImport>>);
    let error = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let checking = RwSignal::new(false);
    let source = RwSignal::new(LibrarySource::Szurubooru);
    let base = RwSignal::new(String::new());
    let login = RwSignal::new(String::new());
    let secret = RwSignal::new(String::new());
    let unrated = RwSignal::new(Rating::Questionable);
    let checked = RwSignal::new(None::<LibraryCheck>);
    // Why the list couldn't be read the first time.
    let load_error = RwSignal::new(None::<String>);
    // Bumped by every change made here, so a reading sent before one and
    // answered after it is dropped rather than undo it.
    let generation = StoredValue::new(0_u64);

    // Goes on while the card is on the page, reading often only while
    // something moves. Stops when the first reading fails; Retry starts it
    // again.
    let follow = move || {
        load_error.set(None);
        spawn_local(async move {
            loop {
                let Some(at) = generation.try_get_value() else {
                    return;
                };
                match api::call(|c| async move { c.library_imports().await }).await {
                    Ok(found) => {
                        if generation.try_get_value() == Some(at)
                            && list.try_set(Some(found)).is_some()
                        {
                            return;
                        }
                    }
                    Err(message) => {
                        // Later on, what's shown stays until a reading works.
                        if list.try_with_untracked(Option::is_none) == Some(true) {
                            load_error.try_set(Some(message));
                            return;
                        }
                    }
                }
                for _ in 0..IDLE_ROUNDS {
                    dom::sleep(POLL_MS).await;
                    let Some(now) = list.try_with_untracked(|l| l.as_deref().is_some_and(moving))
                    else {
                        return;
                    };
                    if now {
                        break;
                    }
                }
            }
        });
    };
    follow();
    let retry_list = Callback::new(move |()| follow());

    let form = move || -> Result<LibrarySite, String> {
        let chosen = source.get_untracked();
        let site = LibrarySite {
            source: chosen,
            base: base.get_untracked(),
            login: given(&login.get_untracked()).filter(|_| chosen.takes_login()),
            secret: given(&secret.get_untracked()).filter(|_| chosen.takes_secret()),
            unrated: unrated.get_untracked(),
        };
        // Read in a click, where the language isn't followed.
        site.clean().map_err(|e| untrack(|| form_error(&e)))
    };
    let check = move |_| {
        let site = match form() {
            Ok(site) => site,
            Err(e) => return error.set(Some(e)),
        };
        busy.set(true);
        checking.set(true);
        error.set(None);
        checked.set(None);
        spawn_local(async move {
            match api::call(move |c| async move { c.check_library(&site).await }).await {
                Ok(found) => {
                    checked.try_set(Some(found));
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
            busy.try_set(false);
            checking.try_set(false);
        });
    };
    let start = move |e: SubmitEvent| {
        e.prevent_default();
        let site = match form() {
            Ok(site) => site,
            Err(e) => return error.set(Some(e)),
        };
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            match api::call(move |c| async move { c.start_library_import(&site).await }).await {
                Ok(import) => {
                    generation.try_update_value(|g| *g = g.wrapping_add(1));
                    list.try_update(|l| l.get_or_insert_default().insert(0, import));
                    base.try_set(String::new());
                    login.try_set(String::new());
                    secret.try_set(String::new());
                    checked.try_set(None);
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
            busy.try_set(false);
        });
    };

    let sources = Signal::derive(|| {
        LibrarySource::ALL
            .into_iter()
            .map(|s| Choice::new(s.as_str(), s.name()))
            .collect::<Vec<_>>()
    });
    let ratings = Signal::derive(|| {
        Rating::ALL
            .into_iter()
            .map(|r| Choice::new(r.as_str(), rating_label(r)))
            .collect::<Vec<_>>()
    });
    let is_full = Memo::new(move |_| list.with(|l| l.as_deref().is_some_and(full)));
    let ready = move || !busy.get() && base.with(|b| !b.trim().is_empty()) && !is_full.get();
    let found = move || {
        checked.get().map(|found| {
            let total = match found.total {
                Some(total) => t!("library-found", count = total),
                None => t!("library-found-some"),
            };
            let rules = (!found.rules).then(|| format!(" {}", t!("library-no-rules")));
            view! {
                <div class="banner info" role="status">
                    <span>{total}{rules}</span>
                </div>
            }
        })
    };
    // Each row stays as the list is read again, with what is typed in it.
    let ids = Memo::new(move |_| {
        list.with(|l| {
            l.as_ref()
                .map(|l| l.iter().map(|i| (i.id, i.source)).collect::<Vec<_>>())
        })
    });
    let imports = move || match ids.get() {
        None => match load_error.get() {
            Some(message) => {
                view! { <ErrorBanner message=message on_retry=retry_list /> }.into_any()
            }
            None => view! { <Spinner /> }.into_any(),
        },
        Some(ids) if ids.is_empty() => ().into_any(),
        Some(_) => view! {
            <h3 class="library-heading">{move || t!("library-list")}</h3>
            <ul class="sessions library-list">
                <For
                    each=move || ids.get().unwrap_or_default()
                    key=|(id, _)| *id
                    children=move |(id, source)| {
                        view! { <ImportRow id=id source=source list=list generation=generation /> }
                    }
                />
            </ul>
        }
        .into_any(),
    };

    view! {
        <section class="card library">
            <h2 class="card-title">{move || t!("library-title")}</h2>
            <p class="muted hint">{move || t!("library-lead")}</p>
            <p class="muted hint">{move || t!("library-left-out")}</p>
            <form class="key-form library-form" on:submit=start>
                // Not a `label`: a click on the list would press the button in it again.
                <div class="field">
                    <span>{move || t!("library-source")}</span>
                    <Select
                        class="boxed"
                        value=Signal::derive(move || source.get().as_str().to_owned())
                        options=sources
                        on_change=Callback::new(move |value: String| {
                            if let Some(picked) = LibrarySource::parse(&value) {
                                source.set(picked);
                                checked.set(None);
                                error.set(None);
                            }
                        })
                        label=Signal::derive(|| t!("library-source"))
                    />
                </div>
                <p class="muted hint">{move || source_hint(source.get())}</p>
                <label class="field">
                    <span>{move || t!("boorus-address")}</span>
                    <input
                        type="url"
                        inputmode="url"
                        autocapitalize="none"
                        autocomplete="off"
                        spellcheck="false"
                        placeholder="http://192.168.1.4:8080"
                        prop:value=move || base.get()
                        on:input=move |e| {
                            base.set(event_target_value(&e));
                            checked.set(None);
                        }
                    />
                </label>
                // Another site's account: the browser mustn't fill in the
                // StarBooru sign-in it keeps.
                <Show when=move || source.get().takes_login()>
                    <label class="field">
                        <span>{move || t!("boorus-login")}</span>
                        <input
                            name="library-login"
                            autocomplete="off"
                            autocapitalize="none"
                            spellcheck="false"
                            prop:value=move || login.get()
                            on:input=move |e| login.set(event_target_value(&e))
                        />
                    </label>
                </Show>
                <Show when=move || source.get().takes_secret()>
                    <label class="field">
                        <span>{move || t!("boorus-key")}</span>
                        <input
                            type="password"
                            name="library-key"
                            autocomplete="new-password"
                            prop:value=move || secret.get()
                            on:input=move |e| secret.set(event_target_value(&e))
                        />
                    </label>
                </Show>
                <div class="field">
                    <span>{move || t!("library-unrated")}</span>
                    <Select
                        class="boxed"
                        value=Signal::derive(move || unrated.get().as_str().to_owned())
                        options=ratings
                        on_change=Callback::new(move |value: String| {
                            if let Some(picked) = Rating::parse(&value) {
                                unrated.set(picked);
                            }
                        })
                        label=Signal::derive(|| t!("library-unrated"))
                    />
                </div>
                {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
                {found}
                <Show when=move || is_full.get()>
                    <p class="muted hint">{move || t!("library-full")}</p>
                </Show>
                <div class="row-end">
                    <button class="btn tonal" type="button" disabled=move || !ready() on:click=check>
                        {move || if checking.get() { t!("library-checking") } else { t!("library-check") }}
                    </button>
                    <button class="btn" type="submit" disabled=move || !ready()>
                        <Ico icon=Icon::Download />
                        {move || t!("library-start")}
                    </button>
                </div>
            </form>
            {imports}
        </section>
    }
}

/// One import. `generation` is bumped with each change made here, as the
/// card's reading of the list wants.
#[component]
fn ImportRow(
    id: i64,
    source: LibrarySource,
    list: RwSignal<Option<Vec<LibraryImport>>>,
    generation: StoredValue<u64>,
) -> impl IntoView {
    let error = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let asking = RwSignal::new(false);
    let login = RwSignal::new(String::new());
    let secret = RwSignal::new(String::new());
    let failures = RwSignal::new(None::<Vec<LibraryFailure>>);
    let import =
        Memo::new(move |_| list.with(|l| l.iter().flatten().find(|i| i.id == id).cloned()));
    let active =
        Memo::new(move |_| import.with(|i| i.as_ref().is_some_and(|i| i.state.is_active())));

    let replace = move |updated: LibraryImport| {
        generation.try_update_value(|g| *g = g.wrapping_add(1));
        list.try_update(|l| {
            if let Some(found) = l.iter_mut().flatten().find(|i| i.id == updated.id) {
                *found = updated;
            }
        });
    };
    let stop = move |_| {
        busy.set(true);
        spawn_local(async move {
            match api::call(move |c| async move { c.stop_library_import(id).await }).await {
                Ok(updated) => replace(updated),
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
            busy.try_set(false);
        });
    };
    let retry = move || {
        let account = LibraryRetry {
            login: given(&login.get_untracked()).filter(|_| source.takes_login()),
            secret: given(&secret.get_untracked()).filter(|_| source.takes_secret()),
        };
        if let Err(e) = clean_credentials(source, account.login.clone(), account.secret.clone()) {
            return error.set(Some(untrack(|| form_error(&e))));
        }
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            match api::call(move |c| async move { c.retry_library_import(id, &account).await })
                .await
            {
                Ok(updated) => {
                    replace(updated);
                    asking.try_set(false);
                    login.try_set(String::new());
                    secret.try_set(String::new());
                    failures.try_set(None);
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
            busy.try_set(false);
        });
    };
    let remove = move |_| {
        busy.set(true);
        spawn_local(async move {
            match api::call(move |c| async move { c.delete_library_import(id).await }).await {
                Ok(()) => {
                    generation.try_update_value(|g| *g = g.wrapping_add(1));
                    list.try_update(|l| {
                        if let Some(all) = l.as_mut() {
                            all.retain(|i| i.id != id);
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
    let show_failures = move |_| {
        if failures.with_untracked(Option::is_some) {
            return failures.set(None);
        }
        spawn_local(async move {
            match api::call(move |c| async move { c.library_failures(id).await }).await {
                Ok(found) => {
                    failures.try_set(Some(found));
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    };

    // What the server says of it, read again with the list.
    let facts = move || {
        let import = import.get()?;
        let label = state_label(&import);
        let when = (!import.state.is_active()).then(|| {
            let at = ago(import.finished_at.unwrap_or(import.created_at), Utc::now());
            view! { <span class="muted">{at}</span> }
        });
        let meter = import.state.is_active().then(|| match import.fraction() {
            Some(f) => view! {
                <span
                    class="meter library-meter"
                    role="meter"
                    aria-valuemin="0"
                    aria-valuemax="100"
                    aria-valuenow=format!("{:.0}", f * 100.0)
                    aria-label=label.clone()
                >
                    <span style=format!("width: {:.1}%", f * 100.0)></span>
                </span>
            }
            .into_any(),
            None => view! {
                <span class="meter library-meter unknown" role="progressbar" aria-label=label.clone()>
                    <span></span>
                </span>
            }
            .into_any(),
        });
        let reason = import
            .error
            .clone()
            .map(|reason| view! { <p class="library-reason">{reason}</p> });
        let note = import
            .note
            .clone()
            .map(|note| view! { <p class="muted hint">{note}</p> });
        Some(view! {
            <div class="library-import-head">
                <strong>{source.name()}</strong>
                <span class="muted break">{import.base.clone()}</span>
            </div>
            <div class="library-import-state">
                <span class=state_class(import.state)>{label.clone()}</span>
                {when}
            </div>
            {meter}
            <span class="muted">{counts(&import)}</span>
            {reason}
            {note}
        })
    };
    let takes_account = source.takes_login() || source.takes_secret();
    let shows_failed = move || import.with(|i| i.as_ref().is_some_and(|i| i.failed > 0));
    view! {
        <li class="library-import">
            {facts}
            {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
            <Show when=move || asking.get()>
                <form
                    class="library-account"
                    on:submit=move |e: SubmitEvent| {
                        e.prevent_default();
                        retry();
                    }
                >
                    <p class="muted hint">{move || t!("library-account-again")}</p>
                    {source.takes_login().then(|| view! {
                        <label class="field">
                            <span>{move || t!("boorus-login")}</span>
                            <input
                                name="library-login"
                                autocomplete="off"
                                autocapitalize="none"
                                spellcheck="false"
                                prop:value=move || login.get()
                                on:input=move |e| login.set(event_target_value(&e))
                            />
                        </label>
                    })}
                    {source.takes_secret().then(|| view! {
                        <label class="field">
                            <span>{move || t!("boorus-key")}</span>
                            <input
                                type="password"
                                name="library-key"
                                autocomplete="new-password"
                                prop:value=move || secret.get()
                                on:input=move |e| secret.set(event_target_value(&e))
                            />
                        </label>
                    })}
                    <div class="row-end">
                        <button class="btn tonal small" type="button" on:click=move |_| asking.set(false)>
                            {move || t!("cancel")}
                        </button>
                        <button class="btn small" type="submit" disabled=move || busy.get()>
                            {move || t!("library-retry")}
                        </button>
                    </div>
                </form>
            </Show>
            <div class="row-end library-actions">
                <Show when=shows_failed>
                    <button
                        class="btn tonal small"
                        type="button"
                        aria-expanded=move || if failures.with(Option::is_some) { "true" } else { "false" }
                        on:click=show_failures
                    >
                        {move || t!("library-show-failed")}
                    </button>
                </Show>
                <Show
                    when=move || active.get()
                    fallback=move || view! {
                        <button class="btn tonal small" type="button" disabled=move || busy.get() on:click=remove>
                            {move || t!("library-remove")}
                        </button>
                        <Show when=move || !asking.get()>
                            <button
                                class="btn tonal small"
                                type="button"
                                disabled=move || busy.get()
                                on:click=move |_| {
                                    if takes_account {
                                        asking.set(true);
                                    } else {
                                        retry();
                                    }
                                }
                            >
                                {move || t!("library-retry")}
                            </button>
                        </Show>
                    }
                >
                    <button class="btn tonal small" type="button" disabled=move || busy.get() on:click=stop>
                        {move || t!("library-stop")}
                    </button>
                </Show>
            </div>
            {move || {
                failures
                    .get()
                    .map(|found| {
                        let rows = found
                            .into_iter()
                            .map(|f| {
                                view! {
                                    <li>
                                        <code>{f.remote_id}</code>
                                        " "
                                        <span class="muted">{f.error}</span>
                                    </li>
                                }
                            })
                            .collect_view();
                        view! { <ul class="library-failures">{rows}</ul> }
                    })
            }}
        </li>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_card_follows_the_reference() {
        let source = include_str!("library.rs");
        assert!(source.contains(r#"class="key-form library-form""#));
        assert!(source.contains(r#"class="sessions library-list""#));
        // The key is a password box, and isn't kept once sent.
        assert!(source.contains(r#"type="password""#));
        assert!(source.contains("secret.try_set(String::new())"));
    }
}
