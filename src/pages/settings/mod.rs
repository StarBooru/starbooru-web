//! Settings, card by card as the reference had them: how the app looks and
//! behaves on this device (`appearance.rs`), the library's tools and the
//! People switch, the administration for administrators, help and the legal pages, then the
//! account, its ways in and its password (`account.rs`), the devices signed
//! in (`sessions.rs`), the shared links, the API keys (`keys.rs`), and what
//! this app is.

mod account;
mod appearance;
mod keys;
mod sessions;

use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::SignInMethods;

use self::account::{AccountCard, PasswordCard, SignInCard};
use self::appearance::{AppearanceCard, GesturesCard, PeopleSwitch};
use self::keys::ApiKeysCard;
use self::sessions::SessionsCard;
use super::library::LibraryImportCard;
use super::share::SharesCard;
use crate::components::icons::{Ico, Icon};
use crate::components::legal::note_opener;
use crate::logic::settings::{Sections, sections, version};
use crate::route;
use crate::store;
use crate::{api, install};

/// Where this app's code lives, and the list of everyone whose work it uses.
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const CREDITS: &str = concat!(env!("CARGO_PKG_REPOSITORY"), "/blob/main/ATTRIBUTION.md");

#[component]
pub fn Settings() -> impl IntoView {
    let info = LocalResource::new(api::server_info);
    // Whether the account has a password, and the provider's accounts linked.
    let methods = RwSignal::new(None::<SignInMethods>);
    spawn_local(async move {
        if let Ok(found) = api::call(|c| async move { c.sign_in_methods().await }).await {
            methods.try_set(Some(found));
        }
    });
    // A new password signs the other devices out: their list is read again.
    let sessions_version = RwSignal::new(0_u32);
    let shown = Memo::new(move |_| {
        let user = store::session()?.user;
        let info = info.get().flatten();
        Some(methods.with(|m| sections(&user, info.as_ref(), m.as_ref())))
    });
    let shows = move |pick: fn(Sections) -> bool| shown.get().is_some_and(pick);
    let provider = Signal::derive(move || info.get().flatten().and_then(|i| i.oidc));

    let library_import =
        move || shows(|s| s.library_import).then(|| view! { <LibraryImportCard /> });
    let admin = move || {
        shows(|s| s.admin).then(|| {
            view! {
                <section class="card">
                    <h2 class="card-title">{move || t!("admin-server-title")}</h2>
                    <a class="list-link" href=route::ADMIN>
                        <Ico icon=Icon::Shield />
                        <span>{move || t!("admin-title")}</span>
                        <span class="muted">{move || t!("settings-admin-note")}</span>
                    </a>
                </section>
            }
        })
    };
    let sign_in = move || {
        shows(|s| s.sign_in).then(|| view! { <SignInCard methods=methods provider=provider /> })
    };
    // The account's own cards. The shell shows the sign-in screen to no one
    // else, but a session can end while the page is open.
    let account = move || {
        store::signed_in().then(|| {
            view! {
                <AccountCard info=info />
                {sign_in}
                <PasswordCard
                    methods=methods
                    on_changed=Callback::new(move |()| sessions_version.update(|v| *v += 1))
                />
                <SessionsCard version=sessions_version />
                <SharesCard />
                <ApiKeysCard />
            }
        })
    };

    view! {
        <div class="page narrow settings">
            <h1 class="page-title break">{move || t!("nav-settings")}</h1>
            <AppearanceCard />
            <GesturesCard />
            <section class="card">
                <h2 class="card-title">{move || t!("settings-library")}</h2>
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
                <PeopleSwitch />
            </section>
            {library_import}
            {admin}
            <section class="card">
                <h2 class="card-title">{move || t!("settings-help-legal")}</h2>
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
            {account}
            <AboutCard />
        </div>
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
        <section class="card">
            <h2 class="card-title">{move || t!("about-title")}</h2>
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
    fn the_cards_stand_in_the_references_order() {
        let page = include_str!("mod.rs");
        let at = |s: &str| page.find(s).unwrap_or_else(|| panic!("{s}"));
        let order = [
            "<AppearanceCard />",
            "<GesturesCard />",
            "href=route::TRASH",
            "href=route::DUPLICATES",
            "<PeopleSwitch />",
            "{library_import}",
            "{admin}",
            "href=route::HELP",
            "href=route::PRIVACY",
            "href=route::TERMS",
            "{account}",
            "<AboutCard />",
        ];
        assert!(order.windows(2).all(|w| at(w[0]) < at(w[1])), "{order:?}");
        let account = [
            "<AccountCard info=info />",
            "{sign_in}",
            "<PasswordCard",
            "<SessionsCard",
            "<SharesCard />",
            "<ApiKeysCard />",
        ];
        assert!(
            account.windows(2).all(|w| at(w[0]) < at(w[1])),
            "{account:?}"
        );
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
    }
}
