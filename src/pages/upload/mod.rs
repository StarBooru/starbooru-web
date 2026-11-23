//! Upload: files and addresses wait here, as tiles, for their tags; then
//! they go to the app-wide queue (`activity.rs`), which keeps going on other
//! pages. Files come from the pickers, a drop (folders too) or a paste;
//! addresses are typed, pasted or dropped, and the server imports them.

mod drop;
mod sheet;
mod tiles;

use std::thread::LocalKey;
use std::time::Duration;

use leptos::ev::{Event, KeyboardEvent, SubmitEvent};
use leptos::prelude::*;
use leptos::task::spawn_local;
use send_wrapper::SendWrapper;
use starbooru_core::Rating;
use wasm_bindgen::JsCast;

pub use self::drop::DropToUpload;
use self::drop::{Veil, watch_drops};
use self::sheet::ItemSheet;
use self::tiles::{FileTile, JobTile};
use crate::activity;
use crate::api;
use crate::components::icons::{Ico, Icon};
use crate::components::rating::rating_choices;
use crate::components::select::Select;
use crate::components::switch::Switch;
use crate::components::tag_box::TagBox;
use crate::components::transfer::TransferBar;
use crate::dom;
use crate::files::{self, Dropped, Picked};
use crate::logic::queue::{Batch, Queue, Status};
use crate::logic::upload::{folder_tag, parse_urls, skipped_names, tag_problem};
use crate::pages::bad_tag_text;

thread_local! {
    // Kept across pages, like the list they go on.
    static SHARED_TAGS: ArcRwSignal<String> = ArcRwSignal::new(String::new());
    static RATING: ArcRwSignal<Rating> = ArcRwSignal::new(Rating::default());
    static SOURCE: ArcRwSignal<String> = ArcRwSignal::new(String::new());
    /// `None` until the switch is pressed: the server's setting decides.
    static AUTOTAG: ArcRwSignal<Option<bool>> = ArcRwSignal::new(None);
    static FOLDER_TAGS: ArcRwSignal<bool> = ArcRwSignal::new(true);
}

/// One of the values above, as a signal a component can hand on.
fn shared<T: Send + Sync + 'static>(value: &'static LocalKey<ArcRwSignal<T>>) -> RwSignal<T> {
    RwSignal::from(value.with(Clone::clone))
}

/// What goes on everything sent together.
fn batch() -> Batch {
    Batch {
        tags: SHARED_TAGS.with(|t| t.get_untracked()),
        rating: RATING.with(|r| r.get_untracked()),
        source: SOURCE.with(|s| s.get_untracked()),
        autotag: AUTOTAG.with(|a| a.get_untracked()),
    }
}

/// What's wrong with the tags of the drafts `keys` picks, in words.
fn batch_problem(keys: Option<&[u64]>) -> Option<String> {
    let shared = SHARED_TAGS.with(|t| t.get_untracked());
    let own: Vec<String> = activity::with_queue(|q| {
        q.items()
            .iter()
            .filter(|i| i.status == Status::Draft)
            .filter(|i| keys.is_none_or(|keys| keys.contains(&i.key)))
            .map(|i| i.tags.clone())
            .collect()
    });
    tag_problem(&shared, own.iter().map(String::as_str)).map(|bad| bad_tag_text(&bad))
}

/// How many tiles show a picture of their own file. A folder of thousands
/// would otherwise decode every picture in it at once.
const MAX_PREVIEWS: usize = 120;

/// Adds the images and videos among `picked` as drafts, those from a folder
/// with its name as a tag when that's on. Gives their keys, and the names of
/// the files left out.
async fn take_files(picked: Vec<Picked>) -> (Vec<u64>, Vec<String>) {
    let (media, skipped) = files::media_files(picked).await;
    let by_folder = FOLDER_TAGS.with(|f| f.get_untracked());
    let keys = media
        .into_iter()
        .map(|p| {
            let tags = by_folder
                .then(|| folder_tag(&p.path))
                .flatten()
                .unwrap_or_default();
            activity::add_file(p, tags)
        })
        .collect();
    (keys, skipped)
}

/// Adds what a drop carried: its files, those in its folders, and its
/// addresses when the server imports them.
async fn take_drop(dropped: Dropped) -> (Vec<u64>, Vec<String>) {
    let Dropped {
        mut files,
        folders,
        urls,
    } = dropped;
    files.extend(files::walk(folders).await);
    let mut keys = Vec::new();
    if !urls.is_empty() && url_import().await {
        keys.extend(urls.into_iter().map(activity::add_url));
    }
    let (taken, skipped) = take_files(files).await;
    keys.extend(taken);
    (keys, skipped)
}

/// Whether the server imports addresses. Taken as yes while it can't be asked.
async fn url_import() -> bool {
    api::server_info().await.is_none_or(|i| i.url_import)
}

/// The notice for files left out, if any were.
fn skipped_notice(names: &[String]) -> Option<String> {
    (!names.is_empty()).then(|| {
        let (shown, more) = skipped_names(names);
        t!("upload-skipped", names = shown, more = more)
    })
}

/// The files an `<input type=file>` was given. It's emptied, so the same
/// file can be picked again.
fn picked(e: &Event) -> Vec<Picked> {
    let Some(input) = e
        .target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
    else {
        return Vec::new();
    };
    let list = files::listed(input.files());
    input.set_value("");
    list
}

/// Whether a dialog is open over the page, which has the paste and the keys.
fn dialog_open() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.query_selector("[role=dialog]").ok().flatten())
        .is_some()
}

#[component]
pub fn Upload() -> impl IntoView {
    let folder_tags = shared(&FOLDER_TAGS);
    let autotag = shared(&AUTOTAG);
    let linking = RwSignal::new(false);
    let url_text = RwSignal::new(String::new());
    let notice = RwSignal::new(None::<String>);
    // The item whose sheet is open, and where its tile is.
    let opened = RwSignal::new(None::<(u64, (f64, f64))>);
    let info = LocalResource::new(api::server_info);
    let ai = LocalResource::new(api::ai_status);
    // Shown while unknown, as the reference did.
    let show_url = Memo::new(move |_| info.get().flatten().is_none_or(|i| i.url_import));
    // The server's setting, or `None` when it can't tag.
    let ai_tagging =
        Memo::new(move |_| ai.get().flatten().filter(|s| s.tagging).map(|s| s.autotag));
    activity::refresh();

    let take = move |list: Vec<Picked>| {
        spawn_local(async move {
            let (_, skipped) = take_files(list).await;
            notice.try_set(skipped_notice(&skipped));
        });
    };

    // Files dragged over the window: the veil grows out of the drop box.
    let veil = Veil::new();
    let watches = watch_drops(
        move |over| {
            if over {
                veil.show(".up-drop");
            } else {
                veil.hide(".up-drop");
            }
        },
        move |dropped| {
            spawn_local(async move {
                let (_, skipped) = take_drop(dropped).await;
                notice.try_set(skipped_notice(&skipped));
                // Back into the box, where it is now that the tiles are in.
                veil.hide(".up-drop");
            });
        },
    );

    // Ctrl+V: files and pictures on the clipboard, and links. Only here, and
    // not into a field or while the sheet is open.
    let paste = web_sys::window().map(|window| {
        dom::listen(&window, "paste", false, move |e| {
            if dom::typing(e.target()) || dialog_open() {
                return;
            }
            let Some(data) = e
                .dyn_ref::<web_sys::ClipboardEvent>()
                .and_then(web_sys::ClipboardEvent::clipboard_data)
            else {
                return;
            };
            let list = files::listed(data.files());
            let text = [data.get_data("text/uri-list"), data.get_data("text/plain")]
                .into_iter()
                .filter_map(Result::ok)
                .find(|t| !t.trim().is_empty())
                .unwrap_or_default();
            if list.is_empty() && text.trim().is_empty() {
                return;
            }
            e.prevent_default();
            let urls = if show_url.get_untracked() {
                parse_urls(&text)
            } else {
                Vec::new()
            };
            let found = !urls.is_empty();
            for url in urls {
                activity::add_url(url);
            }
            spawn_local(async move {
                let had_files = !list.is_empty();
                let (keys, skipped) = take_files(list).await;
                let said = if !skipped.is_empty() {
                    skipped_notice(&skipped)
                } else if !found && !had_files && keys.is_empty() {
                    Some(t!("upload-paste-nothing"))
                } else {
                    None
                };
                notice.try_set(said);
            });
        })
    });
    let stop = SendWrapper::new((watches, paste));
    on_cleanup(move || {
        let (watches, paste) = stop.take();
        for watch in watches.into_iter().chain(paste) {
            watch.stop();
        }
    });

    let add_urls = move || {
        let found = parse_urls(&url_text.get_untracked());
        if found.is_empty() {
            notice.set(Some(t!("upload-urls-only")));
            return;
        }
        notice.set(None);
        for url in found {
            activity::add_url(url);
        }
        url_text.set(String::new());
    };

    let keys = Memo::new(|_| {
        activity::with_queue(|q| q.items().iter().map(|i| i.key).collect::<Vec<_>>())
    });
    let job_ids =
        Memo::new(|_| activity::with_jobs(|j| j.iter().map(|j| j.id).collect::<Vec<_>>()));
    let drafts = Memo::new(|_| activity::with_queue(Queue::drafts));
    let going = Memo::new(|_| activity::with_queue(|q| q.overall().is_some()));
    let clearable = Memo::new(|_| {
        activity::with_queue(|q| q.items().iter().any(|i| i.status != Status::Sending))
            || activity::with_jobs(|j| j.iter().any(|j| !j.state.is_active()))
    });
    let empty = move || keys.with(Vec::is_empty) && job_ids.with(Vec::is_empty);
    let placed = move || keys.get().into_iter().enumerate().collect::<Vec<_>>();
    let open_item = Callback::new(move |key: u64| {
        let from = dom::offset_from_middle(&format!("[data-up-key=\"{key}\"]")).unwrap_or_default();
        opened.set(Some((key, from)));
    });

    view! {
        <div class="page upload" class:sending=move || { drafts.get() != 0 || going.get() }>
            <div class="page-head">
                <h1 class="page-title">{move || t!("nav-upload")}</h1>
                <label class="icon-btn" title=move || t!("upload-files")>
                    <Ico icon=Icon::Add />
                    <input
                        type="file"
                        multiple
                        class="visually-hidden"
                        aria-label=move || t!("upload-files")
                        on:change=move |e| take(picked(&e))
                    />
                </label>
                <label class="icon-btn" title=move || t!("upload-folder")>
                    <Ico icon=Icon::Folder />
                    <input
                        type="file"
                        multiple
                        prop:webkitdirectory=true
                        class="visually-hidden"
                        aria-label=move || t!("upload-folder")
                        on:change=move |e| take(picked(&e))
                    />
                </label>
                <span class="up-switches">
                    <Switch
                        icon=Icon::Label
                        label=Signal::derive(|| t!("upload-folder-tags"))
                        on=folder_tags
                        on_change=Callback::new(move |on| folder_tags.set(on))
                    />
                    {move || {
                        ai_tagging
                            .get()
                            .map(|server| {
                                view! {
                                    <Switch
                                        icon=Icon::Sparkle
                                        label=Signal::derive(|| t!("upload-autotag"))
                                        on=Signal::derive(move || autotag.get().unwrap_or(server))
                                        on_change=Callback::new(move |on| autotag.set(Some(on)))
                                    />
                                }
                            })
                    }}
                </span>
                <Show when=move || show_url.get()>
                    <button
                        type="button"
                        class="icon-btn"
                        class:on=move || linking.get()
                        title=move || t!("upload-link")
                        aria-label=move || t!("upload-link")
                        aria-expanded=move || if linking.get() { "true" } else { "false" }
                        on:click=move |_| linking.update(|l| *l = !*l)
                    >
                        <Ico icon=Icon::Link />
                    </button>
                </Show>
                <Show when=move || clearable.get()>
                    <button
                        type="button"
                        class="icon-btn danger"
                        title=move || t!("upload-clear")
                        aria-label=move || t!("upload-clear")
                        on:click=move |_| {
                            notice.set(None);
                            activity::clear();
                        }
                    >
                        <Ico icon=Icon::Delete />
                    </button>
                </Show>
            </div>
            <Show when=move || linking.get() && show_url.get()>
                <form
                    class="inline-add url-add"
                    on:submit=move |e: SubmitEvent| {
                        e.prevent_default();
                        add_urls();
                    }
                >
                    <textarea
                        rows="2"
                        placeholder=move || t!("upload-urls-placeholder")
                        aria-label=move || t!("upload-urls-label")
                        spellcheck="false"
                        autocapitalize="none"
                        autofocus
                        prop:value=move || url_text.get()
                        on:input=move |e| url_text.set(event_target_value(&e))
                        on:keydown=move |e: KeyboardEvent| {
                            // Enter adds them, as in a one-line box; Shift+Enter starts a line.
                            if e.key() == "Enter" && !e.shift_key() && !e.is_composing() {
                                e.prevent_default();
                                add_urls();
                            }
                        }
                    ></textarea>
                    <button
                        class="btn tonal"
                        type="submit"
                        disabled=move || url_text.with(|t| t.trim().is_empty())
                    >
                        <Ico icon=Icon::Add />
                        {move || t!("add")}
                    </button>
                </form>
            </Show>
            {move || notice.get().map(|message| view! { <div class="banner info" role="status"><span class="break">{message}</span></div> })}
            <div class="up-grid" class:empty=empty>
                <label class="up-drop" class:lifted=move || veil.shown() title=move || t!("upload-files")>
                    <Ico icon=Icon::Upload size=40 />
                    <span>{move || t!("upload-drop")}</span>
                    <input
                        type="file"
                        multiple
                        class="visually-hidden"
                        aria-label=move || t!("upload-files")
                        on:change=move |e| take(picked(&e))
                    />
                </label>
                <For
                    each=placed
                    key=|(_, key)| *key
                    children=move |(place, key)| {
                        view! { <FileTile key=key preview={place < MAX_PREVIEWS} on_open=open_item /> }
                    }
                />
                <For
                    each=move || job_ids.get()
                    key=|id| *id
                    children=move |id| view! { <JobTile id=id /> }
                />
            </div>
            <Show when=move || { drafts.get() != 0 || going.get() }>
                <UpBar drafts=drafts />
            </Show>
            {move || {
                opened
                    .get()
                    .map(|(key, from)| {
                        view! {
                            <ItemSheet
                                key=key
                                from=from
                                rating=shared(&RATING)
                                on_close=Callback::new(move |()| opened.set(None))
                            />
                        }
                    })
            }}
            {veil.view()}
        </div>
    }
}

/// The floating bar at the foot of the page: how far the files on their way
/// have got, and the tags, source and rating everything waiting goes with.
#[component]
fn UpBar(drafts: Memo<usize>) -> impl IntoView {
    let tags = shared(&SHARED_TAGS);
    let rating = shared(&RATING);
    let source = shared(&SOURCE);
    let error = RwSignal::new(None::<String>);
    let overall = Memo::new(|_| activity::with_queue(Queue::overall));
    // A mended tag takes the complaint away.
    Effect::watch(
        move || tags.get(),
        move |_, _, _| {
            if error.with_untracked(Option::is_some) {
                error.set(None);
            }
        },
        false,
    );
    let send = move || {
        if let Some(problem) = batch_problem(None) {
            error.set(Some(problem));
            return;
        }
        error.set(None);
        activity::start(&batch(), None);
    };
    let progress = Signal::derive(move || overall.get().map(|o| o.transfer));
    let count = move || {
        overall
            .get()
            .map(|o| t!("upload-progress", done = o.finished, count = o.files))
            .unwrap_or_default()
    };
    view! {
        <div class="bulk-bar glass up-bar" role="region" aria-label=move || t!("nav-upload")>
            <Show when=move || overall.with(Option::is_some)>
                <div class="up-overall">
                    <span class="up-overall-text">{count}</span>
                    <TransferBar progress=progress name=Signal::derive(count) />
                </div>
            </Show>
            <Show when=move || drafts.get() != 0>
                <form
                    class="up-form"
                    on:submit=move |e: SubmitEvent| {
                        e.prevent_default();
                        send();
                    }
                >
                    <div
                        class="up-tags"
                        on:keydown=move |e: KeyboardEvent| {
                            // One line, so Enter sends, as it did in the reference's field.
                            if e.key() == "Enter" && !e.shift_key() && !e.is_composing() {
                                e.prevent_default();
                                send();
                            }
                        }
                    >
                        <TagBox
                            id="up-shared-tags"
                            text=tags
                            rows=1
                            invalid=Signal::derive(move || error.with(Option::is_some))
                            placeholder=Signal::derive(|| t!("upload-shared-tags"))
                            label=Signal::derive(|| t!("upload-shared-tags"))
                        />
                    </div>
                    <input
                        class="bulk-tags up-source"
                        type="url"
                        inputmode="url"
                        spellcheck="false"
                        autocapitalize="none"
                        placeholder=move || t!("upload-source-placeholder")
                        aria-label=move || t!("upload-source")
                        title=move || t!("upload-source-hint")
                        prop:value=move || source.get()
                        on:input=move |e| source.set(event_target_value(&e))
                    />
                    <Select
                        class="bulk-rating"
                        value=Signal::derive(move || rating.get().as_str().to_owned())
                        options=Signal::derive(rating_choices)
                        on_change=Callback::new(move |v: String| {
                            if let Some(r) = Rating::parse(&v) {
                                rating.set(r);
                            }
                        })
                        label=Signal::derive(|| t!("rating"))
                    />
                    <button class="btn" type="submit">
                        <Ico icon=Icon::Upload />
                        {move || t!("upload-start", count = drafts.get())}
                    </button>
                    {move || {
                        error.get().map(|m| view! { <p class="bulk-error break" role="alert">{m}</p> })
                    }}
                </form>
            </Show>
        </div>
    }
}

/// How long the veil and the sheets take to go, as `--med` in tokens.css.
const LEAVE: Duration = Duration::from_millis(250);

#[cfg(test)]
mod tests {
    #[test]
    fn a_long_list_of_tiles_draws_only_what_is_near() {
        let css = include_str!("../../../style/pages/upload.css");
        assert!(css.contains(".up-tile { content-visibility: auto;"));
        assert!(css.contains(".up-tile:nth-child(n + 40) { animation: none; }"));
    }

    #[test]
    fn the_bar_is_the_only_glass_and_its_lists_open_upwards() {
        let css = include_str!("../../../style/pages/upload.css");
        assert!(!css.contains("backdrop-filter"));
        let source = include_str!("mod.rs");
        assert!(source.contains(r#"class="bulk-bar glass up-bar""#));
        for file in [
            include_str!("tiles.rs"),
            include_str!("sheet.rs"),
            include_str!("drop.rs"),
        ] {
            assert!(!file.contains(" glass"));
        }
        // At the foot of the window, a list below the button would be cut off.
        assert!(css.contains(".up-bar .select-list { top: auto; bottom: calc(100% + 6px);"));
        assert!(css.contains(".up-bar .tag-offers {"));
    }

    #[test]
    fn the_toggles_are_switches_that_slide() {
        let css = include_str!("../../../style/base.css");
        assert!(css.contains(".switch.on .switch-knob { translate: 20px 0;"));
        assert!(
            css.contains(
                ".switch.on { border-color: var(--primary); background: var(--primary); }"
            )
        );
    }

    #[test]
    fn what_moves_goes_quiet_with_reduced_motion() {
        let css = include_str!("../../../style/pages/upload.css");
        let (_, quiet) = css
            .split_once("@media (prefers-reduced-motion: reduce)")
            .unwrap();
        for name in [
            ".up-veil",
            ".up-tile.draft:hover .thumb-media",
            ".item-sheet",
            ".upload-sheet",
        ] {
            assert!(quiet.contains(name), "{name}");
        }
    }
}
