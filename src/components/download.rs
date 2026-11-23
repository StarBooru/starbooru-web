//! Saving a post's original under its own name. `<a download>` names the file
//! only on the page's own origin: for media on another server the browser
//! would open the file instead, so it's fetched first and saved from memory.

use std::time::Duration;

use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use super::icons::{Ico, Icon};
use super::snackbar::notify;
use crate::dom;
use crate::logic::viewer::same_origin;

/// A button that saves `href`, a signed media address, as `name`. `label`
/// says what it saves, when that's more than the post on screen.
#[component]
pub fn DownloadLink(
    href: String,
    name: String,
    #[prop(optional, into)] label: Option<Signal<String>>,
) -> impl IntoView {
    let label = move || label.map_or_else(|| t!("post-download"), |l| l.get());
    let target = href.clone();
    let file = name.clone();
    let click = move |e: MouseEvent| {
        // With a modifier the user wants the link itself, in a tab or a window.
        if e.ctrl_key() || e.meta_key() || e.shift_key() || e.alt_key() {
            return;
        }
        if same_origin(&target, &page_origin()) {
            return;
        }
        e.prevent_default();
        let (href, name) = (target.clone(), file.clone());
        spawn_local(async move {
            if let Err(failed) = save(&href, &name).await {
                notify(failed.text());
            }
        });
    };
    view! {
        <a class="icon-btn" href=href download=name rel="noopener" title=label aria-label=label on:click=click>
            <Ico icon=Icon::Download />
            <span class="bar-label">{label}</span>
        </a>
    }
}

/// Saves `href`, a signed media address, as `name`, as the button does: for
/// a menu, which has no link of its own to click.
pub fn start(href: String, name: String) {
    if same_origin(&href, &page_origin()) {
        click_link(&href, &name);
        return;
    }
    spawn_local(async move {
        if let Err(failed) = save(&href, &name).await {
            notify(failed.text());
        }
    });
}

fn page_origin() -> String {
    dom::origin().unwrap_or_default()
}

/// Why a download failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Failed {
    /// The signed link ran out, or the file has gone since the page loaded.
    Expired,
    /// The network, the server, or the browser: trying again may work.
    Other,
}

impl Failed {
    /// From the status the media server answered with.
    fn of_status(status: u16) -> Self {
        match status {
            403 | 404 | 410 => Failed::Expired,
            _ => Failed::Other,
        }
    }

    fn text(self) -> String {
        match self {
            Failed::Expired => t!("download-expired"),
            Failed::Other => t!("download-failed"),
        }
    }
}

/// The browser's words for what went wrong aren't the user's language, and
/// say nothing to act on.
fn other<T>(_: T) -> Failed {
    Failed::Other
}

async fn save(href: &str, name: &str) -> Result<(), Failed> {
    let window = web_sys::window().ok_or(Failed::Other)?;
    let answer = JsFuture::from(window.fetch_with_str(href))
        .await
        .map_err(other)?;
    let response: web_sys::Response = answer.dyn_into().map_err(other)?;
    if !response.ok() {
        return Err(Failed::of_status(response.status()));
    }
    let body = JsFuture::from(response.blob().map_err(other)?)
        .await
        .map_err(other)?;
    let blob: web_sys::Blob = body.dyn_into().map_err(other)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(other)?;
    click_link(&url, name);
    // Long enough for the browser to have taken the file.
    set_timeout(
        move || {
            let _ = web_sys::Url::revoke_object_url(&url);
        },
        Duration::from_secs(60),
    );
    Ok(())
}

/// Saves `href` as `name` through a link of its own, as the browser saves a
/// link's file.
fn click_link(href: &str, name: &str) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Some(link) = document
        .create_element("a")
        .ok()
        .and_then(|el| el.dyn_into::<web_sys::HtmlAnchorElement>().ok())
    else {
        return;
    };
    link.set_href(href);
    link.set_download(name);
    link.set_rel("noopener");
    if let Some(body) = document.body() {
        let _ = body.append_child(&link);
        link.click();
        link.remove();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_link_run_out_says_to_reload() {
        for status in [403, 404, 410] {
            assert_eq!(Failed::of_status(status), Failed::Expired, "{status}");
        }
        for status in [400, 429, 500, 502, 503] {
            assert_eq!(Failed::of_status(status), Failed::Other, "{status}");
        }
        assert_eq!(
            Failed::Expired.text(),
            "The link to this file has expired. Reload the page and try again."
        );
        assert!(!Failed::Other.text().contains("HTTP"));
    }
}
