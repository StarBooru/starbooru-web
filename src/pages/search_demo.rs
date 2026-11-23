//! What the empty search box shows in the gallery: someone typing searches
//! into it, each with a line on what it does, a walk through the cheat sheet
//! (`help.rs`).
//!
//! It rests while the box has focus, the tab is hidden or the page is
//! scrolled down, ends when the box gets a search or the page changes, and
//! doesn't start under reduced motion or the battery saver: the input's own
//! placeholder shows then.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::dom;
use crate::logic::search::{common_prefix, demo_split, key_delay};

/// The tag the examples search for.
const TAG: &str = "ishtar_(fate)";

/// From the simplest search to all of it at once, as the cheat sheet goes.
fn examples() -> [(&'static str, String); 18] {
    [
        ("ishtar_(fate)", t!("demo-tag")),
        ("ishtar_(fate) smile", t!("demo-tag-all")),
        ("ishtar_(fate) -rin_tohsaka", t!("demo-tag-not")),
        ("{ishtar_(fate) red_eyes}", t!("demo-figure")),
        ("ishtar_(fate) rating:safe", t!("demo-rating")),
        ("ishtar_(fate) rating:safe rating:q", t!("demo-rating-any")),
        ("ishtar_(fate) -rating:e", t!("demo-rating-not")),
        ("ishtar_(fate) type:video", t!("demo-type")),
        ("ishtar_(fate) date:2024-03", t!("demo-date")),
        ("ishtar_(fate) date:2023..2024", t!("demo-date-range")),
        ("ishtar_(fate) date:<7d", t!("demo-date-age")),
        ("ishtar_(fate) width:>1920", t!("demo-size")),
        ("ishtar_(fate) ratio:16:9", t!("demo-ratio")),
        ("ishtar_(fate) color:red", t!("demo-color")),
        ("ishtar_(fate) text:\"gold\"", t!("demo-text")),
        ("ishtar_(fate) order:score", t!("demo-order-score")),
        ("ishtar_(fate) order:random", t!("demo-order-random")),
        (
            "ishtar_(fate) -rin_tohsaka rating:safe order:score",
            t!("demo-combined"),
        ),
    ]
}

/// The gallery is where a search goes, so that's where it's taught.
pub fn plays_on(path: &str) -> bool {
    matches!(path, "" | "/")
}

/// Less motion asked for, or the battery saver on.
fn calm() -> bool {
    dom::reduced_motion()
        || web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.query_selector(r#".app[data-saver="on"]"#).ok().flatten())
            .is_some()
}

fn tab_hidden() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .is_some_and(|d| d.hidden())
}

/// The demo as it plays. `typing` puts `data-typing` on the search box,
/// which hides the placeholder while the demo shows.
#[derive(Clone)]
struct Demo {
    alive: Arc<AtomicBool>,
    typing: RwSignal<bool>,
    focused: Signal<bool>,
    base: RwSignal<String>,
    term: RwSignal<String>,
    note: RwSignal<String>,
    shown: RwSignal<bool>,
    line: NodeRef<html::Span>,
}

impl Demo {
    fn alive(&self) -> bool {
        self.alive.load(Ordering::Relaxed) && !calm()
    }

    /// Away from the top of the page the demo gives way to the placeholder.
    fn at_top(&self) -> bool {
        let (y, _) = dom::scroll();
        let top = y <= 8.0;
        if self.typing.try_get_untracked() != Some(top) {
            let _ = self.typing.try_set(top);
        }
        top
    }

    fn resting(&self) -> bool {
        !self.at_top() || tab_hidden() || self.focused.try_get_untracked().unwrap_or(true)
    }

    /// Waits `ms` in short steps, so that leaving shows at once, then for as
    /// long as it should rest. `false` once the demo has to stop.
    async fn beat(&self, ms: u32) -> bool {
        let mut left = ms;
        while left > 0 {
            let step = left.min(100);
            dom::sleep(i32::try_from(step).unwrap_or(100)).await;
            left -= step;
            if !self.alive() {
                return false;
            }
            self.at_top();
        }
        while self.alive() && self.resting() {
            dom::sleep(250).await;
        }
        self.alive()
    }

    fn show(&self, text: &str) {
        let (base, term) = demo_split(TAG, text);
        self.base.set(base.to_owned());
        self.term.set(term.to_owned());
        // The end of a long example stays in sight.
        let line = self.line;
        request_animation_frame(move || {
            if let Some(el) = line.try_get_untracked().flatten() {
                el.set_scroll_left(el.scroll_width());
            }
        });
    }

    async fn play(&self) {
        if !self.beat(700).await {
            return;
        }
        let mut typed = String::new();
        loop {
            for (query, meaning) in examples() {
                self.shown.set(false);
                let same = common_prefix(&typed, query);
                while typed.chars().count() > same {
                    typed.pop();
                    self.show(&typed);
                    if !self.beat(24).await {
                        return;
                    }
                }
                for c in query.chars().skip(same) {
                    typed.push(c);
                    self.show(&typed);
                    if !self.beat(key_delay(c, js_sys::Math::random())).await {
                        return;
                    }
                }
                self.note.set(meaning);
                self.shown.set(true);
                if !self.beat(2600).await {
                    return;
                }
            }
        }
    }
}

/// `typing` is the search box's `data-typing`; `focused` whether the box has
/// the focus, which the demo waits out.
#[component]
pub fn SearchDemo(typing: RwSignal<bool>, #[prop(into)] focused: Signal<bool>) -> impl IntoView {
    let base = RwSignal::new(String::new());
    let term = RwSignal::new(String::new());
    let note = RwSignal::new(String::new());
    let shown = RwSignal::new(false);
    let line: NodeRef<html::Span> = NodeRef::new();
    let alive = Arc::new(AtomicBool::new(true));
    if !calm() {
        let demo = Demo {
            alive: Arc::clone(&alive),
            typing,
            focused,
            base,
            term,
            note,
            shown,
            line,
        };
        spawn_local(async move {
            demo.at_top();
            demo.play().await;
            // Stopped for less motion: the placeholder comes back. A demo that
            // was taken away leaves the box to the next one.
            if demo.alive.load(Ordering::Relaxed) {
                let _ = demo.typing.try_set(false);
            }
        });
    }
    on_cleanup(move || {
        alive.store(false, Ordering::Relaxed);
        let _ = typing.try_set(false);
    });
    view! {
        <div class="search-demo" aria-hidden="true">
            <span class="demo-line" node_ref=line>
                <span class="demo-base">{move || base.get()}</span>
                <span class="demo-term">{move || term.get()}</span>
                <span class="demo-caret"></span>
            </span>
            <span class="demo-note" class:shown=move || shown.get()>
                {move || note.get()}
            </span>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use starbooru_core::parse_query;

    use super::*;

    #[test]
    fn it_plays_only_in_the_gallery() {
        assert!(plays_on("/"));
        assert!(plays_on(""));
        for path in [
            "/post/1",
            "/explore",
            "/tags",
            "/upload",
            "/settings",
            "/help",
        ] {
            assert!(!plays_on(path), "{path}");
        }
    }

    #[test]
    fn every_example_is_a_search_that_works() {
        for (query, _) in examples() {
            assert!(parse_query(query).is_ok(), "{query}");
            assert!(query.contains(TAG), "{query}");
        }
    }

    #[test]
    fn no_two_examples_are_the_same() {
        let queries = examples().map(|(query, _)| query);
        for (i, query) in queries.iter().enumerate() {
            assert!(!queries[..i].contains(query), "{query}");
        }
    }

    #[test]
    fn the_lines_stay_short_enough_for_a_phone() {
        for lang in crate::i18n::Lang::ALL {
            crate::i18n::set_lang(lang);
            for (query, meaning) in examples() {
                assert!(meaning.chars().count() <= 40, "{query}: {meaning}");
            }
        }
        crate::i18n::set_lang(crate::i18n::Lang::English);
    }

    #[test]
    fn the_demo_keeps_to_calm_motion_and_stops_for_the_user() {
        let source = include_str!("search_demo.rs");
        assert!(source.contains("dom::reduced_motion()"));
        assert!(source.contains(r#".app[data-saver="on"]"#));
        assert!(source.contains("d.hidden()"));
        assert!(source.contains("y <= 8.0"));
        let css = include_str!("../../style/shell.css");
        assert!(css.contains(".search:focus-within .search-demo { visibility: hidden; }"));
        assert!(css.contains(".search[data-typing]:not(:focus-within) input::placeholder"));
        assert!(css.contains(".search:not([data-typing]) .search-demo"));
        // `--primary` falls to 3.5:1 on the light glass over something black.
        assert!(css.contains(".demo-term { color: var(--on-primary-container);"));
    }
}
