//! A new password's box, tinted by how hard the password is to guess: red,
//! through orange, to green. The score is the server's own
//! (`starbooru_core::password_strength`), so the two always agree.

use leptos::prelude::*;
use starbooru_core::{password_strength, strength_hue};

/// The three words a score is read out as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Strength {
    Weak,
    Fair,
    Strong,
}

impl Strength {
    fn of(score: f32) -> Strength {
        if score < 0.4 {
            Strength::Weak
        } else if score < 0.75 {
            Strength::Fair
        } else {
            Strength::Strong
        }
    }

    fn text(self) -> String {
        match self {
            Strength::Weak => t!("password-strength-weak"),
            Strength::Fair => t!("password-strength-fair"),
            Strength::Strong => t!("password-strength-strong"),
        }
    }
}

/// Wraps a new-password input. `value` is what's typed, `username` the
/// account's name, which makes a password that holds it weaker.
#[component]
pub fn StrengthField(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] username: Signal<String>,
    children: Children,
) -> impl IntoView {
    let score = Memo::new(move |_| value.with(|v| username.with(|u| password_strength(v, u))));
    let rated = move || value.with(|v| !v.is_empty());
    view! {
        <div
            class="pw-field"
            class:rated=rated
            style=move || format!("--pw-hue: {:.1}", strength_hue(score.get()))
        >
            {children()}
        </div>
        // The colour, in words for a screen reader.
        <Show when=rated>
            <span class="visually-hidden" aria-live="polite">
                {move || Strength::of(score.get()).text()}
            </span>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_score_reads_as_a_word() {
        assert_eq!(Strength::of(0.0), Strength::Weak);
        assert_eq!(Strength::of(0.39), Strength::Weak);
        assert_eq!(Strength::of(0.4), Strength::Fair);
        assert_eq!(Strength::of(0.74), Strength::Fair);
        assert_eq!(Strength::of(0.75), Strength::Strong);
        assert_eq!(Strength::of(1.0), Strength::Strong);
        // The server refuses what it calls weak; the meter agrees.
        let weak = password_strength("password1", "ada");
        assert_eq!(Strength::of(weak), Strength::Weak);
        let strong = password_strength("Velvet-Comet-Harbor-42", "ada");
        assert_eq!(Strength::of(strong), Strength::Strong);
    }
}
