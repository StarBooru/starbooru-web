//! The people's faces on the picture. Pointing at someone in the People card
//! outlines their face with their number, and pressing their face keeps it
//! outlined, as the page scrolls up to it; while the editor is open every
//! box shows, the one being edited with corners to resize it and the rest
//! of it to move it, and "Mark the face" makes the next drag (or a tap) on
//! the picture draw theirs. Escape lets go. The boxes sit in a layer the
//! size of the picture that zooms and pans with it (`logic::faces` does the
//! sums). Still pictures only: an animation's or a video's crop comes from
//! a frame the stage may not be showing.

use std::sync::Arc;

use leptos::ev::PointerEvent;
use leptos::html;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use starbooru_core::FaceBox;
use wasm_bindgen::JsCast;
use web_sys::{Element, KeyboardEvent};

use crate::dom;
use crate::logic::faces::{
    Corner, Frame, TAP_PIXELS, at, drawn, moved, place_style, resized, step, tapped,
};
use crate::logic::viewer::fit_style;
use crate::logic::zoom::Zoom;

/// One of the editor's people, with the box it holds for them.
#[derive(Clone)]
pub(super) struct Mark {
    pub key: u64,
    pub face: ArcRwSignal<Option<FaceBox>>,
}

/// What the People card, its editor and the stage share about the faces.
/// Outside the editor a person's key is their place in the list.
#[derive(Clone, Copy)]
pub(super) struct Faces {
    /// The person pointed at or focused: their box is outlined.
    pub lit: RwSignal<Option<u64>>,
    /// The person whose face was pressed in the card: outlined until it's
    /// pressed again or another is.
    pub pinned: RwSignal<Option<u64>>,
    /// The editor's people, while it's open.
    pub marks: RwSignal<Vec<Mark>>,
    /// The person whose face the next drag on the picture marks.
    pub drawing: RwSignal<Option<u64>>,
}

impl Faces {
    pub fn new() -> Self {
        Self {
            lit: RwSignal::new(None),
            pinned: RwSignal::new(None),
            marks: RwSignal::new(Vec::new()),
            drawing: RwSignal::new(None),
        }
    }

    /// Lets go of everything, for another post or a closed editor.
    pub fn reset(self) {
        self.lit.try_set(None);
        self.pinned.try_set(None);
        self.drawing.try_set(None);
    }

    /// The box signal of the editor's person `key`.
    fn face_of(self, key: u64) -> Option<ArcRwSignal<Option<FaceBox>>> {
        self.marks
            .try_with_untracked(|m| m.iter().find(|m| m.key == key).map(|m| m.face.clone()))
            .flatten()
    }
}

/// Brings the stage into sight when it's above or below the screen, as on a
/// phone, where the editor is under it.
pub(super) fn reveal_stage() {
    let Some(stage) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.query_selector(".stage").ok().flatten())
    else {
        return;
    };
    let r = stage.get_bounding_client_rect();
    let (top, bottom) = (dom::bar_bottom(), dom::dock_top());
    if r.top() >= top - 1.0 && r.bottom() <= bottom + 1.0 {
        return;
    }
    let (y, _) = dom::scroll();
    dom::scroll_to(y + r.top() - top - 8.0, true);
}

/// What a press on the layer is doing.
#[derive(Clone, Copy, Debug)]
enum Drag {
    Draw {
        pointer: i32,
        key: u64,
        from: (f64, f64),
        at_px: (f64, f64),
    },
    Move {
        pointer: i32,
        key: u64,
        start: FaceBox,
        frame: Frame,
        at_px: (f64, f64),
    },
    Resize {
        pointer: i32,
        key: u64,
        start: FaceBox,
        corner: Corner,
    },
}

impl Drag {
    fn pointer(self) -> i32 {
        match self {
            Drag::Draw { pointer, .. }
            | Drag::Move { pointer, .. }
            | Drag::Resize { pointer, .. } => pointer,
        }
    }
}

fn frame_of(el: &Element) -> Frame {
    let r = el.get_bounding_client_rect();
    Frame {
        left: r.left(),
        top: r.top(),
        width: r.width(),
        height: r.height(),
    }
}

/// The nearest element at or above `target` that `selector` matches.
fn closest(target: Option<web_sys::EventTarget>, selector: &str) -> Option<Element> {
    target
        .and_then(|t| t.dyn_into::<Element>().ok())
        .and_then(|el| el.closest(selector).ok().flatten())
}

fn corner_of(name: &str) -> Option<Corner> {
    Corner::ALL.into_iter().find(|c| c.as_str() == name)
}

/// The layer over the picture. `saved` are the post's own boxes, for the
/// card's pointing; `editing` says the editor is open; `size` is the
/// picture's, which the layer takes as the picture's box does.
#[component]
pub(super) fn FaceLayer(
    faces: Faces,
    #[prop(into)] saved: Signal<Arc<Vec<Option<FaceBox>>>>,
    #[prop(into)] editing: Signal<bool>,
    zoom: RwSignal<Zoom>,
    #[prop(into)] size: Signal<Option<(i32, i32)>>,
) -> impl IntoView {
    let layer: NodeRef<html::Div> = NodeRef::new();
    let drag = StoredValue::new(None::<Drag>);
    // The box being drawn, before it's let go.
    let band = RwSignal::new(None::<FaceBox>);
    let drawing = Memo::new(move |_| faces.drawing.get().is_some());

    // Puts back what a press changed, or stops drawing.
    let let_go = move || {
        match drag.try_update_value(Option::take).flatten() {
            Some(Drag::Move { key, start, .. } | Drag::Resize { key, start, .. }) => {
                if let Some(face) = faces.face_of(key) {
                    face.try_set(Some(start));
                }
            }
            Some(Drag::Draw { .. }) | None => {
                faces.drawing.try_set(None);
            }
        }
        band.try_set(None);
    };

    let down = move |e: PointerEvent| {
        if e.pointer_type() == "mouse" && e.button() != 0 {
            return;
        }
        let Some(el) = layer.get_untracked() else {
            return;
        };
        let frame = frame_of(&el);
        let (x, y) = (f64::from(e.client_x()), f64::from(e.client_y()));
        let pointer = e.pointer_id();
        let started = if let Some(key) = faces.drawing.get_untracked() {
            at(frame, x, y).map(|from| Drag::Draw {
                pointer,
                key,
                from,
                at_px: (x, y),
            })
        } else if editing.get_untracked() {
            let target = e.target();
            let found = closest(target.clone(), ".face-box[data-key]").and_then(|b| {
                let key = b.get_attribute("data-key")?.parse::<u64>().ok()?;
                let start = faces.face_of(key)?.get_untracked()?;
                Some((key, start))
            });
            found.map(|(key, start)| {
                faces.lit.set(Some(key));
                let corner = closest(target, ".face-handle")
                    .and_then(|h| h.get_attribute("data-corner"))
                    .and_then(|c| corner_of(&c));
                match corner {
                    Some(corner) => Drag::Resize {
                        pointer,
                        key,
                        start,
                        corner,
                    },
                    None => Drag::Move {
                        pointer,
                        key,
                        start,
                        frame,
                        at_px: (x, y),
                    },
                }
            })
        } else {
            None
        };
        let Some(started) = started else {
            return;
        };
        // The stage neither swipes nor pans for this press.
        e.prevent_default();
        e.stop_propagation();
        let _ = el.set_pointer_capture(pointer);
        drag.set_value(Some(started));
    };

    let moved_to = move |e: PointerEvent| {
        let Some(now) = drag.get_value().filter(|d| d.pointer() == e.pointer_id()) else {
            return;
        };
        let Some(el) = layer.get_untracked() else {
            return;
        };
        let (x, y) = (f64::from(e.client_x()), f64::from(e.client_y()));
        let shape = size.get_untracked();
        match now {
            Drag::Draw { from, .. } => {
                if let Some(to) = at(frame_of(&el), x, y) {
                    band.set(Some(drawn(from, to, shape)));
                }
            }
            Drag::Move {
                key,
                start,
                frame,
                at_px,
                ..
            } => {
                let (dx, dy) = step(frame, x - at_px.0, y - at_px.1);
                if let Some(face) = faces.face_of(key) {
                    face.set(Some(moved(start, dx, dy)));
                }
            }
            Drag::Resize {
                key, start, corner, ..
            } => {
                if let (Some(face), Some(to)) = (faces.face_of(key), at(frame_of(&el), x, y)) {
                    face.set(Some(resized(start, corner, to, shape)));
                }
            }
        }
    };

    let up = move |e: PointerEvent| {
        let Some(done) = drag.get_value().filter(|d| d.pointer() == e.pointer_id()) else {
            return;
        };
        drag.set_value(None);
        band.set(None);
        let Drag::Draw {
            key, from, at_px, ..
        } = done
        else {
            return;
        };
        let (x, y) = (f64::from(e.client_x()), f64::from(e.client_y()));
        let shape = size.get_untracked();
        let tap = (x - at_px.0).hypot(y - at_px.1) < TAP_PIXELS;
        let made = match layer.get_untracked().and_then(|el| at(frame_of(&el), x, y)) {
            Some(to) if !tap => drawn(from, to, shape),
            _ => tapped(from, shape),
        };
        if let Some(face) = faces.face_of(key) {
            face.set(Some(made));
        }
        faces.drawing.set(None);
        faces.lit.set(Some(key));
    };

    // A press the browser took away puts things back as they were.
    let cancel = move |e: PointerEvent| {
        if drag.with_value(|d| d.is_some_and(|d| d.pointer() == e.pointer_id())) {
            let_go();
        }
    };

    // Escape lets go of a press, or stops drawing, before anything else
    // hears it.
    let watch = StoredValue::new(None::<dom::Watch>);
    layer.on_load(move |_| {
        let Some(window) = web_sys::window() else {
            return;
        };
        let w = dom::listen_capture(&window, "keydown", move |e| {
            let Some(e) = e.dyn_ref::<KeyboardEvent>() else {
                return;
            };
            let busy = drag.try_with_value(Option::is_some) == Some(true)
                || faces.drawing.try_get_untracked().flatten().is_some();
            if e.key() == "Escape" && busy {
                e.prevent_default();
                e.stop_propagation();
                let_go();
            }
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

    let style = move || {
        let mut style = zoom.with(Zoom::css);
        style.push_str(&fit_style(size.get()));
        style
    };

    // The editor's boxes, each with its number; the one lit has corners.
    let edited = move || {
        view! {
            <For
                each=move || faces.marks.get()
                key=|mark| mark.key
                children=move |mark: Mark| {
                    let key = mark.key;
                    let face = mark.face.clone();
                    let number = move || {
                        faces.marks.with(|m| m.iter().position(|m| m.key == key).map_or(0, |i| i + 1))
                    };
                    let lit = move || faces.lit.get() == Some(key);
                    move || {
                        let shown = face.get()?;
                        Some(view! {
                            <div
                                class="face-box edit face-edit"
                                class:lit=lit
                                data-key=key.to_string()
                                style=place_style(shown)
                            >
                                <span class="face-number">{number}</span>
                                <Show when=lit>
                                    {Corner::ALL
                                        .into_iter()
                                        .map(|c| {
                                            view! { <span class="face-handle" data-corner=c.as_str()></span> }
                                        })
                                        .collect_view()}
                                </Show>
                            </div>
                        })
                    }
                }
            />
        }
    };
    // Outside the editor, the box of the person pointed at in the card.
    let pointed = move || {
        let i = usize::try_from(faces.lit.get().or(faces.pinned.get())?).ok()?;
        let shown = saved.with(|s| s.get(i).copied().flatten())?;
        Some(view! {
            <div class="face-box" style=place_style(shown)>
                <span class="face-number">{i + 1}</span>
            </div>
        })
    };

    view! {
        <div
            node_ref=layer
            class="face-layer"
            class:drawing=move || drawing.get()
            class:face-edit=move || drawing.get()
            style=style
            on:pointerdown=down
            on:pointermove=moved_to
            on:pointerup=up
            on:pointercancel=cancel
            on:lostpointercapture=cancel
        >
            {move || if editing.get() { edited().into_any() } else { pointed().into_any() }}
            {move || {
                band.get()
                    .map(|b| {
                        view! { <div class="face-box band" style=place_style(b)></div> }
                    })
            }}
            <Show when=move || drawing.get()>
                <span class="face-hint" aria-live="polite">
                    {move || t!("people-draw-hint")}
                </span>
            </Show>
        </div>
    }
}
