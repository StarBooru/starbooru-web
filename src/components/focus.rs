//! The focus in a modal dialog: Tab goes round inside it while it's open,
//! and the focus can go back where it was when it closes.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use leptos::html::ElementType;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, KeyboardEvent, Node};

use crate::dom;

/// What Tab can land on.
const FOCUSABLE: &str = concat!(
    "a[href], button:not([disabled]), input:not([disabled]):not([type=\"hidden\"]), ",
    "select:not([disabled]), textarea:not([disabled]), video[controls], ",
    "[tabindex]:not([tabindex=\"-1\"])",
);

/// The places in `dialog` Tab stops at, in order, leaving out hidden ones.
fn stops(dialog: &Element) -> Vec<HtmlElement> {
    let Ok(found) = dialog.query_selector_all(FOCUSABLE) else {
        return Vec::new();
    };
    (0..found.length())
        .filter_map(|i| found.item(i)?.dyn_into::<HtmlElement>().ok())
        .filter(|el| {
            let r = el.get_bounding_client_rect();
            r.width() > 0.0 || r.height() > 0.0
        })
        .collect()
}

fn active_element() -> Option<HtmlElement> {
    web_sys::window()?
        .document()?
        .active_element()?
        .dyn_into::<HtmlElement>()
        .ok()
}

/// Keeps Tab and Shift+Tab inside `dialog` while `open`: past the last stop
/// to the first and back, and focus that strays out comes back in.
fn trap(dialog: &Element, open: Rc<Cell<bool>>) -> Vec<dom::Watch> {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return Vec::new();
    };
    // Which way the last Tab went, for focus that left anyway.
    let back = Rc::new(Cell::new(false));
    // On the dialog, not the page: a sheet keeps its keys from the page
    // behind it, and what's inside it has first say over Tab.
    let keys = {
        let (dialog, open, back, d) = (
            dialog.clone(),
            Rc::clone(&open),
            Rc::clone(&back),
            doc.clone(),
        );
        let target = dialog.clone();
        dom::listen(&target, "keydown", false, move |e| {
            let Some(k) = e.dyn_ref::<KeyboardEvent>() else {
                return;
            };
            if !open.get()
                || k.key() != "Tab"
                || k.default_prevented()
                || k.ctrl_key()
                || k.alt_key()
                || k.meta_key()
            {
                return;
            }
            back.set(k.shift_key());
            let all = stops(&dialog);
            let here = d
                .active_element()
                .filter(|a| *a != dialog && dialog.contains(Some(a)));
            let on = |el: Option<&HtmlElement>| {
                el.zip(here.as_ref())
                    .is_some_and(|(el, a)| AsRef::<Element>::as_ref(el) == a)
            };
            let to = if k.shift_key() {
                (here.is_none() || on(all.first())).then(|| all.last())
            } else {
                (here.is_none() || on(all.last())).then(|| all.first())
            };
            if let Some(to) = to {
                k.prevent_default();
                if let Some(el) = to {
                    let _ = el.focus();
                }
            }
        })
    };
    let strays = {
        let dialog = dialog.clone();
        dom::listen(&doc, "focusin", false, move |e| {
            let target = e.target().and_then(|t| t.dyn_into::<Node>().ok());
            if !open.get() || target.is_some_and(|t| dialog.contains(Some(&t))) {
                return;
            }
            // A moment later: focusing from here would run this listener
            // again inside itself, which the browser refuses.
            let (dialog, open, back) = (dialog.clone(), Rc::clone(&open), back.get());
            let later = SendWrapper::new(move || {
                let outside = active_element()
                    .is_none_or(|a| !dialog.contains(Some(AsRef::<Node>::as_ref(&a))));
                if !open.get() || !outside {
                    return;
                }
                let all = stops(&dialog);
                let to = if back { all.last() } else { all.first() };
                if let Some(el) = to {
                    let _ = el.focus();
                }
            });
            set_timeout(move || (later.take())(), Duration::ZERO);
        })
    };
    vec![keys, strays]
}

/// Keeps Tab inside the dialog `node` holds for as long as it's on the
/// page. With `restore`, the focus then goes back to what had it when the
/// dialog opened.
pub fn hold_focus<E>(node: NodeRef<E>, restore: bool)
where
    E: ElementType,
    E::Output: JsCast + Clone + 'static,
{
    let before = if restore { active_element() } else { None };
    let open = Rc::new(Cell::new(true));
    let watches = Rc::new(RefCell::new(Vec::<dom::Watch>::new()));
    {
        let (open, watches) = (Rc::clone(&open), Rc::clone(&watches));
        node.on_load(move |el| {
            if let Ok(el) = el.dyn_into::<Element>() {
                watches.borrow_mut().extend(trap(&el, open));
            }
        });
    }
    let kept = SendWrapper::new((open, watches, before));
    on_cleanup(move || {
        let (open, watches, before) = kept.take();
        // Off first, or giving the focus back would count as straying.
        open.set(false);
        if let Some(el) = before.filter(|el| el.is_connected()) {
            let _ = el.focus();
        }
        // A key can close the dialog from inside one of these listeners,
        // which can't be dropped while it runs.
        set_timeout(
            move || {
                for watch in watches.take() {
                    watch.stop();
                }
            },
            Duration::ZERO,
        );
    });
}
