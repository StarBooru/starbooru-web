//! The box a forum post is written in: plain text with Markdown, a preview
//! of how it will read, and what's left of the limit once it's close. A
//! post sent within 15 seconds of the account's last waits out the rest,
//! counting down on the button, with a way to take it back; a limit the
//! server still reports says how long in words.

use std::time::Duration;

use chrono::Utc;
use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;
use starbooru_core::forum::MAX_BODY_CHARS;

use super::last_post;
use crate::components::icons::{Ico, Icon};
use crate::components::markdown::Markdown;
use crate::logic::forum::{chars_left, check_body, error_text, left_words, wait_left};

/// The box. `on_send` gets the post once it can go: checked, and after any
/// wait. `waits` is false for an edit, which the server doesn't pace.
/// `ready` holds the box back while something beside it isn't, such as a
/// thread's title.
#[component]
pub fn Composer(
    text: RwSignal<String>,
    #[prop(into)] label: Signal<String>,
    #[prop(into)] send_label: Signal<String>,
    #[prop(into)] busy: Signal<bool>,
    on_send: Callback<String>,
    #[prop(optional)] waits: bool,
    #[prop(optional)] on_cancel: Option<Callback<()>>,
    #[prop(optional, into)] ready: Option<Signal<bool>>,
    /// Hears a press while `ready` holds the box back, to say why.
    #[prop(optional)]
    on_not_ready: Option<Callback<()>>,
    #[prop(optional)] focus: bool,
) -> impl IntoView {
    let previewing = RwSignal::new(false);
    let tried = RwSignal::new(false);
    // Seconds left before a post held back goes.
    let held = RwSignal::new(None::<u64>);
    let ticker = StoredValue::new(None::<IntervalHandle>);
    let area: NodeRef<html::Textarea> = NodeRef::new();
    if focus {
        area.on_load(|el| {
            request_animation_frame(move || {
                let _ = el.focus();
            });
        });
    }
    let admin = untrack(|| super::me().1);
    let problem = Memo::new(move |_| text.with(|t| check_body(t).err()));
    let left = move || text.with(|t| chars_left(t, MAX_BODY_CHARS));
    let ready = move || ready.is_none_or(|r| r.get());

    let stop = move || {
        if let Some(t) = ticker.try_update_value(Option::take).flatten() {
            t.clear();
        }
        held.try_set(None);
    };
    on_cleanup(stop);
    let go = move || {
        stop();
        if let Ok(body) = text.with_untracked(|t| check_body(t)) {
            // The box empties once it's sent: nothing to say about that.
            tried.set(false);
            on_send.run(body);
        }
    };
    let send = move || {
        tried.set(true);
        if busy.get_untracked() || held.get_untracked().is_some() {
            return;
        }
        if !ready() {
            if let Some(cb) = on_not_ready {
                cb.run(());
            }
            return;
        }
        if problem.with_untracked(Option::is_some) {
            if let Some(el) = area.get_untracked() {
                let _ = el.focus();
            }
            return;
        }
        let wait = if waits {
            wait_left(last_post(), Utc::now(), admin)
        } else {
            None
        };
        let Some(wait) = wait else {
            go();
            return;
        };
        held.set(Some(wait));
        let tick = set_interval_with_handle(
            move || {
                let next = held
                    .try_get_untracked()
                    .flatten()
                    .map(|s| s.saturating_sub(1));
                match next {
                    Some(0) => go(),
                    Some(n) => {
                        held.try_set(Some(n));
                    }
                    None => stop(),
                }
            },
            Duration::from_secs(1),
        );
        ticker.set_value(tick.ok());
    };
    let key = move |e: KeyboardEvent| {
        if e.key() == "Enter" && (e.ctrl_key() || e.meta_key()) {
            e.prevent_default();
            send();
        }
    };
    let tab = move |preview: bool| {
        let on = move || previewing.get() == preview;
        view! {
            <button
                type="button"
                role="tab"
                class="seg"
                class:on=on
                aria-selected=move || if on() { "true" } else { "false" }
                on:click=move |_| previewing.set(preview)
            >
                {move || if preview { t!("forum-preview") } else { t!("forum-write") }}
            </button>
        }
    };
    let preview = move || {
        if text.with(|t| t.trim().is_empty()) {
            view! { <p class="muted compose-empty">{move || t!("forum-preview-empty")}</p> }
                .into_any()
        } else {
            view! { <Markdown text=text.get() mentions=true /> }.into_any()
        }
    };
    view! {
        <div class="compose" class:held=move || held.with(Option::is_some)>
            <div class="segmented compose-tabs" role="tablist" aria-label=move || label.get()>
                {tab(false)}
                {tab(true)}
            </div>
            <div class="compose-body">
                <textarea
                    node_ref=area
                    class="compose-text"
                    class:hidden=move || previewing.get()
                    rows="5"
                    aria-label=move || label.get()
                    placeholder=move || t!("forum-placeholder")
                    aria-invalid=move || if tried.get() && problem.with(Option::is_some) { "true" } else { "false" }
                    prop:value=move || text.get()
                    on:input=move |e| text.set(event_target_value(&e))
                    on:keydown=key
                    disabled=move || held.with(Option::is_some)
                ></textarea>
                <Show when=move || previewing.get()>
                    <div class="compose-preview" aria-live="polite">
                        {preview}
                    </div>
                </Show>
            </div>
            <div class="compose-foot">
                <span class="compose-note" role="status">
                    {move || match held.get() {
                        Some(n) => Some(t!("forum-wait", seconds = n)),
                        None => {
                            let shown = tried.get().then(|| problem.get()).flatten().map(|e| error_text(&e));
                            shown.or_else(|| left().map(left_words))
                        }
                    }}
                </span>
                {on_cancel.map(|cancel| {
                    view! {
                        <button type="button" class="btn text" on:click=move |_| { stop(); cancel.run(()); }>
                            {move || t!("cancel")}
                        </button>
                    }
                })}
                <Show
                    when=move || held.with(Option::is_some)
                    fallback=move || {
                        view! {
                            <button
                                type="button"
                                class="btn"
                                disabled=move || busy.get() || text.with(|t| t.trim().is_empty())
                                on:click=move |_| send()
                            >
                                <Ico icon=Icon::Send />
                                {move || if busy.get() { t!("forum-sending") } else { send_label.get() }}
                            </button>
                        }
                    }
                >
                    <button type="button" class="btn tonal compose-held" on:click=move |_| stop()>
                        <span class="compose-count" aria-hidden="true">{move || held.get().unwrap_or(0)}</span>
                        {move || t!("forum-wait-cancel")}
                    </button>
                </Show>
            </div>
        </div>
    }
}
