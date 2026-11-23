//! The frame every page lives in: floating capsules for the brand and the
//! search, and a slider for the navigation that sits at the top right of a
//! wide window, stands up as a rail at the side of a narrower one and floats
//! at the bottom of a phone (`style/shell.css` has the breakpoints).

use std::cmp::Ordering;

use leptos::ev::SubmitEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::components::Outlet;
use leptos_router::hooks::{use_location, use_navigate, use_query_map};

use super::login::Login;
use crate::api;
use crate::components::brand::{Mark, Wordmark};
use crate::components::icons::{Ico, Icon};
use crate::route::{self, Tab};
use crate::store;

/// The pages under the navigation, or the sign-in screen while no one is
/// signed in.
#[component]
pub fn Shell() -> impl IntoView {
    view! {
        <Show when=store::signed_in fallback=Login>
            <Frame />
        </Show>
    }
}

#[component]
fn Frame() -> impl IntoView {
    // Whether the session kept from last time still works. A token signed out
    // elsewhere or past its 90 days signs out here, with a note on the
    // sign-in screen, instead of failing on every page.
    spawn_local(async {
        if let Ok(user) = api::call(|c| async move { c.me().await }).await {
            store::update_user(user);
        }
    });
    let location = use_location();
    let path = location.pathname;
    let here = Memo::new(move |_| path.with(|p| route::tab_of(p)));
    // Opening the app carries on from the last search.
    let navigate = use_navigate();
    let query = use_query_map();
    Effect::new(move |_| {
        let last = store::last_search_untracked();
        let at_home = path.get_untracked() == "/" && query.with_untracked(|q| q.get("q").is_none());
        if at_home && !last.trim().is_empty() {
            let options = NavigateOptions {
                replace: true,
                ..NavigateOptions::default()
            };
            navigate(&route::gallery(&last), options);
        }
    });
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
/// all of it. It shows the search of the gallery or post it's on, and the last
/// one elsewhere.
#[component]
fn SearchBox() -> impl IntoView {
    let query = use_query_map();
    let path = use_location().pathname;
    let last = Memo::new(|_| store::last_search());
    let current = Memo::new(move |_| {
        if path.with(|p| route::has_search(p)) {
            query.with(|q| q.get("q").unwrap_or_default())
        } else {
            last.get()
        }
    });
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
    // The gallery goes back to the search it showed last.
    let last = Memo::new(|_| store::last_search());
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
                                href=move || {
                                    if tab == Tab::Gallery {
                                        route::gallery(&last.get())
                                    } else {
                                        tab.href().to_owned()
                                    }
                                }
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
