//! A box for a list of tags, one per word, that offers the collection's tags
//! for the word being typed, from the same list as the search box. The
//! offers are chips under the box: a click or Tab puts the first one in place
//! of the word, Escape puts them away. A category prefix (`artist:mi`) stays
//! and the rest is completed. The grip in the corner makes the box taller.
//! A one-line box has no grip, and Enter sends its form as a field's would.

use leptos::ev::{KeyboardEvent, PointerEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::dom;
use crate::logic::search::{
    Hint, Latest, TYPING_PAUSE_MS, byte_index, complete, short_count, still_fitting, tag_word_at,
    utf16_index,
};
use crate::logic::tags::offers;

/// Where the caret is in `el`, in bytes into `value`.
fn caret_of(el: &web_sys::HtmlTextAreaElement, value: &str) -> usize {
    el.selection_start()
        .ok()
        .flatten()
        .and_then(|c| usize::try_from(c).ok())
        .map_or(value.len(), |c| byte_index(value, c))
}

/// The tag box: a `textarea` with its grip, and the offers under it. `id` is
/// the textarea's, for its label; the offers are `{id}-offers`.
#[component]
pub fn TagBox(
    #[prop(into)] id: String,
    /// What's in the box.
    text: RwSignal<String>,
    #[prop(default = 5)] rows: u32,
    /// A box the height of a line, like a field: for a few tags.
    #[prop(optional)]
    one_line: bool,
    /// The box itself, for the page to focus.
    #[prop(optional)]
    area: Option<NodeRef<html::Textarea>>,
    /// A word in the box can't be a tag.
    #[prop(optional, into)]
    invalid: Signal<bool>,
    /// The box lost the focus.
    #[prop(optional)]
    on_blur: Option<Callback<()>>,
    #[prop(optional, into)] placeholder: Option<Signal<String>>,
    /// Its name, where no `label` names it.
    #[prop(optional, into)]
    label: Option<Signal<String>>,
) -> impl IntoView {
    let area = area.unwrap_or_default();
    // Offers only follow typing: none for the tags the box opened with.
    let typing = RwSignal::new(false);
    let hints = RwSignal::new(Vec::<Hint>::new());
    // The word the hints are for, and the latest question asked about one.
    let asked = StoredValue::new(None::<String>);
    let turn = StoredValue::new(Latest::default());
    let shown = Memo::new(move |_| {
        if !typing.get() {
            return Vec::new();
        }
        let found = hints.get();
        text.with(|t| offers(t, found))
    });
    let list_id = format!("{id}-offers");
    let controls = list_id.clone();

    // Asks for the tags starting with `typed` once the typing pauses, as the
    // search box does.
    let look_up = move |typed: Option<String>| {
        let Some(n) = turn.try_update_value(Latest::next) else {
            return;
        };
        let Some(typed) = typed else {
            hints.set(Vec::new());
            return;
        };
        if let Some(known) = api::kept_tags(&typed) {
            hints.set(known);
            return;
        }
        hints.update(|h| still_fitting(h, &typed));
        let latest = move || turn.try_with_value(|t| t.is(n)).unwrap_or(false);
        spawn_local(async move {
            dom::sleep(TYPING_PAUSE_MS).await;
            if !latest() {
                return;
            }
            let found = api::tag_hints(typed).await;
            if latest() {
                let _ = hints.try_set(found);
            }
        });
    };

    // Looks up the word at the caret, when it isn't the one looked up last.
    let follow = move || {
        let Some(el) = area.get_untracked() else {
            return;
        };
        let value = el.value();
        let typed = tag_word_at(&value, caret_of(&el, &value)).map(|w| w.typed.to_lowercase());
        if asked.with_value(|a| *a == typed) {
            return;
        }
        asked.set_value(typed.clone());
        look_up(typed);
    };

    // Puts offer `i` in place of the word at the caret.
    let accept = move |i: usize| {
        let Some(name) = shown.with_untracked(|h| h.get(i).map(|h| h.name.clone())) else {
            return;
        };
        let Some(el) = area.get_untracked() else {
            return;
        };
        let value = el.value();
        let Some((next, at)) =
            tag_word_at(&value, caret_of(&el, &value)).and_then(|w| complete(&value, &w, &name))
        else {
            return;
        };
        // Straight into the box, so the caret can go after the tag at once.
        el.set_value(&next);
        let caret = u32::try_from(utf16_index(&next, at)).unwrap_or(u32::MAX);
        let _ = el.set_selection_range(caret, caret);
        let _ = el.focus();
        text.set(next);
        typing.set(false);
        asked.set_value(None);
        turn.update_value(|t| {
            t.next();
        });
        hints.set(Vec::new());
    };

    let on_key = move |e: KeyboardEvent| {
        if e.is_composing() {
            return;
        }
        let offered = !shown.with_untracked(Vec::is_empty);
        match e.key().as_str() {
            "Enter" if one_line => {
                e.prevent_default();
                if let Some(form) = area.get_untracked().and_then(|el| el.form()) {
                    let _ = form.request_submit();
                }
            }
            "Tab" if offered && !e.shift_key() => {
                e.prevent_default();
                accept(0);
            }
            "Escape" if offered => {
                // The offers go, and nothing around the box hears it.
                e.prevent_default();
                e.stop_propagation();
                typing.set(false);
            }
            _ => {}
        }
    };

    let list = move || {
        shown
            .get()
            .into_iter()
            .enumerate()
            .map(|(i, Hint { name, class, count })| {
                // The first is the one Tab takes.
                let first = if i == 0 { "true" } else { "false" };
                view! {
                    <button
                        type="button"
                        role="option"
                        class=format!("chip {class}")
                        aria-selected=first
                        tabindex="-1"
                        // A press would take the caret out of the box first.
                        on:mousedown=|e| e.prevent_default()
                        on:click=move |_| accept(i)
                    >
                        {name}
                        {count.map(|n| view! { <span class="count">{short_count(n)}</span> })}
                    </button>
                }
            })
            .collect_view()
    };
    view! {
        <span class="text-box" class:one-line=one_line>
            <textarea
                node_ref=area
                id=id
                rows=if one_line { 1 } else { rows }
                spellcheck="false"
                autocapitalize="none"
                autocomplete="off"
                aria-autocomplete="list"
                aria-controls=controls
                aria-invalid=move || invalid.get().then_some("true")
                aria-label=move || label.map(|l| l.get())
                placeholder=move || placeholder.map(|p| p.get())
                prop:value=move || text.get()
                on:input=move |e| {
                    let mut value = event_target_value(&e);
                    // Pasted lines become words on the one line.
                    if one_line && value.contains(['\n', '\r']) {
                        value = value.replace(['\r', '\n'], " ");
                        if let Some(el) = area.get_untracked() {
                            el.set_value(&value);
                        }
                    }
                    text.set(value);
                    typing.set(true);
                    follow();
                }
                on:keydown=on_key
                // The caret moved to another word.
                on:keyup=move |e: KeyboardEvent| {
                    if matches!(
                        e.key().as_str(),
                        "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown" | "Home" | "End"
                    ) {
                        follow();
                    }
                }
                on:click=move |_| follow()
                on:blur=move |_| {
                    typing.set(false);
                    if let Some(blur) = on_blur {
                        blur.run(());
                    }
                }
            ></textarea>
            {(!one_line).then(|| view! { <Grip area=area /> })}
        </span>
        <Show when=move || shown.with(|s| !s.is_empty())>
            <div
                class="tag-offers"
                id=list_id.clone()
                role="listbox"
                aria-label=move || t!("tag-offers")
            >
                {list}
            </div>
        </Show>
    }
}

/// The least a box can be dragged to, in pixels: two lines.
const LEAST_HEIGHT: f64 = 64.0;

/// The box's height while its grip is dragged from `from` to `y`, starting
/// at `start` pixels tall, and never under `least`.
pub fn grip_height(start: f64, from: f64, y: f64, least: f64) -> f64 {
    let h = start + y - from;
    if h.is_finite() { h.max(least) } else { least }
}

/// The box's own `min-height`, in pixels, if it has one.
fn min_height(el: &web_sys::Element) -> Option<f64> {
    let style = web_sys::window()?.get_computed_style(el).ok()??;
    let value = style.get_property_value("min-height").ok()?;
    value.trim().trim_end_matches("px").parse().ok()
}

/// A drag of the grip: which pointer, where it started and where it is.
#[derive(Clone, Copy, Debug)]
struct Drag {
    pointer: i32,
    from: f64,
    start: f64,
    least: f64,
    y: f64,
}

/// The handle that makes a text box taller. Browsers draw their own, each
/// differently and none in the app's colours, so CSS turns it off
/// (`resize: none`) and this stands in for it. A move is applied once a
/// frame.
#[component]
fn Grip(area: NodeRef<html::Textarea>) -> impl IntoView {
    let grip: NodeRef<html::Span> = NodeRef::new();
    let drag = StoredValue::new(None::<Drag>);
    let held = RwSignal::new(false);
    let frame = StoredValue::new(false);

    let size = move || {
        frame.try_set_value(false);
        let (Some(Some(d)), Some(el)) = (drag.try_get_value(), area.get_untracked()) else {
            return;
        };
        let h = grip_height(d.start, d.from, d.y, d.least);
        let _ = web_sys::HtmlElement::style(&el).set_property("height", &format!("{h:.0}px"));
    };
    let down = move |e: PointerEvent| {
        if e.button() != 0 {
            return;
        }
        let (Some(el), Some(handle)) = (area.get_untracked(), grip.get_untracked()) else {
            return;
        };
        e.prevent_default();
        let from = f64::from(e.client_y());
        drag.set_value(Some(Drag {
            pointer: e.pointer_id(),
            from,
            start: el.get_bounding_client_rect().height(),
            least: min_height(&el).unwrap_or(0.0).max(LEAST_HEIGHT),
            y: from,
        }));
        let _ = handle.set_pointer_capture(e.pointer_id());
        held.set(true);
    };
    let moved = move |e: PointerEvent| {
        let y = f64::from(e.client_y());
        let mine = drag
            .try_update_value(|d| match d {
                Some(d) if d.pointer == e.pointer_id() => {
                    d.y = y;
                    true
                }
                _ => false,
            })
            .unwrap_or(false);
        if mine && !frame.get_value() {
            frame.set_value(true);
            request_animation_frame(move || {
                if frame.try_get_value().is_some() {
                    size();
                }
            });
        }
    };
    let up = move |e: PointerEvent| {
        if drag.with_value(|d| d.is_some_and(|d| d.pointer == e.pointer_id())) {
            size();
            drag.set_value(None);
            held.set(false);
        }
    };
    view! {
        <span
            class="grip"
            class:held=move || held.get()
            node_ref=grip
            aria-hidden="true"
            on:pointerdown=down
            on:pointermove=moved
            on:pointerup=up
            on:pointercancel=up
        ></span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grip_follows_the_pointer_but_keeps_two_lines() {
        assert_eq!(grip_height(120.0, 300.0, 340.0, 64.0), 160.0);
        assert_eq!(grip_height(120.0, 300.0, 250.0, 64.0), 70.0);
        assert_eq!(grip_height(120.0, 300.0, 100.0, 64.0), 64.0);
        assert_eq!(grip_height(f64::NAN, 300.0, 100.0, 64.0), 64.0);
    }

    #[test]
    fn the_browsers_handle_is_off_and_ours_is_in_the_apps_colours() {
        let css = include_str!("../../style/base.css");
        assert!(css.contains("textarea { resize: none; }"));
        for file in [
            css,
            include_str!("../../style/shell.css"),
            include_str!("../../style/pages/post.css"),
            include_str!("../../style/pages/tags.css"),
        ] {
            assert!(!file.contains("resize: vertical") && !file.contains("resize: both"));
        }
        let (_, grip) = css.split_once("\n.grip {").unwrap();
        assert!(grip.contains("touch-action: none"));
        assert!(css.contains(".grip::before"));
        assert!(css.contains(".grip:is(:hover, .held)"));
    }

    #[test]
    fn the_box_is_a_combobox_over_its_offers() {
        let source = include_str!("tag_box.rs");
        for attr in [
            r#"aria-autocomplete="list""#,
            "aria-controls=controls",
            r#"role="listbox""#,
            r#"role="option""#,
            "<Grip area=area />",
        ] {
            assert!(source.contains(attr), "{attr}");
        }
    }
}
