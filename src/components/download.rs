//! Saving a post's original under its own name. `<a download>` names the file
//! only on the page's own origin: for media on another server the browser
//! would open the file instead, so it's fetched first and saved from memory.

use std::time::Duration;

use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use super::icons::{Ico, Icon};
use super::snackbar::notify;
use crate::logic::viewer::same_origin;

/// A button that saves `href`, a signed media address, as `name`.
#[component]
pub fn DownloadLink(href: String, name: String) -> impl IntoView {
    let label = move || t!("post-download");
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
            if let Err(reason) = save(&href, &name).await {
                notify(t!("download-failed", reason = reason));
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
        if let Err(reason) = save(&href, &name).await {
            notify(t!("download-failed", reason = reason));
        }
    });
}

fn page_origin() -> String {
    web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default()
}

/// What went wrong, as the browser put it.
fn reason(e: JsValue) -> String {
    e.dyn_ref::<js_sys::Error>()
        .map(|e| String::from(e.message()))
        .or_else(|| e.as_string())
        .unwrap_or_default()
}

async fn save(href: &str, name: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or_else(String::new)?;
    let answer = JsFuture::from(window.fetch_with_str(href))
        .await
        .map_err(reason)?;
    let response: web_sys::Response = answer.dyn_into().map_err(reason)?;
    if !response.ok() {
        return Err(format!("HTTP {}", response.status()));
    }
    let body = JsFuture::from(response.blob().map_err(reason)?)
        .await
        .map_err(reason)?;
    let blob: web_sys::Blob = body.dyn_into().map_err(reason)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(reason)?;
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
