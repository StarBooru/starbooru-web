//! The people page at `/people?q=…`: everyone in the collection as round
//! faces, each leading to their posts, grouped by franchise or in the order
//! picked, more coming as the list nears its end. The search box filters
//! them by name (`shell.rs`). A guest on a public server sees it too; with
//! People off in Settings it says so instead. The list is kept a while, so
//! coming back from someone's posts shows it at once, where it was.

use std::cell::{Cell, RefCell};

use chrono::Utc;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_query_map;
use starbooru_core::{PeopleOrder, Person};

use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::icons::{Ico, Icon};
use crate::components::person::PersonFace;
use crate::components::tag_menu::TagPress;
use crate::logic::characters::{
    Counts, FRANCHISES, Listed, ORDERS, grouped, joined, query, spoken,
};
use crate::logic::settings::{Group, anchored};
use crate::{api, dom, instance, route, store};

thread_local! {
    static ORDER: Cell<PeopleOrder> = const { Cell::new(PeopleOrder::Franchise) };
    static KEPT: RefCell<Option<Listed>> = const { RefCell::new(None) };
    /// Someone on the page was opened: coming back goes to where they were.
    static LEFT: Cell<bool> = const { Cell::new(false) };
}

/// Drops the list kept, after a change to who is in the collection or how
/// many posts they're on: an upload, an edit, the blacklist.
pub fn forget() {
    KEPT.with_borrow_mut(|k| *k = None);
}

/// Whose the list is: the account, or a guest of this server.
fn reader() -> String {
    format!(
        "{}|{}",
        store::owner(),
        instance::guest_server().unwrap_or_default()
    )
}

fn order_label(order: PeopleOrder) -> String {
    match order {
        PeopleOrder::Franchise => t!("people-by-franchise"),
        PeopleOrder::Posts => t!("people-most-posts"),
        PeopleOrder::Name => t!("people-a-z"),
    }
}

/// The people listed, in a grid. Under a franchise's heading the franchise
/// goes unsaid.
fn faces(people: Vec<Person>, franchise: bool) -> impl IntoView {
    let list = people
        .into_iter()
        .enumerate()
        .map(|(index, person)| {
            let href = route::gallery(&person.name);
            view! { <PersonFace person=person href=href franchise=franchise index=index /> }
        })
        .collect_view();
    view! { <div class="people-grid">{list}</div> }
}

#[component]
pub fn People() -> impl IntoView {
    let on = Memo::new(|_| store::prefs().people);
    view! {
        <div class="page people-page">
            {move || {
                if on.get() {
                    view! { <Everyone /> }.into_any()
                } else {
                    view! { <PeopleOff /> }.into_any()
                }
            }}
        </div>
    }
}

/// People are off in Settings: the page says so, with the way back on.
#[component]
fn PeopleOff() -> impl IntoView {
    view! {
        <h1 class="page-title">{move || t!("people-title")}</h1>
        <div class="card empty">
            <span class="people-art" aria-hidden="true">
                <Ico icon=Icon::People size=40 />
            </span>
            <h2>{move || t!("people-off")}</h2>
            <a class="btn tonal" href=anchored(Group::Viewing, "viewer")>
                <Ico icon=Icon::Tune />
                {move || t!("people-turn-on")}
            </a>
        </div>
    }
}

#[component]
fn Everyone() -> impl IntoView {
    let query_map = use_query_map();
    let filter =
        Memo::new(move |_| query_map.with(|q| q.get("q").unwrap_or_default().trim().to_owned()));
    let order = RwSignal::new(ORDER.get());
    let people = RwSignal::new(Vec::<Person>::new());
    let counts = RwSignal::new(None::<Counts>);
    let next_page = StoredValue::new(1_u32);
    let has_more = RwSignal::new(true);
    let loading = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let generation = StoredValue::new(0_u64);
    let sentinel: NodeRef<html::Div> = NodeRef::new();
    // Where the page is scrolled, to come back to.
    let scrolled = StoredValue::new(0.0_f64);
    // The newest post, for the way to mark people when there are none.
    let newest = RwSignal::new(None::<i64>);

    // Keeps what's listed, for coming back.
    let keep = move || {
        let Some(list) = people.try_get_untracked() else {
            return;
        };
        let listed = Listed {
            owner: reader(),
            q: filter.get_untracked(),
            order: order.get_untracked(),
            people: list,
            next: next_page.get_value(),
            has_more: has_more.get_untracked(),
            counts: counts.get_untracked(),
            scroll: scrolled.get_value(),
            at: Utc::now(),
        };
        KEPT.with_borrow_mut(|k| *k = Some(listed));
    };

    let load_more = move || {
        if loading.get_untracked()
            || !has_more.get_untracked()
            || error.with_untracked(Option::is_some)
        {
            return;
        }
        let at = generation.get_value();
        let asked = query(
            &filter.get_untracked(),
            order.get_untracked(),
            next_page.get_value(),
        );
        let page = asked.page;
        loading.set(true);
        spawn_local(async move {
            let result = api::call(move |c| async move { c.people(&asked).await }).await;
            if generation.try_get_value() != Some(at) {
                return;
            }
            loading.set(false);
            match result {
                Ok(found) => {
                    has_more.set(found.has_more);
                    next_page.set_value(page.saturating_add(1));
                    people.update(|list| joined(list, found.people));
                    keep();
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };
    // The headings' counts, for the grouped order. Without them a heading
    // counts those listed.
    let load_counts = move || {
        let at = generation.get_value();
        spawn_local(async move {
            let result =
                api::call(move |c| async move { c.franchises("", 1, FRANCHISES).await }).await;
            if generation.try_get_value() == Some(at)
                && let Ok(page) = result
            {
                counts.try_set(Some(Counts::of(&page)));
                keep();
            }
        });
    };
    let restart = move || {
        generation.update_value(|g| *g = g.wrapping_add(1));
        people.set(Vec::new());
        counts.set(None);
        next_page.set_value(1);
        has_more.set(true);
        loading.set(false);
        error.set(None);
        if order.get_untracked() == PeopleOrder::Franchise {
            load_counts();
        }
        load_more();
    };
    // What was left here a moment ago comes back as it was, scrolled where
    // it was if someone on it was opened.
    let back = LEFT.replace(false);
    let kept = KEPT.with_borrow(|k| {
        k.clone().filter(|k| {
            k.fits(
                &reader(),
                &filter.get_untracked(),
                order.get_untracked(),
                Utc::now(),
            )
        })
    });
    let resumed = kept.is_some();
    if let Some(k) = kept {
        people.set(k.people);
        counts.set(k.counts);
        next_page.set_value(k.next);
        has_more.set(k.has_more);
        scrolled.set_value(k.scroll);
        if back {
            let y = k.scroll;
            request_animation_frame(move || dom::scroll_to(y, false));
        }
    }
    Effect::watch(
        move || (filter.get(), order.get()),
        move |_, _, _| restart(),
        !resumed,
    );

    // The end of the list in sight, after a page that filled too little of
    // the screen: the next one comes by itself.
    let near_end = move || {
        sentinel.get_untracked().is_some_and(|el| {
            let (_, tall) = dom::scroll();
            el.get_bounding_client_rect().top() < tall * 2.5
        })
    };
    Effect::new(move |_| {
        let ready = has_more.get() && !loading.get() && error.with(Option::is_none);
        people.track();
        if ready && near_end() {
            load_more();
        }
    });
    let watches = StoredValue::new(Vec::<dom::Watch>::new());
    sentinel.on_load(move |el| {
        let near = dom::on_near(&el, "0px 0px 150% 0px", load_more);
        // Only while the address is still this page's: leaving scrolls the
        // next page to its top.
        let scroll = dom::on_window("scroll", move || {
            let here = web_sys::window()
                .and_then(|w| w.location().pathname().ok())
                .is_some_and(|p| route::is_people(&p));
            if here {
                let (y, _) = dom::scroll();
                scrolled.try_set_value(y);
                KEPT.with_borrow_mut(|k| {
                    if let Some(k) = k.as_mut() {
                        k.scroll = y;
                    }
                });
            }
        });
        watches.set_value(vec![near, scroll]);
    });
    on_cleanup(move || {
        if let Some(all) = watches.try_update_value(std::mem::take) {
            for w in all {
                w.stop();
            }
        }
    });

    // Nobody yet: the way to mark someone leads to the newest post's people.
    let empty = Memo::new(move |_| {
        !loading.get()
            && !has_more.get()
            && error.with(Option::is_none)
            && people.with(Vec::is_empty)
    });
    Effect::new(move |_| {
        if empty.get()
            && filter.with(String::is_empty)
            && !instance::guest()
            && newest.get_untracked().is_none()
        {
            spawn_local(async move {
                let found =
                    api::call(|c| async move { c.search_posts("", None, 1, 1).await }).await;
                if let Some(id) = found.ok().and_then(|p| p.posts.first().map(|p| p.id)) {
                    newest.try_set(Some(id));
                }
            });
        }
    });

    let sorts = ORDERS.map(|o| {
        let on = move || order.get() == o;
        view! {
            <button
                type="button"
                role="radio"
                class="seg"
                class:on=on
                aria-checked=move || if on() { "true" } else { "false" }
                on:click=move |_| {
                    ORDER.set(o);
                    order.set(o);
                }
            >
                {move || order_label(o)}
            </button>
        }
    });

    let list = move || {
        let all = people.get();
        if all.is_empty() {
            return None;
        }
        if order.get() != PeopleOrder::Franchise {
            return Some(faces(all, true).into_any());
        }
        let narrowed = filter.with(|f| !f.is_empty());
        let groups = grouped(&all)
            .into_iter()
            .map(|group| {
                let shown = counts.with(|c| {
                    c.as_ref().map_or(group.people.len(), |c| {
                        c.heading(group.franchise.as_deref(), group.people.len(), narrowed)
                    })
                });
                let title = t!("people-count", count = shown);
                let head = match group.franchise.clone() {
                    Some(name) => {
                        let label = t!("people-franchise-search", franchise = spoken(&name));
                        view! {
                            <a
                                class="franchise-link"
                                href=route::gallery(&name)
                                data-tag=name.clone()
                                aria-label=label
                            >
                                <span class="franchise-name">{spoken(&name)}</span>
                                <Ico icon=Icon::ChevronRight size=20 />
                            </a>
                        }
                        .into_any()
                    }
                    None => view! {
                        <span class="franchise-name none">{move || t!("people-no-franchise")}</span>
                    }
                    .into_any(),
                };
                view! {
                    <section class="franchise">
                        <h2 class="franchise-head">
                            {head}
                            <span class="franchise-count" title=title>
                                {shown}
                            </span>
                        </h2>
                        {faces(group.people, false)}
                    </section>
                }
            })
            .collect_view();
        Some(view! { <div class="franchises">{groups}</div> }.into_any())
    };

    let nobody = move || {
        let narrowed = filter.with(|f| !f.is_empty());
        let line = if narrowed {
            let typed = filter.get();
            view! {
                <p class="muted break">{move || t!("people-none-match", query = typed.clone())}</p>
                <a class="btn tonal" href=route::PEOPLE>
                    {move || t!("people-everyone")}
                </a>
            }
            .into_any()
        } else if instance::guest() {
            view! { <p class="muted">{move || t!("people-none-yet")}</p> }.into_any()
        } else {
            // The viewer's People card is where people are marked.
            let to = move || match newest.get() {
                Some(id) => format!("{}#people", route::post(id, "", None)),
                None => route::UPLOAD.to_owned(),
            };
            view! {
                <a class="people-start" href=to>
                    {move || t!("people-mark-first")}
                    <Ico icon=Icon::ChevronRight size=20 />
                </a>
            }
            .into_any()
        };
        view! {
            <div class="card empty">
                <span class="people-art" aria-hidden="true">
                    <Ico icon=Icon::People size=40 />
                </span>
                {line}
            </div>
        }
    };

    let retry = Callback::new(move |()| {
        error.set(None);
        load_more();
    });
    view! {
        <div class="page-head">
            <h1 class="page-title">{move || t!("people-title")}</h1>
            <div class="segmented" role="radiogroup" aria-label=move || t!("people-order")>
                {sorts}
            </div>
        </div>
        {move || error.get().map(|message| view! { <ErrorBanner message=message on_retry=retry /> })}
        <Show when=move || empty.get()>{nobody}</Show>
        // Opening someone keeps the place, for coming back.
        <div data-press="tag" on:click=move |_| LEFT.set(true)>
            {list}
        </div>
        <div
            class="sentinel"
            class:done=move || !has_more.get() || error.with(Option::is_some)
            node_ref=sentinel
        >
            {move || {
                if loading.get() {
                    view! { <Spinner /> }.into_any()
                } else {
                    view! {
                        <button type="button" class="btn tonal" on:click=move |_| load_more()>
                            {move || t!("load-more")}
                        </button>
                    }
                        .into_any()
                }
            }}
        </div>
        <TagPress q=filter />
    }
}
