//! Administration › Server › Link previews: whether chats and sites show a
//! shared address with its picture. On a public server it's every post's
//! address; on a private one it lets each account choose for its share
//! links. It saves as soon as it's flipped.

use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{Mode, ServerSettings};

use super::server::save_part;
use crate::components::banner::ErrorBanner;
use crate::components::icons::{Ico, Icon};
use crate::components::link_card::LinkCardPicture;
use crate::instance;

#[component]
pub(super) fn PreviewsCard(settings: RwSignal<Option<ServerSettings>>) -> impl IntoView {
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let saved = RwSignal::new(false);
    let on = Memo::new(move |_| {
        settings.with(|s| s.as_ref().and_then(|s| s.link_previews).unwrap_or(false))
    });
    let public =
        Memo::new(move |_| settings.with(|s| s.as_ref().is_some_and(|s| s.mode == Mode::Public)));
    let flip = move |_| {
        if busy.get_untracked() || settings.with_untracked(Option::is_none) {
            return;
        }
        let next = !on.get_untracked();
        busy.set(true);
        error.set(None);
        saved.set(false);
        spawn_local(async move {
            match save_part(settings, |s| s.link_previews = Some(next)).await {
                Ok(s) => {
                    instance::settled_previews(s.link_previews.unwrap_or(false));
                    saved.try_set(true);
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
            busy.try_set(false);
        });
    };
    let name = move || {
        if public.get() {
            t!("admin-previews-public")
        } else {
            t!("admin-previews-private")
        }
    };
    let hint = move || {
        if public.get() {
            t!("admin-previews-public-hint")
        } else {
            t!("admin-previews-private-hint")
        }
    };

    view! {
        <section class="card previews-card" id="previews">
            <h2 class="card-title">{move || t!("previews-title")}</h2>
            <div class="previews-body">
                <LinkCardPicture on=on />
                <div class="previews-side">
                    <button
                        type="button"
                        class="list-link switch-row"
                        role="switch"
                        aria-checked=move || if on.get() { "true" } else { "false" }
                        aria-labelledby="admin-previews-name"
                        aria-describedby="admin-previews-hint"
                        disabled=move || busy.get() || settings.with(Option::is_none)
                        on:click=flip
                    >
                        <span id="admin-previews-name">{name}</span>
                        <span id="admin-previews-hint" class="muted">
                            {hint}
                        </span>
                        <span class="switch" class:on=move || on.get() aria-hidden="true">
                            <span class="switch-knob">
                                {move || {
                                    let icon = if on.get() { Icon::Check } else { Icon::Close };
                                    view! { <Ico icon=icon size=16 /> }
                                }}
                            </span>
                        </span>
                    </button>
                    <p class="muted fine">{move || t!("previews-copies")}</p>
                    <Show when=move || saved.get()>
                        <span class="muted saved-note" role="status">
                            <Ico icon=Icon::Check size=18 />
                            {move || t!("admin-saved")}
                        </span>
                    </Show>
                </div>
            </div>
            {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
        </section>
    }
}
