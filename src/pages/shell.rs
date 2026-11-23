//! The frame every page lives in: floating capsules for the brand and the
//! search, and a slider for the navigation that sits at the top right of a
//! wide window, stands up as a rail at the side of a narrower one and floats
//! at the bottom of a phone (`style/shell.css` has the breakpoints).

use std::cmp::Ordering;

use leptos::ev::SubmitEvent;
use leptos::html;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::components::Outlet;
use leptos_router::hooks::{use_location, use_navigate, use_query_map};

use crate::components::brand::{Mark, Wordmark};
use crate::components::icons::{Ico, Icon};
use crate::route::{self, Tab};

/// The pages under the navigation. Item 1.3 puts the sign-in screen in
/// front of it: until someone is signed in, that shows instead.
#[component]
pub fn Shell() -> impl IntoView {
    let location = use_location();
    let path = location.pathname;
    let here = Memo::new(move |_| path.with(|p| route::tab_of(p)));
    view! {
        <header class="topbar">
            <a class="brand glass float" href="/" aria-label=move || t!("app-name")>
                <Mark height=30 compact=true />
                <Wordmark height=19 />
            </a>
            <SearchBox />
            <NavSlider place="nav-top" here=here />
        </header>
        <main class="content">
            <Outlet />
        </main>
        <NavSlider place="nav-bottom" here=here />
    }
}

/// The search: Enter shows the gallery for what was typed, and an empty box
/// all of it. It shows the search of the page it's on.
#[component]
fn SearchBox() -> impl IntoView {
    let query = use_query_map();
    let current = Memo::new(move |_| query.with(|q| q.get("q").unwrap_or_default()));
    let text = RwSignal::new(current.get_untracked());
    // A new page, or Back, brings its own search.
    Effect::watch(
        move || current.get(),
        move |q, _, _| text.set(q.clone()),
        false,
    );
    let input: NodeRef<html::Input> = NodeRef::new();
    let navigate = use_navigate();
    let submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        navigate(
            &route::gallery(&text.get_untracked()),
            NavigateOptions::default(),
        );
        // On a phone, this puts the keyboard away.
        if let Some(input) = input.get_untracked() {
            let _ = input.blur();
        }
    };
    let clear = move |_| {
        text.set(String::new());
        if let Some(input) = input.get_untracked() {
            let _ = input.focus();
        }
    };
    view! {
        <form class="search glass float" role="search" on:submit=submit>
            // Phones: the crown takes the search icon's place, and the way home with it.
            <a class="search-home" href="/" aria-label=move || t!("app-name")>
                <Mark height=28 compact=true />
            </a>
            <Ico icon=Icon::Search />
            <div class="search-field">
                <input
                    node_ref=input
                    type="search"
                    enterkeyhint="search"
                    autocomplete="off"
                    spellcheck="false"
                    placeholder=move || t!("search-placeholder")
                    aria-label=move || t!("search-label")
                    bind:value=text
                />
            </div>
            <Show when=move || text.with(|t| !t.is_empty())>
                <button
                    type="button"
                    class="icon-btn search-clear"
                    aria-label=move || t!("search-clear")
                    title=move || t!("search-clear")
                    on:click=clear
                >
                    <Ico icon=Icon::Close />
                </button>
            </Show>
        </form>
    }
}

/// The navigation's items in order, with their icons and names.
fn items() -> [(Tab, Icon, String); 5] {
    [
        (Tab::Gallery, Icon::Gallery, t!("nav-gallery")),
        (Tab::Explore, Icon::Explore, t!("nav-explore")),
        (Tab::Tags, Icon::Label, t!("nav-tags")),
        (Tab::Upload, Icon::Upload, t!("nav-upload")),
        (Tab::Settings, Icon::Tune, t!("nav-settings-tab")),
    ]
}

/// Which way the thumb went: `next` when it moved on to a later item, `back`
/// to an earlier one, and the way it last went when it stayed. The way says
/// which of its edges leads.
fn way(was: usize, at: usize, last: &'static str) -> &'static str {
    match at.cmp(&was) {
        Ordering::Greater => "next",
        Ordering::Less => "back",
        Ordering::Equal => last,
    }
}

/// `place` is the class that says where this copy of the navigation shows:
/// `nav-top` or `nav-bottom`, which also makes the rail.
#[component]
fn NavSlider(place: &'static str, here: Memo<Option<Tab>>) -> impl IntoView {
    let index = move || {
        here.get()
            .and_then(|tab| Tab::ALL.iter().position(|t| *t == tab))
    };
    // Where the thumb stands and the way it last went. With no item selected
    // it keeps its place, out of sight.
    let thumb = Memo::new(move |last: Option<&(usize, &'static str)>| {
        let (was, last_way) = last.copied().unwrap_or((index().unwrap_or(0), "next"));
        let at = index().unwrap_or(was);
        (at, way(was, at, last_way))
    });
    let cells = Tab::ALL.len();
    view! {
        <nav
            class=format!("{place} slider glass float")
            aria-label=move || t!("nav-main")
            style=move || format!("--n: {cells}; --i: {}", thumb.get().0)
            data-dir=move || thumb.get().1
        >
            <span class="slider-thumb" class:off=move || here.get().is_none()>
                <span class="slider-clip">
                    <span class="slider-unclip">
                        <span class="slider-fill"></span>
                    </span>
                </span>
            </span>
            {move || {
                items()
                    .into_iter()
                    .map(|(tab, icon, label)| {
                        let active = move || here.get() == Some(tab);
                        view! {
                            <a
                                class="nav-item"
                                class:active=active
                                href=tab.href()
                                aria-current=move || if active() { "page" } else { "false" }
                            >
                                <span class="nav-icon">
                                    // Its own box for the pop, apart from the press on `nav-icon`.
                                    <span class="nav-pop">
                                        <Ico icon=icon />
                                    </span>
                                </span>
                                <span class="nav-label">{label}</span>
                            </a>
                        }
                    })
                    .collect_view()
            }}
        </nav>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_thumb_leads_with_the_edge_it_moves_toward() {
        assert_eq!(way(0, 3, "back"), "next");
        assert_eq!(way(3, 1, "next"), "back");
        assert_eq!(way(2, 2, "back"), "back");
        assert_eq!(way(2, 2, "next"), "next");
    }

    #[test]
    fn the_items_are_the_tabs_in_order() {
        let tabs: Vec<Tab> = items().into_iter().map(|(tab, ..)| tab).collect();
        assert_eq!(tabs, Tab::ALL);
    }
}
