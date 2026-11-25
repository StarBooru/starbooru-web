//! Settings: a list of groups, and the group open. A wide window shows both,
//! the list beside the group; a narrower one shows the list at `/settings`
//! and each group as a page of its own at `/settings/<group>`, with a Back.
//! The groups (`logic::settings::Group`): the account, its ways in, its
//! password and devices (`account.rs`, `sessions.rs`); how the app looks
//! (`appearance.rs`); the blacklist, thumbnails, the viewer and gestures
//! (`viewing.rs`, `blacklist.rs`);
//! the battery saver and effects (`battery.rs`); the library's tools; the
//! shared links and the API keys (`keys.rs`); help and what this app is.
//! Administrators have a way to the administration at the foot of the list.

mod account;
mod appearance;
mod battery;
mod blacklist;
mod controls;
mod keys;
mod sessions;
mod viewing;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_location, use_navigate, use_params_map};
use starbooru_core::{ServerInfo, SignInMethods};
use wasm_bindgen::JsCast;

use self::account::{AccountCard, PasswordCard, SignInCard};
use self::appearance::AppearanceGroup;
use self::battery::BatteryGroup;
use self::keys::ApiKeysCard;
use self::sessions::SessionsCard;
use self::viewing::ViewingGroup;
use super::library::LibraryImportCard;
use super::profile::edit::ProfileSettings;
use super::share::SharesCard;
use crate::components::avatar::Avatar;
use crate::components::back::BackButton;
use crate::components::icons::{Ico, Icon};
use crate::components::invite::SignInCard as SignInInvite;
use crate::components::legal::note_opener;
use crate::logic::profile::shown_name;
use crate::logic::settings::{
    FIRST, FocusTo, Group, Sections, anchored, focus_after, group_for_anchor, sections, version,
};
use crate::{api, dom, install, instance, me, route, store};

/// Where this app's code lives, and the list of everyone whose work it uses.
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const CREDITS: &str = concat!(env!("CARGO_PKG_REPOSITORY"), "/blob/main/ATTRIBUTION.md");

/// The id of the open group's title, which takes the focus when it opens.
const TITLE_ID: &str = "settings-group-title";

/// What the server says of this account, read once for every group.
#[derive(Clone, Copy)]
struct Ctx {
    info: LocalResource<Option<ServerInfo>>,
    /// Whether the account has a password, and the provider's accounts linked.
    methods: RwSignal<Option<SignInMethods>>,
    /// A new password signs the other devices out: their list is read again.
    sessions_version: RwSignal<u32>,
    shown: Memo<Option<Sections>>,
}

impl Ctx {
    fn shows(self, pick: fn(Sections) -> bool) -> bool {
        self.shown.get().is_some_and(pick)
    }
}

fn group_name(group: Group) -> String {
    match group {
        Group::Account => t!("settings-group-account"),
        Group::Appearance => t!("settings-appearance"),
        Group::Viewing => t!("settings-group-viewing"),
        Group::Battery => t!("settings-group-battery"),
        Group::Library => t!("settings-library"),
        Group::Sharing => t!("settings-group-sharing"),
        Group::About => t!("settings-group-about"),
    }
}

fn group_note(group: Group) -> String {
    match group {
        // The account's row shows the username, and the group's name under it.
        Group::Account => t!("settings-group-account"),
        Group::Appearance => t!("settings-group-appearance-note"),
        Group::Viewing if instance::guest() => t!("blacklist-title"),
        Group::Viewing => t!("settings-group-viewing-note"),
        Group::Battery => t!("settings-group-battery-note"),
        Group::Library => t!("settings-group-library-note"),
        Group::Sharing => t!("settings-group-sharing-note"),
        Group::About => t!("settings-group-about-note"),
    }
}

fn group_icon(group: Group) -> Icon {
    match group {
        Group::Account => Icon::User,
        Group::Appearance => Icon::Palette,
        Group::Viewing => Icon::Eye,
        Group::Battery => Icon::Battery,
        Group::Library => Icon::Folder,
        Group::Sharing => Icon::Share,
        Group::About => Icon::Info,
    }
}

/// Scrolls to the card `id`, and keeps it there for about a second while
/// the cards above it come in with the server's answers. `at` is where the
/// page was put last; once the user scrolls away from it, it stops.
fn reveal(id: String, tries: u32, at: Option<f64>) {
    let (y, _) = dom::scroll();
    if at.is_some_and(|at| (at - y).abs() > 2.0) {
        return;
    }
    let found = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(&id));
    let at = match found {
        Some(el) => {
            let options = web_sys::ScrollIntoViewOptions::new();
            options.set_block(web_sys::ScrollLogicalPosition::Start);
            options.set_behavior(web_sys::ScrollBehavior::Instant);
            el.scroll_into_view_with_scroll_into_view_options(&options);
            Some(dom::scroll().0)
        }
        None => None,
    };
    if tries > 0 {
        request_animation_frame(move || reveal(id, tries - 1, at));
    }
}

/// Moves the focus to the element `selector` finds, once it's drawn.
fn focus_on(selector: String) {
    request_animation_frame(move || {
        let el = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.query_selector(&selector).ok().flatten())
            .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok());
        if let Some(el) = el {
            let _ = el.focus();
        }
    });
}

#[component]
pub fn Settings() -> impl IntoView {
    let params = use_params_map();
    let location = use_location();
    let navigate = use_navigate();
    let asked = Memo::new(move |_| params.with(|p| p.get("group")));
    // The group in the address; `None` is the list alone.
    let group = Memo::new(move |_| asked.get().as_deref().and_then(Group::parse));
    // What a wide window shows beside the list.
    let open = Memo::new(move |_| group.get().unwrap_or(FIRST));

    // An address from before the groups, `/settings#keys`, or a card's
    // name in place of a group's, goes to the group with that card.
    Effect::new(move |_| {
        let hash = location.hash.get();
        let to = match asked.get() {
            Some(name) if Group::parse(&name).is_none() => Some(
                group_for_anchor(&name)
                    .map_or_else(|| route::SETTINGS.to_owned(), |(g, card)| anchored(g, card)),
            ),
            None => group_for_anchor(&hash).map(|(g, card)| anchored(g, card)),
            Some(_) => None,
        };
        if let Some(to) = to {
            let options = NavigateOptions {
                replace: true,
                ..NavigateOptions::default()
            };
            navigate(&to, options);
        }
    });
    // Down to the card the address names.
    Effect::new(move |_| {
        open.track();
        let id = location.hash.get().trim_start_matches('#').to_owned();
        if !id.is_empty() && group.get_untracked().is_some() {
            reveal(id, 60, None);
        }
    });
    // The focus goes with the page a phone shows: to the group's title when
    // one opens, and back to its row in the list after Back.
    Effect::new(move |before: Option<Option<Group>>| {
        let now = group.get();
        let to = before.and_then(|before| focus_after(before, now));
        match to {
            Some(FocusTo::Title) if location.hash.get_untracked().is_empty() => {
                focus_on(format!("#{TITLE_ID}"));
            }
            Some(FocusTo::Link(left)) => {
                focus_on(format!(r#".settings-link[data-group="{}"]"#, left.slug()));
            }
            _ => {}
        }
        now
    });

    let guest = untrack(instance::guest);
    let info = LocalResource::new(api::server_info);
    let methods = RwSignal::new(None::<SignInMethods>);
    if !guest {
        spawn_local(async move {
            if let Ok(found) = api::call(|c| async move { c.sign_in_methods().await }).await {
                methods.try_set(Some(found));
            }
        });
    }
    let shown = Memo::new(move |_| {
        let user = store::session()?.user;
        let info = info.get().flatten();
        // A switch of mode shows here without waiting for the page again.
        let info = instance::info().or(info);
        Some(methods.with(|m| sections(&user, info.as_ref(), m.as_ref())))
    });
    let ctx = Ctx {
        info,
        methods,
        sessions_version: RwSignal::new(0_u32),
        shown,
    };

    let rows = Group::shown(guest)
        .into_iter()
        .map(|g| view! { <GroupLink group=g current=group open=open /> })
        .collect_view();
    let admin = move || {
        ctx.shows(|s| s.admin).then(|| {
            view! {
                <li class="admin-row">
                    <a class="settings-link" href=route::ADMIN>
                        <span class="group-icon">
                            <Ico icon=Icon::Shield />
                        </span>
                        <span class="settings-link-text">
                            <span>{move || t!("admin-title")}</span>
                            <span class="muted">{move || t!("settings-admin-note")}</span>
                        </span>
                        <Ico icon=Icon::ChevronRight />
                    </a>
                </li>
            }
        })
    };

    view! {
        <div class="page settings" class:open=move || group.get().is_some()>
            <nav class="settings-nav" aria-labelledby="settings-title">
                <h1 id="settings-title" class="page-title break">{move || t!("nav-settings")}</h1>
                <ul class="settings-list">
                    {rows}
                    {admin}
                </ul>
            </nav>
            {move || {
                let group = open.get();
                if guest && !group.for_guests() {
                    view! { <GuestGroup /> }.into_any()
                } else {
                    view! { <GroupPage group=group ctx=ctx /> }.into_any()
                }
            }}
        </div>
    }
}

/// A group's row in the list: its icon, its name and what's in it. The
/// account's shows who is signed in, with their initial.
#[component]
fn GroupLink(group: Group, current: Memo<Option<Group>>, open: Memo<Group>) -> impl IntoView {
    let account = move || store::session().map(|s| s.user.username);
    // A public server's profile has a picture and a name to show.
    let display = Signal::derive(|| me::profile().and_then(|p| p.display_name));
    let picture = Signal::derive(|| me::profile().and_then(|p| p.avatar_small_url));
    let icon = move || match (group, account()) {
        (Group::Account, Some(name)) => {
            view! { <Avatar name=name display=display url=picture /> }.into_any()
        }
        _ => view! {
            <span class="group-icon">
                <Ico icon=group_icon(group) />
            </span>
        }
        .into_any(),
    };
    let name = move || match (group, account()) {
        (Group::Account, Some(name)) => shown_name(&name, display.get().as_deref()),
        (Group::Account, None) => t!("guest-sign-in"),
        _ => group_name(group),
    };
    let note = move || match (group, account()) {
        (Group::Account, None) => t!("guest-account-note"),
        _ => group_note(group),
    };
    view! {
        <li>
            <a
                class="settings-link"
                class:shown=move || open.get() == group
                href=group.href()
                data-group=group.slug()
                aria-current=move || (current.get() == Some(group)).then_some("page")
            >
                {icon}
                <span class="settings-link-text">
                    <span class="break">{name}</span>
                    <span class="muted">{note}</span>
                </span>
                <Ico icon=Icon::ChevronRight />
            </a>
        </li>
    }
}

/// The group open, under its title, with the way back to the list where
/// the list isn't beside it.
#[component]
fn GroupPage(group: Group, ctx: Ctx) -> impl IntoView {
    let body = match group {
        Group::Account => view! { <AccountGroup ctx=ctx /> }.into_any(),
        Group::Appearance => view! { <AppearanceGroup /> }.into_any(),
        Group::Viewing => view! { <ViewingGroup /> }.into_any(),
        Group::Battery => view! { <BatteryGroup /> }.into_any(),
        Group::Library => view! { <LibraryGroup ctx=ctx /> }.into_any(),
        Group::Sharing => view! { <SharingGroup ctx=ctx /> }.into_any(),
        Group::About => view! { <AboutGroup /> }.into_any(),
    };
    view! {
        <section class="settings-group" data-group=group.slug() aria-labelledby=TITLE_ID>
            <div class="titled group-head">
                <BackButton up=route::SETTINGS />
                <span class="group-icon" aria-hidden="true">
                    <Ico icon=group_icon(group) />
                </span>
                <h2 id=TITLE_ID class="page-title" tabindex="-1">
                    {move || group_name(group)}
                </h2>
            </div>
            <div class="group-cards">{body}</div>
        </section>
    }
}

/// A group that needs an account, opened by a guest from an address.
#[component]
fn GuestGroup() -> impl IntoView {
    view! {
        <section class="settings-group" aria-labelledby=TITLE_ID>
            <div class="titled group-head">
                <BackButton up=route::SETTINGS />
                <h2 id=TITLE_ID class="page-title" tabindex="-1">
                    {move || t!("guest-sign-in")}
                </h2>
            </div>
            <div class="group-cards">
                <SignInInvite why=Signal::derive(|| t!("guest-page-needs-account")) />
            </div>
        </section>
    }
}

/// The account, its ways in, its password and the devices signed in. A
/// guest gets the way to one instead.
#[component]
fn AccountGroup(ctx: Ctx) -> impl IntoView {
    let provider = Signal::derive(move || ctx.info.get().flatten().and_then(|i| i.oidc));
    let sign_in = move || {
        ctx.shows(|s| s.sign_in)
            .then(|| view! { <SignInCard methods=ctx.methods provider=provider /> })
    };
    // Profiles are a public server's.
    let profile = move || instance::public().then(|| view! { <ProfileSettings /> });
    move || {
        if instance::guest() {
            return view! { <SignInInvite why=Signal::derive(|| t!("guest-account-why")) /> }
                .into_any();
        }
        store::signed_in()
            .then(|| {
                view! {
                    <AccountCard info=ctx.info />
                    {profile}
                    {sign_in}
                    <PasswordCard
                        methods=ctx.methods
                        on_changed=Callback::new(move |()| ctx.sessions_version.update(|v| *v += 1))
                    />
                    <SessionsCard version=ctx.sessions_version />
                }
            })
            .into_any()
    }
}

/// The recycle bin and the duplicate finder, then the import of a whole
/// library, unless the server never reaches out.
#[component]
fn LibraryGroup(ctx: Ctx) -> impl IntoView {
    let library_import = move || {
        ctx.shows(|s| s.library_import)
            .then(|| view! { <LibraryImportCard /> })
    };
    view! {
        <section class="card" id="library">
            <a class="list-link" href=route::TRASH>
                <Ico icon=Icon::Delete />
                <span>{move || t!("trash-title")}</span>
                <span class="muted">{move || t!("settings-trash-note")}</span>
            </a>
            <a class="list-link" href=route::DUPLICATES>
                <Ico icon=Icon::Copy />
                <span>{move || t!("dup-title")}</span>
                <span class="muted">{move || t!("settings-dup-note")}</span>
            </a>
        </section>
        {library_import}
    }
}

/// The links shared, then the API keys. A public server shares a post by
/// its address, and has no links to list.
#[component]
fn SharingGroup(ctx: Ctx) -> impl IntoView {
    let shares = move || ctx.shows(|s| s.shares).then(|| view! { <SharesCard /> });
    move || {
        store::signed_in().then(|| {
            view! {
                {shares}
                <ApiKeysCard />
            }
        })
    }
}

/// The cheat sheet and the legal pages, then what this app is.
#[component]
fn AboutGroup() -> impl IntoView {
    view! {
        <section class="card" id="help">
            <h3 class="card-title">{move || t!("settings-help-legal")}</h3>
            <a class="list-link" href=route::HELP>
                <Ico icon=Icon::Help />
                <span>{move || t!("help-title")}</span>
                <span class="muted">{move || t!("settings-help-note")}</span>
            </a>
            <a class="list-link" href=route::PRIVACY on:click=move |e| note_opener(&e)>
                <Ico icon=Icon::Shield />
                <span>{move || t!("privacy-title")}</span>
                <span class="muted">{move || t!("settings-privacy-note")}</span>
            </a>
            <a class="list-link" href=route::TERMS on:click=move |e| note_opener(&e)>
                <Ico icon=Icon::Article />
                <span>{move || t!("terms-title")}</span>
                <span class="muted">{move || t!("settings-terms-note")}</span>
            </a>
        </section>
        <AboutCard />
    }
}

/// Who made StarBooru, with what, where its code lives and which version this
/// is, and installing it while the browser offers that.
#[component]
fn AboutCard() -> impl IntoView {
    let install_row = move || {
        install::offered().then(|| {
            view! {
                <button
                    type="button"
                    class="list-link install-row"
                    disabled=install::asking
                    on:click=|_| install::ask()
                >
                    <Ico icon=Icon::Install />
                    <span>{move || t!("install-app")}</span>
                    <span class="muted">{move || t!("install-note")}</span>
                </button>
            }
        })
    };
    view! {
        <section class="card" id="about">
            <h3 class="card-title">{move || t!("about-title")}</h3>
            {install_row}
            <dl class="meta about-credits">
                <dt>{move || t!("about-made-by")}</dt>
                <dd>{move || t!("about-made-by-name")}</dd>
                <dt>{move || t!("about-icons")}</dt>
                <dd>{move || t!("about-icons-names")}</dd>
                <dt>{move || t!("about-lettering")}</dt>
                <dd>{move || t!("about-lettering-name")}</dd>
                <dt>{move || t!("about-version")}</dt>
                <dd class="about-version">{version()}</dd>
            </dl>
            <a class="list-link" href=REPOSITORY target="_blank" rel="noopener noreferrer">
                <Ico icon=Icon::GitHub />
                <span>{move || t!("about-github")}</span>
                <span class="muted break">{REPOSITORY.trim_start_matches("https://")}</span>
            </a>
            <a class="list-link" href=CREDITS target="_blank" rel="noopener noreferrer">
                <Ico icon=Icon::Star />
                <span>{move || t!("about-credits")}</span>
                <span class="muted">{move || t!("about-credits-note")}</span>
            </a>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_about_card_links_to_this_project() {
        assert_eq!(REPOSITORY, "https://github.com/StarBooru/starbooru-web");
        // The credits link is only right while the file stays where it is.
        let attribution = concat!(env!("CARGO_MANIFEST_DIR"), "/ATTRIBUTION.md");
        assert!(std::path::Path::new(attribution).exists());
    }

    #[test]
    fn every_group_has_its_words_and_its_own_icon() {
        let icons: std::collections::BTreeSet<String> = Group::ALL
            .into_iter()
            .map(|g| format!("{:?}", group_icon(g)))
            .collect();
        assert_eq!(icons.len(), Group::ALL.len());
        for g in Group::ALL {
            assert!(!group_name(g).is_empty() && !group_note(g).is_empty());
        }
        assert_eq!(group_name(Group::Viewing), "Viewing and gestures");
    }

    #[test]
    fn every_card_stands_in_its_group() {
        let page = include_str!("mod.rs");
        let at = |s: &str| page.find(s).unwrap_or_else(|| panic!("{s}"));
        // In the order of the list, and within a group, in the reference's.
        let order = [
            "fn AccountGroup",
            "<AccountCard info=ctx.info />",
            "{profile}",
            "{sign_in}",
            "<PasswordCard",
            "<SessionsCard",
            "fn LibraryGroup",
            "href=route::TRASH",
            "href=route::DUPLICATES",
            "{library_import}",
            "fn SharingGroup",
            "<SharesCard />",
            "<ApiKeysCard />",
            "fn AboutGroup",
            "href=route::HELP",
            "href=route::PRIVACY",
            "href=route::TERMS",
            "<AboutCard />",
        ];
        assert!(order.windows(2).all(|w| at(w[0]) < at(w[1])), "{order:?}");
        // Every group's page, from the list's order.
        for (g, view) in [
            (Group::Account, "<AccountGroup ctx=ctx />"),
            (Group::Appearance, "<AppearanceGroup />"),
            (Group::Viewing, "<ViewingGroup />"),
            (Group::Battery, "<BatteryGroup />"),
            (Group::Library, "<LibraryGroup ctx=ctx />"),
            (Group::Sharing, "<SharingGroup ctx=ctx />"),
            (Group::About, "<AboutGroup />"),
        ] {
            assert!(
                page.contains(&format!("Group::{g:?} => view! {{ {view} }}")),
                "{g:?}"
            );
        }
    }

    #[test]
    fn a_phone_shows_the_list_or_a_group_and_a_wide_window_both() {
        let css = include_str!("../../../style/pages/settings.css");
        assert!(css.contains(".settings.open .settings-nav { display: none; }"));
        assert!(css.contains(".settings:not(.open) .settings-group { display: none; }"));
        assert!(css.contains("@media (min-width: 1000px)"));
        // The way back to the list only where the list isn't beside it.
        assert!(css.contains(".group-head .page-back { display: none; }"));
    }

    #[test]
    fn the_settings_styles_keep_to_the_tokens() {
        let css = include_str!("../../../style/pages/settings.css");
        assert!(!css.contains("backdrop-filter"));
        // The presets show their own hue from the tokens, with a ring that
        // is a shadow, so hovering moves nothing.
        assert!(
            css.contains("background: oklch(var(--swatch-l) var(--swatch-c) var(--swatch-hue));")
        );
        assert!(css.contains(".accent.on { box-shadow:"));
        assert!(css.contains(".key-prefix {"));
        assert!(css.contains(".meter.full span { background: var(--danger); }"));
        // What moves is still with reduced effects; reduced motion and the
        // battery saver cut every animation short (style/base.css).
        assert!(css.contains(r#".app[data-effects="lite"] :is(.settings-group"#));
    }
}
