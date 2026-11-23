//! The gallery at `/`, for the search in `?q=`: pages of results laid out by
//! `logic::layout`, more of them as the end comes near, and only the tiles
//! near the screen on the page. It remembers where it was in each search and
//! goes back there.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use chrono::Utc;
use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_query_map};
use send_wrapper::SendWrapper;
use starbooru_core::color::{NAMED_COLORS, to_hex};
use starbooru_core::{Post, parse_query};

use crate::api;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::color_picker::ColorPicker;
use crate::components::icons::{Ico, Icon};
use crate::components::select::{Choice, Select};
use crate::components::thumb::Thumb;
use crate::dom;
use crate::logic::gallery::{
    List, Lists, MAX_RESTORE, PAGE_SIZE, SORTS, Seeds, colored, current_color, current_sort,
    is_random, merge_head, mix_seed, page_after_restore, rest, restore_request, single_tag, sorted,
    title,
};
use crate::logic::layout::{self, Layout, TILE_GAP};
use crate::route;
use crate::store::{self, GalleryView, Spot};

thread_local! {
    static LISTS: RefCell<Lists> = RefCell::new(Lists::default());
    static SEEDS: RefCell<Seeds> = RefCell::new(Seeds::default());
    /// The grid's width, kept so coming back lays it out at once.
    static GRID_WIDTH: Cell<f64> = const { Cell::new(0.0) };
}

/// The seed to shuffle search `q` with, picking one the first time. `None`
/// unless `q` is `order:random`.
pub fn seed(q: &str) -> Option<i64> {
    if !is_random(q) {
        return None;
    }
    SEEDS.with_borrow(|s| s.get(q)).or_else(|| Some(reseed(q)))
}

/// A new shuffle for search `q`.
fn reseed(q: &str) -> i64 {
    static COUNT: AtomicU64 = AtomicU64::new(0);
    let nanos = Utc::now().timestamp_nanos_opt().unwrap_or_default();
    let seed = mix_seed(nanos, COUNT.fetch_add(1, Ordering::Relaxed));
    SEEDS.with_borrow_mut(|s| s.put(q, seed));
    seed
}

fn cached_list(q: &str) -> Option<List> {
    let owner = store::owner();
    LISTS.with_borrow(|l| l.find(q, &owner, Utc::now()))
}

fn keep_list(list: List) {
    LISTS.with_borrow_mut(|l| l.keep(list));
}

/// Drops the kept results, after a change that can move posts in or out of
/// any of them.
#[allow(dead_code, reason = "editing and deleting posts call it")]
pub fn forget_lists() {
    LISTS.with_borrow_mut(Lists::clear);
}

/// The grid's measurements as they are now.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Grid {
    view: GalleryView,
    columns: u32,
    gap: f64,
    width: f64,
    /// A tile's width in pixels.
    tile: f64,
}

impl Grid {
    fn new((view, chosen, flush): (GalleryView, Option<u8>, bool), width: f64) -> Self {
        let gap = if flush { 0.0 } else { TILE_GAP };
        let columns = layout::columns(width, gap, view, chosen);
        Self {
            view,
            columns,
            gap,
            width,
            tile: layout::tile_width(width, gap, columns),
        }
    }

    fn place(self, posts: &[Arc<Post>]) -> Layout {
        layout::place(
            self.view,
            posts
                .iter()
                .map(|p| layout::tile_ratio(self.view, p.width, p.height)),
            self.columns,
        )
    }

    /// The top of post `id`'s tile in pixels, from the top of the grid.
    fn tile_top(self, layout: &Layout, posts: &[Arc<Post>], id: i64) -> Option<f64> {
        let index = posts.iter().position(|p| p.id == id)?;
        let place = layout.tiles.get(index)?;
        Some(place.top.px(self.tile, self.gap))
    }
}

/// The part of the grid to put on the page for going back to a spot, so the
/// pictures above it don't start loading first.
fn near_spot(grid: Grid, posts: &[Arc<Post>], spot: Option<&Spot>) -> (f64, f64) {
    let top = spot.filter(|s| s.y >= 1.0).map_or(0.0, |s| {
        s.post
            .and_then(|id| grid.tile_top(&grid.place(posts), posts, id))
            .map_or(s.y, |top| top - s.offset)
    });
    layout::near(top)
}

/// Scrolls so the tile `top` pixels down the grid is `offset` pixels from the
/// top of the window, or to `y` without a tile. The list can land on the page
/// a frame or two after it's set, so it tries again for a few frames while the
/// grid is too short.
fn put_back(
    tries: u32,
    top: Option<f64>,
    offset: f64,
    y: f64,
    masonry: NodeRef<html::Div>,
    done: impl Fn() + 'static,
) {
    let Some(el) = masonry.try_get_untracked().flatten() else {
        return;
    };
    let rect = el.get_bounding_client_rect();
    let anchored = y >= 1.0 && top.is_some();
    let tile = top.unwrap_or(0.0);
    if anchored && tile > rect.height() && tries > 0 {
        request_animation_frame(move || put_back(tries - 1, top, offset, y, masonry, done));
        return;
    }
    let (now, _) = dom::scroll();
    dom::scroll_to(
        if anchored {
            now + rect.top() + tile - offset
        } else {
            y
        },
        false,
    );
    done();
}

/// The text of a sort menu entry.
fn sort_label(order: &str) -> String {
    match order {
        "new" => t!("sort-new"),
        "old" => t!("sort-old"),
        "score" => t!("sort-score"),
        "updated" => t!("sort-updated"),
        "size" => t!("sort-size"),
        "size_asc" => t!("sort-size-asc"),
        "pixels" => t!("sort-pixels"),
        "longside" => t!("sort-longside"),
        "tags" => t!("sort-tags"),
        "tags_asc" => t!("sort-tags-asc"),
        "random" => t!("sort-random"),
        other => other.to_owned(),
    }
}

fn color_label(name: &str) -> String {
    match name {
        "red" => t!("color-red"),
        "orange" => t!("color-orange"),
        "yellow" => t!("color-yellow"),
        "green" => t!("color-green"),
        "teal" => t!("color-teal"),
        "blue" => t!("color-blue"),
        "purple" => t!("color-purple"),
        "pink" => t!("color-pink"),
        "brown" => t!("color-brown"),
        "black" => t!("color-black"),
        "gray" => t!("color-gray"),
        "white" => t!("color-white"),
        other => other.to_owned(),
    }
}

fn view_label(view: GalleryView) -> String {
    match view {
        GalleryView::Square => t!("view-square"),
        GalleryView::Landscape | GalleryView::StaggeredLandscape => t!("view-landscape"),
        GalleryView::Portrait | GalleryView::StaggeredPortrait => t!("view-portrait"),
        GalleryView::StaggeredAll => t!("view-everything"),
    }
}

/// Goes to the gallery for a search, from a menu.
fn search_with() -> Callback<String> {
    let navigate = SendWrapper::new(use_navigate());
    Callback::new(move |q: String| (*navigate)(&route::gallery(&q), NavigateOptions::default()))
}

/// The scroll position's timer: the light under the pointer comes back once
/// the page has been still a moment, and the spot is remembered a moment
/// after that (`logic::gallery::rest`).
struct Settle {
    last: Cell<i64>,
    moving: Cell<bool>,
    waiting: Cell<bool>,
    alive: Arc<AtomicBool>,
    report: Box<dyn Fn()>,
}

impl Settle {
    fn moved(self: &Rc<Self>) {
        self.last.set(Utc::now().timestamp_millis());
        if !self.moving.replace(true) {
            // While the page moves, the tile under a still pointer changes every
            // frame, and each would start a fade of its own (gallery.css).
            dom::root_class("sb-scrolling", true);
        }
        if !self.waiting.replace(true) {
            self.wait(crate::logic::gallery::LIGHT_MS);
        }
    }

    fn wait(self: &Rc<Self>, ms: i64) {
        let me = Rc::clone(self);
        set_timeout(
            move || me.settle(),
            Duration::from_millis(u64::try_from(ms).unwrap_or(0)),
        );
    }

    fn settle(self: &Rc<Self>) {
        if !self.alive.load(Ordering::Relaxed) {
            return;
        }
        let still = Utc::now().timestamp_millis() - self.last.get();
        let next = rest(still, self.moving.get());
        if next.calm {
            self.moving.set(false);
            dom::root_class("sb-scrolling", false);
        }
        if next.report {
            self.waiting.set(false);
            (self.report)();
        }
        if let Some(ms) = next.wait {
            self.wait(ms);
        }
    }
}

#[component]
pub fn Gallery() -> impl IntoView {
    let query = use_query_map();
    let q = Memo::new(move |_| query.with(|m| m.get("q").unwrap_or_default()));
    let first = q.get_untracked();
    // A list kept from earlier is there from the first frame, so that coming
    // back never flashes an empty page.
    let cached = cached_list(&first);

    // The search these results are for, which can trail `q` by a moment.
    let shown_q = RwSignal::new(first);
    let posts = RwSignal::new(cached.as_ref().map(|l| l.posts.clone()).unwrap_or_default());
    let next_page = RwSignal::new(1_u32);
    let has_more = RwSignal::new(cached.as_ref().is_none_or(|l| l.has_more));
    let loading = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    // Bumped on every new search, so a late answer for an old one is dropped.
    let generation = StoredValue::new(0_u64);
    let started = RwSignal::new(false);
    // Tiles below this index aren't new, and don't make an entrance.
    let settled = RwSignal::new(posts.with_untracked(Vec::len));
    let loaded_at = StoredValue::new(Utc::now());
    let width = RwSignal::new(GRID_WIDTH.get());
    let shape = Memo::new(|_| {
        let p = store::prefs();
        (p.gallery_view, p.gallery_columns, p.flush_tiles)
    });
    let grid = Memo::new(move |_| Grid::new(shape.get(), width.get()));
    let tiles = Memo::new(move |_| posts.with(|p| grid.get().place(p)));
    let near = RwSignal::new(near_spot(
        grid.get_untracked(),
        &posts.read_untracked(),
        store::spot(&shown_q.get_untracked()).as_ref(),
    ));
    let masonry: NodeRef<html::Div> = NodeRef::new();
    let alive = Arc::new(AtomicBool::new(true));

    // The part of the grid near the screen, a quarter of a screen at a time.
    let check_near = move |force: bool| {
        // A frame can come after the page has gone.
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

    // Puts the remembered tile back where it was on the screen, or failing
    // that, the scroll position, once the list is on the page.
    let go_to_spot = move || {
        let spot = store::spot(&shown_q.get_untracked());
        let (post, offset, y) = spot.map_or((None, 0.0, 0.0), |s| (s.post, s.offset, s.y));
        let g = grid.get_untracked();
        let top = post.and_then(|id| {
            tiles.with_untracked(|t| posts.with_untracked(|p| g.tile_top(t, p, id)))
        });
        request_animation_frame(move || {
            put_back(5, top, offset, y, masonry, move || check_near(true));
        });
    };

    let keep = move || {
        let q = shown_q.get_untracked();
        keep_list(List {
            seed: seed(&q),
            q,
            owner: store::owner(),
            posts: posts.get_untracked(),
            next_page: next_page.get_untracked(),
            has_more: has_more.get_untracked(),
            loaded_at: loaded_at.get_value(),
        });
    };

    let load_more = move || {
        if !started.get_untracked()
            || loading.get_untracked()
            || !has_more.get_untracked()
            || error.with_untracked(Option::is_some)
        {
            return;
        }
        let at_start = generation.get_value();
        let page = next_page.get_untracked();
        let q = shown_q.get_untracked();
        // The server reads a search the same way and would only say this back.
        if let Err(e) = parse_query(&q) {
            error.set(Some(e.to_string()));
            return;
        }
        let shuffle = seed(&q);
        loading.set(true);
        spawn_local(async move {
            let result =
                api::call(
                    move |c| async move { c.search_posts(&q, shuffle, page, PAGE_SIZE).await },
                )
                .await;
            if generation.try_get_value() != Some(at_start) {
                return;
            }
            loading.set(false);
            match result {
                Ok(found) => {
                    // Posts added since can push earlier ones onto this page.
                    posts.update(|list| {
                        let seen: HashSet<i64> = list.iter().map(|p| p.id).collect();
                        let new = found.posts.into_iter().filter(|p| !seen.contains(&p.id));
                        list.extend(new.map(Arc::new));
                    });
                    has_more.set(found.has_more);
                    next_page.set(page.saturating_add(1));
                    keep();
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };

    // Loads as far as the user had got, in big pages, and goes back there.
    let restore = move |spot: Spot| {
        let at_start = generation.get_value();
        let wanted = usize::try_from(spot.count.min(MAX_RESTORE)).unwrap_or(0);
        loading.set(true);
        spawn_local(async move {
            let mut found = Vec::<Arc<Post>>::new();
            let mut seen = HashSet::new();
            let mut done = 0_u32;
            let mut more = true;
            let mut failure = None;
            while more && found.len() < wanted {
                let (page, size) = restore_request(done);
                let q = spot.q.clone();
                let result =
                    api::call(move |c| async move { c.search_posts(&q, None, page, size).await })
                        .await;
                if generation.try_get_value() != Some(at_start) {
                    return;
                }
                match result {
                    Ok(results) => {
                        let new = results.posts.into_iter().filter(|p| seen.insert(p.id));
                        found.extend(new.map(Arc::new));
                        more = results.has_more;
                        done = done.saturating_add(1);
                    }
                    Err(message) => {
                        failure = Some(message);
                        break;
                    }
                }
            }
            loading.set(false);
            settled.set(found.len());
            if failure.is_none() {
                near.set(near_spot(grid.get_untracked(), &found, Some(&spot)));
            }
            posts.set(found);
            next_page.set(page_after_restore(done));
            has_more.set(more);
            if failure.is_none() {
                keep();
                go_to_spot();
            }
            error.set(failure);
        });
    };

    // Picks up what changed at the top of a list kept from earlier.
    let refresh_head = move || {
        let q = shown_q.get_untracked();
        if is_random(&q) {
            return;
        }
        let at_start = generation.get_value();
        spawn_local(async move {
            let Ok(found) =
                api::call(move |c| async move { c.search_posts(&q, None, 1, PAGE_SIZE).await })
                    .await
            else {
                return;
            };
            if generation.try_get_value() != Some(at_start) {
                return;
            }
            let (head, more) = (found.posts.len(), found.has_more);
            let Some(merged) = posts.with_untracked(|p| merge_head(p, found.posts, more)) else {
                return;
            };
            if merged.len() == head {
                next_page.set(2);
                has_more.set(more);
            }
            settled.set(merged.len());
            posts.set(merged);
            keep();
            go_to_spot();
        });
    };

    let restart = move |q: String| {
        generation.update_value(|g| *g = g.wrapping_add(1));
        started.set(true);
        if parse_query(&q).is_ok() {
            store::visit_search(&q);
        }
        shown_q.set(q.clone());
        loading.set(false);
        error.set(None);
        if let Some(list) = cached_list(&q) {
            if let Some(seed) = list.seed {
                SEEDS.with_borrow_mut(|s| s.put(&q, seed));
            }
            settled.set(list.posts.len());
            near.set(near_spot(
                grid.get_untracked(),
                &list.posts,
                store::spot(&q).as_ref(),
            ));
            posts.set(list.posts);
            next_page.set(list.next_page);
            has_more.set(list.has_more);
            loaded_at.set_value(list.loaded_at);
            go_to_spot();
            refresh_head();
            return;
        }
        posts.set(Vec::new());
        settled.set(0);
        next_page.set(1);
        has_more.set(true);
        loaded_at.set_value(Utc::now());
        near.set(layout::near(0.0));
        // A search run again is shuffled again.
        if is_random(&q) {
            reseed(&q);
        }
        match store::spot(&q).filter(|s| s.y >= 1.0 && s.count > 0 && !is_random(&q)) {
            Some(spot) => restore(spot),
            None => {
                dom::scroll_to(0.0, false);
                load_more();
            }
        }
    };
    Effect::watch(move || q.get(), move |q, _, _| restart(q.clone()), true);

    // More of the list comes before the user gets to the end of it.
    Effect::new(move |_| {
        let (_, to) = near.get();
        let Grid { gap, tile, .. } = grid.get();
        let end = tiles.with(|t| t.height.px(tile, gap));
        if tile > 0.0
            && to >= end
            && has_more.get()
            && !loading.get()
            && error.with(Option::is_none)
        {
            load_more();
        }
    });

    // Where the gallery is: the first whole tile under the top bar, worked out
    // from the layout rather than by measuring tiles.
    let report = move || {
        let count = posts.with_untracked(Vec::len);
        let Some(el) = masonry.get_untracked().filter(|_| count > 0) else {
            return;
        };
        let grid_top = el.get_bounding_client_rect().top();
        let Grid { tile, gap, .. } = grid.get_untracked();
        let line = dom::bar_bottom() - grid_top;
        let Some((index, top)) = tiles.with_untracked(|t| t.first_below(tile, gap, line)) else {
            return;
        };
        let (y, _) = dom::scroll();
        store::remember_spot(Spot {
            q: shown_q.get_untracked(),
            post: posts.with_untracked(|p| p.get(index).map(|p| p.id)),
            offset: grid_top + top,
            y,
            count: u32::try_from(count).unwrap_or(u32::MAX),
        });
    };

    // A new layout moves everything, so the same tile goes back to the top. A
    // width within the same columns only scales it.
    let arrangement = Memo::new(move |_| grid.with(|g| (g.view, g.columns, g.gap.to_bits())));
    Effect::watch(
        move || arrangement.get(),
        move |_, before, _| {
            if before.is_some() {
                go_to_spot();
            }
        },
        false,
    );

    // The width of the grid, the scroll and the end of the list.
    let sentinel: NodeRef<html::Div> = NodeRef::new();
    let watches = StoredValue::new_local(Vec::<dom::Watch>::new());
    masonry.on_load({
        let alive = Arc::clone(&alive);
        move |el| {
            let mut all = Vec::new();
            all.push(dom::on_width(&el, move |w| {
                GRID_WIDTH.set(w);
                if width.get_untracked() != w {
                    width.set(w);
                }
            }));
            let settle = Rc::new(Settle {
                last: Cell::new(0),
                moving: Cell::new(false),
                waiting: Cell::new(false),
                alive: Arc::clone(&alive),
                report: Box::new(report),
            });
            let frame = Rc::new(Cell::new(false));
            let alive = Arc::clone(&alive);
            all.push(dom::on_window("scroll", move || {
                settle.moved();
                if !frame.replace(true) {
                    let frame = Rc::clone(&frame);
                    let alive = Arc::clone(&alive);
                    request_animation_frame(move || {
                        frame.set(false);
                        if alive.load(Ordering::Relaxed) {
                            check_near(false);
                        }
                    });
                }
            }));
            all.push(dom::on_window("resize", move || check_near(false)));
            watches.update_value(|w| w.extend(all));
            check_near(true);
        }
    });
    sentinel.on_load(move |el| {
        let watch = dom::on_near(&el, "0px 0px 150% 0px", load_more);
        watches.update_value(|w| w.push(watch));
    });
    on_cleanup({
        let watches = SendWrapper::new(watches);
        move || {
            alive.store(false, Ordering::Relaxed);
            dom::root_class("sb-scrolling", false);
            if let Some(all) = watches.try_update_value(std::mem::take) {
                for watch in all {
                    watch.stop();
                }
            }
        }
    });

    // The tiles near the screen, by place in the list and post.
    let shown = Memo::new(move |_| {
        let Grid { tile, gap, .. } = grid.get();
        let (from, to) = near.get();
        let indices = tiles.with(|t| t.within(tile, gap, from, to));
        posts.with(|p| {
            indices
                .into_iter()
                .filter_map(|i| Some((i, p.get(i)?.id)))
                .collect::<Vec<_>>()
        })
    });
    // New tiles only make an entrance where it can be seen.
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
    let tile_view = move |(index, _): (usize, i64)| {
        let post = posts.with_untracked(|p| p.get(index).cloned())?;
        let id = post.id;
        let place = Signal::derive(move || tiles.with(|t| t.tiles.get(index).copied()));
        let href = Signal::derive(move || {
            let q = shown_q.get();
            route::post(id, &q, seed(&q))
        });
        let still = index < settled.get_untracked() || !in_sight(index);
        Some(view! { <Thumb post=post index=index href=href place=place still=still /> })
    };
    // The entrance plays for what arrives with a page, not for tiles there
    // before it.
    Effect::new(move |_| {
        let count = posts.with(Vec::len);
        request_animation_frame(move || {
            let _ = settled.try_set(count);
        });
    });

    let retry = Callback::new(move |()| {
        error.set(None);
        load_more();
    });
    let heading = Memo::new(move |_| title(&q.get()));
    let tag = move || heading.with(|h| single_tag(h).map(str::to_owned));
    let empty = move || {
        started.get()
            && posts.with(Vec::is_empty)
            && !loading.get()
            && error.with(Option::is_none)
            && !has_more.get()
    };
    let bad_query = move || q.with(|q| parse_query(q).is_err());
    let masonry_style = move || {
        let columns = grid.with(|g| g.columns);
        tiles.with(|t| {
            format!(
                "--columns: {columns}; --height: {:.5}; --height-gaps: {:.5}",
                t.height.tiles, t.height.gaps
            )
        })
    };

    view! {
        <div class="page gallery">
            <div class="page-head">
                <h1 class="page-title break" class:visually-hidden=move || heading.with(String::is_empty)>
                    {move || {
                        let h = heading.get();
                        if h.is_empty() { t!("nav-gallery") } else { h }
                    }}
                </h1>
                {move || {
                    tag()
                        .map(|name| {
                            view! {
                                <a
                                    class="icon-btn tag-info"
                                    href=route::tag(&name)
                                    title=move || t!("gallery-tag-info")
                                    aria-label=move || t!("gallery-tag-info")
                                >
                                    <Ico icon=Icon::Label />
                                </a>
                            }
                        })
                }}
                <div class="page-tools">
                    <SortMenu q=q />
                    <ColorMenu q=q />
                    <ViewMenu grid=grid />
                </div>
            </div>
            {move || {
                error
                    .get()
                    .map(|message| {
                        // Trying the same search again can't mend it; writing
                        // it another way can.
                        if bad_query() {
                            view! {
                                <ErrorBanner message=message />
                                <a class="btn text help-link" href=route::HELP>
                                    <Ico icon=Icon::Help />
                                    {move || t!("gallery-syntax")}
                                </a>
                            }
                                .into_any()
                        } else {
                            view! { <ErrorBanner message=message on_retry=retry /> }.into_any()
                        }
                    })
            }}
            <Show when=empty>
                <div class="card empty">
                    <Ico icon=Icon::Gallery size=48 />
                    {move || {
                        if q.with(|q| q.trim().is_empty()) {
                            view! {
                                <h2>{move || t!("gallery-empty")}</h2>
                                <p class="muted">{move || t!("gallery-empty-body")}</p>
                                <a class="btn" href=route::UPLOAD>
                                    <Ico icon=Icon::Upload />
                                    {move || t!("nav-upload")}
                                </a>
                            }
                                .into_any()
                        } else {
                            view! {
                                <h2>{move || t!("no-matches")}</h2>
                                <p class="muted">{move || t!("gallery-try-fewer")}</p>
                                <a class="btn text" href=route::HELP>
                                    {move || t!("gallery-syntax")}
                                </a>
                            }
                                .into_any()
                        }
                    }}
                </div>
            </Show>
            <div class="masonry-box">
                <div class="masonry" node_ref=masonry style=masonry_style>
                    <For each=move || shown.get() key=|(index, id)| (*id, *index) children=tile_view />
                </div>
            </div>
            <div
                class="sentinel"
                class:done=move || !started.get() || !has_more.get() || error.with(Option::is_some)
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
        </div>
        <ToTop />
    }
}

/// The arrow back to the top, once the page is a screen down. A component of
/// its own, so the gallery isn't drawn again each time it comes or goes.
#[component]
fn ToTop() -> impl IntoView {
    let far = RwSignal::new(false);
    let check = move || {
        let (y, tall) = dom::scroll();
        let now = y > tall;
        if far.get_untracked() != now {
            far.set(now);
        }
    };
    let frame = Rc::new(Cell::new(false));
    let watch = dom::on_window("scroll", move || {
        if !frame.replace(true) {
            let frame = Rc::clone(&frame);
            request_animation_frame(move || {
                frame.set(false);
                if far.try_get_untracked().is_some() {
                    check();
                }
            });
        }
    });
    let watch = SendWrapper::new(watch);
    on_cleanup(move || watch.take().stop());
    check();
    view! {
        <button
            type="button"
            class="to-top"
            class:shown=move || far.get()
            title=move || t!("gallery-to-top")
            aria-label=move || t!("gallery-to-top")
            on:click=move |_| dom::scroll_to(0.0, true)
        >
            <Ico icon=Icon::ArrowUp />
        </button>
    }
}

#[component]
fn SortMenu(q: Memo<String>) -> impl IntoView {
    let go = search_with();
    let options = Signal::derive(|| {
        SORTS
            .iter()
            .map(|v| Choice::new(*v, sort_label(v)))
            .collect::<Vec<_>>()
    });
    let value = Signal::derive(move || current_sort(&q.get()).to_owned());
    let on_change = Callback::new(move |v: String| go.run(sorted(&q.get_untracked(), &v)));
    view! {
        <div class="sort">
            <span class="muted" aria-hidden="true">
                {move || t!("sort")}
            </span>
            <Select
                value=value
                options=options
                on_change=on_change
                label=Signal::derive(|| t!("sort-by"))
            />
        </div>
    }
}

/// Closes a menu on Escape, before a page or a sheet hears it.
fn escape_closes(open: RwSignal<bool>) -> impl Fn(KeyboardEvent) + Copy {
    move |e: KeyboardEvent| {
        if open.get_untracked() && e.key() == "Escape" {
            e.stop_propagation();
            open.set(false);
        }
    }
}

/// Rewrites the search's `color:` term from the wheel or the named swatches.
#[component]
fn ColorMenu(q: Memo<String>) -> impl IntoView {
    let open = RwSignal::new(false);
    let go = search_with();
    let pick = Callback::new(move |value: Option<String>| {
        open.set(false);
        go.run(colored(&q.get_untracked(), value.as_deref()));
    });
    let current = Memo::new(move |_| q.with(|q| current_color(q)));
    let dot = move || {
        current.with(|c| {
            c.as_ref().map(|c| {
                let style = format!("background-color: #{}", to_hex(c.rgb));
                view! { <span class="color-dot" style=style></span> }
            })
        })
    };
    let menu = move || {
        let now = current.get_untracked();
        let name = now.as_ref().map(|c| c.raw.clone());
        let swatches = NAMED_COLORS
            .into_iter()
            .map(|(swatch, rgb)| {
                let chosen = name.as_deref() == Some(swatch);
                view! {
                    <button
                        type="button"
                        class="swatch"
                        style=format!("background-color: #{}", to_hex(rgb))
                        title=color_label(swatch)
                        aria-label=color_label(swatch)
                        aria-pressed=if chosen { "true" } else { "false" }
                        on:click=move |_| pick.run(Some(swatch.to_owned()))
                    ></button>
                }
            })
            .collect_view();
        let clear = name.is_some().then(|| {
            view! {
                <button type="button" class="btn text" on:click=move |_| pick.run(None)>
                    {move || t!("color-clear")}
                </button>
            }
        });
        view! {
            <div class="select-backdrop" on:click=move |_| open.set(false)></div>
            <div class="color-menu" role="dialog" aria-label=move || t!("color-search")>
                <ColorPicker
                    value=now.as_ref().map(|c| c.rgb)
                    on_pick=Callback::new(move |rgb| pick.run(Some(to_hex(rgb))))
                />
                <div class="swatches">{swatches}</div>
                {clear}
            </div>
        }
    };
    view! {
        <div class="color-pick" on:keydown=escape_closes(open)>
            <button
                type="button"
                class="icon-btn color-toggle"
                class:on=move || current.with(Option::is_some)
                title=move || t!("color-search")
                aria-label=move || t!("color-search")
                aria-haspopup="dialog"
                aria-expanded=move || if open.get() { "true" } else { "false" }
                on:click=move |_| open.update(|o| *o = !*o)
            >
                <Ico icon=Icon::Palette />
                {dot}
            </button>
            <Show when=move || open.get()>{menu}</Show>
        </div>
    }
}

/// How the gallery's tiles look: shape, number of columns and the spaces
/// between them. The column limits follow the grid's width, so the slider only
/// offers counts that work here.
#[component]
fn ViewMenu(grid: Memo<Grid>) -> impl IntoView {
    let open = RwSignal::new(false);
    let prefs = Memo::new(|_| {
        let p = store::prefs();
        (p.gallery_view, p.gallery_columns, p.flush_tiles)
    });
    let limits = move || grid.with(|g| layout::column_limits(g.width, g.gap, g.view));
    let staggered = move || prefs.get().0.is_staggered();
    let menu = move || {
        let fixed = GalleryView::FIXED
            .into_iter()
            .map(|shape| shape_choice(prefs, shape))
            .collect_view();
        let staggered_by = move || {
            staggered().then(|| {
                let shapes = GalleryView::STAGGERED
                    .into_iter()
                    .map(|shape| shape_choice(prefs, shape))
                    .collect_view();
                view! {
                    <div class="view-row">
                        <span class="muted">{move || t!("view-staggered-by")}</span>
                        <div class="segmented" role="radiogroup" aria-label=move || t!("view-staggered-by")>
                            {shapes}
                        </div>
                    </div>
                }
            })
        };
        let columns = move || grid.with(|g| g.columns);
        let count_label = move || {
            let count = columns();
            if prefs.get().0 == GalleryView::StaggeredLandscape {
                t!("view-per-row", count = count)
            } else {
                t!("view-columns", count = count)
            }
        };
        let flush = move || prefs.get().2;
        view! {
            <div class="select-backdrop" on:click=move |_| open.set(false)></div>
            <div class="view-menu" role="dialog" aria-label=move || t!("view-title")>
                <div class="segmented" role="radiogroup" aria-label=move || t!("view-shape")>
                    {fixed}
                    <button
                        type="button"
                        role="radio"
                        class="seg"
                        class:on=staggered
                        aria-checked=move || if staggered() { "true" } else { "false" }
                        on:click=move |_| {
                            if !prefs.get_untracked().0.is_staggered() {
                                store::update_prefs(|p| p.gallery_view = GalleryView::default());
                            }
                        }
                    >
                        {move || t!("view-staggered")}
                    </button>
                </div>
                {staggered_by}
                <div class="view-row">
                    <label class="muted" for="view-columns">
                        {count_label}
                    </label>
                    <input
                        id="view-columns"
                        type="range"
                        step="1"
                        min=move || limits().0
                        max=move || limits().1
                        disabled=move || {
                            let (fewest, most) = limits();
                            fewest == most
                        }
                        prop:value=move || columns().to_string()
                        on:input=move |e| {
                            if let Ok(n) = event_target_value(&e).parse::<u8>() {
                                store::update_prefs(|p| p.gallery_columns = Some(n));
                            }
                        }
                    />
                    <button
                        type="button"
                        class="btn text small"
                        aria-pressed=move || if prefs.get().1.is_none() { "true" } else { "false" }
                        on:click=move |_| store::update_prefs(|p| p.gallery_columns = None)
                    >
                        {move || t!("view-columns-auto")}
                    </button>
                </div>
                <div class="view-switch">
                    <span>{move || t!("view-flush")}</span>
                    <button
                        type="button"
                        class="switch"
                        class:on=flush
                        role="switch"
                        aria-checked=move || if flush() { "true" } else { "false" }
                        aria-label=move || t!("view-flush")
                        on:click=move |_| store::update_prefs(|p| p.flush_tiles = !p.flush_tiles)
                    >
                        <span class="switch-knob">
                            {move || {
                                view! { <Ico icon=if flush() { Icon::Check } else { Icon::Close } size=16 /> }
                            }}
                        </span>
                    </button>
                </div>
            </div>
        }
    };
    view! {
        <div class="view-pick" on:keydown=escape_closes(open)>
            <button
                type="button"
                class="icon-btn view-toggle"
                title=move || t!("view-title")
                aria-label=move || t!("view-title")
                aria-haspopup="dialog"
                aria-expanded=move || if open.get() { "true" } else { "false" }
                on:click=move |_| open.update(|o| *o = !*o)
            >
                <Ico icon=Icon::Layout />
            </button>
            <Show when=move || open.get()>{menu}</Show>
        </div>
    }
}

/// One tile shape in the view menu, chosen when it's the gallery's.
fn shape_choice(prefs: Memo<(GalleryView, Option<u8>, bool)>, shape: GalleryView) -> impl IntoView {
    let on = move || prefs.get().0 == shape;
    view! {
        <button
            type="button"
            role="radio"
            class="seg"
            class:on=on
            aria-checked=move || if on() { "true" } else { "false" }
            on:click=move |_| store::update_prefs(|p| p.gallery_view = shape)
        >
            {move || view_label(shape)}
        </button>
    }
}
