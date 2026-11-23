//! The stage: the picture or video, with zoom and pan for pictures, previous
//! and next, fullscreen, and the keys and gestures for all of them. The booru
//! browser's post page has it too.

use std::time::Duration;

use leptos::ev::{MouseEvent, PointerEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use send_wrapper::SendWrapper;
use starbooru_core::{MediaKind, Post};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, KeyboardEvent, WheelEvent};

use crate::components::icons::{Ico, Icon};
use crate::dom;
use crate::logic::viewer::{
    Tap, css_url, double_tap, enter_class, fit_style, resist, sideways, speed, swipe_turns,
};
use crate::logic::zoom::{MAX_SCALE, STEP, WheelDelta, WheelTurn, Zoom, wheel_factor, wheel_zooms};

/// In fullscreen, the controls rest after this long without input.
const REST_MS: u64 = 2800;
/// A touch that lifts within this long, without moving, is a tap.
const TAP_MS: f64 = 300.0;
/// The strip at the foot of a video where its own controls are, in pixels.
const VIDEO_CONTROLS: f64 = 64.0;

/// What the fingers on the stage are doing. Kept out of signals: nothing on
/// the page needs to follow each move.
#[derive(Default)]
struct Fingers {
    /// Pointer id, and where it was last.
    down: Vec<(i32, f64, f64)>,
    /// When and where the first finger touched.
    start: Option<(f64, f64, f64)>,
    /// `(time, x)` of a swipe, for its speed as it lets go.
    trail: Vec<(f64, f64)>,
    /// Once decided: whether the finger went sideways.
    sideways: Option<bool>,
    /// A second finger came: no swipe and no tap.
    pinched: bool,
    last_tap: Option<Tap>,
    /// The last pointer was a mouse, whose double click zooms.
    mouse: bool,
}

impl Fingers {
    fn get(&self, id: i32) -> Option<(f64, f64)> {
        self.down
            .iter()
            .find(|(p, _, _)| *p == id)
            .map(|&(_, x, y)| (x, y))
    }

    fn other(&self, id: i32) -> Option<(f64, f64)> {
        self.down
            .iter()
            .find(|(p, _, _)| *p != id)
            .map(|&(_, x, y)| (x, y))
    }

    fn put(&mut self, id: i32, x: f64, y: f64) {
        match self.down.iter_mut().find(|(p, _, _)| *p == id) {
            Some(slot) => *slot = (id, x, y),
            None => self.down.push((id, x, y)),
        }
    }

    fn lift(&mut self, id: i32) {
        self.down.retain(|(p, _, _)| *p != id);
    }
}

/// What the stage shows, from this server or another booru.
#[derive(Clone, Debug, PartialEq)]
pub struct Shown {
    /// Tells one post from the next, so a new one makes its entrance.
    pub key: String,
    pub kind: MediaKind,
    pub size: Option<(i32, i32)>,
    /// The picture's main colour, which fills its box while it loads.
    pub tint: Option<String>,
    pub alt: String,
    /// A video's still, before it plays.
    pub poster: Option<String>,
    /// The stage's name for a screen reader.
    pub label: String,
}

impl Shown {
    /// A post on this server.
    pub fn of(p: &Post) -> Self {
        Self {
            key: p.id.to_string(),
            kind: p.kind,
            size: p.width.zip(p.height),
            tint: p.palette.first().cloned(),
            alt: p.tags.join(" "),
            poster: p.sample_url.as_deref().map(super::absolute),
            label: t!("post-number", id = p.id),
        }
    }
}

fn document() -> Option<web_sys::Document> {
    web_sys::window()?.document()
}

fn now() -> f64 {
    js_sys::Date::now()
}

/// Calls a browser method that answers with a promise, and lets a refusal go
/// quietly: fullscreen can be refused, and the stage then covers the window.
fn call_quietly(target: &JsValue, method: &str) {
    let Ok(f) = js_sys::Reflect::get(target, &JsValue::from_str(method)) else {
        return;
    };
    let Some(f) = f.dyn_ref::<js_sys::Function>() else {
        return;
    };
    if let Ok(answer) = f.call0(target)
        && let Ok(promise) = answer.dyn_into::<js_sys::Promise>()
    {
        spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
        });
    }
}

/// Whether `target` is inside an element `selector` matches.
fn within(target: Option<web_sys::EventTarget>, selector: &str) -> bool {
    target
        .and_then(|t| t.dyn_into::<Element>().ok())
        .and_then(|el| el.closest(selector).ok().flatten())
        .is_some()
}

#[component]
pub fn Stage(
    #[prop(into)] shown: Signal<Option<Shown>>,
    /// What the stage shows: the sample or the original, absolute.
    #[prop(into)]
    src: Signal<String>,
    #[prop(into)] thumb: Signal<Option<String>>,
    zoom: RwSignal<Zoom>,
    full: RwSignal<bool>,
    /// The ends of the list, once the neighbours are known.
    #[prop(into)]
    first: Signal<bool>,
    #[prop(into)] last: Signal<bool>,
    /// The way the last step went, for the entrance.
    #[prop(into)]
    enter: Signal<Option<bool>>,
    /// The tag editor is open: the keys are left to it, and swipes still
    /// turn.
    #[prop(into)]
    editing: Signal<bool>,
    on_step: Callback<bool>,
    on_back: Callback<()>,
    /// `E` opens the tag editor.
    on_edit: Callback<()>,
) -> impl IntoView {
    let stage: NodeRef<html::Section> = NodeRef::new();
    let kind = Memo::new(move |_| shown.with(|s| s.as_ref().map(|s| s.kind)));
    let zoomable = move || kind.get().is_some_and(|k| k != MediaKind::Video);
    let fingers = StoredValue::new_local(Fingers::default());
    let dragging = RwSignal::new(false);
    let swipe_x = RwSignal::new(None::<f64>);
    let rest = RwSignal::new(false);
    let rest_timer = StoredValue::new_local(None::<TimeoutHandle>);
    let turn = StoredValue::new_local(WheelTurn::default());

    // On its way back to its size the picture is still bigger than the stage
    // for a moment, and stays inside it until it's done.
    let shrinking = RwSignal::new(false);
    let shrink_timer = StoredValue::new_local(None::<TimeoutHandle>);
    Effect::watch(
        move || zoom.with(Zoom::is_zoomed),
        move |grown, before, _| {
            if let Some(timer) = shrink_timer.try_update_value(Option::take).flatten() {
                timer.clear();
            }
            if !grown && before == Some(&true) {
                shrinking.set(true);
                let timer = set_timeout_with_handle(
                    move || {
                        shrinking.try_set(false);
                    },
                    Duration::from_millis(200),
                );
                shrink_timer.set_value(timer.ok());
            }
        },
        false,
    );

    // From the stage's centre, where the picture's transform starts.
    let from_centre = move |x: f64, y: f64| -> (f64, f64) {
        stage.get_untracked().map_or((0.0, 0.0), |el| {
            let r = el.get_bounding_client_rect();
            (
                x - r.left() - r.width() / 2.0,
                y - r.top() - r.height() / 2.0,
            )
        })
    };
    // The picture's size at fit. Panning stays within what the zoom added to it.
    let bounds = move || -> (f64, f64) {
        document()
            .and_then(|d| d.get_element_by_id("stage-img"))
            .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
            .map_or((0.0, 0.0), |el| {
                (f64::from(el.offset_width()), f64::from(el.offset_height()))
            })
    };
    let zoom_by = move |factor: f64, at: (f64, f64)| {
        if zoomable() {
            zoom.update(|z| *z = z.zoom_at(factor, at, bounds()));
        }
    };

    // The controls come back on any input, and rest again a while after.
    let wake = move || {
        if let Some(timer) = rest_timer.try_update_value(Option::take).flatten() {
            timer.clear();
        }
        if rest.try_get_untracked() == Some(true) {
            rest.set(false);
        }
        if full.try_get_untracked() != Some(true) {
            return;
        }
        let timer = set_timeout_with_handle(
            move || {
                if full.try_get_untracked() != Some(true) {
                    return;
                }
                // Not while the pointer is on them or a key has put focus there:
                // the next move starts the wait again.
                let held = stage
                    .try_get_untracked()
                    .flatten()
                    .and_then(|el| {
                        el.query_selector(".stage-tools:hover, .stage-btn:hover, :focus-visible")
                            .ok()
                            .flatten()
                    })
                    .is_some();
                if !held {
                    rest.try_set(true);
                }
            },
            Duration::from_millis(REST_MS),
        );
        rest_timer.try_set_value(timer.ok());
    };

    let set_full = move |on: bool| {
        if full.get_untracked() == on {
            return;
        }
        full.set(on);
        zoom.set(Zoom::default());
        wake();
        let Some(doc) = document() else {
            return;
        };
        if on {
            if let Some(el) = stage.get_untracked() {
                call_quietly(&el, "requestFullscreen");
            }
        } else if doc.fullscreen_element().is_some() {
            call_quietly(&doc, "exitFullscreen");
        }
    };

    let press = move |forward: bool| {
        let blocked = if forward {
            last.get_untracked()
        } else {
            first.get_untracked()
        };
        if !blocked {
            on_step.run(forward);
        }
    };

    // Keys, the wheel, and fullscreen ending from the browser's own controls.
    let watches = StoredValue::new_local(Vec::<dom::Watch>::new());
    stage.on_load(move |el| {
        let mut all = Vec::new();
        all.push(dom::listen(&el, "wheel", false, move |e| {
            let Some(e) = e.dyn_ref::<WheelEvent>() else {
                return;
            };
            if !zoomable() {
                return;
            }
            let on_picture = within(e.target(), "#stage-img");
            let (y, _) = dom::scroll();
            let zoomed = zoom.with_untracked(Zoom::is_zoomed);
            let decide = || {
                wheel_zooms(
                    on_picture,
                    full.get_untracked(),
                    zoomed,
                    e.ctrl_key(),
                    e.delta_y() > 0.0,
                    y > 0.5,
                )
            };
            let zooms = turn
                .try_update_value(|t| t.zooms(e.time_stamp(), decide))
                .unwrap_or(false);
            if !zooms {
                return;
            }
            e.prevent_default();
            let factor = wheel_factor(WheelDelta::new(e.delta_mode(), e.delta_y()));
            zoom_by(
                factor,
                from_centre(f64::from(e.client_x()), f64::from(e.client_y())),
            );
        }));
        if let Some(doc) = document() {
            all.push(dom::listen(&doc, "fullscreenchange", true, move |_| {
                let out = document().is_some_and(|d| d.fullscreen_element().is_none());
                if out && full.get_untracked() {
                    full.set(false);
                    zoom.set(Zoom::default());
                }
            }));
        }
        if let Some(window) = web_sys::window() {
            all.push(dom::listen(&window, "keydown", false, move |e| {
                let Some(e) = e.dyn_ref::<KeyboardEvent>() else {
                    return;
                };
                if full.get_untracked() {
                    wake();
                }
                if e.default_prevented()
                    || e.ctrl_key()
                    || e.meta_key()
                    || e.alt_key()
                    || dom::typing(e.target())
                    || editing.get_untracked()
                {
                    return;
                }
                // A video's own controls take the arrows.
                let on_video = within(e.target(), "video");
                match e.key().as_str() {
                    "ArrowLeft" if !on_video => press(false),
                    "ArrowRight" if !on_video => press(true),
                    "+" | "=" if zoomable() => zoom_by(STEP, (0.0, 0.0)),
                    "-" | "_" if zoomable() => zoom_by(1.0 / STEP, (0.0, 0.0)),
                    "0" if zoomable() => zoom.set(Zoom::default()),
                    "e" | "E" if !full.get_untracked() => on_edit.run(()),
                    "f" | "F" => set_full(!full.get_untracked()),
                    "Escape" if full.get_untracked() => set_full(false),
                    "Escape" => on_back.run(()),
                    _ => return,
                }
                e.prevent_default();
            }));
            all.push(dom::on_window("resize", move || {
                if zoom.with_untracked(Zoom::is_zoomed) {
                    zoom.set(Zoom::default());
                }
            }));
        }
        watches.update_value(|w| w.extend(all));
    });
    on_cleanup({
        let watches = SendWrapper::new(watches);
        let timers = SendWrapper::new((rest_timer, shrink_timer));
        move || {
            if let Some(all) = watches.try_update_value(std::mem::take) {
                for watch in all {
                    watch.stop();
                }
            }
            for timer in [&timers.0, &timers.1] {
                if let Some(t) = timer.try_update_value(Option::take).flatten() {
                    t.clear();
                }
            }
            // Leaving the page in fullscreen leaves fullscreen too.
            if let Some(doc) = document()
                && doc.fullscreen_element().is_some()
            {
                call_quietly(&doc, "exitFullscreen");
            }
        }
    });

    // Fingers and the mouse.
    let down = move |e: PointerEvent| {
        if full.get_untracked() {
            wake();
        }
        let mouse = e.pointer_type() == "mouse";
        fingers.update_value(|f| f.mouse = mouse);
        if within(e.target(), ".stage-btn, .stage-tools") {
            return;
        }
        // A video's own controls, along its foot, keep their drags.
        if let Some(video) = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlVideoElement>().ok())
            && f64::from(e.client_y()) > video.get_bounding_client_rect().bottom() - VIDEO_CONTROLS
        {
            return;
        }
        let (x, y) = (f64::from(e.client_x()), f64::from(e.client_y()));
        let zoomed = zoom.with_untracked(Zoom::is_zoomed);
        if mouse {
            // A mouse only drags a picture that has grown.
            if e.button() != 0 || !zoomed {
                return;
            }
            e.prevent_default();
            if let Some(el) = stage.get_untracked() {
                let _ = el.set_pointer_capture(e.pointer_id());
            }
        }
        let t = now();
        fingers.update_value(|f| {
            f.put(e.pointer_id(), x, y);
            if f.down.len() == 1 {
                f.start = Some((t, x, y));
                f.trail = vec![(t, x)];
                f.sideways = None;
                f.pinched = false;
            } else {
                f.pinched = true;
            }
        });
        if fingers.with_value(|f| f.pinched) {
            swipe_x.set(None);
        }
        dragging.set(zoomed || fingers.with_value(|f| f.pinched));
    };

    let moved = move |e: PointerEvent| {
        if full.get_untracked() {
            wake();
        }
        let id = e.pointer_id();
        let Some((px, py)) = fingers.with_value(|f| f.get(id)) else {
            return;
        };
        let (x, y) = (f64::from(e.client_x()), f64::from(e.client_y()));
        let two = fingers.with_value(|f| f.down.len() >= 2);
        if two {
            if let Some((ox, oy)) = fingers.with_value(|f| f.other(id)) {
                let before = (px - ox).hypot(py - oy);
                let after = (x - ox).hypot(y - oy);
                if before > 1.0 {
                    zoom_by(after / before, from_centre((x + ox) / 2.0, (y + oy) / 2.0));
                }
            }
        } else if zoom.with_untracked(Zoom::is_zoomed) {
            zoom.update(|z| *z = z.pan(x - px, y - py, bounds()));
        } else if !fingers.with_value(|f| f.mouse || f.pinched) {
            let Some((_, sx, sy)) = fingers.with_value(|f| f.start) else {
                return;
            };
            let (dx, dy) = (x - sx, y - sy);
            let way = fingers.with_value(|f| f.sideways).or_else(|| {
                let way = sideways(dx, dy);
                fingers.update_value(|f| f.sideways = way);
                way
            });
            if way == Some(true) {
                let at_end = if dx < 0.0 {
                    last.get_untracked()
                } else {
                    first.get_untracked()
                };
                swipe_x.set(Some(resist(dx, at_end)));
                fingers.update_value(|f| f.trail.push((now(), x)));
            }
        }
        fingers.update_value(|f| f.put(id, x, y));
    };

    let up = move |e: PointerEvent| {
        let id = e.pointer_id();
        let t = now();
        let (x, y) = (f64::from(e.client_x()), f64::from(e.client_y()));
        let single = fingers.with_value(|f| f.down.len() == 1 && f.get(id).is_some());
        fingers.update_value(|f| f.lift(id));
        dragging.set(
            fingers.with_value(|f| !f.down.is_empty()) && zoom.with_untracked(Zoom::is_zoomed),
        );
        let (start, way, pinched, mouse) =
            fingers.with_value(|f| (f.start, f.sideways, f.pinched, f.mouse));
        if !single || pinched || mouse {
            return;
        }
        let Some((t0, sx, _)) = start else {
            return;
        };
        if way == Some(true) {
            let width = stage
                .get_untracked()
                .map_or(0.0, |el| el.get_bounding_client_rect().width());
            let gone = fingers.with_value(|f| speed(&f.trail));
            swipe_x.set(None);
            if let Some(forward) = swipe_turns(x - sx, gone, width) {
                press(forward);
            }
            return;
        }
        if way.is_none() && t - t0 <= TAP_MS {
            let tap = Tap { time: t, x, y };
            let second = fingers.with_value(|f| double_tap(f.last_tap, tap));
            fingers.update_value(|f| f.last_tap = (!second).then_some(tap));
            if second && zoomable() {
                zoom.update(|z| *z = z.toggle_at(from_centre(x, y), bounds()));
            }
        }
    };

    let cancel = move |e: PointerEvent| {
        fingers.update_value(|f| {
            f.lift(e.pointer_id());
            f.sideways = None;
        });
        swipe_x.set(None);
        dragging.set(false);
    };

    let double_click = move |e: MouseEvent| {
        // A double tap is handled as it lifts.
        if !fingers.with_value(|f| f.mouse)
            || !zoomable()
            || within(e.target(), ".stage-btn, .stage-tools")
        {
            return;
        }
        let at = from_centre(f64::from(e.client_x()), f64::from(e.client_y()));
        zoom.update(|z| *z = z.toggle_at(at, bounds()));
    };

    // The media, made anew for each post so its entrance plays.
    let media_key = Memo::new(move |_| shown.with(|s| s.as_ref().map(|s| (s.key.clone(), s.kind))));
    let media = move || {
        let (_, kind) = media_key.get()?;
        let entrance = enter_class(enter.get_untracked());
        let size = shown.with_untracked(|s| s.as_ref().and_then(|s| s.size));
        let tint = shown.with_untracked(|s| {
            s.as_ref()
                .and_then(|s| s.tint.as_ref())
                .map(|hex| format!("; --tint: {hex}"))
                .unwrap_or_default()
        });
        let alt = shown.with_untracked(|s| s.as_ref().map(|s| s.alt.clone()).unwrap_or_default());
        let swiped = move || {
            swipe_x
                .get()
                .map(|x| format!("; translate: {x:.1}px 0"))
                .unwrap_or_default()
        };
        Some(match kind {
            MediaKind::Video => view! {
                <video
                    class=entrance
                    src=move || src.get()
                    poster=move || {
                        shown.with(|s| s.as_ref().and_then(|s| s.poster.clone())).or_else(|| thumb.get())
                    }
                    style=move || format!("{}{tint}{}", fit_style(size), swiped())
                    controls=true
                    playsinline=true
                    preload="metadata"
                    loop=true
                ></video>
            }
            .into_any(),
            MediaKind::Image | MediaKind::Animation => {
                let loaded = RwSignal::new(false);
                let style = move || {
                    let mut style = zoom.with(Zoom::css);
                    style.push_str(&fit_style(size));
                    style.push_str(&tint);
                    if let Some(t) = thumb.get() {
                        style.push_str("; --thumb: ");
                        style.push_str(&css_url(&t));
                    }
                    style.push_str(&swiped());
                    style
                };
                view! {
                    <img
                        id="stage-img"
                        class=entrance
                        class:loading=move || !loaded.get()
                        src=move || src.get()
                        alt=alt
                        draggable="false"
                        decoding="async"
                        fetchpriority="high"
                        style=style
                        on:load=move |_| loaded.set(true)
                    />
                }
                .into_any()
            }
        })
    };

    let z = move || zoom.get();
    view! {
        <section
            node_ref=stage
            class="stage"
            class:full=move || full.get()
            class:zoomed=move || z().is_zoomed()
            class:shrinking=move || !z().is_zoomed() && shrinking.get()
            class:dragging=move || dragging.get()
            class:swiping=move || swipe_x.with(Option::is_some)
            data-rest=move || rest.get().then_some("")
            aria-label=move || shown.with(|s| s.as_ref().map(|s| s.label.clone()))
            on:pointerdown=down
            on:pointermove=moved
            on:pointerup=up
            on:pointercancel=cancel
            on:dblclick=double_click
        >
            <canvas class="stage-glow" aria-hidden="true"></canvas>
            {media}
            <button
                type="button"
                class="stage-btn stage-back glass float"
                title=move || t!("post-back")
                aria-label=move || t!("post-back")
                on:click=move |_| on_back.run(())
            >
                <Ico icon=Icon::ArrowBack />
            </button>
            <Show when=move || !first.get()>
                <button
                    type="button"
                    class="stage-btn stage-prev glass float"
                    title=move || t!("post-prev-title")
                    aria-label=move || t!("post-prev")
                    on:click=move |_| press(false)
                >
                    <Ico icon=Icon::ChevronLeft size=32 />
                </button>
            </Show>
            <Show when=move || !last.get()>
                <button
                    type="button"
                    class="stage-btn stage-next glass float"
                    title=move || t!("post-next-title")
                    aria-label=move || t!("post-next")
                    on:click=move |_| press(true)
                >
                    <Ico icon=Icon::ChevronRight size=32 />
                </button>
            </Show>
            <div class="stage-tools glass float">
                <Show when=zoomable>
                    <button
                        type="button"
                        class="stage-btn"
                        title=move || t!("zoom-out-title")
                        aria-label=move || t!("zoom-out")
                        disabled=move || !z().is_zoomed()
                        on:click=move |_| zoom_by(1.0 / STEP, (0.0, 0.0))
                    >
                        <Ico icon=Icon::ZoomOut />
                    </button>
                    <button
                        type="button"
                        class="stage-zoom"
                        title=move || t!("zoom-fit-title")
                        aria-label=move || t!("zoom-reset")
                        on:click=move |_| zoom.set(Zoom::default())
                    >
                        {move || {
                            let z = z();
                            if z.is_zoomed() { format!("{}%", z.percent()) } else { t!("zoom-fit") }
                        }}
                    </button>
                    <button
                        type="button"
                        class="stage-btn"
                        title=move || t!("zoom-in-title")
                        aria-label=move || t!("zoom-in")
                        disabled=move || { z().scale >= MAX_SCALE }
                        on:click=move |_| zoom_by(STEP, (0.0, 0.0))
                    >
                        <Ico icon=Icon::ZoomIn />
                    </button>
                </Show>
                <button
                    type="button"
                    class="stage-btn"
                    title=move || if full.get() { t!("fullscreen-exit-title") } else { t!("fullscreen-title") }
                    aria-label=move || if full.get() { t!("fullscreen-exit") } else { t!("fullscreen") }
                    on:click=move |_| set_full(!full.get_untracked())
                >
                    {move || {
                        let icon = if full.get() { Icon::FullscreenExit } else { Icon::Fullscreen };
                        view! { <Ico icon=icon /> }
                    }}
                </button>
            </div>
        </section>
    }
}
