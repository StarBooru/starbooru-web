//! The frame every page lives in: floating capsules for the brand and the
//! search, and a slider for the navigation that sits at the top right of a
//! wide window, stands up as a rail at the side of a narrower one and floats
//! at the bottom of a phone (`style/shell.css` has the breakpoints).

use std::cell::RefCell;
use std::cmp::Ordering;

use chrono::Utc;
use leptos::ev::{KeyboardEvent, SubmitEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::components::Outlet;
use leptos_router::hooks::{use_location, use_navigate, use_query_map};

use super::login::Login;
use super::search_demo::{SearchDemo, plays_on};
use crate::api;
use crate::components::brand::{Mark, Wordmark};
use crate::components::icons::{Ico, Icon};
use crate::dom;
use crate::logic::search::{
    Answers, Hint, Latest, ListKey, SUGGESTIONS, TYPING_PAUSE_MS, byte_index, complete, list_key,
    short_count, still_fitting, utf16_index, word_at,
};
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

/// What the search box searches. The booru browser's pages will search the
/// sites chosen there, with their own tags to suggest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scope {
    Collection,
}

fn scope_of(_path: &str) -> Scope {
    Scope::Collection
}

/// Where a search typed in the box goes.
fn search_href(scope: Scope, q: &str) -> String {
    match scope {
        Scope::Collection => route::gallery(q),
    }
}

thread_local! {
    static KEPT: RefCell<Answers<Hint>> = RefCell::new(Answers::default());
}

/// The collection's tags starting with `typed`, kept from a moment ago.
fn kept_tags(typed: &str) -> Option<Vec<Hint>> {
    let owner = store::owner();
    KEPT.with_borrow(|k| k.get(&owner, typed, Utc::now()))
}

/// The collection's tags starting with `typed` (or with an alias that does),
/// most used first. A failed request offers none: the search still works
/// without them.
async fn local_tags(typed: String) -> Vec<Hint> {
    if let Some(known) = kept_tags(&typed) {
        return known;
    }
    let owner = store::owner();
    let asked = typed.clone();
    let Ok(tags) = api::call(move |c| async move { c.tags(&asked, SUGGESTIONS).await }).await
    else {
        return Vec::new();
    };
    let hints: Vec<Hint> = tags.into_iter().map(Hint::from).collect();
    KEPT.with_borrow_mut(|k| k.put(&owner, typed, hints.clone(), Utc::now()));
    hints
}

/// Suggestions for `typed` in `scope`.
async fn suggest(scope: Scope, typed: String) -> Vec<Hint> {
    match scope {
        Scope::Collection => local_tags(typed).await,
    }
}

const LIST_ID: &str = "search-suggestions";

fn option_id(i: usize) -> String {
    format!("suggestion-{i}")
}

/// Where the caret is in `el`, in bytes into `value`.
fn caret_of(el: &web_sys::HtmlInputElement, value: &str) -> usize {
    el.selection_start()
        .ok()
        .flatten()
        .and_then(|c| usize::try_from(c).ok())
        .map_or(value.len(), |c| byte_index(value, c))
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
/// all of it. It shows the search of the gallery or post it's on, and the last
/// one elsewhere. The tag under the caret is completed from the collection's
/// tags (the ARIA combobox pattern).
#[component]
fn SearchBox() -> impl IntoView {
    let query = use_query_map();
    let location = use_location();
    let path = location.pathname;
    let search = location.search;
    let last = Memo::new(|_| store::last_search());
    let current = Memo::new(move |_| {
        if path.with(|p| route::has_search(p)) {
            query.with(|q| q.get("q").unwrap_or_default())
        } else {
            last.get()
        }
    });
    let scope = Memo::new(move |_| path.with(|p| scope_of(p)));
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
    let expanded = Memo::new(move |_| focused.get() && open.get() && hints.with(|h| !h.is_empty()));

    let close = move || {
        open.set(false);
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
    // A new page, or Back, brings its own search, and closes the list.
    Effect::watch(
        move || current.get(),
        move |q, _, _| text.set(q.clone()),
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
        if let Some(known) = kept_tags(&typed) {
            hints.set(known);
            return;
        }
        hints.update(|h| still_fitting(h, &typed));
        let scope = scope.get_untracked();
        let latest = move || turn.try_with_value(|t| t.is(n)).unwrap_or(false);
        spawn_local(async move {
            dom::sleep(TYPING_PAUSE_MS).await;
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
        text.set(next);
        forget(now);
    };

    let on_key = move |e: KeyboardEvent| {
        if e.is_composing() {
            return;
        }
        let count = hints.with_untracked(Vec::len);
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
                accept(i);
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
        navigate(
            &search_href(scope.get_untracked(), &text.get_untracked()),
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
            input.set_value("");
            let _ = input.focus();
        }
        follow();
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
                        on:mouseenter=move |_| active.set(Some(i))
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
            role="search"
            data-typing=move || typing.get().then_some("")
            on:submit=submit
        >
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
                    role="combobox"
                    aria-autocomplete="list"
                    aria-controls=LIST_ID
                    aria-expanded=move || if expanded.get() { "true" } else { "false" }
                    aria-activedescendant=move || {
                        active.get().filter(|_| expanded.get()).map(option_id)
                    }
                    prop:value=move || text.get()
                    on:input=move |e| {
                        text.set(event_target_value(&e));
                        open.set(true);
                        follow();
                    }
                    on:keydown=on_key
                    // The caret moved to another word.
                    on:keyup=move |e: KeyboardEvent| {
                        if matches!(e.key().as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End") {
                            follow();
                        }
                    }
                    on:click=move |_| follow()
                    on:focus=move |_| {
                        focused.set(true);
                        // Coming back to the same word opens its list again.
                        asked.set_value(None);
                        follow();
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
                    id=LIST_ID
                    role="listbox"
                    aria-label=move || t!("search-suggestions")
                    // A press would take the focus from the box and close the list first.
                    on:mousedown=|e| e.prevent_default()
                >
                    {list}
                </ul>
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
    fn the_box_searches_the_collection() {
        for path in ["/", "/post/3", "/tags", "/help"] {
            assert_eq!(scope_of(path), Scope::Collection);
        }
        assert_eq!(
            search_href(Scope::Collection, " fractal -abstract "),
            "/?q=fractal+-abstract"
        );
        assert_eq!(search_href(Scope::Collection, ""), "/");
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
        let tabs: Vec<Tab> = items().into_iter().map(|(tab, ..)| tab).collect();
        assert_eq!(tabs, Tab::ALL);
    }
}
