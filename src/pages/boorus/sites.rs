//! The sites a search goes to: a row of chips under the title, one per site,
//! and the sheet with an account for each and the sites added by address.

use leptos::ev::SubmitEvent;
use leptos::html;
use leptos::prelude::*;
use starbooru_core::remote::clean_base;
use starbooru_core::{BooruAccount, BooruApi, MAX_SEARCH_SITES, RemoteSite};

use super::accounts::{SiteRow, forget_account};
use wasm_bindgen::JsCast;

use crate::components::focus::hold_focus;
use crate::components::icons::{Ico, Icon};
use crate::components::select::{Choice, Select};
use crate::dom;
use crate::logic::boorus::{
    Site, all_sites, api_of, host_of, preset_of, remember, remote_error_text, site_key, tidy,
    toggled,
};
use crate::store::{self, SavedBooru};

/// Every site with a chip, for the sites chosen now.
fn chips_for(sites: &[RemoteSite]) -> Vec<Site> {
    all_sites(&store::prefs().boorus, sites)
}

/// On a phone the chips are one row to scroll: the first site searched is
/// brought into it.
fn show_chosen(row: &web_sys::Element) {
    let Ok(Some(chip)) = row.query_selector(".chip.on") else {
        return;
    };
    if row.scroll_width() <= row.client_width() {
        return;
    }
    let (chip, all) = (
        chip.get_bounding_client_rect(),
        row.get_bounding_client_rect(),
    );
    if chip.left() < all.left() || chip.right() > all.right() - 44.0 {
        row.set_scroll_left(row.scroll_left() + (chip.left() - all.left() - 8.0) as i32);
    }
}

/// The row under the title: a click on a chip searches its site too, or no
/// longer. A site that failed says why on its chip, with a way to try again.
#[component]
pub(super) fn SiteChips(
    sites: Memo<Vec<RemoteSite>>,
    failed: Memo<Vec<(RemoteSite, String)>>,
    loading: RwSignal<bool>,
    on_go: Callback<Vec<RemoteSite>>,
    on_again: Callback<()>,
    on_sheet: Callback<()>,
) -> impl IntoView {
    let chips = Memo::new(move |_| sites.with(|s| chips_for(s)));
    let row: NodeRef<html::Div> = NodeRef::new();
    Effect::watch(
        move || sites.get(),
        move |_, _, _| {
            // The page may be gone by the next frame.
            request_animation_frame(move || {
                if let Some(row) = row.try_get_untracked().flatten() {
                    show_chosen(&row);
                }
            });
        },
        true,
    );
    let chip = move |chip: Site| {
        let site = chip.site.clone();
        let on = {
            let site = site.clone();
            move || sites.with(|s| s.contains(&site))
        };
        let error = {
            let site = site.clone();
            move || {
                failed.with(|f| {
                    f.iter()
                        .find(|(failed, _)| *failed == site)
                        .map(|(_, e)| e.clone())
                })
            }
        };
        let warn = {
            let error = error.clone();
            move || error().is_some()
        };
        let pressed = {
            let on = on.clone();
            move || if on() { "true" } else { "false" }
        };
        view! {
            <button
                type="button"
                class="chip"
                class:on=on
                class:warn=warn
                aria-pressed=pressed
                title=error
                on:click=move |_| {
                    let next = sites.with_untracked(|s| chips.with_untracked(|c| toggled(s, &site, c)));
                    on_go.run(next);
                }
            >
                {chip.name}
            </button>
        }
    };
    view! {
        <div class="booru-sites" role="group" aria-label=move || t!("boorus-sites") node_ref=row>
            <For each=move || chips.get() key=|c| site_key(&c.site) children=chip />
            <Show when=move || failed.with(|f| !f.is_empty()) && !loading.get()>
                <button
                    type="button"
                    class="icon-btn small"
                    title=move || t!("retry")
                    aria-label=move || t!("retry")
                    on:click=move |_| on_again.run(())
                >
                    <Ico icon=Icon::Restore size=20 />
                </button>
            </Show>
            <button
                type="button"
                class="icon-btn small sites-edit"
                title=move || t!("boorus-sites")
                aria-label=move || t!("boorus-sites")
                aria-haspopup="dialog"
                on:click=move |_| on_sheet.run(())
            >
                <Ico icon=Icon::Tune size=20 />
            </button>
        </div>
    }
}

/// The sites, over the page: an account for each, saved on the server, and
/// the ones added by their address.
#[component]
pub(super) fn SitesSheet(
    sites: Memo<Vec<RemoteSite>>,
    accounts: RwSignal<Vec<BooruAccount>>,
    on_go: Callback<Vec<RemoteSite>>,
    on_close: Callback<()>,
) -> impl IntoView {
    let chips = Memo::new(move |_| sites.with(|s| chips_for(s)));
    let error = RwSignal::new(None::<String>);
    // The address of the site whose account is being typed.
    let editing = RwSignal::new(None::<String>);
    let address = RwSignal::new(String::new());
    let kind = RwSignal::new(BooruApi::Danbooru);
    let shut: NodeRef<html::Button> = NodeRef::new();
    let dialog: NodeRef<html::Div> = NodeRef::new();
    Effect::new(move |_| {
        if let Some(el) = shut.get() {
            let _ = el.focus();
        }
    });
    // Tab stays in the sheet while it's open, and goes back after.
    hold_focus(dialog, true);
    // Escape closes it wherever the focus is, unless an open list has it.
    let keys = web_sys::window().map(|window| {
        dom::listen(&window, "keydown", false, move |e| {
            let escape = e
                .dyn_ref::<web_sys::KeyboardEvent>()
                .is_some_and(|k| k.key() == "Escape");
            if escape && !e.default_prevented() {
                e.prevent_default();
                on_close.try_run(());
            }
        })
    });
    on_cleanup(move || {
        // It closes from inside this very listener, which can't be dropped
        // while it runs.
        set_timeout(
            move || {
                if let Some(keys) = keys {
                    keys.stop();
                }
            },
            std::time::Duration::ZERO,
        );
    });

    let add = move |e: SubmitEvent| {
        e.prevent_default();
        let site = match clean_base(&address.get_untracked()) {
            Ok(base) => RemoteSite {
                api: kind.get_untracked(),
                base,
            },
            Err(e) => {
                error.set(Some(untrack(|| remote_error_text(&e))));
                return;
            }
        };
        error.set(None);
        address.set(String::new());
        if preset_of(&site).is_none() {
            let added = SavedBooru {
                api: site.api.as_str().to_owned(),
                base: site.base.clone(),
                ..SavedBooru::default()
            };
            store::update_prefs(|p| remember(&mut p.boorus, added));
        }
        let chosen = sites.get_untracked();
        if !chosen.contains(&site) && chosen.len() < MAX_SEARCH_SITES {
            let mut order = chips.get_untracked();
            order.push(Site {
                name: host_of(&site.base),
                site: site.clone(),
                custom: true,
            });
            on_go.run(toggled(&chosen, &site, &order));
        }
    };
    let remove = Callback::new(move |site: RemoteSite| {
        store::update_prefs(|p| {
            p.boorus
                .retain(|saved| tidy(&saved.base) != tidy(&site.base))
        });
        let chosen = sites.get_untracked();
        if chosen.contains(&site) {
            let rest: Vec<RemoteSite> = chosen.into_iter().filter(|s| *s != site).collect();
            // With none left the address names none, and the first preset shows.
            if rest.is_empty() {
                store::update_prefs(|p| p.booru_sites.clear());
            }
            on_go.run(rest);
        }
    });
    // Accounts saved for a site with no row here, such as one removed from
    // the list. Nothing else would show them.
    let left_behind = Memo::new(move |_| {
        chips.with(|chips| {
            accounts.with(|list| {
                list.iter()
                    .filter(|a| !chips.iter().any(|c| tidy(&c.site.base) == tidy(&a.base)))
                    .cloned()
                    .collect::<Vec<_>>()
            })
        })
    });
    let left_row = move |account: BooruAccount| {
        let id = account.id;
        let (login, site) = (account.login, host_of(&account.base));
        view! {
            <div class="banner info site-left" role="status">
                <span class="break">
                    {move || t!("boorus-account-left", login = login.clone(), site = site.clone())}
                </span>
                <button type="button" class="btn text small" on:click=move |_| forget_account(id, accounts, error)>
                    {move || t!("boorus-account-forget")}
                </button>
            </div>
        }
    };
    let kinds = Signal::derive(|| {
        BooruApi::ALL
            .into_iter()
            .map(|a| Choice::new(a.as_str(), a.name()))
            .collect::<Vec<_>>()
    });
    let row = move |site: Site| {
        let base = site.site.base.clone();
        let account = Signal::derive({
            let base = base.clone();
            move || {
                accounts.with(|list| list.iter().find(|a| tidy(&a.base) == tidy(&base)).cloned())
            }
        });
        let open = Signal::derive(move || editing.with(|e| e.as_deref() == Some(base.as_str())));
        view! {
            <SiteRow
                site=site
                account=account
                open=open
                accounts=accounts
                editing=editing
                error=error
                on_remove=remove
            />
        }
    };
    view! {
        <div class="sheet-scrim" on:click=move |_| on_close.run(())>
            <div
                class="card sheet sites-sheet"
                role="dialog"
                aria-modal="true"
                aria-labelledby="sites-sheet-title"
                node_ref=dialog
                on:click=|e| e.stop_propagation()
            >
                <div class="sheet-head">
                    <h2 id="sites-sheet-title" class="card-title">{move || t!("boorus-sites")}</h2>
                    <button
                        type="button"
                        class="icon-btn"
                        node_ref=shut
                        aria-label=move || t!("notice-close")
                        title=move || t!("notice-close")
                        on:click=move |_| on_close.run(())
                    >
                        <Ico icon=Icon::Close />
                    </button>
                </div>
                {move || error.get().map(|m| view! { <p class="field-error break" role="alert">{m}</p> })}
                <ul class="site-list">
                    <For each=move || chips.get() key=|c| (site_key(&c.site), c.custom) children=row />
                </ul>
                <For each=move || left_behind.get() key=|a| a.id children=left_row />
                // Our own words for a bad address, not the browser's.
                <form class="site-add" novalidate=true on:submit=add>
                    <input
                        type="url"
                        placeholder="https://booru.example"
                        aria-label=move || t!("boorus-address")
                        spellcheck="false"
                        autocapitalize="none"
                        autocomplete="off"
                        prop:value=move || address.get()
                        on:input=move |e| address.set(event_target_value(&e))
                    />
                    <Select
                        class="boxed site-kind-select"
                        value=Signal::derive(move || kind.get().as_str().to_owned())
                        options=kinds
                        on_change=Callback::new(move |v: String| {
                            if let Some(api) = api_of(&v) {
                                kind.set(api);
                            }
                        })
                        label=Signal::derive(|| t!("boorus-api"))
                    />
                    <button
                        class="btn tonal"
                        type="submit"
                        disabled=move || address.with(|a| a.trim().is_empty())
                    >
                        <Ico icon=Icon::Add />
                        {move || t!("add")}
                    </button>
                </form>
            </div>
        </div>
    }
}
