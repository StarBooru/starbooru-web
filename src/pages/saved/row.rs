//! Saved searches on the page: one row over the grid, the ones used last
//! first, ending in the way to all of them; and that sheet, with a filter,
//! the sort, and each one's rename, move and remove. Each label leads with
//! what sets a search apart from the ones like it (`logic::saved::labels`).

use leptos::ev::{DragEvent, KeyboardEvent, SubmitEvent};
use leptos::html;
use leptos::prelude::*;
use starbooru_core::saved::MAX_SEARCH_NAME_CHARS;
use wasm_bindgen::JsCast;

use super::{
    MOVED, SHEET, arranged, close_sheet, forget, href, open_sheet, rename_to, reorder, shown_ids,
    used,
};
use crate::components::focus::hold_focus;
use crate::components::icons::{Ico, Icon};
use crate::components::menu::{self, ActionMenu, MenuAt, MenuItem};
use crate::components::preview::SearchPreview;
use crate::components::select::{Choice, Select};
use crate::dom;
use crate::logic::boorus::site_name;
use crate::logic::saved::{IN_ROW, Item, Kind, Label, Sort, landing, matches, move_to, step};
use crate::store;

fn sort_label(sort: Sort) -> String {
    match sort {
        Sort::Recent => t!("saved-sort-recent"),
        Sort::Newest => t!("saved-sort-newest"),
        Sort::Name => t!("saved-sort-name"),
        Sort::Mine => t!("saved-sort-mine"),
    }
}

/// Saved search `id` of `kind` was just moved.
fn moved(kind: Kind, id: i64) -> bool {
    MOVED.with(|m| m.get()) == Some((kind, id))
}

/// What a saved search finds, as a picture: the gallery's first post, or
/// for a booru search its first site's initials, which ask nothing.
fn picture(item: &Item) -> AnyView {
    match item.kind {
        Kind::Gallery => view! { <SearchPreview q=item.query.clone() /> }.into_any(),
        Kind::Boorus => {
            let first = item
                .sites
                .first()
                .map(|s| site_name(&s.base))
                .unwrap_or_default();
            let initials: String = first.chars().take(2).collect();
            let more = item.sites.len().saturating_sub(1);
            view! {
                <span class="preview site-tile" aria-hidden="true">
                    {initials}
                    {(more > 0).then(|| view! { <span class="more">{format!("+{more}")}</span> })}
                </span>
            }
            .into_any()
        }
    }
}

/// A saved search's label: what sets it apart, then what it shares, dimmed.
fn label_view(label: &Label) -> AnyView {
    let rest = (!label.rest.is_empty()).then(|| {
        view! { <span class="label-rest">{label.rest.clone()}</span> }
    });
    view! {
        <span class="label-lead">{label.lead.clone()}</span>
        {rest}
    }
    .into_any()
}

/// Its full search, and the sites for a booru search, for a tooltip.
fn full(item: &Item) -> String {
    let what = if item.query.trim().is_empty() {
        item.name.clone()
    } else {
        item.query.clone()
    };
    match item.kind {
        Kind::Gallery => what,
        Kind::Boorus => format!("{what} · {}", item.site_names()),
    }
}

/// What tells a saved search apart, as the sheet and the search box show
/// it: its picture, its label, and its sites or its search.
#[component]
pub fn SavedLine(item: Item, label: Label) -> impl IntoView {
    let detail = match item.kind {
        Kind::Boorus => Some(item.site_names()),
        Kind::Gallery => item.named().then(|| item.query.clone()),
    }
    // Not twice, as for a booru search named after its sites.
    .filter(|d| *d != label.lead);
    view! {
        {picture(&item)}
        <span class="saved-text">
            <span class="saved-label">{label_view(&label)}</span>
            {detail.map(|d| view! { <span class="saved-detail">{d}</span> })}
        </span>
    }
}

/// The saved searches of `kind` as one row over the grid: the first ones in
/// the order chosen, then the way to all of them.
#[component]
pub fn SavedRow(kind: Kind) -> impl IntoView {
    let list = Memo::new(move |_| arranged(kind).unwrap_or_default());
    let first =
        move || -> Vec<(Item, Label)> { list.with(|l| l.iter().take(IN_ROW).cloned().collect()) };
    let pill = move |(item, label): (Item, Label)| {
        let id = item.id;
        let opened = item.clone();
        view! {
            <li class="saved-pill" class:moved=move || moved(kind, id)>
                <a href=href(&item) title=full(&item) on:click=move |_| used(&opened)>
                    {picture(&item)}
                    <span class="saved-label">{label_view(&label)}</span>
                </a>
            </li>
        }
    };
    let count = move || list.with(Vec::len);
    view! {
        <Show when=move || list.with(|l| !l.is_empty())>
            <nav class="saved-row" aria-label=move || t!("saved-searches")>
                <ul class="saved-strip">
                    <For
                        each=first
                        key=|(s, l)| (s.id, l.lead.clone(), l.rest.clone())
                        children=pill
                    />
                </ul>
                <button
                    type="button"
                    class="saved-all"
                    aria-haspopup="dialog"
                    title=move || t!("saved-all")
                    on:click=move |_| open_sheet(kind)
                >
                    <span class="saved-all-label">{move || t!("saved-all-short")}</span>
                    <span class="badge">{count}</span>
                    <Ico icon=Icon::ChevronRight size=18 />
                </button>
            </nav>
        </Show>
    }
}

/// The sheet with every saved search of the kind asked for, over any page.
#[component]
pub fn SavedSheet() -> impl IntoView {
    let open = Memo::new(|_| SHEET.with(|s| s.get()));
    move || open.get().map(|kind| view! { <Sheet kind=kind /> })
}

/// Puts the focus on saved search `id`'s menu button in the sheet.
fn focus_more(id: i64) {
    request_animation_frame(move || {
        let button = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| {
                d.query_selector(&format!(r#".saved-item[data-id="{id}"] .saved-more"#))
                    .ok()
                    .flatten()
            })
            .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok());
        if let Some(button) = button {
            let _ = button.focus();
        }
    });
}

#[component]
fn Sheet(kind: Kind) -> impl IntoView {
    let filter = RwSignal::new(String::new());
    let list = Memo::new(move |_| arranged(kind).unwrap_or_default());
    let shown = Memo::new(move |_| {
        filter.with(|f| {
            list.with(|l| {
                l.iter()
                    .filter(|(s, _)| matches(s, f))
                    .cloned()
                    .collect::<Vec<_>>()
            })
        })
    });
    let sort = Memo::new(|_| store::prefs().saved.sort);
    // Moving only makes sense over the whole list; the menu's moves switch
    // to "My order", and in it a mouse can drag them by their grips.
    let movable = Memo::new(move |_| filter.with(|f| f.trim().is_empty()));
    let draggable = Memo::new(move |_| movable.get() && sort.get() == Sort::Mine);
    let menu_at = RwSignal::new(None::<(Item, MenuAt)>);
    let renaming = RwSignal::new(None::<i64>);
    // The search being dragged, from what place, and where it would land:
    // before the one at that place.
    let dragging = RwSignal::new(None::<(i64, usize)>);
    let over = RwSignal::new(None::<usize>);
    let dialog: NodeRef<html::Div> = NodeRef::new();
    let find_box: NodeRef<html::Input> = NodeRef::new();
    let alive = StoredValue::new(true);

    hold_focus(dialog, true);
    Effect::new(move |_| {
        if let Some(el) = find_box.get() {
            let _ = el.focus();
        }
    });
    // Escape closes it wherever the focus is, unless something in it took
    // the key first (a menu, the name being changed).
    let keys = web_sys::window().map(|window| {
        dom::listen(&window, "keydown", false, move |e| {
            let escape = e
                .dyn_ref::<web_sys::KeyboardEvent>()
                .is_some_and(|k| k.key() == "Escape");
            if escape && !e.default_prevented() && alive.try_get_value() == Some(true) {
                e.prevent_default();
                close_sheet();
            }
        })
    });
    on_cleanup(move || {
        // It closes from inside this very listener, which can't be dropped
        // while it runs.
        set_timeout(
            move || {
                if let Some(keys) = keys {
                    keys.stop();
                }
            },
            std::time::Duration::ZERO,
        );
    });
    // Leaving for a search closes it.
    let location = leptos_router::hooks::use_location();
    Effect::watch(
        move || (location.pathname.get(), location.search.get()),
        move |_, _, _| close_sheet(),
        false,
    );

    let drop_here = move || {
        let (Some((id, from)), Some(before)) = (dragging.get_untracked(), over.get_untracked())
        else {
            return;
        };
        dragging.set(None);
        over.set(None);
        let item =
            list.with_untracked(|l| l.iter().find(|(s, _)| s.id == id).map(|(s, _)| s.clone()));
        if let Some(item) = item {
            reorder(&item, move_to(&shown_ids(kind), id, landing(from, before)));
        }
    };
    let row = move |(item, label): (Item, Label)| {
        let id = item.id;
        let at = move || shown.with(|l| l.iter().position(|(s, _)| s.id == id).unwrap_or(0));
        let more_label = t!("saved-more", name = item.name.clone());
        let for_menu = item.clone();
        let main = {
            let item = item.clone();
            move || {
                if renaming.get() == Some(id) {
                    view! { <RenameBox item=item.clone() renaming=renaming /> }.into_any()
                } else {
                    let opened = item.clone();
                    view! {
                        <a
                            class="saved-open"
                            href=href(&item)
                            title=full(&item)
                            on:click=move |_| used(&opened)
                        >
                            <SavedLine item=item.clone() label=label.clone() />
                        </a>
                    }
                    .into_any()
                }
            }
        };
        view! {
            <li
                class="saved-item"
                data-id=id
                class:dragging=move || dragging.get().is_some_and(|(d, _)| d == id)
                class:drop-before=move || over.get() == Some(at())
                class:drop-after=move || {
                    over.get() == Some(at() + 1) && at() + 1 == shown.with(Vec::len)
                }
                class:moved=move || moved(kind, id)
                draggable=move || {
                    if draggable.get() && renaming.get() != Some(id) { "true" } else { "false" }
                }
                on:dragstart=move |e: DragEvent| {
                    if let Some(data) = e.data_transfer() {
                        let _ = data.set_data("text/plain", &for_menu.query);
                        data.set_effect_allowed("move");
                    }
                    dragging.set(Some((id, at())));
                }
                on:dragover=move |e: DragEvent| {
                    if dragging.get_untracked().is_none() {
                        return;
                    }
                    e.prevent_default();
                    // The nearer half says before or after.
                    let after = e
                        .current_target()
                        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                        .is_some_and(|el| {
                            let r = el.get_bounding_client_rect();
                            f64::from(e.client_y()) > r.top() + r.height() / 2.0
                        });
                    let before = if after { at() + 1 } else { at() };
                    if over.get_untracked() != Some(before) {
                        over.set(Some(before));
                    }
                }
                on:drop=move |e: DragEvent| {
                    e.prevent_default();
                    drop_here();
                }
                on:dragend=move |_| {
                    dragging.set(None);
                    over.set(None);
                }
            >
                {main}
                // Beside ⋮, which stays at the end whether it's there or not.
                <Show when=move || draggable.get() && renaming.get() != Some(id)>
                    <span class="saved-grip" title=move || t!("saved-drag") aria-hidden="true"></span>
                </Show>
                <button
                    type="button"
                    class="icon-btn saved-more"
                    aria-haspopup="menu"
                    title=more_label.clone()
                    aria-label=more_label
                    on:click={
                        let item = item.clone();
                        move |e| {
                            if let Some(el) = e
                                .current_target()
                                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                            {
                                menu_at.set(Some((item.clone(), menu::beside(&el))));
                            }
                        }
                    }
                >
                    <Ico icon=Icon::More />
                </button>
            </li>
        }
    };
    let item_menu = move || {
        let (item, at) = menu_at.get()?;
        let close = Callback::new(move |()| menu_at.set(None));
        let ids = shown_ids(kind);
        let id = item.id;
        let first = ids.first() == Some(&id);
        let last = ids.last() == Some(&id);
        let can_move = movable.get_untracked();
        let (up, down, gone) = (item.clone(), item.clone(), item.clone());
        Some(view! {
            <ActionMenu at=at title=item.name.clone() on_close=close>
                <MenuItem
                    icon=Icon::Edit
                    label=Signal::derive(move || t!("saved-rename"))
                    on_click=Callback::new(move |()| {
                        menu_at.set(None);
                        renaming.set(Some(id));
                    })
                />
                {(can_move && !first)
                    .then(|| {
                        view! {
                            <MenuItem
                                icon=Icon::ArrowUp
                                label=Signal::derive(move || t!("saved-move-earlier"))
                                on_click=Callback::new(move |()| {
                                    menu_at.set(None);
                                    reorder(&up, step(&shown_ids(kind), id, false));
                                    focus_more(id);
                                })
                            />
                        }
                    })}
                {(can_move && !last)
                    .then(|| {
                        view! {
                            <MenuItem
                                icon=Icon::ArrowDown
                                label=Signal::derive(move || t!("saved-move-later"))
                                on_click=Callback::new(move |()| {
                                    menu_at.set(None);
                                    reorder(&down, step(&shown_ids(kind), id, true));
                                    focus_more(id);
                                })
                            />
                        }
                    })}
                <MenuItem
                    icon=Icon::Delete
                    danger=true
                    label=Signal::derive(move || t!("saved-remove"))
                    on_click=Callback::new(move |()| {
                        // The focus goes on to the search beside it.
                        let ids: Vec<i64> = shown.with_untracked(|l| l.iter().map(|(s, _)| s.id).collect());
                        let at = ids.iter().position(|i| *i == id).unwrap_or(0);
                        let near = ids
                            .get(at + 1)
                            .or_else(|| at.checked_sub(1).and_then(|b| ids.get(b)))
                            .copied();
                        menu_at.set(None);
                        forget(gone.clone());
                        if let Some(near) = near {
                            focus_more(near);
                        }
                    })
                />
            </ActionMenu>
        })
    };
    let sorts = Signal::derive(|| {
        Sort::ALL
            .into_iter()
            .map(|s| Choice::new(s.as_str(), sort_label(s)))
            .collect::<Vec<_>>()
    });
    let title = move || match kind {
        Kind::Gallery => t!("saved-searches"),
        Kind::Boorus => t!("saved-booru-searches"),
    };
    let none = move || shown.with(Vec::is_empty) && list.with(|l| !l.is_empty());
    view! {
        <div class="sheet-scrim saved-scrim" on:click=move |_| close_sheet()>
            <div
                class="card sheet saved-sheet"
                role="dialog"
                aria-modal="true"
                aria-labelledby="saved-sheet-title"
                node_ref=dialog
                on:click=|e| e.stop_propagation()
            >
                <div class="sheet-head">
                    <h2 id="saved-sheet-title" class="card-title">
                        {title}
                        <span class="badge">{move || list.with(Vec::len)}</span>
                    </h2>
                    <button
                        type="button"
                        class="icon-btn"
                        aria-label=move || t!("notice-close")
                        title=move || t!("notice-close")
                        on:click=move |_| close_sheet()
                    >
                        <Ico icon=Icon::Close />
                    </button>
                </div>
                <div class="saved-tools">
                    <label class="saved-filter">
                        <Ico icon=Icon::Search size=20 />
                        <input
                            node_ref=find_box
                            type="search"
                            autocomplete="off"
                            spellcheck="false"
                            autocapitalize="none"
                            aria-label=move || t!("saved-filter")
                            placeholder=move || t!("saved-filter")
                            prop:value=move || filter.get()
                            on:input=move |e| filter.set(event_target_value(&e))
                        />
                    </label>
                    <Select
                        class="boxed saved-sort"
                        value=Signal::derive(move || sort.get().as_str().to_owned())
                        options=sorts
                        on_change=Callback::new(move |v: String| {
                            if let Some(s) = Sort::of(&v) {
                                store::update_prefs(|p| p.saved.sort = s);
                            }
                        })
                        label=Signal::derive(|| t!("saved-sort"))
                    />
                </div>
                <Show when=none>
                    <p class="muted saved-none">{move || t!("saved-none-match")}</p>
                </Show>
                <ul
                    class="saved-list"
                    class:sorting=move || dragging.get().is_some()
                    on:dragover=move |e: DragEvent| {
                        if dragging.get_untracked().is_some() {
                            e.prevent_default();
                        }
                    }
                    on:drop=move |e: DragEvent| {
                        e.prevent_default();
                        drop_here();
                    }
                >
                    <For
                        each=move || shown.get()
                        key=|(s, l)| (s.id, s.name.clone(), l.lead.clone(), l.rest.clone())
                        children=row
                    />
                </ul>
                // In the dialog, so its focus is the sheet's.
                {item_menu}
            </div>
        </div>
    }
}

/// The name of a saved search being renamed: Enter or leaving the box
/// renames it, Escape keeps the name.
#[component]
fn RenameBox(item: Item, renaming: RwSignal<Option<i64>>) -> impl IntoView {
    let text = RwSignal::new(item.name.clone());
    let input: NodeRef<html::Input> = NodeRef::new();
    input.on_load(|el| {
        request_animation_frame(move || {
            let _ = el.focus();
            el.select();
        });
    });
    let id = item.id;
    let done = StoredValue::new(false);
    // `refocus` when it ends from the keyboard, not by clicking elsewhere.
    let finish = move |keep: bool, refocus: bool| {
        if done.get_value() {
            return;
        }
        done.set_value(true);
        let typed = text.get_untracked();
        renaming.set(None);
        if !keep {
            rename_to(item.clone(), typed, move |now| {
                if refocus {
                    focus_more(now);
                }
            });
        } else if refocus {
            focus_more(id);
        }
    };
    let finish_key = finish.clone();
    let finish_blur = finish.clone();
    view! {
        <form
            class="saved-rename"
            on:submit=move |e: SubmitEvent| {
                e.prevent_default();
                finish(false, true);
            }
        >
            <input
                node_ref=input
                aria-label=move || t!("saved-name")
                maxlength=MAX_SEARCH_NAME_CHARS.to_string()
                prop:value=move || text.get()
                on:input=move |e| text.set(event_target_value(&e))
                on:keydown=move |e: KeyboardEvent| {
                    if e.key() == "Escape" {
                        e.prevent_default();
                        e.stop_propagation();
                        finish_key(true, true);
                    }
                }
                on:blur=move |_| {
                    if renaming.get_untracked() == Some(id) {
                        finish_blur(false, false);
                    }
                }
            />
        </form>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_sheet_is_a_dialog_that_holds_the_focus() {
        let source = include_str!("row.rs");
        for part in [
            r#"role="dialog""#,
            r#"aria-modal="true""#,
            "hold_focus(dialog, true)",
            r#"aria-haspopup="dialog""#,
        ] {
            assert!(source.contains(part), "{part}");
        }
        // No star in the saved searches' own design.
        assert!(!source.contains(concat!("Icon::", "Star")));
    }
}
