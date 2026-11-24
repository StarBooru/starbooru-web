//! Administration › Updates: the version running and the newest, with its
//! notes; Check now; updating from here, or by hand where the server can't;
//! an update under way, through the server's restart to a reload on the new
//! version; and the checks and automatic updates.

use chrono::{DateTime, Local, Offset, Utc};
use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{UpdateInstall, UpdateKind, UpdateSettings, UpdateState, UpdateStatus};

use super::Waiting;
use crate::components::banner::ErrorBanner;
use crate::components::icons::{Ico, Icon};
use crate::components::markdown::Markdown;
use crate::components::select::{Choice, Select};
use crate::components::snackbar::notify;
use crate::logic::admin::{Tab, sentence};
use crate::logic::library::ago;
use crate::logic::markdown::safe_href;
use crate::logic::updates::{
    Back, ByHand, Mark, Offer, RELOAD_AFTER_MS, RESTART_POLL_MS, RUN_POLL_MS, SLOW_RESTART,
    UPDATE_COMMAND, back, by_hand, check_wait, hour_elsewhere, hour_label, hours, install_refusal,
    offer, pinned_note, release_title, run_result, state_label, state_note, steps, zone_label,
};
use crate::{api, dom, updates};

type Status = RwSignal<Option<UpdateStatus>>;

/// Shows what the server said, and tells the notice.
fn take(status: Status, s: UpdateStatus) {
    updates::put(s.clone());
    status.try_set(Some(s));
}

#[component]
pub(super) fn Updates() -> impl IntoView {
    let status: Status = RwSignal::new(None);
    let load_error = RwSignal::new(None::<String>);
    // A server from before updates.
    let old = RwSignal::new(false);
    let load = move || {
        load_error.set(None);
        spawn_local(async move {
            match api::attempt(|c| async move { c.update_status().await }).await {
                Ok(s) => take(status, s),
                Err(r) if r.status == Some(404) => {
                    updates::put_missing();
                    old.try_set(true);
                }
                Err(r) => {
                    load_error.try_set(Some(r.message));
                }
            }
        });
    };
    load();
    let retry = Callback::new(move |()| load());
    let waiting =
        move || (load_error.with(Option::is_none) && !old.get()).then(|| view! { <Waiting /> });
    view! {
        {move || load_error.get().map(|message| view! { <ErrorBanner message=message on_retry=retry /> })}
        <Show when=move || old.get()>
            <section class="card">
                <h2 class="card-title">{move || t!("admin-tab-updates")}</h2>
                <p class="muted hint">{move || t!("updates-old-server")}</p>
                <Command text=UPDATE_COMMAND />
            </section>
        </Show>
        <Show when=move || status.with(Option::is_some) fallback=waiting>
            <VersionCard status=status />
            <NotesCard status=status />
            <SettingsCard status=status />
        </Show>
    }
}

/// Follows an update: asks every moment while it goes and, while the server
/// restarts, knocks on `/info` until the new version answers, then reloads
/// the page so the new version's app loads.
#[derive(Clone, Copy)]
struct Follow {
    status: Status,
    /// The version that came up, while the page waits to reload.
    arrived: RwSignal<Option<String>>,
    slow: RwSignal<bool>,
    on: StoredValue<bool>,
}

impl Follow {
    fn start(self) {
        if self.on.try_get_value() != Some(false) {
            return;
        }
        self.on.set_value(true);
        spawn_local(async move {
            self.run().await;
            self.on.try_set_value(false);
        });
    }

    async fn run(self) {
        let mut knocking = false;
        let mut away_since: Option<DateTime<Utc>> = None;
        loop {
            let Some((to, state)) = self
                .status
                .try_with_untracked(|s| {
                    s.as_ref()
                        .and_then(|s| s.run.as_ref())
                        .filter(|r| r.state.is_running())
                        .map(|r| (r.to.clone(), r.state))
                })
                .flatten()
            else {
                return;
            };
            knocking |= state == UpdateState::Restarting;
            if knocking {
                away_since.get_or_insert_with(Utc::now);
            }
            dom::sleep(if knocking {
                RESTART_POLL_MS
            } else {
                RUN_POLL_MS
            })
            .await;
            let slow = || away_since.is_some_and(|at| Utc::now() - at > SLOW_RESTART);
            let version = if knocking {
                match api::attempt(|c| async move { c.info().await }).await {
                    Ok(info) => Some(info.version),
                    Err(_) => {
                        if self.slow.try_set(slow()).is_some() {
                            return;
                        }
                        continue;
                    }
                }
            } else {
                None
            };
            let fresh = api::attempt(|c| async move { c.update_status().await })
                .await
                .ok();
            if fresh.is_none() && !knocking {
                // Gone already: the restart came between two looks.
                knocking = true;
                continue;
            }
            match back(&to, version.as_deref(), fresh.as_ref()) {
                Back::Updated => {
                    if let Some(s) = fresh {
                        take(self.status, s);
                    }
                    if self.arrived.try_set(Some(to)).is_some() {
                        return;
                    }
                    dom::sleep(RELOAD_AFTER_MS).await;
                    // Only while the page still waits for it.
                    if self.arrived.try_with_untracked(Option::is_some) == Some(true)
                        && let Some(w) = web_sys::window()
                    {
                        let _ = w.location().reload();
                    }
                    return;
                }
                Back::Failed | Back::Waiting => {
                    if let Some(s) = fresh {
                        knocking = s
                            .run
                            .as_ref()
                            .is_some_and(|r| r.state == UpdateState::Restarting);
                        take(self.status, s);
                    }
                }
            }
            if self.slow.try_set(slow()).is_some() {
                return;
            }
        }
    }
}

/// The versions, the last check and Check now, and what can be done about a
/// newer one: update from here, by hand, or the update under way.
#[component]
fn VersionCard(status: Status) -> impl IntoView {
    let follow = Follow {
        status,
        arrived: RwSignal::new(None),
        slow: RwSignal::new(false),
        on: StoredValue::new(false),
    };
    let arrived = follow.arrived;
    // The backup was turned off for the update this page started.
    let no_backup = RwSignal::new(false);
    let offered = Memo::new(move |_| status.with(|s| s.as_ref().map_or(Offer::Unknown, offer)));
    Effect::new(move |_| {
        if offered.get() == Offer::Running {
            follow.start();
        }
    });
    let pinned = Memo::new(move |_| status.with(|s| s.as_ref().and_then(pinned_note)));
    let result = move || {
        if arrived.with(Option::is_some) || offered.get() == Offer::Running {
            return None;
        }
        let (run, current) = status.with(|s| {
            s.as_ref()
                .and_then(|s| Some((s.run.clone()?, s.current.clone())))
        })?;
        let (ok, line) = run_result(&run, &current, &Local)?;
        let backup = run.backup.clone().filter(|_| ok);
        Some(if ok {
            view! {
                <div class="update-result" role="status">
                    <Ico icon=Icon::Check size=18 />
                    <span class="break">{line}</span>
                    {backup
                        .map(|name| {
                            view! {
                                <a class="update-result-backup break" href=Tab::Backups.href()>
                                    {move || t!("updates-backup-made", name = name.clone())}
                                </a>
                            }
                        })}
                </div>
            }
            .into_any()
        } else {
            view! {
                <div class="banner error update-failed" role="alert">
                    <span class="break">{line}</span>
                </div>
            }
            .into_any()
        })
    };
    let action = move || {
        if let Some(version) = arrived.get() {
            return view! {
                <div class="update-arrived" role="status">
                    <span class="update-arrived-mark">
                        <Ico icon=Icon::Check />
                    </span>
                    <span class="update-arrived-text">
                        <strong>{move || t!("updates-done", version = version.clone())}</strong>
                        <span class="muted">{move || t!("updates-reloading")}</span>
                    </span>
                </div>
            }
            .into_any();
        }
        match offered.get() {
            Offer::Update(version) => {
                view! { <UpdateAction version=version status=status no_backup=no_backup /> }
                    .into_any()
            }
            Offer::ByHand(_) => {
                let method = status.with_untracked(|s| s.as_ref().map(|s| by_hand(&s.method)));
                method.map(|b| view! { <ByHandBox by_hand=b /> }).into_any()
            }
            Offer::Running => {
                view! { <Progress status=status no_backup=no_backup slow=follow.slow /> }.into_any()
            }
            Offer::UpToDate | Offer::Unknown => ().into_any(),
        }
    };

    view! {
        <section class="card update-card">
            <h2 class="card-title">{move || t!("updates-title")}</h2>
            <Versions status=status />
            <CheckLine status=status />
            {result}
            {action}
            {move || {
                pinned
                    .get()
                    .map(|note| {
                        view! {
                            <div class="banner warn update-pinned">
                                <span class="break">{note}</span>
                            </div>
                        }
                    })
            }}
        </section>
    }
}

/// The version running and, when there is one, the newer one, side by side.
#[component]
fn Versions(status: Status) -> impl IntoView {
    let shown = Memo::new(move |_| {
        status.with(|s| {
            s.as_ref().map(|s| {
                let newer = s.latest.clone().filter(|_| s.available);
                (s.current.clone(), newer, s.latest.is_some())
            })
        })
    });
    move || {
        let (current, newer, known) = shown.get()?;
        let up_to_date = newer.is_none() && known;
        Some(view! {
            <div class="update-versions">
                <div class="update-version">
                    <span class="update-version-label">{move || t!("updates-running")}</span>
                    <strong class="break">{current}</strong>
                    {up_to_date
                        .then(|| {
                            view! {
                                <span class="update-version-note ok">
                                    <Ico icon=Icon::Check size=16 />
                                    {move || t!("updates-up-to-date")}
                                </span>
                            }
                        })}
                </div>
                {newer
                    .map(|release| {
                        let released = release
                            .published_at
                            .map(|at| move || t!("updates-released", when = ago(at, Utc::now())));
                        view! {
                            <span class="update-versions-arrow" aria-hidden="true">
                                <Ico icon=Icon::ChevronRight />
                            </span>
                            <div class="update-version new">
                                <span class="update-version-label">{move || t!("updates-newest")}</span>
                                <strong class="break">{release.version}</strong>
                                {released.map(|r| view! { <span class="update-version-note">{r}</span> })}
                            </div>
                        }
                    })}
            </div>
        })
    }
}

/// When the server last asked GitHub, why that failed, and Check now. The
/// server takes one a minute: the button counts down to the next.
#[component]
fn CheckLine(status: Status) -> impl IntoView {
    let checking = RwSignal::new(false);
    let wait = RwSignal::new(0_u64);
    let problem = RwSignal::new(None::<String>);
    let checks = Memo::new(move |_| status.with(|s| s.as_ref().is_some_and(|s| s.settings.check)));
    let running = Memo::new(move |_| {
        status.with(|s| {
            s.as_ref()
                .and_then(|s| s.run.as_ref())
                .is_some_and(|r| r.state.is_running())
        })
    });
    let line = move || {
        let (checked, error) =
            status.with(|s| s.as_ref().map(|s| (s.checked_at, s.check_error.clone())))?;
        let checked = match checked {
            Some(at) => t!("updates-checked", when = ago(at, Utc::now())),
            None => t!("updates-not-checked"),
        };
        Some((checked, error))
    };
    let countdown = move |secs: u64| {
        wait.set(secs);
        spawn_local(async move {
            loop {
                dom::sleep(1000).await;
                let Some(left) = wait.try_get_untracked() else {
                    return;
                };
                let left = left.saturating_sub(1);
                wait.try_set(left);
                if left == 0 {
                    return;
                }
            }
        });
    };
    let check = move |_| {
        if checking.get_untracked() || wait.get_untracked() > 0 {
            return;
        }
        checking.set(true);
        problem.set(None);
        spawn_local(async move {
            match api::attempt(|c| async move { c.check_for_updates().await }).await {
                Ok(s) => {
                    let said = match s.latest.as_ref() {
                        Some(l) if s.available => {
                            t!("updates-checked-new", version = l.version.clone())
                        }
                        _ if s.check_error.is_some() => String::new(),
                        _ => t!("updates-checked-none", version = s.current.clone()),
                    };
                    take(status, s);
                    if checking.try_set(false).is_none() {
                        if !said.is_empty() {
                            notify(said);
                        }
                        countdown(check_wait(None));
                    }
                    return;
                }
                Err(r) if r.status == Some(429) => {
                    if checking.try_get_untracked().is_some() {
                        countdown(check_wait(r.retry_after));
                    }
                }
                Err(r) if r.status == Some(403) => {
                    problem.try_set(Some(t!("updates-error-checks-off")));
                }
                Err(r) => {
                    problem.try_set(Some(r.message));
                }
            }
            checking.try_set(false);
        });
    };
    let label = move || {
        let left = wait.get();
        if checking.get() {
            t!("updates-checking")
        } else if left > 0 {
            t!("updates-check-wait", seconds = left)
        } else {
            t!("updates-check-now")
        }
    };
    view! {
        <div class="update-check">
            <div class="update-check-text">
                {move || line().map(|(checked, _)| view! { <span class="muted">{checked}</span> })}
                {move || {
                    line()
                        .and_then(|(_, error)| error)
                        .map(|why| {
                            view! {
                                <span class="update-check-error">
                                    {move || t!("updates-check-failed", reason = sentence(&why))}
                                </span>
                            }
                        })
                }}
                <Show when=move || !checks.get()>
                    <span class="muted">{move || t!("updates-checks-off")}</span>
                </Show>
            </div>
            <button
                type="button"
                class="btn tonal small"
                disabled=move || checking.get() || (wait.get() > 0) || !checks.get() || running.get()
                aria-busy=move || if checking.get() { "true" } else { "false" }
                on:click=check
            >
                {move || {
                    if checking.get() {
                        view! { <span class="spinner small" aria-hidden="true"></span> }.into_any()
                    } else {
                        view! { <Ico icon=Icon::Restore /> }.into_any()
                    }
                }}
                <span class="update-check-label">{label}</span>
            </button>
        </div>
        {move || problem.get().map(|message| view! { <ErrorBanner message=message /> })}
    }
}

/// "Update to …", with the backup first unless it's turned off. The first
/// press says what will happen; the second starts it.
#[component]
fn UpdateAction(version: String, status: Status, no_backup: RwSignal<bool>) -> impl IntoView {
    let backup = RwSignal::new(true);
    let confirming = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let problem = RwSignal::new(None::<String>);
    let ask_ref: NodeRef<html::Button> = NodeRef::new();
    let go_ref: NodeRef<html::Button> = NodeRef::new();
    let focus = |button: NodeRef<html::Button>| {
        request_animation_frame(move || {
            if let Some(b) = button.try_get_untracked().flatten() {
                let _ = b.focus();
            }
        });
    };
    let ask = move |_| {
        problem.set(None);
        confirming.set(true);
        focus(go_ref);
    };
    let cancel = move || {
        if busy.get_untracked() {
            return;
        }
        confirming.set(false);
        focus(ask_ref);
    };
    let install = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        problem.set(None);
        let wanted = backup.get_untracked();
        spawn_local(async move {
            let request = UpdateInstall { backup: wanted };
            match api::attempt(move |c| async move { c.install_update(&request).await }).await {
                Ok(s) => {
                    no_backup.try_set(!wanted);
                    take(status, s);
                }
                Err(r) => {
                    let message = install_refusal(r.code.as_deref()).unwrap_or(r.message);
                    problem.try_set(Some(message));
                    confirming.try_set(false);
                    // Another admin may have started one: show it.
                    if r.code.as_deref() == Some("update_running")
                        && let Ok(s) =
                            api::attempt(|c| async move { c.update_status().await }).await
                    {
                        take(status, s);
                    }
                }
            }
            busy.try_set(false);
        });
    };
    let on_key = move |e: KeyboardEvent| {
        if e.key() == "Escape" {
            e.stop_propagation();
            cancel();
        }
    };
    let to = version.clone();
    view! {
        <div class="update-action">
            {move || problem.get().map(|message| view! { <ErrorBanner message=message /> })}
            <Show
                when=move || confirming.get()
                fallback=move || {
                    let version = version.clone();
                    view! {
                        <div class="update-offer">
                            <label class="check">
                                <input type="checkbox" bind:checked=backup />
                                <span>{move || t!("updates-backup-first")}</span>
                            </label>
                            <button type="button" class="btn" node_ref=ask_ref on:click=ask>
                                <Ico icon=Icon::ArrowUp />
                                {move || t!("updates-install", version = version.clone())}
                            </button>
                        </div>
                    }
                }
            >
                {
                    let to = to.clone();
                    view! {
                        <div class="update-confirm" role="group" aria-labelledby="update-confirm-text" on:keydown=on_key>
                            <p id="update-confirm-text">{move || t!("updates-confirm", version = to.clone())}</p>
                            <Show when=move || !backup.get()>
                                <p class="muted">{move || t!("updates-confirm-no-backup")}</p>
                            </Show>
                            <div class="row-end">
                                <button type="button" class="btn text" disabled=move || busy.get() on:click=move |_| cancel()>
                                    {move || t!("updates-cancel")}
                                </button>
                                <button type="button" class="btn" node_ref=go_ref disabled=move || busy.get() on:click=install>
                                    {move || busy.get().then(|| view! { <span class="spinner small" aria-hidden="true"></span> })}
                                    {move || t!("updates-confirm-go")}
                                </button>
                            </div>
                        </div>
                    }
                }
            </Show>
        </div>
    }
}

/// The update under way, step by step.
#[component]
fn Progress(status: Status, no_backup: RwSignal<bool>, slow: RwSignal<bool>) -> impl IntoView {
    let run = Memo::new(move |_| {
        status.with(|s| {
            s.as_ref()
                .and_then(|s| s.run.clone())
                .filter(|r| r.state.is_running())
        })
    });
    let title = move || {
        run.with(|r| {
            r.as_ref()
                .map(|r| (t!("updates-updating", version = r.to.clone()), r.automatic))
        })
    };
    // Keyed by step and mark, so only a step that moves on is drawn anew.
    let kind = Memo::new(move |_| status.with(|s| s.as_ref().map(|s| s.method.kind)));
    let rows = move || {
        let kind = kind.get().unwrap_or(UpdateKind::Docker);
        run.with(|r| {
            r.as_ref().map_or_else(Vec::new, |r| {
                steps(r, kind, no_backup.get())
                    .into_iter()
                    .enumerate()
                    .map(|(i, (state, mark))| (i, state, mark, r.to.clone()))
                    .collect::<Vec<_>>()
            })
        })
    };
    let row = |(_, state, mark, to): (usize, UpdateState, Mark, String)| {
        let class = match mark {
            Mark::Done => "update-step done",
            Mark::Now => "update-step now",
            Mark::Next => "update-step",
            Mark::Skipped => "update-step skipped",
        };
        let sign = match mark {
            Mark::Done => view! { <Ico icon=Icon::Check size=16 /> }.into_any(),
            Mark::Now => view! { <span class="spinner small"></span> }.into_any(),
            Mark::Skipped => view! { <Ico icon=Icon::Remove size=16 /> }.into_any(),
            Mark::Next => ().into_any(),
        };
        let note = move || match mark {
            Mark::Now => Some(state_note(state, &to)),
            Mark::Skipped => Some(t!("updates-step-skipped")),
            Mark::Done | Mark::Next => None,
        };
        view! {
            <li class=class aria-current=(mark == Mark::Now).then_some("step")>
                <span class="update-step-mark" aria-hidden="true">{sign}</span>
                <span class="update-step-text">
                    <strong>{move || state_label(state)}</strong>
                    {(mark == Mark::Done)
                        .then(|| view! { <span class="visually-hidden">{move || t!("updates-step-done-mark")}</span> })}
                    {move || note().map(|n| view! { <span class="muted">{n}</span> })}
                </span>
            </li>
        }
    };
    // Read out as each step starts.
    let said = move || {
        run.with(|r| {
            r.as_ref()
                .map(|r| format!("{}. {}", state_label(r.state), state_note(r.state, &r.to)))
        })
    };
    view! {
        <div class="update-progress">
            {move || {
                title()
                    .map(|(text, automatic)| {
                        view! {
                            <h3 class="update-progress-title">
                                <span class="break">{text}</span>
                                {automatic.then(|| view! { <span class="pill">{move || t!("updates-automatic")}</span> })}
                            </h3>
                        }
                    })
            }}
            <ol class="update-steps">
                <For each=rows key=|(i, _, mark, _)| (*i, *mark) children=row />
            </ol>
            <p class="visually-hidden" role="status">{said}</p>
            <Show when=move || slow.get()>
                <p class="muted hint update-slow">{move || t!("updates-slow")}</p>
            </Show>
        </div>
    }
}

/// Why the server can't update itself, and how to do it by hand.
#[component]
fn ByHandBox(by_hand: ByHand) -> impl IntoView {
    let ByHand {
        why,
        how,
        command,
        redeploy,
        socket,
    } = by_hand;
    view! {
        <div class="update-by-hand">
            <p>
                <strong>{move || t!("updates-cannot")}</strong>
                " "
                {why}
            </p>
            <p class="muted">{how}</p>
            {command.map(|text| view! { <Command text=text /> })}
            {redeploy.then(|| view! { <p class="muted">{move || t!("updates-how-redeploy")}</p> })}
            {socket.then(|| view! { <p class="muted fine">{move || t!("updates-how-socket")}</p> })}
        </div>
    }
}

/// A command to run, with a button that copies it. Where the browser won't
/// copy, the command is there to select by hand.
#[component]
fn Command(text: &'static str) -> impl IntoView {
    let copied = RwSignal::new(None::<bool>);
    let copy = move |_| {
        spawn_local(async move {
            let done = dom::copy_text(text).await;
            copied.try_set(Some(done));
        });
    };
    view! {
        <div class="update-command">
            <pre class="command">
                <code>{text}</code>
            </pre>
            <button type="button" class="btn tonal small" on:click=copy>
                {move || {
                    let icon = if copied.get() == Some(true) { Icon::Check } else { Icon::Copy };
                    view! { <Ico icon=icon /> }
                }}
                {move || {
                    if copied.get() == Some(true) { t!("updates-copied") } else { t!("updates-copy") }
                }}
            </button>
            <Show when=move || copied.get() == Some(false)>
                <p class="muted update-copy-failed" role="status">{move || t!("updates-copy-failed")}</p>
            </Show>
        </div>
    }
}

/// The newest release's notes, and a way to its page.
#[component]
fn NotesCard(status: Status) -> impl IntoView {
    let release = Memo::new(move |_| {
        status.with(|s| {
            s.as_ref()
                .and_then(|s| s.latest.clone().map(|l| (l, s.available)))
        })
    });
    move || {
        let (release, available) = release.get()?;
        let version = release.version.clone();
        let title = move || {
            if available {
                t!("updates-notes-new", version = version.clone())
            } else {
                t!("updates-notes", version = version.clone())
            }
        };
        let page = safe_href(&release.url).filter(|h| h.starts_with("http"));
        let notes = release.notes.trim().to_owned();
        let name = release_title(&release.name, &release.version);
        Some(view! {
            <section class="card update-notes">
                <h2 class="card-title break">{title}</h2>
                {name.map(|name| view! { <p class="muted hint break">{name}</p> })}
                {if notes.is_empty() {
                    view! { <p class="muted">{move || t!("updates-notes-empty")}</p> }.into_any()
                } else {
                    view! { <Markdown text=notes /> }.into_any()
                }}
                {page
                    .map(|href| {
                        view! {
                            <div class="row-end">
                                <a class="btn text" href=href target="_blank" rel="noopener noreferrer">
                                    <Ico icon=Icon::OpenInNew />
                                    {move || t!("updates-release-page")}
                                </a>
                            </div>
                        }
                    })}
            </section>
        })
    }
}

/// Checks and automatic updates, saved as they change.
#[component]
fn SettingsCard(status: Status) -> impl IntoView {
    let settings = Memo::new(move |_| status.with(|s| s.as_ref().map(|s| s.settings)));
    let can_update =
        Memo::new(move |_| status.with(|s| s.as_ref().is_some_and(|s| s.method.can_update)));
    let zone = Memo::new(move |_| status.with(|s| s.as_ref().map(|s| *s.server_time.offset())));
    // Why there are no automatic updates to turn on.
    let no_auto = move || {
        status.with(|s| {
            s.as_ref()
                .filter(|s| !s.method.can_update)
                .map(|s| by_hand(&s.method).why)
        })
    };
    let problem = RwSignal::new(None::<String>);
    let saved = RwSignal::new(false);
    let generation = StoredValue::new(0_u64);
    // Shown at once; a refusal puts it back.
    let save = move |change: &dyn Fn(&mut UpdateSettings)| {
        let Some(before) = settings.get_untracked() else {
            return;
        };
        let mut next = before;
        change(&mut next);
        if next == before {
            return;
        }
        status.update(|s| {
            if let Some(s) = s {
                s.settings = next;
            }
        });
        problem.set(None);
        saved.set(false);
        let at = generation.get_value().wrapping_add(1);
        generation.set_value(at);
        spawn_local(async move {
            let result =
                api::attempt(move |c| async move { c.set_update_settings(&next).await }).await;
            let latest = generation.try_get_value() == Some(at);
            match result {
                Ok(s) if latest => {
                    take(status, s);
                    saved.try_set(true);
                }
                Ok(_) => {}
                Err(r) => {
                    if latest {
                        status.try_update(|s| {
                            if let Some(s) = s {
                                s.settings = before;
                            }
                        });
                    }
                    problem.try_set(Some(r.message));
                }
            }
        });
    };
    let check_on = Signal::derive(move || settings.get().is_some_and(|s| s.check));
    let auto_on = Signal::derive(move || settings.get().is_some_and(|s| s.auto));
    let hour = Signal::derive(move || settings.get().map_or(0, |s| s.auto_hour).to_string());
    let choices = Signal::derive(|| {
        hours()
            .map(|h| Choice::new(h.to_string(), hour_label(h)))
            .collect::<Vec<_>>()
    });
    let clock = move || {
        let zone = zone.get()?;
        let hour = settings.get()?.auto_hour;
        let here = Local::now().offset().fix();
        let mut text = t!("updates-hour-zone", zone = zone_label(zone));
        if let Some(time) = hour_elsewhere(hour, zone, here, Utc::now()) {
            text = format!("{text} · {}", t!("updates-hour-here", time = time));
        }
        Some(text)
    };
    view! {
        <section class="card update-settings">
            <h2 class="card-title">{move || t!("updates-settings-title")}</h2>
            <SwitchRow
                id="update-check-switch"
                icon=Icon::Search
                name=Signal::derive(|| t!("updates-check-setting"))
                hint=Signal::derive(|| t!("updates-check-setting-hint"))
                on=check_on
                disabled=Signal::derive(|| false)
                on_toggle=Callback::new(move |()| save(&|s: &mut UpdateSettings| s.check = !s.check))
            />
            {move || {
                no_auto()
                    .map(|why| {
                        view! {
                            <p class="muted update-auto-off">
                                {move || t!("updates-auto-unavailable")}
                                " "
                                {why}
                            </p>
                        }
                    })
            }}
            <Show when=move || can_update.get()>
                <SwitchRow
                    id="update-auto-switch"
                    icon=Icon::Restore
                    name=Signal::derive(|| t!("updates-auto-setting"))
                    hint=Signal::derive(move || {
                        if check_on.get() {
                            t!("updates-auto-setting-hint")
                        } else {
                            t!("updates-auto-needs-check")
                        }
                    })
                    on=auto_on
                    disabled=Signal::derive(move || !check_on.get())
                    on_toggle=Callback::new(move |()| save(&|s: &mut UpdateSettings| s.auto = !s.auto))
                />
                <Show when=move || auto_on.get() && check_on.get()>
                    <div class="update-hour">
                        <span class="update-hour-label">{move || t!("updates-hour")}</span>
                        <Select
                            class="boxed"
                            value=hour
                            options=choices
                            label=Signal::derive(|| t!("updates-hour-label"))
                            on_change=Callback::new(move |value: String| {
                                if let Ok(h) = value.parse::<u8>() {
                                    save(&move |s: &mut UpdateSettings| s.auto_hour = h);
                                }
                            })
                        />
                        <span class="muted update-hour-zone">{clock}</span>
                    </div>
                </Show>
            </Show>
            {move || problem.get().map(|message| view! { <ErrorBanner message=message /> })}
            <Show when=move || saved.get()>
                <div class="row-end">
                    <span class="muted saved-note" role="status">
                        <Ico icon=Icon::Check size=18 />
                        {move || t!("updates-saved")}
                    </span>
                </div>
            </Show>
        </section>
    }
}

/// A setting as a row, with its switch at the end; a press anywhere on the
/// row flips it.
#[component]
fn SwitchRow(
    id: &'static str,
    icon: Icon,
    name: Signal<String>,
    hint: Signal<String>,
    on: Signal<bool>,
    disabled: Signal<bool>,
    on_toggle: Callback<()>,
) -> impl IntoView {
    let name_id = format!("{id}-name");
    let hint_id = format!("{id}-hint");
    let (labelled, described) = (name_id.clone(), hint_id.clone());
    view! {
        <button
            type="button"
            class="list-link switch-row"
            role="switch"
            aria-checked=move || if on.get() { "true" } else { "false" }
            aria-labelledby=labelled
            aria-describedby=described
            disabled=move || disabled.get()
            on:click=move |_| on_toggle.run(())
        >
            <Ico icon=icon />
            <span id=name_id>{move || name.get()}</span>
            <span id=hint_id class="muted">
                {move || hint.get()}
            </span>
            <span class="switch" class:on=move || on.get() aria-hidden="true">
                <span class="switch-knob">
                    {move || {
                        let icon = if on.get() { Icon::Check } else { Icon::Close };
                        view! { <Ico icon=icon size=16 /> }
                    }}
                </span>
            </span>
        </button>
    }
}
