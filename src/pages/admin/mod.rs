//! Administration, for administrators only: the server's numbers
//! (`overview.rs`), the accounts (`accounts.rs`), the database backups
//! (`backups.rs`), and the server's settings (`server.rs`) with its AI tools
//! (`ai.rs`). The tab is in the address, so a reload keeps it. Everyone else
//! gets a short note and none of the requests.

mod accounts;
mod ai;
mod backups;
mod overview;
mod server;

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use starbooru_core::Role;

use self::accounts::Accounts;
use self::ai::AiCard;
use self::backups::Backups;
use self::overview::Overview;
use self::server::{BinCard, ServerCard};
use crate::components::back::BackButton;
use crate::components::banner::Spinner;
use crate::logic::admin::Tab;
use crate::{route, store};

fn tab_label(tab: Tab) -> String {
    match tab {
        Tab::Overview => t!("admin-tab-overview"),
        Tab::Accounts => t!("admin-tab-accounts"),
        Tab::Backups => t!("admin-tab-backups"),
        Tab::Server => t!("admin-tab-server"),
    }
}

#[component]
pub fn Admin() -> impl IntoView {
    let admin = Memo::new(|_| store::session().is_some_and(|s| s.user.role == Role::Admin));
    view! {
        <div class="page narrow admin">
            <div class="titled">
                <BackButton up=route::SETTINGS />
                <h1 class="page-title break">{move || t!("admin-title")}</h1>
            </div>
            <Show
                when=move || admin.get()
                fallback=|| {
                    view! {
                        <div class="card empty">
                            <p class="muted">{move || t!("admin-only")}</p>
                        </div>
                    }
                }
            >
                <AdminTabs />
            </Show>
        </div>
    }
}

#[component]
fn AdminTabs() -> impl IntoView {
    let query = use_query_map();
    let tab = Memo::new(move |_| query.with(|q| Tab::parse(q.get("tab").as_deref())));
    let links = Tab::ALL
        .into_iter()
        .map(|t| {
            let on = move || tab.get() == t;
            view! {
                <a
                    class="seg"
                    class:on=on
                    href=t.href()
                    aria-current=move || on().then_some("page")
                >
                    {move || tab_label(t)}
                </a>
            }
        })
        .collect_view();
    view! {
        <nav class="segmented admin-tabs" aria-label=move || t!("admin-title")>
            {links}
        </nav>
        {move || match tab.get() {
            Tab::Overview => view! { <Overview /> }.into_any(),
            Tab::Accounts => view! { <Accounts /> }.into_any(),
            Tab::Backups => view! { <Backups /> }.into_any(),
            Tab::Server => {
                view! {
                    <ServerCard />
                    <BinCard />
                    <AiCard />
                }
                    .into_any()
            }
        }}
    }
}

/// A section's spinner, while its first answer is on the way.
#[component]
fn Waiting() -> impl IntoView {
    view! {
        <div class="admin-wait">
            <Spinner />
        </div>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_admin_styles_keep_to_the_tokens_and_the_motion_settings() {
        let css = include_str!("../../../style/pages/admin.css");
        // Glass belongs to the bars, the viewer and the sign-in card.
        assert!(!css.contains("backdrop-filter"));
        // The bars grow from the bottom; reduced motion and the battery saver
        // cut that short with the rules in base.css.
        assert!(css.contains("@keyframes grow"));
        assert!(css.contains("@media (prefers-reduced-motion: reduce)"));
        let page = include_str!("../../../index.html");
        assert!(page.contains(r#"<link data-trunk rel="css" href="style/pages/admin.css">"#));
    }

    #[test]
    fn only_administrators_get_the_tabs() {
        let page = include_str!("mod.rs");
        let at = |s: &str| page.find(s).unwrap_or_else(|| panic!("{s}"));
        // The tabs, and with them every admin request, sit inside the guard.
        assert!(at("when=move || admin.get()") < at("<AdminTabs />"));
        assert!(at("t!(\"admin-only\")") < at("<AdminTabs />"));
    }
}
