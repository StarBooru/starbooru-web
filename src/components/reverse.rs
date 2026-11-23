//! Looking a post's picture up on Google Lens, iqdb, SauceNAO and trace.moe,
//! from a menu of the four; `logic/reverse.rs` says how each one gets it.
//! Nothing goes to a site before it's picked. The copy to send is read from
//! this server as the menu opens, so a pick can post it straight away, while
//! the click still lets the browser open a tab.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::Post;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{JsFuture, future_to_promise};
use web_sys::{Blob, File};

use super::download;
use super::icons::Icon;
use super::menu::{ActionMenu, MenuAt, MenuItem};
use super::snackbar::{notify, notify_undo};
use crate::api;
use crate::logic::reverse::{self, Browser, Site, Way};

/// The copy to send.
#[derive(Clone, Debug)]
struct Picture {
    /// As the API gave it, on this server.
    path: String,
    url: String,
    name: String,
    mime: &'static str,
}

impl Picture {
    fn of(post: &Post) -> Option<Self> {
        let path = reverse::picture(post)?;
        let (mime, _) = reverse::file_type(path)?;
        Some(Picture {
            path: path.to_owned(),
            url: api::client().map_or_else(|| path.to_owned(), |c| c.absolute(path)),
            name: reverse::file_name(post.id, path),
            mime,
        })
    }
}

/// The copy read last, and the same as PNG when the clipboard will want it.
struct Ready {
    path: String,
    file: File,
    png: Option<Blob>,
}

thread_local! {
    static READY: RefCell<Option<Ready>> = const { RefCell::new(None) };
}

/// The four sites for `post`, as a menu at `at`. Each says how the picture
/// gets there: uploaded, by its link, copied to paste, or saved to upload.
#[component]
pub fn ReverseMenu(post: Arc<Post>, at: MenuAt, on_close: Callback<()>) -> impl IntoView {
    let picture = Picture::of(&post);
    let browser = browser();
    let reachable = picture.as_ref().is_some_and(|p| reverse::reachable(&p.url));
    let ways: Vec<(Site, Way)> = picture
        .as_ref()
        .map(|p| {
            Site::ALL
                .iter()
                .map(|&site| (site, reverse::way(site, &p.url, reachable, browser)))
                .collect()
        })
        .unwrap_or_default();
    if let Some(p) = &picture
        && ways.iter().any(|(_, way)| way.needs_file())
    {
        warm(p, ways.iter().any(|(_, way)| matches!(way, Way::Paste(_))));
    }
    let items = ways
        .into_iter()
        .map(|(site, way)| {
            let (icon, hint): (Icon, fn() -> String) = match way {
                Way::Upload { .. } => (Icon::Upload, || t!("reverse-upload")),
                Way::Address(_) => (Icon::Link, || t!("reverse-address")),
                Way::Paste(_) => (Icon::Copy, || t!("reverse-paste")),
                Way::Save(_) => (Icon::Download, || t!("reverse-save")),
            };
            let picture = picture.clone();
            view! {
                <MenuItem
                    icon=icon
                    label=move || name(site)
                    hint=move || hint()
                    on_click=Callback::new(move |()| {
                        on_close.run(());
                        if let Some(picture) = picture.clone() {
                            go(site, way.clone(), picture);
                        }
                    })
                />
            }
        })
        .collect_view();
    view! {
        <ActionMenu at=at title=t!("reverse-menu") on_close=on_close>
            {items}
        </ActionMenu>
    }
}

fn name(site: Site) -> String {
    match site {
        Site::Lens => t!("reverse-lens"),
        Site::Iqdb => t!("reverse-iqdb"),
        Site::SauceNao => t!("reverse-saucenao"),
        Site::TraceMoe => t!("reverse-trace"),
    }
}

/// What the browser can do with a picture.
fn browser() -> Browser {
    let Some(window) = web_sys::window() else {
        return Browser::default();
    };
    let fine = window
        .match_media("(any-pointer: fine)")
        .ok()
        .flatten()
        .is_some_and(|m| m.matches());
    Browser {
        clipboard: has(&window.navigator(), "clipboard") && has(&window, "ClipboardItem"),
        touch: !fine,
    }
}

fn has(target: &JsValue, name: &str) -> bool {
    js_sys::Reflect::get(target, &JsValue::from_str(name))
        .is_ok_and(|v| !v.is_undefined() && !v.is_null())
}

/// Sends the picture to `site` the way it takes it.
fn go(site: Site, way: Way, picture: Picture) {
    match way {
        Way::Address(url) => {
            if !open_tab(&url) {
                blocked(site, url, t!("reverse-blocked"));
            }
        }
        Way::Upload { action, field } => {
            if let Some((file, _)) = ready(&picture.path) {
                upload(site, action, field, &file, &picture);
                return;
            }
            spawn_local(async move {
                match read(&picture).await {
                    Ok(file) => upload(site, action, field, &file, &picture),
                    Err(message) => notify(message),
                }
            });
        }
        Way::Paste(page) => paste(site, page, picture),
        Way::Save(page) => save(site, page, &picture, t!("reverse-saved", site = name(site))),
    }
}

/// Posts `file` to the site's form in a new tab. A browser that can't fill
/// in a file field saves it instead, for the user to upload there.
fn upload(site: Site, action: &'static str, field: &'static str, file: &File, picture: &Picture) {
    if post_form(action, field, file).is_none() {
        save(
            site,
            action,
            picture,
            t!("reverse-saved", site = name(site)),
        );
    }
}

fn post_form(action: &str, field: &str, file: &File) -> Option<()> {
    let document = web_sys::window()?.document()?;
    let form: web_sys::HtmlFormElement = document.create_element("form").ok()?.dyn_into().ok()?;
    form.set_method("post");
    form.set_enctype("multipart/form-data");
    form.set_action(action);
    form.set_target("_blank");
    form.set_attribute("rel", "noopener noreferrer").ok()?;
    form.set_hidden(true);
    let input: web_sys::HtmlInputElement =
        document.create_element("input").ok()?.dyn_into().ok()?;
    input.set_type("file");
    input.set_name(field);
    let files = web_sys::DataTransfer::new().ok()?;
    files.items().add_with_file(file).ok()?;
    input.set_files(files.files().as_ref());
    input.files().filter(|f| f.length() == 1)?;
    form.append_child(&input).ok()?;
    document.body()?.append_child(&form).ok()?;
    let sent = form.submit();
    // The browser has read the form by now; the page needn't keep it.
    set_timeout(move || form.remove(), Duration::from_secs(1));
    sent.ok()
}

/// Puts the picture on the clipboard as PNG, then opens `page` to paste it
/// there. When the clipboard says no, it's saved instead.
fn paste(site: Site, page: &'static str, picture: Picture) {
    // Read now if it wasn't ready: Safari wants the clipboard asked during
    // the click, and takes the picture as a promise.
    let unread = Rc::new(Cell::new(None::<String>));
    let png: JsValue = match ready(&picture.path).and_then(|(_, png)| png) {
        Some(png) => png.into(),
        None => {
            let (picture, unread) = (picture.clone(), Rc::clone(&unread));
            future_to_promise(async move {
                let file = read(&picture).await.map_err(|message| {
                    unread.set(Some(message));
                    JsValue::NULL
                })?;
                to_png(&file).await.map(JsValue::from).ok_or(JsValue::NULL)
            })
            .into()
        }
    };
    let write = write_png(&png);
    spawn_local(async move {
        let copied = match write {
            Some(write) => JsFuture::from(write).await.is_ok(),
            None => false,
        };
        if let Some(message) = unread.take() {
            notify(message);
            return;
        }
        let site_name = name(site);
        if !copied {
            save(
                site,
                page,
                &picture,
                t!("reverse-copy-refused", site = site_name),
            );
            return;
        }
        let keys = reverse::paste_keys(&user_agent());
        let copied = t!("reverse-copied", site = site_name, keys = keys);
        if open_tab(page) {
            notify(copied);
        } else {
            blocked(site, page.to_owned(), copied);
        }
    });
}

fn write_png(png: &JsValue) -> Option<js_sys::Promise> {
    let navigator = web_sys::window()?.navigator();
    if !has(&navigator, "clipboard") {
        return None;
    }
    let record = js_sys::Object::new();
    js_sys::Reflect::set(&record, &JsValue::from_str("image/png"), png).ok()?;
    let item = web_sys::ClipboardItem::new_with_record_from_str_to_blob_promise(&record).ok()?;
    Some(navigator.clipboard().write(&js_sys::Array::of1(&item)))
}

fn user_agent() -> String {
    web_sys::window()
        .and_then(|w| w.navigator().user_agent().ok())
        .unwrap_or_default()
}

/// Saves the picture and opens `page` to upload it there, then says so.
fn save(site: Site, page: &str, picture: &Picture, message: String) {
    download::start(picture.url.clone(), picture.name.clone());
    if open_tab(page) {
        notify(message);
    } else {
        blocked(site, page.to_owned(), message);
    }
}

/// Opens `url` in a tab of its own that can't reach back to this one.
/// `false` when the browser blocked it.
fn open_tab(url: &str) -> bool {
    let tab =
        web_sys::window().and_then(|w| w.open_with_url_and_target(url, "_blank").ok().flatten());
    let Some(tab) = tab else {
        return false;
    };
    let _ = tab.set_opener(&JsValue::NULL);
    true
}

/// The tab didn't open: `message`, with a button that opens it, a click the
/// browser lets through.
fn blocked(site: Site, url: String, message: String) {
    notify_undo(message, t!("reverse-open", site = name(site)), move || {
        open_tab(&url);
    });
}

/// The ready copy of `path`, if the menu has read it.
fn ready(path: &str) -> Option<(File, Option<Blob>)> {
    READY.with_borrow(|ready| {
        ready
            .as_ref()
            .filter(|r| r.path == path)
            .map(|r| (r.file.clone(), r.png.clone()))
    })
}

/// Reads the copy ahead of a pick, and makes the PNG when `png`.
fn warm(picture: &Picture, png: bool) {
    let done = READY.with_borrow(|ready| {
        ready
            .as_ref()
            .is_some_and(|r| r.path == picture.path && (r.png.is_some() || !png))
    });
    if done {
        return;
    }
    let picture = picture.clone();
    spawn_local(async move {
        // A pick reads it again and says what went wrong.
        let Ok(file) = read(&picture).await else {
            return;
        };
        let png = if png { to_png(&file).await } else { None };
        READY.with_borrow_mut(|ready| {
            *ready = Some(Ready {
                path: picture.path,
                file,
                png,
            });
        });
    });
}

/// The copy, from this server. A failure comes back as words to show.
async fn read(picture: &Picture) -> Result<File, String> {
    let path = picture.path.clone();
    let bytes =
        api::attempt(move |c| async move { c.media_bytes(&path, reverse::MAX_BYTES).await })
            .await
            .map_err(|refused| match refused.status {
                // A signed link lasts hours; the page may have been open longer.
                Some(403 | 404 | 410) => t!("download-expired"),
                _ => t!("reverse-read-failed", reason = refused.message),
            })?;
    let parts = js_sys::Array::of1(&js_sys::Uint8Array::from(bytes.as_slice()));
    let options = web_sys::FilePropertyBag::new();
    options.set_type(picture.mime);
    File::new_with_u8_array_sequence_and_options(&parts, &picture.name, &options)
        .map_err(|_| t!("reverse-unreadable"))
}

/// `file` as PNG, the one picture type every browser's clipboard takes.
async fn to_png(file: &File) -> Option<Blob> {
    if file.type_() == "image/png" {
        return Some(file.clone().into());
    }
    let window = web_sys::window()?;
    let decoded = window.create_image_bitmap_with_blob(file).ok()?;
    let bitmap: web_sys::ImageBitmap = JsFuture::from(decoded).await.ok()?.dyn_into().ok()?;
    let canvas: web_sys::HtmlCanvasElement = window
        .document()?
        .create_element("canvas")
        .ok()?
        .dyn_into()
        .ok()?;
    canvas.set_width(bitmap.width());
    canvas.set_height(bitmap.height());
    let context: web_sys::CanvasRenderingContext2d =
        canvas.get_context("2d").ok()??.dyn_into().ok()?;
    let drawn = context.draw_image_with_image_bitmap(&bitmap, 0.0, 0.0);
    bitmap.close();
    drawn.ok()?;
    let encoded = js_sys::Promise::new(&mut |resolve, reject| {
        if canvas.to_blob_with_type(&resolve, "image/png").is_err() {
            let _ = reject.call0(&JsValue::NULL);
        }
    });
    JsFuture::from(encoded).await.ok()?.dyn_into().ok()
}
