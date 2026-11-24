//! Settings › Viewing and gestures: how thumbnails sit, what the viewer
//! opens and keeps, the People card, what a swipe does, and the hand the
//! app is held in. Each picture here shows the choice it stands for.

use leptos::prelude::*;

use super::controls::{pick, switch_row};
use crate::components::icons::{Ico, Icon};
use crate::components::select::{Choice, Select};
use crate::store::{self, Hand, Prefs, SwipeAction};

fn hand_label(hand: Hand) -> String {
    match hand {
        Hand::Either => t!("hand-either"),
        Hand::Left => t!("hand-left"),
        Hand::Right => t!("hand-right"),
    }
}

fn swipe_label(action: SwipeAction) -> String {
    match action {
        SwipeAction::Off => t!("swipe-off"),
        SwipeAction::Like => t!("swipe-like"),
        SwipeAction::Select => t!("swipe-select"),
        SwipeAction::Share => t!("share-button"),
        SwipeAction::Download => t!("post-download"),
        SwipeAction::Delete => t!("swipe-delete"),
    }
}

/// The icon a swipe uncovers, as the gallery's tiles show it.
fn swipe_icon(action: SwipeAction) -> Icon {
    match action {
        SwipeAction::Off => Icon::Close,
        SwipeAction::Like => Icon::ThumbUp,
        SwipeAction::Select => Icon::Check,
        SwipeAction::Share => Icon::Share,
        SwipeAction::Download => Icon::Download,
        SwipeAction::Delete => Icon::Delete,
    }
}

/// Six thumbnails, spaced or edge to edge.
fn tiles(flush: bool) -> impl IntoView {
    view! {
        <span class="tiles-demo" class:flush=flush>
            <span></span>
            <span></span>
            <span></span>
            <span></span>
            <span></span>
            <span></span>
        </span>
    }
}

/// A tablet with the side navigation where the hand puts it, and the hand.
fn tablet(hand: Hand) -> impl IntoView {
    view! {
        <span class="hand-demo" data-hand=hand.as_str()>
            <span class="demo-rail">
                <span></span>
                <span></span>
                <span></span>
            </span>
            <span class="demo-page"></span>
            <span class="demo-hand">
                <Ico icon=Icon::Hand size=22 />
            </span>
        </span>
    }
}

/// A thumbnail pushed aside, uncovering what the swipe does. It plays the
/// swipe again whenever the choice changes.
fn swipe_demo(side: &'static str, action: SwipeAction) -> impl IntoView {
    view! {
        <span class="swipe-demo" data-side=side data-action=action.as_str() aria-hidden="true">
            <span class="swipe-demo-hint">
                <Ico icon=swipe_icon(action) size=20 />
            </span>
            <span class="swipe-demo-tile"></span>
        </span>
    }
}

#[component]
fn GesturesCard() -> impl IntoView {
    let options = Signal::derive(|| {
        SwipeAction::ALL
            .into_iter()
            .map(|a| Choice::new(a.as_str(), swipe_label(a)))
            .collect::<Vec<_>>()
    });
    let side = |side: &'static str,
                name: fn() -> String,
                get: fn(&Prefs) -> SwipeAction,
                set: fn(&mut Prefs, SwipeAction)| {
        let action = Memo::new(move |_| get(&store::prefs()));
        view! {
            <div class="swipe-side">
                {move || swipe_demo(side, action.get())}
                <div class="field">
                    <span aria-hidden="true">{name}</span>
                    <Select
                        value=Signal::derive(move || action.get().as_str().to_owned())
                        options=options
                        label=Signal::derive(name)
                        on_change=Callback::new(move |v: String| {
                            if let Some(action) = SwipeAction::parse(&v) {
                                store::update_prefs(|p| set(p, action));
                            }
                        })
                    />
                </div>
            </div>
        }
    };
    view! {
        <section class="card" id="gestures">
            <h3 class="card-title">{move || t!("settings-gestures")}</h3>
            <div class="swipe-sides">
                {side("left", || t!("settings-swipe-left"), |p| p.swipe_left, |p, a| p.swipe_left = a)}
                {side("right", || t!("settings-swipe-right"), |p| p.swipe_right, |p, a| p.swipe_right = a)}
            </div>
            <p class="muted fine">{move || t!("settings-hold-hint")}</p>
        </section>
    }
}

/// Settings › Viewing and gestures.
#[component]
pub(super) fn ViewingGroup() -> impl IntoView {
    let flush = Memo::new(|_| store::prefs().flush_tiles);
    let hand = Memo::new(|_| store::prefs().hand);
    let originals = Memo::new(|_| store::prefs().originals);
    let keep = Memo::new(|_| store::prefs().keep_original);
    let people = Memo::new(|_| store::prefs().people);

    let spacing = [
        (false, (|| t!("settings-tiles-spaced")) as fn() -> String),
        (true, || t!("settings-tiles-flush")),
    ]
    .into_iter()
    .map(|(value, label)| {
        pick(
            label,
            move || flush.get() == value,
            move || store::update_prefs(|p| p.flush_tiles = value),
            tiles(value),
        )
    })
    .collect_view();
    let hands = Hand::ALL
        .into_iter()
        .map(|value| {
            pick(
                move || hand_label(value),
                move || hand.get() == value,
                move || store::update_prefs(|p| p.hand = value),
                tablet(value),
            )
        })
        .collect_view();

    view! {
        <section class="card" id="thumbnails">
            <h3 class="card-title">{move || t!("settings-tiles")}</h3>
            <div class="picks two" role="radiogroup" aria-label=move || t!("settings-tiles")>
                {spacing}
            </div>
        </section>
        <section class="card switches" id="viewer">
            <h3 class="card-title">{move || t!("settings-viewer")}</h3>
            {switch_row(
                "originals-switch",
                Icon::ZoomIn,
                || t!("settings-originals"),
                || t!("settings-originals-note"),
                originals,
                |p| p.originals = !p.originals,
            )}
            {switch_row(
                "keep-switch",
                Icon::Crop,
                || t!("settings-keep-original"),
                || t!("settings-keep-original-note"),
                keep,
                |p| p.keep_original = !p.keep_original,
            )}
            // People on or off. Off only hides the card: posts keep their people.
            {switch_row(
                "people-switch",
                Icon::People,
                || t!("settings-people"),
                || t!("settings-people-hint"),
                people,
                |p| p.people = !p.people,
            )}
        </section>
        <GesturesCard />
        <section class="card" id="hand">
            <h3 class="card-title">{move || t!("settings-hand")}</h3>
            <div class="picks" role="radiogroup" aria-label=move || t!("settings-hand")>
                {hands}
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_choice_has_its_words_and_its_picture() {
        assert_eq!(hand_label(Hand::Either), "Either");
        let labels: std::collections::BTreeSet<String> =
            SwipeAction::ALL.into_iter().map(swipe_label).collect();
        assert_eq!(labels.len(), SwipeAction::ALL.len());
        // The demo uncovers what a tile does, with the gallery's icon.
        let tile = include_str!("../../components/thumb.rs");
        for action in SwipeAction::ALL {
            let icon = format!("{:?}", swipe_icon(action));
            let arm = format!("SwipeAction::{action:?} => Icon::{icon},");
            assert!(tile.contains(&arm), "{arm}");
        }
    }

    #[test]
    fn the_switches_and_pictures_keep_the_stored_settings() {
        let page = include_str!("viewing.rs");
        for set in [
            "p.flush_tiles = value",
            "p.originals = !p.originals",
            "p.keep_original = !p.keep_original",
            "p.people = !p.people",
            "p.hand = value",
            "p.swipe_left = a",
            "p.swipe_right = a",
        ] {
            assert!(page.contains(set), "{set}");
        }
        let css = include_str!("../../../style/pages/settings.css");
        // The demo tablet puts its rail where the app does.
        assert!(css.contains(r#".hand-demo[data-hand="right"] .demo-rail"#));
        assert!(css.contains(r#".swipe-demo[data-action="delete"]"#));
    }
}
