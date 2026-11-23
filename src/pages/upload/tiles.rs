//! The Upload page's tiles: a file or an address waiting, on its way or
//! done, and an address the server imports.

use leptos::prelude::*;
use starbooru_core::JobState;

use crate::activity;
use crate::components::icons::{Ico, Icon};
use crate::components::snackbar::notify;
use crate::components::transfer::TransferBar;
use crate::files;
use crate::logic::queue::{Source, Status};
use crate::logic::upload::{duplicate_post, host_of, human_size, is_picture_url, is_video};
use crate::route;

/// What a tile or the item sheet shows of a picked file: its own address in
/// the browser, which is handed back when the tile goes. An address on
/// another site isn't loaded here: the app only talks to its server.
pub(super) fn object_face(key: u64, source: &Source, preview: bool) -> Option<String> {
    if !preview || !matches!(source, Source::File { .. }) {
        return None;
    }
    let url = files::object_url(&activity::file(key)?)?;
    let kept = url.clone();
    on_cleanup(move || files::release(&kept));
    Some(url)
}

/// The icon and the words a tile shows when it has no picture.
pub(super) fn face_words(source: &Source) -> (Icon, String) {
    match source {
        Source::File { name, size, .. } => {
            (Icon::Gallery, format!("{name} · {}", human_size(*size)))
        }
        Source::Url(url) if is_picture_url(url) => (Icon::Gallery, host_of(url)),
        Source::Url(url) => (Icon::Link, host_of(url)),
    }
}

/// Whether a file is a video, to show its first frame.
pub(super) fn face_is_video(source: &Source) -> bool {
    matches!(source, Source::File { name, mime, .. } if is_video(name, mime))
}

/// A file's picture, or an icon with a few words when there is none to show.
#[component]
pub(super) fn TileFace(
    src: Option<String>,
    #[prop(optional)] video: bool,
    icon: Icon,
    caption: String,
    /// The picture was drawn, with its size.
    #[prop(optional)]
    on_size: Option<Callback<(u32, u32)>>,
) -> impl IntoView {
    // A file the browser can't draw (an odd format) falls back.
    let broken = RwSignal::new(false);
    let face = move || match src.clone().filter(|_| !broken.get()) {
        Some(src) if video => view! {
            // A moment in: some browsers draw nothing at 0.
            <video
                src=format!("{src}#t=0.1")
                muted=true
                prop:muted=true
                playsinline
                preload="metadata"
                on:error=move |_| broken.set(true)
            ></video>
        }
        .into_any(),
        Some(src) => view! {
            <img
                src=src
                alt=""
                decoding="async"
                on:error=move |_| broken.set(true)
                on:load=move |e| {
                    let drawn = event_target::<web_sys::HtmlImageElement>(&e);
                    if let Some(said) = on_size
                        && drawn.natural_width() > 0
                    {
                        said.run((drawn.natural_width(), drawn.natural_height()));
                    }
                }
            />
        }
        .into_any(),
        None => view! {
            <div class="thumb-placeholder up-face">
                <Ico icon=icon size=32 />
                <span class="up-name">{caption.clone()}</span>
            </div>
        }
        .into_any(),
    };
    view! { <div class="thumb-media">{face}</div> }
}

/// The cross in a tile's corner.
#[component]
fn CrossButton(label: String, on_press: Callback<()>) -> impl IntoView {
    view! {
        <button
            type="button"
            class="up-btn up-x"
            aria-label=label.clone()
            title=label
            on:click=move |_| on_press.run(())
        >
            <Ico icon=Icon::Close size=18 />
        </button>
    }
}

/// The mark on a tile that failed. Its reason shows on hover, and in a
/// notice when pressed, as a finger has no hover.
#[component]
fn FailMark(message: String) -> impl IntoView {
    let said = message.clone();
    view! {
        <button
            type="button"
            class="up-mark bad"
            title=message.clone()
            aria-label=message
            on:click=move |_| notify(said.clone())
        >
            <Ico icon=Icon::Close size=18 />
        </button>
    }
}

/// The button that sends a failed or cancelled item again.
#[component]
fn AgainButton(name: String, key: u64) -> impl IntoView {
    let label = t!("upload-retry", name = name);
    view! {
        <button
            type="button"
            class="up-btn up-again"
            aria-label=label.clone()
            title=label
            on:click=move |_| activity::retry(key)
        >
            <Ico icon=Icon::Restore size=18 />
        </button>
    }
}

/// One file or address: waiting for Upload (it opens its sheet), on its way,
/// or done.
#[component]
pub(super) fn FileTile(key: u64, preview: bool, on_open: Callback<u64>) -> impl IntoView {
    let Some(first) = activity::with_queue(|q| q.get(key).cloned()) else {
        return ().into_any();
    };
    let name = first.name().to_owned();
    let (icon, caption) = face_words(&first.source);
    let src = object_face(key, &first.source, preview);
    let video = face_is_video(&first.source);
    let status = Memo::new(move |_| activity::with_queue(|q| q.get(key).map(|i| i.status.clone())));
    let progress =
        Signal::derive(move || activity::with_queue(|q| q.get(key).and_then(|i| i.progress)));
    let tag_count = Memo::new(move |_| {
        activity::with_queue(|q| q.get(key).map_or(0, |i| i.tags.split_whitespace().count()))
    });
    let is_file = matches!(first.source, Source::File { .. });

    let overlay = {
        let name = name.clone();
        move || {
            let name = name.clone();
            let gone = name.clone();
            let remove = move || {
                view! {
                    <CrossButton
                        label=t!("upload-remove", name = gone.clone())
                        on_press=Callback::new(move |()| activity::remove(key))
                    />
                }
            };
            match status.get()? {
                Status::Draft => Some(
                    view! {
                        <button
                            type="button"
                            class="up-open"
                            aria-label=t!("upload-open", name = name.clone())
                            on:click=move |_| on_open.run(key)
                        ></button>
                        <Show when=move || tag_count.get() != 0>
                            <span class="up-count" aria-hidden="true">
                                <Ico icon=Icon::Label size=16 />
                                {move || tag_count.get()}
                            </span>
                        </Show>
                        {remove()}
                    }
                    .into_any(),
                ),
                Status::Waiting => Some(
                    view! {
                        <div class="up-progress">
                            <span class="up-state">{t!("upload-queued")}</span>
                        </div>
                        <CrossButton
                            label=t!("upload-cancel", name = name.clone())
                            on_press=Callback::new(move |()| activity::cancel(key))
                        />
                    }
                    .into_any(),
                ),
                Status::Sending => {
                    let label = t!("upload-uploading");
                    Some(
                        view! {
                            <div class="up-progress">
                                <TransferBar
                                    progress=Signal::derive(move || progress.get().filter(|_| is_file))
                                    name=Signal::derive(move || label.clone())
                                />
                            </div>
                            <Show when=move || is_file>
                                <CrossButton
                                    label=t!("upload-cancel", name = name.clone())
                                    on_press=Callback::new(move |()| activity::cancel(key))
                                />
                            </Show>
                        }
                        .into_any(),
                    )
                }
                Status::Done(id) => Some(
                    view! {
                        <a
                            class="up-open"
                            href=route::post(id, "", None)
                            aria-label=t!("upload-open-post", id = id)
                        ></a>
                        <span class="up-mark ok" title=t!("upload-uploaded")>
                            <Ico icon=Icon::Check size=18 />
                        </span>
                    }
                    .into_any(),
                ),
                Status::Duplicate(post) => {
                    let said = match post {
                        Some(id) => t!("upload-known-post", id = id),
                        None => t!("upload-known"),
                    };
                    Some(
                        view! {
                            {post
                                .map(|id| {
                                    view! {
                                        <a
                                            class="up-open"
                                            href=route::post(id, "", None)
                                            aria-label=said.clone()
                                        ></a>
                                    }
                                })}
                            <span class="up-mark" title=said.clone() aria-label=said.clone()>
                                <Ico icon=Icon::History size=18 />
                            </span>
                            {remove()}
                        }
                        .into_any(),
                    )
                }
                Status::Failed(message) => Some(
                    view! {
                        <FailMark message=message />
                        <AgainButton name=name.clone() key=key />
                        {remove()}
                    }
                    .into_any(),
                ),
                Status::Stopped => Some(
                    view! {
                        <div class="up-progress">
                            <span class="up-state">{t!("upload-stopped")}</span>
                        </div>
                        <AgainButton name=name.clone() key=key />
                        {remove()}
                    }
                    .into_any(),
                ),
            }
        }
    };
    let class_of =
        move |wanted: fn(&Status) -> bool| move || status.with(|s| s.as_ref().is_some_and(wanted));
    view! {
        <div
            class="thumb up-tile"
            class:draft=class_of(|s| *s == Status::Draft)
            class:busy=class_of(|s| matches!(s, Status::Sending | Status::Waiting))
            class:failed=class_of(|s| matches!(s, Status::Failed(_) | Status::Stopped))
            class:done=class_of(|s| matches!(s, Status::Done(_)))
            data-up-key=key
            title=name
        >
            <TileFace src=src video=video icon=icon caption=caption />
            {overlay}
        </div>
    }
    .into_any()
}

/// One address the server is importing, or has.
#[component]
pub(super) fn JobTile(id: i64) -> impl IntoView {
    let job = Memo::new(move |_| activity::with_jobs(|j| j.iter().find(|j| j.id == id).cloned()));
    let Some(first) = job.get_untracked() else {
        return ().into_any();
    };
    let name = first.source.clone().unwrap_or_else(|| first.url.clone());
    let icon = if is_picture_url(&first.url) {
        Icon::Gallery
    } else {
        Icon::Link
    };
    let state =
        Memo::new(move |_| job.with(|j| j.as_ref().map(|j| (j.state, j.post_id, j.error.clone()))));
    let progress = Signal::derive(move || job.with(|j| j.as_ref().and_then(|j| j.progress)));
    let overlay = move || {
        let (state, post, note) = state.get()?;
        let note = note.unwrap_or_default();
        Some(match state {
            JobState::Queued => view! {
                <div class="up-progress">
                    <span class="up-state">{t!("upload-queued")}</span>
                </div>
                <CrossButton
                    label=t!("upload-job-cancel")
                    on_press=Callback::new(move |()| activity::cancel_job(id))
                />
            }
            .into_any(),
            JobState::Running => {
                let label = t!("upload-job-running");
                view! {
                    <div class="up-progress">
                        <TransferBar progress=progress name=Signal::derive(move || label.clone()) />
                    </div>
                }
                .into_any()
            }
            JobState::Done => view! {
                {post
                    .map(|id| {
                        view! {
                            <a
                                class="up-open"
                                href=route::post(id, "", None)
                                aria-label=t!("upload-open-post", id = id)
                            ></a>
                        }
                    })}
                <span class="up-mark ok" title=t!("upload-done")>
                    <Ico icon=Icon::Check size=18 />
                </span>
            }
            .into_any(),
            JobState::Duplicate => {
                let known = post.or_else(|| duplicate_post(&note));
                let said = if note.is_empty() {
                    t!("upload-known")
                } else {
                    note
                };
                view! {
                    {known
                        .map(|id| {
                            view! {
                                <a class="up-open" href=route::post(id, "", None) aria-label=said.clone()></a>
                            }
                        })}
                    <span class="up-mark" title=said.clone() aria-label=said.clone()>
                        <Ico icon=Icon::History size=18 />
                    </span>
                }
                .into_any()
            }
            JobState::Failed => {
                let message = if note.is_empty() {
                    t!("upload-failed")
                } else {
                    note
                };
                view! { <FailMark message=message /> }.into_any()
            }
        })
    };
    let has =
        move |wanted: JobState| move || state.with(|s| s.as_ref().is_some_and(|s| s.0 == wanted));
    view! {
        <div
            class="thumb up-tile"
            class:busy=has(JobState::Running)
            class:failed=has(JobState::Failed)
            class:done=has(JobState::Done)
            title=name.clone()
        >
            <TileFace src=None icon=icon caption=host_of(&name) />
            {overlay}
        </div>
    }
    .into_any()
}
