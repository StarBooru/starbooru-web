//! Administration › Server: the mode and the score kind (`modes.rs`), the
//! settings that override the server's environment (its name, sign-ups, how
//! long the recycle bin keeps posts, automatic meta tags), emptying the bin
//! at once, and the AI tools (`ai.rs`). The settings are read once for the
//! cards that show them.

use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::ServerSettings;
use starbooru_core::admin::{MAX_INSTANCE_NAME_CHARS, MAX_TRASH_DAYS};

use super::Waiting;
use super::ai::AiCard;
use super::modes::{ModeCard, ScoreCard};
use crate::api;
use crate::components::banner::ErrorBanner;
use crate::components::confirm::ConfirmButton;
use crate::components::icons::{Ico, Icon};
use crate::logic::admin::check_settings;
use crate::logic::viewer::human_size;
use crate::pages::trash::empty_whole_bin;
use crate::route;

/// The tab: the settings, read once, for the cards that show them.
#[component]
pub(super) fn ServerTab() -> impl IntoView {
    let settings = RwSignal::new(None::<ServerSettings>);
    let load_error = RwSignal::new(None::<String>);
    let load = move || {
        load_error.set(None);
        spawn_local(async move {
            match api::call(|c| async move { c.admin_settings().await }).await {
                Ok(s) => {
                    settings.try_set(Some(s));
                }
                Err(message) => {
                    load_error.try_set(Some(message));
                }
            }
        });
    };
    load();
    let retry = Callback::new(move |()| load());
    view! {
        {move || load_error.get().map(|message| view! { <ErrorBanner message=message on_retry=retry /> })}
        <Show when=move || load_error.with(Option::is_none)>
            <ModeCard settings=settings on_reload=retry />
            <ScoreCard settings=settings />
            <ServerCard settings=settings />
        </Show>
        <BinCard />
        <AiCard />
    }
}

#[component]
fn ServerCard(settings: RwSignal<Option<ServerSettings>>) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let open = RwSignal::new(false);
    let days = RwSignal::new(String::new());
    let meta_tags = RwSignal::new(true);
    let loaded = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let saved = RwSignal::new(false);
    let fill = move |s: &ServerSettings| {
        name.try_set(s.name.clone());
        open.try_set(s.allow_registration);
        days.try_set(s.trash_days.to_string());
        meta_tags.try_set(s.auto_meta_tags);
        loaded.try_set(true);
    };
    // The form takes the settings as they first come; a switch of mode or
    // score kind later leaves what's typed alone.
    Effect::new(move |_| {
        if !loaded.get_untracked()
            && let Some(s) = settings.get()
        {
            fill(&s);
        }
    });
    // Sent back as they are: each has a way of its own to change.
    let kept = Memo::new(move |_| {
        settings.with(|s| {
            s.as_ref()
                .map(|s| (s.mode, s.score_kind))
                .unwrap_or_default()
        })
    });
    let check = Memo::new(move |_| {
        check_settings(
            &name.read(),
            open.get(),
            &days.read(),
            meta_tags.get(),
            kept.get(),
        )
    });
    let save = move |e: SubmitEvent| {
        e.prevent_default();
        if busy.get_untracked() {
            return;
        }
        let Some(request) = check.get_untracked().request else {
            return;
        };
        busy.set(true);
        error.set(None);
        saved.set(false);
        spawn_local(async move {
            match api::call(move |c| async move { c.set_admin_settings(&request).await }).await {
                Ok(s) => {
                    fill(&s);
                    settings.try_set(Some(s));
                    saved.try_set(true);
                    api::forget_server_answers();
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
            busy.try_set(false);
        });
    };
    let invalid = |bad: bool| if bad { "true" } else { "false" };

    view! {
        <section class="card">
            <h2 class="card-title">{move || t!("admin-tab-server")}</h2>
            <p class="muted hint">{move || t!("admin-server-lead")}</p>
            <Show when=move || loaded.get() fallback=|| view! { <Waiting /> }>
                <form on:submit=save on:input=move |_| saved.set(false)>
                    <label class="field">
                        <span>{move || t!("admin-name")}</span>
                        <input
                            maxlength=MAX_INSTANCE_NAME_CHARS
                            autocomplete="off"
                            aria-invalid=move || invalid(!check.get().name_ok)
                            bind:value=name
                        />
                        <Show when=move || !check.get().name_ok>
                            <small class="field-error">
                                {move || t!("admin-name-error", max = MAX_INSTANCE_NAME_CHARS)}
                            </small>
                        </Show>
                        <small class="muted field-hint">{move || t!("admin-name-hint")}</small>
                    </label>
                    <div class="field">
                        <label class="check">
                            <input type="checkbox" bind:checked=open />
                            <span>{move || t!("admin-registration")}</span>
                        </label>
                        <small class="muted field-hint">{move || t!("admin-registration-hint")}</small>
                    </div>
                    <label class="field">
                        <span>{move || t!("admin-trash-days")}</span>
                        <input
                            type="number"
                            inputmode="numeric"
                            min="0"
                            max=MAX_TRASH_DAYS
                            aria-invalid=move || invalid(!check.get().days_ok)
                            bind:value=days
                        />
                        <Show when=move || !check.get().days_ok>
                            <small class="field-error">
                                {move || t!("admin-trash-days-error", max = MAX_TRASH_DAYS)}
                            </small>
                        </Show>
                        <small class="muted field-hint">{move || t!("admin-trash-days-hint")}</small>
                    </label>
                    <div class="field">
                        <label class="check">
                            <input type="checkbox" bind:checked=meta_tags />
                            <span>{move || t!("admin-auto-tags")}</span>
                        </label>
                        <small class="muted field-hint">{move || t!("admin-auto-tags-hint")}</small>
                    </div>
                    {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
                    <div class="row-end">
                        <Show when=move || saved.get()>
                            <span class="muted saved-note" role="status">
                                <Ico icon=Icon::Check size=18 />
                                {move || t!("admin-saved")}
                            </span>
                        </Show>
                        <button
                            class="btn"
                            type="submit"
                            disabled=move || busy.get() || check.with(|c| c.request.is_none())
                        >
                            {move || t!("admin-save")}
                        </button>
                    </div>
                </form>
            </Show>
        </section>
    }
}

/// What the recycle bin holds, a way to it, and emptying it at once, which
/// asks first and says how many posts go.
#[component]
fn BinCard() -> impl IntoView {
    let stats = LocalResource::new(|| api::call(|c| async move { c.admin_stats().await }));
    let busy = RwSignal::new(false);
    let notice = RwSignal::new(None::<String>);
    let problem = RwSignal::new(None::<String>);
    let held = Memo::new(move |_| {
        stats
            .get()
            .and_then(Result::ok)
            .map(|s| (s.bin_posts, s.bin_bytes))
    });
    let count = move || held.get().map_or(0, |(posts, _)| posts);
    let empty = Callback::new(move |()| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        notice.set(None);
        problem.set(None);
        spawn_local(async move {
            let outcome = empty_whole_bin().await;
            if busy.try_set(false).is_some() {
                return;
            }
            match outcome {
                Ok(message) => notice.set(Some(message)),
                Err(message) => problem.set(Some(message)),
            }
            stats.refetch();
        });
    });
    let summary = move || match stats.get() {
        None => String::new(),
        Some(Err(_)) => String::new(),
        Some(Ok(s)) if s.bin_posts == 0 => t!("trash-none"),
        Some(Ok(s)) => format!(
            "{} · {}",
            t!("posts-count", count = s.bin_posts),
            human_size(s.bin_bytes)
        ),
    };
    let load_error = move || match stats.get() {
        Some(Err(message)) => Some(view! {
            <ErrorBanner message=message on_retry=Callback::new(move |()| stats.refetch()) />
        }),
        _ => None,
    };

    view! {
        <section class="card">
            <h2 class="card-title">{move || t!("trash-title")}</h2>
            <p class="muted hint">{summary}</p>
            {load_error}
            {move || {
                notice
                    .get()
                    .map(|message| {
                        view! {
                            <div class="banner info" role="status">
                                <span class="break">{message}</span>
                            </div>
                        }
                    })
            }}
            {move || problem.get().map(|message| view! { <ErrorBanner message=message /> })}
            <div class="row-end">
                <a class="btn tonal" href=route::TRASH>
                    <Ico icon=Icon::Delete />
                    {move || t!("admin-bin-open")}
                </a>
                <ConfirmButton
                    icon=Icon::DeleteForever
                    title=Signal::derive(|| t!("trash-empty"))
                    label=Signal::derive(|| t!("trash-empty"))
                    text=Signal::derive(|| t!("trash-empty"))
                    sure=Signal::derive(move || t!("admin-bin-empty-confirm", count = count()))
                    disabled=Signal::derive(move || busy.get() || count() == 0)
                    on_confirm=empty
                />
            </div>
        </section>
    }
}
