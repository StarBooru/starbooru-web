//! StarBooru for the browser: a Leptos single-page app over the StarBooru API.

#[macro_use]
mod i18n;
mod activity;
mod api;
mod app;
mod components;
mod dom;
mod files;
mod install;
mod logic;
mod oidc;
mod pace;
mod pages;
mod power;
mod route;
mod store;
mod theme;
mod updates;

use wasm_bindgen::JsCast;

fn main() {
    store::load();
    // Before the first paint, so the app starts in its language.
    let lang = i18n::resolve(
        store::prefs_untracked().language,
        &i18n::browser_languages(),
    );
    i18n::set_lang(lang);
    dom::manual_scroll_restoration();
    // Into `#main`, so the loading screen after it knows when to go.
    let root = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("main"))
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());
    match root {
        Some(root) => leptos::mount::mount_to(root, app::App).forget(),
        None => leptos::mount::mount_to_body(app::App),
    }
}
