//! Administration › Backups: database backups made by hand or on a schedule,
//! each to download or delete, the schedule, and how to restore one.

use chrono::Local;
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::admin::{MAX_BACKUP_DAYS, MAX_BACKUP_KEEP};
use starbooru_core::{Backup, BackupSchedule, Backups as List};

use super::Waiting;
use crate::api;
use crate::components::banner::ErrorBanner;
use crate::components::confirm::ConfirmButton;
use crate::components::download::DownloadLink;
use crate::components::icons::{Ico, Icon};
use crate::components::snackbar::notify;
use crate::logic::admin::{RESTORE_COMMAND, backup_line, backup_time, check_schedule};

#[component]
pub(super) fn Backups() -> impl IntoView {
    let data = RwSignal::new(None::<List>);
    let load_error = RwSignal::new(None::<String>);
    let problem = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let making = RwSignal::new(false);
    let load = move || {
        load_error.set(None);
        spawn_local(async move {
            match api::call(|c| async move { c.backups().await }).await {
                Ok(found) => {
                    data.try_set(Some(found));
                }
                Err(message) => {
                    load_error.try_set(Some(message));
                }
            }
        });
    };
    load();
    let retry = Callback::new(move |()| load());
    let back_up = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        making.set(true);
        problem.set(None);
        spawn_local(async move {
            match api::call(|c| async move { c.create_backup().await }).await {
                Ok(made) => {
                    data.try_update(|d| {
                        if let Some(d) = d {
                            d.items.insert(0, made);
                        }
                    });
                    notify(untrack(|| t!("backups-made")));
                }
                Err(message) => {
                    problem.try_set(Some(message));
                }
            }
            busy.try_set(false);
            making.try_set(false);
        });
    };
    let remove = Callback::new(move |name: String| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        problem.set(None);
        spawn_local(async move {
            let asked = name.clone();
            match api::call(move |c| async move { c.delete_backup(&asked).await }).await {
                Ok(()) => {
                    data.try_update(|d| {
                        if let Some(d) = d {
                            d.items.retain(|b| b.name != name);
                        }
                    });
                }
                Err(message) => {
                    problem.try_set(Some(message));
                }
            }
            busy.try_set(false);
        });
    });
    let saved = Callback::new(move |s: BackupSchedule| {
        data.update(|d| {
            if let Some(d) = d {
                d.schedule = s;
            }
        });
        // Saving may have removed old automatic backups.
        load();
    });

    let loaded = move || data.with(Option::is_some);
    let available = move || data.with(|d| d.as_ref().is_some_and(|d| d.available));
    let running = move || making.get() || data.with(|d| d.as_ref().is_some_and(|d| d.running));
    let names = Memo::new(move |_| {
        data.with(|d| {
            d.iter()
                .flat_map(|d| d.items.iter().map(|b| b.name.clone()))
                .collect::<Vec<_>>()
        })
    });
    let waiting = move || {
        load_error
            .with(Option::is_none)
            .then(|| view! { <Waiting /> })
    };
    // Built once from the first answer, so reloading the list doesn't wipe
    // what's typed.
    let ready = Memo::new(move |_| data.with(Option::is_some));
    let schedule = move || {
        if !ready.get() {
            return None;
        }
        data.with_untracked(|d| d.as_ref().map(|d| d.schedule))
            .map(|schedule| view! { <ScheduleCard schedule=schedule on_saved=saved /> })
    };

    view! {
        <section class="card">
            <h2 class="card-title">{move || t!("backups-title")}</h2>
            <p class="muted hint">{move || t!("backups-lead")}</p>
            {move || load_error.get().map(|message| view! { <ErrorBanner message=message on_retry=retry /> })}
            {move || problem.get().map(|message| view! { <ErrorBanner message=message /> })}
            <Show when=loaded fallback=waiting>
                <Show when=move || !available()>
                    <div class="banner info" role="status">
                        {move || t!("backups-unavailable")}
                    </div>
                </Show>
                <Show
                    when=move || !names.with(Vec::is_empty)
                    fallback=|| view! { <p class="muted backups-none">{move || t!("backups-none")}</p> }
                >
                    <ul class="sessions backups">
                        <For
                            each=move || names.get()
                            key=|name| name.clone()
                            children=move |name| {
                                let backup = data
                                    .with_untracked(|d| {
                                        d.iter().flat_map(|d| d.items.iter()).find(|b| b.name == name).cloned()
                                    });
                                backup.map(|b| view! { <BackupRow backup=b busy=busy on_delete=remove /> })
                            }
                        />
                    </ul>
                </Show>
                <div class="row-end">
                    <button
                        class="btn"
                        type="button"
                        disabled=move || busy.get() || running() || !available()
                        on:click=back_up
                    >
                        <Ico icon=Icon::History />
                        {move || if running() { t!("backups-running") } else { t!("backups-now") }}
                    </button>
                </div>
            </Show>
        </section>
        {schedule}
        <section class="card">
            <h2 class="card-title">{move || t!("backups-restore-title")}</h2>
            <p class="muted hint">{move || t!("backups-restore")}</p>
            <pre class="command">
                <code>{RESTORE_COMMAND}</code>
            </pre>
            <p class="muted hint">{move || t!("backups-restore-note")}</p>
        </section>
    }
}

/// One backup: when, how big, its file to download, and a delete that asks
/// first. The link is signed, so it needs no token.
#[component]
fn BackupRow(backup: Backup, busy: RwSignal<bool>, on_delete: Callback<String>) -> impl IntoView {
    let href = api::client().map_or_else(|| backup.url.clone(), |c| c.absolute(&backup.url));
    let name = backup.name.clone();
    let download = {
        let name = name.clone();
        Signal::derive(move || t!("backups-download-named", name = name.clone()))
    };
    let label = {
        let name = name.clone();
        Signal::derive(move || t!("backups-delete-named", name = name.clone()))
    };
    let gone = name.clone();
    view! {
        <li class="session">
            <div class="session-main">
                <strong>{backup_time(&backup, &Local)}</strong>
                {backup.automatic.then(|| view! { <span class="pill">{move || t!("backups-automatic")}</span> })}
                <span class="muted break">{backup_line(&backup)}</span>
            </div>
            <div class="backup-actions">
                <DownloadLink href=href name=name label=download />
                <ConfirmButton
                    icon=Icon::Delete
                    title=Signal::derive(|| t!("backups-delete"))
                    label=label
                    disabled=Signal::derive(move || busy.get())
                    on_confirm=Callback::new(move |()| on_delete.run(gone.clone()))
                />
            </div>
        </li>
    }
}

/// How often backups are made by themselves, and how many are kept.
#[component]
fn ScheduleCard(schedule: BackupSchedule, on_saved: Callback<BackupSchedule>) -> impl IntoView {
    let every = RwSignal::new(schedule.every_days.to_string());
    let keep = RwSignal::new(schedule.keep.to_string());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let saved = RwSignal::new(false);
    let check = Memo::new(move |_| check_schedule(&every.read(), &keep.read()));
    let save = move |e: SubmitEvent| {
        e.prevent_default();
        if busy.get_untracked() {
            return;
        }
        let Some(schedule) = check.get_untracked().schedule else {
            return;
        };
        busy.set(true);
        error.set(None);
        saved.set(false);
        spawn_local(async move {
            match api::call(move |c| async move { c.set_backup_schedule(&schedule).await }).await {
                Ok(s) => {
                    every.try_set(s.every_days.to_string());
                    keep.try_set(s.keep.to_string());
                    saved.try_set(true);
                    on_saved.try_run(s);
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
            <h2 class="card-title">{move || t!("backups-schedule-title")}</h2>
            {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
            <form on:submit=save on:input=move |_| saved.set(false)>
                <div class="schedule-fields">
                    <label class="field">
                        <span>{move || t!("backups-every")}</span>
                        <input
                            type="number"
                            inputmode="numeric"
                            min="0"
                            max=MAX_BACKUP_DAYS
                            aria-invalid=move || invalid(!check.get().every_ok)
                            bind:value=every
                        />
                        <Show when=move || !check.get().every_ok>
                            <small class="field-error">
                                {move || t!("backups-every-error", max = MAX_BACKUP_DAYS)}
                            </small>
                        </Show>
                        <small class="muted field-hint">{move || t!("backups-every-hint")}</small>
                    </label>
                    <label class="field">
                        <span>{move || t!("backups-keep")}</span>
                        <input
                            type="number"
                            inputmode="numeric"
                            min="1"
                            max=MAX_BACKUP_KEEP
                            aria-invalid=move || invalid(!check.get().keep_ok)
                            bind:value=keep
                        />
                        <Show when=move || !check.get().keep_ok>
                            <small class="field-error">
                                {move || t!("backups-keep-error", max = MAX_BACKUP_KEEP)}
                            </small>
                        </Show>
                        <small class="muted field-hint">{move || t!("backups-keep-hint")}</small>
                    </label>
                </div>
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
                        disabled=move || busy.get() || check.with(|c| c.schedule.is_none())
                    >
                        {move || t!("admin-save")}
                    </button>
                </div>
            </form>
        </section>
    }
}
