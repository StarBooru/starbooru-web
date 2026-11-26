//! A small picture of a link sent in a chat. With a preview the link's
//! picture and title grow in above the address; without, the address stands
//! alone.

use leptos::prelude::*;

#[component]
pub fn LinkCardPicture(
    #[prop(into)] on: Signal<bool>,
    /// An icon's size, for a row.
    #[prop(optional)]
    small: bool,
) -> impl IntoView {
    view! {
        <span class="link-card" class:on=move || on.get() class:small=small aria-hidden="true">
            <span class="link-card-bubble">
                <span class="link-card-preview">
                    <span>
                        <span class="link-card-image"></span>
                        <span class="link-card-site"></span>
                        <span class="link-card-title"></span>
                    </span>
                </span>
                <span class="link-card-url"></span>
            </span>
        </span>
    }
}
