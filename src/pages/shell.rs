//! The frame every page lives in: floating capsules for the brand and the
//! search, and a slider for the navigation that sits at the top right of a
//! wide window, stands up as a rail at the side of a narrower one and floats
//! at the bottom of a phone (`style/shell.css` has the breakpoints). A guest
//! on a server that takes them gets the same frame, with what a guest can
//! use and a way to sign in in the navigation.

use std::cmp::Ordering;

use leptos::ev::{KeyboardEvent, SubmitEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::components::Outlet;
use leptos_router::hooks::{use_location, use_navigate, use_query_map};
use send_wrapper::SendWrapper;
use starbooru_core::TagCategory;

use super::explore;
use super::login::Login;
use super::saved::{self, SavedLine, SavedSheet};
use super::search_demo::{SearchDemo, plays_on};
use crate::activity::{self, Activity};
use crate::components::banner::Spinner;
use crate::components::brand::{Mark, Wordmark};
use crate::components::icons::{Ico, Icon};
use crate::dom::{self, caret_of};
use crate::logic::boorus;
use crate::logic::explore::{tidy, window};
use crate::logic::home::{Start, start};
use crate::logic::modes::{Screen, screen};
use crate::logic::saved::Kind;
use crate::logic::search::{
    Hint, Latest, ListKey, REMOTE_PAUSE_MS, TYPING_PAUSE_MS, category_class, complete, list_key,
    short_count, still_fitting, utf16_index, word_at,
};
use crate::pages::account::AccountButton;
use crate::route::{self, ForumPage, Place, Tab};
use crate::updates::UpdateNotice;
use crate::{api, instance, store};

/// The pages under the navigation for someone signed in, or for a guest on
/// a server that takes guests; otherwise the sign-in screen.
#[component]
pub fn Shell() -> impl IntoView {
    // Without a session, whether this server takes guests. Signing in ends a
    // guest's wish to.
    Effect::new(move |_| {
        if store::signed_in() {
            instance::stop_asking();
        } else {
            untrack(instance::ask_guests);
        }
    });
    // Back, or another page, takes a guest away from the sign-in screen.
    let location = use_location();
    Effect::watch(
        move || (location.pathname.get(), location.search.get()),
        |_, _, _| instance::stop_asking(),
        false,
    );
    let shown = Memo::new(|_| {
        screen(
            store::signed_in(),
            instance::guests(),
            instance::asked().is_some(),
        )
    });
    move || match shown.get() {
        Screen::Waiting => view! {
            <div class="center-page shell-waiting">
                <Spinner />
            </div>
        }
        .into_any(),
        Screen::Member => view! { <Frame guest=false /> }.into_any(),
        Screen::Guest => view! { <Frame guest=true /> }.into_any(),
        Screen::SignIn => view! { <Login /> }.into_any(),
    }
}

#[component]
fn Frame(guest: bool) -> impl IntoView {
    // Whether the session kept from last time still works. A token signed out
    // elsewhere or past its 90 days signs out here, with a note on the
    // sign-in screen, instead of failing on every page.
    if !guest {
        let owner = store::owner();
        spawn_local(async move {
            if let Ok(user) = api::call(|c| async move { c.me().await }).await
                && store::owner() == owner
            {
                store::update_user(user);
            }
        });
    }
    // What the server is now: its mode, what a vote is, its home page.
    instance::refresh();
    let location = use_location();
    let path = location.pathname;
    // A visit that starts at the bare address opens on the home page, when
    // the server has one; otherwise the app carries on from the last
    // search. Until the server says, the page waits, so the gallery doesn't
    // flash by on the way.
    let navigate = use_navigate();
    let query = use_query_map();
    let fresh = !dom::visited();
    let decide = move || {
        let has_q = query.with_untracked(|q| q.get("q").is_some());
        path.with_untracked(|p| {
            start(
                p,
                has_q,
                fresh,
                instance::home(),
                &store::last_search_untracked(),
            )
        })
    };
    let holding = RwSignal::new(untrack(decide) == Start::Wait);
    let decided = StoredValue::new(false);
    Effect::new(move |_| {
        // Follows the server's answer while waiting for it.
        let _ = instance::home();
        if decided.get_value() {
            return;
        }
        let options = NavigateOptions {
            replace: true,
            ..NavigateOptions::default()
        };
        match untrack(decide) {
            Start::Wait => return,
            Start::Stay => {}
            Start::Home => navigate(route::HOME, options),
            Start::Search(last) => navigate(&route::gallery(&last), options),
        }
        decided.set_value(true);
        dom::mark_visited();
        holding.set(false);
    });
    // The home page has the search box and the mark of its own.
    let on_home = Memo::new(move |_| path.with(|p| p == route::HOME));
    let home_on = Memo::new(|_| instance::home() == Some(true));
    let brand_href = move || if home_on.get() { route::HOME } else { "/" };
    // A public server has the forum in Upload's place and the account
    // button beside the navigation; a private one stays as it was.
    let public = Memo::new(|_| instance::public());
    let me = Memo::new(|_| store::session().map(|s| s.user.username));
    let place = Memo::new(move |_| {
        path.with(|p| me.with(|me| route::place_of(p, public.get(), me.as_deref())))
    });
    let here = Memo::new(move |_| match place.get() {
        Some(Place::Tab(tab)) => Some(tab),
        _ => None,
    });
    let mine = Signal::derive(move || place.get() == Some(Place::Account));
    view! {
        <header class="topbar">
            <Show when=move || !on_home.get()>
                <a class="brand glass float" href=brand_href aria-label=move || t!("app-name")>
                    <Mark height=30 compact=true />
                    <Wordmark height=19 />
                </a>
                <SearchBox />
            </Show>
            {move || {
                if public.get() {
                    view! {
                        <div class="top-end">
                            <NavSlider place="nav-top" here=here guest=guest public=true />
                            <AccountButton place="account-top" here=mine guest=guest />
                        </div>
                    }
                        .into_any()
                } else {
                    view! { <NavSlider place="nav-top" here=here guest=guest public=false /> }
                        .into_any()
                }
            }}
        </header>
        <main class="content">
            <UpdateNotice />
            <Show when=move || !holding.get()>
                <Outlet />
            </Show>
        </main>
        {move || {
            if public.get() {
                view! {
                    <div class="dock">
                        <NavSlider place="nav-bottom" here=here guest=guest public=true />
                        <AccountButton place="account-bottom" here=mine guest=guest />
                    </div>
                }
                    .into_any()
            } else {
                view! { <NavSlider place="nav-bottom" here=here guest=guest public=false /> }
                    .into_any()
            }
        }}
        {(!guest).then(|| view! { <Activity /> })}
        {(!guest).then(|| view! { <SavedSheet /> })}
    }
}

/// What the search box searches: the collection, Explore while it's open,
/// or on the booru browser's pages the sites its address names (`s`), with
/// their own tags to suggest.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Scope {
    Collection,
    /// With the window of a public server's Trending, when the page has one.
    Explore(Option<u32>),
    Boorus(String),
    /// The Tags page, which the box filters as it's typed in.
    Tags,
    /// The people page, which the box filters by name as it's typed in,
    /// completing the collection's characters.
    People,
    /// The forum's threads, in one category or in all of them.
    Forum(Option<i64>),
}

impl Scope {
    /// Whose saved searches the empty box offers, if any.
    fn saved(&self) -> Option<Kind> {
        match self {
            Scope::Collection | Scope::Explore(_) => Some(Kind::Gallery),
            Scope::Boorus(_) => Some(Kind::Boorus),
            Scope::Tags | Scope::People | Scope::Forum(_) => None,
        }
    }
}

/// What the box searches at `path`, whose address has the booru browser's
/// sites in `s` and Explore's window in `days`.
fn scope_of(path: &str, s: Option<String>, days: Option<String>) -> Scope {
    if path == route::BOORUS || path.starts_with("/boorus/") {
        Scope::Boorus(s.unwrap_or_default())
    } else if let Some(page) = route::forum_page(path) {
        Scope::Forum(match page {
            ForumPage::Category(id) => Some(id),
            ForumPage::Index | ForumPage::Thread(_) => None,
        })
    } else if route::is_explore(path) {
        Scope::Explore(days.map(|d| window(Some(&d))))
    } else if route::is_tags(path) {
        Scope::Tags
    } else if route::is_people(path) {
        Scope::People
    } else {
        Scope::Collection
    }
}

/// Where a search typed in the box goes. On the boorus it goes to the sites
/// the page searched, or else to those chosen last; on Explore it keeps the
/// window.
fn search_href(scope: &Scope, q: &str, last_sites: &[String]) -> String {
    match scope {
        Scope::Collection => route::gallery(q),
        Scope::Explore(None) => route::explore(&tidy(q)),
        Scope::Explore(Some(days)) => route::explore_over(&tidy(q), *days),
        Scope::Tags => route::tags(q),
        Scope::People => route::people(q),
        Scope::Boorus(s) => {
            let sites = boorus::sites_param(&boorus::chosen_sites(s, last_sites));
            route::boorus(&boorus::tidy_query(q), &sites)
        }
        Scope::Forum(None) => route::forum(q),
        Scope::Forum(Some(id)) => route::forum_category(*id, q),
    }
}

/// The search a page shows in the box: the gallery's, a post's and
/// Explore's own, the booru browser's as its address carries it, the Tags
/// page's filter, none on the home page, and the last search elsewhere.
fn shown_search(path: &str, q: Option<String>, last: &str) -> String {
    if path == route::HOME {
        return String::new();
    }
    match scope_of(path, None, None) {
        Scope::Boorus(_) => boorus::from_address(&q.unwrap_or_default()),
        Scope::Explore(_) | Scope::Tags | Scope::People | Scope::Forum(_) => q.unwrap_or_default(),
        Scope::Collection if route::has_search(path) => q.unwrap_or_default(),
        Scope::Collection => last.to_owned(),
    }
}

/// What the empty box says it searches.
fn placeholder(scope: &Scope) -> String {
    match scope {
        Scope::Collection => t!("search-placeholder"),
        Scope::Explore(_) => t!("explore-search"),
        Scope::Boorus(_) => t!("boorus-search"),
        Scope::Tags => t!("tags-filter-placeholder"),
        Scope::People => t!("people-filter-placeholder"),
        Scope::Forum(None) => t!("forum-search"),
        Scope::Forum(Some(_)) => t!("forum-search-category"),
    }
}

/// The box's name for screen readers.
fn label(scope: &Scope) -> String {
    match scope {
        Scope::Collection => t!("search-label"),
        Scope::Explore(_) => t!("explore-search"),
        Scope::Boorus(_) => t!("boorus-search"),
        Scope::Tags => t!("tags-filter-label"),
        Scope::People => t!("people-filter-label"),
        Scope::Forum(None) => t!("forum-search"),
        Scope::Forum(Some(_)) => t!("forum-search-category"),
    }
}

/// The characters among tags offered, for the people page.
fn characters(hints: Vec<Hint>) -> Vec<Hint> {
    let character = category_class(TagCategory::Character);
    hints.into_iter().filter(|h| h.class == character).collect()
}

/// Suggestions kept from a moment ago for `typed` in `scope`.
fn kept(scope: &Scope, typed: &str) -> Option<Vec<Hint>> {
    match scope {
        Scope::Collection | Scope::Explore(_) => api::kept_tags(typed),
        Scope::People => api::kept_tags(typed).map(characters),
        // The list below is the answer.
        Scope::Tags => Some(Vec::new()),
        Scope::Boorus(s) => {
            let site = boorus::completing_site(s, &store::prefs_untracked().booru_sites)?;
            api::kept_remote_tags(&site, typed)
        }
        // Words, not tags.
        Scope::Forum(_) => Some(Vec::new()),
    }
}

/// How long the typing pauses before `scope` is asked.
fn pause(scope: &Scope) -> i32 {
    match scope {
        Scope::Collection | Scope::Explore(_) | Scope::Tags | Scope::People | Scope::Forum(_) => {
            TYPING_PAUSE_MS
        }
        Scope::Boorus(_) => REMOTE_PAUSE_MS,
    }
}

/// Suggestions for `typed` in `scope`. Explore completes the collection's
/// tags; on the boorus, the first site chosen completes them.
async fn suggest(scope: Scope, typed: String) -> Vec<Hint> {
    match scope {
        Scope::Collection | Scope::Explore(_) => api::tag_hints(typed).await,
        Scope::People => characters(api::tag_hints(typed).await),
        Scope::Tags => Vec::new(),
        Scope::Boorus(s) => {
            match boorus::completing_site(&s, &store::prefs_untracked().booru_sites) {
                Some(site) => api::remote_tag_hints(site, typed).await,
                None => Vec::new(),
            }
        }
        Scope::Forum(_) => Vec::new(),
    }
}

const LIST_ID: &str = "search-suggestions";

fn option_id(i: usize) -> String {
    format!("suggestion-{i}")
}

/// Keeps suggestion `i` in sight when the list scrolls.
fn reveal(i: usize) {
    request_animation_frame(move || {
        if let Some(li) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(&option_id(i)))
        {
            let options = web_sys::ScrollIntoViewOptions::new();
            options.set_block(web_sys::ScrollLogicalPosition::Nearest);
            li.scroll_into_view_with_scroll_into_view_options(&options);
        }
    });
}

/// The search: Enter shows the gallery for what was typed, and an empty box
/// all of it; on Explore and the booru browser, that page instead. It shows
/// the search of the page it's on, and the gallery's last one elsewhere. The
/// tag under the caret is completed from the collection's tags, or the
/// boorus' (the ARIA combobox pattern). The top bar has it, and the home page
/// in the top bar's place (`hero`).
#[component]
pub(crate) fn SearchBox(#[prop(optional)] hero: bool) -> impl IntoView {
    let query = use_query_map();
    let location = use_location();
    let path = location.pathname;
    let search = location.search;
    let last = Memo::new(|_| store::last_search());
    let current = Memo::new(move |_| {
        let q = query.with(|q| q.get("q"));
        path.with(|p| last.with(|last| shown_search(p, q, last)))
    });
    let scope = Memo::new(move |_| {
        let (s, days) = query.with(|q| (q.get("s"), q.get("days")));
        match path.with(|p| scope_of(p, s, days)) {
            // A public server has no boorus, and a private one no forum:
            // the box searches the collection.
            Scope::Boorus(_) if instance::public() => Scope::Collection,
            Scope::Forum(_) if !instance::public() => Scope::Collection,
            scope => scope,
        }
    });
    let text = RwSignal::new(current.get_untracked());
    let focused = RwSignal::new(false);
    // Typing opens the list; Escape, a choice and leaving close it.
    let open = RwSignal::new(false);
    let hints = RwSignal::new(Vec::<Hint>::new());
    let active = RwSignal::new(None::<usize>);
    // The demo is showing (`data-typing`).
    let typing = RwSignal::new(false);
    // The word the hints are for, and the latest question asked about one.
    let asked = StoredValue::new(None::<String>);
    let turn = StoredValue::new(Latest::default());
    let input: NodeRef<html::Input> = NodeRef::new();
    // While the box is empty, it offers the saved searches instead, the
    // gallery's or the boorus', until Escape or a choice puts them away.
    // The last option opens all of them.
    let saved_open = RwSignal::new(false);
    let saved_kind = Memo::new(move |_| scope.with(Scope::saved));
    let saved = Memo::new(move |_| {
        let wanted = focused.get() && saved_open.get() && text.with(|t| t.trim().is_empty());
        match saved_kind.get() {
            Some(kind) if wanted => saved::offered(kind),
            _ => (Vec::new(), 0),
        }
    });
    let offering = Memo::new(move |_| saved.with(|(s, _)| !s.is_empty()));
    // On the Tags page and the people page the box filters the list as
    // it's typed in. The Tags page's list is the answer: nothing to complete.
    let filtering = Memo::new(move |_| scope.with(|s| matches!(s, Scope::Tags | Scope::People)));
    let listing = Memo::new(move |_| scope.with(|s| *s == Scope::Tags));
    let filter_turn = StoredValue::new(Latest::default());
    let expanded = Memo::new(move |_| {
        focused.get() && (offering.get() || open.get() && hints.with(|h| !h.is_empty()))
    });

    let close = move || {
        open.set(false);
        saved_open.set(false);
        active.set(None);
    };
    // Drops the hints and any answer still on its way.
    let forget = move |now: Option<String>| {
        turn.update_value(|t| {
            t.next();
        });
        asked.set_value(now);
        hints.set(Vec::new());
        close();
    };
    // A new page, or Back, brings its own search, and closes the list. Not
    // the filter the Tags page was just given from here, which would undo
    // what was typed since.
    Effect::watch(
        move || current.get(),
        move |q, _, _| {
            if !(filtering.get_untracked() && focused.get_untracked()) {
                text.set(q.clone());
            }
        },
        false,
    );
    Effect::watch(
        move || (path.get(), search.get()),
        move |_, _, _| forget(None),
        false,
    );

    // Asks for the tags starting with `typed` once the typing pauses. A
    // newer word makes the answer for this one stale.
    let look_up = move |typed: Option<String>| {
        let Some(n) = turn.try_update_value(Latest::next) else {
            return;
        };
        let Some(typed) = typed else {
            hints.set(Vec::new());
            return;
        };
        let scope = scope.get_untracked();
        if let Some(known) = kept(&scope, &typed) {
            hints.set(known);
            return;
        }
        hints.update(|h| still_fitting(h, &typed));
        let latest = move || turn.try_with_value(|t| t.is(n)).unwrap_or(false);
        spawn_local(async move {
            dom::sleep(pause(&scope)).await;
            if !latest() {
                return;
            }
            let found = suggest(scope, typed).await;
            if latest() {
                let _ = hints.try_set(found);
            }
        });
    };

    // Looks up the word at the caret, when it isn't the one looked up last.
    let follow = move || {
        let Some(el) = input.get_untracked() else {
            return;
        };
        let value = el.value();
        let typed = word_at(&value, caret_of(&el, &value)).map(|w| w.typed.to_lowercase());
        if asked.with_value(|a| *a == typed) {
            return;
        }
        asked.set_value(typed.clone());
        active.set(None);
        open.set(typed.is_some());
        look_up(typed);
    };

    // The page's filter, once the typing pauses: in place, so Back leaves
    // the page rather than going through every letter.
    let go_filter = SendWrapper::new(use_navigate());
    let set_filter = Callback::new(move |value: String| {
        let to = match scope.get_untracked() {
            Scope::People => route::people(&value),
            _ => route::tags(&value),
        };
        let replace = NavigateOptions {
            replace: true,
            scroll: false,
            ..NavigateOptions::default()
        };
        (*go_filter)(&to, replace);
    });
    // Puts suggestion `i` in place of the word at the caret.
    let accept = move |i: usize| {
        let Some(name) = hints.with_untracked(|h| h.get(i).map(|h| h.name.clone())) else {
            return;
        };
        let Some(el) = input.get_untracked() else {
            return;
        };
        let value = el.value();
        let Some((next, at)) =
            word_at(&value, caret_of(&el, &value)).and_then(|w| complete(&value, &w, &name))
        else {
            return;
        };
        // Straight into the box, so the caret can go after the tag at once.
        el.set_value(&next);
        let caret = u32::try_from(utf16_index(&next, at)).unwrap_or(u32::MAX);
        let _ = el.set_selection_range(caret, caret);
        let now = word_at(&next, at).map(|w| w.typed.to_lowercase());
        text.set(next.clone());
        forget(now);
        // The people page shows whom was picked at once.
        if filtering.get_untracked() {
            filter_turn.update_value(|t| {
                t.next();
            });
            set_filter.run(next);
        }
    };

    // A saved search opens where it was left; the last option opens them all.
    let go = SendWrapper::new(use_navigate());
    let choose = Callback::new(move |i: usize| {
        let Some(kind) = saved_kind.get_untracked() else {
            return;
        };
        let item = saved.with_untracked(|(s, _)| s.get(i).map(|(s, _)| s.clone()));
        close();
        if let Some(input) = input.get_untracked() {
            let _ = input.blur();
        }
        match item {
            Some(item) => {
                saved::used(&item);
                (*go)(&saved::href(&item), NavigateOptions::default());
            }
            None => saved::open_sheet(kind),
        }
    });
    let filter_later = move |value: String| {
        let Some(n) = filter_turn.try_update_value(Latest::next) else {
            return;
        };
        spawn_local(async move {
            dom::sleep(TYPING_PAUSE_MS).await;
            let latest = filter_turn.try_with_value(|t| t.is(n)) == Some(true);
            if latest && filtering.try_get_untracked() == Some(true) {
                set_filter.run(value);
            }
        });
    };
    let pick = move |i: usize| {
        if offering.get_untracked() {
            choose.run(i);
        } else {
            accept(i);
        }
    };

    let on_key = move |e: KeyboardEvent| {
        if e.is_composing() {
            return;
        }
        let count = if offering.get_untracked() {
            // And "All saved searches".
            saved.with_untracked(|(s, _)| s.len() + 1)
        } else {
            hints.with_untracked(Vec::len)
        };
        match list_key(
            &e.key(),
            expanded.get_untracked(),
            active.get_untracked(),
            count,
        ) {
            ListKey::Move(i) => {
                e.prevent_default();
                active.set(Some(i));
                reveal(i);
            }
            ListKey::Accept(i) => {
                e.prevent_default();
                pick(i);
            }
            ListKey::Close => {
                // Not the page's Escape, nor the box's own, which empties it.
                e.prevent_default();
                e.stop_propagation();
                close();
            }
            ListKey::Pass => {}
        }
    };

    let navigate = use_navigate();
    let submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        close();
        let last_sites = store::prefs_untracked().booru_sites;
        // On the Tags page, Enter keeps the filter as it is.
        let options = NavigateOptions {
            replace: filtering.get_untracked(),
            scroll: !filtering.get_untracked(),
            ..NavigateOptions::default()
        };
        filter_turn.update_value(|t| {
            t.next();
        });
        navigate(
            &search_href(&scope.get_untracked(), &text.get_untracked(), &last_sites),
            options,
        );
        // On a phone, this puts the keyboard away.
        if let Some(input) = input.get_untracked() {
            let _ = input.blur();
        }
    };
    let clear = move |_| {
        text.set(String::new());
        saved_open.set(true);
        if let Some(input) = input.get_untracked() {
            input.set_value("");
            let _ = input.focus();
        }
        if filtering.get_untracked() {
            filter_turn.update_value(|t| {
                t.next();
            });
            set_filter.run(String::new());
        } else {
            follow();
        }
    };
    // The pointer picks an option only by moving: one resting where the
    // list opens doesn't choose for Enter.
    let hover = move |i: usize| {
        if active.get_untracked() != Some(i) {
            active.set(Some(i));
        }
    };
    let saved_list = move || {
        let (list, all) = saved.get();
        let last = list.len();
        let options = list
            .into_iter()
            .enumerate()
            .map(|(i, (item, label))| {
                let on = move || active.get() == Some(i);
                let name = item.name.clone();
                view! {
                    <li
                        id=option_id(i)
                        role="option"
                        class="saved-option"
                        class:active=on
                        aria-selected=move || if on() { "true" } else { "false" }
                        aria-label=name
                        on:mousemove=move |_| hover(i)
                        on:click=move |_| choose.run(i)
                    >
                        <SavedLine item=item label=label />
                    </li>
                }
            })
            .collect_view();
        let on = move || active.get() == Some(last);
        view! {
            {options}
            <li
                id=option_id(last)
                role="option"
                class="saved-option saved-every"
                class:active=on
                aria-selected=move || if on() { "true" } else { "false" }
                on:mousemove=move |_| hover(last)
                on:click=move |_| choose.run(last)
            >
                <span class="preview every-tile" aria-hidden="true">
                    <Ico icon=Icon::Layout size=20 />
                </span>
                <span class="saved-text">
                    <span class="saved-name">{move || t!("saved-all")}</span>
                </span>
                <span class="badge">{all}</span>
            </li>
        }
    };
    let list = move || {
        hints
            .get()
            .into_iter()
            .enumerate()
            .map(|(i, hint)| {
                let Hint { name, class, count } = hint;
                let label = count.map(|n| t!("search-hint", tag = name.clone(), count = n));
                let on = move || active.get() == Some(i);
                view! {
                    <li
                        id=option_id(i)
                        role="option"
                        class:active=on
                        aria-selected=move || if on() { "true" } else { "false" }
                        aria-label=label
                        on:mousemove=move |_| hover(i)
                        on:click=move |_| accept(i)
                    >
                        <span class=format!("tag-name {class}")>{name}</span>
                        {count.map(|n| view! { <span class="count">{short_count(n)}</span> })}
                    </li>
                }
            })
            .collect_view()
    };
    view! {
        <form
            class="search glass float"
            class:hero=hero
            role="search"
            data-typing=move || typing.get().then_some("")
            on:submit=submit
        >
            // Phones: the crown takes the search icon's place, and the way home with it.
            <a
                class="search-home"
                href=move || if instance::home() == Some(true) { route::HOME } else { "/" }
                aria-label=move || t!("app-name")
            >
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
                    placeholder=move || scope.with(placeholder)
                    aria-label=move || scope.with(label)
                    role="combobox"
                    aria-autocomplete="list"
                    aria-controls=move || expanded.get().then_some(LIST_ID)
                    aria-expanded=move || if expanded.get() { "true" } else { "false" }
                    aria-activedescendant=move || {
                        active.get().filter(|_| expanded.get()).map(option_id)
                    }
                    prop:value=move || text.get()
                    on:input=move |e| {
                        let value = event_target_value(&e);
                        // Emptied: the saved searches again.
                        saved_open.set(value.trim().is_empty());
                        text.set(value.clone());
                        if filtering.get_untracked() {
                            filter_later(value);
                        }
                        if listing.get_untracked() {
                            return;
                        }
                        open.set(true);
                        follow();
                    }
                    on:keydown=on_key
                    // The caret moved to another word.
                    on:keyup=move |e: KeyboardEvent| {
                        if matches!(e.key().as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End")
                            && !listing.get_untracked()
                        {
                            follow();
                        }
                    }
                    on:click=move |_| {
                        if !listing.get_untracked() {
                            follow();
                        }
                    }
                    on:focus=move |_| {
                        focused.set(true);
                        saved_open.set(true);
                        // Coming back to the same word opens its list again.
                        asked.set_value(None);
                        if !listing.get_untracked() {
                            follow();
                        }
                    }
                    on:blur=move |_| {
                        focused.set(false);
                        close();
                    }
                />
                <Show when=move || text.with(String::is_empty) && path.with(|p| plays_on(p))>
                    <SearchDemo typing=typing focused=focused />
                </Show>
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
            <Show when=move || expanded.get()>
                <ul
                    class="suggestions"
                    class:saved-offers=move || offering.get()
                    id=LIST_ID
                    role="listbox"
                    aria-label=move || {
                        if offering.get() { saved_heading(saved_kind.get()) } else { t!("search-suggestions") }
                    }
                    // A press would take the focus from the box and close the list first.
                    on:mousedown=|e| e.prevent_default()
                >
                    {move || {
                        if offering.get() {
                            view! {
                                <li class="list-head" role="presentation" aria-hidden="true">
                                    {move || saved_heading(saved_kind.get())}
                                </li>
                                {saved_list}
                            }
                                .into_any()
                        } else {
                            list.into_any()
                        }
                    }}
                </ul>
            </Show>
        </form>
    }
}

/// The heading of the saved searches the empty box offers.
fn saved_heading(kind: Option<Kind>) -> String {
    match kind {
        Some(Kind::Boorus) => t!("saved-booru-searches"),
        _ => t!("saved-searches"),
    }
}

/// A tab's icon and name in the navigation.
fn item(tab: Tab) -> (Icon, String) {
    match tab {
        Tab::Gallery => (Icon::Gallery, t!("nav-gallery")),
        Tab::Explore => (Icon::Explore, t!("nav-explore")),
        Tab::Tags => (Icon::Label, t!("nav-tags")),
        Tab::Forum => (Icon::Forum, t!("nav-forum")),
        Tab::Upload => (Icon::Upload, t!("nav-upload")),
        Tab::Settings => (Icon::Tune, t!("nav-settings-tab")),
    }
}

/// The navigation's items in order, with their icons and names: a public
/// server's or a private one's, a guest's or an account's.
fn items(public: bool, guest: bool) -> Vec<(Tab, Icon, String)> {
    Tab::shown(public, guest)
        .iter()
        .map(|&tab| {
            let (icon, label) = item(tab);
            (tab, icon, label)
        })
        .collect()
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
/// `nav-top` or `nav-bottom`, which also makes the rail. On a private server
/// a guest's ends with Sign in, where an account's Settings holds the
/// account; a public server has the account button beside it instead.
#[component]
fn NavSlider(
    place: &'static str,
    here: Memo<Option<Tab>>,
    guest: bool,
    public: bool,
) -> impl IntoView {
    let tabs = Tab::shown(public, guest);
    let sign_in = guest && !public;
    let index = move || {
        here.get()
            .and_then(|tab| tabs.iter().position(|t| *t == tab))
    };
    // Where the thumb stands and the way it last went. With no item selected
    // it keeps its place, out of sight.
    let thumb = Memo::new(move |last: Option<&(usize, &'static str)>| {
        let (was, last_way) = last.copied().unwrap_or((index().unwrap_or(0), "next"));
        let at = index().unwrap_or(was);
        (at, way(was, at, last_way))
    });
    let cells = tabs.len() + usize::from(sign_in);
    // The gallery goes back to the search it showed last, and Explore to
    // where it was left.
    let last = Memo::new(|_| store::last_search());
    let last_explore = Memo::new(|_| explore::last_route());
    // Uploads and imports still going, on the Upload item.
    let pending = Memo::new(|_| activity::pending());
    view! {
        <nav
            class=format!("{place} slider glass float")
            aria-label=move || t!("nav-main")
            style=move || format!("--n: {cells}; --i: {}", thumb.get().0)
            data-dir=move || thumb.get().1
        >
            // Off too for a page whose item this navigation hasn't.
            <span class="slider-thumb" class:off=move || index().is_none()>
                <span class="slider-clip">
                    <span class="slider-unclip">
                        <span class="slider-fill"></span>
                    </span>
                </span>
            </span>
            {move || {
                items(public, guest)
                    .into_iter()
                    .map(|(tab, icon, label)| {
                        let active = move || here.get() == Some(tab);
                        let upload = tab == Tab::Upload;
                        let badge = move || pending.get().min(99);
                        let note = move || {
                            (upload && pending.get() != 0)
                                .then(|| t!("activity-pending", count = pending.get()))
                        };
                        view! {
                            <a
                                class="nav-item"
                                // The sheet of files dropped on the gallery comes out of this item.
                                class:nav-upload=upload
                                class:active=active
                                href=move || match tab {
                                    Tab::Gallery => route::gallery(&last.get()),
                                    Tab::Explore => last_explore.get(),
                                    _ => tab.href().to_owned(),
                                }
                                title=note
                                aria-current=move || if active() { "page" } else { "false" }
                            >
                                <span class="nav-icon">
                                    // Its own box for the pop, apart from the press on `nav-icon`.
                                    <span class="nav-pop">
                                        <Ico icon=icon />
                                    </span>
                                    <Show when=move || upload && pending.get() != 0>
                                        <span class="nav-badge" aria-hidden="true">
                                            {badge}
                                        </span>
                                    </Show>
                                </span>
                                <span class="nav-label">{label}</span>
                                <Show when=move || upload && pending.get() != 0>
                                    <span class="visually-hidden">{note}</span>
                                </Show>
                            </a>
                        }
                    })
                    .collect_view()
            }}
            {sign_in.then(|| view! { <SignInItem /> })}
        </nav>
    }
}

/// A guest's way to an account, last in the navigation: it opens the
/// sign-in screen over the page, which comes back here signed in.
#[component]
fn SignInItem() -> impl IntoView {
    view! {
        <button
            type="button"
            class="nav-item nav-sign-in"
            on:click=move |_| instance::ask_sign_in(None)
        >
            <span class="nav-icon">
                <span class="nav-pop">
                    <Ico icon=Icon::Login />
                </span>
            </span>
            <span class="nav-label">{move || t!("guest-sign-in")}</span>
        </button>
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
    fn the_box_searches_the_collection() {
        for path in ["/", "/post/3", "/tag", "/help", "/booruss", "/explorer"] {
            assert_eq!(scope_of(path, Some("e926".into()), None), Scope::Collection);
        }
        assert_eq!(
            search_href(&Scope::Collection, " fractal -abstract ", &[]),
            "/?q=fractal+-abstract"
        );
        assert_eq!(search_href(&Scope::Collection, "", &[]), "/");
        assert_eq!(shown_search("/", Some("sky".into()), "sea"), "sky");
        assert_eq!(shown_search("/help", Some("sky".into()), "sea"), "sea");
        assert_eq!(pause(&Scope::Collection), TYPING_PAUSE_MS);
        assert_eq!(Scope::Collection.saved(), Some(Kind::Gallery));
    }

    #[test]
    fn on_explore_the_box_narrows_explore() {
        for path in ["/explore", "/explore/"] {
            assert_eq!(
                scope_of(path, Some("e926".into()), None),
                Scope::Explore(None)
            );
        }
        // Its boorus side searches the boorus.
        assert!(matches!(scope_of("/boorus", None, None), Scope::Boorus(_)));
        assert_eq!(
            search_href(
                &Scope::Explore(None),
                "  fractal   -abstract ",
                &["e926".into()]
            ),
            "/explore?q=fractal+-abstract"
        );
        // An empty box shows all of Explore.
        assert_eq!(search_href(&Scope::Explore(None), "  ", &[]), "/explore");
        // A public server's Trending keeps the window it was over.
        let month = scope_of("/explore", None, Some("30".into()));
        assert_eq!(month, Scope::Explore(Some(30)));
        assert_eq!(search_href(&month, "cat", &[]), "/explore?q=cat&days=30");
        assert_eq!(
            search_href(&scope_of("/explore", None, Some("7".into())), "", &[]),
            "/explore"
        );
        // The box shows Explore's own search, not the gallery's last one.
        assert_eq!(
            shown_search("/explore", Some("fractal".into()), "sea"),
            "fractal"
        );
        assert_eq!(shown_search("/explore", None, "sea"), "");
        // Leaving brings back the gallery's search.
        assert_eq!(shown_search("/", Some("sky".into()), "sea"), "sky");
        assert_eq!(shown_search("/help", None, "sea"), "sea");
        // The gallery's saved searches, as on the gallery.
        assert_eq!(Scope::Explore(None).saved(), Some(Kind::Gallery));
        // The collection's tags complete it, as fast as on the gallery.
        assert_eq!(pause(&Scope::Explore(None)), pause(&Scope::Collection));
    }

    #[test]
    fn on_the_tags_page_the_box_filters_the_tags() {
        for path in ["/tags", "/tags/"] {
            assert_eq!(scope_of(path, Some("e926".into()), None), Scope::Tags);
        }
        assert_eq!(search_href(&Scope::Tags, "  fra ", &[]), "/tags?q=fra");
        // Cleared, it shows them all.
        assert_eq!(search_href(&Scope::Tags, "  ", &[]), "/tags");
        // The box shows the page's filter, not the gallery's last search.
        assert_eq!(shown_search("/tags", Some("fra".into()), "sea"), "fra");
        assert_eq!(shown_search("/tags", None, "sea"), "");
        // The list is the answer: nothing to suggest, no saved searches.
        assert_eq!(kept(&Scope::Tags, "fra"), Some(Vec::new()));
        assert_eq!(Scope::Tags.saved(), None);
        assert_eq!(pause(&Scope::Tags), TYPING_PAUSE_MS);
        // A tag's own page is the collection's again.
        assert_eq!(scope_of("/tag", None, None), Scope::Collection);
    }

    #[test]
    fn on_the_people_page_the_box_filters_them_and_completes_characters() {
        for path in ["/people", "/people/"] {
            assert_eq!(scope_of(path, None, None), Scope::People);
        }
        assert_eq!(
            search_href(&Scope::People, " hatsune miku ", &[]),
            "/people?q=hatsune+miku"
        );
        assert_eq!(search_href(&Scope::People, "  ", &[]), "/people");
        assert_eq!(shown_search("/people", Some("miku".into()), "sea"), "miku");
        assert_eq!(shown_search("/people", None, "sea"), "");
        assert_eq!(Scope::People.saved(), None);
        let hint = |name: &str, class: &'static str| Hint {
            name: name.into(),
            class,
            count: None,
        };
        let offered = characters(vec![
            hint("miku", "cat-character"),
            hint("mikan", ""),
            hint("vocaloid", "cat-copyright"),
        ]);
        assert_eq!(offered, [hint("miku", "cat-character")]);
        // It filters as typed, as the Tags page does, but offers names too.
        let source = include_str!("shell.rs");
        assert!(source.contains("matches!(s, Scope::Tags | Scope::People)"));
        assert!(source.contains("Scope::People => route::people(&value),"));
    }

    #[test]
    fn the_empty_box_says_what_it_searches() {
        let scopes = [
            Scope::Collection,
            Scope::Explore(None),
            Scope::Boorus(String::new()),
            Scope::Tags,
            Scope::People,
            Scope::Forum(None),
            Scope::Forum(Some(1)),
        ];
        for (i, a) in scopes.iter().enumerate() {
            assert!(!placeholder(a).is_empty());
            assert!(!label(a).is_empty());
            for b in scopes.iter().skip(i + 1) {
                assert_ne!(placeholder(a), placeholder(b), "{a:?} {b:?}");
                assert_ne!(label(a), label(b), "{a:?} {b:?}");
            }
        }
    }

    #[test]
    fn on_the_boorus_the_box_searches_the_sites_chosen() {
        let scope = scope_of("/boorus", Some("gelbooru,e926".into()), None);
        assert_eq!(scope, Scope::Boorus("gelbooru,e926".into()));
        assert_eq!(
            scope_of("/boorus/post", None, None),
            Scope::Boorus(String::new())
        );
        assert_eq!(
            search_href(&scope, "  blue_sky   rating:g ", &[]),
            "/boorus?q=blue_sky+rating:g&s=gelbooru,e926"
        );
        // A page that names none searches the ones chosen last, or the first.
        let none = Scope::Boorus(String::new());
        assert_eq!(
            search_href(&none, "sky", &["yande.re".into()]),
            "/boorus?q=sky&s=yande.re"
        );
        assert_eq!(search_href(&none, "", &[]), "/boorus?s=danbooru");
        // What the address carries reads back as it was typed.
        assert_eq!(
            shown_search("/boorus", Some("panty_~26_stocking".into()), "sea"),
            "panty_&_stocking"
        );
        assert_eq!(shown_search("/boorus/post", None, "sea"), "");
        // Each site takes so many requests: the box waits a little longer.
        assert!(pause(&scope) > TYPING_PAUSE_MS);
        // The boorus' own saved searches.
        assert_eq!(scope.saved(), Some(Kind::Boorus));
    }

    #[test]
    fn the_box_is_a_combobox_over_its_list() {
        let source = include_str!("shell.rs");
        for attr in [
            r#"role="combobox""#,
            r#"aria-autocomplete="list""#,
            "aria-controls=LIST_ID",
            "aria-activedescendant=",
            r#"role="listbox""#,
            r#"role="option""#,
        ] {
            assert!(source.contains(attr), "{attr}");
        }
        // The list isn't glass: that's for the bars alone.
        let css = include_str!("../../style/shell.css");
        let (_, list) = css.split_once("\n.suggestions {").unwrap();
        let (list, _) = list.split_once('}').unwrap();
        assert!(list.contains("background: var(--surface)"));
        assert!(!css.contains("backdrop-filter"));
    }

    #[test]
    fn the_items_are_the_tabs_in_order() {
        let tabs = |public, guest| -> Vec<Tab> {
            items(public, guest)
                .into_iter()
                .map(|(tab, ..)| tab)
                .collect()
        };
        assert_eq!(tabs(false, false), Tab::ALL);
        assert_eq!(tabs(false, true), Tab::GUEST);
        assert_eq!(tabs(true, false), Tab::PUBLIC);
        assert_eq!(tabs(true, true), Tab::PUBLIC);
        // A private server's guest ends with the way to sign in; a public
        // server's has the account button for it. Nobody's uploads to show.
        let source = include_str!("shell.rs");
        assert!(source.contains("let sign_in = guest && !public;"));
        assert!(source.contains("{sign_in.then(|| view! { <SignInItem /> })}"));
        assert!(source.contains("{(!guest).then(|| view! { <Activity /> })}"));
        assert!(
            source.contains(r#"<AccountButton place="account-bottom" here=mine guest=guest />"#)
        );
    }

    #[test]
    fn on_the_forum_the_box_searches_its_threads() {
        assert_eq!(scope_of("/forum", None, None), Scope::Forum(None));
        assert_eq!(scope_of("/forum/t/4", None, None), Scope::Forum(None));
        assert_eq!(scope_of("/forum/c/2", None, None), Scope::Forum(Some(2)));
        assert_eq!(
            search_href(&Scope::Forum(None), " best sky ", &[]),
            "/forum?q=best+sky"
        );
        assert_eq!(
            search_href(&Scope::Forum(Some(2)), "sky", &[]),
            "/forum/c/2?q=sky"
        );
        assert_eq!(search_href(&Scope::Forum(Some(2)), " ", &[]), "/forum/c/2");
        assert_eq!(shown_search("/forum", Some("sky".into()), "sea"), "sky");
        assert_eq!(shown_search("/forum/t/4", None, "sea"), "");
        // Words, not tags: nothing to complete.
        assert_eq!(kept(&Scope::Forum(None), "sk"), Some(Vec::new()));
    }
}
