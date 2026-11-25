//! A person of the collection as a round face over their name, for Explore
//! and the people page: the crop of their face, or else the thumbnail of
//! one of their posts cut to the circle, the franchise in small type and
//! how many posts they are on. It leads to the gallery searching for them,
//! and right-clicked it has a tag's menu (`data-tag`).

use leptos::prelude::*;
use starbooru_core::Person;

use super::icons::{Ico, Icon};
use crate::api;
use crate::logic::characters::{signed, spoken};
use crate::logic::search::short_count;

/// One person. `href` is where it leads; `votes`, given for popular
/// characters, shows as a badge; `franchise` says whether to name the
/// franchise, which a heading over it may say already. `index` staggers
/// the faces coming in.
#[component]
pub fn PersonFace(
    person: Person,
    href: String,
    #[prop(default = None)] votes: Option<i64>,
    #[prop(default = true)] franchise: bool,
    #[prop(optional)] index: usize,
) -> impl IntoView {
    let Person {
        name,
        posts,
        franchise: series,
        face_url,
        thumb_url,
        ..
    } = person;
    let words = spoken(&name);
    let count = u64::try_from(posts).unwrap_or(0);
    let posts_label = t!("people-posts", count = count);
    let label = match &series {
        Some(series) if franchise => t!(
            "people-person-of",
            name = words.clone(),
            franchise = spoken(series),
            posts = posts_label.clone()
        ),
        _ => t!(
            "people-person",
            name = words.clone(),
            posts = posts_label.clone()
        ),
    };
    let label = match votes {
        Some(v) => t!("people-person-voted", person = label, votes = signed(v)),
        None => label,
    };
    // The face, else a post's thumbnail; a picture that won't load gives
    // way to the icon.
    let cut = face_url.is_none() && thumb_url.is_some();
    let picture = face_url.or(thumb_url);
    let loaded = RwSignal::new(false);
    let broken = RwSignal::new(false);
    let face = move || match picture.clone().filter(|_| !broken.get()) {
        Some(path) => view! {
            <img
                src=api::media(&path)
                alt=""
                loading="lazy"
                decoding="async"
                draggable="false"
                on:load=move |_| loaded.set(true)
                on:error=move |_| broken.set(true)
            />
        }
        .into_any(),
        None => view! { <Ico icon=Icon::User size=32 /> }.into_any(),
    };
    let series = series.filter(|_| franchise).map(|s| {
        view! { <span class="person-franchise">{spoken(&s)}</span> }
    });
    let votes = votes.map(|v| {
        view! {
            <span class="person-votes">
                <Ico icon=Icon::ThumbUp size=14 />
                {signed(v)}
            </span>
        }
    });
    view! {
        <a
            class="person"
            href=href
            data-tag=name
            aria-label=label
            style=format!("--i: {}", index.min(30))
        >
            <span class="person-face" class:cut=cut class:loaded=move || loaded.get() aria-hidden="true">
                {face}
                <span class="person-count" title=posts_label>
                    {short_count(count)}
                </span>
            </span>
            <span class="person-name">{words}</span>
            {series}
            {votes}
        </a>
    }
}
