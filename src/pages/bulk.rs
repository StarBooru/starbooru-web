//! Mass edits in the gallery: the bar a selection brings up at the foot of
//! the window, with the count, select all, and what can be done to every
//! post picked: tags and a rating from a sheet, a link to share them all, or
//! the recycle bin with Undo. A report follows in the snackbar, and the
//! gallery loads again.

use std::sync::Arc;

use leptos::ev::{KeyboardEvent, SubmitEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{BulkEdit, Post, Rating};
use wasm_bindgen::JsCast;

use super::share::ShareSheet;
use super::{bad_tag_text, gallery, post, trash};
use crate::api;
use crate::components::icons::{Ico, Icon};
use crate::components::rating::own_rating_choices;
use crate::components::select::Select;
use crate::components::snackbar::{notify, notify_undo};
use crate::components::tag_box::TagBox;
use crate::dom;
use crate::logic::bulk::{
    Done, EditProblem, Report, Selection, Skipped, bulk_delete, bulk_edit, in_gallery_order,
    live_ones, report,
};

/// What to say after an edit.
fn report_text(r: Report) -> String {
    let done = match r.done {
        Done::Moved(count) => t!("bulk-moved", count = count),
        Done::Updated(count) => t!("bulk-updated", count = count),
    };
    match r.skipped {
        None => t!("bulk-done", done = done),
        Some(Skipped::Undeletable(count)) => t!(
            "bulk-done-skipped",
            done = done,
            skipped = t!("bulk-skipped-undeletable", count = count)
        ),
        Some(Skipped::Left(count)) => t!(
            "bulk-done-skipped",
            done = done,
            skipped = t!("bulk-skipped", count = count)
        ),
    }
}

/// Why the sheet can't send its edit.
fn problem_text(problem: &EditProblem) -> String {
    match problem {
        EditProblem::Bad(bad) => bad_tag_text(bad),
        EditProblem::Both(tag) => t!("bulk-tag-both", tag = tag.clone()),
        EditProblem::Nothing => t!("bulk-nothing"),
    }
}

/// Posts `ids` changed: the copies the app keeps of them, and of the lists
/// and tags they were in, are out of date.
fn changed(ids: &[i64]) {
    post::forget_kept(ids);
    api::forget_tags();
    gallery::forget_lists();
}

/// The bar of a selection. `posts` is what the gallery has loaded, in its
/// order; `on_close` ends the selection.
#[component]
pub fn BulkBar(
    selection: RwSignal<Selection>,
    posts: RwSignal<Vec<Arc<Post>>>,
    on_close: Callback<()>,
) -> impl IntoView {
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let sheet = RwSignal::new(false);
    // The posts the share sheet is open for, in the gallery's order.
    let sharing = RwSignal::new(None::<Vec<i64>>);
    // What the sheet holds, kept while it's closed and opened again.
    let add = RwSignal::new(String::new());
    let remove = RwSignal::new(String::new());
    let rating = RwSignal::new(None::<Rating>);
    let count = Memo::new(move |_| selection.with(Selection::len));
    let loaded = move || posts.with_untracked(|p| p.iter().map(|p| p.id).collect::<Vec<_>>());
    let full = Memo::new(move |_| {
        let ids: Vec<i64> = posts.with(|p| p.iter().map(|p| p.id).collect());
        selection.with(|s| s.is_full(&ids))
    });
    let picked = move || selection.with_untracked(|s| in_gallery_order(&loaded(), s.ids()));

    // Escape ends the selection, unless something on top of the bar has it.
    let keys = web_sys::window().map(|window| {
        dom::listen(&window, "keydown", false, move |e| {
            let Some(key) = e.dyn_ref::<web_sys::KeyboardEvent>().map(|k| k.key()) else {
                return;
            };
            if key == "Escape"
                && !e.default_prevented()
                && !dom::typing(e.target())
                && sheet.try_get_untracked() == Some(false)
                && sharing.try_with_untracked(Option::is_none) == Some(true)
            {
                on_close.try_run(());
            }
        })
    });
    on_cleanup(move || {
        // Escape ends it from inside this very listener, which can't be
        // dropped while it runs.
        set_timeout(
            move || {
                if let Some(keys) = keys {
                    keys.stop();
                }
            },
            std::time::Duration::ZERO,
        );
    });

    let send = move |edit: BulkEdit, undo: Vec<i64>| {
        let deleting = edit.delete;
        let ids = edit.ids.clone();
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            let result = api::call(move |c| async move { c.bulk_edit(&edit).await }).await;
            busy.try_set(false);
            match result {
                Ok(done) => {
                    let message = report_text(report(done, deleting));
                    // The bar goes before the gallery loads again.
                    on_close.try_run(());
                    changed(&ids);
                    if deleting && done.updated > 0 && !undo.is_empty() {
                        notify_undo(message, t!("press-undo"), move || {
                            trash::undo_delete(undo.clone());
                        });
                    } else {
                        notify(message);
                    }
                }
                Err(message) => {
                    let message = if deleting {
                        t!("bulk-delete-failed", reason = message)
                    } else {
                        message
                    };
                    error.try_set(Some(message));
                }
            }
        });
    };
    let apply = Callback::new(move |()| {
        if busy.get_untracked() {
            return;
        }
        let edit = bulk_edit(
            picked(),
            &add.get_untracked(),
            &remove.get_untracked(),
            rating.get_untracked(),
        );
        match edit {
            Ok(edit) => send(edit, Vec::new()),
            Err(problem) => error.set(Some(problem_text(&problem))),
        }
    });
    let delete = move || {
        if busy.get_untracked() {
            return;
        }
        let ids = picked();
        if ids.is_empty() {
            return;
        }
        // Posts in the bin already stay there on Undo.
        let undo = posts.with_untracked(|p| {
            live_ones(&ids, |id| {
                p.iter().any(|post| post.id == id && post.deleted)
            })
        });
        send(bulk_delete(ids), undo);
    };
    // A link shows only live posts: any picked from the bin stay out of it.
    let share = move || {
        let ids = picked();
        let live = posts.with_untracked(|p| {
            live_ones(&ids, |id| {
                p.iter().any(|post| post.id == id && post.deleted)
            })
        });
        error.set(None);
        sharing.set(Some(live));
    };

    let idle = move || busy.get() || count.get() == 0;
    view! {
        <div class="bulk-bar glass" role="region" aria-label=move || t!("bulk-label")>
            <div class="bulk-count">
                <button
                    type="button"
                    class="icon-btn bulk-close"
                    title=move || t!("gallery-stop-selecting")
                    aria-label=move || t!("gallery-stop-selecting")
                    on:click=move |_| on_close.run(())
                >
                    <Ico icon=Icon::Close />
                </button>
                <strong role="status">
                    {move || {
                        if count.get() == 0 {
                            t!("gallery-select")
                        } else {
                            t!("bulk-selected", count = count.get())
                        }
                    }}
                </strong>
                <button
                    type="button"
                    class="btn text small"
                    disabled=move || posts.with(Vec::is_empty)
                    on:click=move |_| {
                        let ids = loaded();
                        selection.update(|s| s.toggle_all(&ids));
                    }
                >
                    {move || if full.get() { t!("bulk-select-none") } else { t!("bulk-select-all") }}
                </button>
            </div>
            <button
                type="button"
                class="btn bulk-edit"
                aria-haspopup="dialog"
                title=move || t!("bulk-edit-title")
                disabled=idle
                on:click=move |_| {
                    error.set(None);
                    sheet.set(true);
                }
            >
                <Ico icon=Icon::Edit />
                {move || t!("post-edit")}
            </button>
            <button
                type="button"
                class="btn tonal bulk-share"
                aria-haspopup="dialog"
                disabled=idle
                title=move || t!("share-button")
                aria-label=move || t!("share-button")
                on:click=move |_| share()
            >
                <Ico icon=Icon::Share />
                <span class="bulk-label">{move || t!("share-button")}</span>
            </button>
            <button
                type="button"
                class="btn tonal bulk-delete"
                disabled=idle
                title=move || t!("post-delete-title")
                aria-label=move || t!("post-delete-title")
                on:click=move |_| delete()
            >
                <Ico icon=Icon::Delete />
                <span class="bulk-label">{move || t!("post-delete-title")}</span>
            </button>
            {move || {
                error
                    .get()
                    .filter(|_| !sheet.get())
                    .map(|m| view! { <p class="bulk-error break" role="alert">{m}</p> })
            }}
        </div>
        <Show when=move || sheet.get()>
            <BulkSheet
                count=count
                add=add
                remove=remove
                rating=rating
                busy=busy
                error=error
                on_apply=apply
                on_close=Callback::new(move |()| sheet.set(false))
            />
        </Show>
        {move || {
            sharing
                .get()
                .map(|ids| {
                    view! {
                        <ShareSheet post_ids=ids on_close=Callback::new(move |()| sharing.set(None)) />
                    }
                })
        }}
    }
}

/// The tags to add and to remove, and the rating, for every post picked.
#[component]
fn BulkSheet(
    count: Memo<usize>,
    add: RwSignal<String>,
    remove: RwSignal<String>,
    rating: RwSignal<Option<Rating>>,
    busy: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    on_apply: Callback<()>,
    on_close: Callback<()>,
) -> impl IntoView {
    let first: NodeRef<html::Textarea> = NodeRef::new();
    first.on_load(|el| {
        request_animation_frame(move || {
            let _ = el.focus();
        });
    });
    // A mended word takes the complaint away.
    Effect::watch(
        move || (add.get(), remove.get(), rating.get()),
        move |_, _, _| {
            if error.with_untracked(Option::is_some) {
                error.set(None);
            }
        },
        false,
    );
    let choices = Signal::derive(|| own_rating_choices(t!("bulk-keep-rating")));
    view! {
        <div
            class="sheet-scrim"
            on:click=move |_| on_close.run(())
            on:keydown=move |e: KeyboardEvent| {
                if e.key() == "Escape" {
                    e.stop_propagation();
                    on_close.run(());
                }
            }
        >
            <form
                class="card sheet bulk-sheet"
                role="dialog"
                aria-modal="true"
                aria-labelledby="bulk-sheet-title"
                on:click=|e| e.stop_propagation()
                on:submit=move |e: SubmitEvent| {
                    e.prevent_default();
                    on_apply.run(());
                }
            >
                <div class="sheet-head">
                    <h2 id="bulk-sheet-title" class="card-title">
                        {move || t!("bulk-sheet-title", count = count.get())}
                    </h2>
                    <button
                        type="button"
                        class="icon-btn"
                        aria-label=move || t!("cancel")
                        title=move || t!("cancel")
                        on:click=move |_| on_close.run(())
                    >
                        <Ico icon=Icon::Close />
                    </button>
                </div>
                <div class="field">
                    <label for="bulk-add">{move || t!("bulk-add-tags")}</label>
                    <TagBox id="bulk-add" text=add rows=2 area=first />
                </div>
                <div class="field">
                    <label for="bulk-remove">{move || t!("bulk-remove-tags")}</label>
                    <TagBox id="bulk-remove" text=remove rows=2 />
                </div>
                <p class="muted hint">{move || t!("post-tags-hint")}</p>
                // Not a `label`: a click on the list would press the button in it
                // again. The list opens upwards, over the boxes, inside the sheet.
                <div class="field">
                    <span>{move || t!("rating")}</span>
                    <Select
                        class="boxed bulk-sheet-rating"
                        value=Signal::derive(move || {
                            rating.get().map(|r| r.as_str().to_owned()).unwrap_or_default()
                        })
                        options=choices
                        on_change=Callback::new(move |v: String| rating.set(Rating::parse(&v)))
                        label=Signal::derive(|| t!("rating"))
                    />
                </div>
                {move || {
                    error.get().map(|m| view! { <p class="field-error break" role="alert">{m}</p> })
                }}
                <div class="row-end">
                    <button class="btn text" type="button" on:click=move |_| on_close.run(())>
                        {move || t!("cancel")}
                    </button>
                    <button class="btn" type="submit" disabled=move || busy.get()>
                        <Ico icon=Icon::Check />
                        {move || t!("bulk-apply")}
                    </button>
                </div>
            </form>
        </div>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_bar_is_glass_and_the_sheets_list_opens_upwards() {
        let source = include_str!("bulk.rs");
        assert!(source.contains(r#"class="bulk-bar glass""#));
        assert!(source.contains(r#"class="card sheet bulk-sheet""#));
        let css = include_str!("../../style/pages/gallery.css");
        assert!(css.contains(".bulk-sheet .select-list { top: auto; bottom: calc(100% + 6px);"));
        // The last tiles aren't left under the bar.
        assert!(css.contains(".masonry-box.selecting { padding-bottom:"));
    }
}
