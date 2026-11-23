//! Files dragged over a page: the veil that says where they go, and on the
//! gallery the sheet that asks for their tags before they go up.

use std::cell::Cell;
use std::rc::Rc;

use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;

use super::{LEAVE, RATING, SHARED_TAGS, batch, batch_problem, shared, skipped_notice};
use crate::activity;
use crate::components::icons::{Ico, Icon};
use crate::components::rating::RatingPicker;
use crate::components::snackbar::notify;
use crate::components::tag_box::TagBox;
use crate::dom::{self, Watch};
use crate::files::{self, Dropped};
use crate::logic::queue::Status;
use crate::logic::upload::{host_of, human_size};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Off,
    In,
    /// Going back into the box it came from.
    Out,
}

/// The veil over the page while files are dragged over it. On the Upload
/// page it grows out of the drop box and shrinks back into it.
#[derive(Clone, Copy)]
pub struct Veil {
    phase: RwSignal<Phase>,
    /// The box's distance from each edge of the window, and its corners.
    edges: RwSignal<[f64; 5]>,
    /// Counts the times it went, so a late end of one leaves a later one be.
    round: RwSignal<u32>,
}

impl Veil {
    pub fn new() -> Self {
        Self {
            phase: RwSignal::new(Phase::Off),
            edges: RwSignal::new([0.0; 5]),
            round: RwSignal::new(0),
        }
    }

    /// Shows the veil, grown from the element `from` picks.
    pub fn show(self, from: &str) {
        if self.phase.get_untracked() == Phase::In {
            return;
        }
        if let Some(edges) = dom::edges_of(from) {
            self.edges.set(edges);
        }
        self.phase.set(Phase::In);
    }

    /// Takes the veil away into the element `into` picks, where it is now.
    pub fn hide(self, into: &str) {
        // Also called once a drop is read, when the page may have gone.
        if self.phase.try_get_untracked() != Some(Phase::In) {
            return;
        }
        if let Some(edges) = dom::edges_of(into) {
            self.edges.set(edges);
        }
        self.phase.set(Phase::Out);
        let Some(round) = self.round.try_update(|r| {
            *r = r.wrapping_add(1);
            *r
        }) else {
            return;
        };
        spawn_local(async move {
            dom::sleep(LEAVE.as_millis().try_into().unwrap_or(250)).await;
            if self.round.try_get_untracked() == Some(round) {
                self.phase.try_set(Phase::Off);
            }
        });
    }

    /// Takes the veil away at once.
    fn hide_now(self) {
        self.phase.set(Phase::Off);
    }

    /// Whether it's up or on its way out.
    pub fn shown(self) -> bool {
        self.phase.get() != Phase::Off
    }

    /// The veil itself. `grow` makes it come out of its box and go back in.
    pub fn view(self) -> impl IntoView {
        self.view_as(true)
    }

    fn view_as(self, grow: bool) -> impl IntoView {
        move || {
            let phase = self.phase.get();
            (phase != Phase::Off).then(|| {
                let [t, r, b, l, radius] = self.edges.get();
                let style = format!(
                    "--box-t: {t:.1}px; --box-r: {r:.1}px; --box-b: {b:.1}px; --box-l: {l:.1}px; --box-radius: {radius:.1}px"
                );
                view! {
                    <div
                        class="drop-veil"
                        class:up-veil=grow
                        class:closing=phase == Phase::Out
                        style=style
                        aria-hidden="true"
                    >
                        <div class="drop-veil-card">
                            <Ico icon=Icon::Upload size=40 />
                            <strong>{move || t!("upload-drop-now")}</strong>
                        </div>
                    </div>
                }
            })
        }
    }
}

/// Whether the sheet for dropped files is open.
fn sheet_open() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.query_selector(".upload-sheet").ok().flatten())
        .is_some()
}

/// Watches the window for files dragged over it: it, and not the page,
/// as the page doesn't fill the window, and a file dropped outside it would
/// have the browser open that file. Pictures dragged off the page itself
/// don't count. `over` hears when the veil should come and go, `dropped`
/// gets what a drop carried.
pub(super) fn watch_drops(
    over: impl Fn(bool) + Copy + 'static,
    dropped: impl Fn(Dropped) + 'static,
) -> Vec<Watch> {
    let Some(window) = web_sys::window() else {
        return Vec::new();
    };
    let own = Rc::new(Cell::new(false));
    let carries = |e: &web_sys::Event| {
        e.dyn_ref::<web_sys::DragEvent>()
            .and_then(web_sys::DragEvent::data_transfer)
            .filter(files::carries_files)
    };
    let enter = {
        let own = Rc::clone(&own);
        move |e: web_sys::Event| {
            if own.get() || carries(&e).is_none() {
                return;
            }
            // Taken, so the browser lets it drop here.
            e.prevent_default();
            if !sheet_open() {
                over(true);
            }
        }
    };
    let leave = move |e: web_sys::Event| {
        // The veil covers the window: leaving it is leaving the window.
        let off_veil = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .is_some_and(|el| el.class_list().contains("drop-veil"));
        if off_veil {
            over(false);
        }
    };
    let drop = {
        let own = Rc::clone(&own);
        move |e: web_sys::Event| {
            let mine = own.replace(false);
            let Some(data) = carries(&e).filter(|_| !mine) else {
                return;
            };
            e.prevent_default();
            // Read now: a drop's list can't be asked for afterwards.
            dropped(files::read_drop(&data));
        }
    };
    let start = {
        let own = Rc::clone(&own);
        move |_| own.set(true)
    };
    let end = move |_| own.set(false);
    vec![
        dom::listen(&window, "dragenter", false, enter.clone()),
        dom::listen(&window, "dragover", false, enter),
        dom::listen(&window, "dragleave", true, leave),
        dom::listen(&window, "drop", false, drop),
        dom::listen(&window, "dragstart", true, start),
        dom::listen(&window, "dragend", true, end),
    ]
}

/// Files dropped on the gallery: a sheet, out of the Upload item in the
/// navigation and back into it, asks for their tags and sends them.
#[component]
pub fn DropToUpload() -> impl IntoView {
    let veil = Veil::new();
    // The drafts in the sheet, and where it comes from once it is open.
    let keys = RwSignal::new(Vec::<u64>::new());
    let from = RwSignal::new(None::<(f64, f64)>);
    let closing = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let watches = watch_drops(
        move |on| {
            if on {
                veil.show(".nav-upload");
            } else {
                veil.hide_now();
            }
        },
        move |dropped| {
            veil.hide_now();
            if closing.get_untracked() || dropped.is_empty() {
                return;
            }
            spawn_local(async move {
                let (added, skipped) = super::take_drop(dropped).await;
                let gone = closing.try_get_untracked() != Some(false);
                // Reading a folder takes a while: the gallery, or its sheet,
                // may have gone, and the files wait on the Upload page.
                let said = if gone && !added.is_empty() {
                    Some(t!("upload-drop-waiting", count = added.len()))
                } else {
                    skipped_notice(&skipped)
                };
                if let Some(said) = said {
                    notify(said);
                }
                if added.is_empty() || gone {
                    return;
                }
                keys.update(|k| k.extend(added));
                if from.get_untracked().is_none() {
                    // The item at the bottom on a phone, in the rail or the top bar elsewhere.
                    let at = dom::offset_from_middle(".nav-upload").unwrap_or_default();
                    from.set(Some(at));
                }
            });
        },
    );
    let watches = SendWrapper::new(watches);
    on_cleanup(move || {
        for watch in watches.take() {
            watch.stop();
        }
    });

    let close = move |discard: bool| {
        if closing.get_untracked() {
            return;
        }
        closing.set(true);
        spawn_local(async move {
            dom::sleep(LEAVE.as_millis().try_into().unwrap_or(250)).await;
            if discard {
                for key in keys.try_get_untracked().unwrap_or_default() {
                    activity::remove(key);
                }
            }
            keys.try_set(Vec::new());
            error.try_set(None);
            from.try_set(None);
            closing.try_set(false);
        });
    };
    let drafts = Memo::new(move |_| {
        let keys = keys.get();
        activity::with_queue(|q| {
            q.items()
                .iter()
                .filter(|i| keys.contains(&i.key) && i.status == Status::Draft)
                .count()
        })
    });
    // Each file taken off the list: nothing left to ask about.
    Effect::watch(
        move || from.get().is_some() && drafts.get() == 0,
        move |emptied, _, _| {
            if *emptied {
                close(false);
            }
        },
        false,
    );
    let send = move || {
        let chosen = keys.get_untracked();
        if let Some(problem) = batch_problem(Some(&chosen)) {
            error.set(Some(problem));
            return;
        }
        activity::start(&batch(), Some(&chosen));
        close(false);
    };
    let tags = shared(&SHARED_TAGS);
    let rating = shared(&RATING);
    // The shared tags have the focus when the sheet opens.
    let area = NodeRef::<leptos::html::Textarea>::new();
    Effect::new(move |_| {
        if let Some(el) = area.get() {
            let _ = el.focus();
        }
    });
    Effect::watch(
        move || tags.get(),
        move |_, _, _| {
            if error.with_untracked(Option::is_some) {
                error.set(None);
            }
        },
        false,
    );

    view! {
        {veil.view_as(false)}
        {move || {
            from.get()
                .map(|(x, y)| {
                    view! {
                        <div
                            class="sheet-scrim"
                            class:closing=move || closing.get()
                            on:click=move |_| close(true)
                            on:keydown=move |e: KeyboardEvent| {
                                if e.key() == "Escape" {
                                    e.stop_propagation();
                                    close(true);
                                }
                            }
                        >
                            <div
                                class="card sheet upload-sheet"
                                style=format!("--from-x: {x:.1}px; --from-y: {y:.1}px")
                                role="dialog"
                                aria-modal="true"
                                aria-labelledby="upload-sheet-title"
                                on:click=|e| e.stop_propagation()
                            >
                                <div class="sheet-head">
                                    <h2 id="upload-sheet-title" class="card-title">
                                        {move || t!("nav-upload")}
                                    </h2>
                                    <button
                                        class="icon-btn"
                                        type="button"
                                        aria-label=move || t!("cancel")
                                        title=move || t!("cancel")
                                        on:click=move |_| close(true)
                                    >
                                        <Ico icon=Icon::Close />
                                    </button>
                                </div>
                                {move || {
                                    error
                                        .get()
                                        .map(|m| view! { <div class="banner error" role="alert"><span class="break">{m}</span></div> })
                                }}
                                <ul class="queue">
                                    <For
                                        each=move || {
                                            let keys = keys.get();
                                            activity::with_queue(|q| {
                                                keys.into_iter().filter(|k| q.get(*k).is_some()).collect::<Vec<_>>()
                                            })
                                        }
                                        key=|key| *key
                                        children=move |key| view! { <QueueRow key=key /> }
                                    />
                                </ul>
                                <div class="field">
                                    <label for="drop-tags">{move || t!("upload-shared-tags")}</label>
                                    <TagBox id="drop-tags" text=tags rows=2 area=area />
                                </div>
                                <div class="field">
                                    <span>{move || t!("rating")}</span>
                                    <RatingPicker value=rating />
                                </div>
                                <div class="row-end">
                                    <button class="btn text" type="button" on:click=move |_| close(true)>
                                        {move || t!("cancel")}
                                    </button>
                                    <button
                                        class="btn"
                                        type="button"
                                        disabled=move || drafts.get() == 0
                                        on:click=move |_| send()
                                    >
                                        <Ico icon=Icon::Upload />
                                        {move || t!("upload-start", count = drafts.get())}
                                    </button>
                                </div>
                            </div>
                        </div>
                    }
                })
        }}
    }
}

/// One file or address in the sheet, with its own tags while it waits.
#[component]
fn QueueRow(key: u64) -> impl IntoView {
    let first = activity::with_queue(|q| q.get(key).cloned());
    let Some(item) = first else {
        return ().into_any();
    };
    let name = item.name().to_owned();
    let detail = match item.size() {
        Some(size) => human_size(size),
        None => host_of(&name),
    };
    let remove = t!("upload-remove", name = name.clone());
    let own = RwSignal::new(item.tags.clone());
    Effect::watch(
        move || own.get(),
        move |tags, _, _| activity::set_tags(key, tags.clone()),
        false,
    );
    let draft = Memo::new(move |_| {
        activity::with_queue(|q| q.get(key).is_some_and(|i| i.status == Status::Draft))
    });
    let label = t!("upload-file-tags-label", name = name.clone());
    view! {
        <li class="queue-item">
            <div class="queue-main">
                <div class="queue-name">
                    <strong title=name.clone()>{name.clone()}</strong>
                    <span class="muted">{detail}</span>
                </div>
                <Show when=move || draft.get()>
                    <button
                        type="button"
                        class="icon-btn"
                        aria-label=remove.clone()
                        title=remove.clone()
                        on:click=move |_| activity::remove(key)
                    >
                        <Ico icon=Icon::Close size=20 />
                    </button>
                </Show>
            </div>
            <Show when=move || draft.get()>
                <label class="queue-tags">
                    <Ico icon=Icon::Label size=18 />
                    <input
                        type="text"
                        spellcheck="false"
                        autocapitalize="none"
                        placeholder=move || t!("upload-file-tags-placeholder")
                        aria-label=label.clone()
                        prop:value=move || own.get()
                        on:input=move |e| own.set(event_target_value(&e))
                    />
                </label>
            </Show>
        </li>
    }
    .into_any()
}
