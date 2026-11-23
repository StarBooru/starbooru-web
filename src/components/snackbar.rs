//! Short notices at the bottom of the window, for things that finish after the
//! click (a download, say) or can be taken back. Any page calls [`notify`] or
//! [`notify_undo`]; the one [`Snackbar`] at the root shows them.

use std::rc::Rc;
use std::time::Duration;

use leptos::prelude::*;

use super::icons::{Ico, Icon};

/// How long a notice stays.
const NOTICE_MS: u32 = 6000;
/// How long an undo is offered.
const UNDO_MS: u32 = 8000;
/// How often the time a notice has left is counted down.
const TICK_MS: u32 = 200;

/// What the snackbar shows, numbered so a new one restarts its timer.
#[derive(Clone)]
struct Notice {
    n: u64,
    message: String,
    action: Option<Action>,
    ms: u32,
}

/// A button on a notice, such as Undo. It runs in the snackbar, which stays
/// when the page changes.
#[derive(Clone)]
struct Action {
    label: String,
    run: Rc<dyn Fn()>,
}

thread_local! {
    // Reference counted: an arena signal would belong to whichever component
    // touched it first and go when that one does.
    static NOTICE: ArcRwSignal<Option<Notice>> = ArcRwSignal::new(None);
    /// The pointer is over the notice, and the focus is in it. Either holds
    /// its time, so it doesn't go while being read or reached for.
    static HELD: ArcRwSignal<(bool, bool)> = ArcRwSignal::new((false, false));
}

/// Shows `message` for a few seconds.
pub fn notify(message: String) {
    show(message, None, NOTICE_MS);
}

/// A notice with a button that takes back what just happened. It stays a
/// little longer than others, and a line under it runs out with the time.
pub fn notify_undo(message: String, label: String, run: impl Fn() + 'static) {
    let action = Action {
        label,
        run: Rc::new(run),
    };
    show(message, Some(action), UNDO_MS);
}

fn show(message: String, action: Option<Action>, ms: u32) {
    let n = NOTICE.with(|notice| {
        let n = notice.with_untracked(|m| m.as_ref().map_or(0, |m| m.n.wrapping_add(1)));
        notice.set(Some(Notice {
            n,
            message,
            action,
            ms,
        }));
        n
    });
    // The pointer or the focus on the one before says nothing of this one.
    release();
    tick(n, ms, js_sys::Date::now());
}

/// Counts down the time notice `n` has left, a tick at a time. By the clock,
/// as a tab out of sight runs its timers late.
fn tick(n: u64, left: u32, at: f64) {
    set_timeout(
        move || {
            // Only this notice: a newer one has its own count.
            if !NOTICE
                .with(|notice| notice.with_untracked(|m| m.as_ref().is_some_and(|m| m.n == n)))
            {
                return;
            }
            let held = HELD.with(|h| h.with_untracked(|(pointer, focus)| *pointer || *focus));
            let now = js_sys::Date::now();
            match countdown(left, now - at, held) {
                0 => dismiss(),
                left => tick(n, left, now),
            }
        },
        Duration::from_millis(u64::from(TICK_MS)),
    );
}

/// The time left after `passed` milliseconds. None passes while the notice
/// is held, or when the clock was set back.
fn countdown(left: u32, passed: f64, held: bool) -> u32 {
    if held || passed.is_nan() || passed <= 0.0 {
        return left;
    }
    // Saturates: a passing larger than any notice ends it.
    let passed = passed.min(f64::from(u32::MAX)) as u32;
    left.saturating_sub(passed)
}

fn hold(pointer: Option<bool>, focus: Option<bool>) {
    HELD.with(|h| {
        let (p, f) = h.get_untracked();
        let now = (pointer.unwrap_or(p), focus.unwrap_or(f));
        if now != (p, f) {
            h.set(now);
        }
    });
}

fn release() {
    hold(Some(false), Some(false));
}

fn dismiss() {
    NOTICE.with(|notice| notice.set(None));
    release();
}

fn current() -> Option<Notice> {
    NOTICE.with(|notice| notice.get())
}

#[component]
pub fn Snackbar() -> impl IntoView {
    let held = Memo::new(|_| HELD.with(|h| h.with(|(pointer, focus)| *pointer || *focus)));
    view! {
        <div class="snackbar-slot" role="status" aria-live="polite">
            {move || {
                current()
                    .map(|Notice { message, action, ms, .. }| {
                        let timed = action.is_some();
                        let action = action
                            .map(|Action { label, run }| {
                                view! {
                                    <button
                                        type="button"
                                        class="btn text snackbar-action"
                                        on:click=move |_| {
                                            dismiss();
                                            run();
                                        }
                                    >
                                        {label}
                                    </button>
                                }
                            });
                        view! {
                            <div
                                class="snackbar"
                                class:timed=timed
                                class:held=move || held.get()
                                style=format!("--notice-ms: {ms}ms")
                                on:pointerenter=|_| hold(Some(true), None)
                                on:pointerleave=|_| hold(Some(false), None)
                                on:focusin=|_| hold(None, Some(true))
                                on:focusout=|_| hold(None, Some(false))
                            >
                                <span class="break">{message}</span>
                                {action}
                                <button
                                    type="button"
                                    class="icon-btn"
                                    aria-label=move || t!("notice-close")
                                    title=move || t!("notice-close")
                                    on:click=move |_| dismiss()
                                >
                                    <Ico icon=Icon::Close />
                                </button>
                            </div>
                        }
                    })
            }}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_notice_waits_while_it_is_held() {
        assert_eq!(countdown(6000, 200.0, false), 5800);
        assert_eq!(countdown(6000, 200.0, true), 6000);
        assert_eq!(countdown(100, 200.0, false), 0);
        // A tab out of sight, whose timer came late.
        assert_eq!(countdown(6000, 4000.4, false), 2000);
        assert_eq!(countdown(6000, 1e30, false), 0);
        // A clock set back, or nonsense.
        assert_eq!(countdown(6000, -50.0, false), 6000);
        assert_eq!(countdown(6000, f64::NAN, false), 6000);
        const { assert!(TICK_MS < NOTICE_MS && TICK_MS < UNDO_MS) };
        // The line under an undo stops with it.
        let css = include_str!("../../style/base.css");
        assert!(css.contains(".snackbar.held::after { animation-play-state: paused; }"));
    }
}
