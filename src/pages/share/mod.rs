//! Share links: the sheet that makes one for a post or a selection
//! (`sheet.rs`), the account's links for Settings (`links.rs`), and the
//! public page a link opens without an account (`public.rs`).

mod links;
mod public;
mod sheet;

use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;

pub use self::links::SharesCard;
pub use self::public::SharedPage;
pub use self::sheet::ShareSheet;
use crate::components::icons::{Ico, Icon};
use crate::dom;

/// A link to give out, in a field to select, with a button that copies it.
/// Where the clipboard won't take it, the link is selected to copy by hand.
#[component]
fn LinkField(url: String, #[prop(optional)] focus: bool) -> impl IntoView {
    let copied = RwSignal::new(false);
    let failed = RwSignal::new(false);
    let field: NodeRef<html::Input> = NodeRef::new();
    let button: NodeRef<html::Button> = NodeRef::new();
    if focus {
        button.on_load(|el| {
            request_animation_frame(move || {
                let _ = el.focus();
            });
        });
    }
    let select = move || {
        if let Some(el) = field.get_untracked() {
            let _ = el.focus();
            el.select();
        }
    };
    let text = url.clone();
    let copy = move |_| {
        let text = text.clone();
        spawn_local(async move {
            let done = dom::copy_text(&text).await;
            copied.try_set(done);
            failed.try_set(!done);
            if !done {
                select();
            }
        });
    };
    view! {
        <div class="key-token">
            <input
                node_ref=field
                readonly=true
                spellcheck="false"
                aria-label=move || t!("share-link")
                prop:value=url
                on:focus=move |_| {
                    if let Some(el) = field.get_untracked() {
                        el.select();
                    }
                }
            />
            <button
                node_ref=button
                class="btn tonal small"
                type="button"
                title=move || t!("share-copy")
                on:click=copy
            >
                {move || {
                    let icon = if copied.get() { Icon::Check } else { Icon::Copy };
                    view! { <Ico icon=icon size=18 /> }
                }}
                <span class="copy-label">
                    {move || if copied.get() { t!("share-copied") } else { t!("share-copy") }}
                </span>
            </button>
        </div>
        <Show when=move || failed.get()>
            <p class="field-error" role="alert">
                {move || t!("share-copy-failed")}
            </p>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_sheet_comes_from_every_place_the_reference_shares() {
        let viewer = include_str!("../post/mod.rs");
        let menu = include_str!("../post_menu.rs");
        let bulk = include_str!("../bulk.rs");
        for (name, source) in [("viewer", viewer), ("menu", menu), ("bulk bar", bulk)] {
            assert!(source.contains("<ShareSheet"), "{name}");
        }
        // The list stands where Settings will have it.
        assert!(include_str!("../soon.rs").contains("<SharesCard"));
        // The public page is outside the shell and asks no session's client.
        let app = include_str!("../../app.rs");
        assert!(app.contains(r#"<Route path=path!("/s/:token") view=SharedPage />"#));
        let public = include_str!("public.rs");
        assert!(!public.contains("api::client") && !public.contains("api::call"));
        assert!(!public.contains("with_token"));
    }
}
