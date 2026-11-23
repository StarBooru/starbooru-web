//! The menu a right-clicked tag opens, on any page that lists tags as links
//! with `data-tag` under `data-press="tag"`: search for it, add it to the
//! search or leave it out, open its page, copy its name. A long press on a
//! phone opens it too where the browser reports one as a right click. It
//! grows out of the point it was opened at.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
use send_wrapper::SendWrapper;
use starbooru_core::normalize_tag;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::MouseEvent;

use super::icons::Icon;
use super::menu::{self, ActionMenu, MenuAt, MenuItem};
use super::snackbar::notify;
use crate::dom;
use crate::logic::tags::{exclude, include, words};
use crate::route;

/// The tag a right click was on, and where.
fn pressed(e: &MouseEvent) -> Option<(String, MenuAt)> {
    let (link, at) = menu::pressed(e, r#"[data-press="tag"] [data-tag]"#)?;
    let name = normalize_tag(&link.get_attribute("data-tag")?).ok()?;
    Some((name, at))
}

/// Listens for right clicks on tags while it's on the page, and shows their
/// menu. `q` is the search the page belongs to, which a tag can narrow.
#[component]
pub fn TagPress(#[prop(into)] q: Signal<String>) -> impl IntoView {
    let menu = RwSignal::new(None::<(String, MenuAt)>);
    let watch = web_sys::window().and_then(|w| w.document()).map(|doc| {
        dom::listen(&doc, "contextmenu", false, move |e| {
            let Some(found) = e.dyn_ref::<MouseEvent>().and_then(pressed) else {
                return;
            };
            e.prevent_default();
            menu.set(Some(found));
        })
    });
    on_cleanup(move || {
        if let Some(watch) = watch {
            watch.stop();
        }
    });

    let navigate = SendWrapper::new(use_navigate());
    let go = Callback::new(move |to: String| {
        menu.set(None);
        (*navigate)(&to, NavigateOptions::default());
    });
    let close = Callback::new(move |()| menu.set(None));
    move || {
        let (name, at) = menu.get()?;
        let q = q.get_untracked();
        let searching = !q.trim().is_empty();
        let has = words(&q).contains(&name.as_str());
        let search = route::gallery(&name);
        let narrowed = (searching && !has).then(|| route::gallery(&include(&q, &name)));
        let without = route::gallery(&exclude(if searching { &q } else { "" }, &name));
        let details = route::tag(&name);
        let copied = name.clone();
        Some(view! {
            <ActionMenu at=at title=name on_close=close>
                <MenuItem
                    icon=Icon::Search
                    label=move || t!("tag-menu-search")
                    on_click=Callback::new(move |()| go.run(search.clone()))
                />
                {narrowed
                    .map(|to| {
                        view! {
                            <MenuItem
                                icon=Icon::Add
                                label=move || t!("tag-menu-add")
                                on_click=Callback::new(move |()| go.run(to.clone()))
                            />
                        }
                    })}
                <MenuItem
                    icon=Icon::Remove
                    label=move || t!("tag-menu-exclude")
                    on_click=Callback::new(move |()| go.run(without.clone()))
                />
                <MenuItem
                    icon=Icon::Label
                    label=move || t!("tag-menu-details")
                    on_click=Callback::new(move |()| go.run(details.clone()))
                />
                <MenuItem
                    icon=Icon::Copy
                    label=move || t!("tag-menu-copy")
                    on_click=Callback::new(move |()| {
                        menu.set(None);
                        copy_name(copied.clone());
                    })
                />
            </ActionMenu>
        })
    }
}

/// Copies `name` and says so.
fn copy_name(name: String) {
    spawn_local(async move {
        let done = copy_text(&name).await;
        notify(if done {
            t!("tag-copied")
        } else {
            t!("tag-copy-failed")
        });
    });
}

/// Puts `text` on the clipboard. A page on plain http, other than on this
/// computer, has no clipboard API, and the old way still works there.
async fn copy_text(text: &str) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let navigator = window.navigator();
    let api = js_sys::Reflect::get(&navigator, &JsValue::from_str("clipboard"))
        .is_ok_and(|c| !c.is_undefined() && !c.is_null());
    if api {
        let promise = navigator.clipboard().write_text(text);
        if wasm_bindgen_futures::JsFuture::from(promise).await.is_ok() {
            return true;
        }
    }
    copy_by_selection(text).unwrap_or(false)
}

fn copy_by_selection(text: &str) -> Option<bool> {
    let doc = web_sys::window()?.document()?;
    let field = doc
        .create_element("textarea")
        .ok()?
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .ok()?;
    field.set_value(text);
    let _ = field.set_attribute("style", "position: fixed; opacity: 0");
    doc.body()?.append_child(&field).ok()?;
    field.select();
    let done = doc
        .dyn_ref::<web_sys::HtmlDocument>()
        .and_then(|d| d.exec_command("copy").ok());
    field.remove();
    done
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_menu_opens_for_tags_under_a_press_area() {
        let source = include_str!("tag_menu.rs");
        assert!(source.contains(r#"[data-press="tag"] [data-tag]"#));
    }
}
