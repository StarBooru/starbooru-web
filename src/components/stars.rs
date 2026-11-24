//! Voting with stars, on a server that counts them: five stars that light up
//! as far as the pointer or the keyboard goes, a press that gives that many
//! or, on the star already given, takes the vote back, the arrows to walk
//! along them, and the post's average and count beside them.

use leptos::html;
use leptos::prelude::*;
use starbooru_core::MAX_STARS;
use wasm_bindgen::JsCast;

use crate::components::icons::{Ico, Icon};
use crate::i18n::{self, Lang};
use crate::logic::stars::{average, lit, mine_label, moved, star_label, summary, tab_stop};

/// Whether the element an event came to shows its focus, as after a key
/// rather than a press.
fn keyboard_focus(e: &web_sys::FocusEvent) -> bool {
    e.target()
        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
        .is_some_and(|el| el.matches(":focus-visible").unwrap_or(false))
}

/// The five stars, with `mine` lit, and the post's average and votes. A
/// press sends the star pressed to `on_press`, which works out the vote.
#[component]
pub fn StarVote(
    #[prop(into)] mine: Signal<u8>,
    #[prop(into)] stars: Signal<Option<f64>>,
    #[prop(into)] votes: Signal<u32>,
    #[prop(into)] disabled: Signal<bool>,
    on_press: Callback<u8>,
) -> impl IntoView {
    // The star under the pointer or the keyboard's focus, which the row
    // previews; a press shows the vote again.
    let pointed = RwSignal::new(None::<u8>);
    // The star just given, for its pop.
    let given = RwSignal::new(None::<u8>);
    let buttons: [NodeRef<html::Button>; MAX_STARS as usize] =
        std::array::from_fn(|_| NodeRef::new());
    let focus = move |star: u8| {
        let el = usize::from(star)
            .checked_sub(1)
            .and_then(|i| buttons.get(i))
            .and_then(|b| b.get_untracked());
        if let Some(el) = el {
            let _ = el.focus();
        }
    };
    let row = (1..=MAX_STARS)
        .map(|star| {
            let node = usize::from(star - 1);
            let node_ref = buttons.get(node).copied().unwrap_or_default();
            let on = move || lit(star, mine.get(), pointed.get());
            view! {
                <button
                    type="button"
                    class="star"
                    node_ref=node_ref
                    class:lit=on
                    class:given=move || given.get() == Some(star)
                    disabled=move || disabled.get()
                    tabindex=move || if tab_stop(mine.get()) == star { "0" } else { "-1" }
                    aria-pressed=move || if mine.get() == star { "true" } else { "false" }
                    aria-label=move || star_label(star)
                    title=move || star_label(star)
                    on:pointerenter=move |e| {
                        if e.pointer_type() != "touch" {
                            pointed.set(Some(star));
                        }
                    }
                    on:pointermove=move |e| {
                        if e.pointer_type() != "touch" && pointed.get_untracked() != Some(star) {
                            pointed.set(Some(star));
                        }
                    }
                    on:focus=move |e| {
                        if keyboard_focus(&e) {
                            pointed.set(Some(star));
                        }
                    }
                    on:blur=move |_| pointed.set(None)
                    on:keydown=move |e| {
                        if let Some(to) = moved(&e.key(), star) {
                            e.prevent_default();
                            focus(to);
                        }
                    }
                    on:click=move |_| {
                        pointed.set(None);
                        given.set(Some(star));
                        on_press.run(star);
                    }
                >
                    {move || {
                        let icon = if on() { Icon::Star } else { Icon::StarOutline };
                        view! { <Ico icon=icon size=26 /> }
                    }}
                </button>
            }
        })
        .collect_view();
    let comma = move || i18n::lang() == Lang::Italian;
    view! {
        <div class="star-vote" role="group" aria-label=move || mine_label(mine.get())>
            <div
                class="stars"
                class:previewing=move || pointed.get().is_some()
                on:pointerleave=move |_| pointed.set(None)
            >
                {row}
            </div>
            <span class="star-summary" aria-label=move || summary(stars.get(), votes.get(), comma())>
                {move || match stars.get().filter(|_| votes.get() > 0) {
                    Some(stars) => view! {
                        <strong aria-hidden="true">{average(stars, comma())}</strong>
                        <span class="muted" aria-hidden="true">
                            {t!("stars-votes", count = votes.get())}
                        </span>
                    }
                    .into_any(),
                    None => view! { <span class="muted">{t!("stars-no-votes")}</span> }.into_any(),
                }}
            </span>
        </div>
    }
}
