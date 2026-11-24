//! Administration › Server: the mode, private or public, and on a private
//! server what a vote is, likes or stars. Each is a choice of two pictures.
//! Picking the other opens the switch in place: what the server says it
//! would delete or convert, in the app's words, a backup first for a mode,
//! and a confirmation that names what goes for good. Then the switch under
//! way, what it did, and the app as the new mode has it at once.

use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{Mode, ModeSwitch, ScoreKind, ServerSettings};

use super::Waiting;
use crate::components::banner::ErrorBanner;
use crate::components::icons::{Ico, Icon};
use crate::logic::admin::Tab;
use crate::logic::modes::{
    Kind, Refusal, Row, deletion_warning, mode_name, refusal, refusal_text, rows, score_name,
};
use crate::{api, instance};

/// A row's icon: what the things are.
fn kind_icon(kind: Option<Kind>) -> Icon {
    match kind {
        Some(Kind::ShareLinks) => Icon::Link,
        Some(Kind::BooruAccounts) => Icon::User,
        Some(Kind::RemoteSearches) => Icon::Search,
        Some(Kind::StarVotesConverted) => Icon::ThumbUp,
        Some(Kind::StarVotesDropped) => Icon::StarOutline,
        Some(Kind::LikeVotesConverted) => Icon::Star,
        None => Icon::Info,
    }
}

/// What a switch deletes and converts, a row each, those that go for good
/// marked as such; or that it touches nothing.
fn change_list(rows: Vec<Row>) -> impl IntoView {
    if rows.is_empty() {
        return view! {
            <p class="mode-nothing">
                <Ico icon=Icon::Check size=20 />
                <span>{move || t!("mode-nothing")}</span>
            </p>
        }
        .into_any();
    }
    let items = rows
        .into_iter()
        .map(|row| {
            let Row {
                kind,
                what,
                fate,
                deletes,
                ..
            } = row;
            view! {
                <li class="mode-change" class:gone=deletes>
                    <span class="mode-change-icon" aria-hidden="true">
                        <Ico icon=kind_icon(kind) size=20 />
                        {deletes.then(|| view! { <span class="mode-change-x"></span> })}
                    </span>
                    <span class="mode-change-text">
                        <strong class="break">{what}</strong>
                        {(!fate.is_empty()).then(|| view! { <span class="muted">{fate}</span> })}
                    </span>
                </li>
            }
        })
        .collect_view();
    view! { <ul class="mode-changes">{items}</ul> }.into_any()
}

/// One person, standing: a head over rounded shoulders, from `x` across and
/// `top` down to the floor. A guest is drawn in outline only.
fn person(x: f64, top: f64, size: f64, class: &'static str) -> impl IntoView {
    let r = size * 0.6;
    let shoulders = format!(
        "M{} 78 v-{} a{w} {w} 0 0 1 {} 0 v{} z",
        x - size,
        78.0 - top - 2.4 * r - size,
        2.0 * size,
        78.0 - top - 2.4 * r - size,
        w = size
    );
    view! {
        <g class=class>
            <circle cx=x cy=top + r r=r />
            <path d=shoulders />
        </g>
    }
}

/// A mode as a picture. Private: a closed door with a lock, and one person.
/// Public: the door open, people inside, and guests looking in.
fn mode_picture(mode: Mode) -> impl IntoView {
    let scene = match mode {
        Mode::Private => view! {
            <rect class="mp-frame" x="43" y="19" width="34" height="61" rx="3" />
            <rect class="mp-door" x="46" y="22" width="28" height="56" rx="2" />
            <circle class="mp-knob" cx="69.5" cy="52" r="2" />
            <path class="mp-lock-arc" d="M56.5 45v-3.5a3.5 3.5 0 0 1 7 0V45" />
            <rect class="mp-lock" x="54.5" y="44.5" width="11" height="9" rx="2" />
            {person(95.0, 40.0, 9.0, "mp-person")}
        }
        .into_any(),
        Mode::Public => view! {
            <rect class="mp-frame" x="43" y="19" width="34" height="61" rx="3" />
            <rect class="mp-inside" x="46" y="22" width="28" height="56" rx="2" />
            {person(53.0, 46.0, 5.0, "mp-person")}
            {person(66.0, 42.0, 5.0, "mp-person")}
            <polygon class="mp-door" points="46,22 33,16 33,84 46,78" />
            {person(19.0, 44.0, 7.0, "mp-person")}
            {person(90.0, 46.0, 7.0, "mp-guest")}
            {person(105.0, 50.0, 6.0, "mp-guest")}
        }
        .into_any(),
    };
    view! {
        <svg class="mode-picture" viewBox="0 0 120 90" aria-hidden="true">
            <rect class="mp-wall" x="0" y="0" width="120" height="90" />
            <rect class="mp-floor" x="0" y="78" width="120" height="12" />
            {scene}
        </svg>
    }
}

/// What a vote is, as a picture: a thumb up and one down, or five stars.
fn score_picture(kind: ScoreKind) -> impl IntoView {
    let inner = match kind {
        ScoreKind::Likes => view! {
            <span class="sp-like">
                <Ico icon=Icon::ThumbUp size=26 />
            </span>
            <span class="sp-count">"+12"</span>
            <span class="sp-dislike">
                <Ico icon=Icon::ThumbDown size=26 />
            </span>
        }
        .into_any(),
        ScoreKind::Stars => view! {
            <span class="sp-stars">
                <Ico icon=Icon::Star size=18 />
                <Ico icon=Icon::Star size=18 />
                <Ico icon=Icon::Star size=18 />
                <Ico icon=Icon::Star size=18 />
                <Ico icon=Icon::StarOutline size=18 />
            </span>
            <span class="sp-count">"4.3"</span>
        }
        .into_any(),
    };
    view! { <span class="score-picture" data-kind=kind.as_str()>{inner}</span> }
}

/// A choice of two as pictures, the one in force ticked and the one being
/// switched to outlined in dashes.
fn choice(
    label: String,
    picture: impl IntoView + 'static,
    on: Signal<bool>,
    aimed: Signal<bool>,
    disabled: Signal<bool>,
    choose: Callback<()>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            role="radio"
            class="pick"
            class:on=move || on.get()
            class:aimed=move || aimed.get()
            disabled=move || disabled.get()
            aria-checked=move || if on.get() { "true" } else { "false" }
            on:click=move |_| choose.run(())
        >
            <span class="pick-picture" aria-hidden="true">
                {picture}
            </span>
            <span class="pick-tick" aria-hidden="true">
                <Ico icon=Icon::Check size=14 />
            </span>
            <span class="pick-label">{label}</span>
        </button>
    }
}

/// The two steps of a switch, the first left out without a backup: under
/// way until the server answers, which it does once both are done.
fn progress(steps: Vec<(String, bool)>) -> impl IntoView {
    let rows = steps
        .into_iter()
        .enumerate()
        .map(|(i, (label, skipped))| {
            let class = match (skipped, i) {
                (true, _) => "update-step skipped",
                (false, 0) => "update-step now",
                _ => "update-step",
            };
            let mark = match (skipped, i) {
                (true, _) => view! { <Ico icon=Icon::Remove size=16 /> }.into_any(),
                (false, 0) => view! { <span class="spinner small"></span> }.into_any(),
                _ => ().into_any(),
            };
            view! {
                <li class=class>
                    <span class="update-step-mark" aria-hidden="true">{mark}</span>
                    <span class="update-step-text">
                        <strong>{label}</strong>
                    </span>
                </li>
            }
        })
        .collect_view();
    view! {
        <div class="mode-progress" role="status">
            <ol class="update-steps">{rows}</ol>
            <div class="mode-progress-bar" aria-hidden="true"></div>
            <p class="muted hint">{move || t!("mode-keep-open")}</p>
        </div>
    }
}

/// Where a switch of mode is.
#[derive(Clone, Debug, PartialEq)]
enum Switch {
    Closed,
    /// What switching to `to` would do; `None` while the server is asked.
    Preview {
        to: Mode,
        rows: Option<Result<Vec<Row>, String>>,
    },
    Running {
        to: Mode,
        backup: bool,
    },
    Done {
        to: Mode,
        rows: Vec<Row>,
        backup: Option<String>,
    },
}

/// The mode, as a choice between Private and Public.
#[component]
pub(super) fn ModeCard(
    settings: RwSignal<Option<ServerSettings>>,
    on_reload: Callback<()>,
) -> impl IntoView {
    let switch = RwSignal::new(Switch::Closed);
    let backup = RwSignal::new(true);
    let refused = RwSignal::new(None::<(Refusal, String)>);
    let mode = Memo::new(move |_| settings.with(|s| s.as_ref().map(|s| s.mode)));

    let preview = move |to: Mode| {
        switch.set(Switch::Preview { to, rows: None });
        refused.set(None);
        backup.set(true);
        spawn_local(async move {
            let found = api::call(move |c| async move { c.mode_preview(to).await }).await;
            switch.try_update(|s| {
                if matches!(s, Switch::Preview { to: now, rows: None } if *now == to) {
                    *s = Switch::Preview {
                        to,
                        rows: Some(found.map(|p| rows(&p.changes, false))),
                    };
                }
            });
        });
    };
    let pick = move |picked: Mode| {
        let current = mode.get_untracked();
        let busy = switch.with_untracked(|s| matches!(s, Switch::Running { .. }));
        if busy || current.is_none() {
            return;
        }
        let aimed = switch.with_untracked(|s| match s {
            Switch::Preview { to, .. } => Some(*to),
            _ => None,
        });
        if Some(picked) == current || aimed == Some(picked) {
            // The mode in force, or the one already open: the switch closes.
            switch.set(Switch::Closed);
        } else {
            preview(picked);
        }
    };
    let run = move |with_backup: bool| {
        let Switch::Preview {
            to,
            rows: Some(Ok(planned)),
        } = switch.get_untracked()
        else {
            return;
        };
        refused.set(None);
        switch.set(Switch::Running {
            to,
            backup: with_backup,
        });
        spawn_local(async move {
            let asked = ModeSwitch {
                to,
                backup: with_backup,
            };
            match api::attempt(move |c| async move { c.switch_mode(&asked).await }).await {
                Ok(done) => {
                    let (now, kind) = (done.settings.mode, done.settings.score_kind);
                    settings.try_set(Some(done.settings));
                    instance::settled(now, kind);
                    switch.try_set(Switch::Done {
                        to,
                        rows: rows(&done.changes, true),
                        backup: done.backup,
                    });
                }
                Err(failure) => {
                    let why = refusal(failure.code.as_deref());
                    if why == Refusal::SameMode {
                        // Switched already, elsewhere: the card catches up.
                        on_reload.try_run(());
                    }
                    switch.try_set(Switch::Preview {
                        to,
                        rows: Some(Ok(planned)),
                    });
                    refused.try_set(Some((why, refusal_text(why, &failure.message))));
                }
            }
        });
    };

    let aimed = move |m: Mode| {
        switch.with(
            |s| matches!(s, Switch::Preview { to, .. } | Switch::Running { to, .. } if *to == m),
        )
    };
    let choices = move || {
        Mode::ALL
            .into_iter()
            .map(|m| {
                choice(
                    mode_name(m),
                    mode_picture(m),
                    Signal::derive(move || mode.get() == Some(m)),
                    Signal::derive(move || aimed(m)),
                    Signal::derive(move || switch.with(|s| matches!(s, Switch::Running { .. }))),
                    Callback::new(move |()| pick(m)),
                )
            })
            .collect_view()
    };

    let panel = move || match switch.get() {
        Switch::Closed => None,
        Switch::Preview { to, rows: None } => Some(
            view! {
                <div class="mode-panel">
                    <h3 class="mode-panel-title">{t!("mode-switch-to", mode = mode_name(to))}</h3>
                    <Waiting />
                </div>
            }
            .into_any(),
        ),
        Switch::Preview {
            to,
            rows: Some(Err(message)),
        } => Some(
            view! {
                <div class="mode-panel">
                    <h3 class="mode-panel-title">{t!("mode-switch-to", mode = mode_name(to))}</h3>
                    <ErrorBanner message=message on_retry=Callback::new(move |()| preview(to)) />
                </div>
            }
            .into_any(),
        ),
        Switch::Preview {
            to,
            rows: Some(Ok(planned)),
        } => {
            let warning = deletion_warning(&planned);
            let deletes = warning.is_some();
            let go = move || {
                if deletes {
                    t!("mode-delete-and-switch")
                } else {
                    t!("mode-switch-to", mode = mode_name(to))
                }
            };
            let refusal_view = move || {
                refused.get().map(|(why, text)| {
                    let without = (why == Refusal::BackupUnavailable).then(|| {
                        view! {
                            <button
                                type="button"
                                class="btn tonal"
                                class:danger=deletes
                                on:click=move |_| run(false)
                            >
                                {move || t!("mode-switch-without-backup")}
                            </button>
                        }
                    });
                    view! {
                        <div class="banner error mode-refused" role="alert">
                            <span class="break">{text}</span>
                            {without}
                        </div>
                    }
                })
            };
            Some(
                view! {
                    <div class="mode-panel">
                        <h3 class="mode-panel-title">{t!("mode-switch-to", mode = mode_name(to))}</h3>
                        <p class="muted hint">
                            {match to {
                                Mode::Public => t!("mode-public-means"),
                                Mode::Private => t!("mode-private-means"),
                            }}
                        </p>
                        {change_list(planned)}
                        <button
                            type="button"
                            class="list-link switch-row"
                            role="switch"
                            aria-checked=move || if backup.get() { "true" } else { "false" }
                            aria-labelledby="mode-backup-name"
                            aria-describedby="mode-backup-hint"
                            on:click=move |_| backup.update(|b| *b = !*b)
                        >
                            <Ico icon=Icon::History />
                            <span id="mode-backup-name">{move || t!("mode-backup-first")}</span>
                            <span id="mode-backup-hint" class="muted">
                                {move || t!("mode-backup-hint")}
                            </span>
                            <span class="switch" class:on=move || backup.get() aria-hidden="true">
                                <span class="switch-knob">
                                    {move || {
                                        let icon = if backup.get() { Icon::Check } else { Icon::Close };
                                        view! { <Ico icon=icon size=16 /> }
                                    }}
                                </span>
                            </span>
                        </button>
                        {warning
                            .map(|text| {
                                view! {
                                    <p class="mode-warning">
                                        <Ico icon=Icon::DeleteForever size=20 />
                                        <span>{text}</span>
                                    </p>
                                }
                            })}
                        {refusal_view}
                        <div class="row-end">
                            <button
                                type="button"
                                class="btn text"
                                on:click=move |_| switch.set(Switch::Closed)
                            >
                                {move || t!("cancel")}
                            </button>
                            <button
                                type="button"
                                class="btn"
                                class:danger=deletes
                                on:click=move |_| run(backup.get_untracked())
                            >
                                {go}
                            </button>
                        </div>
                    </div>
                }
                .into_any(),
            )
        }
        Switch::Running { to, backup } => {
            let steps = vec![
                (t!("mode-step-backup"), !backup),
                (t!("mode-step-switch", mode = mode_name(to)), false),
            ];
            Some(
                view! {
                    <div class="mode-panel">
                        <h3 class="mode-panel-title">{t!("mode-switching", mode = mode_name(to))}</h3>
                        {progress(steps)}
                    </div>
                }
                .into_any(),
            )
        }
        Switch::Done { to, rows, backup } => {
            let kept = backup.map(|name| {
                view! {
                    <a class="mode-backup-link" href=Tab::Backups.href()>
                        <Ico icon=Icon::History size=18 />
                        <span class="break">{t!("mode-backup-made", name = name)}</span>
                        <Ico icon=Icon::ChevronRight size=18 />
                    </a>
                }
            });
            Some(
                view! {
                    <div class="mode-panel mode-done" role="status">
                        <div class="mode-done-head">
                            <span class="mode-done-mark" aria-hidden="true">
                                <Ico icon=Icon::Check />
                            </span>
                            <strong>{t!("mode-now", mode = mode_name(to))}</strong>
                        </div>
                        {change_list(rows)}
                        {kept}
                        <div class="row-end">
                            <button type="button" class="btn" on:click=move |_| switch.set(Switch::Closed)>
                                {move || t!("mode-done")}
                            </button>
                        </div>
                    </div>
                }
                .into_any(),
            )
        }
    };

    view! {
        <section class="card mode-card" id="mode">
            <h2 class="card-title">{move || t!("mode-title")}</h2>
            <Show when=move || mode.get().is_some() fallback=|| view! { <Waiting /> }>
                <div class="picks two mode-picks" role="radiogroup" aria-label=move || t!("mode-title")>
                    {choices}
                </div>
                {panel}
            </Show>
        </section>
    }
}

/// Where a change of score kind is.
#[derive(Clone, Debug, PartialEq)]
enum Rescore {
    Closed,
    Preview {
        to: ScoreKind,
        rows: Option<Result<Vec<Row>, String>>,
    },
    Saving {
        to: ScoreKind,
    },
    Done {
        to: ScoreKind,
        rows: Vec<Row>,
    },
}

/// What a vote is on a private server: likes and dislikes, or stars. A
/// public server counts likes, and the card shows why it can't change.
#[component]
pub(super) fn ScoreCard(settings: RwSignal<Option<ServerSettings>>) -> impl IntoView {
    let step = RwSignal::new(Rescore::Closed);
    let problem = RwSignal::new(None::<String>);
    let kind =
        Memo::new(move |_| settings.with(|s| s.as_ref().map(|s| s.score_kind.in_mode(s.mode))));
    let public =
        Memo::new(move |_| settings.with(|s| s.as_ref().is_some_and(|s| s.mode == Mode::Public)));
    // A switch to public closes what was open here.
    Effect::watch(
        move || public.get(),
        move |now, _, _| {
            if *now {
                step.set(Rescore::Closed);
            }
        },
        false,
    );

    let preview = move |to: ScoreKind| {
        step.set(Rescore::Preview { to, rows: None });
        problem.set(None);
        spawn_local(async move {
            let found = api::call(move |c| async move { c.score_preview(to).await }).await;
            step.try_update(|s| {
                if matches!(s, Rescore::Preview { to: now, rows: None } if *now == to) {
                    *s = Rescore::Preview {
                        to,
                        rows: Some(found.map(|p| rows(&p.changes, false))),
                    };
                }
            });
        });
    };
    let pick = move |picked: ScoreKind| {
        if public.get_untracked() || matches!(step.get_untracked(), Rescore::Saving { .. }) {
            return;
        }
        let aimed = match step.get_untracked() {
            Rescore::Preview { to, .. } => Some(to),
            _ => None,
        };
        if Some(picked) == kind.get_untracked() || aimed == Some(picked) {
            step.set(Rescore::Closed);
        } else {
            preview(picked);
        }
    };
    let save = move || {
        let Rescore::Preview {
            to,
            rows: Some(Ok(planned)),
        } = step.get_untracked()
        else {
            return;
        };
        let Some(mut asked) = settings.get_untracked() else {
            return;
        };
        asked.score_kind = to;
        asked.changes.clear();
        problem.set(None);
        step.set(Rescore::Saving { to });
        spawn_local(async move {
            match api::attempt(move |c| async move { c.set_admin_settings(&asked).await }).await {
                Ok(mut saved) => {
                    let done = rows(&std::mem::take(&mut saved.changes), true);
                    let (mode, kind) = (saved.mode, saved.score_kind);
                    settings.try_set(Some(saved));
                    instance::settled(mode, kind);
                    step.try_set(Rescore::Done { to, rows: done });
                }
                Err(failure) => {
                    let why = refusal(failure.code.as_deref());
                    step.try_set(Rescore::Preview {
                        to,
                        rows: Some(Ok(planned)),
                    });
                    problem.try_set(Some(refusal_text(why, &failure.message)));
                }
            }
        });
    };

    let aimed = move |k: ScoreKind| {
        step.with(|s| matches!(s, Rescore::Preview { to, .. } | Rescore::Saving { to } if *to == k))
    };
    let choices = move || {
        ScoreKind::ALL
            .into_iter()
            .map(|k| {
                choice(
                    score_name(k),
                    score_picture(k),
                    Signal::derive(move || kind.get() == Some(k)),
                    Signal::derive(move || aimed(k)),
                    Signal::derive(move || {
                        public.get() || step.with(|s| matches!(s, Rescore::Saving { .. }))
                    }),
                    Callback::new(move |()| pick(k)),
                )
            })
            .collect_view()
    };

    let panel = move || {
        match step.get() {
        Rescore::Closed => None,
        Rescore::Preview { to, rows: None } | Rescore::Saving { to } => Some(
            view! {
                <div class="mode-panel">
                    <h3 class="mode-panel-title">{t!("scores-switch-to", kind = score_name(to))}</h3>
                    <Waiting />
                </div>
            }
            .into_any(),
        ),
        Rescore::Preview {
            to,
            rows: Some(Err(message)),
        } => Some(
            view! {
                <div class="mode-panel">
                    <h3 class="mode-panel-title">{t!("scores-switch-to", kind = score_name(to))}</h3>
                    <ErrorBanner message=message on_retry=Callback::new(move |()| preview(to)) />
                </div>
            }
            .into_any(),
        ),
        Rescore::Preview {
            to,
            rows: Some(Ok(planned)),
        } => {
            let warning = deletion_warning(&planned);
            let deletes = warning.is_some();
            Some(
                view! {
                    <div class="mode-panel">
                        <h3 class="mode-panel-title">{t!("scores-switch-to", kind = score_name(to))}</h3>
                        <p class="muted hint">
                            {match to {
                                ScoreKind::Stars => t!("scores-stars-means"),
                                ScoreKind::Likes => t!("scores-likes-means"),
                            }}
                        </p>
                        {change_list(planned)}
                        {warning
                            .map(|text| {
                                view! {
                                    <p class="mode-warning">
                                        <Ico icon=Icon::DeleteForever size=20 />
                                        <span>{text}</span>
                                    </p>
                                }
                            })}
                        {move || problem.get().map(|message| view! { <ErrorBanner message=message /> })}
                        <div class="row-end">
                            <button type="button" class="btn text" on:click=move |_| step.set(Rescore::Closed)>
                                {move || t!("cancel")}
                            </button>
                            <button type="button" class="btn" class:danger=deletes on:click=move |_| save()>
                                {move || t!("scores-switch-to", kind = score_name(to))}
                            </button>
                        </div>
                    </div>
                }
                .into_any(),
            )
        }
        Rescore::Done { to, rows } => Some(
            view! {
                <div class="mode-panel mode-done" role="status">
                    <div class="mode-done-head">
                        <span class="mode-done-mark" aria-hidden="true">
                            <Ico icon=Icon::Check />
                        </span>
                        <strong>{t!("scores-now", kind = score_name(to))}</strong>
                    </div>
                    {change_list(rows)}
                    <div class="row-end">
                        <button type="button" class="btn" on:click=move |_| step.set(Rescore::Closed)>
                            {move || t!("mode-done")}
                        </button>
                    </div>
                </div>
            }
            .into_any(),
        ),
    }
    };

    view! {
        <section class="card mode-card" id="scores" class:locked=move || public.get()>
            <h2 class="card-title">{move || t!("scores-title")}</h2>
            <Show when=move || kind.get().is_some() fallback=|| view! { <Waiting /> }>
                <div class="picks two mode-picks" role="radiogroup" aria-label=move || t!("scores-title")>
                    {choices}
                </div>
                // On a public server: why, with the picture of that mode.
                <Show when=move || public.get()>
                    <p class="scores-locked">
                        <span class="scores-locked-picture" aria-hidden="true">
                            {mode_picture(Mode::Public)}
                        </span>
                        <span>{move || t!("scores-public-likes")}</span>
                    </p>
                </Show>
                {panel}
            </Show>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_has_an_icon_of_its_own() {
        let icons: std::collections::BTreeSet<String> = Kind::ALL
            .into_iter()
            .map(|k| format!("{:?}", kind_icon(Some(k))))
            .collect();
        assert_eq!(icons.len(), Kind::ALL.len());
        assert_eq!(kind_icon(None), Icon::Info);
    }

    #[test]
    fn the_switch_asks_for_a_backup_and_names_what_goes() {
        let source = include_str!("modes.rs");
        let (page, _) = source.split_once("#[cfg(test)]").unwrap_or_default();
        for needle in [
            "let backup = RwSignal::new(true);",
            "deletion_warning(&planned)",
            "Refusal::BackupUnavailable",
            "t!(\"mode-switch-without-backup\")",
            "instance::settled(now, kind)",
            "Tab::Backups.href()",
        ] {
            assert!(page.contains(needle), "{needle}");
        }
    }
}
