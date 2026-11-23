//! The video player of the viewer and the share page: the app's own controls
//! over a `<video>`, in place of the browser's (the rules are in
//! `logic/player.rs`). The page around it owns fullscreen, so the viewer's
//! floating controls and its stepping keep working there.
//!
//! A click on the picture plays or pauses it and a double click goes
//! fullscreen. A finger's tap plays a paused video, and otherwise brings the
//! controls up or puts them away; a double tap goes fullscreen. While the
//! video plays, the controls rest after a while without input.

use std::cell::Cell;
use std::time::Duration;

use leptos::ev::{FocusEvent, MouseEvent, PointerEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use send_wrapper::SendWrapper;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, HtmlElement, KeyboardEvent};

use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::icons::{Ico, Icon};
use crate::logic::player::{
    Act, BADGE_MS, Badge, Failure, Held, LEAVE_MS, Press, REST_MS, SPEEDS, Tap as TapDoes, act,
    badge_after, clock, failure, known, loaded, loops, may_rest, percent, played, share_at,
    speed_text, stepped, tap, time_at, times, unmuted_volume,
};
use crate::logic::viewer::{Tap, double_tap};
use crate::{dom, i18n, power, store};

/// A finger that lifts within this long, without moving, taps.
const TAP_MS: f64 = 300.0;
/// How far a tapping finger may wander, in pixels.
const TAP_SLOP: f64 = 10.0;

thread_local! {
    /// The speed picked last, for the next video in this tab.
    static SPEED: Cell<f64> = const { Cell::new(1.0) };
}

/// A finger down on the picture, which may become a tap.
#[derive(Clone, Copy, Debug)]
struct Down {
    id: i32,
    time: f64,
    x: f64,
    y: f64,
}

fn document() -> Option<web_sys::Document> {
    web_sys::window()?.document()
}

/// Whether `target` is inside an element `selector` matches.
fn within(target: Option<web_sys::EventTarget>, selector: &str) -> bool {
    target
        .and_then(|t| t.dyn_into::<Element>().ok())
        .and_then(|el| el.closest(selector).ok().flatten())
        .is_some()
}

/// Whether a key comes from a box the user types in. A slider isn't one.
fn in_text(target: Option<&web_sys::EventTarget>) -> bool {
    let Some(el) = target.and_then(|t| t.dyn_ref::<HtmlElement>()) else {
        return false;
    };
    if let Some(input) = el.dyn_ref::<web_sys::HtmlInputElement>() {
        return !matches!(
            input.type_().as_str(),
            "range" | "checkbox" | "radio" | "button"
        );
    }
    el.is_content_editable() || matches!(el.tag_name().as_str(), "TEXTAREA" | "SELECT")
}

/// A property of the document, where the browser has it.
fn document_prop(name: &str) -> Option<JsValue> {
    let doc = document()?;
    js_sys::Reflect::get(&doc, &JsValue::from_str(name))
        .ok()
        .filter(|v| !v.is_undefined() && !v.is_null())
}

/// The parts of the video loaded, in seconds.
fn ranges(v: &web_sys::HtmlMediaElement) -> Vec<(f64, f64)> {
    let r = v.buffered();
    (0..r.length())
        .filter_map(|i| Some((r.start(i).ok()?, r.end(i).ok()?)))
        .collect()
}

fn failure_text(f: Failure) -> String {
    match f {
        Failure::Network => t!("player-failed-network"),
        Failure::Format => t!("player-failed-format"),
        Failure::Missing => t!("player-failed"),
    }
}

/// Moves the progress along with every frame while the video plays: the
/// browser's own time updates come four times a second.
fn tick(
    video: NodeRef<html::Video>,
    now: RwSignal<f64>,
    paused: RwSignal<bool>,
    scrub: RwSignal<Option<i32>>,
    ticking: StoredValue<bool>,
) {
    request_animation_frame(move || {
        if paused.try_get_untracked() != Some(false) {
            ticking.try_set_value(false);
            return;
        }
        if scrub.try_get_untracked() == Some(None)
            && let Some(v) = video.try_get_untracked().flatten()
        {
            now.try_set(v.current_time());
        }
        tick(video, now, paused, scrub, ticking);
    });
}

/// A video with the app's controls. `style` sizes its box; `full` and
/// `on_full` are the page's fullscreen. While `keys` holds, the page's keys
/// work the video too (Space, K, J, L, M, F, digits, Shift with the arrows).
/// `on_retry` gets fresh addresses before the file loads again.
#[component]
pub fn Player(
    #[prop(into)] src: Signal<String>,
    #[prop(into)] poster: Signal<Option<String>>,
    /// More classes on the box, such as its entrance.
    #[prop(optional)]
    class: &'static str,
    #[prop(into)] style: Signal<String>,
    #[prop(into)] full: Signal<bool>,
    on_full: Callback<()>,
    #[prop(into)] keys: Signal<bool>,
    #[prop(optional)] autoplay: bool,
    #[prop(optional)] on_retry: Option<Callback<Callback<()>>>,
) -> impl IntoView {
    let root: NodeRef<html::Div> = NodeRef::new();
    let video: NodeRef<html::Video> = NodeRef::new();
    let seek: NodeRef<html::Div> = NodeRef::new();
    let bar: NodeRef<html::Div> = NodeRef::new();

    let paused = RwSignal::new(true);
    let started = RwSignal::new(false);
    let ended = RwSignal::new(false);
    let waiting = RwSignal::new(false);
    let now = RwSignal::new(0.0_f64);
    let duration = RwSignal::new(f64::NAN);
    let buffered = RwSignal::new(Vec::<(f64, f64)>::new());
    let kept = store::prefs_untracked();
    let muted = RwSignal::new(kept.video_muted);
    let volume = RwSignal::new(kept.video_volume);
    let rate = RwSignal::new(SPEED.with(Cell::get));
    let looping = RwSignal::new(false);
    // Once the user has said, the length no longer decides.
    let loop_chosen = StoredValue::new(false);
    let pip = RwSignal::new(false);
    let pip_offered =
        document_prop("pictureInPictureEnabled").and_then(|v| v.as_bool()) == Some(true);
    let failed = RwSignal::new(None::<Failure>);
    // The controls are resting.
    let idle = RwSignal::new(false);
    // The pointer dragging along the seek bar, and whether to play on after.
    let scrub = RwSignal::new(None::<i32>);
    let resume = StoredValue::new(false);
    // The share of the seek bar under the pointer, for the time shown there.
    let hover = RwSignal::new(None::<f64>);
    let menu = RwSignal::new(false);
    let badge = RwSignal::new(None::<Badge>);
    // Play or pause, just pressed on the picture, and a count so the same
    // one twice still shows.
    let flash = RwSignal::new(None::<(bool, u32)>);
    let ticking = StoredValue::new(false);
    // The last pointer was a mouse, which hovers and clicks; a finger taps.
    let mouse = StoredValue::new(true);
    let down = StoredValue::new(None::<Down>);
    let last_tap = StoredValue::new(None::<Tap>);
    let last_wake = StoredValue::new(f64::NEG_INFINITY);
    // Whether the focus came by a key. A clicked button keeps the focus,
    // and that shouldn't hold the bar up.
    let pressed_at = StoredValue::new(f64::NEG_INFINITY);
    let key_focus = StoredValue::new(false);
    let rest_timer = StoredValue::new_local(None::<TimeoutHandle>);
    let badge_timer = StoredValue::new_local(None::<TimeoutHandle>);
    let watches = StoredValue::new_local(Vec::<dom::Watch>::new());

    let el = move || video.try_get_untracked().flatten();
    let play = move || {
        if let Some(promise) = el().and_then(|v| v.play().ok()) {
            // Refused (no gesture yet, or paused again on the way): it stays
            // paused, and the controls say so.
            spawn_local(async move {
                let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
            });
        }
    };
    let toggle_play = move |show: bool| {
        let Some(v) = el() else {
            return;
        };
        let start = v.paused() || v.ended();
        if start {
            play();
        } else {
            let _ = v.pause();
        }
        if show {
            flash.update(|f| *f = Some((start, f.map_or(0, |(_, n)| n.wrapping_add(1)))));
        }
    };

    // The controls: up on any input, resting a while after if nothing holds
    // them.
    let held = move || -> Option<Held> {
        let bar_el = bar.try_get_untracked().flatten()?;
        let hovered =
            mouse.try_get_value() == Some(true) && bar_el.matches(":hover").unwrap_or(false);
        let focused = key_focus.try_get_value() == Some(true)
            && bar_el.query_selector(":focus").ok().flatten().is_some();
        Some(Held {
            paused: paused.try_get_untracked()?,
            scrubbing: scrub.try_get_untracked()?.is_some(),
            focused,
            menu: menu.try_get_untracked()?,
            hovered,
            failed: failed.try_get_untracked()?.is_some(),
        })
    };
    let clear_rest = move || {
        if let Some(timer) = rest_timer.try_update_value(Option::take).flatten() {
            timer.clear();
        }
    };
    let rest_after = move |ms: u64| {
        clear_rest();
        let timer = set_timeout_with_handle(
            move || {
                if held().is_some_and(may_rest) {
                    idle.try_set(true);
                }
            },
            Duration::from_millis(ms),
        );
        rest_timer.try_set_value(timer.ok());
    };
    let wake = move || {
        if idle.try_get_untracked() == Some(true) {
            idle.try_set(false);
        }
        rest_after(REST_MS);
    };

    let seek_to = move |t: f64| {
        if let Some(v) = el() {
            v.set_current_time(t);
            now.set(t);
        }
    };
    let seek_by = move |by: i32| {
        let from = el().map_or_else(|| now.get_untracked(), |v| v.current_time());
        let Some(t) = stepped(from, f64::from(by), duration.get_untracked()) else {
            return;
        };
        seek_to(t);
        badge.set(Some(badge_after(
            badge.get_untracked(),
            by,
            js_sys::Date::now(),
        )));
        if let Some(timer) = badge_timer.try_update_value(Option::take).flatten() {
            timer.clear();
        }
        let timer = set_timeout_with_handle(
            move || {
                badge.try_set(None);
            },
            Duration::from_millis(BADGE_MS as u64),
        );
        badge_timer.try_set_value(timer.ok());
    };
    // Somewhere else entirely: the steps so far no longer add up.
    let jump = move |share: f64| {
        if let Some(t) = time_at(share, duration.get_untracked()) {
            seek_to(t);
            badge.set(None);
        }
    };

    // Sound, kept for the next video and the next visit.
    let remember = move || {
        if let Some(v) = el() {
            let (level, silent) = (v.volume(), v.muted());
            store::update_prefs(|p| {
                p.video_volume = level;
                p.video_muted = silent;
            });
        }
    };
    let toggle_mute = move || {
        let Some(v) = el() else {
            return;
        };
        if v.muted() || v.volume() < 0.01 {
            v.set_volume(unmuted_volume(v.volume()));
            v.set_muted(false);
        } else {
            v.set_muted(true);
        }
        remember();
    };
    let set_level = move |level: f64| {
        if let Some(v) = el() {
            let level = if level.is_finite() {
                level.clamp(0.0, 1.0)
            } else {
                1.0
            };
            v.set_volume(level);
            v.set_muted(level < 0.01);
        }
    };
    let set_rate = move |r: f64| {
        if let Some(v) = el() {
            // The default too, so loading the file again keeps it.
            v.set_default_playback_rate(r);
            v.set_playback_rate(r);
        }
        SPEED.with(|s| s.set(r));
    };
    let toggle_loop = move || {
        loop_chosen.set_value(true);
        looping.update(|l| *l = !*l);
    };
    let toggle_pip = move || {
        if document_prop("pictureInPictureElement").is_some() {
            if let Some(doc) = document() {
                dom::call_quietly(&doc, "exitPictureInPicture");
            }
        } else if let Some(v) = el() {
            dom::call_quietly(&v, "requestPictureInPicture");
        }
    };

    // The speed and, on a narrow player, loop and picture in picture: a
    // panel over the bar.
    let opener = move || -> Option<HtmlElement> {
        let list = root
            .try_get_untracked()
            .flatten()?
            .query_selector_all(".player-opener")
            .ok()?;
        (0..list.length())
            .filter_map(|i| list.item(i)?.dyn_into::<HtmlElement>().ok())
            .find(|b| b.offset_width() > 0)
    };
    let close_menu = move |refocus: bool| {
        menu.try_set(false);
        if refocus && let Some(b) = opener() {
            let _ = b.focus();
        }
    };
    let toggle_menu = move || {
        let open = !menu.get_untracked();
        menu.set(open);
        if open {
            request_animation_frame(move || {
                let picked = root
                    .try_get_untracked()
                    .flatten()
                    .and_then(|r| {
                        r.query_selector(".player-speeds [aria-pressed=\"true\"]")
                            .ok()
                            .flatten()
                    })
                    .and_then(|b| b.dyn_into::<HtmlElement>().ok());
                if let Some(b) = picked {
                    let _ = b.focus();
                }
            });
        }
    };
    let step_menu = move |by: i32| {
        let Some(list) = root
            .try_get_untracked()
            .flatten()
            .and_then(|r| r.query_selector_all(".player-panel button").ok())
        else {
            return;
        };
        let items: Vec<HtmlElement> = (0..list.length())
            .filter_map(|i| list.item(i)?.dyn_into::<HtmlElement>().ok())
            .filter(|b| b.offset_width() > 0)
            .collect();
        let Ok(n) = i32::try_from(items.len()) else {
            return;
        };
        if n == 0 {
            return;
        }
        let active = document().and_then(|d| d.active_element());
        let at = items.iter().position(|b| {
            active
                .as_ref()
                .is_some_and(|a| a == AsRef::<Element>::as_ref(b))
        });
        let next = match at.and_then(|i| i32::try_from(i).ok()) {
            Some(i) => (i + by).rem_euclid(n),
            None => 0,
        };
        if let Some(b) = usize::try_from(next).ok().and_then(|i| items.get(i)) {
            let _ = b.focus();
        }
    };

    // The page's keys, before the viewer's on the window hear them: what
    // the player takes, the viewer doesn't.
    let on_key = move |e: web_sys::Event| {
        let Some(k) = e.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        if k.default_prevented()
            || k.ctrl_key()
            || k.meta_key()
            || k.alt_key()
            || keys.try_get_untracked() != Some(true)
        {
            return;
        }
        let key = k.key();
        if menu.try_get_untracked() == Some(true) {
            let handled = match key.as_str() {
                "Escape" => {
                    close_menu(true);
                    true
                }
                "ArrowLeft" | "ArrowUp" => {
                    step_menu(-1);
                    true
                }
                "ArrowRight" | "ArrowDown" => {
                    step_menu(1);
                    true
                }
                _ => false,
            };
            if handled {
                e.prevent_default();
                return;
            }
        }
        let target = e.target();
        if in_text(target.as_ref()) {
            return;
        }
        // The volume slider keeps its own arrows.
        let steering = matches!(
            key.as_str(),
            "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown" | "Home" | "End"
        );
        if steering && within(target.clone(), ".player-volume") {
            return;
        }
        let press = Press {
            key: &key,
            shift: k.shift_key(),
            on_seek: within(target.clone(), ".player-seek"),
            on_button: within(target, "button, a[href], summary"),
        };
        let Some(action) = act(press) else {
            return;
        };
        e.prevent_default();
        match action {
            Act::TogglePlay => toggle_play(true),
            Act::Seek(by) => seek_by(by),
            Act::Jump(share) => jump(share),
            Act::ToggleMute => toggle_mute(),
            Act::Fullscreen => on_full.run(()),
        }
        wake();
    };

    // Fingers: a tap is a finger down on the picture that lifts soon, where
    // it went down. It lifts wherever the viewer's stage has taken it, so
    // the lift is heard on the document.
    let tapped = move |t: Tap| {
        let second = double_tap(last_tap.try_get_value().flatten(), t);
        last_tap.try_set_value((!second).then_some(t));
        if second {
            on_full.run(());
            wake();
            return;
        }
        let paused_now = paused.try_get_untracked().unwrap_or(true);
        let shown = idle.try_get_untracked() == Some(false);
        match tap(paused_now, shown) {
            TapDoes::Play => play(),
            TapDoes::ShowControls => wake(),
            TapDoes::HideControls => {
                clear_rest();
                idle.try_set(true);
            }
        }
    };
    let lifted = move |e: web_sys::Event| {
        let Some(p) = e.dyn_ref::<web_sys::PointerEvent>() else {
            return;
        };
        let Some(d) = down
            .try_get_value()
            .flatten()
            .filter(|d| d.id == p.pointer_id())
        else {
            return;
        };
        down.try_set_value(None);
        if e.type_() != "pointerup" {
            return;
        }
        let (x, y) = (f64::from(p.client_x()), f64::from(p.client_y()));
        let quick = p.time_stamp() - d.time <= TAP_MS;
        if quick && (x - d.x).hypot(y - d.y) <= TAP_SLOP {
            tapped(Tap {
                time: p.time_stamp(),
                x,
                y,
            });
        }
    };
    let picture_down = move |e: PointerEvent| {
        let by_mouse = e.pointer_type() == "mouse";
        mouse.set_value(by_mouse);
        if by_mouse {
            return;
        }
        let here = Down {
            id: e.pointer_id(),
            time: e.time_stamp(),
            x: f64::from(e.client_x()),
            y: f64::from(e.client_y()),
        };
        // A second finger makes a pinch, not a tap.
        let first = down.get_value().is_none();
        down.set_value(first.then_some(here));
    };
    let outside = move |e: web_sys::Event| {
        if menu.try_get_untracked() == Some(true)
            && !within(e.target(), ".player-panel, .player-opener")
        {
            menu.try_set(false);
        }
    };

    if let Some(doc) = document() {
        let all = vec![
            dom::listen_capture(&doc, "keydown", on_key),
            dom::listen(&doc, "pointerup", true, lifted),
            dom::listen(&doc, "pointercancel", true, lifted),
            dom::listen(&doc, "pointerdown", true, outside),
        ];
        watches.update_value(|w| w.extend(all));
    }
    video.on_load(move |v| {
        v.set_volume(volume.get_untracked());
        v.set_muted(muted.get_untracked());
        let r = rate.get_untracked();
        v.set_default_playback_rate(r);
        v.set_playback_rate(r);
        let all = [
            ("enterpictureinpicture", true),
            ("leavepictureinpicture", false),
        ]
        .map(|(event, on)| {
            dom::listen(&v, event, true, move |_| {
                pip.try_set(on);
            })
        });
        watches.update_value(|w| w.extend(all));
    });
    on_cleanup({
        let watches = SendWrapper::new(watches);
        let timers = SendWrapper::new((rest_timer, badge_timer));
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
        }
    });

    // New addresses for the file: whatever failed is tried again.
    Effect::watch(
        move || src.get(),
        move |_, _, _| {
            failed.set(None);
            waiting.set(false);
        },
        false,
    );
    // Going in or out of fullscreen shows where the controls went.
    Effect::watch(move || full.get(), move |_, _, _| wake(), false);

    let reload = Callback::new(move |()| {
        if let Some(v) = el() {
            v.load();
        }
    });
    let retry = Callback::new(move |()| {
        failed.set(None);
        match on_retry {
            Some(again) => again.run(reload),
            None => reload.run(()),
        }
    });

    // What the media element says, into the controls.
    let took_length = move || {
        if let Some(v) = el() {
            let d = v.duration();
            duration.set(d);
            if !loop_chosen.get_value() {
                looping.set(loops(d));
            }
        }
    };
    let on_play = move || {
        paused.set(false);
        started.set(true);
        ended.set(false);
        wake();
        if !ticking.get_value() && !power::active_untracked() {
            ticking.set_value(true);
            tick(video, now, paused, scrub, ticking);
        }
    };
    let on_pause = move || {
        paused.set(true);
        clear_rest();
        idle.set(false);
    };

    // The seek bar: a press goes there at once, and a drag shows the frames
    // on the way, the video paused until it lets go.
    let share_of = move |e: &PointerEvent| -> f64 {
        seek.get_untracked().map_or(0.0, |s| {
            let r = s.get_bounding_client_rect();
            share_at(f64::from(e.client_x()), r.left(), r.width())
        })
    };
    let preview = move |share: f64| {
        if let Some(t) = time_at(share, duration.get_untracked()) {
            seek_to(t);
            badge.set(None);
        }
    };
    let seek_down = move |e: PointerEvent| {
        let by_mouse = e.pointer_type() == "mouse";
        mouse.set_value(by_mouse);
        if !e.is_primary() || (by_mouse && e.button() != 0) {
            return;
        }
        // No focus from a press, so the arrows still step between posts.
        e.prevent_default();
        if let Some(s) = seek.get_untracked() {
            let _ = s.set_pointer_capture(e.pointer_id());
        }
        scrub.set(Some(e.pointer_id()));
        let playing = !paused.get_untracked();
        resume.set_value(playing);
        if playing && let Some(v) = el() {
            let _ = v.pause();
        }
        let share = share_of(&e);
        hover.set(Some(share));
        preview(share);
        wake();
    };
    let seek_move = move |e: PointerEvent| {
        let dragging = scrub.get_untracked() == Some(e.pointer_id());
        if dragging || e.pointer_type() == "mouse" {
            let share = share_of(&e);
            hover.set(Some(share));
            if dragging {
                preview(share);
            }
        }
    };
    let seek_up = move |e: PointerEvent| {
        if scrub.get_untracked() != Some(e.pointer_id()) {
            return;
        }
        scrub.set(None);
        if e.pointer_type() != "mouse" {
            hover.set(None);
        }
        if resume.get_value() {
            play();
        }
        wake();
    };

    let shown_times = Memo::new(move |_| times(now.get(), duration.get()));
    let level = move || if muted.get() { 0.0 } else { volume.get() };
    let mark = move || i18n::lang().decimal_mark();
    let speed_now = move || t!("player-speed-value", speed = speed_text(rate.get(), mark()));
    let play_icon = move || {
        let icon = if ended.get() {
            Icon::Replay
        } else if paused.get() {
            Icon::Play
        } else {
            Icon::Pause
        };
        view! { <Ico icon=icon /> }
    };
    let play_label = move || {
        if ended.get() {
            t!("player-replay")
        } else if paused.get() {
            t!("player-play")
        } else {
            t!("player-pause")
        }
    };
    let play_title = move || {
        if paused.get() {
            t!("player-play-title")
        } else {
            t!("player-pause-title")
        }
    };
    let loop_label = move || t!("player-loop");
    let pip_label = move || {
        if pip.get() {
            t!("player-pip-exit")
        } else {
            t!("player-pip")
        }
    };

    let speeds = move || {
        SPEEDS
            .into_iter()
            .map(|s| {
                view! {
                    <button
                        type="button"
                        class="player-chip"
                        aria-pressed=move || ((rate.get() - s).abs() < 0.001).to_string()
                        on:click=move |_| {
                            set_rate(s);
                            close_menu(true);
                        }
                    >
                        {move || t!("player-speed-value", speed = speed_text(s, mark()))}
                    </button>
                }
            })
            .collect_view()
    };

    view! {
        <div
            node_ref=root
            class=format!("player {class}")
            class:paused=move || paused.get()
            class:started=move || started.get()
            class:ended=move || ended.get()
            class:idle=move || idle.get()
            class:scrubbing=move || scrub.with(Option::is_some)
            class:failed=move || failed.with(Option::is_some)
            style=move || style.get()
            role="group"
            aria-label=move || t!("player")
            on:pointermove=move |e: PointerEvent| {
                if e.pointer_type() != "mouse" {
                    return;
                }
                mouse.set_value(true);
                // Not a new timer for every move.
                let t = e.time_stamp();
                if idle.get_untracked() || t - last_wake.get_value() > 200.0 {
                    last_wake.set_value(t);
                    wake();
                }
            }
            on:pointerleave=move |e: PointerEvent| {
                if e.pointer_type() == "mouse" && !paused.get_untracked() {
                    rest_after(LEAVE_MS);
                }
            }
            on:pointerdown=move |e: PointerEvent| pressed_at.set_value(e.time_stamp())
            on:focusin=move |e: FocusEvent| {
                key_focus.set_value(e.time_stamp() - pressed_at.get_value() > 500.0);
                wake();
            }
            on:keydown=move |_| wake()
        >
            <video
                node_ref=video
                src=move || src.get()
                poster=move || poster.get()
                playsinline=true
                preload="metadata"
                autoplay=autoplay
                loop=move || looping.get()
                on:pointerdown=picture_down
                on:click=move |_: MouseEvent| {
                    if mouse.get_value() {
                        toggle_play(true);
                        wake();
                    }
                }
                on:dblclick=move |_: MouseEvent| {
                    // Fingers have their own double tap.
                    if mouse.get_value() {
                        on_full.run(());
                    }
                }
                on:play=move |_| on_play()
                on:pause=move |_| on_pause()
                on:ended=move |_| ended.set(true)
                on:waiting=move |_| waiting.set(true)
                on:seeking=move |_| {
                    waiting.set(true);
                    ended.set(false);
                }
                on:playing=move |_| waiting.set(false)
                on:canplay=move |_| waiting.set(false)
                on:seeked=move |_| {
                    waiting.set(false);
                    if scrub.get_untracked().is_none()
                        && let Some(v) = el()
                    {
                        now.set(v.current_time());
                    }
                }
                on:emptied=move |_| {
                    waiting.set(false);
                    buffered.set(Vec::new());
                }
                on:timeupdate=move |_| {
                    if scrub.get_untracked().is_none()
                        && let Some(v) = el()
                    {
                        now.set(v.current_time());
                    }
                }
                on:loadedmetadata=move |_| took_length()
                on:durationchange=move |_| took_length()
                on:progress=move |_| {
                    if let Some(v) = el() {
                        buffered.set(ranges(&v));
                    }
                }
                on:volumechange=move |_| {
                    if let Some(v) = el() {
                        muted.set(v.muted());
                        volume.set(v.volume());
                    }
                }
                on:ratechange=move |_| {
                    if let Some(v) = el() {
                        rate.set(v.playback_rate());
                    }
                }
                on:error=move |_| {
                    let code = el().and_then(|v| v.error()).map_or(0, |e| e.code());
                    waiting.set(false);
                    failed.set(Some(failure(code)));
                }
            ></video>
            // The same as the bar's play button, big over the still, and
            // for a finger whenever the controls are up.
            <button
                type="button"
                class="player-big glass float"
                tabindex="-1"
                aria-hidden="true"
                on:click=move |_| toggle_play(false)
            >
                {move || {
                    let icon = if ended.get() {
                        Icon::Replay
                    } else if paused.get() {
                        Icon::Play
                    } else {
                        Icon::Pause
                    };
                    view! { <Ico icon=icon size=32 /> }
                }}
            </button>
            {move || {
                flash
                    .get()
                    .map(|(playing, _)| {
                        let icon = if playing { Icon::Play } else { Icon::Pause };
                        view! {
                            <span class="player-flash" aria-hidden="true">
                                <Ico icon=icon size=32 />
                            </span>
                        }
                    })
            }}
            {move || {
                badge
                    .get()
                    .map(|b| {
                        let back = b.seconds < 0;
                        let seconds = b.seconds.saturating_abs();
                        let text = if back {
                            t!("player-back", seconds = seconds)
                        } else {
                            t!("player-forward", seconds = seconds)
                        };
                        view! {
                            <span class="player-badge" class:back=back aria-hidden="true">
                                {text}
                            </span>
                        }
                    })
            }}
            <Show when=move || waiting.get() && failed.with(Option::is_none)>
                <div class="player-wait">
                    <Spinner />
                </div>
            </Show>
            {move || {
                failed
                    .get()
                    .map(|f| {
                        view! {
                            <div class="player-error">
                                <ErrorBanner message=Signal::derive(move || failure_text(f)) on_retry=retry />
                            </div>
                        }
                    })
            }}
            <div class="player-bar glass float" node_ref=bar>
                <button
                    type="button"
                    class="player-btn"
                    title=play_title
                    aria-label=play_label
                    on:click=move |_| toggle_play(false)
                >
                    {play_icon}
                </button>
                <span class="player-time" aria-hidden="true">
                    {move || shown_times.get().0}
                </span>
                <div
                    node_ref=seek
                    class="player-seek"
                    role="slider"
                    tabindex="0"
                    aria-label=move || t!("player-position")
                    aria-valuemin="0"
                    aria-valuemax=move || {
                        let d = duration.get();
                        if known(d) { format!("{:.0}", d.round()) } else { "0".to_owned() }
                    }
                    aria-valuenow=move || format!("{:.0}", now.get().max(0.0).floor())
                    aria-valuetext=move || {
                        let (at, total) = shown_times.get();
                        t!("player-time-of", now = at, total = total)
                    }
                    style=move || format!("--played: {:.4}", played(now.get(), duration.get()))
                    on:pointerdown=seek_down
                    on:pointermove=seek_move
                    on:pointerup=seek_up
                    on:pointercancel=seek_up
                    on:lostpointercapture=seek_up
                    on:pointerleave=move |_| {
                        if scrub.get_untracked().is_none() {
                            hover.set(None);
                        }
                    }
                >
                    <div class="player-track">
                        {move || {
                            loaded(&buffered.get(), duration.get())
                                .into_iter()
                                .map(|(from, width)| {
                                    let style = format!(
                                        "left: {:.3}%; width: {:.3}%",
                                        from * 100.0,
                                        width * 100.0,
                                    );
                                    view! { <div class="player-loaded" style=style></div> }
                                })
                                .collect_view()
                        }}
                        <div class="player-played"></div>
                    </div>
                    <div class="player-knob"></div>
                    {move || {
                        hover
                            .get()
                            .and_then(|share| {
                                let text = time_at(share, duration.get()).and_then(clock)?;
                                Some(
                                    view! {
                                        <span class="player-tip" style=format!("--at: {share:.4}")>
                                            {text}
                                        </span>
                                    },
                                )
                            })
                    }}
                </div>
                <span class="player-time" aria-hidden="true">
                    {move || shown_times.get().1}
                </span>
                <div class="player-sound">
                    <button
                        type="button"
                        class="player-btn"
                        title=move || {
                            if muted.get() { t!("player-unmute-title") } else { t!("player-mute-title") }
                        }
                        aria-label=move || {
                            if muted.get() { t!("player-unmute") } else { t!("player-mute") }
                        }
                        on:click=move |_| toggle_mute()
                    >
                        {move || {
                            let l = level();
                            let icon = if l < 0.01 {
                                Icon::VolumeMute
                            } else if l < 0.5 {
                                Icon::VolumeLow
                            } else {
                                Icon::Volume
                            };
                            view! { <Ico icon=icon /> }
                        }}
                    </button>
                    <input
                        type="range"
                        class="player-volume"
                        min="0"
                        max="1"
                        step="0.05"
                        aria-label=move || t!("player-volume")
                        aria-valuetext=move || t!("player-volume-level", percent = percent(level()))
                        prop:value=move || level().to_string()
                        style=move || format!("--level: {:.3}", level())
                        on:input=move |e| {
                            if let Ok(v) = event_target_value(&e).parse::<f64>() {
                                set_level(v);
                            }
                        }
                        on:change=move |_| remember()
                    />
                </div>
                <button
                    type="button"
                    class="player-btn player-opener player-rate"
                    title=move || t!("player-speed")
                    aria-label=move || t!("player-speed-now", speed = speed_now())
                    aria-haspopup="true"
                    aria-expanded=move || menu.get().to_string()
                    on:click=move |_| toggle_menu()
                >
                    {speed_now}
                </button>
                <button
                    type="button"
                    class="player-btn player-opener player-more"
                    title=move || t!("player-settings")
                    aria-label=move || t!("player-settings")
                    aria-haspopup="true"
                    aria-expanded=move || menu.get().to_string()
                    on:click=move |_| toggle_menu()
                >
                    <Ico icon=Icon::Tune />
                </button>
                <button
                    type="button"
                    class="player-btn player-loop"
                    class:on=move || looping.get()
                    title=loop_label
                    aria-label=loop_label
                    aria-pressed=move || looping.get().to_string()
                    on:click=move |_| toggle_loop()
                >
                    <Ico icon=Icon::Repeat />
                </button>
                {pip_offered
                    .then(|| {
                        view! {
                            <button
                                type="button"
                                class="player-btn player-pip"
                                class:on=move || pip.get()
                                title=pip_label
                                aria-label=pip_label
                                aria-pressed=move || pip.get().to_string()
                                on:click=move |_| toggle_pip()
                            >
                                <Ico icon=Icon::PictureInPicture />
                            </button>
                        }
                    })}
                <button
                    type="button"
                    class="player-btn"
                    title=move || if full.get() { t!("fullscreen-exit-title") } else { t!("fullscreen-title") }
                    aria-label=move || if full.get() { t!("fullscreen-exit") } else { t!("fullscreen") }
                    on:click=move |_| on_full.run(())
                >
                    {move || {
                        let icon = if full.get() { Icon::FullscreenExit } else { Icon::Fullscreen };
                        view! { <Ico icon=icon /> }
                    }}
                </button>
                <Show when=move || menu.get()>
                    <div
                        class="player-panel float"
                        role="group"
                        aria-label=move || t!("player-settings")
                        on:focusout=move |e: FocusEvent| {
                            if !within(e.related_target(), ".player-panel, .player-opener") {
                                menu.set(false);
                            }
                        }
                    >
                        <div class="player-speeds" role="group" aria-label=move || t!("player-speed")>
                            {speeds()}
                        </div>
                        <div class="player-extras">
                            <button
                                type="button"
                                class="player-chip"
                                aria-pressed=move || looping.get().to_string()
                                on:click=move |_| toggle_loop()
                            >
                                <Ico icon=Icon::Repeat size=20 />
                                {loop_label}
                            </button>
                            {pip_offered
                                .then(|| {
                                    view! {
                                        <button
                                            type="button"
                                            class="player-chip"
                                            aria-pressed=move || pip.get().to_string()
                                            on:click=move |_| toggle_pip()
                                        >
                                            <Ico icon=Icon::PictureInPicture size=20 />
                                            {pip_label}
                                        </button>
                                    }
                                })}
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}
