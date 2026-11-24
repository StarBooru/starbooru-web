//! The Back button of a page under a tab: Trash, Duplicates, Help and the
//! administration under Settings, a tag's page under Tags. An installed app
//! has no browser Back button, so each of these pages has its own.
//!
//! It goes back through history to the page before, down to where that page
//! was left, when there is one in the app (`logic::trail`). A page opened
//! first goes up to the page above it instead.

use std::cell::RefCell;

use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

use super::icons::{Ico, Icon};
use super::legal::{Opener, back_to};
use crate::dom::{self, Watch};
use crate::logic::legal::plain_click;
use crate::logic::trail::{Spot, before, path_of, scrolled, step};

thread_local! {
    static TRAIL: RefCell<Vec<Spot>> = const { RefCell::new(Vec::new()) };
    static WATCH: RefCell<Option<Watch>> = const { RefCell::new(None) };
}

/// Follows the router for [`BackButton`]. Call once, inside the router.
pub fn follow() {
    let location = use_location();
    Effect::new(move |_| {
        let path = location.pathname.get();
        let search = location.search.get();
        let here = if search.is_empty() {
            path
        } else {
            format!("{path}?{}", search.trim_start_matches('?'))
        };
        TRAIL.with_borrow_mut(|trail| step(trail, &here));
    });
    // How far down a page is when something on it is clicked, before a link
    // takes the tab away.
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let watch = dom::listen_capture(&document, "click", |_| {
        let (y, _) = dom::scroll();
        TRAIL.with_borrow_mut(|trail| scrolled(trail, y));
    });
    WATCH.with_borrow_mut(|w| *w = Some(watch));
}

/// An arrow before the page's title. `up` is the page above this one.
#[component]
pub fn BackButton(#[prop(into)] up: String) -> impl IntoView {
    let back = move |e: MouseEvent| {
        let modifier = e.ctrl_key() || e.meta_key() || e.shift_key() || e.alt_key();
        if !plain_click(e.button(), modifier) {
            return;
        }
        let Some(spot) = TRAIL.with_borrow(|t| before(t).cloned()) else {
            return;
        };
        if web_sys::window().and_then(|w| w.history().ok()).is_none() {
            return;
        }
        e.prevent_default();
        back_to(Opener {
            path: path_of(&spot.at).to_owned(),
            y: spot.y,
        });
    };
    view! {
        <a
            class="icon-btn page-back"
            href=up
            title=move || t!("page-back")
            aria-label=move || t!("page-back")
            on:click=back
        >
            <Ico icon=Icon::ArrowBack />
        </a>
    }
}
