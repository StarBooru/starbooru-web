//! The booru browser, Explore's other side: one search on the sites chosen
//! in the row of chips, in the gallery's grid, through the server's
//! `/remote/search`. Picked posts are saved here as the server's imports
//! (`activity`), which carry on when the app closes. The results are kept
//! while the app runs, so coming back from a post finds the grid as it was.

mod accounts;
mod grid;
mod post;
mod searches;
mod sites;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::{DateTime, Utc};
use futures_util::future::join_all;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_query_map};
use send_wrapper::SendWrapper;
use starbooru_core::{
    BooruAccount, ImportJob, ImportRequest, RemotePost, RemoteSearch, RemoteSite,
};
use wasm_bindgen::JsCast;

use self::accounts::load_accounts;
use self::grid::{RemoteThumb, SaveBar};
pub use self::post::BooruPost;
use self::searches::{SaveSearch, SavedList};
use self::sites::{SiteChips, SitesSheet};
use super::bad_tag_text;
use super::explore::{self, Sides};
use super::gallery::{self, Grid, ToTop, ViewMenu};
use crate::activity;
use crate::api;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::icons::{Ico, Icon};
use crate::dom;
use crate::logic::boorus::post::import_request;
use crate::logic::boorus::results::{
    Around, Found, Picks, SitePage, any_newly_saved, append, around, fresh, interleave, mark_saved,
    shown, size_of, status_of,
};
use crate::logic::boorus::{chosen_sites, from_address, site_key, sites_param, tidy_query};
use crate::logic::jobs::by_source;
use crate::logic::layout;
use crate::route;
use crate::store;

/// The last search with what it found, kept while the app runs.
#[derive(Clone)]
struct Kept {
    owner: String,
    q: String,
    sites: Vec<RemoteSite>,
    all: Vec<Arc<RemotePost>>,
    pages: Vec<SitePage>,
    loaded_at: DateTime<Utc>,
    /// How far down the page was.
    y: f64,
}

impl Kept {
    fn is(&self, q: &str, sites: &[RemoteSite]) -> bool {
        self.owner == store::owner() && self.q == q && self.sites == sites
    }
}

thread_local! {
    static KEPT: RefCell<Option<Kept>> = const { RefCell::new(None) };
    // Where the boorus side was left, and whose that was, for the navigation
    // and Explore's switch. Reference counted, as it outlives any one page.
    static LAST: ArcRwSignal<Option<(String, String)>> = ArcRwSignal::new(None);
}

/// What is kept for this search on these sites, while its picture addresses
/// still work.
fn kept_for(q: &str, sites: &[RemoteSite]) -> Option<Kept> {
    let now = Utc::now();
    KEPT.with_borrow(|k| {
        k.as_ref()
            .filter(|k| k.is(q, sites) && fresh(k.loaded_at, now))
            .cloned()
    })
}

/// Notes how far down the page the search is.
fn keep_y(q: &str, sites: &[RemoteSite], y: f64) {
    KEPT.with_borrow_mut(|k| {
        if let Some(kept) = k.as_mut().filter(|k| k.is(q, sites)) {
            kept.y = y;
        }
    });
}

fn kept_y(q: &str, sites: &[RemoteSite]) -> f64 {
    KEPT.with_borrow(|k| k.as_ref().filter(|k| k.is(q, sites)).map_or(0.0, |k| k.y))
}

fn set_last(q: &str, sites: &[RemoteSite]) {
    let now = Some((store::owner(), route::boorus(q, &sites_param(sites))));
    LAST.with(|l| {
        if l.with_untracked(|l| *l != now) {
            l.set(now);
        }
    });
}

/// Where Explore's boorus side opens: back on the last search, as the
/// gallery does on its own. Read inside a view, it follows it.
pub fn last_route() -> String {
    let owner = store::owner();
    LAST.with(|l| {
        l.with(|l| {
            l.as_ref()
                .filter(|(whose, _)| *whose == owner)
                .map(|(_, to)| to.clone())
        })
    })
    .unwrap_or_else(|| route::BOORUS.to_owned())
}

/// The kept post at page `u`, with its neighbours in the grid.
fn kept_around(u: &str) -> Option<Around> {
    let hide = store::prefs_untracked().booru_dedupe;
    KEPT.with_borrow(|k| {
        let kept = k.as_ref().filter(|k| k.owner == store::owner())?;
        let list = shown(&kept.all, hide && kept.sites.len() > 1);
        around(&list, &kept.all, &kept.sites, u)
    })
}

/// Notes on the kept results the imports that have finished since, so their
/// posts stay saved after the jobs are forgotten.
fn note_saved(jobs: &HashMap<String, ImportJob>) {
    KEPT.with_borrow_mut(|k| {
        if let Some(kept) = k.as_mut() {
            mark_saved(&mut kept.all, jobs);
        }
    });
}

/// Focus goes back to the button that opened the sheet of sites.
fn focus_sites_button() {
    let button = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.query_selector(".sites-edit").ok().flatten())
        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok());
    if let Some(button) = button {
        let _ = button.focus();
    }
}

/// Queues imports for `requests`. The words to show when that fails.
async fn queue(requests: Vec<ImportRequest>) -> Result<(), String> {
    activity::queue_imports(requests).await
}

/// The booru browser: search `q` on the sites `s` names, in the gallery's
/// grid.
#[component]
pub fn Boorus() -> impl IntoView {
    explore::on_boorus(true);
    let query = use_query_map();
    let q = Memo::new(move |_| {
        query.with(|m| tidy_query(&from_address(&m.get("q").unwrap_or_default())))
    });
    let s = Memo::new(move |_| query.with(|m| m.get("s").unwrap_or_default()));
    // With no sites in the address, the ones chosen last.
    let sites =
        Memo::new(move |_| s.with(|s| chosen_sites(s, &store::prefs_untracked().booru_sites)));

    // What was kept is there from the first frame; `restart` takes over after.
    let first = kept_for(&q.get_untracked(), &sites.get_untracked());
    let all = RwSignal::new(first.as_ref().map(|k| k.all.clone()).unwrap_or_default());
    let pages = RwSignal::new(first.as_ref().map(|k| k.pages.clone()).unwrap_or_default());
    let current = RwSignal::new((q.get_untracked(), sites.get_untracked()));
    let loading = RwSignal::new(false);
    let started = RwSignal::new(first.is_some());
    let loaded_at = StoredValue::new(first.as_ref().map_or_else(Utc::now, |k| k.loaded_at));
    let error = RwSignal::new(None::<String>);
    // Bumped on every new search, so a late answer for an old one is dropped.
    let generation = StoredValue::new(0_u64);
    let width = RwSignal::new(gallery::kept_width());
    let shape = Memo::new(|_| {
        let p = store::prefs();
        (p.gallery_view, p.gallery_columns, p.flush_tiles)
    });
    let grid = Memo::new(move |_| Grid::new(shape.get(), width.get()));
    let dedupe = Memo::new(|_| store::prefs().booru_dedupe);
    let many = Memo::new(move |_| current.with(|(_, sites)| sites.len() > 1));
    let list = Memo::new(move |_| all.with(|a| shown(a, dedupe.get() && many.get())));
    let tiles =
        Memo::new(move |_| list.with(|l| grid.get().lay(l.iter().map(|f| size_of(&f.post)))));
    let near = RwSignal::new(layout::near(first.as_ref().map_or(0.0, |k| k.y)));
    // Tiles below this index aren't new, and don't make an entrance.
    let settled = RwSignal::new(list.with_untracked(Vec::len));
    let has_more = Memo::new(move |_| pages.with(|p| p.iter().any(SitePage::wanted)));
    let jobs = Memo::new(|_| activity::with_jobs(by_source));
    let masonry: NodeRef<html::Div> = NodeRef::new();
    let alive = Arc::new(AtomicBool::new(true));

    let selecting = RwSignal::new(false);
    let picks = RwSignal::new(Picks::default());
    let saving = RwSignal::new(false);
    let save_error = RwSignal::new(None::<String>);
    let stop_selecting = Callback::new(move |()| {
        selecting.set(false);
        picks.update(Picks::clear);
    });

    let keep = move || {
        let Some((q, sites)) = current.try_get_untracked() else {
            return;
        };
        let y = kept_y(&q, &sites);
        let kept = Kept {
            owner: store::owner(),
            all: all.get_untracked(),
            pages: pages.get_untracked(),
            loaded_at: loaded_at.get_value(),
            q,
            sites,
            y,
        };
        KEPT.with_borrow_mut(|k| *k = Some(kept));
    };

    // One more page from every site that has one.
    let load = move || {
        if loading.get_untracked() {
            return;
        }
        let q = current.with_untracked(|(q, _)| q.clone());
        let asked: Vec<(usize, RemoteSearch)> = pages.with_untracked(|pages| {
            pages
                .iter()
                .enumerate()
                .filter(|(_, page)| page.wanted())
                .map(|(index, page)| {
                    let search = RemoteSearch {
                        api: page.site.api,
                        base: page.site.base.clone(),
                        tags: q.clone(),
                        page: page.next,
                        // The server uses the account saved for the site.
                        login: None,
                        api_key: None,
                    };
                    (index, search)
                })
                .collect()
        });
        if asked.is_empty() {
            return;
        }
        let at = generation.get_value();
        loading.set(true);
        spawn_local(async move {
            let answers = join_all(asked.iter().map(|(_, search)| {
                let search = search.clone();
                api::call(move |c| async move { c.remote_search(&search).await })
            }))
            .await;
            if generation.try_get_value() != Some(at) {
                return;
            }
            let known: HashSet<String> =
                all.with_untracked(|a| a.iter().map(|p| p.page_url.clone()).collect());
            let mut found = Vec::new();
            // One site's trouble is its own: the others' posts still show.
            pages.update(|pages| {
                for ((index, _), answer) in asked.iter().zip(answers) {
                    let Some(page) = pages.get_mut(*index) else {
                        continue;
                    };
                    match answer {
                        Ok(result) => {
                            page.answered(&result.posts, result.more, &known);
                            found.push(result.posts);
                        }
                        Err(message) => {
                            page.error = Some(message);
                            found.push(Vec::new());
                        }
                    }
                }
            });
            all.update(|a| {
                append(a, interleave(found));
            });
            let failed = pages.with_untracked(|p| p.iter().find_map(|p| p.error.clone()));
            error.set(failed.filter(|_| all.with_untracked(Vec::is_empty)));
            loading.set(false);
            started.set(true);
            keep();
        });
    };

    // The part of the grid near the screen, a quarter of a screen at a time.
    let check_near = move |force: bool| {
        let Some(el) = masonry.try_get_untracked().flatten() else {
            return;
        };
        let top = -el.get_bounding_client_rect().top();
        let (_, tall) = dom::scroll();
        let held = (!force).then(|| near.get_untracked());
        if let Some(part) = layout::hold(held, top, tall) {
            near.set(part);
        }
    };
    // Back where the page was, once the grid is tall enough to be there.
    let put_back = move |y: f64| {
        fn go(tries: u32, y: f64, done: impl Fn() + 'static) {
            let tall = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.document_element())
                .map_or(0.0, |el| f64::from(el.scroll_height()));
            let (_, screen) = dom::scroll();
            if tall - screen < y && tries > 0 {
                request_animation_frame(move || go(tries - 1, y, done));
                return;
            }
            dom::scroll_to(y, false);
            done();
        }
        request_animation_frame(move || go(5, y, move || check_near(true)));
    };

    let restart = move |q: String, sites: Vec<RemoteSite>| {
        generation.update_value(|g| *g = g.wrapping_add(1));
        let keys: Vec<String> = sites.iter().map(site_key).collect();
        if store::prefs_untracked().booru_sites != keys {
            store::update_prefs(|p| p.booru_sites = keys);
        }
        current.set((q.clone(), sites.clone()));
        error.set(None);
        stop_selecting.run(());
        save_error.set(None);
        loading.set(false);
        set_last(&q, &sites);
        if let Some(kept) = kept_for(&q, &sites) {
            loaded_at.set_value(kept.loaded_at);
            error.set(
                kept.pages
                    .iter()
                    .find_map(|p| p.error.clone())
                    .filter(|_| kept.all.is_empty()),
            );
            // Nothing that was there before makes an entrance.
            settled.set(kept.all.len());
            near.set(layout::near(kept.y));
            all.set(kept.all);
            pages.set(kept.pages);
            started.set(true);
            put_back(kept.y);
            return;
        }
        all.set(Vec::new());
        settled.set(0);
        pages.set(sites.iter().map(SitePage::first).collect());
        loaded_at.set_value(Utc::now());
        near.set(layout::near(0.0));
        started.set(false);
        dom::scroll_to(0.0, false);
        load();
    };
    Effect::watch(
        move || (q.get(), sites.get()),
        move |(q, sites), _, _| restart(q.clone(), sites.clone()),
        true,
    );
    let again = Callback::new(move |()| {
        KEPT.with_borrow_mut(|k| *k = None);
        let (q, sites) = current.get_untracked();
        restart(q, sites);
    });

    // More comes before the user gets to the end of what is there.
    Effect::new(move |_| {
        let (_, to) = near.get();
        let Grid { gap, tile, .. } = grid.get();
        let end = tiles.with(|t| t.height.px(tile, gap));
        if tile > 0.0 && to >= end && started.get() && has_more.get() && !loading.get() {
            load();
        }
    });
    // The entrance plays for what arrives with a page, not for tiles there
    // before it.
    Effect::new(move |_| {
        let count = list.with(Vec::len);
        request_animation_frame(move || {
            let _ = settled.try_set(count);
        });
    });
    // A post saved from here stays marked after its job is forgotten.
    Effect::new(move |_| {
        let jobs = jobs.get();
        if all.with_untracked(|a| any_newly_saved(a, &jobs)) {
            all.update(|a| {
                mark_saved(a, &jobs);
            });
            keep();
        }
    });
    // A new layout moves everything: back to the same place down the page.
    let arrangement = Memo::new(move |_| grid.with(|g| (g.view, g.columns, g.gap.to_bits())));
    Effect::watch(
        move || arrangement.get(),
        move |_, before, _| {
            if before.is_some() {
                let y = current.with_untracked(|(q, sites)| kept_y(q, sites));
                put_back(y);
            }
        },
        false,
    );

    let watches = StoredValue::new_local(Vec::<dom::Watch>::new());
    masonry.on_load({
        let alive = Arc::clone(&alive);
        move |el| {
            let mut all = Vec::new();
            all.push(dom::on_width(&el, move |w| {
                gallery::keep_width(w);
                if width.get_untracked() != w {
                    width.set(w);
                }
            }));
            let frame = Rc::new(std::cell::Cell::new(false));
            all.push(dom::on_window("scroll", move || {
                if frame.replace(true) {
                    return;
                }
                let frame = Rc::clone(&frame);
                let alive = Arc::clone(&alive);
                request_animation_frame(move || {
                    frame.set(false);
                    if !alive.load(Ordering::Relaxed) {
                        return;
                    }
                    check_near(false);
                    if let Some((q, sites)) = current.try_get_untracked() {
                        keep_y(&q, &sites, dom::scroll().0);
                    }
                });
            }));
            all.push(dom::on_window("resize", move || check_near(false)));
            watches.update_value(|w| w.extend(all));
            check_near(true);
        }
    });
    on_cleanup({
        let watches = SendWrapper::new(watches);
        move || {
            alive.store(false, Ordering::Relaxed);
            if let Some(all) = watches.try_update_value(std::mem::take) {
                for watch in all {
                    watch.stop();
                }
            }
        }
    });

    // The accounts saved on the server; what an older app kept on this
    // device moves there first.
    let accounts = RwSignal::new(Vec::<BooruAccount>::new());
    load_accounts(accounts);
    searches::load();

    // Notes where the page is before leaving it for a post, and that the
    // post's Back can go back through history to it.
    let open = Callback::new(move |()| {
        if let Some((q, sites)) = current.try_get_untracked() {
            keep_y(&q, &sites, dom::scroll().0);
            post::opening(&q, &s.get_untracked());
        }
    });
    let toggle = Callback::new(move |(index, shift): (usize, bool)| {
        let free: Vec<(String, bool)> = list.with_untracked(|l| {
            jobs.with_untracked(|j| {
                l.iter()
                    .map(|f| (f.post.page_url.clone(), status_of(&f.post, j).is_none()))
                    .collect()
            })
        });
        picks.update(|p| p.toggle(&free, index, shift));
    });
    // What can still be picked: not what is saved or on its way.
    let free = Memo::new(move |_| {
        list.with(|l| {
            jobs.with(|j| {
                l.iter()
                    .filter(|f| status_of(&f.post, j).is_none())
                    .map(|f| f.post.page_url.clone())
                    .collect::<Vec<_>>()
            })
        })
    });
    let save = Callback::new(move |extra: String| {
        if saving.get_untracked() {
            return;
        }
        // Only what the grid still shows, and still free to save.
        let requests: Result<Vec<ImportRequest>, _> = list.with_untracked(|l| {
            picks.with_untracked(|p| {
                jobs.with_untracked(|j| {
                    l.iter()
                        .filter(|f| p.contains(&f.post.page_url) && status_of(&f.post, j).is_none())
                        .map(|f| import_request(&f.post, &extra))
                        .collect()
                })
            })
        });
        let requests = match requests {
            Ok(requests) if requests.is_empty() => return,
            Ok(requests) => requests,
            Err(bad) => {
                save_error.set(Some(untrack(|| bad_tag_text(&bad))));
                return;
            }
        };
        saving.set(true);
        save_error.set(None);
        // The server runs the imports; their progress comes with the jobs.
        spawn_local(async move {
            match queue(requests).await {
                // The page may have gone meanwhile.
                Ok(()) => {
                    stop_selecting.try_run(());
                }
                Err(message) => {
                    save_error.try_set(Some(message));
                }
            }
            saving.try_set(false);
        });
    });

    let near_tiles = Memo::new(move |_| {
        let Grid { tile, gap, .. } = grid.get();
        let (from, to) = near.get();
        let indices = tiles.with(|t| t.within(tile, gap, from, to));
        list.with(|l| {
            indices
                .into_iter()
                .filter_map(|i| Some((i, l.get(i)?.post.page_url.clone())))
                .collect::<Vec<_>>()
        })
    });
    let in_sight = move |index: usize| {
        let Grid { tile, gap, .. } = grid.get_untracked();
        let (from, to) = layout::may_be_on_screen(near.get_untracked());
        tiles.with_untracked(|t| {
            t.tiles.get(index).is_some_and(|p| {
                let top = p.top.px(tile, gap);
                top < to && top + p.height.px(tile, gap) > from
            })
        })
    };
    let tile_view = move |(index, page): (usize, String)| {
        let found: Found = list.with_untracked(|l| l.get(index).cloned())?;
        let place = Signal::derive(move || tiles.with(|t| t.tiles.get(index).copied()));
        let href = Signal::derive(move || {
            route::booru_post(&page, &current.with(|(q, _)| q.clone()), &s.get())
        });
        let still = index < settled.get_untracked() || !in_sight(index);
        let post = Arc::clone(&found.post);
        let status = Signal::derive(move || jobs.with(|j| status_of(&post, j)));
        let page = found.post.page_url.clone();
        let selected = Signal::derive(move || picks.with(|p| p.contains(&page)));
        let site = Signal::derive(move || many.get());
        Some(view! {
            <RemoteThumb
                post=found.post
                index=index
                href=href
                place=place
                still=still
                many=site
                status=status
                selecting=selecting
                selected=selected
                on_open=open
                on_toggle=toggle
            />
        })
    };
    let masonry_style = move || {
        let columns = grid.with(|g| g.columns);
        tiles.with(|t| {
            format!(
                "--columns: {columns}; --height: {:.5}; --height-gaps: {:.5}",
                t.height.tiles, t.height.gaps
            )
        })
    };

    let navigate = SendWrapper::new(use_navigate());
    let go = Callback::new(move |next: Vec<RemoteSite>| {
        let to = route::boorus(&q.get_untracked(), &sites_param(&next));
        let options = NavigateOptions {
            replace: true,
            scroll: false,
            ..NavigateOptions::default()
        };
        (*navigate)(&to, options);
    });
    let sheet = RwSignal::new(false);
    let failed = Memo::new(move |_| {
        pages.with(|p| {
            p.iter()
                .filter_map(|p| Some((p.site.clone(), p.error.clone()?)))
                .collect::<Vec<_>>()
        })
    });
    let empty = move || {
        started.get()
            && list.with(Vec::is_empty)
            && !loading.get()
            && error.with(Option::is_none)
            && !has_more.get()
    };

    view! {
        <div class="page boorus">
            <div class="page-head">
                // The search box above shows the search; the two sides of
                // Explore stand where the gallery has its title.
                <h1 class="page-title visually-hidden">
                    {move || {
                        let q = q.get();
                        if q.is_empty() { t!("nav-boorus") } else { q }
                    }}
                </h1>
                <Sides boorus=true />
                <div class="page-tools">
                    <SaveSearch q=q sites=sites />
                    <Show when=move || many.get()>
                        <button
                            type="button"
                            class="icon-btn dedupe"
                            class:on=move || dedupe.get()
                            title=move || t!("boorus-dedupe")
                            aria-label=move || t!("boorus-dedupe")
                            aria-pressed=move || if dedupe.get() { "true" } else { "false" }
                            on:click=move |_| store::update_prefs(|p| p.booru_dedupe = !p.booru_dedupe)
                        >
                            <Ico icon=Icon::Copy />
                        </button>
                    </Show>
                    <ViewMenu grid=grid />
                    <button
                        type="button"
                        class="icon-btn select-toggle"
                        class:on=move || selecting.get()
                        title=move || {
                            if selecting.get() { t!("gallery-stop-selecting") } else { t!("gallery-select") }
                        }
                        aria-label=move || {
                            if selecting.get() { t!("gallery-stop-selecting") } else { t!("gallery-select") }
                        }
                        aria-pressed=move || if selecting.get() { "true" } else { "false" }
                        on:click=move |_| {
                            if selecting.get_untracked() {
                                stop_selecting.run(());
                            } else {
                                save_error.set(None);
                                selecting.set(true);
                            }
                        }
                    >
                        {move || {
                            view! { <Ico icon=if selecting.get() { Icon::Close } else { Icon::Check } /> }
                        }}
                    </button>
                </div>
            </div>
            <SiteChips
                sites=sites
                failed=failed
                loading=loading
                on_go=go
                on_again=again
                on_sheet=Callback::new(move |()| sheet.set(true))
            />
            <Show when=move || q.with(String::is_empty)>
                <SavedList />
            </Show>
            {move || {
                error.get().map(|message| view! { <ErrorBanner message=message on_retry=again /> })
            }}
            <Show when=empty>
                <div class="card empty">
                    <Ico icon=Icon::Search size=48 />
                    <h2>{move || t!("no-matches")}</h2>
                </div>
            </Show>
            <div class="masonry-box" class:selecting=move || selecting.get()>
                <div class="masonry" node_ref=masonry style=masonry_style>
                    <For each=move || near_tiles.get() key=|(index, page)| (page.clone(), *index) children=tile_view />
                </div>
            </div>
            <div class="sentinel" class:done=move || !has_more.get() && !loading.get()>
                {move || {
                    if loading.get() {
                        view! { <Spinner /> }.into_any()
                    } else {
                        view! {
                            <button type="button" class="btn tonal" on:click=move |_| load()>
                                {move || t!("load-more")}
                            </button>
                        }
                            .into_any()
                    }
                }}
            </div>
            <Show when=move || selecting.get()>
                <SaveBar
                    picks=picks
                    free=free
                    busy=saving
                    error=save_error
                    on_save=save
                    on_close=stop_selecting
                />
            </Show>
        </div>
        <ToTop selecting=selecting />
        <Show when=move || sheet.get()>
            <SitesSheet
                sites=sites
                accounts=accounts
                on_go=go
                on_close=Callback::new(move |()| {
                    sheet.set(false);
                    // Once the sheet is gone, which also lets go of the focus.
                    request_animation_frame(focus_sites_button);
                })
            />
        </Show>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_page_is_split_and_its_grid_is_the_gallerys() {
        let source = include_str!("mod.rs");
        assert!(source.contains(r#"<div class="masonry" node_ref=masonry"#));
        assert!(source.contains("<ViewMenu grid=grid />"));
        assert!(source.contains("<Sides boorus=true />"));
        // A site's own trouble shows on its chip, and the rest still load.
        assert!(source.contains("page.error = Some(message)"));
    }
}
