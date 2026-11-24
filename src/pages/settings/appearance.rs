//! Settings › Appearance: the theme with its black background, the accent
//! and the language, what this browser keeps for itself. Each choice is
//! saved the moment it's made and shows at once, in the pictures here too.

use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;
use starbooru_core::color::{Rgb, to_hex};

use wasm_bindgen::JsCast;

use super::controls::{pick, segment};
use crate::components::color_picker::ColorPicker;
use crate::components::focus::hold_focus;
use crate::components::icons::{Ico, Icon};
use crate::i18n::Lang;
use crate::logic::settings::{
    NEUTRALS, SWATCHES, accent_rgb, custom_in_use, neutral_name, oled_applies, pick as pick_hue,
    pick_color, picked, swatch_name,
};
use crate::store::{self, Prefs, Theme};
use crate::{dom, theme};

fn theme_label(theme: Theme) -> String {
    match theme {
        Theme::System => t!("theme-system"),
        Theme::Light => t!("theme-light"),
        Theme::Dark => t!("theme-dark"),
    }
}

/// A language's name in that language, as it's looked for.
fn lang_name(lang: Lang) -> String {
    match lang {
        Lang::English => t!("lang-english"),
        Lang::Italian => t!("lang-italian"),
    }
}

/// A small window of the app in a theme's colours, the accent's included.
fn window(scheme: &'static str, oled: Memo<bool>) -> impl IntoView {
    view! {
        <span class=format!("theme-demo {scheme}") class:oled=move || oled.get()>
            <span class="demo-bar"></span>
            <span class="demo-tiles">
                <span></span>
                <span class="accent-tile"></span>
                <span></span>
                <span></span>
                <span></span>
                <span></span>
            </span>
            <span class="demo-nav"></span>
        </span>
    }
}

/// The theme as three small windows, System split between the two, with
/// the black background under them.
#[component]
fn ThemeCard() -> impl IntoView {
    let theme = Memo::new(|_| store::prefs().theme);
    let oled = Memo::new(|_| store::prefs().oled);
    let picks = Theme::ALL
        .into_iter()
        .map(|value| {
            let picture = match value {
                Theme::Light => window("scheme-light", oled).into_any(),
                Theme::Dark => window("scheme-dark", oled).into_any(),
                Theme::System => view! {
                    <span class="theme-split">
                        {window("scheme-light", oled)}
                        {window("scheme-dark", oled)}
                    </span>
                }
                .into_any(),
            };
            pick(
                move || theme_label(value),
                move || theme.get() == value,
                move || store::update_prefs(|p| p.theme = value),
                picture,
            )
        })
        .collect_view();
    view! {
        <section class="card" id="theme">
            <h3 class="card-title">{move || t!("settings-theme")}</h3>
            <div class="picks" role="radiogroup" aria-label=move || t!("settings-theme")>
                {picks}
            </div>
            <OledSwitch theme=theme />
        </section>
    }
}

/// Settings › Appearance.
#[component]
pub(super) fn AppearanceGroup() -> impl IntoView {
    let language = Memo::new(|_| store::prefs().language);
    let languages = Lang::ALL
        .into_iter()
        .map(|lang| {
            segment(
                move || lang_name(lang),
                move || language.get() == Some(lang),
                move || store::update_prefs(|p| p.language = Some(lang)),
                Some(lang.code()),
            )
        })
        .collect_view();

    view! {
        <ThemeCard />
        <section class="card" id="accent">
            <h3 class="card-title">{move || t!("settings-accent")}</h3>
            <Accents />
        </section>
        <section class="card" id="language">
            <h3 class="card-title">{move || t!("settings-language")}</h3>
            <div class="segmented" role="radiogroup" aria-label=move || t!("settings-language")>
                {segment(
                    || t!("settings-language-system"),
                    move || language.get().is_none(),
                    || store::update_prefs(|p| p.language = None),
                    None,
                )}
                {languages}
            </div>
        </section>
    }
}

/// The black background, under the theme it belongs to. Light has no dark to
/// black out, so the switch greys out while Light is picked, keeping its
/// state for when it's back.
#[component]
fn OledSwitch(theme: Memo<Theme>) -> impl IntoView {
    let on = Memo::new(|_| store::prefs().oled);
    let idle = move || !oled_applies(theme.get());
    view! {
        <button
            type="button"
            class="oled-switch"
            role="switch"
            aria-checked=move || if on.get() { "true" } else { "false" }
            aria-labelledby="oled-switch-name"
            aria-describedby="oled-switch-hint"
            disabled=idle
            on:click=move |_| store::update_prefs(|p| p.oled = !p.oled)
        >
            <span class="oled-switch-text">
                <span id="oled-switch-name">{move || t!("settings-oled")}</span>
                <span id="oled-switch-hint" class="muted">
                    {move || t!("settings-oled-hint")}
                </span>
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
    }
}

/// The presets, then the picker for any other colour.
#[component]
fn Accents() -> impl IntoView {
    let prefs = Memo::new(|_| {
        let p = store::prefs();
        (p.accent_hue, p.accent_color)
    });
    let current = move || {
        let (accent_hue, accent_color) = prefs.get();
        Prefs {
            accent_hue,
            accent_color,
            ..Prefs::default()
        }
    };
    let swatches = SWATCHES
        .into_iter()
        .map(|hue| {
            let on = move || picked(&current(), hue);
            view! {
                <button
                    type="button"
                    class="accent"
                    class:on=on
                    style=format!("--swatch-hue: {hue}")
                    aria-pressed=move || if on() { "true" } else { "false" }
                    aria-label=move || swatch_name(hue)
                    title=move || swatch_name(hue)
                    on:click=move |_| store::update_prefs(|p| pick_hue(p, hue))
                >
                    <Show when=on>
                        <Ico icon=Icon::Check size=18 />
                    </Show>
                </button>
            }
        })
        .collect_view();
    view! {
        <div class="accents">
            {swatches}
            <AccentPicker
                value=Signal::derive(move || accent_rgb(&current()))
                on=Signal::derive(move || custom_in_use(&current()))
            />
        </div>
    }
}

/// The last choice of the accent: the colour picker, behind a button that
/// shows the colour when it's the one in use, white and black too.
#[component]
fn AccentPicker(value: Signal<Rgb>, on: Signal<bool>) -> impl IntoView {
    let open = RwSignal::new(false);
    let close_on_escape = move |e: KeyboardEvent| {
        if open.get_untracked() && e.key() == "Escape" {
            e.stop_propagation();
            open.set(false);
        }
    };
    let label = move || t!("settings-accent-custom");
    view! {
        <div class="color-pick" on:keydown=close_on_escape>
            <button
                type="button"
                class="accent custom"
                class:on=move || on.get()
                class:pale=move || on.get() && theme::dark_ink(value.get())
                style=move || {
                    on.get().then(|| format!("background-color: #{}", to_hex(value.get())))
                }
                title=label
                aria-label=label
                aria-haspopup="dialog"
                aria-expanded=move || if open.get() { "true" } else { "false" }
                on:click=move |_| open.update(|o| *o = !*o)
            >
                <Ico icon=Icon::Palette size=20 />
            </button>
            <Show when=move || open.get()>
                <div class="select-backdrop" on:click=move |_| open.set(false)></div>
                <AccentMenu on_close=Callback::new(move |()| open.set(false)) />
            </Show>
        </div>
    }
}

/// The picker itself, on the colour in use, with the greys the wheel can't
/// make under it. It takes the focus, keeps Tab inside, and gives the focus
/// back to its button when it closes.
#[component]
fn AccentMenu(on_close: Callback<()>) -> impl IntoView {
    let menu: NodeRef<html::Div> = NodeRef::new();
    hold_focus(menu, true);
    // Escape closes it wherever the focus went; inside, `.color-pick` hears
    // it first.
    let keys = web_sys::window().map(|window| {
        dom::listen(&window, "keydown", false, move |e| {
            let escape = e
                .dyn_ref::<web_sys::KeyboardEvent>()
                .is_some_and(|k| k.key() == "Escape");
            if escape && !e.default_prevented() {
                e.prevent_default();
                on_close.try_run(());
            }
        })
    });
    on_cleanup(move || {
        // It can close from inside this very listener, which can't be
        // dropped while it runs.
        set_timeout(
            move || {
                if let Some(keys) = keys {
                    keys.stop();
                }
            },
            std::time::Duration::ZERO,
        );
    });
    // Under its button, as far as the window allows: the row of colours
    // wraps, so the button can be anywhere across it. The page scrolls to
    // show all of it, the greys at the foot too. The hex field takes the
    // focus, as it's how the keyboard picks.
    menu.on_load(|el| {
        if let Some(button) = el.parent_element() {
            let room = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.document_element())
                .map_or(0.0, |d| f64::from(d.client_width()));
            let b = button.get_bounding_client_rect();
            let width = f64::from(el.offset_width());
            let middle = (b.left() + b.right()) / 2.0;
            let left = (middle - width / 2.0).min(room - width - 8.0).max(8.0);
            let style = web_sys::HtmlElement::style(&el);
            let _ = style.set_property("left", &format!("{:.0}px", left - b.left()));
            let _ = style.set_property("transform-origin", &format!("{:.0}px top", middle - left));
        }
        let options = web_sys::ScrollIntoViewOptions::new();
        options.set_block(web_sys::ScrollLogicalPosition::Nearest);
        el.scroll_into_view_with_scroll_into_view_options(&options);
        let field = el
            .query_selector("input")
            .ok()
            .flatten()
            .and_then(|f| f.dyn_into::<web_sys::HtmlElement>().ok());
        if let Some(field) = field {
            let _ = field.focus();
        }
    });
    let chosen = Callback::new(move |rgb: Rgb| store::update_prefs(|p| pick_color(p, rgb)));
    let in_use = Memo::new(|_| store::prefs().accent_color);
    let neutrals = NEUTRALS
        .into_iter()
        .map(|rgb| {
            view! {
                <button
                    type="button"
                    class="swatch"
                    style=format!("background-color: #{}", to_hex(rgb))
                    title=move || neutral_name(rgb)
                    aria-label=move || neutral_name(rgb)
                    aria-pressed=move || if in_use.get() == Some(rgb) { "true" } else { "false" }
                    on:click=move |_| chosen.run(rgb)
                ></button>
            }
        })
        .collect_view();
    view! {
        <div
            class="color-menu accent-menu"
            role="dialog"
            aria-label=move || t!("settings-accent-custom")
            node_ref=menu
        >
            <ColorPicker value=Signal::derive(|| Some(accent_rgb(&store::prefs()))) on_pick=chosen />
            <div class="swatches">{neutrals}</div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_choice_has_its_words() {
        for theme in Theme::ALL {
            assert!(!theme_label(theme).is_empty());
        }
        assert_eq!(lang_name(Lang::Italian), "Italiano");
        assert_eq!(lang_name(Lang::English), "English");
    }

    #[test]
    fn each_theme_shows_itself_in_its_own_colours() {
        let page = include_str!("appearance.rs");
        assert!(page.contains(r#"Theme::Light => window("scheme-light", oled)"#));
        assert!(page.contains(r#"Theme::Dark => window("scheme-dark", oled)"#));
        // The pictures take the other theme's roles, black included.
        let tokens = include_str!("../../../style/tokens.css");
        assert!(tokens.contains(".app, .scheme-light {"));
        assert!(tokens.contains(r#".app[data-theme="dark"], .scheme-dark {"#));
        assert!(tokens.contains(r#".app[data-theme="dark"][data-oled], .scheme-dark.oled {"#));
        let css = include_str!("../../../style/pages/settings.css");
        assert!(css.contains(".theme-demo {"));
        assert!(css.contains("background: var(--canvas); color: var(--on-surface);"));
    }

    #[test]
    fn the_black_background_sits_under_the_theme_and_rests_under_light() {
        let page = include_str!("appearance.rs");
        let at = |s: &str| page.find(s).unwrap_or_else(|| panic!("{s}"));
        assert!(at("t!(\"settings-theme\")") < at("<OledSwitch theme=theme />"));
        assert!(at("<OledSwitch theme=theme />") < at("t!(\"settings-language\")"));
        // A switch the keyboard and screen readers know as one.
        for attr in [
            r#"role="switch""#,
            "aria-checked=",
            "disabled=idle",
            "!oled_applies(theme.get())",
        ] {
            assert!(page.contains(attr), "{attr}");
        }
        let css = include_str!("../../../style/pages/settings.css");
        assert!(css.contains(".oled-switch:disabled { opacity: 0.45;"));
    }

    #[test]
    fn the_picker_offers_the_greys_and_shows_what_was_picked() {
        let page = include_str!("appearance.rs");
        assert!(page.contains("NEUTRALS\n        .into_iter()"));
        assert!(page.contains("aria-pressed=move || if in_use.get() == Some(rgb)"));
        // Its field follows the colour in use while it stays open.
        assert!(page.contains("value=Signal::derive(|| Some(accent_rgb(&store::prefs())))"));
        // The button is the colour in use, white too, with a dark icon on a
        // pale one.
        assert!(page.contains("class:pale=move || on.get() && theme::dark_ink(value.get())"));
        let css = include_str!("../../../style/pages/settings.css");
        assert!(css.contains(".accent.custom.pale { color: var(--on-pale-swatch); }"));
        assert!(css.contains(".accent-menu { scroll-margin:"));
    }
}
