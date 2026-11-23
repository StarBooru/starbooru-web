//! A file or an address opened from its tile: a bigger picture, what is
//! known of it, and its own rating and tags.

use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{Rating, SIZE_BYTES, detect_mime, picture_size};

use super::LEAVE;
use super::tiles::{TileFace, face_is_video, face_words, object_face};
use crate::activity;
use crate::components::icons::{Ico, Icon};
use crate::components::rating::own_rating_choices;
use crate::components::select::Select;
use crate::components::tag_box::TagBox;
use crate::dom;
use crate::files;
use crate::logic::queue::{Source, Status};
use crate::logic::upload::{format_by_name, format_of, host_of, human_size, tag_problem};
use crate::pages::{bad_tag_text, rating_label};

/// What the sheet says under the picture; each is left out when unknown.
#[derive(Clone, Debug, Default, PartialEq)]
struct Facts {
    /// Where it comes from: the path in a picked folder, or the site.
    source: Option<String>,
    size: Option<String>,
    dimensions: Option<(u32, u32)>,
    kind: Option<String>,
}

/// What the start of a file said.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Head {
    mime: Option<&'static str>,
    size: Option<(u32, u32)>,
}

impl Facts {
    /// `head` is what the file's start said, `drawn` the picture's size once
    /// shown.
    fn of(source: &Source, head: Head, drawn: Option<(u32, u32)>) -> Self {
        match source {
            Source::File {
                name, size, path, ..
            } => Self {
                source: Some(path.trim_start_matches('/').to_owned()).filter(|p| !p.is_empty()),
                size: Some(human_size(*size)),
                dimensions: head.size.or(drawn),
                kind: head
                    .mime
                    .and_then(format_of)
                    .or_else(|| format_by_name(name)),
            },
            Source::Url(url) => Self {
                source: Some(host_of(url)).filter(|h| !h.is_empty()),
                size: None,
                dimensions: drawn,
                kind: format_by_name(url),
            },
        }
    }
}

#[component]
pub(super) fn ItemSheet(
    key: u64,
    /// How far its tile is from the middle of the window.
    from: (f64, f64),
    /// The rating everything goes with.
    rating: RwSignal<Rating>,
    on_close: Callback<()>,
) -> impl IntoView {
    let Some(first) = activity::with_queue(|q| q.get(key).cloned()) else {
        on_close.run(());
        return ().into_any();
    };
    let closing = RwSignal::new(false);
    // Taken off the list from here: it fades, as its tile is going too.
    let fading = RwSignal::new(false);
    let close = move || {
        if closing.get_untracked() {
            return;
        }
        closing.set(true);
        spawn_local(async move {
            dom::sleep(LEAVE.as_millis().try_into().unwrap_or(250)).await;
            // The page may have gone meanwhile.
            on_close.try_run(());
        });
    };
    // Sent, or gone some other way: nothing left to change.
    let gone = Memo::new(move |_| {
        activity::with_queue(|q| q.get(key).is_none_or(|i| i.status != Status::Draft))
    });
    Effect::watch(
        move || gone.get(),
        move |gone, _, _| {
            if *gone {
                close();
            }
        },
        true,
    );

    let name = first.name().to_owned();
    let (icon, caption) = face_words(&first.source);
    let src = object_face(key, &first.source, true);
    let video = face_is_video(&first.source);
    let head = RwSignal::new(Head::default());
    if let Some(file) = activity::file(key).filter(|_| !video) {
        spawn_local(async move {
            if let Some(start) = files::head(&file, SIZE_BYTES).await {
                head.try_set(Head {
                    mime: detect_mime(&start),
                    size: picture_size(&start),
                });
            }
        });
    }
    let drawn = RwSignal::new(None::<(u32, u32)>);
    let source = first.source.clone();
    let facts = Memo::new(move |_| Facts::of(&source, head.get(), drawn.get()));

    let tags = RwSignal::new(first.tags.clone());
    Effect::watch(
        move || tags.get(),
        move |tags, _, _| activity::set_tags(key, tags.clone()),
        false,
    );
    let problem = Memo::new(move |_| {
        tags.with(|t| tag_problem("", [t.as_str()]))
            .map(|b| bad_tag_text(&b))
    });
    let own = Memo::new(move |_| activity::with_queue(|q| q.get(key).and_then(|i| i.rating)));
    let choices = Signal::derive(move || {
        own_rating_choices(t!(
            "upload-rating-usual",
            rating = rating_label(rating.get())
        ))
    });
    let shut: NodeRef<html::Button> = NodeRef::new();
    Effect::new(move |_| {
        if let Some(el) = shut.get() {
            let _ = el.focus();
        }
    });
    let link = match &first.source {
        Source::Url(url) => Some(url.clone()),
        Source::File { .. } => None,
    };
    let (x, y) = from;
    let label_for = t!("upload-file-tags-label", name = name.clone());

    view! {
        <div
            class="sheet-scrim"
            class:closing=move || closing.get()
            class:fading=move || fading.get()
            on:click=move |_| close()
            on:keydown=move |e: KeyboardEvent| {
                if e.key() == "Escape" {
                    e.stop_propagation();
                    close();
                }
            }
        >
            <div
                class="card sheet item-sheet"
                style=format!("--from-x: {x:.1}px; --from-y: {y:.1}px")
                role="dialog"
                aria-modal="true"
                aria-labelledby="item-sheet-title"
                on:click=|e| e.stop_propagation()
            >
                <div class="sheet-head item-head">
                    <h2 id="item-sheet-title" class="card-title item-title" title=name.clone()>
                        {name.clone()}
                    </h2>
                    <button
                        type="button"
                        class="icon-btn"
                        node_ref=shut
                        aria-label=move || t!("upload-close")
                        title=move || t!("upload-close")
                        on:click=move |_| close()
                    >
                        <Ico icon=Icon::Close />
                    </button>
                </div>
                <div class="item-body">
                    <div class="item-preview">
                        <TileFace
                            src=src
                            video=video
                            icon=icon
                            caption=caption
                            on_size=Callback::new(move |size| drawn.set(Some(size)))
                        />
                    </div>
                    <div class="item-about">
                        <dl class="meta item-facts">
                            {move || {
                                let f = facts.get();
                                view! {
                                    {f.source.map(|s| view! {
                                        <dt>{t!("post-source")}</dt>
                                        <dd class="break" title=s.clone()>{s.clone()}</dd>
                                    })}
                                    {f.size.map(|s| view! {
                                        <dt>{t!("post-size")}</dt>
                                        <dd>{s}</dd>
                                    })}
                                    {f.dimensions.map(|(w, h)| view! {
                                        <dt>{t!("post-dimensions")}</dt>
                                        <dd>{format!("{w} × {h}")}</dd>
                                    })}
                                    {f.kind.map(|k| view! {
                                        <dt>{t!("post-type")}</dt>
                                        <dd>{k}</dd>
                                    })}
                                }
                            }}
                        </dl>
                    </div>
                    <div class="item-fields">
                        <div class="item-top">
                            // Not a `label`: a click on the list would press the button in it again.
                            <div class="field">
                                <span>{move || t!("rating")}</span>
                                <Select
                                    class="boxed item-rating"
                                    value=Signal::derive(move || {
                                        own.get().map(|r| r.as_str().to_owned()).unwrap_or_default()
                                    })
                                    options=choices
                                    on_change=Callback::new(move |v: String| {
                                        activity::set_rating(key, Rating::parse(&v));
                                    })
                                    label=Signal::derive(|| t!("rating"))
                                />
                            </div>
                            <div class="field item-tag-field">
                                <label for="item-tags">{move || t!("post-tags")}</label>
                                <TagBox
                                    id="item-tags"
                                    text=tags
                                    rows=8
                                    invalid=Signal::derive(move || problem.with(Option::is_some))
                                    placeholder=Signal::derive(|| t!("upload-file-tags-placeholder"))
                                    label=Signal::derive(move || label_for.clone())
                                />
                            </div>
                        </div>
                        <div class="item-note">
                            {move || {
                                problem.get().map(|p| view! { <p class="queue-error" role="alert">{p}</p> })
                            }}
                        </div>
                    </div>
                </div>
                <div class="item-foot">
                    {link
                        .map(|url| {
                            view! {
                                <a class="btn text" href=url target="_blank" rel="noopener noreferrer">
                                    <Ico icon=Icon::Link />
                                    {move || t!("upload-open-link")}
                                </a>
                            }
                        })}
                    <button
                        type="button"
                        class="btn text bin"
                        on:click=move |_| {
                            fading.set(true);
                            close();
                            activity::remove(key);
                        }
                    >
                        <Ico icon=Icon::Delete />
                        {move || t!("upload-remove-item")}
                    </button>
                </div>
            </div>
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facts_come_from_the_files_start_or_its_name() {
        let file = Source::File {
            name: "a.jpeg".into(),
            size: 2048,
            mime: String::new(),
            path: "/Cats/a.jpeg".into(),
        };
        let facts = Facts::of(&file, Head::default(), Some((10, 20)));
        assert_eq!(facts.source.as_deref(), Some("Cats/a.jpeg"));
        assert_eq!(facts.size.as_deref(), Some("2.0 KB"));
        assert_eq!(facts.dimensions, Some((10, 20)));
        assert_eq!(facts.kind.as_deref(), Some("JPG"));
        let read = Head {
            mime: Some("image/png"),
            size: Some((3, 4)),
        };
        let facts = Facts::of(&file, read, Some((10, 20)));
        assert_eq!(
            (facts.dimensions, facts.kind.as_deref()),
            (Some((3, 4)), Some("PNG"))
        );
        let url = Source::Url("https://www.example.org/a.webp?x".into());
        let facts = Facts::of(&url, Head::default(), None);
        assert_eq!(facts.source.as_deref(), Some("example.org"));
        assert_eq!((facts.size, facts.kind.as_deref()), (None, Some("WEBP")));
    }
}
