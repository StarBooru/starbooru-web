//! The tag editor, in the side column in place of the tags: the tags in a
//! box that completes them, "did you mean" for new ones spelled like a tag
//! that exists, the AI's suggestions where the server offers them, and the
//! rating. Saving sends them all with `PATCH /posts/{id}`.

use std::sync::Arc;

use leptos::ev::SubmitEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{Post, PostUpdate, Rating, TagCorrection, TagSuggestions};

use crate::api;
use crate::components::banner::ErrorBanner;
use crate::components::icons::{Ico, Icon};
use crate::components::rating::RatingPicker;
use crate::components::tag_box::TagBox;
use crate::logic::tags::{new_words, parse_tags, replace_tag, shown_fixes, untyped, with_tag};
use crate::pages::{bad_tag_text, rating_label};

/// The tag box's id, for its label.
const TAG_BOX: &str = "post-tags";

/// The editor for `post` as it was when editing began. `on_saved` gets the
/// post as the server has it after saving.
#[component]
pub fn EditCard(
    post: Arc<Post>,
    on_cancel: Callback<()>,
    on_saved: Callback<Post>,
) -> impl IntoView {
    let id = post.id;
    let text = RwSignal::new(post.tags.join(" "));
    // The text as of the last finished word, for "did you mean".
    let check = RwSignal::new(String::new());
    let rating = RwSignal::new(post.rating);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let invalid = RwSignal::new(None::<String>);
    let area: NodeRef<html::Textarea> = NodeRef::new();

    // Straight into typing, after what's there.
    area.on_load(|el| {
        request_animation_frame(move || {
            let end = u32::try_from(el.value().encode_utf16().count()).unwrap_or(u32::MAX);
            let _ = el.focus();
            let _ = el.set_selection_range(end, end);
        });
    });
    Effect::watch(
        move || text.get(),
        move |now, _, _| {
            if now.ends_with(char::is_whitespace) {
                check.set(now.clone());
            }
            // Mended, perhaps: it says again on saving.
            if invalid.with_untracked(Option::is_some) {
                invalid.set(None);
            }
        },
        false,
    );

    let save = move |e: SubmitEvent| {
        e.prevent_default();
        if busy.get_untracked() {
            return;
        }
        let tags = match parse_tags(&text.get_untracked()) {
            Ok(tags) => tags,
            Err(bad) => {
                invalid.set(Some(bad_tag_text(&bad)));
                return;
            }
        };
        let update = PostUpdate {
            tags: Some(tags),
            rating: Some(rating.get_untracked()),
            source: None,
        };
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            let result = api::call(move |c| async move { c.update_post(id, &update).await }).await;
            busy.try_set(false);
            match result {
                Ok(saved) => on_saved.run(saved),
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    };

    view! {
        <form class="card edit" on:submit=save>
            <div class="field">
                <label for=TAG_BOX>{move || t!("post-tags")}</label>
                <TagBox
                    id=TAG_BOX
                    text=text
                    area=area
                    invalid=Signal::derive(move || invalid.with(Option::is_some))
                    on_blur=Callback::new(move |()| check.set(text.get_untracked()))
                />
                {move || invalid.get().map(|m| view! { <p class="field-error break" role="alert">{m}</p> })}
            </div>
            <p class="muted hint">{move || t!("post-tags-hint")}</p>
            <DidYouMean text=text check=check saved=post.tags.clone() />
            <TagSuggester id=id text=text rating=rating />
            <div class="field">
                <span>{move || t!("rating")}</span>
                <RatingPicker value=rating />
            </div>
            {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
            <div class="row-end">
                <button class="btn text" type="button" on:click=move |_| on_cancel.run(())>
                    {move || t!("cancel")}
                </button>
                <button class="btn" type="submit" disabled=move || busy.get()>
                    <Ico icon=Icon::Check />
                    {move || t!("save")}
                </button>
            </div>
        </form>
    }
}

/// A chip for each new tag spelled like one that exists, which puts that one
/// in its place. Only a hint: a failed request shows nothing.
#[component]
fn DidYouMean(
    text: RwSignal<String>,
    check: RwSignal<String>,
    saved: Vec<String>,
) -> impl IntoView {
    // Asks again only when the new words change.
    let words = Memo::new(move |_| new_words(&check.get(), &saved));
    let fixes = LocalResource::new(move || {
        let words = words.get();
        async move {
            if words.is_empty() {
                return Vec::new();
            }
            api::call(move |c| async move { c.tag_corrections(&words).await })
                .await
                .unwrap_or_default()
        }
    });
    let shown = move || -> Vec<TagCorrection> {
        fixes
            .get()
            .map(|f| text.with(|t| shown_fixes(&f, t)))
            .unwrap_or_default()
    };
    let chips = move || {
        shown()
            .into_iter()
            .map(|TagCorrection { from, to }| {
                let title = t!("post-replace-tag", from = from.clone(), to = to.clone());
                let shown = format!("{from} → {to}");
                view! {
                    <button
                        type="button"
                        class="chip"
                        title=title
                        on:click=move |_| {
                            let next = replace_tag(&text.get_untracked(), &from, &to);
                            text.set(next.clone());
                            check.set(next);
                        }
                    >
                        {shown}
                    </button>
                }
            })
            .collect_view()
    };
    view! {
        <Show when=move || !shown().is_empty()>
            <div class="did-you-mean" role="status">
                <span class="muted">{move || t!("post-did-you-mean")}</span>
                <div class="chips">{chips}</div>
            </div>
        </Show>
    }
}

/// "Suggest tags", where the server has an AI model for it: chips to add one
/// at a time or all at once, and the rating it would give.
#[component]
fn TagSuggester(id: i64, text: RwSignal<String>, rating: RwSignal<Rating>) -> impl IntoView {
    let status = LocalResource::new(api::ai_status);
    let suggestions = RwSignal::new(None::<TagSuggestions>);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let enabled = move || status.get().flatten().is_some_and(|s| s.tagging);
    let ask = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            let result = api::call(move |c| async move { c.suggest_tags(id).await }).await;
            busy.try_set(false);
            match result {
                Ok(found) => {
                    suggestions.try_set(Some(found));
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    };
    let answer = move || {
        let found = suggestions.get()?;
        let left = text.with(|t| untyped(&found.tags, t));
        let rated = found.rating.filter(|r| *r != rating.get());
        let none = left.is_empty() && found.rating.is_none();
        let names: Vec<String> = left.iter().map(|s| s.name.clone()).collect();
        let chips = left
            .into_iter()
            .map(|s| {
                let name = s.name.clone();
                view! {
                    <button
                        type="button"
                        class="chip"
                        class:suggest-new=!s.exists
                        on:click=move |_| text.update(|t| *t = with_tag(t, &name))
                    >
                        {s.name}
                        {(!s.exists).then(|| view! { <span class="suggest-badge">{move || t!("ai-new-tag")}</span> })}
                    </button>
                }
            })
            .collect_view();
        let add_all = (!names.is_empty()).then(|| {
            view! {
                <button
                    type="button"
                    class="btn text"
                    on:click=move |_| {
                        text.update(|t| {
                            for name in &names {
                                *t = with_tag(t, name);
                            }
                        });
                    }
                >
                    {move || t!("ai-add-all")}
                </button>
            }
        });
        let take_rating = rated.map(|r| {
            view! {
                <button type="button" class="btn text" on:click=move |_| rating.set(r)>
                    {move || t!("ai-rating", rating = rating_label(r))}
                </button>
            }
        });
        Some(view! {
            {none.then(|| view! { <p class="muted hint">{move || t!("ai-no-suggestions")}</p> })}
            <div class="chips">{chips}</div>
            <div class="row-end">{take_rating} {add_all}</div>
        })
    };
    view! {
        <Show when=enabled>
            <div class="suggest">
                <button type="button" class="btn text" disabled=move || busy.get() on:click=ask>
                    {move || if busy.get() { t!("ai-suggesting") } else { t!("ai-suggest") }}
                </button>
                {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
                {answer}
            </div>
        </Show>
    }
}
