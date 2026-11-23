//! The sheet that makes a link for one post or a selection: an optional
//! title and how long it works, then the link to copy or hand to the
//! browser's own share sheet.

use std::time::Duration;

use leptos::ev::{KeyboardEvent, SubmitEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_client::ClientError;
use starbooru_core::share::MAX_SHARE_TITLE_CHARS;
use wasm_bindgen::JsCast;

use super::LinkField;
use crate::api;
use crate::components::banner::ErrorBanner;
use crate::components::icons::{Ico, Icon};
use crate::components::select::{Choice, Select};
use crate::dom;
use crate::logic::share::{
    DEFAULT_LIFETIME, LIFETIMES, lifetime_days, lifetime_label, lifetime_value, new_share,
    share_error_text, share_url,
};

/// A sheet over the page that makes a link for `post_ids`, in that order,
/// and then shows it.
#[component]
pub fn ShareSheet(post_ids: Vec<i64>, on_close: Callback<()>) -> impl IntoView {
    let count = post_ids.len();
    let ids = StoredValue::new(post_ids);
    let title = RwSignal::new(String::new());
    let lifetime = RwSignal::new(DEFAULT_LIFETIME);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let made = RwSignal::new(None::<String>);
    let first: NodeRef<html::Input> = NodeRef::new();
    first.on_load(|el| {
        request_animation_frame(move || {
            let _ = el.focus();
        });
    });
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
        let request = match new_share(
            ids.get_value(),
            &title.get_untracked(),
            lifetime.get_untracked(),
        ) {
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
                Ok::<_, ClientError>(share_url(&c, &share.token))
            })
            .await;
            busy.try_set(false);
            match result {
                Ok(url) => {
                    made.try_set(Some(url));
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
            .collect::<Vec<_>>()
    });
    let native = dom::can_share();
    let ready = move |url: String| {
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
            <form on:submit=create>
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
                // it again. The list opens upwards, inside the sheet.
                <div class="field">
                    <span>{move || t!("share-lifetime")}</span>
                    <Select
                        class="share-lifetime"
                        value=Signal::derive(move || lifetime_value(lifetime.get()))
                        options=choices
                        on_change=Callback::new(move |v: String| lifetime.set(lifetime_days(&v)))
                        label=Signal::derive(|| t!("share-lifetime"))
                    />
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
                class="card sheet share-sheet"
                role="dialog"
                aria-modal="true"
                aria-labelledby="share-sheet-title"
                on:click=|e| e.stop_propagation()
            >
                <div class="sheet-head">
                    <h2 id="share-sheet-title" class="card-title">
                        {move || t!("share-title", count = count)}
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
                <p class="muted hint">{move || t!("share-lead", count = count)}</p>
                {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
                {move || match made.get() {
                    Some(url) => ready(url).into_any(),
                    None => form().into_any(),
                }}
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_sheet_is_a_plain_card_whose_list_opens_upwards() {
        let source = include_str!("sheet.rs");
        // Glass is for the bars and the viewer; the sheet is a card.
        assert!(source.contains(r#"class="card sheet share-sheet""#));
        let css = include_str!("../../../style/pages/share.css");
        assert!(css.contains(".share-sheet .select-list { top: auto; bottom: calc(100% + 6px);"));
    }
}
