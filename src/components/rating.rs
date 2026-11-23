//! Picking a rating: four segments side by side, or a drop-down where room
//! is short.

use leptos::prelude::*;
use starbooru_core::Rating;

use super::select::Choice;
use crate::pages::rating_label;

/// The four ratings, one of them chosen.
#[component]
pub fn RatingPicker(value: RwSignal<Rating>) -> impl IntoView {
    let choices = Rating::ALL
        .into_iter()
        .map(|r| {
            let on = move || value.get() == r;
            view! {
                <button
                    type="button"
                    role="radio"
                    class="seg"
                    class:on=on
                    aria-checked=move || if on() { "true" } else { "false" }
                    on:click=move |_| value.set(r)
                >
                    {move || rating_label(r)}
                </button>
            }
        })
        .collect_view();
    view! {
        <div class="segmented" role="radiogroup" aria-label=move || t!("rating")>
            {choices}
        </div>
    }
}

/// The ratings as the choices of a drop-down.
pub fn rating_choices() -> Vec<Choice> {
    Rating::ALL
        .into_iter()
        .map(|r| Choice::new(r.as_str(), rating_label(r)))
        .collect()
}

/// The same, after a first choice with no rating of its own that goes with
/// the rest: `usual` says which rating that is.
pub fn own_rating_choices(usual: String) -> Vec<Choice> {
    let mut choices = vec![Choice::new("", usual)];
    choices.extend(rating_choices());
    choices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_item_can_go_with_the_rest_or_have_its_own_rating() {
        let choices = own_rating_choices("Same as the rest (safe)".into());
        assert_eq!(choices.len(), Rating::ALL.len() + 1);
        assert_eq!(choices.first().map(|c| c.value.as_str()), Some(""));
        assert_eq!(Rating::parse(""), None);
        for (choice, rating) in choices.iter().skip(1).zip(Rating::ALL) {
            assert_eq!(Rating::parse(&choice.value), Some(rating));
        }
    }
}
