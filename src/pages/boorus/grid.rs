//! The booru browser's tiles and the bar that saves a selection. A tile is
//! the gallery's, with the site it's on and how saving it here stands.

use std::sync::Arc;

use leptos::ev::{KeyboardEvent, MouseEvent, SubmitEvent};
use leptos::prelude::*;
use starbooru_core::RemotePost;

use wasm_bindgen::JsCast;

use crate::components::icons::{Ico, Icon};
use crate::components::thumb::place_style;
use crate::logic::boorus::results::{Picks, Status, picture_src, status_note};
use crate::logic::boorus::site_name;
use crate::logic::layout::Place;
use crate::{api, dom};

/// A picture's address as the browser may load it, or `None` for the
/// tile's colour.
pub(super) fn src_of(url: &str) -> Option<String> {
    let server = api::client().map(|c| c.base_url().to_owned());
    picture_src(url, server.as_deref().unwrap_or_default())
}

/// How saving a post here stands, as a short mark on its tile: on its way,
/// in the collection, or failed. The tile's description says it in full;
/// the post's own page links to the post here.
#[component]
fn StatusMark(#[prop(into)] status: Signal<Option<Status>>) -> impl IntoView {
    move || {
        let view = match status.get()? {
            Status::Busy(percent) => {
                let shown = percent.map_or_else(|| "…".to_owned(), |p| format!("{p}%"));
                view! {
                    <span class="pill busy remote-status" aria-hidden="true" title=move || t!("upload-job-running")>
                        {shown}
                    </span>
                }
                .into_any()
            }
            Status::Saved(Some(id), _) => view! {
                <span class="remote-mark" aria-hidden="true" title=move || t!("post-number", id = id)>
                    <Ico icon=Icon::Check size=18 />
                </span>
            }
            .into_any(),
            Status::Saved(None, note) => {
                let title = if note.is_empty() {
                    t!("boorus-known")
                } else {
                    note
                };
                view! {
                    <span class="remote-mark" aria-hidden="true" title=title>
                        <Ico icon=Icon::Check size=18 />
                    </span>
                }
                .into_any()
            }
            Status::Failed(message) => view! {
                <span class="pill bad remote-status" aria-hidden="true" title=message>
                    {move || t!("upload-failed")}
                </span>
            }
            .into_any(),
        };
        Some(view)
    }
}

/// A result in the grid. A click opens its page (`on_open` hears of it
/// first); while `selecting`, it picks it instead, unless it's saved or on
/// its way. `still` skips the entrance.
#[component]
pub(super) fn RemoteThumb(
    post: Arc<RemotePost>,
    index: usize,
    #[prop(into)] href: Signal<String>,
    #[prop(into)] place: Signal<Option<Place>>,
    still: bool,
    /// Whether more than one site is searched, so each tile names its own.
    #[prop(into)]
    many: Signal<bool>,
    #[prop(into)] status: Signal<Option<Status>>,
    #[prop(into)] selecting: Signal<bool>,
    #[prop(into)] selected: Signal<bool>,
    on_open: Callback<()>,
    on_toggle: Callback<(usize, bool)>,
) -> impl IntoView {
    let pickable = move || selecting.get() && status.with(Option::is_none);
    let click = move |e: MouseEvent| {
        if selecting.get_untracked() {
            e.prevent_default();
            on_toggle.run((index, e.shift_key()));
            return;
        }
        // With a modifier the browser opens the link elsewhere.
        if e.button() == 0 && !(e.ctrl_key() || e.meta_key() || e.shift_key() || e.alt_key()) {
            on_open.run(());
        }
    };
    let key = move |e: KeyboardEvent| {
        if selecting.get_untracked() && e.key() == " " {
            e.prevent_default();
            on_toggle.run((index, e.shift_key()));
        }
    };
    let loaded = RwSignal::new(false);
    let delay = (index % 40) * 16;
    let style = move || {
        let mut style = format!("--delay: {delay}ms");
        if let Some(p) = place.get() {
            style.push_str(&place_style(p));
        }
        style
    };
    let alt = post
        .tags
        .iter()
        .take(8)
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    let label = {
        let (alt, id) = (alt.clone(), post.id);
        move || {
            if alt.is_empty() {
                t!("post-alt", id = id)
            } else {
                alt.clone()
            }
        }
    };
    let picture = match src_of(&post.preview_url) {
        Some(src) => view! {
            <img
                src=src
                alt=alt
                loading="lazy"
                decoding="async"
                referrerpolicy="no-referrer"
                on:load=move |_| loaded.set(true)
            />
        }
        .into_any(),
        None => view! {
            <div class="thumb-placeholder">
                <Ico icon=Icon::Gallery size=40 />
            </div>
        }
        .into_any(),
    };
    let site = site_name(&post.site);
    let video = post.is_video();
    // How saving it stands, read with the tile; the marks on it are short.
    let note_id = format!("booru-tile-note-{index}");
    let described = {
        let note_id = note_id.clone();
        move || status.with(Option::is_some).then(|| note_id.clone())
    };
    let note = move || status.with(|s| s.as_ref().map(status_note));
    view! {
        <a
            class="thumb"
            class:still=still
            class:loaded=loaded
            class:selectable=pickable
            class:on=move || pickable() && selected.get()
            style=style
            href=move || href.get()
            aria-label=label
            aria-describedby=described
            role=move || pickable().then_some("button")
            aria-pressed=move || pickable().then(|| if selected.get() { "true" } else { "false" })
            on:click=click
            on:keydown=key
            // Shift+click picks a range, not the text in between.
            on:mousedown=move |e: MouseEvent| {
                if e.shift_key() && selecting.get_untracked() {
                    e.prevent_default();
                }
            }
        >
            <div class="thumb-media">{picture}</div>
            <Show when=move || many.get()>
                <span class="badge remote-site">{site.clone()}</span>
            </Show>
            {video.then(|| view! {
                <span class="badge">
                    <Ico icon=Icon::Play size=16 />
                </span>
            })}
            <StatusMark status=status />
            <span id=note_id class="visually-hidden">{note}</span>
            <Show when=pickable>
                <span class="check-mark" aria-hidden="true">
                    <Ico icon=Icon::Check size=20 />
                </span>
            </Show>
        </a>
    }
}

/// The bar of a selection: tags to add to every picked post, and the button
/// that saves them here. `free` is what can be picked.
#[component]
pub(super) fn SaveBar(
    picks: RwSignal<Picks>,
    free: Memo<Vec<String>>,
    busy: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    on_save: Callback<String>,
    on_close: Callback<()>,
) -> impl IntoView {
    let extra = RwSignal::new(String::new());
    // Only what the grid still shows and can still save.
    let count = Memo::new(move |_| free.with(|f| picks.with(|p| p.count_in(f))));
    // Escape ends the selection, unless the sheet of sites or a box has it.
    let keys = web_sys::window().map(|window| {
        dom::listen(&window, "keydown", false, move |e| {
            let escape = e
                .dyn_ref::<web_sys::KeyboardEvent>()
                .is_some_and(|k| k.key() == "Escape");
            let sheet_open = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.query_selector(".sites-sheet").ok().flatten())
                .is_some();
            if escape && !e.default_prevented() && !dom::typing(e.target()) && !sheet_open {
                on_close.try_run(());
            }
        })
    });
    on_cleanup(move || {
        set_timeout(
            move || {
                if let Some(keys) = keys {
                    keys.stop();
                }
            },
            std::time::Duration::ZERO,
        );
    });
    let all = Memo::new(move |_| free.with(|f| picks.with(|p| p.all_of(f))));
    view! {
        <form
            class="bulk-bar glass save-bar"
            aria-label=move || t!("boorus-save-label")
            on:submit=move |e: SubmitEvent| {
                e.prevent_default();
                on_save.run(extra.get_untracked());
            }
        >
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
                    disabled=move || free.with(Vec::is_empty)
                    on:click=move |_| {
                        let free = free.get_untracked();
                        picks.update(|p| p.toggle_all(&free));
                    }
                >
                    {move || if all.get() { t!("bulk-select-none") } else { t!("bulk-select-all") }}
                </button>
            </div>
            <input
                class="bulk-tags"
                type="text"
                placeholder=move || t!("boorus-extra-tags")
                aria-label=move || t!("boorus-extra-tags")
                autocapitalize="none"
                autocomplete="off"
                spellcheck="false"
                prop:value=move || extra.get()
                on:input=move |e| {
                    extra.set(event_target_value(&e));
                    if error.with_untracked(Option::is_some) {
                        error.set(None);
                    }
                }
            />
            <button class="btn" type="submit" disabled=move || busy.get() || count.get() == 0>
                <Ico icon=Icon::Download />
                {move || {
                    if busy.get() { t!("boorus-saving") } else { t!("boorus-save", count = count.get()) }
                }}
            </button>
            {move || error.get().map(|m| view! { <p class="bulk-error break" role="alert">{m}</p> })}
        </form>
    }
}
