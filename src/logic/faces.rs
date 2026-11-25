//! Where the people's faces are in a picture, without a browser. A box is in
//! thousandths of the picture's width and height from its top-left corner,
//! as the server keeps it: from the pointer on the screen to a box, drawn,
//! moved and resized inside the picture, and back to the screen, with the
//! square the server cuts a face to and when two boxes mark the same face.

use starbooru_core::{FACE_SCALE, FaceBox};

/// The whole picture on the box's scale.
const SCALE: f64 = FACE_SCALE as f64;
/// The least a box may be, in thousandths of the picture's shorter side.
const LEAST: f64 = 30.0;
/// A tap makes a square this many thousandths of the shorter side.
const TAP: f64 = 160.0;
/// A press that moves less than this many CSS pixels is a tap.
pub const TAP_PIXELS: f64 = 4.0;
/// Room the server leaves around a face, on each side, as a share of the
/// box's longer side (`faces.rs` in starbooru-server).
const CROP_MARGIN: f64 = 0.25;

/// The picture's box on the screen, in CSS pixels, as the browser measures
/// it (`getBoundingClientRect`), zoom and pan included.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

/// The point under the pointer at `x`, `y` on the screen, on the box's scale
/// and inside the picture. `None` for a picture with no size.
pub fn at(frame: Frame, x: f64, y: f64) -> Option<(f64, f64)> {
    let fine = [frame.left, frame.top, frame.width, frame.height, x, y]
        .iter()
        .all(|v| v.is_finite());
    if !fine || frame.width <= 0.0 || frame.height <= 0.0 {
        return None;
    }
    let along = |from: f64, size: f64, v: f64| ((v - from) * SCALE / size).clamp(0.0, SCALE);
    Some((
        along(frame.left, frame.width, x),
        along(frame.top, frame.height, y),
    ))
}

/// How far a move of `dx`, `dy` CSS pixels goes on the box's scale.
pub fn step(frame: Frame, dx: f64, dy: f64) -> (f64, f64) {
    let along = |d: f64, size: f64| {
        if size > 0.0 && d.is_finite() {
            d * SCALE / size
        } else {
            0.0
        }
    };
    (along(dx, frame.width), along(dy, frame.height))
}

/// A picture's width and height in pixels, or a square when unknown.
fn shape_of(shape: Option<(i32, i32)>) -> (f64, f64) {
    match shape {
        Some((w, h)) if w > 0 && h > 0 => (f64::from(w), f64::from(h)),
        _ => (1.0, 1.0),
    }
}

/// `per_mille` thousandths of the picture's shorter side, on the box's scale
/// of each side: a square on the picture.
fn square(shape: Option<(i32, i32)>, per_mille: f64) -> (f64, f64) {
    let (w, h) = shape_of(shape);
    let short = w.min(h);
    (
        (per_mille * short / w).max(1.0),
        (per_mille * short / h).max(1.0),
    )
}

/// The least a box may be across and down, on the box's scale.
pub fn least(shape: Option<(i32, i32)>) -> (f64, f64) {
    square(shape, LEAST)
}

/// Edges on the box's scale, as a box inside the picture: rounded, at least
/// one thousandth, and moved in where they stick out.
fn fit(left: f64, top: f64, right: f64, bottom: f64) -> FaceBox {
    let edge = |from: f64, to: f64| -> (u16, u16) {
        let (from, to) = if from <= to { (from, to) } else { (to, from) };
        let len = (to - from).round().clamp(1.0, SCALE);
        let at = from.round().clamp(0.0, SCALE - len);
        // Both are whole numbers within 0..=1000 by now.
        (at as u16, len as u16)
    };
    let fine = |v: f64| if v.is_finite() { v } else { 0.0 };
    let (x, w) = edge(fine(left), fine(right));
    let (y, h) = edge(fine(top), fine(bottom));
    FaceBox { x, y, w, h }
}

/// The box a drag from `from` to `to` draws, both on the box's scale. Too
/// small a drag grows from where it started, the way it went, to the least.
pub fn drawn(from: (f64, f64), to: (f64, f64), shape: Option<(i32, i32)>) -> FaceBox {
    let (least_w, least_h) = least(shape);
    let reach = |start: f64, end: f64, least: f64| {
        if (end - start).abs() >= least {
            end
        } else if end < start {
            start - least
        } else {
            start + least
        }
    };
    let right = reach(from.0, to.0, least_w);
    let bottom = reach(from.1, to.1, least_h);
    fit(from.0, from.1, right, bottom)
}

/// The square box a tap at `at` makes, around it.
pub fn tapped(at: (f64, f64), shape: Option<(i32, i32)>) -> FaceBox {
    let (w, h) = square(shape, TAP);
    let (left, top) = (at.0 - w / 2.0, at.1 - h / 2.0);
    // Kept whole at the picture's edges: moved in rather than cut.
    let left = left.clamp(0.0, (SCALE - w).max(0.0));
    let top = top.clamp(0.0, (SCALE - h).max(0.0));
    fit(left, top, left + w, top + h)
}

/// `face` moved by `dx`, `dy` on the box's scale, its size kept and its
/// edges inside the picture.
pub fn moved(face: FaceBox, dx: f64, dy: f64) -> FaceBox {
    let (w, h) = (f64::from(face.w), f64::from(face.h));
    let shift = |at: u16, len: f64, d: f64| {
        let d = if d.is_finite() { d } else { 0.0 };
        (f64::from(at) + d).clamp(0.0, (SCALE - len).max(0.0))
    };
    let (x, y) = (shift(face.x, w, dx), shift(face.y, h, dy));
    fit(x, y, x + w, y + h)
}

/// A corner of a box, to resize it by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    pub const ALL: [Corner; 4] = [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ];

    /// Its name in the page, for the handle's place and pointer.
    pub fn as_str(self) -> &'static str {
        match self {
            Corner::TopLeft => "tl",
            Corner::TopRight => "tr",
            Corner::BottomLeft => "bl",
            Corner::BottomRight => "br",
        }
    }
}

/// `face` with `corner` dragged to `to`, on the box's scale. The corner
/// across from it stays, and the box never turns inside out or gets smaller
/// than the least.
pub fn resized(
    face: FaceBox,
    corner: Corner,
    to: (f64, f64),
    shape: Option<(i32, i32)>,
) -> FaceBox {
    let (least_w, least_h) = least(shape);
    let (left, top) = (f64::from(face.x), f64::from(face.y));
    let (right, bottom) = (left + f64::from(face.w), top + f64::from(face.h));
    let to = (to.0.clamp(0.0, SCALE), to.1.clamp(0.0, SCALE));
    let lower = |v: f64, stay: f64, least: f64| v.min(stay - least).max(0.0);
    let upper = |v: f64, stay: f64, least: f64| v.max(stay + least).min(SCALE);
    let (left, right) = match corner {
        Corner::TopLeft | Corner::BottomLeft => (lower(to.0, right, least_w), right),
        Corner::TopRight | Corner::BottomRight => (left, upper(to.0, left, least_w)),
    };
    let (top, bottom) = match corner {
        Corner::TopLeft | Corner::TopRight => (lower(to.1, bottom, least_h), bottom),
        Corner::BottomLeft | Corner::BottomRight => (top, upper(to.1, top, least_h)),
    };
    fit(left, top, right, bottom)
}

/// Where `face` sits in the picture's box on the page, in percent of it:
/// left, top, width and height.
pub fn percents(face: FaceBox) -> [f64; 4] {
    [face.x, face.y, face.w, face.h].map(|v| f64::from(v) * 100.0 / SCALE)
}

/// `face`'s place as an inline style, for an element over the picture.
pub fn place_style(face: FaceBox) -> String {
    let [left, top, width, height] = percents(face);
    format!("left: {left:.2}%; top: {top:.2}%; width: {width:.2}%; height: {height:.2}%")
}

/// How much of the smaller of two boxes they share, from 0 to 1. A face
/// marked by hand may be larger or smaller than the one the AI finds, and
/// one inside the other is the same face. The share is the same in pixels,
/// whatever the picture's shape.
pub fn overlap(a: FaceBox, b: FaceBox) -> f64 {
    let span = |at: u16, len: u16| (f64::from(at), f64::from(at) + f64::from(len));
    let shared = |(a0, a1): (f64, f64), (b0, b1): (f64, f64)| (a1.min(b1) - a0.max(b0)).max(0.0);
    let across = shared(span(a.x, a.w), span(b.x, b.w));
    let down = shared(span(a.y, a.h), span(b.y, b.h));
    let smaller = (f64::from(a.w) * f64::from(a.h)).min(f64::from(b.w) * f64::from(b.h));
    if smaller > 0.0 {
        (across * down / smaller).min(1.0)
    } else {
        0.0
    }
}

/// Two boxes sharing at least this much of the smaller are one face.
pub const SAME_FACE: f64 = 0.5;

/// Whether two boxes mark the same face.
pub fn same_face(a: FaceBox, b: FaceBox) -> bool {
    overlap(a, b) >= SAME_FACE
}

/// Whether `url` is a crop the server sent before it kept one: a JPEG or
/// PNG in a `data:` address, as finding people answers. Anything else in a
/// `data:` address isn't shown.
pub fn is_preview(url: &str) -> bool {
    ["data:image/jpeg;base64,", "data:image/png;base64,"]
        .iter()
        .any(|head| url.starts_with(head))
}

/// The square the server cuts `face` to, as a picture's background: its size
/// and position in percent, so a circle shows the crop before the server
/// has made it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CropView {
    pub size: (f64, f64),
    pub position: (f64, f64),
}

impl CropView {
    /// As inline CSS for the circle.
    pub fn style(&self) -> String {
        format!(
            "background-size: {:.2}% {:.2}%; background-position: {:.2}% {:.2}%",
            self.size.0, self.size.1, self.position.0, self.position.1
        )
    }
}

/// The server's square around `face` on a `shape` picture: around the box's
/// middle, a quarter of its longer side larger on each side, and moved or
/// shrunk to stay inside. `None` without a size.
pub fn crop_view(face: FaceBox, shape: Option<(i32, i32)>) -> Option<CropView> {
    let (w, h) = match shape {
        Some((w, h)) if w > 0 && h > 0 => (f64::from(w), f64::from(h)),
        _ => return None,
    };
    let [x, y, bw, bh] = [face.x, face.y, face.w, face.h].map(|v| f64::from(v) / SCALE);
    let (bw, bh) = (bw * w, bh * h);
    let (cx, cy) = (x * w + bw / 2.0, y * h + bh / 2.0);
    let side = (bw.max(bh) * (1.0 + 2.0 * CROP_MARGIN))
        .max(1.0)
        .min(w.min(h));
    let left = (cx - side / 2.0).clamp(0.0, w - side);
    let top = (cy - side / 2.0).clamp(0.0, h - side);
    // Background positions are shares of the room left over.
    let share = |at: f64, room: f64| if room > 0.0 { at / room * 100.0 } else { 0.0 };
    Some(CropView {
        size: (w / side * 100.0, h / side * 100.0),
        position: (share(left, w - side), share(top, h - side)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at_box(x: u16, y: u16, w: u16, h: u16) -> FaceBox {
        FaceBox { x, y, w, h }
    }

    const FRAME: Frame = Frame {
        left: 100.0,
        top: 50.0,
        width: 400.0,
        height: 200.0,
    };

    #[test]
    fn the_pointer_lands_on_the_picture_in_thousandths() {
        assert_eq!(at(FRAME, 100.0, 50.0), Some((0.0, 0.0)));
        assert_eq!(at(FRAME, 300.0, 150.0), Some((500.0, 500.0)));
        assert_eq!(at(FRAME, 500.0, 250.0), Some((1000.0, 1000.0)));
        // Off the picture, it stays at its edge.
        assert_eq!(at(FRAME, 0.0, 400.0), Some((0.0, 1000.0)));
        // Zoomed in twice, the same point is twice as far on the screen.
        let zoomed = Frame {
            width: 800.0,
            height: 400.0,
            ..FRAME
        };
        assert_eq!(at(zoomed, 500.0, 250.0), Some((500.0, 500.0)));
        // Nothing to land on.
        let flat = Frame {
            width: 0.0,
            ..FRAME
        };
        assert_eq!(at(flat, 1.0, 1.0), None);
        assert_eq!(at(FRAME, f64::NAN, 1.0), None);
        assert_eq!(step(FRAME, 40.0, -20.0), (100.0, -100.0));
        assert_eq!(step(flat, 40.0, f64::NAN), (0.0, 0.0));
    }

    #[test]
    fn the_least_box_is_a_square_on_the_picture() {
        // 2000 × 1000: 3% of 1000 px is 30 px, 15 thousandths across.
        assert_eq!(least(Some((2000, 1000))), (15.0, 30.0));
        assert_eq!(least(None), (30.0, 30.0));
        assert_eq!(least(Some((0, 5))), (30.0, 30.0));
    }

    #[test]
    fn a_drag_draws_a_box_whichever_way_it_goes() {
        let shape = Some((1000, 1000));
        assert_eq!(
            drawn((100.0, 200.0), (300.0, 500.0), shape),
            at_box(100, 200, 200, 300)
        );
        // Up and to the left: the same box.
        assert_eq!(
            drawn((300.0, 500.0), (100.0, 200.0), shape),
            at_box(100, 200, 200, 300)
        );
        // Too small: it grows from the start, the way the drag went.
        assert_eq!(
            drawn((500.0, 500.0), (495.0, 510.0), shape),
            at_box(470, 500, 30, 30)
        );
        // At the edge it is moved in, and never empty.
        let edge = drawn((999.0, 999.0), (1000.0, 1000.0), shape);
        assert!(edge.is_valid(), "{edge:?}");
        assert_eq!(edge, at_box(970, 970, 30, 30));
        let nonsense = drawn((f64::NAN, 0.0), (f64::INFINITY, 0.0), shape);
        assert!(nonsense.is_valid(), "{nonsense:?}");
    }

    #[test]
    fn a_tap_makes_a_square_around_it_inside_the_picture() {
        // 16% of the shorter 1000 px of a 2000 × 1000 picture: 160 px.
        assert_eq!(
            tapped((500.0, 500.0), Some((2000, 1000))),
            at_box(460, 420, 80, 160)
        );
        // In the corner, moved in whole.
        assert_eq!(
            tapped((0.0, 1000.0), Some((1000, 1000))),
            at_box(0, 840, 160, 160)
        );
        assert!(tapped((1000.0, 0.0), None).is_valid());
    }

    #[test]
    fn a_box_moves_but_stays_inside() {
        let face = at_box(100, 100, 200, 300);
        assert_eq!(moved(face, 50.0, -20.0), at_box(150, 80, 200, 300));
        assert_eq!(moved(face, 5000.0, 5000.0), at_box(800, 700, 200, 300));
        assert_eq!(moved(face, -5000.0, -5000.0), at_box(0, 0, 200, 300));
        assert_eq!(moved(face, f64::NAN, 0.0), face);
        // Rounded to the scale.
        assert_eq!(moved(face, 0.4, 0.6), at_box(100, 101, 200, 300));
    }

    #[test]
    fn a_corner_resizes_and_the_one_across_stays() {
        let shape = Some((1000, 1000));
        let face = at_box(100, 100, 200, 200);
        assert_eq!(
            resized(face, Corner::BottomRight, (500.0, 400.0), shape),
            at_box(100, 100, 400, 300)
        );
        assert_eq!(
            resized(face, Corner::TopLeft, (50.0, 0.0), shape),
            at_box(50, 0, 250, 300)
        );
        assert_eq!(
            resized(face, Corner::TopRight, (350.0, 150.0), shape),
            at_box(100, 150, 250, 150)
        );
        assert_eq!(
            resized(face, Corner::BottomLeft, (150.0, 250.0), shape),
            at_box(150, 100, 150, 150)
        );
        // Past the corner across, it stops at the least size.
        assert_eq!(
            resized(face, Corner::BottomRight, (0.0, 0.0), shape),
            at_box(100, 100, 30, 30)
        );
        assert_eq!(
            resized(face, Corner::TopLeft, (900.0, 900.0), shape),
            at_box(270, 270, 30, 30)
        );
        // Out of the picture, it stops at the edge.
        assert_eq!(
            resized(face, Corner::BottomRight, (2000.0, -50.0), shape),
            at_box(100, 100, 900, 30)
        );
        for corner in Corner::ALL {
            assert!(!corner.as_str().is_empty());
            let any = resized(face, corner, (f64::NAN, 1e9), shape);
            assert!(any.is_valid(), "{corner:?} {any:?}");
        }
    }

    #[test]
    fn a_box_is_placed_in_percent_of_the_picture() {
        let face = at_box(412, 96, 130, 180);
        assert_eq!(percents(face), [41.2, 9.6, 13.0, 18.0]);
        assert_eq!(
            place_style(face),
            "left: 41.20%; top: 9.60%; width: 13.00%; height: 18.00%"
        );
    }

    #[test]
    fn boxes_over_the_same_face_overlap() {
        let face = at_box(400, 200, 100, 100);
        assert_eq!(overlap(face, face), 1.0);
        // Marked loosely by hand around the AI's: the small one is inside.
        let loose = at_box(370, 160, 180, 200);
        assert_eq!(overlap(face, loose), 1.0);
        assert!(same_face(loose, face));
        // Half of it over the other.
        let half = at_box(450, 200, 100, 100);
        assert!((overlap(face, half) - 0.5).abs() < 1e-9);
        assert!(same_face(face, half));
        // Side by side, touching or a little over: two people.
        assert_eq!(overlap(face, at_box(500, 200, 100, 100)), 0.0);
        let neighbour = at_box(480, 220, 100, 100);
        assert!(!same_face(face, neighbour), "{}", overlap(face, neighbour));
        assert_eq!(overlap(face, at_box(0, 0, 10, 10)), 0.0);
        // An empty box overlaps nothing.
        assert_eq!(overlap(face, at_box(420, 220, 0, 10)), 0.0);
        // Whichever goes first.
        assert_eq!(overlap(half, face), overlap(face, half));
    }

    #[test]
    fn only_a_pictures_crop_shows_from_a_data_address() {
        assert!(is_preview("data:image/jpeg;base64,/9j/4AAQ"));
        assert!(is_preview("data:image/png;base64,iVBOR"));
        assert!(!is_preview("data:text/html;base64,PHNjcmlwdD4="));
        assert!(!is_preview("data:image/svg+xml;base64,PHN2Zz4="));
        assert!(!is_preview("/media/face/a.jpg?sig=x"));
        assert!(!is_preview(""));
    }

    #[test]
    fn the_crop_shown_is_the_servers_square() {
        // A 1000 × 1000 picture with a 100 px box in the middle: a 150 px
        // square around it, so the picture is 666.67% of the circle.
        let crop = crop_view(at_box(450, 450, 100, 100), Some((1000, 1000))).unwrap();
        assert!((crop.size.0 - 666.666).abs() < 0.01, "{crop:?}");
        assert!((crop.position.0 - 50.0).abs() < 1e-9, "{crop:?}");
        // The longer side counts, and a box at the corner stays inside.
        let corner = crop_view(at_box(0, 0, 50, 200), Some((1000, 1000))).unwrap();
        assert_eq!(corner.position, (0.0, 0.0));
        assert!((corner.size.1 - 1000.0 / 3.0).abs() < 0.01, "{corner:?}");
        // Bigger than the picture's shorter side: the whole of it.
        let all = crop_view(at_box(0, 0, 1000, 1000), Some((800, 400))).unwrap();
        assert_eq!(all.size, (200.0, 100.0));
        assert_eq!(all.position.1, 0.0);
        assert!(crop_view(at_box(0, 0, 10, 10), None).is_none());
        assert!(crop.style().starts_with("background-size: 666.67% 666.67%"));
    }
}
