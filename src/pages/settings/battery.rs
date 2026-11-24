//! Settings › Battery and effects: the battery saver and the visual
//! effects, each with what it changes, lit up while it's in effect.

use leptos::prelude::*;

use super::controls::{choices, effects_list};
use crate::components::icons::Icon;
use crate::store::{self, Effects, Saver};
use crate::{pace, power};

fn saver_label(saver: Saver) -> String {
    match saver {
        Saver::Auto => t!("saver-auto"),
        Saver::Always => t!("saver-always"),
        Saver::Never => t!("saver-never"),
    }
}

fn effects_label(effects: Effects) -> String {
    match effects {
        Effects::Auto => t!("effects-auto"),
        Effects::Full => t!("effects-full"),
        Effects::Lite => t!("effects-lite"),
    }
}

/// Settings › Battery and effects.
#[component]
pub(super) fn BatteryGroup() -> impl IntoView {
    let saver = Memo::new(|_| store::prefs().battery_saver);
    let effects = Memo::new(|_| store::prefs().effects);
    let reduced = Memo::new(|_| store::prefs().effects_reduced);

    // What Automatic goes by, while it's the choice; and when it has
    // turned the saver on, or the effects down, that it has.
    let saver_auto = move || {
        (saver.get() == Saver::Auto).then(|| {
            if power::device_low() {
                view! { <p class="hint saver-now">{move || t!("settings-saver-now")}</p> }
                    .into_any()
            } else {
                view! { <p class="muted fine">{move || t!("settings-saver-auto")}</p> }.into_any()
            }
        })
    };
    let effects_auto = move || {
        (effects.get() == Effects::Auto).then(|| {
            if reduced.get() {
                view! { <p class="hint saver-now">{move || t!("settings-effects-now")}</p> }
                    .into_any()
            } else {
                view! { <p class="muted fine">{move || t!("settings-effects-auto")}</p> }.into_any()
            }
        })
    };

    view! {
        <section class="card" id="battery-saver">
            <h3 class="card-title">{move || t!("settings-saver")}</h3>
            {choices(
                || t!("settings-saver"),
                &Saver::ALL,
                saver,
                saver_label,
                |p, s| p.battery_saver = s,
            )}
            {saver_auto}
            {effects_list(
                || t!("settings-saver-does"),
                [
                    (Icon::Play, || t!("saver-does-videos")),
                    (Icon::Download, || t!("saver-does-preload")),
                    (Icon::Sparkle, || t!("saver-does-motion")),
                ],
                power::active,
            )}
        </section>
        <section class="card" id="effects">
            <h3 class="card-title">{move || t!("settings-effects")}</h3>
            // Choosing again, Automatic too, measures the device afresh.
            {choices(
                || t!("settings-effects"),
                &Effects::ALL,
                effects,
                effects_label,
                |p, e| {
                    p.effects = e;
                    p.effects_reduced = false;
                },
            )}
            {effects_auto}
            {effects_list(
                || t!("settings-effects-does"),
                [
                    (Icon::Layout, || t!("effects-does-glass")),
                    (Icon::Gallery, || t!("effects-does-thumbs")),
                    (Icon::Star, || t!("effects-does-glow")),
                ],
                pace::lite,
            )}
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_choice_has_its_words() {
        assert_eq!(saver_label(Saver::Auto), "Automatic");
        assert_eq!(effects_label(Effects::Lite), "Reduced");
        for s in Saver::ALL {
            assert!(!saver_label(s).is_empty());
        }
        for e in Effects::ALL {
            assert!(!effects_label(e).is_empty());
        }
    }

    #[test]
    fn the_lists_light_up_with_what_is_in_effect() {
        let page = include_str!("battery.rs");
        assert!(page.contains("                power::active,\n"));
        assert!(page.contains("                pace::lite,\n"));
        // Choosing the effects again measures the device afresh.
        assert!(page.contains("p.effects_reduced = false;"));
    }
}
