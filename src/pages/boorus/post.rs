//! One post on another booru, at `/boorus/post?u=…&q=…&s=…`: on the viewer's
//! stage, with a way to save it here and to its own page, and its tags and
//! details beside it. Previous and next walk the grid it was opened from;
//! its tags search the same sites. A post the search brought without its
//! file or its tags' categories is asked of its site by itself.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use chrono::Local;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_query_map};
use send_wrapper::SendWrapper;
use starbooru_core::{MediaKind, RemotePost};

use super::grid::src_of;
use super::{kept_around, note_saved, queue};
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::icons::{Ico, Icon};
use crate::logic::boorus::post::{
    grouped, import_request, merged, stage_size, stage_source, wants_full,
};
use crate::logic::boorus::results::{Around, Status, status_of};
use crate::logic::boorus::{from_address, site_name};
use crate::logic::jobs::by_source;
use crate::logic::viewer::{category_class, dimensions, human_size, source_link, when};
use crate::logic::zoom::Zoom;
use crate::pages::post::glow::PostGlow;
use crate::pages::post::stage::{Shown, Stage};
use crate::pages::{bad_tag_text, category_label, rating_label};
use crate::{activity, api, route, store};

/// Posts asked of their sites are kept, a handful at most.
const MAX_LOOKED: usize = 64;

/// What a site said of a post asked for by itself.
type Answer = Result<Arc<RemotePost>, String>;

thread_local! {
    // Each answer by whose it was and the post's page.
    static LOOKED: RefCell<HashMap<(String, String), Answer>> = RefCell::new(HashMap::new());
    // The grid's search a post was opened from, for its Back.
    static OPENED_FROM: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// The grid is opening a post, from its results for `q` on the sites `s`.
pub(super) fn opening(q: &str, s: &str) {
    OPENED_FROM.with_borrow_mut(|from| *from = Some(route::boorus(q, s)));
}

fn looked(page: &str) -> Option<Answer> {
    let whose = (store::owner(), page.to_owned());
    LOOKED.with_borrow(|l| l.get(&whose).cloned())
}

fn look(page: String, answer: Answer) {
    let whose = (store::owner(), page);
    LOOKED.with_borrow_mut(|l| {
        if l.len() >= MAX_LOOKED {
            l.clear();
        }
        l.insert(whose, answer);
    });
}

/// What the stage shows of `post`.
fn shown_of(post: &RemotePost) -> Shown {
    Shown {
        key: post.page_url.clone(),
        kind: if post.is_video() {
            MediaKind::Video
        } else {
            MediaKind::Image
        },
        size: stage_size(post),
        tint: None,
        alt: post.tags.join(" "),
        poster: src_of(&post.preview_url),
        label: format!(
            "{} · {}",
            site_name(&post.site),
            t!("post-alt", id = post.id)
        ),
    }
}

#[component]
pub fn BooruPost() -> impl IntoView {
    super::explore::on_boorus(true);
    let query = use_query_map();
    // The post's page on its site, which is how it is known.
    let page = Memo::new(move |_| query.with(|m| from_address(&m.get("u").unwrap_or_default())));
    let q = Memo::new(move |_| query.with(|m| from_address(&m.get("q").unwrap_or_default())));
    let s = Memo::new(move |_| query.with(|m| m.get("s").unwrap_or_default()));
    let opened_from = StoredValue::new(OPENED_FROM.with_borrow_mut(Option::take));

    let around = Memo::new(move |_| page.with(|p| kept_around(p)));
    // Bumped when a site's answer comes, which `LOOKED` alone doesn't say.
    let answered = RwSignal::new(0_u32);
    let answer = Memo::new(move |_| {
        answered.track();
        page.with(|p| looked(p))
    });
    let post = Memo::new(move |_| {
        let kept = around.with(|a| a.as_ref().map(|a| Arc::clone(&a.found.post)));
        let answer = answer.with(|a| a.as_ref().and_then(|a| a.as_ref().ok().cloned()));
        merged(kept.as_deref(), answer.as_deref()).map(Arc::new)
    });
    let jobs = Memo::new(|_| activity::with_jobs(by_source));
    let status =
        Memo::new(move |_| post.with(|p| p.as_ref().and_then(|p| jobs.with(|j| status_of(p, j)))));
    let saving = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let zoom = RwSignal::new(Zoom::default());
    let full = RwSignal::new(false);
    let enter = RwSignal::new(None::<bool>);

    // Asks the site for the post by itself, when the search left something out.
    Effect::watch(
        move || page.get(),
        move |page, _, _| {
            error.set(None);
            zoom.set(Zoom::default());
            let known = around.get_untracked();
            let missing = wants_full(
                known.as_ref().map(|a| &*a.found.post),
                known.as_ref().and_then(|a| a.api),
            );
            if page.is_empty() || looked(page).is_some() || !missing {
                return;
            }
            let page = page.clone();
            spawn_local(async move {
                let url = page.clone();
                let result = api::call(move |c| async move { c.remote_post(&url).await }).await;
                look(page, result.map(Arc::new));
                answered.try_update(|n| *n = n.wrapping_add(1));
            });
        },
        true,
    );
    // A post saved from here stays marked in the results it came from.
    Effect::new(move |_| note_saved(&jobs.get()));

    let navigate = SendWrapper::new(use_navigate());
    let go = Callback::new(move |(to, replace): (String, bool)| {
        let options = NavigateOptions {
            replace,
            scroll: !replace,
            ..NavigateOptions::default()
        };
        (*navigate)(&to, options);
    });
    // Back to the results: through history when they're the page before, so
    // they come back as they were.
    let back = Callback::new(move |()| {
        let grid = route::boorus(&q.get_untracked(), &s.get_untracked());
        let history = web_sys::window().and_then(|w| w.history().ok());
        let through = opened_from.get_value().as_deref() == Some(grid.as_str())
            && history
                .as_ref()
                .and_then(|h| h.length().ok())
                .is_some_and(|n| n > 1);
        match history.filter(|_| through) {
            Some(history) => {
                let _ = history.back();
            }
            None => go.run((grid, false)),
        }
    });
    // Another post takes this one's place, so Back still leads to the results.
    let step = Callback::new(move |forward: bool| {
        let next = around.with_untracked(|a| {
            a.as_ref().and_then(|a| {
                if forward {
                    a.next.clone()
                } else {
                    a.prev.clone()
                }
            })
        });
        if let Some(next) = next {
            enter.set(Some(forward));
            go.run((
                route::booru_post(&next, &q.get_untracked(), &s.get_untracked()),
                true,
            ));
        }
    });
    let first =
        Signal::derive(move || around.with(|a| a.as_ref().is_none_or(|a| a.prev.is_none())));
    let last = Signal::derive(move || around.with(|a| a.as_ref().is_none_or(|a| a.next.is_none())));

    let save = move |_| {
        let Some(p) = post.get_untracked() else {
            return;
        };
        if saving.get_untracked() {
            return;
        }
        let request = match import_request(&p, "") {
            Ok(request) => request,
            Err(bad) => {
                error.set(Some(untrack(|| bad_tag_text(&bad))));
                return;
            }
        };
        saving.set(true);
        error.set(None);
        spawn_local(async move {
            if let Err(message) = queue(vec![request]).await {
                error.try_set(Some(message));
            }
            saving.try_set(false);
        });
    };

    let shown = Memo::new(move |_| post.with(|p| p.as_deref().map(shown_of)));
    let src = Signal::derive(move || {
        post.with(|p| p.as_deref().and_then(stage_source).and_then(src_of))
            .unwrap_or_default()
    });
    let thumb =
        Signal::derive(move || post.with(|p| p.as_ref().and_then(|p| src_of(&p.preview_url))));

    let bar = move || {
        let page_url = post.with(|p| p.as_ref().map(|p| p.page_url.clone()))?;
        let action = match status.get() {
            Some(Status::Saved(Some(id), _)) => view! {
                <a class="btn tonal" href=route::post(id, "", None)>
                    <Ico icon=Icon::Check />
                    {move || t!("post-number", id = id)}
                </a>
            }
            .into_any(),
            Some(Status::Saved(None, note)) => view! {
                <span class="pill ok" role="status" title=note>
                    {move || t!("boorus-known")}
                </span>
            }
            .into_any(),
            Some(Status::Busy(percent)) => {
                let label = percent.map_or_else(|| t!("upload-job-running"), |p| format!("{p}%"));
                view! {
                    <button class="btn" type="button" disabled=true>
                        <Ico icon=Icon::Download />
                        {label}
                    </button>
                }
                .into_any()
            }
            Some(Status::Failed(_)) | None => view! {
                <button class="btn" type="button" disabled=move || saving.get() on:click=save>
                    <Ico icon=Icon::Download />
                    {move || t!("save")}
                </button>
            }
            .into_any(),
        };
        Some(view! {
            {action}
            <a class="btn text" href=page_url target="_blank" rel="noopener noreferrer">
                <Ico icon=Icon::OpenInNew />
                {move || t!("boorus-open-site")}
            </a>
        })
    };
    let problem = move || {
        let failed = status.with(|s| match s {
            Some(Status::Failed(message)) => Some(message.clone()),
            _ => None,
        });
        error
            .get()
            .or(failed)
            .map(|m| view! { <ErrorBanner message=m /> })
    };

    let ready = move || {
        view! {
            <div class="page post-layout booru-post" class:full-open=move || full.get()>
                <PostGlow thumb=thumb />
                <div class="post-main">
                    <Stage
                        shown=shown
                        src=src
                        thumb=thumb
                        zoom=zoom
                        full=full
                        first=first
                        last=last
                        enter=enter
                        editing=false
                        on_step=step
                        on_back=back
                        on_edit=Callback::new(|()| {})
                    />
                    <div class="post-bar glass float" role="toolbar" aria-label=move || t!("post-actions")>
                        {bar}
                    </div>
                    {problem}
                </div>
                <aside class="side">
                    <TagsCard post=post s=s />
                    <DetailsCard post=post around=around />
                </aside>
            </div>
        }
    };
    let waiting = move || {
        let failed = answer.with(|a| a.as_ref().and_then(|a| a.as_ref().err().cloned()));
        match failed {
            Some(message) => view! {
                <div class="page narrow">
                    <ErrorBanner message=message />
                    <a class="btn tonal" href=move || route::boorus(&q.get(), &s.get())>
                        <Ico icon=Icon::ArrowBack />
                        {move || t!("post-back")}
                    </a>
                </div>
            }
            .into_any(),
            None => view! {
                <div class="center-page">
                    <Spinner />
                </div>
            }
            .into_any(),
        }
    };
    let has_post = Memo::new(move |_| post.with(Option::is_some));
    move || {
        if has_post.get() {
            ready().into_any()
        } else {
            waiting().into_any()
        }
    }
}

/// The post's tags under their categories, each searching the same sites.
#[component]
fn TagsCard(post: Memo<Option<Arc<RemotePost>>>, s: Memo<String>) -> impl IntoView {
    move || {
        let groups = post.with(|p| p.as_deref().map(grouped))?;
        if groups.is_empty() {
            return None;
        }
        let headed = groups.len() > 1;
        let sections = groups
            .into_iter()
            .map(|(category, tags)| {
                let head = headed.then(|| view! { <h3>{category_label(category)}</h3> });
                let chips = tags
                    .into_iter()
                    .map(|tag| {
                        let href = route::boorus(&tag, &s.get_untracked());
                        view! {
                            <a class=category_class(category) href=href>
                                {tag}
                            </a>
                        }
                    })
                    .collect_view();
                view! {
                    <section class="tag-group">
                        {head}
                        <div class="chips">{chips}</div>
                    </section>
                }
            })
            .collect_view();
        Some(view! {
            <section class="card">
                <h2 class="card-title">{move || t!("post-tags")}</h2>
                <div class="tag-groups">{sections}</div>
            </section>
        })
    }
}

/// Where the post is and what its site says of its file.
#[component]
fn DetailsCard(post: Memo<Option<Arc<RemotePost>>>, around: Memo<Option<Around>>) -> impl IntoView {
    let facts = move || {
        let p = post.get()?;
        let also = around.with(|a| a.as_ref().map(|a| a.found.also.clone()).unwrap_or_default());
        let also = (!also.is_empty()).then(|| {
            let links = also
                .into_iter()
                .map(|twin| {
                    view! {
                        <a href=twin.page_url target="_blank" rel="noopener noreferrer">
                            {site_name(&twin.site)}
                        </a>
                    }
                })
                .collect_view();
            view! {
                <dt>{move || t!("boorus-also")}</dt>
                <dd class="booru-also">{links}</dd>
            }
        });
        let size = p.file_size.map(|bytes| {
            let shown = human_size(i64::try_from(bytes).unwrap_or(i64::MAX));
            view! {
                <dt>{move || t!("post-size")}</dt>
                <dd>{shown}</dd>
            }
        });
        let shape = stage_size(&p).map(|(w, h)| {
            view! {
                <dt>{move || t!("post-dimensions")}</dt>
                <dd>{dimensions(Some(w), Some(h))}</dd>
            }
        });
        let score = p.score.map(|score| {
            view! {
                <dt>{move || t!("boorus-score")}</dt>
                <dd>{score}</dd>
            }
        });
        let added = p.created_at.map(|at| {
            view! {
                <dt>{move || t!("post-added")}</dt>
                <dd>{when(&at, &Local)}</dd>
            }
        });
        let source = p.source.clone().filter(|s| !s.trim().is_empty()).map(|s| {
            let shown = match source_link(&s) {
                Some(url) => view! {
                    <a href=url.to_owned() target="_blank" rel="noopener noreferrer">
                        {s.trim().to_owned()}
                    </a>
                }
                .into_any(),
                None => s.into_any(),
            };
            view! {
                <dt>{move || t!("post-source")}</dt>
                <dd class="break">{shown}</dd>
            }
        });
        let rating = p.rating;
        Some(view! {
            <dt>{move || t!("boorus-site")}</dt>
            <dd>{site_name(&p.site)}</dd>
            {also}
            <dt>{move || t!("rating")}</dt>
            <dd>{move || rating_label(rating)}</dd>
            {size}
            {shape}
            {score}
            {added}
            {source}
        })
    };
    view! {
        <section class="card">
            <h2 class="card-title">{move || t!("post-details")}</h2>
            <dl class="meta">{facts}</dl>
        </section>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_page_is_the_viewers_stage_without_its_editor() {
        let source = include_str!("post.rs");
        assert!(source.contains("<Stage"));
        assert!(source.contains("on_edit=Callback::new(|()| {})"));
        assert!(source.contains(r#"class="post-bar glass float""#));
        // A link to the site leaves no address behind.
        assert!(source.contains(r#"rel="noopener noreferrer""#));
    }
}
