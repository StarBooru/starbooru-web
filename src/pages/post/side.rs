//! The viewer's side column: the post's tags under their headings, with
//! their votes on a public server (`tag_votes.rs`), and what the server
//! knows of its file, and on a public server who uploaded it.

use std::sync::Arc;

use chrono::Local;
use leptos::prelude::*;
use starbooru_core::{MediaKind, Post, PostTag};

use super::tag_votes::{TagVoteChip, Votes};
use crate::components::avatar::{Size, UserLink};
use crate::components::icons::{Ico, Icon};
use crate::logic::gallery::tint;
use crate::logic::viewer::{
    Group, category_class, color_href, dimensions, file_date, format_duration, group_tags, headed,
    human_size, source_link, when,
};
use crate::pages::{category_label, rating_label};
use crate::route;

fn section_label(name: &str) -> String {
    match name {
        "people" => t!("section-people"),
        "hair" => t!("section-hair"),
        "eyes" => t!("section-eyes"),
        "skin" => t!("section-skin"),
        "body" => t!("section-body"),
        "expression" => t!("section-expression"),
        "pose" => t!("section-pose"),
        "clothing" => t!("section-clothing"),
        "setting" => t!("section-setting"),
        "style" => t!("section-style"),
        other => {
            // A section someone made: `hair_accessories` reads "Hair accessories".
            let words = other.replace('_', " ");
            let mut chars = words.chars();
            chars
                .next()
                .map(|c| c.to_uppercase().chain(chars).collect())
                .unwrap_or_default()
        }
    }
}

pub(super) fn group_label(group: &Group) -> String {
    match group {
        Group::Category(c) => category_label(*c),
        Group::Section(name) => section_label(name),
        Group::Other => t!("section-other"),
    }
}

/// A tag that searches for itself. Its menu (`components/tag_menu.rs`) has
/// the way to its page.
pub(super) fn chip(name: String, class: &'static str) -> impl IntoView {
    let href = route::gallery(&name);
    let key = name.clone();
    view! {
        <a class=class href=href data-tag=key>
            {name}
        </a>
    }
}

/// The tags, grouped when the server sent their categories, and the way to
/// edit them for whoever `editable` says may. On a public server each tag
/// takes votes, whose answers go to `on_voted`.
#[component]
pub fn TagsCard(
    #[prop(into)] post: Signal<Option<Arc<Post>>>,
    #[prop(into)] editable: Signal<bool>,
    on_edit: Callback<()>,
    #[prop(optional)] on_voted: Option<Callback<Post>>,
) -> impl IntoView {
    // The tags alone, so a vote redraws its own score and keeps the focus.
    let shape = Memo::new(move |_| {
        post.with(|p| {
            p.as_ref()
                .map(|p| (p.id, p.tags.clone(), p.tag_details.clone()))
        })
    });
    let votes = on_voted.map(|on_voted| Votes {
        post,
        leaving: RwSignal::new(Vec::new()),
        on_voted,
    });
    let public = Memo::new(|_| crate::instance::public());
    let one = move |name: String, class: &'static str| match votes.filter(|_| public.get()) {
        Some(votes) => {
            let tag = name.clone();
            view! { <TagVoteChip tag=tag votes=votes>{chip(name, class)}</TagVoteChip> }.into_any()
        }
        None => chip(name, class).into_any(),
    };
    let tags = move || {
        let (_, names, details): (i64, Vec<String>, Vec<PostTag>) = shape.get()?;
        if names.is_empty() {
            return Some(view! { <p class="muted">{move || t!("post-no-tags")}</p> }.into_any());
        }
        // A post from a list has its names only.
        if details.is_empty() {
            let chips = names.into_iter().map(|t| one(t, "chip")).collect_view();
            return Some(view! { <div class="chips" data-press="tag">{chips}</div> }.into_any());
        }
        let groups = group_tags(&details);
        let headed = headed(&groups);
        let sections = groups
            .into_iter()
            .map(|(group, tags)| {
                let head = headed.then(|| view! { <h3>{group_label(&group)}</h3> });
                let chips = tags
                    .into_iter()
                    .map(|t| one(t.name, category_class(t.category)))
                    .collect_view();
                view! {
                    <section class="tag-group">
                        {head}
                        <div class="chips">{chips}</div>
                    </section>
                }
            })
            .collect_view();
        Some(view! { <div class="tag-groups" data-press="tag">{sections}</div> }.into_any())
    };
    view! {
        <section class="card">
            <div class="card-head">
                <h2 class="card-title">{move || t!("post-tags")}</h2>
                <Show when=move || editable.get()>
                    <button
                        type="button"
                        class="btn text small"
                        title=move || t!("post-edit-title")
                        aria-label=move || t!("post-edit-tags")
                        on:click=move |_| on_edit.run(())
                    >
                        <Ico icon=Icon::Edit size=18 />
                        {move || t!("post-edit")}
                    </button>
                </Show>
            </div>
            {tags}
        </section>
    }
}

/// The file: its rating, size, shape, type, dates, colours and source.
/// `sampled` says the stage shows the smaller copy, with a way to the
/// original.
#[component]
pub fn DetailsCard(
    #[prop(into)] post: Signal<Option<Arc<Post>>>,
    #[prop(into)] sampled: Signal<bool>,
    on_original: Callback<()>,
) -> impl IntoView {
    let facts = move || {
        let p = post.get()?;
        let length = (p.kind == MediaKind::Video)
            .then(|| p.duration_ms.and_then(format_duration))
            .flatten()
            .map(|d| {
                view! {
                    <dt>{move || t!("post-length")}</dt>
                    <dd>{d}</dd>
                }
            });
        let taken = p.file_date.map(|at| {
            let shown = file_date(p.kind, &at, &Local);
            view! {
                <dt>{move || t!("post-file-date")}</dt>
                <dd>{shown}</dd>
            }
        });
        // Only plain `#rrggbb`: each goes into a style attribute.
        let palette: Vec<&str> = p
            .palette
            .iter()
            .filter_map(|c| tint(std::slice::from_ref(c)))
            .collect();
        let colours = (!palette.is_empty()).then(|| {
            let swatches = palette
                .into_iter()
                .map(|hex| {
                    let color = hex.to_owned();
                    let title = move || t!("post-color-search", color = color.clone());
                    let label = title.clone();
                    view! {
                        <a
                            class="swatch"
                            href=color_href(hex)
                            style=format!("background-color: {hex}")
                            title=title
                            aria-label=label
                        ></a>
                    }
                })
                .collect_view();
            view! {
                <dt>{move || t!("post-colors")}</dt>
                <dd class="post-swatches">{swatches}</dd>
            }
        });
        let source = p.source.clone().filter(|s| !s.trim().is_empty()).map(|s| {
            let shown = match source_link(&s) {
                Some(url) => view! {
                    <a href=url.to_owned() target="_blank" rel="noopener noreferrer">
                        {s.trim().to_owned()}
                    </a>
                }
                .into_any(),
                None => s.into_any(),
            };
            view! {
                <dt>{move || t!("post-source")}</dt>
                <dd class="break">{shown}</dd>
            }
        });
        let rating = p.rating;
        // On a public server, who put it here.
        let uploader = p.uploader.clone().map(|card| {
            view! {
                <dt>{move || t!("post-uploader")}</dt>
                <dd class="post-uploader">
                    <UserLink card=Some(card) size=Size::Small />
                </dd>
            }
        });
        Some(view! {
            {uploader}
            <dt>{move || t!("rating")}</dt>
            <dd>{move || rating_label(rating)}</dd>
            <dt>{move || t!("post-size")}</dt>
            <dd>{human_size(p.file_size)}</dd>
            <dt>{move || t!("post-dimensions")}</dt>
            <dd>{dimensions(p.width, p.height)}</dd>
            {length}
            <dt>{move || t!("post-type")}</dt>
            <dd class="break">{p.mime.clone()}</dd>
            {taken}
            {colours}
            <dt>{move || t!("post-added")}</dt>
            <dd>{when(&p.created_at, &Local)}</dd>
            {source}
        })
    };
    view! {
        <section class="card">
            <h2 class="card-title">{move || t!("post-details")}</h2>
            <dl class="meta">{facts}</dl>
            <Show when=move || sampled.get()>
                <div class="sample-note">
                    <p class="muted hint">{move || t!("post-sample-note")}</p>
                    <button type="button" class="btn text" on:click=move |_| on_original.run(())>
                        {move || t!("post-show-original")}
                    </button>
                </div>
            </Show>
        </section>
    }
}
