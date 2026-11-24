//! The controls the groups of Settings share: rows of choices, choices that
//! show what they look like, and switches. Each saves at once.

use leptos::prelude::*;

use crate::components::icons::{Ico, Icon};
use crate::store::{self, Prefs};

/// One choice of a segmented control, on while `on` says so.
pub(super) fn segment(
    label: impl Fn() -> String + Send + Sync + 'static,
    on: impl Fn() -> bool + Copy + Send + Sync + 'static,
    choose: impl Fn() + 'static,
    lang: Option<&'static str>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            role="radio"
            class="seg"
            class:on=on
            lang=lang
            aria-checked=move || if on() { "true" } else { "false" }
            on:click=move |_| choose()
        >
            {label}
        </button>
    }
}

/// A row of choices under a name, all from the preferences.
pub(super) fn choices<T: Copy + PartialEq + Send + Sync + 'static>(
    name: fn() -> String,
    all: &[T],
    current: Memo<T>,
    label: fn(T) -> String,
    set: fn(&mut Prefs, T),
) -> impl IntoView {
    let buttons = all
        .iter()
        .map(|&value| {
            segment(
                move || label(value),
                move || current.get() == value,
                move || store::update_prefs(|p| set(p, value)),
                None,
            )
        })
        .collect_view();
    view! {
        <div class="segmented" role="radiogroup" aria-label=name>
            {buttons}
        </div>
    }
}

/// A choice shown as a small picture of what it looks like, its name under
/// it and a tick on the one in use. Goes in a `.picks` radio group.
pub(super) fn pick(
    label: impl Fn() -> String + Send + Sync + 'static,
    on: impl Fn() -> bool + Copy + Send + Sync + 'static,
    choose: impl Fn() + 'static,
    picture: impl IntoView + 'static,
) -> impl IntoView {
    view! {
        <button
            type="button"
            role="radio"
            class="pick"
            class:on=on
            aria-checked=move || if on() { "true" } else { "false" }
            on:click=move |_| choose()
        >
            <span class="pick-picture" aria-hidden="true">
                {picture}
            </span>
            // On the corner, clear of what the picture shows.
            <span class="pick-tick" aria-hidden="true">
                <Ico icon=Icon::Check size=14 />
            </span>
            <span class="pick-label">{label}</span>
        </button>
    }
}

/// An on/off setting as a row: its icon, its name, a line on what it does,
/// and the switch at the end. A press anywhere on the row flips it. `id`
/// names its words for a screen reader.
pub(super) fn switch_row(
    id: &'static str,
    icon: Icon,
    name: fn() -> String,
    note: fn() -> String,
    on: Memo<bool>,
    flip: fn(&mut Prefs),
) -> impl IntoView {
    view! {
        <button
            type="button"
            class="list-link switch-row"
            role="switch"
            aria-checked=move || if on.get() { "true" } else { "false" }
            aria-labelledby=format!("{id}-name")
            aria-describedby=format!("{id}-hint")
            on:click=move |_| store::update_prefs(flip)
        >
            <Ico icon=icon />
            <span id=format!("{id}-name")>{name}</span>
            <span id=format!("{id}-hint") class="muted">
                {note}
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

/// What a setting changes, a line each with its icon, lit up while it's in
/// effect.
pub(super) fn effects_list(
    title: fn() -> String,
    lines: [(Icon, fn() -> String); 3],
    live: impl Fn() -> bool + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let rows = lines
        .into_iter()
        .map(|(icon, text)| {
            view! {
                <li>
                    <Ico icon=icon size=20 />
                    <span>{text}</span>
                </li>
            }
        })
        .collect_view();
    view! {
        <div class="does" class:live=live>
            <span class="does-title">{title}</span>
            <ul>{rows}</ul>
        </div>
    }
}
