//! The search cheat sheet at `/help`: every term of the query language with
//! examples that run a search, a box that explains a query as it's typed,
//! and the viewer's keys and gestures.

use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
use starbooru_core::filter::SizeField;
use starbooru_core::{FigureTerm, MediaKind, Order, Rating, SortKey, parse_query};

use super::{figure_kind_label, query_error_text, rating_label};
use crate::components::icons::{Ico, Icon};
use crate::route;

/// The examples, section by section, as the cheat sheet lists them. Each is
/// a search that runs from its link; `sections` gives each its meaning.
const TAGS: [&str; 8] = [
    "blue_sky",
    "blue_sky night",
    "-cloud",
    "footwear",
    "artist:some_name",
    "{girl blonde_hair boots}",
    "blue sky",
    "highres -sound",
];
const FILTERS: [&str; 17] = [
    "rating:safe",
    "rating:safe rating:q",
    "-rating:e",
    "type:image",
    "status:deleted",
    "date:2024-03",
    "date:2023..2024",
    "date:>2023-06",
    "date:>30d",
    "width:>1920",
    "mpixels:>8",
    "ratio:16:9",
    "-height:<1000",
    "color:red",
    "color:2060e0",
    "-color:white",
    "exact:true",
];
const TEXT: [&str; 3] = ["text:hello", "text:\"no parking\"", "-text:sale"];
const SORTING: [&str; 9] = [
    "order:new",
    "order:score",
    "order:size",
    "order:pixels",
    "order:longside",
    "order:tags",
    "order:updated",
    "order:size_asc",
    "order:random",
];
const COMBINED: [&str; 2] = ["cat -dog rating:safe order:score", "type:video order:size"];

/// Pairs each example with its meaning; the lengths have to agree.
fn rows<const N: usize>(
    terms: [&'static str; N],
    meanings: [String; N],
) -> Vec<(&'static str, String)> {
    terms.into_iter().zip(meanings).collect()
}

/// Each section's heading and its examples with what they do.
fn sections() -> [(String, Vec<(&'static str, String)>); 5] {
    [
        (
            t!("help-tags"),
            rows(
                TAGS,
                [
                    t!("help-tag"),
                    t!("help-tag-all"),
                    t!("help-tag-not"),
                    t!("help-tag-tree"),
                    t!("help-tag-category"),
                    t!("help-figure"),
                    t!("help-tag-spaces"),
                    t!("help-tag-meta"),
                ],
            ),
        ),
        (
            t!("help-filters"),
            rows(
                FILTERS,
                [
                    t!("help-rating"),
                    t!("help-rating-any"),
                    t!("help-rating-not"),
                    t!("help-type"),
                    t!("help-deleted"),
                    t!("help-date"),
                    t!("help-date-range"),
                    t!("help-date-compare"),
                    t!("help-date-age"),
                    t!("help-size"),
                    t!("help-mpixels"),
                    t!("help-ratio"),
                    t!("help-size-not"),
                    t!("help-color"),
                    t!("help-color-hex"),
                    t!("help-color-not"),
                    t!("help-exact"),
                ],
            ),
        ),
        (
            t!("help-text"),
            rows(
                TEXT,
                [
                    t!("help-text-word"),
                    t!("help-text-phrase"),
                    t!("help-text-not"),
                ],
            ),
        ),
        (
            t!("help-sorting"),
            rows(
                SORTING,
                [
                    t!("help-order-new"),
                    t!("help-order-score"),
                    t!("help-order-size"),
                    t!("help-order-pixels"),
                    t!("help-order-longside"),
                    t!("help-order-tags"),
                    t!("help-order-updated"),
                    t!("help-order-direction"),
                    t!("help-order-random"),
                ],
            ),
        ),
        (
            t!("help-combined"),
            rows(
                COMBINED,
                [t!("help-combined-cats"), t!("help-combined-videos")],
            ),
        ),
    ]
}

/// The viewer's keys and gestures, as they work in this app, a video's
/// among them. Selection and the thumbnails' gestures add theirs when they
/// land.
fn shortcuts() -> [(String, String); 12] {
    [
        ("← →".into(), t!("key-prev-next")),
        ("E".into(), t!("key-edit")),
        ("F".into(), t!("key-fullscreen")),
        ("+ −".into(), t!("key-zoom")),
        ("0".into(), t!("key-fit")),
        ("Esc".into(), t!("key-escape")),
        (format!("{} K", t!("key-space")), t!("key-play")),
        ("J L".into(), t!("key-seek")),
        ("M".into(), t!("key-mute")),
        ("0 – 9".into(), t!("key-jump")),
        (t!("key-double-click"), t!("key-double-click-action")),
        (t!("key-swipe"), t!("key-swipe-action")),
    ]
}

#[component]
pub fn Help() -> impl IntoView {
    let text = RwSignal::new(String::new());
    let explained = Memo::new(move |_| text.with(|t| explain(t)));
    let navigate = use_navigate();
    let submit = move |e: SubmitEvent| {
        e.prevent_default();
        if explained.with_untracked(Result::is_ok) {
            navigate(
                &route::gallery(&text.get_untracked()),
                NavigateOptions::default(),
            );
        }
    };
    let reading = move || match explained.get() {
        Ok(lines) => view! {
            <ul class="explain">
                {lines.into_iter().map(|line| view! { <li>{line}</li> }).collect_view()}
            </ul>
        }
        .into_any(),
        Err(message) => view! { <p class="field-error">{message}</p> }.into_any(),
    };
    let cards = move || {
        sections()
            .into_iter()
            .map(|(title, examples)| {
                let rows = examples
                    .into_iter()
                    .map(|(term, meaning)| {
                        view! {
                            <dt>
                                <a
                                    class="code-chip"
                                    href=route::gallery(term)
                                    title=t!("help-search-for", term = term)
                                >
                                    {term}
                                </a>
                            </dt>
                            <dd>{meaning}</dd>
                        }
                    })
                    .collect_view();
                view! {
                    <section class="card">
                        <h2 class="card-title">{title}</h2>
                        <dl class="syntax">{rows}</dl>
                    </section>
                }
            })
            .collect_view()
    };
    let keys = move || {
        shortcuts()
            .into_iter()
            .map(|(keys, action)| {
                view! {
                    <dt>
                        <kbd>{keys}</kbd>
                    </dt>
                    <dd>{action}</dd>
                }
            })
            .collect_view()
    };
    view! {
        <div class="page narrow help">
            <h1 class="page-title">{move || t!("help-title")}</h1>
            <section class="card">
                <h2 class="card-title">{move || t!("help-try")}</h2>
                <form class="try-query" on:submit=submit>
                    <input
                        type="search"
                        placeholder=move || t!("help-try-placeholder")
                        aria-label=move || t!("help-try-label")
                        autocomplete="off"
                        spellcheck="false"
                        prop:value=move || text.get()
                        on:input=move |e| text.set(event_target_value(&e))
                    />
                    <button
                        class="btn"
                        type="submit"
                        disabled=move || explained.with(Result::is_err)
                    >
                        <Ico icon=Icon::Search />
                        {move || t!("help-search")}
                    </button>
                </form>
                // Always on the page, so a screen reader reads each change
                // once it's idle.
                <div class="explained" aria-live="polite">
                    {reading}
                </div>
            </section>
            {cards}
            <section class="card">
                <h2 class="card-title">{move || t!("help-tips")}</h2>
                <ul class="tips">
                    <li>{move || t!("help-tip-normalise")}</li>
                    <li>{move || t!("help-tip-aliases")}</li>
                    <li>{move || t!("help-tip-terms")}</li>
                </ul>
            </section>
            <section class="card">
                <h2 class="card-title">{move || t!("help-shortcuts")}</h2>
                <dl class="syntax">{keys}</dl>
            </section>
        </div>
    }
}

/// A query in plain words, one line per kind of term, or the parser's error.
pub fn explain(text: &str) -> Result<Vec<String>, String> {
    let q = parse_query(text).map_err(|e| query_error_text(&e))?;
    let mut lines = Vec::new();
    if q.deleted {
        lines.push(t!("explain-deleted"));
    }
    match q.include.as_slice() {
        [] => {}
        [tag] => lines.push(t!("explain-tagged", tag = tag.as_str())),
        tags => lines.push(t!("explain-tagged-all", tags = tags.join(", "))),
    }
    match q.exclude.as_slice() {
        [] => {}
        [tag] => lines.push(t!("explain-not-tagged", tag = tag.as_str())),
        tags => lines.push(t!("explain-tagged-none", tags = tags.join(", "))),
    }
    if !q.ratings.is_empty() {
        lines.push(t!("explain-rated", ratings = ratings(&q.ratings)));
    }
    if !q.exclude_ratings.is_empty() {
        lines.push(t!(
            "explain-not-rated",
            ratings = ratings(&q.exclude_ratings)
        ));
    }
    for term in &q.figures {
        lines.push(figure_line(term, false));
    }
    for term in &q.exclude_figures {
        lines.push(figure_line(term, true));
    }
    for date in &q.dates {
        lines.push(t!("explain-date", value = date.raw.as_str()));
    }
    for date in &q.exclude_dates {
        lines.push(t!("explain-not-date", value = date.raw.as_str()));
    }
    for size in &q.sizes {
        lines.push(t!(
            "explain-size",
            field = size_label(size.field),
            value = size.raw.as_str()
        ));
    }
    for size in &q.exclude_sizes {
        lines.push(t!(
            "explain-not-size",
            field = size_label(size.field),
            value = size.raw.as_str()
        ));
    }
    for color in &q.colors {
        lines.push(t!("explain-color", value = color.raw.as_str()));
    }
    for color in &q.exclude_colors {
        lines.push(t!("explain-not-color", value = color.raw.as_str()));
    }
    for text in &q.text {
        lines.push(t!("explain-text", text = text.as_str()));
    }
    for text in &q.exclude_text {
        lines.push(t!("explain-not-text", text = text.as_str()));
    }
    if let Some(kind) = q.kind {
        lines.push(match kind {
            MediaKind::Image => t!("explain-images"),
            MediaKind::Animation => t!("explain-animations"),
            MediaKind::Video => t!("explain-videos"),
        });
    }
    if lines.is_empty() || (lines.len() == 1 && q.deleted) {
        lines.push(t!("explain-every"));
    }
    if q.exact {
        lines.push(t!("explain-exact"));
    }
    lines.push(order_phrase(q.order));
    Ok(lines)
}

fn size_label(field: SizeField) -> String {
    match field {
        SizeField::Width => t!("explain-size-width"),
        SizeField::Height => t!("explain-size-height"),
        SizeField::Pixels => t!("explain-size-mpixels"),
        SizeField::Ratio => t!("explain-size-ratio"),
    }
}

/// One `{...}` term in words.
fn figure_line(term: &FigureTerm, negated: bool) -> String {
    let tags = term.tags.join(", ");
    let kind = term.kind.map(|k| figure_kind_label(k).to_lowercase());
    match (kind, negated) {
        (None, false) => t!("explain-figure", tags = tags),
        (None, true) => t!("explain-no-figure", tags = tags),
        (Some(kind), false) if term.tags.is_empty() => t!("explain-figure-only", kind = kind),
        (Some(kind), true) if term.tags.is_empty() => t!("explain-no-figure-only", kind = kind),
        (Some(kind), false) => t!("explain-figure-kind", kind = kind, tags = tags),
        (Some(kind), true) => t!("explain-no-figure-kind", kind = kind, tags = tags),
    }
}

/// "safe, sensitive or questionable".
fn ratings(list: &[Rating]) -> String {
    let names: Vec<String> = list.iter().map(|r| rating_label(*r)).collect();
    match names.split_last() {
        Some((last, rest)) if !rest.is_empty() => {
            t!("list-or", rest = rest.join(", "), last = last.as_str())
        }
        _ => names.concat(),
    }
}

fn order_phrase(order: Order) -> String {
    let d = order.descending;
    match order.key {
        SortKey::Id if d => t!("explain-order-new"),
        SortKey::Id => t!("explain-order-old"),
        SortKey::Random => t!("explain-order-random"),
        SortKey::Score if d => t!("explain-order-score-desc"),
        SortKey::Score => t!("explain-order-score-asc"),
        SortKey::Size if d => t!("explain-order-size-desc"),
        SortKey::Size => t!("explain-order-size-asc"),
        SortKey::Pixels if d => t!("explain-order-pixels-desc"),
        SortKey::Pixels => t!("explain-order-pixels-asc"),
        SortKey::LongSide if d => t!("explain-order-longside-desc"),
        SortKey::LongSide => t!("explain-order-longside-asc"),
        SortKey::TagCount if d => t!("explain-order-tags-desc"),
        SortKey::TagCount => t!("explain-order-tags-asc"),
        SortKey::Updated if d => t!("explain-order-updated-desc"),
        SortKey::Updated => t!("explain-order-updated-asc"),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    const EXAMPLES: [&[&str]; 5] = [&TAGS, &FILTERS, &TEXT, &SORTING, &COMBINED];

    #[test]
    fn every_example_parses() {
        for term in EXAMPLES.iter().flat_map(|s| s.iter()) {
            assert!(parse_query(term).is_ok(), "{term}");
            assert!(explain(term).is_ok(), "{term}");
        }
    }

    #[test]
    fn every_example_has_its_meaning_and_comes_once() {
        let shown: Vec<&str> = sections()
            .iter()
            .flat_map(|(_, rows)| rows.iter().map(|(term, _)| *term))
            .collect();
        let listed: Vec<&str> = EXAMPLES.iter().flat_map(|s| s.iter().copied()).collect();
        assert_eq!(shown, listed);
        assert_eq!(listed.iter().collect::<HashSet<_>>().len(), listed.len());
        for (_, rows) in sections() {
            for (term, meaning) in rows {
                assert!(!meaning.starts_with("help-"), "{term}: {meaning}");
            }
        }
    }

    #[test]
    fn each_example_links_to_its_search() {
        assert_eq!(route::gallery("blue_sky night"), "/?q=blue_sky+night");
        assert_eq!(route::gallery("-cloud"), "/?q=-cloud");
        assert_eq!(
            route::gallery("text:\"no parking\""),
            "/?q=text:%22no+parking%22"
        );
        assert_eq!(route::gallery("date:>30d"), "/?q=date:%3E30d");
    }

    #[test]
    fn explains_exact() {
        let exact = "Tags are taken exactly as typed".to_owned();
        assert!(explain("exact:true cat").unwrap().contains(&exact));
        assert!(!explain("cat").unwrap().contains(&exact));
        // With nothing else, it still searches everything.
        assert_eq!(
            explain("exact:true").unwrap(),
            ["Every post".to_owned(), exact, "Newest first".to_owned()]
        );
    }

    #[test]
    fn explains_colors() {
        assert_eq!(
            explain("color:RED -color:#2060e0").unwrap(),
            [
                "Has the colour red, or the nearest one stored",
                "Doesn't have the colour 2060e0",
                "Newest first"
            ]
        );
        assert!(explain("color:bluish").is_err());
    }

    #[test]
    fn explains_queries() {
        assert_eq!(explain("").unwrap(), ["Every post", "Newest first"]);
        assert_eq!(
            explain("cat -dog -bird rating:g rating:s -rating:e type:video order:size_asc")
                .unwrap(),
            [
                "Tagged cat",
                "Tagged with none of dog, bird",
                "Rated safe or sensitive",
                "Not rated explicit",
                "Videos only",
                "By file size, smallest first",
            ]
        );
        assert_eq!(
            explain("status:deleted order:random").unwrap(),
            ["In the recycle bin", "Every post", "In random order"]
        );
        assert_eq!(
            explain("a b c order:old").unwrap(),
            ["Tagged with all of a, b, c", "Oldest first"]
        );
        assert_eq!(
            explain("{girl boots hat} {scarf} -{boy cap} -{sock} {other} -{girl}").unwrap(),
            [
                "A person (girl) has all of boots, hat",
                "One person has all of scarf",
                "A person marked as other",
                "No person (boy) has all of cap",
                "No one has all of sock",
                "No person marked as girl",
                "Newest first"
            ]
        );
        assert_eq!(
            explain(r#"text:"no parking" -text:sale"#).unwrap(),
            [
                "Text in the image has “no parking”",
                "Text in the image doesn’t have “sale”",
                "Newest first"
            ]
        );
        assert_eq!(
            explain("date:2024-03 -date:>30d width:>1920 -ratio:16:9 mpixels:8 height:1080 order:longside_asc")
                .unwrap(),
            [
                "Dated 2024-03",
                "Not dated >30d",
                "Width >1920",
                "Megapixels 8",
                "Height 1080",
                "Aspect ratio not 16:9",
                "By longest side, shortest first",
            ]
        );
        assert!(explain("date:soon").unwrap_err().contains("date:soon"));
        assert!(explain("width:wide").unwrap_err().contains("width:wide"));
        let three = explain("rating:g rating:s rating:q").unwrap();
        assert_eq!(three[0], "Rated safe, sensitive or questionable");
        assert!(
            explain("rating:z")
                .unwrap_err()
                .starts_with("Unknown rating")
        );
    }

    #[test]
    fn the_reading_is_a_live_region_that_doesnt_shout() {
        let source = include_str!("help.rs");
        assert!(source.contains(r#"<div class="explained" aria-live="polite">"#));
        // Split so this line doesn't match itself.
        assert!(!source.contains(concat!(r#"role=""#, r#"alert""#)));
    }

    #[test]
    fn the_shortcuts_are_the_viewers() {
        // Each key listed is one the viewer answers (post/stage.rs).
        let stage = include_str!("post/stage.rs");
        for key in [
            r#""ArrowLeft""#,
            r#""ArrowRight""#,
            r#""f" | "F""#,
            r#""+" | "=""#,
            r#""-" | "_""#,
            r#""0""#,
            r#""Escape""#,
            "on:dblclick",
            "swipe_turns",
        ] {
            assert!(stage.contains(key), "{key}");
        }
        // And a video's, which its player answers.
        use crate::logic::player::{Act, Press, act};
        let press = |key| {
            act(Press {
                key,
                ..Press::default()
            })
        };
        assert_eq!(press(" "), Some(Act::TogglePlay));
        assert_eq!(press("k"), Some(Act::TogglePlay));
        assert_eq!(press("j"), Some(Act::Seek(-5)));
        assert_eq!(press("l"), Some(Act::Seek(5)));
        assert_eq!(press("m"), Some(Act::ToggleMute));
        assert_eq!(press("9"), Some(Act::Jump(0.9)));
        let shifted = act(Press {
            key: "ArrowRight",
            shift: true,
            ..Press::default()
        });
        assert_eq!(shifted, Some(Act::Seek(5)));
        let player = include_str!("../components/player.rs");
        assert!(player.contains("on:dblclick"));
    }
}
