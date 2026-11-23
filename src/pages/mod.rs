//! The app's pages. `app.rs` says which address shows which.

pub mod boorus;
mod bulk;
pub mod duplicates;
pub mod explore;
pub mod gallery;
pub mod help;
mod library;
pub mod login;
pub mod post;
mod post_menu;
mod saved;
mod search_demo;
pub mod settings;
pub mod share;
pub mod shell;
pub mod soon;
pub mod tags;
pub mod trash;
pub mod upload;

use leptos::prelude::*;
use leptos_router::hooks::use_location;
use starbooru_core::tag::{MAX_TAG_CHARS, MAX_TAGS_PER_POST, TagError};
use starbooru_core::{FigureKind, Rating, TagCategory};

use crate::logic::tags::BadTag;

/// An address that leads nowhere.
#[component]
pub fn NotFound() -> impl IntoView {
    let path = use_location().pathname;
    view! {
        <div class="solo-page">
            <div class="card empty">
                <h1>{move || t!("notfound-title")}</h1>
                <p class="muted break">{move || t!("notfound-body", path = path.get())}</p>
                <a class="btn" href="/">
                    {move || t!("notfound-back")}
                </a>
            </div>
        </div>
    }
}

/// A rating's name in the current language.
pub(crate) fn rating_label(rating: Rating) -> String {
    match rating {
        Rating::Safe => t!("rating-safe"),
        Rating::Sensitive => t!("rating-sensitive"),
        Rating::Questionable => t!("rating-questionable"),
        Rating::Explicit => t!("rating-explicit"),
    }
}

/// A tag category's name in the current language.
pub(crate) fn category_label(category: TagCategory) -> String {
    match category {
        TagCategory::General => t!("category-general"),
        TagCategory::Artist => t!("category-artist"),
        TagCategory::Character => t!("category-character"),
        TagCategory::Copyright => t!("category-copyright"),
        TagCategory::Meta => t!("category-meta"),
    }
}

/// A kind of person in a picture, in the current language.
pub(crate) fn figure_kind_label(kind: FigureKind) -> String {
    match kind {
        FigureKind::Girl => t!("people-girl"),
        FigureKind::Boy => t!("people-boy"),
        FigureKind::Other => t!("people-other"),
    }
}

/// The categories, in the order the pages list them.
pub(crate) const CATEGORIES: [TagCategory; 5] = [
    TagCategory::General,
    TagCategory::Artist,
    TagCategory::Character,
    TagCategory::Copyright,
    TagCategory::Meta,
];

/// Why a word can't be a tag, in words.
pub(crate) fn bad_tag_text(bad: &BadTag) -> String {
    let tag = bad.word.clone();
    match bad.error {
        TagError::BadPrefix(_) => t!("tag-error-prefix", tag = tag),
        TagError::TooLong => t!("tag-error-long", tag = tag, max = MAX_TAG_CHARS),
        TagError::TooMany => t!("tag-error-many", max = MAX_TAGS_PER_POST),
        TagError::ControlChar | TagError::Empty => t!("tag-error-other", tag = tag),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bad_tag_says_which_and_why() {
        let bad = |word: &str, error| BadTag {
            word: word.into(),
            error,
        };
        assert!(bad_tag_text(&bad("-dog", TagError::BadPrefix("-dog".into()))).contains("-dog"));
        assert!(bad_tag_text(&bad("x", TagError::TooLong)).contains("128"));
        assert!(bad_tag_text(&bad("", TagError::TooMany)).contains("512"));
    }
}
