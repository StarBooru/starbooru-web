//! The links to the privacy policy and the terms, under the sign-in card and
//! in Settings, and the page they were followed from, for the legal pages'
//! Back (`pages/legal.rs`).

use std::cell::RefCell;

use leptos::ev::MouseEvent;
use leptos::prelude::*;

use crate::logic::legal::plain_click;
use crate::{dom, route};

/// The page a legal page was opened from, and how far down it was.
#[derive(Clone, Debug, PartialEq)]
pub struct Opener {
    pub path: String,
    pub y: f64,
}

thread_local! {
    static OPENED_FROM: RefCell<Option<Opener>> = const { RefCell::new(None) };
}

/// Notes the page a link to a legal page is followed from. A click that
/// opens a new tab or window leaves this one where it is, so it notes nothing.
pub fn note_opener(e: &MouseEvent) {
    let modifier = e.ctrl_key() || e.meta_key() || e.shift_key() || e.alt_key();
    if !plain_click(e.button(), modifier) {
        return;
    }
    let Some(path) = web_sys::window().and_then(|w| w.location().pathname().ok()) else {
        return;
    };
    keep_opener(Some(Opener {
        path,
        y: dom::scroll().0,
    }));
}

/// Hands the page a legal page was opened from on to the next one.
pub fn keep_opener(opener: Option<Opener>) {
    OPENED_FROM.with_borrow_mut(|from| *from = opener);
}

/// The page the legal page now opening was opened from, once.
pub fn take_opener() -> Option<Opener> {
    OPENED_FROM.with_borrow_mut(Option::take)
}

/// Back through history to `opener`, then down to where it was once it's
/// on the screen and tall enough: Settings fills in its cards as they load.
pub fn back_to(opener: Opener) {
    let Some(history) = web_sys::window().and_then(|w| w.history().ok()) else {
        return;
    };
    if history.back().is_err() {
        return;
    }
    // About a second, at a frame a try.
    put_back(opener, 60);
}

fn put_back(opener: Opener, tries: u32) {
    let window = web_sys::window();
    let there = window
        .as_ref()
        .and_then(|w| w.location().pathname().ok())
        .is_some_and(|path| path == opener.path);
    let tall = window
        .as_ref()
        .and_then(web_sys::Window::document)
        .and_then(|d| d.document_element())
        .map_or(0.0, |el| f64::from(el.scroll_height()));
    let (_, screen) = dom::scroll();
    if (!there || tall - screen < opener.y) && tries > 0 {
        request_animation_frame(move || put_back(opener, tries - 1));
        return;
    }
    if there {
        dom::scroll_to(opener.y, false);
    }
}

#[component]
pub fn LegalLinks() -> impl IntoView {
    view! {
        <nav class="legal-links" aria-label=move || t!("legal-nav")>
            <a href=route::PRIVACY on:click=move |e| note_opener(&e)>
                {move || t!("legal-privacy-short")}
            </a>
            <a href=route::TERMS on:click=move |e| note_opener(&e)>
                {move || t!("legal-terms-short")}
            </a>
        </nav>
    }
}
