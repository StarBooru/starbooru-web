//! Editing a blacklist: its tags as chips with the × that takes each off,
//! the box that adds more with the collection's tags offered as it's typed
//! (a space or Enter makes a chip), "did you mean" for a tag the collection
//! doesn't have, and the ratings as toggles. `rating:e` typed in the box
//! turns its toggle on. The page saves each change; this only says what
//! changed. Settings has the reader's own list, the administration the
//! default one.

use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use starbooru_core::blacklist::{BlacklistError, Entry};
use starbooru_core::{Rating, TagCorrection};

use super::icons::{Ico, Icon};
use super::tag_box::TagBox;
use crate::api;
use crate::logic::blacklist::{BadEntry, Typed, read_typed, split, with_typed};
use crate::logic::tags::BadTag;
use crate::pages::{bad_tag_text, query_error_text, rating_label};

/// An entry as people read it: the tag, or the rating's name.
pub fn entry_label(entry: &str) -> String {
    match Entry::parse(entry) {
        Ok(Entry::Rating(rating)) => rating_label(rating),
        Ok(Entry::Tag(tag)) => tag,
        Err(_) => entry.to_owned(),
    }
}

/// Why a word can't go on the list, in words.
pub fn bad_entry_text(bad: &BadEntry) -> String {
    let word = bad.word.clone();
    match &bad.error {
        BlacklistError::TooMany(max) => t!("blacklist-error-many", max = *max),
        BlacklistError::NotATag(_) => t!("blacklist-error-term", word = word),
        BlacklistError::Tag(error) => bad_tag_text(&BadTag {
            word,
            error: error.clone(),
        }),
        BlacklistError::Query(error) => query_error_text(error),
    }
}

#[component]
pub fn BlacklistEditor(
    /// The start of its fields' ids.
    id: &'static str,
    /// The list as it is now.
    #[prop(into)]
    entries: Signal<Vec<String>>,
    /// The most entries it takes.
    max: usize,
    /// A change is on its way and the next has to wait for it.
    #[prop(optional, into)]
    busy: Signal<bool>,
    on_add: Callback<Typed>,
    on_remove: Callback<String>,
    on_toggle: Callback<Rating>,
    /// A "did you mean" taken: the tag typed, and the one meant.
    on_replace: Callback<(String, String)>,
) -> impl IntoView {
    let tags = Memo::new(move |_| entries.with(|e| split(e).0));
    let ratings = Memo::new(move |_| entries.with(|e| split(e).1));
    let adding = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let box_id = format!("{id}-add");
    let error_id = format!("{id}-error");

    // What's typed goes on the list, or says why it can't.
    let commit = move || {
        let typed = adding.get_untracked();
        if typed.trim().is_empty() || busy.get_untracked() {
            return;
        }
        let read = read_typed(&typed).and_then(|t| {
            let count = entries.with_untracked(|e| with_typed(e, &t).len());
            if count > max {
                Err(BadEntry {
                    word: String::new(),
                    error: BlacklistError::TooMany(max),
                })
            } else {
                Ok(t)
            }
        });
        match read {
            Ok(t) => {
                adding.set(String::new());
                error.set(None);
                on_add.run(t);
            }
            Err(bad) => error.set(Some(bad_entry_text(&bad))),
        }
    };
    // A space after a word makes it a chip at once, as in the tag editor.
    Effect::watch(
        move || adding.get(),
        move |now, _, _| {
            if error.with_untracked(Option::is_some) {
                error.set(None);
            }
            if now.ends_with(char::is_whitespace) && !now.trim().is_empty() {
                commit();
            }
        },
        false,
    );
    let add = move |e: SubmitEvent| {
        e.prevent_default();
        commit();
    };

    // Tags the collection doesn't have, and the one each was likely meant to
    // be. Only a hint: a failed request shows none.
    let fixes = LocalResource::new(move || {
        let words = tags.get().join(" ");
        async move {
            if words.is_empty() {
                return Vec::new();
            }
            api::call(move |c| async move { c.tag_corrections(&words).await })
                .await
                .unwrap_or_default()
        }
    });
    let shown_fixes = move || -> Vec<TagCorrection> {
        let have = tags.get();
        fixes
            .get()
            .unwrap_or_default()
            .into_iter()
            .filter(|f| have.contains(&f.from) && f.from != f.to)
            .collect()
    };

    // Keyed, so only a chip just added pops in.
    let chip = move |name: String| {
        let gone = name.clone();
        let tag = name.clone();
        let label = move || t!("blacklist-remove", tag = tag.clone());
        view! {
            <span class="chip removable edit-chip blacklist-chip">
                <span class="chip-name">{name}</span>
                <button
                    type="button"
                    class="chip-x"
                    title=label.clone()
                    aria-label=label
                    disabled=move || busy.get()
                    on:click=move |_| on_remove.run(gone.clone())
                >
                    <Ico icon=Icon::Close />
                </button>
            </span>
        }
    };
    let empty = Memo::new(move |_| tags.with(Vec::is_empty));
    let chips = move || {
        if empty.get() {
            return view! { <p class="muted blacklist-none">{move || t!("blacklist-none")}</p> }
                .into_any();
        }
        view! {
            <div class="chips blacklist-chips">
                <For each=move || tags.get() key=|name| name.clone() children=chip />
            </div>
        }
        .into_any()
    };
    let fix_chips = move || {
        shown_fixes()
            .into_iter()
            .map(|TagCorrection { from, to }| {
                let title = t!("post-replace-tag", from = from.clone(), to = to.clone());
                let shown = format!("{from} → {to}");
                view! {
                    <button
                        type="button"
                        class="chip"
                        title=title
                        on:click=move |_| on_replace.run((from.clone(), to.clone()))
                    >
                        {shown}
                    </button>
                }
            })
            .collect_view()
    };
    let toggles = Rating::ALL
        .into_iter()
        .map(|rating| {
            let on = move || ratings.with(|r| r.contains(&rating));
            view! {
                <button
                    type="button"
                    class="chip rating-toggle"
                    class:on=on
                    aria-pressed=move || if on() { "true" } else { "false" }
                    disabled=move || busy.get()
                    on:click=move |_| on_toggle.run(rating)
                >
                    <span class="toggle-mark" aria-hidden="true">
                        <Ico icon=Icon::EyeOff size=18 />
                    </span>
                    {move || rating_label(rating)}
                </button>
            }
        })
        .collect_view();
    let label_id = format!("{id}-ratings");
    let ratings_id = label_id.clone();

    view! {
        <div class="blacklist-editor">
            {chips}
            <form class="tag-add cat-general blacklist-add" on:submit=add>
                <TagBox
                    id=box_id
                    text=adding
                    one_line=true
                    floating=true
                    invalid=Signal::derive(move || error.with(Option::is_some))
                    described_by=Signal::derive({
                        let error_id = error_id.clone();
                        move || error.with(Option::is_some).then(|| error_id.clone())
                    })
                    placeholder=Signal::derive(|| t!("blacklist-add"))
                    label=Signal::derive(|| t!("blacklist-add"))
                />
                <button
                    type="submit"
                    class="icon-btn tonal"
                    title=move || t!("add")
                    aria-label=move || t!("add")
                    disabled=move || busy.get() || adding.with(|a| a.trim().is_empty())
                >
                    <Ico icon=Icon::Add />
                </button>
            </form>
            {move || {
                error
                    .get()
                    .map(|m| view! { <p class="field-error break" id=error_id.clone() role="alert">{m}</p> })
            }}
            <Show when=move || !shown_fixes().is_empty()>
                <div class="did-you-mean" role="status">
                    <span class="muted">{move || t!("post-did-you-mean")}</span>
                    <div class="chips">{fix_chips}</div>
                </div>
            </Show>
            <div class="blacklist-ratings" role="group" aria-labelledby=ratings_id>
                <span class="blacklist-ratings-label" id=label_id>
                    {move || t!("blacklist-ratings")}
                </span>
                <div class="chips">{toggles}</div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use starbooru_core::tag::TagError;

    use super::*;

    #[test]
    fn an_entry_reads_as_its_tag_or_its_rating() {
        assert_eq!(entry_label("blue_sky"), "blue_sky");
        assert_eq!(entry_label("rating:explicit"), "explicit");
        assert_eq!(entry_label("type:video"), "type:video");
    }

    #[test]
    fn a_word_that_cant_go_on_the_list_says_why() {
        let bad = |word: &str, error| {
            bad_entry_text(&BadEntry {
                word: word.into(),
                error,
            })
        };
        let term = bad("type:video", BlacklistError::NotATag("type:video".into()));
        assert!(
            term.contains("type:video") && !term.starts_with("blacklist-"),
            "{term}"
        );
        assert!(bad("", BlacklistError::TooMany(100)).contains("100"));
        assert!(
            bad(
                "-cat",
                BlacklistError::Tag(TagError::BadPrefix("-cat".into()))
            )
            .contains("-cat")
        );
    }
}
