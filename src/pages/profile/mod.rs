//! A profile on a public server, at `/u/{name}`: the account's picture, its
//! display name and handle, a badge for an administrator, when it joined,
//! what it wrote about itself and what it has done. Your own has Edit,
//! which turns the card into the form in place (`edit.rs`). A private
//! server has no profiles to show.

mod crop;
pub mod edit;

use chrono::Local;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use starbooru_core::{Profile, Role};

use self::edit::ProfileForm;
use crate::api::{self, Refused};
use crate::components::avatar::{Avatar, Size};
use crate::components::back::BackButton;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::icons::{Ico, Icon};
use crate::logic::profile::shown_name;
use crate::logic::settings::role_label;
use crate::{instance, me, store};

/// What the page shows.
#[derive(Clone, Debug, PartialEq)]
enum Shown {
    Waiting,
    Found(Profile),
    Missing,
    /// A private server keeps profiles to itself.
    Off,
    Failed(String),
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let params = use_params_map();
    let wanted = Memo::new(move |_| {
        params.with(|p| {
            p.get("name").map(|n| {
                percent_encoding::percent_decode_str(&n)
                    .decode_utf8_lossy()
                    .into_owned()
            })
        })
    });
    let shown = RwSignal::new(Shown::Waiting);
    let editing = RwSignal::new(false);
    let turn = StoredValue::new(0_u32);
    let load = move |name: String| {
        turn.update_value(|t| *t = t.wrapping_add(1));
        let at = turn.get_value();
        editing.set(false);
        // A private server keeps profiles to itself: nothing to ask.
        if untrack(|| instance::known() && !instance::public()) {
            shown.set(Shown::Off);
            return;
        }
        shown.set(Shown::Waiting);
        spawn_local(async move {
            let asked = name.clone();
            let result = api::attempt(move |c| async move { c.profile(&asked).await }).await;
            if turn.try_get_value() != Some(at) {
                return;
            }
            let now = match result {
                Ok(p) => Shown::Found(p),
                Err(Refused {
                    code: Some(code), ..
                }) if code == "profiles_off" => Shown::Off,
                Err(r) if r.missing => Shown::Missing,
                Err(r) => Shown::Failed(r.message),
            };
            shown.try_set(now);
        });
    };
    Effect::watch(
        move || wanted.get(),
        move |name, _, _| {
            if let Some(name) = name {
                load(name.clone());
            }
        },
        true,
    );
    // Your own picture changed from the account's menu or Settings.
    Effect::new(move |_| {
        let Some(mine) = me::profile() else {
            return;
        };
        shown.update(|s| {
            if let Shown::Found(p) = s
                && p.name.eq_ignore_ascii_case(&mine.name)
                && *p != mine
            {
                *p = Profile {
                    posts: p.posts,
                    forum_posts: p.forum_posts,
                    ..mine
                };
            }
        });
    });

    // Which of them, so a change to the profile shown redraws only what
    // shows it.
    let state = Memo::new(move |_| match shown.get() {
        Shown::Found(_) => 1,
        Shown::Waiting => 0,
        Shown::Missing => 2,
        Shown::Off => 3,
        Shown::Failed(_) => 4,
    });
    let found = Signal::derive(move || match shown.get() {
        Shown::Found(p) => Some(p),
        _ => None,
    });
    let body = move || match state.get() {
        0 => view! {
            <div class="center-page">
                <Spinner />
            </div>
        }
        .into_any(),
        1 => view! { <ProfileCard profile=found shown=shown editing=editing /> }.into_any(),
        2 => view! {
            <div class="card empty">
                <Ico icon=Icon::User size=48 />
                <h2 class="break">
                    {move || t!("profile-missing", name = wanted.get().unwrap_or_default())}
                </h2>
                <p class="muted">{move || t!("profile-missing-body")}</p>
            </div>
        }
        .into_any(),
        3 => view! {
            <div class="card empty">
                <Ico icon=Icon::User size=48 />
                <h2>{move || t!("profile-off")}</h2>
                <p class="muted">{move || t!("profile-off-body")}</p>
            </div>
        }
        .into_any(),
        _ => {
            let retry = Callback::new(move |()| {
                if let Some(name) = wanted.get_untracked() {
                    load(name);
                }
            });
            let message = Signal::derive(move || match shown.get() {
                Shown::Failed(m) => m,
                _ => String::new(),
            });
            view! { <ErrorBanner message=message on_retry=retry /> }.into_any()
        }
    };
    view! {
        <div class="page narrow profile-page">
            <div class="titled">
                <BackButton up="/" />
                <h1 class="page-title">{move || t!("profile-title")}</h1>
            </div>
            {body}
        </div>
    }
}

/// The profile, or your own as the form that edits it.
#[component]
fn ProfileCard(
    profile: Signal<Option<Profile>>,
    shown: RwSignal<Shown>,
    editing: RwSignal<bool>,
) -> impl IntoView {
    let read = move |f: fn(&Profile) -> String| {
        move || profile.with(|p| p.as_ref().map(f)).unwrap_or_default()
    };
    let name = Signal::derive(read(|p| p.name.clone()));
    let display =
        Signal::derive(move || profile.with(|p| p.as_ref().and_then(|p| p.display_name.clone())));
    let picture =
        Signal::derive(move || profile.with(|p| p.as_ref().and_then(|p| p.avatar_url.clone())));
    let mine = Memo::new(move |_| {
        let me = store::session().map(|s| s.user.username);
        instance::public() && name.with(|n| me.is_some_and(|me| me.eq_ignore_ascii_case(n)))
    });
    let admin =
        Memo::new(move |_| profile.with(|p| p.as_ref().is_some_and(|p| p.role == Role::Admin)));
    let shown_as = move || name.with(|n| display.with(|d| shown_name(n, d.as_deref())));
    let about = move || {
        profile.with(|p| {
            p.as_ref()
                .and_then(|p| p.about.clone())
                .filter(|a| !a.trim().is_empty())
        })
    };
    let joined = read(|p| {
        p.joined
            .with_timezone(&Local)
            .format("%Y-%m-%d")
            .to_string()
    });
    let posts = move || profile.with(|p| p.as_ref().map_or(0, |p| p.posts));
    let forum_posts = move || profile.with(|p| p.as_ref().map_or(0, |p| p.forum_posts));
    let on_saved = Callback::new(move |p: Profile| {
        shown.update(|s| {
            if let Shown::Found(old) = s {
                *old = Profile {
                    posts: old.posts,
                    forum_posts: old.forum_posts,
                    ..p
                };
            }
        });
    });
    move || {
        if editing.get() {
            let start = profile.get_untracked()?;
            Some(
                view! {
                    <section class="card profile-card editing">
                        <div class="card-head">
                            <h2 class="card-title">{move || t!("profile-edit")}</h2>
                            <button type="button" class="btn text small" on:click=move |_| editing.set(false)>
                                <Ico icon=Icon::Check size=18 />
                                {move || t!("profile-done")}
                            </button>
                        </div>
                        <ProfileForm profile=start on_saved=on_saved />
                    </section>
                }
                .into_any(),
            )
        } else {
            Some(
                view! {
                    <section class="card profile-card">
                        <div class="profile-head">
                            <Avatar name=name display=display url=picture size=Size::Huge />
                            <div class="profile-names">
                                <h2 class="profile-shown break">{shown_as}</h2>
                                <span class="muted break">"@" {move || name.get()}</span>
                                <Show when=move || admin.get()>
                                    <span class="pill role-pill">{move || role_label(Role::Admin)}</span>
                                </Show>
                            </div>
                            <Show when=move || mine.get()>
                                <button type="button" class="btn tonal profile-edit" on:click=move |_| editing.set(true)>
                                    <Ico icon=Icon::Edit />
                                    {move || t!("profile-edit")}
                                </button>
                            </Show>
                        </div>
                        {move || about().map(|text| view! { <p class="profile-about">{text}</p> })}
                        <dl class="profile-stats">
                            <div>
                                <dt>{move || t!("profile-stat-posts")}</dt>
                                <dd>{posts}</dd>
                            </div>
                            <div>
                                <dt>{move || t!("profile-stat-forum")}</dt>
                                <dd>{forum_posts}</dd>
                            </div>
                            <div>
                                <dt>{move || t!("profile-stat-joined")}</dt>
                                <dd>{joined}</dd>
                            </div>
                        </dl>
                    </section>
                }
                .into_any(),
            )
        }
    }
}
