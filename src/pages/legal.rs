//! The privacy policy and the terms of service. Both open without signing in,
//! so the sign-in card can link to them. The text says what the software
//! does; each server's operator is the one responsible for running it.

use leptos::ev::MouseEvent;
use leptos::prelude::*;
use starbooru_client::Client;

use crate::components::brand::{Mark, Wordmark};
use crate::components::icons::{Ico, Icon};
use crate::components::legal::{back_to, keep_opener, take_opener};
use crate::logic::address;
use crate::logic::legal::{back_href, back_through_history, plain_click};
use crate::{api, route, store};

/// A paragraph, or a list of points.
enum Block {
    Text(fn() -> String),
    List(&'static [fn() -> String]),
}

/// A part of a page under its own heading. `id` is the heading's, for links
/// to it, and the same in every language.
struct Section {
    id: &'static str,
    heading: fn() -> String,
    body: &'static [Block],
}

/// One of the two pages.
struct Doc {
    href: &'static str,
    title: fn() -> String,
    lead: fn() -> String,
    sections: &'static [Section],
}

const PRIVACY: Doc = Doc {
    href: route::PRIVACY,
    title: || t!("privacy-title"),
    lead: || t!("privacy-lead"),
    sections: &[
        Section {
            id: "who-is-responsible",
            heading: || t!("privacy-responsible"),
            body: &[Block::Text(|| t!("privacy-responsible-body"))],
        },
        Section {
            id: "what-the-server-stores",
            heading: || t!("privacy-server"),
            body: &[Block::List(&[
                || t!("privacy-server-account"),
                || t!("privacy-server-sessions"),
                || t!("privacy-server-files"),
                || t!("privacy-server-content"),
                || t!("privacy-server-shares"),
                || t!("privacy-server-sites"),
                || t!("privacy-server-limits"),
                || t!("privacy-server-guest-limits"),
            ])],
        },
        Section {
            id: "on-your-device",
            heading: || t!("privacy-device"),
            body: &[Block::List(&[
                || t!("privacy-device-storage"),
                || t!("privacy-device-sign-in"),
                || t!("privacy-device-sign-out"),
                || t!("privacy-device-cache"),
            ])],
        },
        Section {
            id: "what-starbooru-does-not-do",
            heading: || t!("privacy-not-done"),
            body: &[Block::List(&[
                || t!("privacy-not-done-tracking"),
                || t!("privacy-not-done-contacts"),
                || t!("privacy-not-done-boorus"),
                || t!("privacy-not-done-reverse"),
                || t!("privacy-not-done-ai"),
                || t!("privacy-not-done-library"),
                || t!("privacy-not-done-logs"),
            ])],
        },
        Section {
            id: "who-can-see-your-posts",
            heading: || t!("privacy-who-sees"),
            body: &[
                Block::Text(|| t!("privacy-who-sees-body")),
                Block::Text(|| t!("privacy-who-sees-public")),
            ],
        },
        Section {
            id: "keeping-and-deleting",
            heading: || t!("privacy-keeping"),
            body: &[Block::List(&[
                || t!("privacy-keeping-bin"),
                || t!("privacy-keeping-sessions"),
                || t!("privacy-keeping-admins"),
                || t!("privacy-keeping-backups"),
            ])],
        },
    ],
};

const TERMS: Doc = Doc {
    href: route::TERMS,
    title: || t!("terms-title"),
    lead: || t!("terms-lead"),
    sections: &[
        Section {
            id: "your-account",
            heading: || t!("terms-account"),
            body: &[Block::List(&[
                || t!("terms-account-keys"),
                || t!("terms-account-admins"),
            ])],
        },
        Section {
            id: "your-content",
            heading: || t!("terms-content"),
            body: &[Block::List(&[
                || t!("terms-content-rights"),
                || t!("terms-content-yours"),
                || t!("terms-content-originals"),
            ])],
        },
        Section {
            id: "fair-use",
            heading: || t!("terms-fair"),
            body: &[Block::List(&[
                || t!("terms-fair-access"),
                || t!("terms-fair-load"),
            ])],
        },
        Section {
            id: "no-warranty",
            heading: || t!("terms-warranty"),
            body: &[Block::Text(|| t!("terms-warranty-body"))],
        },
    ],
};

#[component]
pub fn Privacy() -> impl IntoView {
    view! { <LegalPage doc=&PRIVACY other=&TERMS /> }
}

#[component]
pub fn Terms() -> impl IntoView {
    view! { <LegalPage doc=&TERMS other=&PRIVACY /> }
}

/// The server's name, which needs no sign-in: the session's server, or the
/// one the sign-in screen starts with. `None` when it can't be asked.
async fn server_name(signed_in: bool, server: String) -> Option<String> {
    if signed_in {
        return api::server_info().await.map(|i| i.name);
    }
    let client = Client::new(&server).ok()?;
    client.info().await.ok().map(|i| i.name)
}

/// The frame both pages share: a way back, the brand, a readable column with
/// a heading to link to for each section, and the links to both pages.
#[component]
fn LegalPage(doc: &'static Doc, other: &'static Doc) -> impl IntoView {
    let opened_from = StoredValue::new(take_opener());
    let signed_in = Memo::new(|_| store::signed_in());
    let server = store::session_untracked().map_or_else(store::default_server, |s| s.server);
    let name = {
        let server = server.clone();
        let signed_in = signed_in.get_untracked();
        LocalResource::new(move || server_name(signed_in, server.clone()))
    };
    let shown = address::shown(&server).to_owned();
    // Empty until the server answers, so the line doesn't change its words
    // in front of the reader.
    let server_line = move || {
        if shown.is_empty() {
            return String::new();
        }
        let address = shown.clone();
        match name.get() {
            Some(Some(name)) => t!("legal-server", name = name, address = address),
            Some(None) => t!("legal-server-address", address = address),
            None => String::new(),
        }
    };

    let back_label = move || {
        if signed_in.get() {
            t!("legal-back-settings")
        } else {
            t!("legal-back-sign-in")
        }
    };
    // Through history when this page was opened from where Back leads, so
    // that page comes back as it was; otherwise the link goes there.
    let back = move |e: MouseEvent| {
        let modifier = e.ctrl_key() || e.meta_key() || e.shift_key() || e.alt_key();
        if !plain_click(e.button(), modifier) {
            return;
        }
        let Some(from) = opened_from.get_value() else {
            return;
        };
        let length = web_sys::window()
            .and_then(|w| w.history().ok())
            .and_then(|h| h.length().ok())
            .unwrap_or(0);
        if back_through_history(Some(&from.path), signed_in.get_untracked(), length) {
            e.prevent_default();
            back_to(from);
        }
    };
    // The other page takes this one's place, so Back still leads where this
    // one was opened from.
    let to_other = move |_: MouseEvent| keep_opener(opened_from.get_value());

    // A link to a section opened from outside: the browser looked for the
    // heading before the page was drawn.
    Effect::new(|_| scroll_to_hash());

    let sections = doc
        .sections
        .iter()
        .map(|section| {
            let Section { id, heading, body } = *section;
            let body = body.iter().map(block).collect_view();
            view! {
                <section aria-labelledby=id>
                    <h2 id=id>
                        <a
                            class="legal-anchor"
                            href=format!("#{id}")
                            prop:replace=true
                            title=move || t!("legal-section-link")
                        >
                            {move || heading()}
                            <span class="legal-hash" aria-hidden="true">
                                "#"
                            </span>
                        </a>
                    </h2>
                    {body}
                </section>
            }
        })
        .collect_view();
    let (title, lead) = (doc.title, doc.lead);
    let (other_href, other_title) = (other.href, other.title);
    let privacy_first = doc.href == route::PRIVACY;
    let here = move || {
        view! {
            <a href=doc.href aria-current="page">
                {move || title()}
            </a>
        }
    };
    let there = move || {
        view! {
            <a href=other_href prop:replace=true on:click=to_other>
                {move || other_title()}
            </a>
        }
    };
    let (first, second) = if privacy_first {
        (here().into_any(), there().into_any())
    } else {
        (there().into_any(), here().into_any())
    };

    view! {
        <div class="legal-page">
            <nav class="legal-top">
                <a class="btn text small legal-back" href=move || back_href(signed_in.get()) on:click=back>
                    <Ico icon=Icon::ArrowBack size=20 />
                    {back_label}
                </a>
            </nav>
            <article class="legal card">
                <a class="legal-brand" href="/" aria-label=move || t!("app-name")>
                    <Mark height=30 compact=true />
                    <Wordmark height=19 />
                </a>
                <h1>{move || title()}</h1>
                <p class="muted legal-server">{server_line}</p>
                <p class="lead">{move || lead()}</p>
                {sections}
                <nav class="legal-links" aria-label=move || t!("legal-nav")>
                    {first}
                    {second}
                    <a href=move || back_href(signed_in.get()) on:click=back>
                        {back_label}
                    </a>
                </nav>
            </article>
        </div>
    }
}

fn block(block: &Block) -> AnyView {
    match block {
        Block::Text(text) => {
            let text = *text;
            view! { <p>{move || text()}</p> }.into_any()
        }
        Block::List(items) => {
            let items = items
                .iter()
                .map(|item| {
                    let item = *item;
                    view! { <li>{move || item()}</li> }
                })
                .collect_view();
            view! { <ul>{items}</ul> }.into_any()
        }
    }
}

/// Brings the heading the address names into view.
fn scroll_to_hash() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let hash = window.location().hash().unwrap_or_default();
    let id = hash.trim_start_matches('#');
    if id.is_empty() {
        return;
    }
    if let Some(heading) = window.document().and_then(|d| d.get_element_by_id(id)) {
        heading.scroll_into_view();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn every_section_has_a_heading_id_of_its_own() {
        let mut seen = HashSet::new();
        for doc in [&PRIVACY, &TERMS] {
            assert!(!doc.sections.is_empty(), "{}", doc.href);
            for section in doc.sections {
                let id = section.id;
                assert!(!id.is_empty(), "{}: a section without an id", doc.href);
                // Safe in an address's fragment and in CSS, as typed.
                assert!(
                    id.chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                        && id.starts_with(|c: char| c.is_ascii_lowercase()),
                    "{id}"
                );
                // Unique across both pages, so no link can mean two places.
                assert!(seen.insert(id), "{id} twice");
                assert!(!section.body.is_empty(), "{id}: nothing under it");
            }
        }
    }

    #[test]
    fn every_heading_and_paragraph_has_its_text() {
        let has_text = |text: fn() -> String| {
            let shown = text();
            assert!(
                !shown.is_empty() && !shown.starts_with("privacy-") && !shown.starts_with("terms-"),
                "{shown}"
            );
        };
        for doc in [&PRIVACY, &TERMS] {
            has_text(doc.title);
            has_text(doc.lead);
            for section in doc.sections {
                has_text(section.heading);
                for block in section.body {
                    match block {
                        Block::Text(text) => has_text(*text),
                        Block::List(items) => items.iter().copied().for_each(has_text),
                    }
                }
            }
        }
    }
}
