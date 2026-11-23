//! The media editor over the viewer (its sums are in `logic/editor.rs`). It
//! fills the window: the picture takes most of it, and the tools stand beside
//! it when the window is wider than tall, on the hand's side, or under it
//! when taller. Images are cropped, straightened, turned and flipped, with
//! undo and redo; videos are trimmed on a timeline. The server saves the
//! result as a new post, and the original goes to the recycle bin unless the
//! user keeps it, which the preferences remember.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use leptos::ev::{KeyboardEvent, PointerEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use send_wrapper::SendWrapper;
use starbooru_core::{EditKind, MAX_STRAIGHTEN, Post};
use wasm_bindgen::JsCast;

use crate::api;
use crate::components::banner::ErrorBanner;
use crate::components::icons::{Ico, Icon};
use crate::dom;
use crate::logic::editor::{
    Aspect, Command, CropBox, Handle, History, Layout, MAX_RATIO_SIDE, Problem, RATIOS, Shape,
    TrimEnd, arrow, command, frame_size, moved_trim, oriented, parse_seconds, picture_style,
    ratio_side, request, seconds_text, share, spun, time_at, trim_key, trim_of, trim_shown,
};
use crate::logic::viewer::format_duration;
use crate::store;

/// How long the editor takes to fade out.
const CLOSE_MS: u64 = 180;
/// A mouse drag on the proportions has to go this far before it scrolls them
/// instead of pressing one.
const STRIP_DRAG: f64 = 6.0;

/// What the editor saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Saved {
    /// The new post.
    pub id: i64,
    /// The post that was edited.
    pub original: i64,
    /// The original went to the recycle bin.
    pub binned: bool,
}

fn problem_text(p: Problem) -> String {
    match p {
        Problem::NoChange => t!("editor-no-change"),
        Problem::BadTime => t!("editor-bad-time"),
        Problem::TooShort => t!("editor-too-short"),
    }
}

fn handle_label(handle: Handle) -> String {
    match handle {
        Handle::Move => t!("editor-crop-area"),
        Handle::Top => t!("editor-handle-n"),
        Handle::Bottom => t!("editor-handle-s"),
        Handle::Left => t!("editor-handle-w"),
        Handle::Right => t!("editor-handle-e"),
        Handle::TopLeft => t!("editor-handle-nw"),
        Handle::TopRight => t!("editor-handle-ne"),
        Handle::BottomLeft => t!("editor-handle-sw"),
        Handle::BottomRight => t!("editor-handle-se"),
    }
}

/// Whether a key comes from a box the user types in, which keeps its own
/// undo. A slider or a switch doesn't.
fn in_text(target: Option<web_sys::EventTarget>) -> bool {
    let Some(target) = target else {
        return false;
    };
    if let Some(input) = target.dyn_ref::<web_sys::HtmlInputElement>() {
        return !matches!(input.type_().as_str(), "range" | "checkbox" | "button");
    }
    dom::typing(Some(target))
}

/// Pauses the video on the stage under the editor, which would otherwise play
/// on unseen.
pub fn pause_stage_video() {
    if let Some(video) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.query_selector(".stage video").ok().flatten())
        .and_then(|el| el.dyn_into::<web_sys::HtmlVideoElement>().ok())
    {
        let _ = video.pause();
    }
}

/// The editor for `post`, over the whole window. `src` is the picture or
/// video to show, absolute.
#[component]
pub fn EditorSheet(
    post: Arc<Post>,
    kind: EditKind,
    src: String,
    /// The editor has begun to fade: it no longer covers what's under it.
    on_leave: Callback<()>,
    /// It has gone.
    on_close: Callback<()>,
    on_saved: Callback<Saved>,
) -> impl IntoView {
    let id = post.id;
    let width = post.width.and_then(|w| u32::try_from(w).ok()).unwrap_or(0);
    let height = post.height.and_then(|h| u32::try_from(h).ok()).unwrap_or(0);
    let duration_ms = post.duration_ms;
    let duration = duration_ms.unwrap_or(0).max(0);

    let shape = RwSignal::new(Shape::new(width, height));
    let history = RwSignal::new(History::default());
    // Where a drag, a slide or a run of key presses started, to make one
    // step of it.
    let live_from = RwSignal::new(None::<Shape>);
    let spin = RwSignal::new(0_i32);
    let busy = RwSignal::new(false);
    let closing = RwSignal::new(false);
    let menu = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let start = RwSignal::new(seconds_text(0));
    let end = RwSignal::new(seconds_text(duration));
    // A trim handle or the playhead is being dragged.
    let sliding = RwSignal::new(false);
    let playhead = RwSignal::new(0_i64);
    let video: NodeRef<html::Video> = NodeRef::new();
    let keep = Memo::new(|_| store::prefs().keep_original);
    let layout = RwSignal::new({
        let (w, h) = dom::window_size();
        Layout::of(w, h)
    });

    // Every change goes through here, so the picture turns the short way.
    let show = move |next: Shape| {
        let s = spun(spin.get_untracked(), next.edit.rotate);
        if s != spin.get_untracked() {
            spin.set(s);
        }
        shape.set(next);
    };
    let finish_live = move || {
        let Some(before) = live_from.get_untracked() else {
            return;
        };
        live_from.set(None);
        if shape.with_untracked(|now| *now != before) {
            history.update(|h| h.record(before));
        }
    };
    let commit = move |next: Shape| {
        finish_live();
        let now = shape.get_untracked();
        if next != now {
            history.update(|h| h.record(now));
            show(next);
        }
    };
    let live = move |next: Shape| {
        if live_from.with_untracked(Option::is_none) {
            live_from.set(Some(shape.get_untracked()));
        }
        show(next);
    };
    let undo = move || {
        finish_live();
        let now = shape.get_untracked();
        let mut back = None;
        history.update(|h| back = h.undo(&now));
        if let Some(back) = back {
            show(back);
        }
    };
    let redo = move || {
        finish_live();
        let now = shape.get_untracked();
        let mut ahead = None;
        history.update(|h| ahead = h.redo(&now));
        if let Some(ahead) = ahead {
            show(ahead);
        }
    };
    let on_commit = Callback::new(move |next: Shape| commit(next));
    let on_live = Callback::new(move |next: Shape| live(next));
    let on_settle = Callback::new(move |()| finish_live());

    // The request the editor describes, or why it can't be sent.
    let asked = Memo::new(move |_| {
        shape.with(|s| {
            start.with(|a| {
                end.with(|b| {
                    request(
                        kind,
                        s,
                        (a.as_str(), b.as_str()),
                        (width, height),
                        duration_ms,
                    )
                })
            })
        })
    });
    let problem = Memo::new(move |_| asked.with(|r| r.as_ref().err().copied()));
    let changed = Memo::new(move |_| problem.get() != Some(Problem::NoChange));

    let close = move || {
        if busy.get_untracked() || closing.get_untracked() {
            return;
        }
        closing.set(true);
        on_leave.run(());
        let ms = if dom::reduced_motion() { 0 } else { CLOSE_MS };
        set_timeout(
            move || {
                on_close.try_run(());
            },
            Duration::from_millis(ms),
        );
    };
    let save = move || {
        if busy.get_untracked() || closing.get_untracked() {
            return;
        }
        let Ok(mut body) = asked.get_untracked() else {
            return;
        };
        let binned = !store::prefs_untracked().keep_original;
        body.bin_original = binned;
        busy.set(true);
        error.set(None);
        menu.set(false);
        spawn_local(async move {
            let result = api::call(move |c| async move { c.edit_post(id, &body).await }).await;
            busy.try_set(false);
            match result {
                Ok(saved) => {
                    on_saved.try_run(Saved {
                        id: saved.id,
                        original: id,
                        binned,
                    });
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    };
    let keep_original = Callback::new(move |keeps: bool| {
        store::update_prefs(|p| p.keep_original = keeps);
        menu.set(false);
    });

    // The keys, the window's shape, and the focus: in the editor while it's
    // open, and back where it was after.
    let root: NodeRef<html::Div> = NodeRef::new();
    let watches = StoredValue::new_local(Vec::<dom::Watch>::new());
    let before = StoredValue::new_local(
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.active_element())
            .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok()),
    );
    root.on_load(move |el| {
        let _ = el.focus();
        let Some(window) = web_sys::window() else {
            return;
        };
        let mut all = Vec::new();
        all.push(dom::listen(&window, "keydown", false, move |e| {
            let Some(e) = e.dyn_ref::<web_sys::KeyboardEvent>() else {
                return;
            };
            if e.default_prevented() || closing.get_untracked() {
                return;
            }
            let Some(cmd) = command(
                &e.key(),
                e.ctrl_key() || e.meta_key(),
                e.shift_key(),
                e.alt_key(),
            ) else {
                return;
            };
            if matches!(cmd, Command::Undo | Command::Redo)
                && (kind != EditKind::Image || in_text(e.target()))
            {
                return;
            }
            e.prevent_default();
            e.stop_propagation();
            match cmd {
                Command::Undo => undo(),
                Command::Redo => redo(),
                Command::Save => save(),
                Command::Close if menu.get_untracked() => menu.set(false),
                Command::Close => close(),
            }
        }));
        all.push(dom::on_window("resize", move || {
            let (w, h) = dom::window_size();
            let now = Layout::of(w, h);
            if layout.try_get_untracked().is_some_and(|l| l != now) {
                layout.set(now);
            }
        }));
        watches.update_value(|w| w.extend(all));
    });
    on_cleanup({
        let kept = SendWrapper::new((watches, before));
        move || {
            let (watches, before) = &*kept;
            if let Some(all) = watches.try_update_value(std::mem::take) {
                for watch in all {
                    watch.stop();
                }
            }
            if let Some(el) = before.try_update_value(Option::take).flatten()
                && el.is_connected()
            {
                let _ = el.focus();
            }
        }
    });

    let title = move || match kind {
        EditKind::Image => t!("editor-title-image"),
        EditKind::Video => t!("editor-title-video"),
    };
    let (tool_icon, tool_label) = match kind {
        EditKind::Image => (Icon::Crop, Signal::derive(|| t!("editor-crop"))),
        EditKind::Video => (Icon::Cut, Signal::derive(|| t!("editor-trim"))),
    };

    let history_pill = (kind == EditKind::Image).then(|| {
        view! {
            <div
                class="editor-pill glass float editor-history"
                role="group"
                aria-label=move || t!("editor-history")
            >
                <button
                    type="button"
                    class="icon-btn"
                    disabled=move || !history.with(History::can_undo)
                    aria-label=move || t!("editor-undo")
                    title=move || t!("editor-undo-keys")
                    on:click=move |_| undo()
                >
                    <Ico icon=Icon::Undo />
                </button>
                <button
                    type="button"
                    class="icon-btn"
                    disabled=move || !history.with(History::can_redo)
                    aria-label=move || t!("editor-redo")
                    title=move || t!("editor-redo-keys")
                    on:click=move |_| redo()
                >
                    <Ico icon=Icon::Redo />
                </button>
                <button
                    type="button"
                    class="icon-btn"
                    disabled=move || !changed.get()
                    aria-label=move || t!("editor-reset")
                    title=move || t!("editor-reset")
                    on:click=move |_| commit(Shape::new(width, height))
                >
                    <Ico icon=Icon::Restore />
                </button>
            </div>
        }
    });

    // Save, and beside it a menu for what happens to the original.
    let save_group = view! {
        <div class="editor-save-group">
            <div class="editor-save-split">
                <button
                    type="button"
                    class="btn editor-save"
                    disabled=move || busy.get() || problem.with(Option::is_some)
                    title=move || {
                        if problem.get() == Some(Problem::NoChange) {
                            t!("editor-no-change")
                        } else {
                            t!("editor-save-keys")
                        }
                    }
                    on:click=move |_| save()
                >
                    {move || {
                        if busy.get() {
                            view! { <span class="spinner small" aria-hidden="true"></span> }.into_any()
                        } else {
                            view! { <Ico icon=Icon::Check /> }.into_any()
                        }
                    }}
                    <span>
                        {move || if keep.get() { t!("editor-save-copy") } else { t!("editor-save") }}
                    </span>
                </button>
                <button
                    type="button"
                    class="btn editor-save-more"
                    disabled=move || busy.get()
                    aria-label=move || t!("editor-save-options")
                    title=move || t!("editor-save-options")
                    aria-haspopup="menu"
                    aria-expanded=move || menu.get().to_string()
                    on:click=move |_| menu.update(|m| *m = !*m)
                >
                    <Ico icon=Icon::ChevronRight size=20 />
                </button>
            </div>
            <Show when=move || menu.get()>
                <div class="save-backdrop" on:click=move |_| menu.set(false)></div>
                <div class="save-menu" role="menu" aria-label=move || t!("editor-save-options")>
                    <SaveOption
                        keeps=false
                        icon=Icon::Delete
                        label=Signal::derive(|| t!("editor-save-replace"))
                        hint=Signal::derive(|| t!("editor-save-replace-hint"))
                        keep=keep
                        on_pick=keep_original
                    />
                    <SaveOption
                        keeps=true
                        icon=Icon::Copy
                        label=Signal::derive(|| t!("editor-keep-original"))
                        hint=Signal::derive(|| t!("editor-keep-original-hint"))
                        keep=keep
                        on_pick=keep_original
                    />
                </div>
            </Show>
        </div>
    };

    let on_seek = Callback::new(move |ms: i64| {
        if let Some(v) = video.get_untracked() {
            v.set_current_time(ms as f64 / 1000.0);
        }
        playhead.set(ms);
    });
    let follow_video = move || {
        if let Some(v) = video.get_untracked() {
            let ms = (v.current_time() * 1000.0).round();
            if ms.is_finite() {
                playhead.set(ms as i64);
            }
        }
    };

    let canvas = match kind {
        EditKind::Image => view! {
            <ImageCanvas
                src=src
                shape=shape
                spin=spin
                width=width
                height=height
                on_live=on_live
                on_settle=on_settle
            />
        }
        .into_any(),
        EditKind::Video => view! {
            <div class="editor-canvas">
                <video
                    node_ref=video
                    class="editor-video"
                    src=src
                    controls=true
                    playsinline=true
                    preload="metadata"
                    on:timeupdate=move |_| follow_video()
                    on:seeked=move |_| follow_video()
                ></video>
            </div>
        }
        .into_any(),
    };
    let tools = match kind {
        EditKind::Image => view! {
            <CropTools
                shape=shape
                width=width
                height=height
                on_commit=on_commit
                on_live=on_live
                on_settle=on_settle
            />
        }
        .into_any(),
        EditKind::Video => view! {
            <TrimTools
                duration=duration
                start=start
                end=end
                playhead=playhead
                sliding=sliding
                video=video
                on_seek=on_seek
            />
        }
        .into_any(),
    };

    view! {
        <div
            node_ref=root
            class="editor"
            // While something is dragged nothing eases, so it keeps up with the finger.
            class:live=move || live_from.with(Option::is_some) || sliding.get()
            class:closing=move || closing.get()
            data-layout=move || layout.get().as_str()
            role="dialog"
            aria-modal="true"
            aria-label=title
            tabindex="-1"
        >
            <div class="editor-top">
                <button
                    type="button"
                    class="editor-pill glass float icon-only"
                    aria-label=move || t!("cancel")
                    title=move || t!("cancel")
                    disabled=move || busy.get()
                    on:click=move |_| close()
                >
                    <Ico icon=Icon::Close />
                </button>
                {history_pill}
                {save_group}
            </div>
            {canvas}
            <div class="editor-panel glass float">
                <div class="tool-tabs" role="tablist" aria-label=move || t!("editor-tools")>
                    <button type="button" class="tool-tab on" role="tab" aria-selected="true">
                        <Ico icon=tool_icon />
                        <span>{move || tool_label.get()}</span>
                    </button>
                </div>
                <div class="tool-body" role="tabpanel">
                    {tools}
                    {move || {
                        if let Some(message) = error.get() {
                            Some(view! { <ErrorBanner message=message /> }.into_any())
                        } else {
                            problem
                                .get()
                                .filter(|p| changed.get() && *p != Problem::NoChange)
                                .map(|p| {
                                    view! {
                                        <p class="field-error" role="status">
                                            {problem_text(p)}
                                        </p>
                                    }
                                        .into_any()
                                })
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

/// One way to save in the menu by the Save button: replace the original, or
/// keep it.
#[component]
fn SaveOption(
    keeps: bool,
    icon: Icon,
    label: Signal<String>,
    hint: Signal<String>,
    keep: Memo<bool>,
    on_pick: Callback<bool>,
) -> impl IntoView {
    let on = move || keep.get() == keeps;
    view! {
        <button
            type="button"
            class="save-option"
            class:on=on
            role="menuitemradio"
            aria-checked=move || on().to_string()
            title=move || hint.get()
            on:click=move |_| on_pick.run(keeps)
        >
            <Ico icon=icon size=20 />
            <span>{move || label.get()}</span>
            <span class="save-option-check">
                <Ico icon=Icon::Check size=20 />
            </span>
        </button>
    }
}

/// Hands `on_commit` the shape `f` makes from the current one.
fn change(shape: RwSignal<Shape>, on_commit: Callback<Shape>, f: impl FnOnce(&Shape) -> Shape) {
    let next = shape.with_untracked(f);
    on_commit.run(next);
}

/// A proportion as a rounded frame, drawn in the icons' 2-unit line. The
/// frame eases to its new size when the proportion turns.
#[component]
fn RatioGlyph(#[prop(into)] size: Signal<(f64, f64)>) -> impl IntoView {
    let rect = move || {
        let (w, h) = size.get();
        let k = 16.0 / w.max(h).max(1.0);
        let (w, h) = (w * k, h * k);
        (12.0 - w / 2.0, 12.0 - h / 2.0, w, h)
    };
    view! {
        <svg class="icon ratio-glyph" width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
            <rect
                x=move || format!("{:.2}", rect().0)
                y=move || format!("{:.2}", rect().1)
                width=move || format!("{:.2}", rect().2)
                height=move || format!("{:.2}", rect().3)
                style=move || {
                    let (x, y, w, h) = rect();
                    format!("x: {x:.2}px; y: {y:.2}px; width: {w:.2}px; height: {h:.2}px")
                }
                rx="2.5"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
            ></rect>
        </svg>
    }
}

/// Lets a mouse move the proportions when they run off the side: the wheel
/// scrolls them, and so does a drag, which then doesn't count as a press on
/// the chip it started on.
fn scroll_sideways(strip: &web_sys::HtmlElement) -> Vec<dom::Watch> {
    let overflows = |el: &web_sys::HtmlElement| el.scroll_width() > el.client_width();
    let mut all = Vec::new();
    let el = strip.clone();
    all.push(dom::listen(strip, "wheel", false, move |e| {
        let Some(e) = e.dyn_ref::<web_sys::WheelEvent>() else {
            return;
        };
        if e.ctrl_key() || !overflows(&el) {
            return;
        }
        let d = if e.delta_x().abs() > e.delta_y().abs() {
            e.delta_x()
        } else {
            e.delta_y()
        };
        if d == 0.0 {
            return;
        }
        e.prevent_default();
        let unit = match e.delta_mode() {
            1 => 16.0,
            2 => f64::from(el.client_width()),
            _ => 1.0,
        };
        el.set_scroll_left(el.scroll_left() + (d * unit).round() as i32);
    }));
    // Where a mouse drag started, the scroll then, and whether it has moved.
    let drag = Rc::new(Cell::new(None::<(f64, i32, bool)>));
    {
        let (el, drag) = (strip.clone(), Rc::clone(&drag));
        all.push(dom::listen(strip, "pointerdown", true, move |e| {
            let Some(e) = e.dyn_ref::<web_sys::PointerEvent>() else {
                return;
            };
            let mouse = e.pointer_type() == "mouse" && e.button() == 0;
            drag.set(
                (mouse && overflows(&el))
                    .then(|| (f64::from(e.client_x()), el.scroll_left(), false)),
            );
        }));
    }
    {
        let (el, drag) = (strip.clone(), Rc::clone(&drag));
        all.push(dom::listen(strip, "pointermove", true, move |e| {
            let Some(e) = e.dyn_ref::<web_sys::PointerEvent>() else {
                return;
            };
            let Some((x, left, moved)) = drag.get() else {
                return;
            };
            if e.buttons() & 1 == 0 {
                drag.set(None);
                return;
            }
            let dx = f64::from(e.client_x()) - x;
            if !moved && dx.abs() < STRIP_DRAG {
                return;
            }
            drag.set(Some((x, left, true)));
            el.set_scroll_left(left - dx.round() as i32);
        }));
    }
    {
        let drag = Rc::clone(&drag);
        all.push(dom::listen(strip, "pointerup", true, move |_| {
            if drag.get().is_some_and(|(_, _, moved)| !moved) {
                drag.set(None);
            }
        }));
    }
    all.push(dom::listen_capture(strip, "click", move |e| {
        if drag.take().is_some_and(|(_, _, moved)| moved) {
            e.prevent_default();
            e.stop_propagation();
        }
    }));
    all
}

/// Proportions, straightening, turning and flipping.
#[component]
fn CropTools(
    shape: RwSignal<Shape>,
    width: u32,
    height: u32,
    on_commit: Callback<Shape>,
    on_live: Callback<Shape>,
    on_settle: Callback<()>,
) -> impl IntoView {
    let custom_open = RwSignal::new(false);
    let custom_w = RwSignal::new("16".to_owned());
    let custom_h = RwSignal::new("10".to_owned());
    let custom_on = move || custom_open.get() || !shape.with(|s| s.aspect.is_preset());
    let apply_custom = move || {
        if let (Some(w), Some(h)) = (
            ratio_side(&custom_w.get_untracked()),
            ratio_side(&custom_h.get_untracked()),
        ) {
            change(shape, on_commit, |s| {
                s.with_aspect(Aspect::Fixed(w, h), width, height)
            });
        }
    };
    let pick = move |aspect: Aspect| {
        custom_open.set(false);
        change(shape, on_commit, |s| s.with_aspect(aspect, width, height));
    };
    let frame = Memo::new(move |_| shape.with(|s| s.frame(width, height)));
    let wide = Memo::new(move |_| shape.with(|s| s.wide));
    let aspect = Memo::new(move |_| shape.with(|s| s.aspect));
    let degrees = Memo::new(move |_| shape.with(Shape::degrees));
    let max = f64::from(MAX_STRAIGHTEN) / 10.0;

    let strip: NodeRef<html::Div> = NodeRef::new();
    let watches = StoredValue::new_local(Vec::<dom::Watch>::new());
    strip.on_load(move |el| watches.set_value(scroll_sideways(&el)));
    on_cleanup({
        let watches = SendWrapper::new(watches);
        move || {
            if let Some(all) = watches.try_update_value(std::mem::take) {
                for watch in all {
                    watch.stop();
                }
            }
        }
    });

    let chip_on = move |a: Aspect| aspect.get() == a && !custom_on();
    let fixed = RATIOS
        .into_iter()
        .map(|pair| {
            let wh = move || oriented(pair, wide.get());
            let on = move || {
                let (w, h) = wh();
                chip_on(Aspect::Fixed(w, h))
            };
            view! {
                <button
                    type="button"
                    class="ratio"
                    class:on=on
                    aria-pressed=move || on().to_string()
                    on:click=move |_| {
                        let (w, h) = oriented(pair, wide.get_untracked());
                        pick(Aspect::Fixed(w, h));
                    }
                >
                    <RatioGlyph size=Signal::derive(move || {
                        let (w, h) = wh();
                        (f64::from(w), f64::from(h))
                    }) />
                    <span class="num">
                        {move || {
                            let (w, h) = wh();
                            format!("{w}:{h}")
                        }}
                    </span>
                </button>
            }
        })
        .collect_view();

    view! {
        <div
            node_ref=strip
            class="tool-group ratios"
            role="group"
            aria-label=move || t!("editor-ratio")
        >
            <button
                type="button"
                class="ratio"
                class:on=move || chip_on(Aspect::Free)
                aria-pressed=move || chip_on(Aspect::Free).to_string()
                on:click=move |_| pick(Aspect::Free)
            >
                <Ico icon=Icon::RatioFree />
                <span>{move || t!("editor-ratio-free")}</span>
            </button>
            <button
                type="button"
                class="ratio"
                class:on=move || chip_on(Aspect::Original)
                aria-pressed=move || chip_on(Aspect::Original).to_string()
                on:click=move |_| pick(Aspect::Original)
            >
                <RatioGlyph size=frame />
                <span>{move || t!("editor-ratio-original")}</span>
            </button>
            {fixed}
            <button
                type="button"
                class="ratio"
                class:on=custom_on
                aria-pressed=move || custom_on().to_string()
                aria-expanded=move || custom_open.get().to_string()
                on:click=move |_| {
                    if let Aspect::Fixed(w, h) = aspect.get_untracked() {
                        custom_w.set(w.to_string());
                        custom_h.set(h.to_string());
                    }
                    custom_open.update(|o| *o = !*o);
                }
            >
                <Ico icon=Icon::RatioCustom />
                <span>{move || t!("editor-ratio-custom")}</span>
            </button>
            <button
                type="button"
                class="icon-btn swap"
                aria-label=move || t!("editor-ratio-swap")
                title=move || t!("editor-ratio-swap")
                on:click=move |_| change(shape, on_commit, |s| s.swap_orientation(width, height))
            >
                <span class="swap-icon" class:wide=move || wide.get()>
                    <Ico icon=Icon::Orientation />
                </span>
            </button>
        </div>
        <Show when=move || custom_open.get()>
            <div class="custom-ratio">
                <input
                    type="number"
                    inputmode="numeric"
                    min="1"
                    max=MAX_RATIO_SIDE.to_string()
                    aria-label=move || t!("editor-ratio-width")
                    aria-invalid=move || ratio_side(&custom_w.get()).is_none().to_string()
                    prop:value=move || custom_w.get()
                    on:input=move |e| {
                        custom_w.set(event_target_value(&e));
                        apply_custom();
                    }
                />
                <span aria-hidden="true">":"</span>
                <input
                    type="number"
                    inputmode="numeric"
                    min="1"
                    max=MAX_RATIO_SIDE.to_string()
                    aria-label=move || t!("editor-ratio-height")
                    aria-invalid=move || ratio_side(&custom_h.get()).is_none().to_string()
                    prop:value=move || custom_h.get()
                    on:input=move |e| {
                        custom_h.set(event_target_value(&e));
                        apply_custom();
                    }
                />
            </div>
        </Show>
        <div class="tool-group dial" role="group" aria-label=move || t!("editor-straighten")>
            <button
                type="button"
                class="dial-value"
                class:set=move || degrees.get() != 0.0
                title=move || t!("editor-straighten-reset")
                aria-label=move || t!("editor-straighten-reset")
                on:click=move |_| change(shape, on_commit, |s| s.straightened(0.0))
            >
                {move || format!("{:.1}°", degrees.get())}
            </button>
            <div
                class="dial-track"
                style=move || format!("--at: {:.4}", (degrees.get() + max) / (2.0 * max))
            >
                <input
                    type="range"
                    min=format!("{}", -max)
                    max=format!("{max}")
                    step="0.1"
                    aria-label=move || t!("editor-straighten")
                    prop:value=move || format!("{:.1}", degrees.get())
                    on:input=move |e| {
                        if let Ok(v) = event_target_value(&e).parse::<f64>() {
                            on_live.run(shape.with_untracked(|s| s.straightened(v)));
                        }
                    }
                    on:change=move |_| on_settle.run(())
                />
            </div>
        </div>
        <div class="tool-group turns" role="group" aria-label=move || t!("editor-turn-flip")>
            <button
                type="button"
                class="icon-btn"
                title=move || t!("editor-turn-left")
                aria-label=move || t!("editor-turn-left")
                on:click=move |_| change(shape, on_commit, |s| s.turn(false))
            >
                <Ico icon=Icon::RotateLeft />
            </button>
            <button
                type="button"
                class="icon-btn"
                title=move || t!("editor-turn-right")
                aria-label=move || t!("editor-turn-right")
                on:click=move |_| change(shape, on_commit, |s| s.turn(true))
            >
                <Ico icon=Icon::RotateRight />
            </button>
            <button
                type="button"
                class="icon-btn"
                title=move || t!("editor-flip-h")
                aria-label=move || t!("editor-flip-h")
                on:click=move |_| change(shape, on_commit, |s| s.flip(true))
            >
                <Ico icon=Icon::Flip />
            </button>
            <button
                type="button"
                class="icon-btn flip-v"
                title=move || t!("editor-flip-v")
                aria-label=move || t!("editor-flip-v")
                on:click=move |_| change(shape, on_commit, |s| s.flip(false))
            >
                <Ico icon=Icon::Flip />
            </button>
        </div>
    }
}

/// A drag of the crop in progress: what is dragged, from where, and the box
/// it started with.
#[derive(Clone, Copy)]
struct Drag {
    handle: Handle,
    from: (f64, f64),
    start: CropBox,
}

/// The picture as it will be saved, with the crop box over it. The room
/// around it takes the pointer too, so a drag can go past the picture.
#[component]
fn ImageCanvas(
    src: String,
    shape: RwSignal<Shape>,
    spin: RwSignal<i32>,
    width: u32,
    height: u32,
    on_live: Callback<Shape>,
    on_settle: Callback<()>,
) -> impl IntoView {
    let canvas: NodeRef<html::Div> = NodeRef::new();
    let room = RwSignal::new(None::<(f64, f64)>);
    let drag = StoredValue::new(None::<Drag>);
    let dragging = RwSignal::new(false);
    let watch = StoredValue::new_local(None::<dom::Watch>);
    canvas.on_load(move |el| {
        let w = dom::on_size(&el, move |w, h| {
            room.try_set(Some((w, h)));
        });
        watch.set_value(Some(w));
    });
    on_cleanup({
        let watch = SendWrapper::new(watch);
        move || {
            if let Some(w) = watch.try_update_value(Option::take).flatten() {
                w.stop();
            }
        }
    });

    let frame = Memo::new(move |_| shape.with(|s| s.frame(width, height)));
    let size = Memo::new(move |_| {
        let (fw, fh) = frame.get();
        room.get().map(|r| frame_size(fw, fh, r))
    });
    let ratio = move || {
        let (fw, fh) = frame.get_untracked();
        shape.with_untracked(|s| s.aspect.ratio(fw, fh))
    };

    let begin = move |handle: Handle, e: PointerEvent| {
        e.stop_propagation();
        e.prevent_default();
        if !e.is_primary() || (e.pointer_type() == "mouse" && e.button() != 0) {
            return;
        }
        drag.set_value(Some(Drag {
            handle,
            from: (f64::from(e.client_x()), f64::from(e.client_y())),
            start: shape.with_untracked(|s| s.crop),
        }));
        dragging.set(true);
        if let Some(el) = canvas.get_untracked() {
            let _ = el.set_pointer_capture(e.pointer_id());
        }
    };
    let moved = move |e: PointerEvent| {
        let (Some(d), Some((w, h))) = (drag.get_value(), size.get_untracked()) else {
            return;
        };
        if !e.is_primary() {
            return;
        }
        let (fw, fh) = frame.get_untracked();
        // Pointer pixels over frame pixels, as picture pixels.
        let dx = (f64::from(e.client_x()) - d.from.0) * fw / w;
        let dy = (f64::from(e.client_y()) - d.from.1) * fh / h;
        let mut next = shape.get_untracked();
        next.crop = d.start.dragged(d.handle, dx, dy, fw, fh, ratio());
        on_live.run(next);
    };
    let end_drag = move || {
        if drag.get_value().is_some() {
            drag.set_value(None);
            dragging.set(false);
            on_settle.run(());
        }
    };
    // Arrow keys on the box move it, and on a handle move that side or corner.
    let key = move |handle: Handle, e: KeyboardEvent| {
        let Some(way) = arrow(&e.key()) else {
            return;
        };
        e.prevent_default();
        e.stop_propagation();
        let mut next = shape.get_untracked();
        next.crop = next
            .crop
            .nudged(handle, way, e.shift_key(), frame.get_untracked(), ratio());
        on_live.run(next);
    };
    let key_up = move |e: KeyboardEvent| {
        if arrow(&e.key()).is_some() {
            on_settle.run(());
        }
    };

    let crop = Memo::new(move |_| shape.with(|s| s.crop));
    let pixels = Memo::new(move |_| shape.with(|s| s.cropped_size(width, height)));
    let handles = Handle::ALL
        .into_iter()
        .map(|(handle, name)| {
            view! {
                <span
                    class=format!("crop-handle {name}")
                    tabindex="0"
                    aria-label=move || handle_label(handle)
                    on:pointerdown=move |e| begin(handle, e)
                    on:keydown=move |e| key(handle, e)
                    on:keyup=key_up
                    on:blur=move |_| on_settle.run(())
                ></span>
            }
        })
        .collect_view();

    view! {
        <div
            node_ref=canvas
            class="editor-canvas"
            class:dragging=move || dragging.get()
            on:pointermove=moved
            on:pointerup=move |_| end_drag()
            on:pointercancel=move |_| end_drag()
            on:lostpointercapture=move |_| end_drag()
        >
            <div
                class="editor-frame"
                style=move || match size.get() {
                    Some((w, h)) => format!("width: {w:.2}px; height: {h:.2}px"),
                    None => "visibility: hidden".to_owned(),
                }
            >
                <div class="editor-clip">
                    <img
                        src=src
                        alt=""
                        draggable="false"
                        style=move || {
                            let size = size.get().unwrap_or((1.0, 1.0));
                            shape.with(|s| picture_style(&s.edit, spin.get(), size, width, height))
                        }
                    />
                    <div class="crop-shade" style=move || crop.get().style()></div>
                </div>
                <div
                    class="crop-box"
                    style=move || crop.get().style()
                    tabindex="0"
                    aria-label=move || handle_label(Handle::Move)
                    on:pointerdown=move |e| begin(Handle::Move, e)
                    on:keydown=move |e| key(Handle::Move, e)
                    on:keyup=key_up
                    on:blur=move |_| on_settle.run(())
                >
                    <span class="crop-grid"></span>
                    {handles}
                    <span class="crop-size">
                        {move || {
                            let (w, h) = pixels.get();
                            format!("{w} × {h}")
                        }}
                    </span>
                </div>
            </div>
        </div>
    }
}

/// What a drag on the timeline holds.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Grab {
    Trim(TrimEnd),
    Playhead,
}

/// The part of the video to keep: a timeline with a handle at each end and
/// the video's place on it, and the same ends as fields to type in.
#[component]
fn TrimTools(
    duration: i64,
    start: RwSignal<String>,
    end: RwSignal<String>,
    playhead: RwSignal<i64>,
    sliding: RwSignal<bool>,
    video: NodeRef<html::Video>,
    on_seek: Callback<i64>,
) -> impl IntoView {
    let track: NodeRef<html::Div> = NodeRef::new();
    let grab = StoredValue::new(None::<Grab>);
    let shown = Memo::new(move |_| start.with(|s| end.with(|e| trim_shown(s, e, duration))));
    let time_of = move |e: &PointerEvent| -> i64 {
        track.get_untracked().map_or(0, |el| {
            let r = el.get_bounding_client_rect();
            time_at(f64::from(e.client_x()), r.left(), r.width(), duration)
        })
    };
    // One end moves; the fields say where both are, and the video shows the
    // frame there.
    let put = move |which: TrimEnd, ms: i64| {
        let (s, e) = shown.get_untracked();
        let (s, e) = moved_trim(s, e, which, ms, duration);
        for (field, ms) in [(start, s), (end, e)] {
            let text = seconds_text(ms);
            if field.with_untracked(|t| *t != text) {
                field.set(text);
            }
        }
        on_seek.run(match which {
            TrimEnd::Start => s,
            TrimEnd::End => e,
        });
    };
    let follow = move |g: Grab, e: &PointerEvent| {
        let ms = time_of(e);
        match g {
            Grab::Trim(which) => put(which, ms),
            Grab::Playhead => on_seek.run(ms),
        }
    };
    let begin = move |g: Grab, e: &PointerEvent| {
        if !e.is_primary() || (e.pointer_type() == "mouse" && e.button() != 0) {
            return;
        }
        e.prevent_default();
        grab.set_value(Some(g));
        sliding.set(true);
        if let Some(el) = track.get_untracked() {
            let _ = el.set_pointer_capture(e.pointer_id());
        }
        // A handle stays where it is until it moves; the track seeks at once.
        if g == Grab::Playhead {
            follow(g, e);
        }
    };
    let moved = move |e: PointerEvent| {
        if let Some(g) = grab.get_value() {
            follow(g, &e);
        }
    };
    let release = move || {
        if grab.get_value().is_some() {
            grab.set_value(None);
            sliding.set(false);
        }
    };
    let now = move || {
        video.get_untracked().and_then(|v| {
            let ms = (v.current_time() * 1000.0).round();
            ms.is_finite().then_some(ms.max(0.0) as i64)
        })
    };

    let handle = move |which: TrimEnd| {
        let at = move || {
            let (s, e) = shown.get();
            match which {
                TrimEnd::Start => s,
                TrimEnd::End => e,
            }
        };
        let (class, label) = match which {
            TrimEnd::Start => (
                "timeline-handle start",
                Signal::derive(|| t!("editor-start")),
            ),
            TrimEnd::End => ("timeline-handle end", Signal::derive(|| t!("editor-end"))),
        };
        view! {
            <div
                class=class
                role="slider"
                tabindex="0"
                aria-label=move || label.get()
                aria-valuemin="0"
                aria-valuemax=seconds_text(duration)
                aria-valuenow=move || seconds_text(at())
                style=move || format!("left: {:.3}%", share(at(), duration) * 100.0)
                on:pointerdown=move |e: PointerEvent| {
                    e.stop_propagation();
                    begin(Grab::Trim(which), &e);
                }
                on:keydown=move |e: KeyboardEvent| {
                    let (s, en) = shown.get_untracked();
                    let from = if which == TrimEnd::Start { s } else { en };
                    if let Some(ms) = trim_key(&e.key(), e.shift_key(), from, duration) {
                        e.prevent_default();
                        put(which, ms);
                    }
                }
            >
                <span class="timeline-time">{move || seconds_text(at())}</span>
            </div>
        }
    };

    let keeps = move || {
        let (s, e) = (start.get(), end.get());
        match trim_of(&s, &e, duration) {
            Some(t) if t.end_ms > t.start_ms => t!(
                "editor-keeps",
                length = format_duration(t.end_ms - t.start_ms).unwrap_or_default(),
                total = format_duration(duration).unwrap_or_default()
            ),
            _ => t!("editor-bad-time"),
        }
    };

    view! {
        <div class="tool-group timeline-group" role="group" aria-label=move || t!("editor-timeline")>
            <div
                node_ref=track
                class="timeline"
                on:pointerdown=move |e: PointerEvent| begin(Grab::Playhead, &e)
                on:pointermove=moved
                on:pointerup=move |_| release()
                on:pointercancel=move |_| release()
                on:lostpointercapture=move |_| release()
            >
                <div class="timeline-track">
                    <div
                        class="timeline-range"
                        style=move || {
                            let (s, e) = shown.get();
                            format!(
                                "left: {:.3}%; right: {:.3}%",
                                share(s, duration) * 100.0,
                                (1.0 - share(e, duration)) * 100.0,
                            )
                        }
                    ></div>
                </div>
                <div
                    class="timeline-playhead"
                    aria-hidden="true"
                    style=move || format!("left: {:.3}%", share(playhead.get(), duration) * 100.0)
                ></div>
                {handle(TrimEnd::Start)}
                {handle(TrimEnd::End)}
            </div>
            <div class="timeline-scale" aria-hidden="true">
                <span>{format_duration(0)}</span>
                <span>{format_duration(duration)}</span>
            </div>
        </div>
        <div class="trim-fields">
            <TrimField
                id="trim-start"
                label=Signal::derive(|| t!("editor-start"))
                value=start
                on_now=Callback::new(move |()| {
                    if let Some(ms) = now() {
                        start.set(seconds_text(ms));
                    }
                })
            />
            <TrimField
                id="trim-end"
                label=Signal::derive(|| t!("editor-end"))
                value=end
                on_now=Callback::new(move |()| {
                    if let Some(ms) = now() {
                        end.set(seconds_text(ms.min(duration)));
                    }
                })
            />
        </div>
        <p class="muted hint">{keeps} " " {move || t!("editor-keyframe-hint")}</p>
    }
}

#[component]
fn TrimField(
    id: &'static str,
    label: Signal<String>,
    value: RwSignal<String>,
    on_now: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="field">
            <label for=id>{move || label.get()}</label>
            <div class="trim-row">
                <input
                    id=id
                    inputmode="decimal"
                    spellcheck="false"
                    autocomplete="off"
                    aria-invalid=move || parse_seconds(&value.get()).is_none().to_string()
                    prop:value=move || value.get()
                    on:input=move |e| value.set(event_target_value(&e))
                />
                <button
                    type="button"
                    class="btn tonal small"
                    title=move || t!("editor-now-title")
                    on:click=move |_| on_now.run(())
                >
                    {move || t!("editor-now")}
                </button>
            </div>
        </div>
    }
}
