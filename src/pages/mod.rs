//! The app's pages. `app.rs` says which address shows which.

pub mod admin;
pub mod boorus;
mod bulk;
pub mod duplicates;
pub mod explore;
pub mod gallery;
pub mod help;
pub mod legal;
mod library;
pub mod login;
pub mod post;
mod post_menu;
mod saved;
mod search_demo;
pub mod settings;
pub mod share;
pub mod shell;
pub mod tags;
pub mod trash;
pub mod upload;

use leptos::prelude::*;
use leptos_router::hooks::use_location;
use starbooru_core::query::{MAX_QUERY_TERMS, MAX_TEXT_CHARS, QueryError};
use starbooru_core::saved::{MAX_SAVED_QUERY_CHARS, MAX_SEARCH_NAME_CHARS, SavedSearchError};
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

/// Why a search doesn't read, in words. The search box, the gallery and the
/// help page say it before the server would.
pub(crate) fn query_error_text(e: &QueryError) -> String {
    let value = |v: &String| v.clone();
    match e {
        QueryError::TooManyTerms => t!("query-error-terms", max = MAX_QUERY_TERMS),
        QueryError::BadRating(v) => t!("query-error-rating", value = value(v)),
        QueryError::BadKind(v) => t!("query-error-kind", value = value(v)),
        QueryError::BadStatus(v) => t!("query-error-status", value = value(v)),
        QueryError::BadOrder(v) => t!("query-error-order", value = value(v)),
        QueryError::EmptyText => t!("query-error-text-empty"),
        QueryError::TextTooLong => t!("query-error-text-long", max = MAX_TEXT_CHARS),
        QueryError::EmptyFigure => t!("query-error-figure-empty"),
        QueryError::TwoKinds => t!("query-error-figure-kinds"),
        QueryError::BadDate(v) => t!("query-error-date", value = value(v)),
        QueryError::BadSize(field, v) => {
            t!("query-error-size", field = value(field), value = value(v))
        }
        QueryError::BadColor(v) => t!("query-error-color", value = value(v)),
        QueryError::Tag(TagError::BadPrefix(tag)) => t!("tag-error-prefix", tag = value(tag)),
        QueryError::Tag(TagError::TooLong) => t!("query-error-tag-long", max = MAX_TAG_CHARS),
        QueryError::Tag(TagError::TooMany) => t!("tag-error-many", max = MAX_TAGS_PER_POST),
        QueryError::Tag(TagError::ControlChar | TagError::Empty) => t!("query-error-tag-other"),
    }
}

/// Why a search can't be saved, in words.
pub(crate) fn saved_search_error_text(e: &SavedSearchError) -> String {
    match e {
        SavedSearchError::EmptyName => t!("saved-error-no-name"),
        SavedSearchError::NameTooLong => t!("saved-error-name-long", max = MAX_SEARCH_NAME_CHARS),
        SavedSearchError::ControlChar => t!("saved-error-control"),
        SavedSearchError::EmptyQuery => t!("saved-error-empty"),
        SavedSearchError::QueryTooLong => t!("saved-error-long", max = MAX_SAVED_QUERY_CHARS),
        SavedSearchError::Query(e) => query_error_text(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One of each, so a new kind of error in the crate shows up here.
    fn every_query_error() -> Vec<QueryError> {
        let v = |s: &str| s.to_owned();
        vec![
            QueryError::TooManyTerms,
            QueryError::BadRating(v("z")),
            QueryError::BadKind(v("gif")),
            QueryError::BadStatus(v("gone")),
            QueryError::BadOrder(v("best")),
            QueryError::EmptyText,
            QueryError::TextTooLong,
            QueryError::EmptyFigure,
            QueryError::TwoKinds,
            QueryError::BadDate(v("soon")),
            QueryError::BadSize(v("width"), v("wide")),
            QueryError::BadColor(v("bluish")),
            QueryError::Tag(TagError::BadPrefix(v("~cat"))),
            QueryError::Tag(TagError::TooLong),
            QueryError::Tag(TagError::TooMany),
            QueryError::Tag(TagError::ControlChar),
            QueryError::Tag(TagError::Empty),
        ]
    }

    #[test]
    fn every_search_error_has_words_of_its_own() {
        let all = every_query_error();
        let texts: Vec<String> = all.iter().map(query_error_text).collect();
        for (e, text) in all.iter().zip(&texts) {
            // An id that isn't in the catalogue comes back as itself.
            assert!(
                !text.starts_with("query-error-") && !text.is_empty(),
                "{e:?}"
            );
            // Never the crate's English.
            assert_ne!(*text, e.to_string(), "{e:?}");
        }
        for (e, word) in [
            (0, "40"),
            (1, "z"),
            (2, "gif"),
            (3, "gone"),
            (4, "best"),
            (6, "200"),
            (9, "date:soon"),
            (10, "width:wide"),
            (11, "bluish"),
            (12, "~cat"),
            (13, "128"),
        ] {
            assert!(texts[e].contains(word), "{}: {}", word, texts[e]);
        }
        // Tag troubles without the tag still say something of their own.
        assert_ne!(texts[15], texts[13]);
        let saved = [
            SavedSearchError::EmptyName,
            SavedSearchError::NameTooLong,
            SavedSearchError::ControlChar,
            SavedSearchError::EmptyQuery,
            SavedSearchError::QueryTooLong,
            SavedSearchError::Query(QueryError::EmptyText),
        ];
        for e in &saved {
            let text = saved_search_error_text(e);
            assert!(
                !text.starts_with("saved-error-") && text != e.to_string(),
                "{e:?}"
            );
        }
        assert!(saved_search_error_text(&SavedSearchError::QueryTooLong).contains("1000"));
    }

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
