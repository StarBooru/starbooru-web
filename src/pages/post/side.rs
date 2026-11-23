//! The viewer's side column: the post's tags under their headings, and what
//! the server knows of its file.

use std::sync::Arc;

use chrono::Local;
use leptos::prelude::*;
use starbooru_core::{MediaKind, Post};

use crate::components::icons::{Ico, Icon};
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

fn group_label(group: &Group) -> String {
    match group {
        Group::Category(c) => category_label(*c),
        Group::Section(name) => section_label(name),
        Group::Other => t!("section-other"),
    }
}

/// A tag that searches for itself. Its menu (`components/tag_menu.rs`) has
/// the way to its page.
fn chip(name: String, class: &'static str) -> impl IntoView {
    let href = route::gallery(&name);
    let key = name.clone();
    view! {
        <a class=class href=href data-tag=key>
            {name}
        </a>
    }
}

/// The tags, grouped when the server sent their categories, and the way to
/// edit them for whoever `editable` says may.
#[component]
pub fn TagsCard(
    #[prop(into)] post: Signal<Option<Arc<Post>>>,
    #[prop(into)] editable: Signal<bool>,
    on_edit: Callback<()>,
) -> impl IntoView {
    let tags = move || {
        let p = post.get()?;
        if p.tags.is_empty() {
            return Some(view! { <p class="muted">{move || t!("post-no-tags")}</p> }.into_any());
        }
        // A post from a list has its names only.
        if p.tag_details.is_empty() {
            let chips = p
                .tags
                .iter()
                .map(|t| chip(t.clone(), "chip"))
                .collect_view();
            return Some(view! { <div class="chips" data-press="tag">{chips}</div> }.into_any());
        }
        let groups = group_tags(&p.tag_details);
        let headed = headed(&groups);
        let sections = groups
            .into_iter()
            .map(|(group, tags)| {
                let head = headed.then(|| view! { <h3>{group_label(&group)}</h3> });
                let chips = tags
                    .into_iter()
                    .map(|t| chip(t.name, category_class(t.category)))
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
        let colours = (!p.palette.is_empty()).then(|| {
            let swatches = p
                .palette
                .iter()
                .map(|hex| {
                    let color = hex.clone();
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
        Some(view! {
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
