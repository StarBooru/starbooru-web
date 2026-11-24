//! Votes on a post's tags, in the viewer's Tags card on a public server.
//! Each tag shows its score once it has votes; its up and down arrows open
//! beside it on hover or focus, and stay on a touch screen. Your own vote
//! is lit, and pressing it again takes it back. A tag the votes take off
//! leaves the card, shrinking away, and a notice says so. A guest is asked
//! to sign in.

use std::sync::Arc;
use std::time::Duration;

use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{Post, TagVote};

use crate::components::icons::{Ico, Icon};
use crate::components::snackbar::notify;
use crate::logic::tag_votes::{has_votes, merged, next, score_text, taken_off, vote_of, without};
use crate::{api, dom, instance};

/// How long a tag takes to leave (`.tag-vote.leaving` in post.css).
const LEAVE_MS: u64 = 420;

/// What the card around the tags shares: the tags on their way out, and
/// where a vote's answer goes.
#[derive(Clone, Copy)]
pub struct Votes {
    pub post: Signal<Option<Arc<Post>>>,
    pub leaving: RwSignal<Vec<String>>,
    pub on_voted: Callback<Post>,
}

/// The chip `chip` of tag `tag`, with its votes.
#[component]
pub fn TagVoteChip(tag: String, votes: Votes, children: Children) -> impl IntoView {
    // The vote shown while the server answers, so the press shows at once.
    let pending = RwSignal::new(None::<TagVote>);
    let busy = RwSignal::new(false);
    let key = tag.clone();
    let vote = Memo::new(move |_| {
        pending.get().unwrap_or_else(|| {
            votes
                .post
                .with(|p| p.as_deref().map(|p| vote_of(p, &key)).unwrap_or_default())
        })
    });
    let key = tag.clone();
    let counted = Memo::new(move |_| {
        pending.with(Option::is_some)
            || votes
                .post
                .with(|p| p.as_deref().is_some_and(|p| has_votes(p, &key)))
    });
    let key = tag.clone();
    let leaving = Memo::new(move |_| votes.leaving.with(|l| l.contains(&key)));
    let shown_tag = tag.replace('_', " ");
    let name = StoredValue::new(tag);

    let press = move |pressed: i16| {
        if instance::guest() {
            instance::ask_sign_in(Some(t!("guest-tag-vote-why")));
            return;
        }
        if busy.get_untracked() {
            return;
        }
        let Some(before) = votes.post.get_untracked() else {
            return;
        };
        let tag = name.get_value();
        let now = vote.get_untracked();
        let value = next(now.mine, pressed);
        let id = before.id;
        busy.set(true);
        pending.set(Some(crate::logic::tag_votes::after(now, value)));
        spawn_local(async move {
            let asked = tag.clone();
            let result =
                api::call(move |c| async move { c.vote_tag(id, &asked, value).await }).await;
            busy.try_set(false);
            match result {
                Ok(answer) => {
                    let fresh = merged(&before, answer);
                    let gone = taken_off(&before, &fresh);
                    if gone.contains(&tag) {
                        // It shrinks away first; the post without it comes
                        // after, as the card would redraw it at once.
                        votes.leaving.try_update(|l| l.extend(gone.iter().cloned()));
                        let shown = tag.replace('_', " ");
                        notify(untrack(|| t!("tag-voted-off", tag = shown)));
                        let wait = if dom::reduced_motion() { 0 } else { LEAVE_MS };
                        let kept = without(&fresh, &gone);
                        set_timeout(
                            move || {
                                votes.on_voted.try_run(kept);
                                votes
                                    .leaving
                                    .try_update(|l| l.retain(|t| !gone.contains(t)));
                                pending.try_set(None);
                            },
                            Duration::from_millis(wait),
                        );
                    } else {
                        votes.on_voted.try_run(fresh);
                        pending.try_set(None);
                    }
                }
                Err(message) => {
                    pending.try_set(None);
                    notify(untrack(|| t!("tag-vote-failed", reason = message)));
                }
            }
        });
    };

    let mine = move || vote.get().mine;
    let score = move || vote.get().score;
    let arrow = move |up: bool| {
        let value: i16 = if up { 1 } else { -1 };
        let on = move || mine() == value;
        let label = {
            let shown = shown_tag.clone();
            move || {
                let tag = shown.clone();
                match (up, on()) {
                    (true, false) => t!("tag-vote-up", tag = tag),
                    (true, true) => t!("tag-vote-up-undo", tag = tag),
                    (false, false) => t!("tag-vote-down", tag = tag),
                    (false, true) => t!("tag-vote-down-undo", tag = tag),
                }
            }
        };
        let title = label.clone();
        view! {
            <button
                type="button"
                class=if up { "tv-btn tv-up" } else { "tv-btn tv-down" }
                class:on=on
                aria-pressed=move || if on() { "true" } else { "false" }
                aria-label=label
                title=title
                disabled=move || busy.get()
                on:click=move |_| press(value)
            >
                <Ico icon=if up { Icon::ChevronUp } else { Icon::ChevronDown } size=16 />
            </button>
        }
    };
    let up = arrow.clone()(true);
    let down = arrow(false);
    view! {
        <span
            class="tag-vote"
            class:counted=move || counted.get()
            class:voted=move || mine() != 0
            class:leaving=move || leaving.get()
            data-mine=move || mine().to_string()
        >
            {children()}
            <span class="tv-controls">
                {up}
                <span
                    class="tv-score"
                    class:low=move || { score() < 0 }
                    class:high=move || { score() > 0 }
                    aria-label=move || t!("tag-vote-score", score = score())
                >
                    {move || score_text(score())}
                </span>
                {down}
            </span>
        </span>
    }
}
