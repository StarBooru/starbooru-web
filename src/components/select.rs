//! A themed drop-down. The native `<select>` opens a system popup that CSS can't
//! reach, so this draws its own list (the ARIA "select-only combobox" pattern),
//! below its button and lined up with its right edge.

use std::sync::atomic::{AtomicUsize, Ordering};

use leptos::ev::KeyboardEvent;
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    pub value: String,
    pub label: String,
}

impl Choice {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

/// `value` is the chosen option's value, `label` names the drop-down for a
/// screen reader, and `class` goes on the wrapper.
#[component]
pub fn Select(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] options: Signal<Vec<Choice>>,
    on_change: Callback<String>,
    #[prop(into)] label: Signal<String>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let list_id = format!("select-{id}");
    let option_id = move |i: usize| format!("select-{id}-{i}");
    let open = RwSignal::new(false);
    let active = RwSignal::new(0_usize);
    let current = Memo::new(move |_| {
        let value = value.get();
        options.with(|o| o.iter().position(|c| c.value == value))
    });
    let count = move || options.with(Vec::len);

    let show = move |i: usize| {
        active.set(i);
        open.set(true);
        // The highlighted option stays in view when the list scrolls.
        request_animation_frame(move || {
            if let Some(li) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id(&option_id(i)))
            {
                let options = web_sys::ScrollIntoViewOptions::new();
                options.set_block(web_sys::ScrollLogicalPosition::Nearest);
                li.scroll_into_view_with_scroll_into_view_options(&options);
            }
        });
    };
    let pick = move |i: usize| {
        open.set(false);
        if Some(i) != current.get_untracked()
            && let Some(choice) = options.with_untracked(|o| o.get(i).cloned())
        {
            on_change.run(choice.value);
        }
    };
    let on_key = move |e: KeyboardEvent| {
        let key = e.key();
        if key == "Tab" {
            open.set(false);
            return;
        }
        let action = key_action(
            &key,
            open.get_untracked(),
            active.get_untracked(),
            current.get_untracked(),
            count(),
        );
        if action != Action::Nothing {
            // The open list owns these keys: Escape mustn't also close a menu.
            e.prevent_default();
            e.stop_propagation();
        }
        match action {
            Action::Show(i) => show(i),
            Action::Pick(i) => pick(i),
            Action::Close => open.set(false),
            Action::Nothing => {}
        }
    };
    let shown = move || {
        current
            .get()
            .and_then(|i| options.with(|o| o.get(i).map(|c| c.label.clone())))
            .unwrap_or_default()
    };
    let list = list_id.clone();
    view! {
        <div class=format!("select {class}")>
            <button
                type="button"
                id=format!("{list_id}-button")
                class="select-button"
                role="combobox"
                aria-label=move || label.get()
                aria-haspopup="listbox"
                aria-controls=list_id.clone()
                aria-expanded=move || if open.get() { "true" } else { "false" }
                aria-activedescendant=move || open.get().then(|| option_id(active.get()))
                on:click=move |_| {
                    if open.get_untracked() {
                        open.set(false);
                    } else {
                        show(current.get_untracked().unwrap_or(0));
                    }
                }
                on:blur=move |_| open.set(false)
                on:keydown=on_key
            >
                // Every label stacked in one cell, so the button is as wide as the longest.
                <span class="select-sizer" aria-hidden="true">
                    {move || {
                        options
                            .get()
                            .into_iter()
                            .map(|c| view! { <span>{c.label}</span> })
                            .collect_view()
                    }}
                </span>
                <span class="select-value">{shown}</span>
            </button>
            <Show when=move || open.get()>
                <div class="select-backdrop" on:click=move |_| open.set(false)></div>
                <ul
                    class="select-list"
                    id=list.clone()
                    role="listbox"
                    aria-label=move || label.get()
                    // A press would take the focus from the button and close the list first.
                    on:mousedown=|e| e.prevent_default()
                >
                    {move || {
                        options
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, c)| {
                                view! {
                                    <li
                                        id=option_id(i)
                                        role="option"
                                        class:active=move || active.get() == i
                                        aria-selected=move || {
                                            if current.get() == Some(i) { "true" } else { "false" }
                                        }
                                        on:mouseenter=move |_| active.set(i)
                                        on:click=move |_| pick(i)
                                    >
                                        {c.label}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
        </div>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Show(usize),
    Pick(usize),
    Close,
    Nothing,
}

/// What key `key` (as `KeyboardEvent.key` names it) does on the button, given
/// whether the list is `open`, the highlighted option `at`, the chosen one
/// and how many there are.
fn key_action(key: &str, open: bool, at: usize, current: Option<usize>, count: usize) -> Action {
    let Some(last) = count.checked_sub(1) else {
        return Action::Nothing;
    };
    let at = at.min(last);
    match key {
        "ArrowDown" | "ArrowUp" if !open => Action::Show(current.unwrap_or(0).min(last)),
        "ArrowDown" => Action::Show(at.saturating_add(1).min(last)),
        "ArrowUp" => Action::Show(at.saturating_sub(1)),
        "Home" if open => Action::Show(0),
        "End" if open => Action::Show(last),
        "Enter" | " " if open => Action::Pick(at),
        "Escape" if open => Action::Close,
        _ => Action::Nothing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrows_open_on_the_chosen_option() {
        assert_eq!(
            key_action("ArrowDown", false, 0, Some(3), 5),
            Action::Show(3)
        );
        assert_eq!(key_action("ArrowUp", false, 0, None, 5), Action::Show(0));
    }

    #[test]
    fn arrows_stop_at_the_ends() {
        assert_eq!(key_action("ArrowDown", true, 4, None, 5), Action::Show(4));
        assert_eq!(key_action("ArrowUp", true, 0, None, 5), Action::Show(0));
        assert_eq!(key_action("ArrowDown", true, 1, None, 5), Action::Show(2));
        assert_eq!(key_action("End", true, 1, None, 5), Action::Show(4));
        assert_eq!(key_action("Home", true, 3, None, 5), Action::Show(0));
    }

    #[test]
    fn enter_space_and_escape_only_act_when_open() {
        assert_eq!(key_action("Enter", true, 2, None, 5), Action::Pick(2));
        assert_eq!(key_action(" ", true, 2, None, 5), Action::Pick(2));
        assert_eq!(key_action("Escape", true, 2, None, 5), Action::Close);
        // Closed: the button's own click (Enter, Space) and the page (Escape) have them.
        assert_eq!(key_action("Enter", false, 2, None, 5), Action::Nothing);
        assert_eq!(key_action("Escape", false, 2, None, 5), Action::Nothing);
        assert_eq!(key_action("a", true, 2, None, 5), Action::Nothing);
    }

    #[test]
    fn empty_or_stale_lists_never_point_past_the_end() {
        assert_eq!(key_action("ArrowDown", true, 0, None, 0), Action::Nothing);
        assert_eq!(key_action("Enter", true, 9, None, 3), Action::Pick(2));
        assert_eq!(
            key_action("ArrowDown", false, 0, Some(7), 3),
            Action::Show(2)
        );
    }

    #[test]
    fn the_arrow_is_a_drawn_shape() {
        let css = include_str!("../../style/pages/gallery.css");
        let arrow = css
            .split_once(".select-button::after {")
            .and_then(|(_, rest)| rest.split_once('}'))
            .map(|(rule, _)| rule)
            .unwrap_or("");
        assert!(arrow.contains("background: var(--on-surface-2)"));
        assert!(arrow.contains("mask: url(\"data:image/svg+xml"));
        assert!(arrow.contains("pointer-events: none"));
    }
}
