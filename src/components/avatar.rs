//! An account's picture on a disc, or where it has none, or the picture
//! doesn't load, the first letter of its name on the accent.

use leptos::prelude::*;
use starbooru_core::UserCard;

use super::icons::{Ico, Icon};
use crate::logic::profile::{badge, shown_name};
use crate::{api, route};

/// How big the disc is (`style/base.css`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    /// 28 px, in lines of text.
    Small,
    /// 40 px.
    #[default]
    Normal,
    /// 56 px.
    Big,
    /// 112 px, at the head of a profile.
    Huge,
}

impl Size {
    fn class(self) -> &'static str {
        match self {
            Size::Small => "avatar small",
            Size::Normal => "avatar",
            Size::Big => "avatar big",
            Size::Huge => "avatar huge",
        }
    }
}

/// The disc for `name` (the username) and `display`, with the picture at
/// `url` (signed and server-relative, as the API gives it). Decorative: the
/// name beside it says whose it is.
#[component]
pub fn Avatar(
    #[prop(into)] name: Signal<String>,
    #[prop(optional, into)] display: Signal<Option<String>>,
    #[prop(optional, into)] url: Signal<Option<String>>,
    #[prop(optional)] size: Size,
) -> impl IntoView {
    // A picture whose address expired or broke: the letter stands in.
    let broken = RwSignal::new(None::<String>);
    let src = Memo::new(move |_| {
        url.get()
            .filter(|u| !u.is_empty() && broken.with(|b| b.as_deref() != Some(u.as_str())))
    });
    let letter = move || name.with(|n| display.with(|d| badge(n, d.as_deref())));
    view! {
        <span class=size.class() class:pictured=move || src.with(Option::is_some) aria-hidden="true">
            {move || match src.get() {
                Some(path) => {
                    let failed = path.clone();
                    view! {
                        <img
                            src=api::media(&path)
                            alt=""
                            decoding="async"
                            on:error=move |_| broken.set(Some(failed.clone()))
                        />
                    }
                        .into_any()
                }
                None => view! { <span class="avatar-letter">{letter}</span> }.into_any(),
            }}
        </span>
    }
}

/// The disc of whoever a card names, at its small picture for discs under
/// 64 px.
#[component]
pub fn CardAvatar(card: UserCard, #[prop(optional)] size: Size) -> impl IntoView {
    let url = match size {
        Size::Small | Size::Normal => card.avatar_small_url.or(card.avatar_url),
        Size::Big | Size::Huge => card.avatar_url.or(card.avatar_small_url),
    };
    view! { <Avatar name=card.name display=card.display_name url=url size=size /> }
}

/// Who did something: their picture and name, leading to their profile. An
/// account deleted since shows as such.
#[component]
pub fn UserLink(card: Option<UserCard>, #[prop(optional)] size: Size) -> impl IntoView {
    match card {
        Some(card) => {
            let name = shown_name(&card.name, card.display_name.as_deref());
            let href = route::profile(&card.name);
            view! {
                <a class="author" href=href>
                    <CardAvatar card=card size=size />
                    <span class="author-name break">{name}</span>
                </a>
            }
            .into_any()
        }
        None => view! {
            <span class="author gone">
                <span class=size.class() aria-hidden="true">
                    <Ico icon=Icon::User size=16 />
                </span>
                <span class="author-name">{move || t!("forum-author-gone")}</span>
            </span>
        }
        .into_any(),
    }
}
