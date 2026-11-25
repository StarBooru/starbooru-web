//! Administration › Server › Home page: whether the server has one, its
//! title and the line under it, the search its background shows and how
//! many pictures, with a small picture of the page as it will look. Each
//! field says what's wrong with it, before the server would; once saved,
//! the app opens on it at once (`instance::settled_home`).

use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::home::{DEFAULT_HOME_POSTS, HomeError, MAX_HOME_POSTS, MIN_HOME_POSTS};
use starbooru_core::{Post, ServerSettings};

use super::Waiting;
use super::server::save_part;
use crate::components::banner::ErrorBanner;
use crate::components::brand::Mark;
use crate::components::icons::{Ico, Icon};
use crate::components::tag_box::TagBox;
use crate::logic::admin::described;
use crate::logic::gallery::tint;
use crate::logic::home::{HomeField, check_home, field_of, home_error_text, preview_query};
use crate::logic::search::{Latest, TYPING_PAUSE_MS};
use crate::pages::home;
use crate::{api, dom, instance, route};

/// How many pictures the preview shows.
const PREVIEW_POSTS: u32 = 9;

#[component]
pub(super) fn HomeCard(settings: RwSignal<Option<ServerSettings>>) -> impl IntoView {
    let enabled = RwSignal::new(false);
    let title = RwSignal::new(String::new());
    let subtitle = RwSignal::new(String::new());
    let query = RwSignal::new(String::new());
    let posts = RwSignal::new(DEFAULT_HOME_POSTS.to_string());
    let loaded = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let saved = RwSignal::new(false);
    // The server's refusal, by the field it's about, or for the whole card.
    let refused = RwSignal::new(None::<(Option<HomeField>, String)>);
    let fill = move |s: &ServerSettings| {
        let h = s.home.clone().unwrap_or_default();
        enabled.try_set(h.enabled);
        title.try_set(h.title.unwrap_or_default());
        subtitle.try_set(h.subtitle.unwrap_or_default());
        query.try_set(h.query);
        posts.try_set(h.posts.to_string());
        loaded.try_set(true);
    };
    // The form takes the settings as they first come; other cards saving
    // later leave what's typed here alone.
    Effect::new(move |_| {
        if !loaded.get_untracked()
            && let Some(s) = settings.get()
        {
            fill(&s);
        }
    });
    let name = Memo::new(move |_| {
        settings.with(|s| s.as_ref().map(|s| s.name.clone()).unwrap_or_default())
    });
    let check = Memo::new(move |_| {
        check_home(
            enabled.get(),
            &title.read(),
            &subtitle.read(),
            &query.read(),
            &posts.read(),
        )
    });
    // What's wrong with a field: the form's own check, or the server's word.
    let problem =
        move |field: HomeField, own: fn(&crate::logic::home::HomeCheck) -> Option<HomeError>| {
            move || {
                check.with(own).map(|e| home_error_text(&e)).or_else(|| {
                    refused.with(|r| {
                        r.as_ref()
                            .filter(|(f, _)| *f == Some(field))
                            .map(|(_, m)| m.clone())
                    })
                })
            }
        };
    let title_problem = problem(HomeField::Title, |c| c.title.clone());
    let subtitle_problem = problem(HomeField::Subtitle, |c| c.subtitle.clone());
    let query_problem = problem(HomeField::Query, |c| c.query.clone());
    let posts_problem = problem(HomeField::Posts, |c| c.posts.clone());

    let save = move |e: SubmitEvent| {
        e.prevent_default();
        if busy.get_untracked() {
            return;
        }
        let Some(request) = check.get_untracked().request else {
            return;
        };
        busy.set(true);
        refused.set(None);
        saved.set(false);
        spawn_local(async move {
            let on = request.enabled;
            match save_part(settings, move |s| s.home = Some(request)).await {
                Ok(s) => {
                    fill(&s);
                    saved.try_set(true);
                    home::forget();
                    instance::settled_home(on);
                }
                Err(message) => {
                    refused.try_set(Some((field_of(&message), message)));
                }
            }
            busy.try_set(false);
        });
    };
    let invalid = |bad: bool| if bad { "true" } else { "false" };
    // Under the field and outside its label, so it isn't read as its name.
    let error_line = |id: &'static str, text: Option<String>| {
        text.map(|m| view! { <small class="field-error break" id=id>{m}</small> })
    };

    view! {
        <section class="card home-card" id="home">
            <h2 class="card-title">{move || t!("admin-home-title")}</h2>
            <Show when=move || loaded.get() fallback=|| view! { <Waiting /> }>
                <form on:submit=save on:input=move |_| saved.set(false)>
                    <button
                        type="button"
                        class="list-link switch-row"
                        role="switch"
                        aria-checked=move || if enabled.get() { "true" } else { "false" }
                        aria-labelledby="home-switch-name"
                        aria-describedby="home-switch-hint"
                        on:click=move |_| {
                            enabled.update(|on| *on = !*on);
                            saved.set(false);
                        }
                    >
                        <Ico icon=Icon::Layout />
                        <span id="home-switch-name">{move || t!("admin-home-on")}</span>
                        <span id="home-switch-hint" class="muted">
                            {move || t!("admin-home-on-hint")}
                        </span>
                        <span class="switch" class:on=move || enabled.get() aria-hidden="true">
                            <span class="switch-knob">
                                {move || {
                                    let icon = if enabled.get() { Icon::Check } else { Icon::Close };
                                    view! { <Ico icon=icon size=16 /> }
                                }}
                            </span>
                        </span>
                    </button>
                    <div class="home-form">
                        <div class="home-fields">
                            <div class="field">
                                <label for="home-title">{move || t!("admin-home-heading")}</label>
                                <input
                                    id="home-title"
                                    autocomplete="off"
                                    placeholder=move || name.get()
                                    aria-invalid=move || invalid(title_problem().is_some())
                                    aria-describedby=move || described(&[("home-title-error", title_problem().is_some())])
                                    bind:value=title
                                />
                                {move || error_line("home-title-error", title_problem())}
                            </div>
                            <div class="field">
                                <label for="home-subtitle">{move || t!("admin-home-subtitle")}</label>
                                <input
                                    id="home-subtitle"
                                    autocomplete="off"
                                    aria-invalid=move || invalid(subtitle_problem().is_some())
                                    aria-describedby=move || described(&[("home-subtitle-error", subtitle_problem().is_some())])
                                    bind:value=subtitle
                                />
                                {move || error_line("home-subtitle-error", subtitle_problem())}
                            </div>
                            <div class="field">
                                <label for="home-query">{move || t!("admin-home-query")}</label>
                                <TagBox
                                    id="home-query"
                                    text=query
                                    one_line=true
                                    floating=true
                                    invalid=Signal::derive(move || query_problem().is_some())
                                    described_by=Signal::derive(move || described(&[("home-query-error", query_problem().is_some())]))
                                    placeholder=Signal::derive(|| t!("admin-home-query-placeholder"))
                                />
                                {move || error_line("home-query-error", query_problem())}
                            </div>
                            <div class="field">
                                <label class="range-head" for="home-posts">
                                    {move || t!("admin-home-posts")}
                                    <strong class="range-value">{move || posts.get()}</strong>
                                </label>
                                <input
                                    id="home-posts"
                                    type="range"
                                    min=MIN_HOME_POSTS
                                    max=MAX_HOME_POSTS
                                    step="1"
                                    aria-invalid=move || invalid(posts_problem().is_some())
                                    aria-describedby=move || described(&[("home-posts-error", posts_problem().is_some())])
                                    bind:value=posts
                                />
                                {move || error_line("home-posts-error", posts_problem())}
                            </div>
                        </div>
                        <HomePreview
                            name=name
                            title=title
                            subtitle=subtitle
                            query=query
                            enabled=enabled
                        />
                    </div>
                    {move || {
                        refused
                            .get()
                            .filter(|(field, _)| field.is_none())
                            .map(|(_, message)| view! { <ErrorBanner message=message /> })
                    }}
                    <div class="row-end">
                        <Show when=move || saved.get()>
                            <span class="muted saved-note" role="status">
                                <Ico icon=Icon::Check size=18 />
                                {move || t!("admin-saved")}
                            </span>
                        </Show>
                        <Show when=move || saved.get() && enabled.get()>
                            <a class="btn text" href=route::HOME>
                                {move || t!("admin-home-open")}
                            </a>
                        </Show>
                        <button
                            class="btn"
                            type="submit"
                            disabled=move || busy.get() || check.with(|c| c.request.is_none())
                        >
                            {move || t!("admin-save")}
                        </button>
                    </div>
                </form>
            </Show>
        </section>
    }
}

/// The page as it will look, small: the mark, the title, the line under it
/// and the search bar over a few of the pictures the search finds now.
/// Off, it fades back, as the page won't show.
#[component]
fn HomePreview(
    name: Memo<String>,
    title: RwSignal<String>,
    subtitle: RwSignal<String>,
    query: RwSignal<String>,
    enabled: RwSignal<bool>,
) -> impl IntoView {
    // The search, once the typing pauses.
    let asked = RwSignal::new(preview_query(&query.get_untracked()));
    let turn = StoredValue::new(Latest::default());
    Effect::watch(
        move || query.get(),
        move |typed, _, _| {
            let Some(n) = turn.try_update_value(Latest::next) else {
                return;
            };
            let next = preview_query(typed);
            spawn_local(async move {
                dom::sleep(TYPING_PAUSE_MS).await;
                if turn.try_with_value(|t| t.is(n)) == Some(true)
                    && asked.try_get_untracked() != Some(next.clone())
                {
                    asked.try_set(next);
                }
            });
        },
        false,
    );
    let found = LocalResource::new(move || {
        let q = asked.get();
        async move {
            api::call(move |c| async move { c.search_posts(&q, None, 1, PREVIEW_POSTS).await })
                .await
                .map(|page| page.posts)
        }
    });
    // A search that doesn't read keeps the pictures it had.
    let last = RwSignal::new(Vec::<Post>::new());
    Effect::new(move |_| {
        if let Some(Ok(posts)) = found.get() {
            last.set(posts);
        }
    });
    let tiles = move || {
        last.get()
            .into_iter()
            .map(|p| {
                let style = tint(&p.palette)
                    .map(|c| format!("--tint: {c}"))
                    .unwrap_or_default();
                let src = p.thumb_url.as_deref().map(api::media);
                view! {
                    <span class="preview-tile" style=style>
                        {src.map(|src| view! { <img src=src alt="" loading="lazy" decoding="async" /> })}
                    </span>
                }
            })
            .collect_view()
    };
    let shown_title = move || {
        let typed = title.get();
        let typed = typed.trim();
        if typed.is_empty() {
            name.get()
        } else {
            typed.to_owned()
        }
    };
    view! {
        <figure class="home-preview" class:off=move || !enabled.get()>
            <div class="preview-tiles" aria-hidden="true">{tiles}</div>
            <div class="preview-scrim" aria-hidden="true"></div>
            <div class="preview-hero" aria-hidden="true">
                <Mark height=26 />
                <strong class="preview-title">{shown_title}</strong>
                <Show when=move || !subtitle.with(|s| s.trim().is_empty())>
                    <span class="preview-subtitle">{move || subtitle.get()}</span>
                </Show>
                <span class="preview-search">
                    <Ico icon=Icon::Search size=14 />
                </span>
            </div>
            <figcaption class="visually-hidden">{move || t!("admin-home-preview")}</figcaption>
        </figure>
    }
}
