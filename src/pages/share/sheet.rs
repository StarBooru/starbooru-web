//! The sheet that makes a link for one post or a selection: an optional
//! title and how long it works, for a set number of days or until a day
//! picked on a calendar, then the link to copy or hand to the browser's own
//! share sheet. A public server shares a post by its own address, which
//! anyone can open: there the sheet holds the addresses at once.

use std::time::Duration;

use chrono::{DateTime, Local, Utc};
use leptos::ev::{KeyboardEvent, SubmitEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_client::ClientError;
use starbooru_core::share::MAX_SHARE_TITLE_CHARS;
use wasm_bindgen::JsCast;

use super::{LinkField, ListField};
use crate::components::banner::ErrorBanner;
use crate::components::focus::hold_focus;
use crate::components::icons::{Ico, Icon};
use crate::components::select::{Choice, Select};
use crate::dom;
use crate::logic::share::{
    DEFAULT_LIFETIME, LIFETIMES, PICK_DAY, PickError, day_value, ends_after, ends_text, first_day,
    first_pick, last_day, lifetime_days, lifetime_label, lifetime_value, new_share,
    pick_error_text, picked_days, post_url, post_urls, share_error_text, share_url,
};
use crate::{api, instance};

/// A sheet over the page that makes a link for `post_ids`, in that order,
/// and then shows it.
#[component]
pub fn ShareSheet(post_ids: Vec<i64>, on_close: Callback<()>) -> impl IntoView {
    let count = post_ids.len();
    // The posts' own addresses, on a public server.
    let addresses = untrack(instance::public)
        .then(api::client)
        .flatten()
        .map(|c| match post_ids.as_slice() {
            [one] => Address::One(post_url(&c, *one)),
            many => Address::Many(post_urls(&c, many)),
        });
    let ids = StoredValue::new(post_ids);
    let title = RwSignal::new(String::new());
    let lifetime = RwSignal::new(DEFAULT_LIFETIME);
    // The calendar: whether it's the choice, the day it opened on, the
    // field's value now, and whether a link was refused for want of a day.
    let picking = RwSignal::new(false);
    let opened_on = RwSignal::new(String::new());
    let picked = RwSignal::new(String::new());
    let tried = RwSignal::new(false);
    let day_field: NodeRef<html::Input> = NodeRef::new();
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    // The link, and when the server says it ends.
    let made = RwSignal::new(None::<(String, Option<DateTime<Utc>>)>);
    let first: NodeRef<html::Input> = NodeRef::new();
    first.on_load(|el| {
        request_animation_frame(move || {
            let _ = el.focus();
        });
    });
    // Tab stays in the sheet, and the focus goes back to Share after.
    let dialog: NodeRef<html::Div> = NodeRef::new();
    hold_focus(dialog, true);
    // Escape also closes the sheet when the focus is outside it, as it is for
    // a moment while it opens or while the link replaces the form. Inside,
    // the scrim handles it, after an open list has had it.
    let keys = web_sys::window().and_then(|w| w.document()).map(|doc| {
        dom::listen(&doc, "keydown", false, move |e| {
            let escape = e
                .dyn_ref::<web_sys::KeyboardEvent>()
                .is_some_and(|k| k.key() == "Escape");
            let inside = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                .and_then(|el| el.closest(".share-sheet").ok().flatten())
                .is_some();
            if escape && !inside && !e.default_prevented() {
                e.prevent_default();
                e.stop_propagation();
                on_close.try_run(());
            }
        })
    });
    on_cleanup(move || {
        // Escape closes the sheet from inside this very listener, which can't
        // be dropped while it runs.
        set_timeout(
            move || {
                if let Some(keys) = keys {
                    keys.stop();
                }
            },
            Duration::ZERO,
        );
    });

    let create = move |e: SubmitEvent| {
        e.prevent_default();
        if busy.get_untracked() {
            return;
        }
        // Checked again here: the sheet may have stayed open past midnight.
        let days = if picking.get_untracked() {
            match picked_days(&picked.get_untracked(), Utc::now(), &Local) {
                Ok(days) => Some(days),
                Err(_) => {
                    tried.set(true);
                    if let Some(el) = day_field.get_untracked() {
                        let _ = el.focus();
                    }
                    return;
                }
            }
        } else {
            lifetime.get_untracked()
        };
        let request = match new_share(ids.get_value(), &title.get_untracked(), days) {
            Ok(request) => request,
            Err(problem) => {
                // Read in a submit, where the language isn't followed.
                error.set(Some(untrack(|| share_error_text(&problem))));
                return;
            }
        };
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            let result = api::call(move |c| async move {
                let share = c.create_share(&request).await?;
                Ok::<_, ClientError>((share_url(&c, &share.token), share.expires_at))
            })
            .await;
            busy.try_set(false);
            match result {
                Ok(link) => {
                    made.try_set(Some(link));
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    };

    let choices = Signal::derive(|| {
        LIFETIMES
            .into_iter()
            .map(|d| Choice::new(lifetime_value(d), lifetime_label(d)))
            .chain([Choice::new(PICK_DAY, t!("share-pick-day"))])
            .collect::<Vec<_>>()
    });
    let chosen = Signal::derive(move || {
        if picking.get() {
            PICK_DAY.to_owned()
        } else {
            lifetime_value(lifetime.get())
        }
    });
    let choose = Callback::new(move |value: String| {
        if value != PICK_DAY {
            picking.set(false);
            lifetime.set(lifetime_days(&value));
            return;
        }
        // The calendar opens on the day the lifetime chosen would have
        // covered, unless it already holds one that will do.
        let now = Utc::now();
        if picked_days(&picked.get_untracked(), now, &Local).is_err() {
            let day = first_pick(lifetime.get_untracked(), now, &Local)
                .map(day_value)
                .unwrap_or_default();
            opened_on.set(day.clone());
            picked.set(day);
        }
        tried.set(false);
        picking.set(true);
        request_animation_frame(move || {
            if let Some(el) = day_field.try_get_untracked().flatten() {
                let _ = el.focus();
            }
        });
    });
    // Under the lifetime: when the link would stop working, or why the
    // picked day won't do. An empty field is only a hint until Create.
    let ends = Memo::new(move |_| {
        let now = Utc::now();
        if !picking.get() {
            return (ends_text(ends_after(lifetime.get(), now)), false);
        }
        match picked_days(&picked.get(), now, &Local) {
            Ok(days) => (ends_text(ends_after(Some(days), now)), false),
            Err(problem) => (
                pick_error_text(problem, now, &Local),
                problem != PickError::NoDay || tried.get(),
            ),
        }
    });
    let wrong = move || ends.with(|(_, wrong)| *wrong);
    // The bounds follow the clock each time the calendar opens.
    let bound = move |last: bool| {
        picking.track();
        let now = Utc::now();
        let day = if last {
            last_day(now, &Local)
        } else {
            first_day(now, &Local)
        };
        day.map(day_value)
    };
    let native = dom::can_share();
    let ready = move |(url, expires_at): (String, Option<DateTime<Utc>>)| {
        let handed = url.clone();
        let share = native.then(|| {
            view! {
                <button
                    class="btn tonal"
                    type="button"
                    on:click=move |_| {
                        let (title, url) = (title.get_untracked(), handed.clone());
                        spawn_local(async move { dom::share(title.trim(), &url).await });
                    }
                >
                    <Ico icon=Icon::Share />
                    {move || t!("share-native")}
                </button>
            }
        });
        view! {
            <div class="banner info key-created" role="status">
                <strong>{move || t!("share-ready")}</strong>
                <LinkField url=url focus=true />
                <small class="share-ends">{move || ends_text(expires_at)}</small>
            </div>
            <div class="row-end">
                {share}
                <button class="btn" type="button" on:click=move |_| on_close.run(())>
                    {move || t!("share-done")}
                </button>
            </div>
        }
    };
    let form = move || {
        view! {
            // The sheet checks the day itself and says what's wrong in words.
            <form on:submit=create novalidate=true>
                <div class="field">
                    <label for="share-name">{move || t!("share-name")}</label>
                    <input
                        id="share-name"
                        node_ref=first
                        maxlength=MAX_SHARE_TITLE_CHARS.to_string()
                        autocomplete="off"
                        placeholder=move || t!("share-name-placeholder")
                        prop:value=move || title.get()
                        on:input=move |e| title.set(event_target_value(&e))
                    />
                </div>
                // Not a `label`: a click on the list would press the button in
                // it again. The list opens upwards, inside the sheet. The day
                // opens beside the drop-down, so the sheet keeps its height.
                <div class="field share-when">
                    <span>{move || t!("share-lifetime")}</span>
                    <div class="share-lifetime" class:picking=move || picking.get()>
                        <Select
                            class="boxed"
                            value=chosen
                            options=choices
                            on_change=choose
                            label=Signal::derive(|| t!("share-lifetime"))
                        />
                        <div class="share-day">
                            <input
                                type="date"
                                node_ref=day_field
                                disabled=move || !picking.get()
                                min=move || bound(false)
                                max=move || bound(true)
                                aria-label=move || t!("share-day")
                                aria-describedby="share-ends"
                                aria-invalid=move || if wrong() { "true" } else { "false" }
                                prop:value=move || opened_on.get()
                                on:input=move |e| picked.set(event_target_value(&e))
                            />
                        </div>
                    </div>
                    <small
                        id="share-ends"
                        class=move || {
                            if wrong() {
                                "share-ends field-error"
                            } else {
                                "share-ends muted field-hint"
                            }
                        }
                        aria-live="polite"
                    >
                        {move || ends.with(|(text, _)| text.clone())}
                    </small>
                </div>
                <div class="row-end">
                    <button class="btn text" type="button" on:click=move |_| on_close.run(())>
                        {move || t!("cancel")}
                    </button>
                    <button class="btn" type="submit" disabled=move || busy.get()>
                        <Ico icon=Icon::Link />
                        {move || t!("share-create")}
                    </button>
                </div>
            </form>
        }
    };

    let by_address = addresses.is_some();
    let heading = move || {
        if by_address {
            t!("share-address-title", count = count)
        } else {
            t!("share-title", count = count)
        }
    };
    let lead = move || {
        if by_address {
            t!("share-address-lead", count = count)
        } else {
            t!("share-lead", count = count)
        }
    };
    let body = move || match addresses.clone() {
        Some(address) => by_address_view(address, on_close).into_any(),
        None => match made.get() {
            Some(link) => ready(link).into_any(),
            None => form().into_any(),
        },
    };
    view! {
        <div
            class="sheet-scrim"
            on:click=move |_| on_close.run(())
            on:keydown=move |e: KeyboardEvent| {
                // The page under the sheet hears none of its keys: arrows
                // would step the viewer, and Escape end a selection.
                e.stop_propagation();
                if e.key() == "Escape" && !e.default_prevented() {
                    on_close.run(());
                }
            }
        >
            <div
                node_ref=dialog
                class="card sheet share-sheet"
                role="dialog"
                aria-modal="true"
                aria-labelledby="share-sheet-title"
                on:click=|e| e.stop_propagation()
            >
                <div class="sheet-head">
                    <h2 id="share-sheet-title" class="card-title">
                        {heading}
                    </h2>
                    <button
                        type="button"
                        class="icon-btn"
                        aria-label=move || t!("share-close")
                        title=move || t!("share-close")
                        on:click=move |_| on_close.run(())
                    >
                        <Ico icon=Icon::Close />
                    </button>
                </div>
                <p class="muted hint">{lead}</p>
                {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
                {body}
            </div>
        </div>
    }
}

/// What a public server shares: a post's address, or several.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Address {
    One(String),
    Many(String),
}

/// The addresses to copy, and to hand to the browser's share sheet where
/// it has one.
fn by_address_view(address: Address, on_close: Callback<()>) -> impl IntoView {
    let native = dom::can_share().then(|| {
        let address = address.clone();
        view! {
            <button
                class="btn tonal"
                type="button"
                on:click=move |_| {
                    let address = address.clone();
                    spawn_local(async move {
                        match address {
                            Address::One(url) => dom::share("", &url).await,
                            Address::Many(list) => dom::share_text("", &list).await,
                        }
                    });
                }
            >
                <Ico icon=Icon::Share />
                {move || t!("share-native")}
            </button>
        }
    });
    let field = match address {
        Address::One(url) => view! { <LinkField url=url focus=true /> }.into_any(),
        Address::Many(list) => view! { <ListField text=list /> }.into_any(),
    };
    view! {
        <div class="share-address">{field}</div>
        <div class="row-end">
            {native}
            <button class="btn" type="button" on:click=move |_| on_close.run(())>
                {move || t!("share-done")}
            </button>
        </div>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_public_server_shares_the_posts_own_address() {
        let source = include_str!("sheet.rs");
        assert!(source.contains("untrack(instance::public)"));
        assert!(source.contains("[one] => Address::One(post_url(&c, *one))"));
        assert!(source.contains("dom::share_text(\"\", &list)"));
    }

    #[test]
    fn the_sheet_is_a_plain_card_whose_list_opens_upwards() {
        let source = include_str!("sheet.rs");
        // Glass is for the bars and the viewer; the sheet is a card.
        assert!(source.contains(r#"class="card sheet share-sheet""#));
        let css = include_str!("../../../style/pages/share.css");
        assert!(css.contains(".share-sheet .select-list { top: auto; bottom: calc(100% + 6px);"));
    }

    #[test]
    fn the_day_opens_beside_the_lifetime_and_not_below_it() {
        let css = include_str!("../../../style/pages/share.css");
        // Same two columns, one of them closed, so the sheet keeps its height.
        assert!(css.contains("grid-template-columns: minmax(0, 6fr) minmax(0, 0fr);"));
        assert!(css.contains(
            ".share-lifetime.picking { grid-template-columns: minmax(0, 6fr) minmax(0, 5fr);"
        ));
        assert!(css.contains(".app[data-effects=\"lite\"] .share-lifetime { transition: none; }"));
        // The browser's own checks are off, so a refusal is in the app's words.
        assert!(include_str!("sheet.rs").contains("<form on:submit=create novalidate=true>"));
    }
}
