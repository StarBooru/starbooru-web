//! The tag editor, in the side column in place of the tags: the same chips
//! under the same headings, each with a button at its end that takes it off,
//! and under them the row that adds tags: the category new ones get, picked
//! by colour, the box that completes them, and the AI's suggestions where
//! the server offers them. A tag added lands under its category, one the
//! collection doesn't have yet says so, and one taken off can be put back.
//! The whole list can still be typed as text. "Did you mean" for new tags
//! spelled like one that exists, and the rating. Saving sends them all with
//! `PATCH /posts/{id}`.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use leptos::ev::{KeyboardEvent, MouseEvent, SubmitEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use send_wrapper::SendWrapper;
use starbooru_core::{
    Post, PostUpdate, Rating, TagCategory, TagCorrection, TagOrder, TagSuggestions, normalize_tag,
    split_category,
};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

use super::side::group_label;
use crate::api;
use crate::components::banner::ErrorBanner;
use crate::components::icons::{Ico, Icon};
use crate::components::rating::RatingPicker;
use crate::components::snackbar::notify_undo;
use crate::components::tag_box::TagBox;
use crate::dom;
use crate::logic::search::category_class;
use crate::logic::tags::{
    Edit, base_of, edit_of, has_base, new_words, parse_tags, rebase, replace_tag, shown_fixes,
    tags_changed, typed_category, untyped, with_category, with_tag, without_tag,
};
use crate::logic::viewer::{Group, edit_groups, with_added};
use crate::pages::tags::tint_class;
use crate::pages::{CATEGORIES, bad_tag_text, category_label, rating_label};

/// The tag box's id, for its label.
const TAG_BOX: &str = "post-tags";
/// The box that adds tags.
const ADD_BOX: &str = "post-tag-add";
/// How long a chip takes to leave, in milliseconds.
const LEAVE_MS: i32 = 160;

/// Each tag's category as far as it's known, by name without a prefix.
type Known = RwSignal<HashMap<String, TagCategory>>;

/// Whether the collection has tag `name`, and its category if so. `Err`
/// when it couldn't be asked.
async fn look_up(name: String) -> Result<Option<TagCategory>, String> {
    let asked = name.clone();
    let found =
        api::call(move |c| async move { c.tag_list(&asked, TagOrder::Name, 1, 1).await }).await?;
    Ok(found
        .into_iter()
        .find(|t| t.name == name)
        .map(|t| t.category))
}

/// The category word `word` of the editor's text gives its tag: its prefix's,
/// or the tag's own, or general while that isn't known.
fn category_of(word: &str, known: &HashMap<String, TagCategory>) -> TagCategory {
    let tag = normalize_tag(word).unwrap_or_else(|_| word.to_owned());
    match split_category(&tag) {
        (Some(category), _) => category,
        (None, base) => known.get(base).copied().unwrap_or(TagCategory::General),
    }
}

/// The editor for `post` as it was when editing began. `current` is the post
/// on the page, whose tags saving its people may change meanwhile.
/// `unsaved` says whether leaving now would lose changes. `on_saved` gets
/// the post as the server has it after saving.
#[component]
pub fn EditCard(
    post: Arc<Post>,
    #[prop(into)] current: Signal<Option<Arc<Post>>>,
    unsaved: RwSignal<bool>,
    on_cancel: Callback<()>,
    on_saved: Callback<Post>,
) -> impl IntoView {
    let id = post.id;
    let text = RwSignal::new(post.tags.join(" "));
    // The post's tags the text was last brought in line with.
    let base = StoredValue::new(post.tags.clone());
    Effect::watch(
        move || current.with(|p| p.as_ref().filter(|p| p.id == id).map(|p| p.tags.clone())),
        move |now, _, _| {
            let Some(now) = now else {
                return;
            };
            if base.with_value(|b| b == now) {
                return;
            }
            let before = base.get_value();
            // The base first: what follows the text reads it.
            base.set_value(now.clone());
            text.update(|t| *t = rebase(t, &before, now));
        },
        false,
    );
    let edit = Memo::new(move |_| text.with(|t| base.with_value(|b| edit_of(t, b))));
    // The text as of the last finished word, for "did you mean".
    let check = RwSignal::new(String::new());
    let rating = RwSignal::new(post.rating);
    let first_rating = post.rating;
    // What's typed in the box that adds, and why it can't be added.
    let adding = RwSignal::new(String::new());
    let add_error = RwSignal::new(None::<String>);
    // `base` changes only with `text`, so following the text is enough. A
    // word still in the box that adds counts too.
    Effect::new(move |_| {
        let changed = text.with(|t| base.with_value(|b| tags_changed(t, b)))
            || rating.get() != first_rating
            || adding.with(|a| !a.trim().is_empty());
        if unsaved.get_untracked() != changed {
            unsaved.set(changed);
        }
    });
    on_cleanup(move || unsaved.set(false));
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let invalid = RwSignal::new(None::<String>);
    let area: NodeRef<html::Textarea> = NodeRef::new();
    let add_area: NodeRef<html::Textarea> = NodeRef::new();
    // The whole list as text, as the editor always was.
    let as_text = RwSignal::new(false);
    // Chips on their way out.
    let leaving = RwSignal::new(Vec::<String>::new());
    // The category new tags get: picked, or named by a prefix typed.
    let pick = RwSignal::new(TagCategory::General);
    let known: Known = RwSignal::new(
        post.tag_details
            .iter()
            .map(|t| (t.name.clone(), t.category))
            .collect(),
    );
    // Tags the collection doesn't have yet, and the ones asked about.
    let fresh = RwSignal::new(HashSet::<String>::new());
    let asked = StoredValue::new(HashSet::<String>::new());
    // Words given the picked category's prefix rather than typed with it: a
    // tag the collection has loses it again, keeping its own category.
    let applied = StoredValue::new(HashSet::<String>::new());

    // Straight into typing, after what's there.
    let focus_end = |el: web_sys::HtmlTextAreaElement| {
        request_animation_frame(move || {
            let end = u32::try_from(el.value().encode_utf16().count()).unwrap_or(u32::MAX);
            let _ = el.focus();
            let _ = el.set_selection_range(end, end);
        });
    };
    // Each time either box comes, as the editor opens and as text and chips
    // take turns.
    Effect::new(move |_| {
        if let Some(el) = area.get() {
            focus_end(el);
        }
    });
    Effect::new(move |_| {
        if let Some(el) = add_area.get() {
            focus_end(el);
        }
    });
    // Whichever box had the focus may leave the page while it still has it,
    // and a box leaving the page loses the focus.
    let checked = Callback::new(move |()| {
        if let Some(now) = text.try_get_untracked() {
            check.try_set(now);
        }
    });

    // Word `word` is a tag the collection has: it keeps that tag's category.
    let keep_own = move |word: &str, name: &str| {
        let was_applied = applied
            .try_update_value(|a| a.remove(word))
            .unwrap_or(false);
        if was_applied {
            text.try_update(|t| *t = replace_tag(t, word, name));
        }
    };
    // A tag added that the page knows nothing of: is it the collection's,
    // and of what category?
    Effect::new(move |_| {
        let wanted: Vec<(String, String)> = edit.with(|e| {
            e.added
                .iter()
                .map(|w| (w.clone(), base_of(w)))
                .filter(|(_, name)| {
                    !known.with_untracked(|k| k.contains_key(name))
                        && !asked.with_value(|a| a.contains(name))
                })
                .collect()
        });
        for (word, name) in wanted {
            asked.update_value(|a| {
                a.insert(name.clone());
            });
            spawn_local(async move {
                match look_up(name.clone()).await {
                    Ok(Some(category)) => {
                        known.try_update(|k| {
                            k.insert(name.clone(), category);
                        });
                        keep_own(&word, &name);
                    }
                    Ok(None) => {
                        fresh.try_update(|f| {
                            f.insert(name);
                        });
                    }
                    // Asked again with the next change.
                    Err(_) => {
                        asked.try_update_value(|a| a.remove(&name));
                    }
                }
            });
        }
    });

    // Takes a chip off once it has left, with a way to put it back. The
    // focus goes on to the chip beside it, or to the box that adds.
    let remove = move |(name, next): (String, Option<SendWrapper<HtmlElement>>)| {
        if leaving.with_untracked(|l| l.contains(&name)) {
            return;
        }
        leaving.update(|l| l.push(name.clone()));
        let wait = if dom::reduced_motion() { 0 } else { LEAVE_MS };
        spawn_local(async move {
            dom::sleep(wait).await;
            leaving.try_update(|l| l.retain(|n| *n != name));
            if text.try_update(|t| *t = without_tag(t, &name)).is_none() {
                return;
            }
            match next.filter(|el| el.is_connected()) {
                Some(el) => {
                    let _ = el.focus();
                }
                None => {
                    if let Some(el) = add_area.try_get_untracked().flatten() {
                        let _ = el.focus();
                    }
                }
            }
            let back = name.clone();
            notify_undo(
                untrack(|| t!("post-tag-removed", name = base_of(&name))),
                untrack(|| t!("press-undo")),
                move || {
                    text.try_update(|t| *t = with_tag(t, &back));
                },
            );
        });
    };
    // Puts tag `tag` among the chips, with the category picked when it's new.
    let put = move |tag: &str, picked: TagCategory, exists: Option<bool>| {
        if text.with_untracked(|t| has_base(t, tag)) {
            return;
        }
        let name = base_of(tag);
        let is_known = exists.unwrap_or_else(|| known.with_untracked(|k| k.contains_key(&name)));
        let word = with_category(tag, picked, is_known);
        if word != tag {
            applied.update_value(|a| {
                a.insert(word.clone());
            });
        }
        text.update(|t| *t = with_tag(t, &word));
    };
    // Puts what's typed in the box that adds among the chips, or says why
    // it can't be.
    let commit = move || -> bool {
        let typed = adding.get_untracked();
        if typed.trim().is_empty() {
            return false;
        }
        match parse_tags(&typed) {
            Ok(tags) => {
                let picked = pick.get_untracked();
                for tag in &tags {
                    put(tag, picked, None);
                }
                adding.set(String::new());
                add_error.set(None);
                check.set(text.get_untracked());
                true
            }
            Err(bad) => {
                add_error.set(Some(bad_tag_text(&bad)));
                false
            }
        }
    };
    // A space after a word makes it a chip at once, and a prefix typed
    // picks its category.
    Effect::watch(
        move || adding.get(),
        move |now, _, _| {
            if add_error.with_untracked(Option::is_some) {
                add_error.set(None);
            }
            if let Some(category) = typed_category(now)
                && pick.get_untracked() != category
            {
                pick.set(category);
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
    let to_text = move |_| {
        // A word half typed goes along.
        commit();
        as_text.update(|t| *t = !*t);
    };
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

    let save = move || {
        if busy.get_untracked() {
            return;
        }
        // A tag still in the box that adds is meant too.
        if !as_text.get_untracked() && !adding.with_untracked(|a| a.trim().is_empty()) && !commit()
        {
            return;
        }
        if let Err(bad) = parse_tags(&text.get_untracked()) {
            invalid.set(Some(bad_tag_text(&bad)));
            return;
        }
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            // A tag given the picked category that the collection turns out
            // to have keeps its own, even when saved before that was known.
            let unsure: Vec<String> = applied
                .try_with_value(|a| a.iter().cloned().collect())
                .unwrap_or_default();
            for word in unsure {
                let name = base_of(&word);
                let settled = known.with_untracked(|k| k.contains_key(&name))
                    || fresh.with_untracked(|f| f.contains(&name));
                if settled {
                    continue;
                }
                if let Ok(Some(category)) = look_up(name.clone()).await {
                    known.try_update(|k| {
                        k.insert(name.clone(), category);
                    });
                    keep_own(&word, &name);
                }
            }
            let Some(now) = text.try_get_untracked() else {
                return;
            };
            let tags = match parse_tags(&now) {
                Ok(tags) => tags,
                Err(bad) => {
                    invalid.try_set(Some(untrack(|| bad_tag_text(&bad))));
                    busy.try_set(false);
                    return;
                }
            };
            let update = PostUpdate {
                tags: Some(tags),
                rating: rating.try_get_untracked(),
                source: None,
            };
            let result = api::call(move |c| async move { c.update_post(id, &update).await }).await;
            busy.try_set(false);
            match result {
                Ok(saved) => {
                    // Gone with its card if the page moved on meanwhile.
                    on_saved.try_run(saved);
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    };

    // The AI's suggestions, where the server has a model for it.
    let ai = LocalResource::new(api::ai_status);
    let can_suggest = move || ai.get().flatten().is_some_and(|s| s.tagging);
    let suggestions = RwSignal::new(None::<TagSuggestions>);
    let suggesting = RwSignal::new(false);
    let suggest_error = RwSignal::new(None::<String>);
    let ask = move |_: ()| {
        if suggesting.get_untracked() {
            return;
        }
        suggesting.set(true);
        suggest_error.set(None);
        spawn_local(async move {
            let result = api::call(move |c| async move { c.suggest_tags(id).await }).await;
            suggesting.try_set(false);
            match result {
                Ok(found) => {
                    known.try_update(|k| {
                        for s in found.tags.iter().filter(|s| s.exists) {
                            k.entry(s.name.clone()).or_insert(s.category);
                        }
                    });
                    suggestions.try_set(Some(found));
                }
                Err(message) => {
                    suggest_error.try_set(Some(message));
                }
            }
        });
    };
    let take = Callback::new(
        move |(name, category, exists): (String, TagCategory, bool)| {
            put(&name, category, Some(exists));
        },
    );

    // Ctrl+Enter saves from anywhere in the card.
    let on_key = move |e: KeyboardEvent| {
        if e.key() == "Enter" && (e.ctrl_key() || e.meta_key()) && !e.is_composing() {
            e.prevent_default();
            save();
        }
    };
    let rated = move || rating.get() != first_rating;
    let status = move || {
        let (added, removed) = edit.with(|e| (e.added.len() + e.bad.len(), e.removed.len()));
        let rating_changed = rated();
        view! {
            {(added > 0)
                .then(|| {
                    view! {
                        <span class="diff added" title=move || t!("post-tags-added", count = added)>
                            <span aria-hidden="true">{format!("+{added}")}</span>
                            <span class="visually-hidden">{move || t!("post-tags-added", count = added)}</span>
                        </span>
                    }
                })}
            {(removed > 0)
                .then(|| {
                    view! {
                        <span class="diff removed" title=move || t!("post-tags-removed", count = removed)>
                            <span aria-hidden="true">{format!("−{removed}")}</span>
                            <span class="visually-hidden">{move || t!("post-tags-removed", count = removed)}</span>
                        </span>
                    }
                })}
            {rating_changed
                .then(|| {
                    view! { <span class="diff rated">{move || rating_label(rating.get())}</span> }
                })}
        }
    };
    let chips = move || {
        view! {
            <ChipGroups
                edit=edit
                current=current
                known=known
                fresh=fresh
                leaving=leaving
                on_remove=Callback::new(remove)
            />
            <div class="tag-adder">
                <div class="adder-head">
                    <CategoryPick pick=pick />
                    <Show when=can_suggest>
                        <button
                            type="button"
                            class="btn tonal small suggest-btn"
                            class:busy=move || suggesting.get()
                            title=move || t!("ai-suggest")
                            disabled=move || suggesting.get()
                            on:click=move |_| ask(())
                        >
                            <Ico icon=Icon::Sparkle size=18 />
                            <span>{move || t!("ai-suggest-short")}</span>
                        </button>
                    </Show>
                </div>
                <form class=move || format!("tag-add {}", tint_class(pick.get())) on:submit=add>
                    <TagBox
                        id=ADD_BOX
                        text=adding
                        one_line=true
                        floating=true
                        area=add_area
                        invalid=Signal::derive(move || add_error.with(Option::is_some))
                        placeholder=Signal::derive(move || {
                            t!("post-add-as", category = category_label(pick.get()))
                        })
                        label=Signal::derive(move || {
                            t!("post-add-as", category = category_label(pick.get()))
                        })
                        on_blur=checked
                    />
                    <button
                        type="submit"
                        class="icon-btn tonal"
                        title=move || t!("add")
                        aria-label=move || t!("add")
                        disabled=move || adding.with(|a| a.trim().is_empty())
                    >
                        <Ico icon=Icon::Add />
                    </button>
                </form>
                {move || add_error.get().map(|m| view! { <p class="field-error break" role="alert">{m}</p> })}
                <Suggested
                    suggestions=suggestions
                    suggesting=suggesting
                    error=suggest_error
                    text=text
                    rating=rating
                    on_take=take
                    on_retry=Callback::new(move |()| ask(()))
                />
            </div>
        }
    };
    let typed = move || {
        view! {
            <div class="field">
                <label for=TAG_BOX class="visually-hidden">{move || t!("post-tags")}</label>
                <TagBox
                    id=TAG_BOX
                    text=text
                    area=area
                    invalid=Signal::derive(move || invalid.with(Option::is_some))
                    on_blur=checked
                />
            </div>
            <p class="muted hint">{move || t!("post-tags-hint")}</p>
        }
    };

    view! {
        <section
            class="card edit tag-edit"
            class:dirty=move || unsaved.get()
            aria-label=move || t!("post-edit-tags")
            on:keydown=on_key
        >
            <div class="card-head">
                <h2 class="card-title">{move || t!("post-tags")}</h2>
                <span class="edit-status" aria-live="polite">{status}</span>
                <button
                    type="button"
                    class="icon-btn"
                    class:on=move || as_text.get()
                    aria-pressed=move || if as_text.get() { "true" } else { "false" }
                    title=move || t!("post-tags-as-text")
                    aria-label=move || t!("post-tags-as-text")
                    on:click=to_text
                >
                    <Ico icon=Icon::Article />
                </button>
            </div>
            {move || if as_text.get() { typed().into_any() } else { chips().into_any() }}
            {move || invalid.get().map(|m| view! { <p class="field-error break" role="alert">{m}</p> })}
            <DidYouMean text=text check=check saved=post.tags.clone() />
            <div class="field">
                <span>{move || t!("rating")}</span>
                <RatingPicker value=rating />
            </div>
            {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
            <div class="edit-bar">
                <span class="edit-note">
                    {move || {
                        if unsaved.get() { t!("post-tags-unsaved") } else { t!("post-tags-no-changes") }
                    }}
                </span>
                <button class="btn text" type="button" on:click=move |_| on_cancel.run(())>
                    {move || t!("cancel")}
                </button>
                <button
                    class="btn"
                    type="button"
                    disabled=move || busy.get() || !unsaved.get()
                    title=move || t!("post-save-title")
                    on:click=move |_| save()
                >
                    <Ico icon=Icon::Check />
                    {move || t!("save")}
                </button>
            </div>
        </section>
    }
}

/// The category new tags get, as a row of the five colours: the one picked
/// is ringed and the box names it; each names itself on hover and to a
/// screen reader. The arrow keys move along it.
#[component]
fn CategoryPick(pick: RwSignal<TagCategory>) -> impl IntoView {
    let row: NodeRef<html::Div> = NodeRef::new();
    let go = move |to: TagCategory| {
        pick.set(to);
        // The focus goes with the choice.
        if let Some(button) = row
            .get_untracked()
            .and_then(|r| {
                r.query_selector(&format!(".cat-opt.{}", tint_class(to)))
                    .ok()
                    .flatten()
            })
            .and_then(|el| el.dyn_into::<HtmlElement>().ok())
        {
            let _ = button.focus();
        }
    };
    let on_key = move |e: KeyboardEvent| {
        let at = CATEGORIES
            .iter()
            .position(|c| *c == pick.get_untracked())
            .unwrap_or(0);
        let last = CATEGORIES.len().saturating_sub(1);
        let to = match e.key().as_str() {
            "ArrowRight" | "ArrowDown" => Some(if at >= last { 0 } else { at + 1 }),
            "ArrowLeft" | "ArrowUp" => Some(at.checked_sub(1).unwrap_or(last)),
            "Home" => Some(0),
            "End" => Some(last),
            _ => None,
        };
        if let Some(to) = to.and_then(|i| CATEGORIES.get(i)) {
            e.prevent_default();
            go(*to);
        }
    };
    let options = CATEGORIES.map(|c| {
        let on = move || pick.get() == c;
        view! {
            <button
                type="button"
                role="radio"
                class=format!("cat-opt {}", tint_class(c))
                class:on=on
                aria-checked=move || if on() { "true" } else { "false" }
                tabindex=move || if on() { "0" } else { "-1" }
                title=move || category_label(c)
                aria-label=move || category_label(c)
                on:click=move |_| go(c)
            >
                <span class="cat-dot" aria-hidden="true"></span>
            </button>
        }
    });
    view! {
        <div
            class="cat-pick-row"
            role="radiogroup"
            aria-label=move || t!("post-add-category")
            node_ref=row
            on:keydown=on_key
        >
            {options}
        </div>
    }
}

/// The remove button of the chip after the one pressed, or before it, for
/// the focus to go on to once that one has gone.
fn beside(e: &MouseEvent) -> Option<SendWrapper<HtmlElement>> {
    let chip = e
        .current_target()?
        .dyn_into::<Element>()
        .ok()?
        .closest(".edit-chip")
        .ok()??;
    let near = chip
        .next_element_sibling()
        .or_else(|| chip.previous_element_sibling())?;
    let button = near.query_selector(".chip-x").ok()??;
    button.dyn_into::<HtmlElement>().ok().map(SendWrapper::new)
}

/// The chips of the tag editor: the post's tags under their headings, and
/// the ones added under their categories, each with a button that takes it
/// off. Keyed by the word, so only the chips that change come and go.
#[component]
fn ChipGroups(
    edit: Memo<Edit>,
    current: Signal<Option<Arc<Post>>>,
    known: Known,
    fresh: RwSignal<HashSet<String>>,
    leaving: RwSignal<Vec<String>>,
    on_remove: Callback<(String, Option<SendWrapper<HtmlElement>>)>,
) -> impl IntoView {
    // Each group: its key, its heading (none for the plain list) and its tags.
    let groups = Memo::new(move |_| {
        let details = current.with(|p| {
            p.as_ref()
                .map(|p| p.tag_details.clone())
                .unwrap_or_default()
        });
        edit.with(|e| {
            let kept = edit_groups(&details, &e.kept);
            let added: Vec<(String, TagCategory)> = known.with(|k| {
                e.added
                    .iter()
                    .map(|w| (w.clone(), category_of(w, k)))
                    .chain(e.bad.iter().map(|b| (b.clone(), TagCategory::General)))
                    .collect()
            });
            let all = with_added(kept, &added);
            let headed = !matches!(all.as_slice(), [(Group::Other, _)]);
            all.into_iter()
                .map(|(g, names)| {
                    let label = headed.then(|| group_label(&g));
                    (format!("{g:?}"), label, names)
                })
                .collect::<Vec<_>>()
        })
    });
    let bad = move |name: &str| edit.with(|e| e.bad.iter().any(|b| b == name));
    let added = move |name: &str| edit.with(|e| e.added.iter().any(|a| a == name));
    let chip = move |word: String| {
        let name = base_of(&word);
        // One class list: a dynamic `class` beside `class:` toggles would
        // overwrite them as the category arrives.
        let class = {
            let (word, name) = (word.clone(), name.clone());
            move || {
                let category = known.with(|k| category_class(category_of(&word, k)));
                let mut class = format!("chip removable edit-chip {category}");
                for (on, flag) in [
                    (added(&word), " added"),
                    (added(&word) && fresh.with(|f| f.contains(&name)), " fresh"),
                    (bad(&word), " bad"),
                    (leaving.with(|l| l.contains(&word)), " leaving"),
                ] {
                    if on {
                        class.push_str(flag);
                    }
                }
                class
            }
        };
        let label = t!("tags-remove-named", name = name.clone());
        let is_fresh = {
            let (word, name) = (word.clone(), name.clone());
            move || added(&word) && fresh.with(|f| f.contains(&name))
        };
        view! {
            <span class=class>
                <span class="chip-name">{name.clone()}</span>
                <Show when=is_fresh>
                    <span class="suggest-badge">{move || t!("ai-new-tag")}</span>
                </Show>
                <button
                    type="button"
                    class="chip-x"
                    title=label.clone()
                    aria-label=label
                    on:click=move |e| on_remove.run((word.clone(), beside(&e)))
                >
                    <Ico icon=Icon::Close />
                </button>
            </span>
        }
    };
    view! {
        <Show
            when=move || groups.with(|g| !g.is_empty())
            fallback=|| view! { <p class="muted">{move || t!("post-no-tags")}</p> }
        >
            <div class="tag-groups editing">
                <For
                    each=move || groups.get()
                    key=|(key, label, _)| (key.clone(), label.is_some())
                    children=move |(key, label, _)| {
                        let names = move || {
                            groups
                                .with(|g| g.iter().find(|(k, ..)| *k == key).map(|(.., n)| n.clone()))
                                .unwrap_or_default()
                        };
                        view! {
                            <section class="tag-group">
                                {label.map(|l| view! { <h3>{l}</h3> })}
                                <div class="chips">
                                    <For each=names key=|n| n.clone() children=chip />
                                </div>
                            </section>
                        }
                    }
                />
            </div>
        </Show>
    }
}

/// The AI's suggestions, under the row that adds tags: dashed chips that
/// each add their tag, under its category, with one press, the rating it
/// would give, and "Add all".
#[component]
fn Suggested(
    suggestions: RwSignal<Option<TagSuggestions>>,
    suggesting: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    text: RwSignal<String>,
    rating: RwSignal<Rating>,
    on_take: Callback<(String, TagCategory, bool)>,
    on_retry: Callback<()>,
) -> impl IntoView {
    let answer = move || {
        let found = suggestions.get()?;
        let left = text.with(|t| untyped(&found.tags, t));
        let left: Vec<_> = left
            .into_iter()
            .filter(|s| text.with(|t| !has_base(t, &s.name)))
            .collect();
        let rated = found.rating.filter(|r| *r != rating.get());
        if left.is_empty() && rated.is_none() {
            let none = found.tags.is_empty() && found.rating.is_none();
            return Some(
                none.then(|| view! { <p class="muted hint">{move || t!("ai-no-suggestions")}</p> })
                    .into_any(),
            );
        }
        let all: Vec<(String, TagCategory, bool)> = left
            .iter()
            .map(|s| (s.name.clone(), s.category, s.exists))
            .collect();
        let chips = left
            .into_iter()
            .map(|s| {
                let taken = (s.name.clone(), s.category, s.exists);
                let class = format!("chip suggested-chip {}", category_class(s.category));
                let title = t!("post-add-suggested", name = s.name.clone());
                view! {
                    <button
                        type="button"
                        class=class
                        title=title
                        on:click=move |_| on_take.run(taken.clone())
                    >
                        <Ico icon=Icon::Add size=16 />
                        {s.name}
                        {(!s.exists).then(|| view! { <span class="suggest-badge">{move || t!("ai-new-tag")}</span> })}
                    </button>
                }
            })
            .collect_view();
        let add_all = (all.len() > 1).then(|| {
            view! {
                <button
                    type="button"
                    class="btn text small"
                    on:click=move |_| {
                        for taken in &all {
                            on_take.run(taken.clone());
                        }
                    }
                >
                    {move || t!("ai-add-all")}
                </button>
            }
        });
        let take_rating = rated.map(|r| {
            view! {
                <button type="button" class="btn text small" on:click=move |_| rating.set(r)>
                    {move || t!("ai-rating", rating = rating_label(r))}
                </button>
            }
        });
        Some(
            view! {
                <div class="chips">{chips}</div>
                <div class="suggested-actions">
                    {take_rating}
                    {add_all}
                    <button
                        type="button"
                        class="btn text small"
                        on:click=move |_| suggestions.set(None)
                    >
                        {move || t!("ai-dismiss")}
                    </button>
                </div>
            }
            .into_any(),
        )
    };
    // Once every suggestion is taken, the box goes; an answer with none
    // stays to say so.
    let open = move || {
        let left = suggestions.with(|found| {
            found.as_ref().is_some_and(|found| {
                let none = found.tags.is_empty() && found.rating.is_none();
                let rated = found.rating.is_some_and(|r| r != rating.get());
                none || rated || text.with(|t| found.tags.iter().any(|s| !has_base(t, &s.name)))
            })
        });
        suggesting.get() || left || error.with(Option::is_some)
    };
    view! {
        <Show when=open>
            <div class="suggested" class:asking=move || suggesting.get() role="status">
                <span class="suggested-title">
                    <Ico icon=Icon::Sparkle size=16 />
                    {move || if suggesting.get() { t!("ai-suggesting") } else { t!("ai-suggested") }}
                </span>
                {move || {
                    error
                        .get()
                        .filter(|_| !suggesting.get())
                        .map(|message| {
                            view! {
                                <ErrorBanner message=message on_retry=on_retry />
                                <div class="suggested-actions">
                                    <button
                                        type="button"
                                        class="btn text small"
                                        on:click=move |_| error.set(None)
                                    >
                                        {move || t!("ai-dismiss")}
                                    </button>
                                </div>
                            }
                        })
                }}
                {answer}
            </div>
        </Show>
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
