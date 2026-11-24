//! The crop picker for a profile picture: the picture under a round
//! window, dragged with a pointer or a finger, zoomed with the slider, the
//! wheel or two fingers, and moved with the arrow keys. The square it
//! shows goes to the server in the picture's own pixels
//! (`logic::profile::Crop`).

use leptos::ev::{KeyboardEvent, PointerEvent, WheelEvent};
use leptos::html;
use leptos::prelude::*;

use crate::components::icons::{Ico, Icon};
use crate::logic::profile::Crop;

/// A pointer on the frame: its id and where it is.
type Touch = (i32, f64, f64);

/// The slider's steps.
const SLIDER_STEPS: f64 = 1000.0;

/// The picture at `src` (a picked file's address) under the window. `crop`
/// holds the square once the picture has loaded.
#[component]
pub fn CropPicker(src: String, crop: RwSignal<Option<Crop>>) -> impl IntoView {
    let frame: NodeRef<html::Div> = NodeRef::new();
    let picture: NodeRef<html::Img> = NodeRef::new();
    // The pointers down on the frame, and for two fingers, how far apart
    // they started and the zoom then.
    let touches = StoredValue::new(Vec::<Touch>::new());
    let pinch = StoredValue::new(None::<(f64, f64)>);
    let dragging = RwSignal::new(false);

    let view_size = move || {
        frame
            .get_untracked()
            .map_or(0.0, |f| f.get_bounding_client_rect().width())
    };
    // A point of the window as shares of the frame, 0 to 1 across.
    let share_at = move |x: f64, y: f64| {
        frame.get_untracked().map_or((0.5, 0.5), |f| {
            let r = f.get_bounding_client_rect();
            let w = r.width().max(1.0);
            let h = r.height().max(1.0);
            ((x - r.left()) / w, (y - r.top()) / h)
        })
    };
    let loaded = move |_| {
        if let Some(img) = picture.get_untracked() {
            let (w, h) = (
                f64::from(img.natural_width()),
                f64::from(img.natural_height()),
            );
            crop.set(Crop::new(w, h));
        }
    };
    let spread = |list: &[Touch]| -> Option<(f64, f64, f64)> {
        let (a, b) = (list.first()?, list.get(1)?);
        let distance = (a.1 - b.1).hypot(a.2 - b.2);
        Some((distance, (a.1 + b.1) / 2.0, (a.2 + b.2) / 2.0))
    };
    let down = move |e: PointerEvent| {
        if crop.with_untracked(Option::is_none) {
            return;
        }
        e.prevent_default();
        if let Some(f) = frame.get_untracked() {
            let _ = f.set_pointer_capture(e.pointer_id());
            let _ = f.focus();
        }
        touches.update_value(|t| {
            t.retain(|p| p.0 != e.pointer_id());
            t.push((
                e.pointer_id(),
                f64::from(e.client_x()),
                f64::from(e.client_y()),
            ));
        });
        let zoom = crop.with_untracked(|c| c.map_or(1.0, |c| c.zoom));
        pinch.set_value(touches.with_value(|t| spread(t)).map(|(d, ..)| (d, zoom)));
        dragging.set(true);
    };
    let moved = move |e: PointerEvent| {
        let id = e.pointer_id();
        let (x, y) = (f64::from(e.client_x()), f64::from(e.client_y()));
        let Some(before) = touches.with_value(|t| t.iter().find(|p| p.0 == id).copied()) else {
            return;
        };
        let pair_before = touches.with_value(|t| spread(t));
        touches.update_value(|t| {
            if let Some(p) = t.iter_mut().find(|p| p.0 == id) {
                *p = (id, x, y);
            }
        });
        let pair_after = touches.with_value(|t| spread(t));
        let view = view_size();
        match (pair_before, pair_after, pinch.get_value()) {
            // Two fingers: closer or further apart, and moved together.
            (Some((_, bx, by)), Some((distance, ax, ay)), Some((start, zoom))) if start > 0.0 => {
                let (fx, fy) = share_at(ax, ay);
                crop.update(|c| {
                    if let Some(c) = c {
                        *c = c.zoomed_at(zoom * distance / start, fx, fy).dragged(
                            ax - bx,
                            ay - by,
                            view,
                        );
                    }
                });
            }
            _ => crop.update(|c| {
                if let Some(c) = c {
                    *c = c.dragged(x - before.1, y - before.2, view);
                }
            }),
        }
    };
    let up = move |e: PointerEvent| {
        touches.update_value(|t| t.retain(|p| p.0 != e.pointer_id()));
        let zoom = crop.with_untracked(|c| c.map_or(1.0, |c| c.zoom));
        pinch.set_value(touches.with_value(|t| spread(t)).map(|(d, ..)| (d, zoom)));
        if touches.with_value(Vec::is_empty) {
            dragging.set(false);
        }
    };
    let wheel = move |e: WheelEvent| {
        if crop.with_untracked(Option::is_none) {
            return;
        }
        e.prevent_default();
        let (fx, fy) = share_at(f64::from(e.client_x()), f64::from(e.client_y()));
        // A notch of a wheel is about 100; a touchpad sends small steps.
        let factor = (-e.delta_y() / 500.0).exp();
        crop.update(|c| {
            if let Some(c) = c {
                *c = c.zoomed_at(c.zoom * factor, fx, fy);
            }
        });
    };
    let key = move |e: KeyboardEvent| {
        let next = crop.with_untracked(|c| c.and_then(|c| c.keyed(&e.key(), e.shift_key())));
        if let Some(next) = next {
            e.prevent_default();
            crop.set(Some(next));
        }
    };
    let placed = move || {
        crop.with(|c| {
            c.map(|c| {
                let (left, top, width, height) = c.placement();
                format!(
                    "left: {left:.3}%; top: {top:.3}%; width: {width:.3}%; height: {height:.3}%"
                )
            })
        })
        .unwrap_or_else(|| "visibility: hidden".to_owned())
    };
    let share = Memo::new(move |_| crop.with(|c| c.map_or(0.0, |c| c.zoom_share())));
    let slide = move |e: leptos::ev::Event| {
        let value: f64 = event_target_value(&e).parse().unwrap_or(0.0);
        crop.update(|c| {
            if let Some(c) = c {
                *c = c.zoomed(c.zoom_of_share(value / SLIDER_STEPS));
            }
        });
    };
    let step = move |by: &'static str| {
        crop.update(|c| {
            if let Some(next) = c.and_then(|c| c.keyed(by, false)) {
                *c = Some(next);
            }
        });
    };
    view! {
        <div class="crop">
            <div
                class="crop-frame"
                class:dragging=move || dragging.get()
                node_ref=frame
                tabindex="0"
                role="group"
                aria-label=move || t!("profile-crop-label")
                on:pointerdown=down
                on:pointermove=moved
                on:pointerup=up
                on:pointercancel=up
                on:wheel=wheel
                on:keydown=key
            >
                <img
                    node_ref=picture
                    class="crop-picture"
                    src=src
                    alt=""
                    draggable="false"
                    style=placed
                    on:load=loaded
                />
                <span class="crop-window" aria-hidden="true"></span>
            </div>
            <div class="crop-zoom">
                <button
                    type="button"
                    class="icon-btn small"
                    title=move || t!("profile-crop-out")
                    aria-label=move || t!("profile-crop-out")
                    on:click=move |_| step("-")
                >
                    <Ico icon=Icon::ZoomOut size=20 />
                </button>
                <input
                    type="range"
                    min="0"
                    max=SLIDER_STEPS
                    step="1"
                    aria-label=move || t!("profile-crop-zoom")
                    prop:value=move || format!("{:.0}", share.get() * SLIDER_STEPS)
                    on:input=slide
                />
                <button
                    type="button"
                    class="icon-btn small"
                    title=move || t!("profile-crop-in")
                    aria-label=move || t!("profile-crop-in")
                    on:click=move |_| step("+")
                >
                    <Ico icon=Icon::ZoomIn size=20 />
                </button>
            </div>
        </div>
    }
}
