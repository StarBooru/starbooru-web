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
}

/// Shows `message` for a few seconds.
#[allow(dead_code, reason = "pages call it for work that finishes later")]
pub fn notify(message: String) {
    show(message, None, NOTICE_MS);
}

/// A notice with a button that takes back what just happened. It stays a
/// little longer than others, and a line under it runs out with the time.
#[allow(dead_code, reason = "pages call it after a step that can be undone")]
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
    set_timeout(
        move || {
            // Only this notice: a newer one has its own timer.
            if NOTICE.with(|notice| notice.with_untracked(|m| m.as_ref().is_some_and(|m| m.n == n)))
            {
                dismiss();
            }
        },
        Duration::from_millis(u64::from(ms)),
    );
}

fn dismiss() {
    NOTICE.with(|notice| notice.set(None));
}

fn current() -> Option<Notice> {
    NOTICE.with(|notice| notice.get())
}

#[component]
pub fn Snackbar() -> impl IntoView {
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
                                style=format!("--notice-ms: {ms}ms")
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
