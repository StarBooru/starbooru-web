//! The colour picker: a wheel for the pointer and a hex field for an exact
//! colour, which is also how the keyboard reaches it.

use leptos::ev::{MouseEvent, SubmitEvent};
use leptos::prelude::*;
use starbooru_core::color::{Rgb, parse_hex_color, to_hex, wheel_color};

use super::icons::{Ico, Icon};

/// The wheel's width in pixels, as `.color-wheel` in `style/pages/gallery.css`.
const WHEEL_SIZE: f64 = 176.0;

fn hex_text(value: Option<Rgb>) -> String {
    value
        .map(|rgb| format!("#{}", to_hex(rgb)))
        .unwrap_or_default()
}

/// The colour at a point of the wheel, `x` and `y` pixels from its corner.
fn at_wheel(x: f64, y: f64) -> Rgb {
    let radius = WHEEL_SIZE / 2.0;
    wheel_color((x - radius) / radius, (y - radius) / radius)
}

/// Calls `on_pick` for a click on the wheel or a hex colour entered; `value`
/// fills the field, and fills it again whenever it changes, so a picker that
/// stays open shows what was just picked.
#[component]
pub fn ColorPicker(
    #[prop(into)] value: Signal<Option<Rgb>>,
    on_pick: Callback<Rgb>,
) -> impl IntoView {
    let typed = RwSignal::new(hex_text(value.get_untracked()));
    let invalid = RwSignal::new(false);
    Effect::new(move |_| {
        typed.set(hex_text(value.get()));
        invalid.set(false);
    });
    let preview = move || {
        parse_hex_color(&typed.get()).map(|rgb| format!("background-color: #{}", to_hex(rgb)))
    };
    let submit = move |e: SubmitEvent| {
        e.prevent_default();
        let rgb = parse_hex_color(&typed.get_untracked());
        invalid.set(rgb.is_none());
        if let Some(rgb) = rgb {
            on_pick.run(rgb);
        }
    };
    view! {
        <div
            class="color-wheel"
            title=move || t!("color-wheel")
            aria-hidden="true"
            on:click=move |e: MouseEvent| {
                on_pick.run(at_wheel(f64::from(e.offset_x()), f64::from(e.offset_y())));
            }
        ></div>
        <form class="field color-hex" on:submit=submit>
            <span class="color-preview" aria-hidden="true" style=preview></span>
            <input
                type="text"
                maxlength="7"
                spellcheck="false"
                autocomplete="off"
                placeholder="#2060e0"
                aria-label=move || t!("color-hex")
                aria-invalid=move || if invalid.get() { "true" } else { "false" }
                prop:value=move || typed.get()
                on:input=move |e| {
                    typed.set(event_target_value(&e));
                    invalid.set(false);
                }
            />
            <button
                type="submit"
                class="icon-btn"
                title=move || t!("color-hex-apply")
                aria-label=move || t!("color-hex-apply")
            >
                <Ico icon=Icon::Check size=20 />
            </button>
        </form>
        <Show when=move || invalid.get()>
            <p class="field-error" role="alert">
                {move || t!("color-hex-invalid")}
            </p>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wheel_is_white_in_the_middle_and_hues_round_the_edge() {
        let middle = at_wheel(WHEEL_SIZE / 2.0, WHEEL_SIZE / 2.0);
        assert!(middle.iter().all(|c| *c > 240), "{middle:?}");
        // Red at the top, where the CSS gradient starts too.
        let [r, g, b] = at_wheel(WHEEL_SIZE / 2.0, 0.0);
        assert!(r > 200 && g < 80 && b < 80, "{r} {g} {b}");
        let css = include_str!("../../style/tokens.css");
        assert!(css.contains("conic-gradient(#f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00)"));
        assert_eq!(hex_text(Some([0x20, 0x60, 0xe0])), "#2060e0");
        assert_eq!(hex_text(None), "");
    }
}
