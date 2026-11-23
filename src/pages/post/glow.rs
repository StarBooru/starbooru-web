//! The light the viewer stands in: the picture's thumbnail smudged once onto
//! a tiny canvas, which costs one drawing where a CSS blur would cost every
//! frame of a fade. Two layers take turns, so only opacity moves. One fade
//! runs at a time and then goes on to the latest picture, so flipping through
//! costs no more than a slow drift of colour. Reduced motion leaves it out.

use std::cell::{Cell, RefCell};

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{CanvasRenderingContext2d, Document, Element, HtmlCanvasElement, HtmlImageElement};

use crate::dom;

thread_local! {
    static WANTED: RefCell<Option<String>> = const { RefCell::new(None) };
    static BUSY: Cell<bool> = const { Cell::new(false) };
}

/// How long a layer takes to fade, a little past `--fade`.
const FADE_MS: i32 = 700;
/// A thumbnail that hasn't decoded by then is skipped.
const DECODE_MS: i32 = 3000;

/// The glow behind the viewer, with the layers [`want`] paints.
#[component]
pub fn PostGlow(#[prop(into)] thumb: Signal<Option<String>>) -> impl IntoView {
    Effect::new(move |_| want(thumb.get()));
    on_cleanup(|| WANTED.with_borrow_mut(|w| *w = None));
    view! {
        <div class="post-glow" aria-hidden="true">
            <canvas class="glow-layer"></canvas>
            <canvas class="glow-layer"></canvas>
        </div>
    }
}

/// The picture the light should come from.
fn want(url: Option<String>) {
    WANTED.with_borrow_mut(|w| *w = url);
    if BUSY.get() || dom::reduced_motion() || reduced_effects() {
        return;
    }
    BUSY.set(true);
    spawn_local(async {
        run().await;
        BUSY.set(false);
    });
}

/// Effects turned down, or the battery saver on: no glow.
fn reduced_effects() -> bool {
    document()
        .and_then(|d| {
            d.query_selector(r#".app[data-effects="lite"], .app[data-saver="on"]"#)
                .ok()
                .flatten()
        })
        .is_some()
}

fn document() -> Option<Document> {
    web_sys::window()?.document()
}

async fn run() {
    loop {
        let Some(url) = WANTED.with_borrow(Clone::clone) else {
            return;
        };
        let Some(root) = document().and_then(|d| d.query_selector(".post-glow").ok().flatten())
        else {
            return;
        };
        let front = root.query_selector(".glow-layer.on").ok().flatten();
        if front
            .as_ref()
            .and_then(|f| f.get_attribute("data-url"))
            .is_some_and(|u| u == url)
        {
            return;
        }
        let Ok(img) = HtmlImageElement::new() else {
            return;
        };
        img.set_src(&url);
        if !decoded(&img).await {
            // Another picture is wanted by now: on to that one.
            if WANTED.with_borrow(|w| w.as_deref() == Some(url.as_str())) {
                return;
            }
            continue;
        }
        if !root.is_connected() {
            continue;
        }
        let front = root.query_selector(".glow-layer.on").ok().flatten();
        let Some(back) = layers(&root)
            .into_iter()
            .find(|l| !l.class_list().contains("on"))
        else {
            return;
        };
        if paint(&back, &img).is_none() {
            return;
        }
        let _ = back.set_attribute("data-url", &url);
        let _ = back.class_list().add_1("on");
        if let Some(front) = front {
            let _ = front.class_list().remove_1("on");
        }
        // Fullscreen's own light, inside the stage, where the page's can't show.
        if let Some(full) = document()
            .and_then(|d| d.query_selector(".stage-glow").ok().flatten())
            .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
        {
            let _ = paint(&full, &img);
        }
        dom::sleep(FADE_MS).await;
    }
}

fn layers(root: &Element) -> Vec<HtmlCanvasElement> {
    let Ok(list) = root.query_selector_all(".glow-layer") else {
        return Vec::new();
    };
    (0..list.length())
        .filter_map(|i| list.item(i))
        .filter_map(|n| n.dyn_into::<HtmlCanvasElement>().ok())
        .collect()
}

/// Whether the picture decoded in time.
async fn decoded(img: &HtmlImageElement) -> bool {
    let timeout = js_sys::Promise::new(&mut |_, fail| {
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&fail, DECODE_MS);
        }
    });
    let race = js_sys::Promise::race(&js_sys::Array::of2(&img.decode(), &timeout));
    JsFuture::from(race).await.is_ok()
}

fn context(canvas: &HtmlCanvasElement) -> Option<CanvasRenderingContext2d> {
    let ctx: CanvasRenderingContext2d = canvas.get_context("2d").ok()??.dyn_into().ok()?;
    // web-sys has no setter for it.
    let _ = js_sys::Reflect::set(
        &ctx,
        &JsValue::from_str("imageSmoothingQuality"),
        &JsValue::from_str("high"),
    );
    Some(ctx)
}

/// A canvas of `w` by `h` pixels, off the page.
fn scene(
    document: &Document,
    (w, h): (u32, u32),
) -> Option<(HtmlCanvasElement, CanvasRenderingContext2d)> {
    let canvas: HtmlCanvasElement = document.create_element("canvas").ok()?.dyn_into().ok()?;
    canvas.set_width(w);
    canvas.set_height(h);
    let ctx = context(&canvas)?;
    Some((canvas, ctx))
}

/// Paints `img`, smudged and a little more colourful, onto `el` in the shape
/// of its box. `None` if the browser wouldn't draw.
fn paint(el: &HtmlCanvasElement, img: &HtmlImageElement) -> Option<()> {
    let document = document()?;
    let rect = el.get_bounding_client_rect();
    // Fullscreen's canvas is hidden, with no size, until fullscreen starts:
    // it will be the window's shape then.
    let (bw, bh) = if rect.width() > 0.0 && rect.height() > 0.0 {
        (rect.width(), rect.height())
    } else {
        dom::window_size()
    };
    let aspect = if bw > 0.0 && bh > 0.0 { bw / bh } else { 1.0 };
    let size = |long: f64| -> (u32, u32) {
        let short = |v: f64| v.round().max(2.0) as u32;
        if aspect >= 1.0 {
            (long as u32, short(long / aspect))
        } else {
            (short(long * aspect), long as u32)
        }
    };
    let (w, h) = (
        f64::from(img.natural_width()),
        f64::from(img.natural_height()),
    );
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    let (tiny, small) = scene(&document, size(24.0))?;
    let (copy, held) = scene(&document, size(24.0))?;
    let (middle, down) = scene(&document, size(64.0))?;
    let (mw, mh) = (f64::from(middle.width()), f64::from(middle.height()));
    let (tw, th) = (f64::from(tiny.width()), f64::from(tiny.height()));
    // The middle of the picture, in the box's shape.
    let crop = (w / mw).min(h / mh);
    let (cw, ch) = (mw * crop, mh * crop);
    down.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
        img,
        (w - cw) / 2.0,
        (h - ch) / 2.0,
        cw,
        ch,
        0.0,
        0.0,
        mw,
        mh,
    )
    .ok()?;
    small
        .draw_image_with_html_canvas_element_and_dw_and_dh(&middle, 0.0, 0.0, tw, th)
        .ok()?;
    // More colour where the picture has any: the grey between two colours
    // would turn into a third if this came after they're mixed. It can't add
    // any to what's transparent.
    small.set_global_composite_operation("saturation").ok()?;
    small.set_global_alpha(0.4);
    small.set_fill_style_str("#f00");
    small.fill_rect(0.0, 0.0, tw, th);
    small
        .set_global_composite_operation("destination-in")
        .ok()?;
    small.set_global_alpha(1.0);
    small
        .draw_image_with_html_canvas_element_and_dw_and_dh(&middle, 0.0, 0.0, tw, th)
        .ok()?;
    small.set_global_composite_operation("source-over").ok()?;
    // Averaged with copies of itself all around, or scaling up shows its grid.
    for r in [3.0, 2.0, 1.0] {
        held.clear_rect(0.0, 0.0, tw, th);
        held.draw_image_with_html_canvas_element(&tiny, 0.0, 0.0)
            .ok()?;
        let mut seen = 1.0;
        for (x, y) in [
            (r, 0.0),
            (-r, 0.0),
            (0.0, r),
            (0.0, -r),
            (r, r),
            (-r, -r),
            (r, -r),
            (-r, r),
        ] {
            seen += 1.0;
            small.set_global_alpha(1.0 / seen);
            small
                .draw_image_with_html_canvas_element(&copy, x, y)
                .ok()?;
        }
    }
    small.set_global_alpha(1.0);
    let (ow, oh) = size(96.0);
    el.set_width(ow);
    el.set_height(oh);
    let out = context(el)?;
    out.draw_image_with_html_canvas_element_and_dw_and_dh(
        &tiny,
        0.0,
        0.0,
        f64::from(ow),
        f64::from(oh),
    )
    .ok()?;
    Some(())
}
