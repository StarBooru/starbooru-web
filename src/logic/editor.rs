//! The media editor's sums: the crop as shares of the turned picture, drags
//! and key nudges of its sides and corners, proportions, turns, flips and
//! straightening, undo and redo, the picture's style in the preview, the trim
//! on a video's timeline, the request all of it makes, and where the tools
//! stand. Ported from the reference's `pages/editor.rs`, with its tests.

use starbooru_core::edit::MIN_TRIM_MS;
use starbooru_core::{Crop, EditError, EditKind, MAX_STRAIGHTEN, MediaEdit, Post, Trim, edit_kind};

/// Smallest crop side, as a share of the picture's shorter side.
pub const MIN_CROP: f64 = 0.04;
/// How many steps undo can go back.
pub const MAX_HISTORY: usize = 100;
/// Room around the picture for the crop handles, in pixels.
pub const FRAME_PAD: f64 = 20.0;
/// Fixed proportions offered, as width by height of the upright version.
pub const RATIOS: [(u16, u16); 5] = [(1, 1), (3, 4), (2, 3), (4, 5), (9, 16)];
/// The largest side a custom proportion takes.
pub const MAX_RATIO_SIDE: u16 = 100;
/// One press of an arrow key on the crop, as a share of the picture's shorter
/// side; with Shift, ten times that.
const NUDGE: f64 = 0.01;
const BIG_NUDGE: f64 = 0.1;
/// One press of an arrow key on a trim handle, and with Shift.
const TRIM_STEP_MS: i64 = 100;
const BIG_TRIM_STEP_MS: i64 = 1_000;
/// A dragged trim handle lands on tenths of a second, like the fields.
const TRIM_SNAP_MS: i64 = 100;
/// The tools stand beside the picture from this window width, when the
/// window is wider than tall.
const BESIDE_FROM: f64 = 640.0;

/// `v` limited to `lo..=hi`, where `f64::clamp` would panic on `lo > hi` or
/// NaN; `lo` wins then, and NaN is `lo`.
pub fn within(v: f64, lo: f64, hi: f64) -> f64 {
    if v.is_nan() { lo } else { v.min(hi).max(lo) }
}

/// A crop as shares (0 to 1) of the turned and flipped picture, so it doesn't
/// depend on how big the preview is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CropBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// What a drag holds: the whole box, one side, or a corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handle {
    Move,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Handle {
    /// The handles drawn on the box, with their compass names for the CSS.
    pub const ALL: [(Handle, &'static str); 8] = [
        (Handle::Top, "n"),
        (Handle::Bottom, "s"),
        (Handle::Left, "w"),
        (Handle::Right, "e"),
        (Handle::TopLeft, "nw"),
        (Handle::TopRight, "ne"),
        (Handle::BottomLeft, "sw"),
        (Handle::BottomRight, "se"),
    ];

    /// Which sides move: -1 the left (or top), 1 the right (or bottom), 0 neither.
    pub fn sides(self) -> (i8, i8) {
        match self {
            Handle::Move => (0, 0),
            Handle::Top => (0, -1),
            Handle::Bottom => (0, 1),
            Handle::Left => (-1, 0),
            Handle::Right => (1, 0),
            Handle::TopLeft => (-1, -1),
            Handle::TopRight => (1, -1),
            Handle::BottomLeft => (-1, 1),
            Handle::BottomRight => (1, 1),
        }
    }
}

/// The way an arrow key points, as `(dx, dy)`.
pub fn arrow(key: &str) -> Option<(i8, i8)> {
    match key {
        "ArrowLeft" => Some((-1, 0)),
        "ArrowRight" => Some((1, 0)),
        "ArrowUp" => Some((0, -1)),
        "ArrowDown" => Some((0, 1)),
        _ => None,
    }
}

/// The edges of a box in pixels.
#[derive(Debug, Clone, Copy)]
struct Edges {
    l: f64,
    t: f64,
    r: f64,
    b: f64,
}

impl CropBox {
    pub const FULL: CropBox = CropBox {
        x: 0.0,
        y: 0.0,
        w: 1.0,
        h: 1.0,
    };

    fn edges(self, fw: f64, fh: f64) -> Edges {
        Edges {
            l: self.x * fw,
            t: self.y * fh,
            r: (self.x + self.w) * fw,
            b: (self.y + self.h) * fh,
        }
    }

    /// Back from pixels to shares. Anything odd (a zero-sized frame, say)
    /// gives the whole picture.
    fn from_edges(e: Edges, fw: f64, fh: f64) -> CropBox {
        let b = CropBox {
            x: e.l / fw,
            y: e.t / fh,
            w: (e.r - e.l) / fw,
            h: (e.b - e.t) / fh,
        };
        if [b.x, b.y, b.w, b.h].iter().all(|v| v.is_finite()) && b.w > 0.0 && b.h > 0.0 {
            b
        } else {
            CropBox::FULL
        }
    }

    /// The same part of the picture after it is mirrored.
    pub fn mirrored(self, horizontal: bool) -> CropBox {
        if horizontal {
            CropBox {
                x: 1.0 - self.x - self.w,
                ..self
            }
        } else {
            CropBox {
                y: 1.0 - self.y - self.h,
                ..self
            }
        }
    }

    /// The same part of the picture after it turns a quarter.
    pub fn turned(self, clockwise: bool) -> CropBox {
        if clockwise {
            CropBox {
                x: 1.0 - self.y - self.h,
                y: self.x,
                w: self.h,
                h: self.w,
            }
        } else {
            CropBox {
                x: self.y,
                y: 1.0 - self.x - self.w,
                w: self.h,
                h: self.w,
            }
        }
    }

    /// The biggest box of `ratio` (width over height, in pixels) that fits in
    /// a `fw` × `fh` frame, as near the middle of this one as it can be.
    pub fn with_ratio(self, ratio: f64, fw: f64, fh: f64) -> CropBox {
        if !(ratio.is_finite() && ratio > 0.0 && fw > 0.0 && fh > 0.0) {
            return self;
        }
        let w = fw.min(fh * ratio);
        let h = w / ratio;
        let e = self.edges(fw, fh);
        let (cx, cy) = ((e.l + e.r) / 2.0, (e.t + e.b) / 2.0);
        let l = within(cx - w / 2.0, 0.0, fw - w);
        let t = within(cy - h / 2.0, 0.0, fh - h);
        CropBox::from_edges(
            Edges {
                l,
                t,
                r: l + w,
                b: t + h,
            },
            fw,
            fh,
        )
    }

    /// Moves `handle` by `dx`, `dy` pixels of a `fw` × `fh` frame, keeping the
    /// box inside it, no smaller than `MIN_CROP`, and at `ratio` (width over
    /// height in pixels) when there is one.
    pub fn dragged(
        self,
        handle: Handle,
        dx: f64,
        dy: f64,
        fw: f64,
        fh: f64,
        ratio: Option<f64>,
    ) -> CropBox {
        if !(fw > 0.0 && fh > 0.0) {
            return self;
        }
        let e = self.edges(fw, fh);
        let (w, h) = (e.r - e.l, e.b - e.t);
        let min = MIN_CROP * fw.min(fh);
        let (hx, hy) = handle.sides();
        let out = match (handle, ratio.filter(|r| r.is_finite() && *r > 0.0)) {
            (Handle::Move, _) => {
                let l = within(e.l + dx, 0.0, fw - w);
                let t = within(e.t + dy, 0.0, fh - h);
                Edges {
                    l,
                    t,
                    r: l + w,
                    b: t + h,
                }
            }
            (_, None) => {
                let mut n = e;
                match hx {
                    -1 => n.l = within(e.l + dx, 0.0, e.r - min),
                    1 => n.r = within(e.r + dx, e.l + min, fw),
                    _ => {}
                }
                match hy {
                    -1 => n.t = within(e.t + dy, 0.0, e.b - min),
                    1 => n.b = within(e.b + dy, e.t + min, fh),
                    _ => {}
                }
                n
            }
            (_, Some(ratio)) => {
                // The shorter side stays at least `min`.
                let min_w = min * ratio.max(1.0);
                // Room from the side that stays put, or around the middle for
                // the direction a side handle doesn't move.
                let room_w = match hx {
                    1 => fw - e.l,
                    -1 => e.r,
                    _ => 2.0 * ((e.l + e.r) / 2.0).min(fw - (e.l + e.r) / 2.0),
                };
                let room_h = match hy {
                    1 => fh - e.t,
                    -1 => e.b,
                    _ => 2.0 * ((e.t + e.b) / 2.0).min(fh - (e.t + e.b) / 2.0),
                };
                let wanted_w = w + f64::from(hx) * dx;
                let wanted_h = h + f64::from(hy) * dy;
                // A corner follows whichever way the pointer went further.
                let wanted = match (hx, hy) {
                    (0, _) => wanted_h * ratio,
                    (_, 0) => wanted_w,
                    _ => wanted_w.max(wanted_h * ratio),
                };
                let nw = wanted.max(min_w).min(room_w).min(room_h * ratio);
                let nh = nw / ratio;
                let l = match hx {
                    1 => e.l,
                    -1 => e.r - nw,
                    _ => (e.l + e.r) / 2.0 - nw / 2.0,
                };
                let t = match hy {
                    1 => e.t,
                    -1 => e.b - nh,
                    _ => (e.t + e.b) / 2.0 - nh / 2.0,
                };
                Edges {
                    l,
                    t,
                    r: l + nw,
                    b: t + nh,
                }
            }
        };
        CropBox::from_edges(out, fw, fh)
    }

    /// One press of an arrow key `(dx, dy)` on `handle`: a hundredth of the
    /// picture's shorter side, or a tenth with Shift (`big`).
    pub fn nudged(
        self,
        handle: Handle,
        (dx, dy): (i8, i8),
        big: bool,
        (fw, fh): (f64, f64),
        ratio: Option<f64>,
    ) -> CropBox {
        let step = fw.min(fh) * if big { BIG_NUDGE } else { NUDGE };
        let step = step.max(1.0);
        // A corner held to a proportion grows or shrinks along both sides at
        // once, or one key alone would do nothing.
        let (hx, hy) = handle.sides();
        let (dx, dy) = if ratio.is_some() && hx != 0 && hy != 0 && (dx == 0) != (dy == 0) {
            let grow = if dx != 0 { dx * hx } else { dy * hy };
            (grow * hx, grow * hy)
        } else {
            (dx, dy)
        };
        self.dragged(
            handle,
            f64::from(dx) * step,
            f64::from(dy) * step,
            fw,
            fh,
            ratio,
        )
    }

    /// The crop in pixels of a `width` × `height` picture; `None` when it
    /// keeps the whole picture. Always at least one pixel, inside the picture.
    pub fn pixels(self, width: u32, height: u32) -> Option<Crop> {
        let side = |at: f64, len: f64, max: u32| {
            let m = f64::from(max);
            let start = within((at * m).round(), 0.0, (m - 1.0).max(0.0));
            let end = within(((at + len) * m).round(), start + 1.0, m.max(start + 1.0));
            // Whole numbers within 0..=max, so the casts are exact.
            (start as u32, (end - start) as u32)
        };
        let (x, w) = side(self.x, self.w, width);
        let (y, h) = side(self.y, self.h, height);
        let crop = Crop {
            x,
            y,
            width: w,
            height: h,
        };
        (crop
            != Crop {
                x: 0,
                y: 0,
                width,
                height,
            })
        .then_some(crop)
    }

    /// Where the box sits over the frame, for its CSS.
    pub fn style(self) -> String {
        format!(
            "left: {:.3}%; top: {:.3}%; width: {:.3}%; height: {:.3}%",
            self.x * 100.0,
            self.y * 100.0,
            self.w * 100.0,
            self.h * 100.0
        )
    }
}

/// The crop's proportions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aspect {
    Free,
    /// The picture's own, as turned.
    Original,
    /// Width by height.
    Fixed(u16, u16),
}

impl Aspect {
    /// Width over height in pixels, for a `fw` × `fh` picture.
    pub fn ratio(self, fw: f64, fh: f64) -> Option<f64> {
        match self {
            Aspect::Free => None,
            Aspect::Original => (fh > 0.0).then(|| fw / fh),
            Aspect::Fixed(w, h) => (h > 0 && w > 0).then(|| f64::from(w) / f64::from(h)),
        }
    }

    /// The same proportions on their side.
    pub fn swapped(self) -> Aspect {
        match self {
            Aspect::Fixed(w, h) => Aspect::Fixed(h, w),
            other => other,
        }
    }

    /// One of the offered proportions, either way up.
    pub fn is_preset(self) -> bool {
        match self {
            Aspect::Fixed(w, h) => RATIOS
                .iter()
                .any(|&(a, b)| (a, b) == (w, h) || (b, a) == (w, h)),
            _ => true,
        }
    }
}

/// Reads one side of a custom proportion.
pub fn ratio_side(text: &str) -> Option<u16> {
    text.trim()
        .parse::<u16>()
        .ok()
        .filter(|v| (1..=MAX_RATIO_SIDE).contains(v))
}

/// The offered proportion `(a, b)` the way up the chips show: wider than tall
/// when `wide`.
pub fn oriented((a, b): (u16, u16), wide: bool) -> (u16, u16) {
    if wide {
        (a.max(b), a.min(b))
    } else {
        (a.min(b), a.max(b))
    }
}

/// Everything undo and redo step through.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    /// Turns, flips and straightening; its crop and trim stay empty.
    pub edit: MediaEdit,
    pub crop: CropBox,
    pub aspect: Aspect,
    /// The offered proportions are shown wider than tall.
    pub wide: bool,
}

impl Shape {
    pub fn new(width: u32, height: u32) -> Shape {
        Shape {
            edit: MediaEdit::default(),
            crop: CropBox::FULL,
            aspect: Aspect::Free,
            wide: width > height,
        }
    }

    /// The picture as turned, in pixels.
    pub fn frame(&self, width: u32, height: u32) -> (f64, f64) {
        let (w, h) = self.edit.turned_size(width, height);
        (f64::from(w.max(1)), f64::from(h.max(1)))
    }

    /// A quarter turn: the crop and its proportions turn with the picture.
    pub fn turn(&self, clockwise: bool) -> Shape {
        let mut edit = self.edit.clone();
        edit.turn(clockwise);
        Shape {
            edit,
            crop: self.crop.turned(clockwise),
            aspect: self.aspect.swapped(),
            wide: !self.wide,
        }
    }

    /// A mirror of the whole result: the straightening leans the other way.
    pub fn flip(&self, horizontal: bool) -> Shape {
        let mut edit = self.edit.clone();
        if horizontal {
            edit.flip_h = !edit.flip_h;
        } else {
            edit.flip_v = !edit.flip_v;
        }
        edit.straighten = edit.straighten.saturating_neg();
        Shape {
            edit,
            crop: self.crop.mirrored(horizontal),
            ..self.clone()
        }
    }

    /// Straightened by `degrees`, to the nearest tenth and no further than
    /// the server takes.
    pub fn straightened(&self, degrees: f64) -> Shape {
        let max = f64::from(MAX_STRAIGHTEN);
        let tenths = within((degrees * 10.0).round(), -max, max);
        let mut next = self.clone();
        next.edit.straighten = tenths as i16;
        next
    }

    /// How far it's straightened, in degrees.
    pub fn degrees(&self) -> f64 {
        f64::from(self.edit.straighten) / 10.0
    }

    /// New proportions, with the crop made to fit them.
    pub fn with_aspect(&self, aspect: Aspect, width: u32, height: u32) -> Shape {
        let (fw, fh) = self.frame(width, height);
        let crop = match aspect.ratio(fw, fh) {
            Some(r) => self.crop.with_ratio(r, fw, fh),
            None => self.crop,
        };
        let wide = match aspect {
            Aspect::Fixed(w, h) if w != h => w > h,
            _ => self.wide,
        };
        Shape {
            crop,
            aspect,
            wide,
            ..self.clone()
        }
    }

    /// The other way up for the offered proportions, and the crop's.
    pub fn swap_orientation(&self, width: u32, height: u32) -> Shape {
        let flipped = Shape {
            wide: !self.wide,
            ..self.clone()
        };
        match self.aspect {
            Aspect::Fixed(w, h) if w != h => {
                flipped.with_aspect(self.aspect.swapped(), width, height)
            }
            _ => flipped,
        }
    }

    /// The request this shape makes.
    pub fn request(&self, width: u32, height: u32) -> MediaEdit {
        let mut e = self.edit.clone();
        let (w, h) = e.turned_size(width, height);
        e.crop = self.crop.pixels(w, h);
        e
    }

    /// The crop's size in pixels, as it will be saved.
    pub fn cropped_size(&self, width: u32, height: u32) -> (u32, u32) {
        let (w, h) = self.edit.turned_size(width, height);
        self.crop
            .pixels(w, h)
            .map_or((w, h), |c| (c.width, c.height))
    }
}

/// Undo and redo.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct History {
    past: Vec<Shape>,
    future: Vec<Shape>,
}

impl History {
    /// `before` is where a change started from.
    pub fn record(&mut self, before: Shape) {
        self.past.push(before);
        if self.past.len() > MAX_HISTORY {
            self.past.remove(0);
        }
        self.future.clear();
    }

    pub fn undo(&mut self, now: &Shape) -> Option<Shape> {
        let back = self.past.pop()?;
        self.future.push(now.clone());
        Some(back)
    }

    pub fn redo(&mut self, now: &Shape) -> Option<Shape> {
        let ahead = self.future.pop()?;
        self.past.push(now.clone());
        Some(ahead)
    }

    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }
}

/// How far the shown picture turns to reach `rotate`, from `spin`: the short
/// way, so from 270° to 0° it goes a quarter turn forward.
pub fn spun(spin: i32, rotate: u16) -> i32 {
    let delta = (i32::from(rotate) - spin).rem_euclid(360);
    spin + if delta > 180 { delta - 360 } else { delta }
}

/// The frame's size in pixels: the turned picture made as big as `room`
/// allows, less space for the handles.
pub fn frame_size(fw: f64, fh: f64, room: (f64, f64)) -> (f64, f64) {
    let (aw, ah) = (
        (room.0 - 2.0 * FRAME_PAD).max(1.0),
        (room.1 - 2.0 * FRAME_PAD).max(1.0),
    );
    let k = (aw / fw).min(ah / fh);
    if k.is_finite() && k > 0.0 {
        (fw * k, fh * k)
    } else {
        (1.0, 1.0)
    }
}

/// The picture's style inside a frame of `size`: unturned, it is as big as
/// the frame turned back, and then turns, flips, grows and straightens.
/// `spin` is the quarter turn as shown (see [`spun`]).
pub fn picture_style(
    edit: &MediaEdit,
    spin: i32,
    size: (f64, f64),
    width: u32,
    height: u32,
) -> String {
    let (w, h) = if edit.rotate % 180 == 90 {
        (size.1, size.0)
    } else {
        size
    };
    let (tw, th) = edit.turned_size(width, height);
    let scale = edit.straighten_scale(tw, th);
    let sx = if edit.flip_h { -1 } else { 1 };
    let sy = if edit.flip_v { -1 } else { 1 };
    // Applied right to left: turn, flip, grow, straighten, centre.
    format!(
        "width: {w:.2}px; height: {h:.2}px; transform: translate(-50%, -50%) rotate({:.1}deg) scale({scale:.4}) scale({sx}, {sy}) rotate({spin}deg)",
        f64::from(edit.straighten) / 10.0
    )
}

/// Why the edit can't be saved yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Problem {
    NoChange,
    BadTime,
    TooShort,
}

impl From<EditError> for Problem {
    fn from(e: EditError) -> Problem {
        match e {
            EditError::NoChange => Problem::NoChange,
            EditError::TrimTooShort => Problem::TooShort,
            _ => Problem::BadTime,
        }
    }
}

/// "1.5" for 1 500 ms, for the trim fields.
pub fn seconds_text(ms: i64) -> String {
    let tenths = (ms.max(0).saturating_add(50)) / 100;
    format!("{}.{}", tenths / 10, tenths % 10)
}

/// Reads a trim field ("1.5", "1,5", "0:01.5" or "1:02:03") as milliseconds.
pub fn parse_seconds(text: &str) -> Option<i64> {
    let text = text.trim().replace(',', ".");
    if text.is_empty() {
        return None;
    }
    let mut total = 0.0_f64;
    for part in text.split(':') {
        let v: f64 = part.parse().ok()?;
        if !v.is_finite() || v < 0.0 {
            return None;
        }
        total = total * 60.0 + v;
    }
    let ms = (total * 1000.0).round();
    // Longer than a year is not a real video position.
    (ms <= 31_536_000_000.0).then_some(ms as i64)
}

/// The part of a video the trim fields say to keep. The end is never past the
/// video's, and an end the fields can only show as the video's own (its
/// length rounded to a tenth) is the video's end.
pub fn trim_of(start: &str, end: &str, duration: i64) -> Option<Trim> {
    let start_ms = parse_seconds(start)?;
    let end_ms = parse_seconds(end)?.min(duration);
    let end_ms = if duration - end_ms < TRIM_SNAP_MS / 2 {
        duration
    } else {
        end_ms
    };
    Some(Trim { start_ms, end_ms })
}

/// `v` limited to `lo..=hi`; `lo` wins where `i64::clamp` would panic.
fn between(v: i64, lo: i64, hi: i64) -> i64 {
    v.min(hi).max(lo)
}

/// Which end of the clip a timeline handle holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrimEnd {
    Start,
    End,
}

/// Where the handles stand for fields that may not read: a field that doesn't
/// puts its handle at its end of the video.
pub fn trim_shown(start: &str, end: &str, duration: i64) -> (i64, i64) {
    let d = duration.max(0);
    let s = between(parse_seconds(start).unwrap_or(0), 0, d);
    let e = parse_seconds(end).map_or(d, |e| e.min(d));
    let e = if d - e < TRIM_SNAP_MS / 2 { d } else { e };
    (s, e.max(s))
}

/// The clip after `which` end is moved to `ms`: inside the video, at least
/// `MIN_TRIM_MS` long, the other end staying where it is unless it has to.
pub fn moved_trim(start: i64, end: i64, which: TrimEnd, ms: i64, duration: i64) -> (i64, i64) {
    let d = duration.max(0);
    if d < MIN_TRIM_MS {
        return (0, d);
    }
    match which {
        TrimEnd::Start => {
            let end = between(end, MIN_TRIM_MS, d);
            (between(ms, 0, end - MIN_TRIM_MS), end)
        }
        TrimEnd::End => {
            let start = between(start, 0, d - MIN_TRIM_MS);
            (start, between(ms, start + MIN_TRIM_MS, d))
        }
    }
}

/// The time under a pointer at `x` on a timeline from `left`, `width` wide,
/// on tenths of a second like the fields; the last bit of the timeline is the
/// video's very end.
pub fn time_at(x: f64, left: f64, width: f64, duration: i64) -> i64 {
    let d = duration.max(0);
    if width.is_nan() || width <= 0.0 {
        return 0;
    }
    let share = within((x - left) / width, 0.0, 1.0);
    let ms = (share * d as f64).round() as i64;
    if d - ms < TRIM_SNAP_MS / 2 {
        return d;
    }
    ((ms + TRIM_SNAP_MS / 2) / TRIM_SNAP_MS * TRIM_SNAP_MS).min(d)
}

/// Where a handle goes for a key pressed on it, from `now`: arrows a tenth of
/// a second (a second with Shift), Page Up and Down a second, Home and End to
/// the video's ends. `None` for any other key.
pub fn trim_key(key: &str, big: bool, now: i64, duration: i64) -> Option<i64> {
    let step = if big { BIG_TRIM_STEP_MS } else { TRIM_STEP_MS };
    match key {
        "ArrowLeft" | "ArrowDown" => Some(now.saturating_sub(step)),
        "ArrowRight" | "ArrowUp" => Some(now.saturating_add(step)),
        "PageDown" => Some(now.saturating_sub(BIG_TRIM_STEP_MS)),
        "PageUp" => Some(now.saturating_add(BIG_TRIM_STEP_MS)),
        "Home" => Some(0),
        "End" => Some(duration.max(0)),
        _ => None,
    }
}

/// `ms` as a share (0 to 1) of a video `duration` long, for the timeline.
pub fn share(ms: i64, duration: i64) -> f64 {
    if duration <= 0 {
        return 0.0;
    }
    within(ms as f64 / duration as f64, 0.0, 1.0)
}

/// What the editor saves: the image's turns, flips, straightening and crop,
/// or the video's trim from its fields; or why it can't yet.
pub fn request(
    kind: EditKind,
    shape: &Shape,
    (start, end): (&str, &str),
    (width, height): (u32, u32),
    duration_ms: Option<i64>,
) -> Result<MediaEdit, Problem> {
    let e = match kind {
        EditKind::Image => shape.request(width, height),
        EditKind::Video => {
            let duration = duration_ms.ok_or(Problem::BadTime)?;
            MediaEdit {
                trim: Some(trim_of(start, end, duration).ok_or(Problem::BadTime)?),
                ..MediaEdit::default()
            }
        }
    };
    e.check(kind, width, height, duration_ms)
        .map(|()| e)
        .map_err(Problem::from)
}

/// What the editor offers for `p`: an image needs its size, and a video its
/// length and a server that can trim (`video_editing` in `/info`). Nothing
/// for a post in the recycle bin.
pub fn offer(p: &Post, video_editing: bool) -> Option<EditKind> {
    if p.deleted {
        return None;
    }
    edit_kind(p.kind, &p.mime).filter(|k| match k {
        EditKind::Image => p.width.is_some_and(|w| w > 0) && p.height.is_some_and(|h| h > 0),
        EditKind::Video => video_editing && p.duration_ms.is_some_and(|d| d > 0),
    })
}

/// What a key asks of the editor, wherever the focus is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Undo,
    Redo,
    Save,
    /// The save menu if it's open, else the editor.
    Close,
}

/// The command for a key with Ctrl (or ⌘ on a Mac), Shift and Alt as held:
/// Ctrl+Z undoes, Ctrl+Shift+Z and Ctrl+Y redo, Ctrl+S and Ctrl+Enter save,
/// Escape closes.
pub fn command(key: &str, ctrl: bool, shift: bool, alt: bool) -> Option<Command> {
    let key = key.to_lowercase();
    if ctrl && !alt {
        match key.as_str() {
            "z" if shift => Some(Command::Redo),
            "z" => Some(Command::Undo),
            "y" => Some(Command::Redo),
            "s" | "enter" => Some(Command::Save),
            _ => None,
        }
    } else if key == "escape" && !ctrl {
        Some(Command::Close)
    } else {
        None
    }
}

/// Where the tools stand: beside the picture when the window is wider than
/// tall (on the side of the hand set in Settings), or under it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Beside,
    Under,
}

impl Layout {
    /// For a window `width` × `height` CSS pixels.
    pub fn of(width: f64, height: f64) -> Layout {
        if width > height && width >= BESIDE_FROM {
            Layout::Beside
        } else {
            Layout::Under
        }
    }

    /// The value of the editor's `data-layout`.
    pub fn as_str(self) -> &'static str {
        match self {
            Layout::Beside => "beside",
            Layout::Under => "under",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: CropBox, b: CropBox) -> bool {
        [(a.x, b.x), (a.y, b.y), (a.w, b.w), (a.h, b.h)]
            .iter()
            .all(|(p, q)| (p - q).abs() < 1e-9)
    }

    fn b(x: f64, y: f64, w: f64, h: f64) -> CropBox {
        CropBox { x, y, w, h }
    }

    #[test]
    fn limits_never_panic() {
        assert_eq!(within(5.0, 0.0, 1.0), 1.0);
        assert_eq!(within(-5.0, 0.0, 1.0), 0.0);
        // `f64::clamp` panics on both of these.
        assert_eq!(within(0.5, 11.0, 10.0), 11.0);
        assert_eq!(within(f64::NAN, 0.0, 1.0), 0.0);
        let odd = b(f64::NAN, 2.0, -1.0, f64::INFINITY);
        for (handle, _) in Handle::ALL {
            let _ = odd
                .dragged(handle, f64::NAN, 1.0, 10.0, 10.0, None)
                .pixels(10, 10);
            let _ = odd.dragged(handle, 1.0, 1.0, 10.0, 10.0, Some(f64::NAN));
            let _ = odd.dragged(handle, 1.0, 1.0, 0.0, 0.0, Some(2.0));
            let _ = odd.nudged(handle, (1, -1), true, (0.0, f64::NAN), None);
        }
        let _ = odd.pixels(0, 0);
        let _ = odd.with_ratio(0.0, 10.0, 10.0);
        assert_eq!(frame_size(0.0, 0.0, (f64::NAN, 5.0)), (1.0, 1.0));
        assert_eq!(time_at(5.0, 0.0, 0.0, 1_000), 0);
        assert_eq!(time_at(f64::NAN, 0.0, 100.0, 1_000), 0);
        assert_eq!(share(5, 0), 0.0);
        assert_eq!(moved_trim(0, 0, TrimEnd::End, 5, 50), (0, 50));
        assert_eq!(moved_trim(0, 0, TrimEnd::Start, 5, -3), (0, 0));
    }

    #[test]
    fn drags_free_from_any_side() {
        let (fw, fh) = (100.0, 100.0);
        let full = CropBox::FULL;
        let d = |c: CropBox, h, dx, dy| c.dragged(h, dx, dy, fw, fh, None);
        assert!(close(
            d(full, Handle::TopLeft, 25.0, 50.0),
            b(0.25, 0.5, 0.75, 0.5)
        ));
        // Past the edge or the other side, it stops.
        assert!(close(
            d(full, Handle::BottomRight, 50.0, -200.0),
            b(0.0, 0.0, 1.0, MIN_CROP)
        ));
        assert!(close(d(full, Handle::TopRight, 300.0, 0.0), full));
        // A side moves only itself.
        assert!(close(
            d(full, Handle::Left, 30.0, 40.0),
            b(0.3, 0.0, 0.7, 1.0)
        ));
        assert!(close(
            d(full, Handle::Bottom, 30.0, -40.0),
            b(0.0, 0.0, 1.0, 0.6)
        ));
        assert!(close(
            d(full, Handle::Top, 0.0, 10.0),
            b(0.0, 0.1, 1.0, 0.9)
        ));
        assert!(close(
            d(full, Handle::Right, -10.0, 0.0),
            b(0.0, 0.0, 0.9, 1.0)
        ));
        let small = b(0.2, 0.2, 0.5, 0.5);
        assert!(close(
            d(small, Handle::Move, 60.0, -60.0),
            b(0.5, 0.0, 0.5, 0.5)
        ));
        assert!(close(
            d(small, Handle::BottomLeft, -10.0, 10.0),
            b(0.1, 0.2, 0.6, 0.6)
        ));
    }

    #[test]
    fn drags_keep_the_proportions() {
        // A 200 × 100 picture with a 1:1 crop.
        let (fw, fh) = (200.0, 100.0);
        let square = CropBox::FULL.with_ratio(1.0, fw, fh);
        assert!(close(square, b(0.25, 0.0, 0.5, 1.0)));
        let px = |c: CropBox| (c.w * fw, c.h * fh);
        let same = |c: CropBox| {
            let (w, h) = px(c);
            (w - h).abs() < 1e-6
        };
        // Every handle, every way: still square and inside.
        for (handle, _) in Handle::ALL {
            for (dx, dy) in [(-30.0, -10.0), (25.0, 40.0), (-500.0, 500.0), (5.0, -3.0)] {
                let c = square.dragged(handle, dx, dy, fw, fh, Some(1.0));
                assert!(same(c), "{handle:?} {dx} {dy}: {c:?}");
                assert!(c.x >= -1e-9 && c.y >= -1e-9, "{handle:?}: {c:?}");
                assert!(
                    c.x + c.w <= 1.0 + 1e-9 && c.y + c.h <= 1.0 + 1e-9,
                    "{handle:?}: {c:?}"
                );
            }
        }
        // A corner grows the way the pointer went further, from the other corner.
        let near =
            |(w, h): (f64, f64), want: f64| (w - want).abs() < 1e-9 && (h - want).abs() < 1e-9;
        let c = square.dragged(Handle::BottomRight, -20.0, -40.0, fw, fh, Some(1.0));
        assert!(near(px(c), 80.0), "{c:?}");
        assert!((c.x * fw - 50.0).abs() < 1e-9 && c.y == 0.0, "{c:?}");
        // A side keeps the box's middle on the other axis.
        let c = square.dragged(Handle::Left, 40.0, 0.0, fw, fh, Some(1.0));
        assert!(near(px(c), 60.0), "{c:?}");
        assert!((c.y * fh - 20.0).abs() < 1e-9, "{c:?}");
        // Moving keeps the size.
        let c = square.dragged(Handle::Move, 70.0, 0.0, fw, fh, Some(1.0));
        assert!((c.x - 0.5).abs() < 1e-9 && same(c), "{c:?}");
    }

    #[test]
    fn keys_nudge_the_box_and_its_sides() {
        assert_eq!(arrow("ArrowLeft"), Some((-1, 0)));
        assert_eq!(arrow("ArrowDown"), Some((0, 1)));
        assert_eq!(arrow("a"), None);
        // A 400 × 200 picture: a step is 2 pixels, 20 with Shift.
        let frame = (400.0, 200.0);
        let small = b(0.25, 0.25, 0.5, 0.5);
        let c = small.nudged(Handle::Move, (1, 0), false, frame, None);
        assert!(close(c, b(0.255, 0.25, 0.5, 0.5)), "{c:?}");
        let c = small.nudged(Handle::Move, (0, -1), true, frame, None);
        assert!(close(c, b(0.25, 0.15, 0.5, 0.5)), "{c:?}");
        // A side handle moves that side only, and stops at the edge.
        let c = small.nudged(Handle::Right, (1, 0), true, frame, None);
        assert!(close(c, b(0.25, 0.25, 0.55, 0.5)), "{c:?}");
        let c = CropBox::FULL.nudged(Handle::Left, (-1, 0), true, frame, None);
        assert!(close(c, CropBox::FULL), "{c:?}");
        // A tiny picture still moves a whole pixel.
        let c = b(0.0, 0.0, 0.5, 0.5).nudged(Handle::Move, (1, 0), false, (10.0, 10.0), None);
        assert!(close(c, b(0.1, 0.0, 0.5, 0.5)), "{c:?}");
        // With a proportion, it keeps it.
        let square = CropBox::FULL.with_ratio(1.0, 400.0, 200.0);
        let c = square.nudged(Handle::BottomRight, (-1, 0), true, frame, Some(1.0));
        assert!((c.w * 400.0 - 180.0).abs() < 1e-6, "{c:?}");
        assert!((c.h * 200.0 - 180.0).abs() < 1e-6, "{c:?}");
        let c = square.nudged(Handle::TopLeft, (0, 1), false, frame, Some(1.0));
        assert!((c.w * 400.0 - 198.0).abs() < 1e-6, "{c:?}");
    }

    #[test]
    fn proportions_fit_the_picture() {
        // 3:4 in a landscape 400 × 300 frame takes the full height.
        let c = CropBox::FULL.with_ratio(0.75, 400.0, 300.0);
        assert!(
            (c.h - 1.0).abs() < 1e-9 && (c.w * 400.0 - 225.0).abs() < 1e-9,
            "{c:?}"
        );
        // Centred on the old crop, but pushed back inside.
        let c = b(0.8, 0.0, 0.2, 0.2).with_ratio(1.0, 100.0, 100.0);
        assert!(close(c, CropBox::FULL));
        let mut shape = Shape::new(400, 300);
        assert!(shape.wide);
        shape = shape.with_aspect(Aspect::Fixed(4, 3), 400, 300);
        assert!(close(shape.crop, CropBox::FULL));
        assert_eq!(shape.request(400, 300).crop, None);
        // The other way up, then turned: the turn takes the crop along.
        shape = shape.swap_orientation(400, 300);
        assert_eq!(shape.aspect, Aspect::Fixed(3, 4));
        assert!(!shape.wide);
        let turned = shape.turn(true);
        assert_eq!(turned.aspect, Aspect::Fixed(4, 3));
        let (w, h) = turned.frame(400, 300);
        assert_eq!((w, h), (300.0, 400.0));
        let crop = turned.request(400, 300).crop.unwrap();
        assert_eq!((crop.width, crop.height), (300, 225));
        assert_eq!(turned.cropped_size(400, 300), (300, 225));
        assert_eq!(Aspect::Original.ratio(300.0, 400.0), Some(0.75));
        assert!(Aspect::Fixed(16, 9).is_preset() && Aspect::Fixed(9, 16).is_preset());
        assert!(!Aspect::Fixed(5, 7).is_preset());
        assert_eq!(ratio_side(" 21 "), Some(21));
        assert_eq!(ratio_side("0"), None);
        assert_eq!(ratio_side("101"), None);
        assert_eq!(ratio_side("x"), None);
        assert_eq!(oriented((3, 4), true), (4, 3));
        assert_eq!(oriented((3, 4), false), (3, 4));
        assert_eq!(oriented((16, 9), false), (9, 16));
    }

    #[test]
    fn turns_and_flips_take_the_crop_along() {
        let c = b(0.1, 0.2, 0.3, 0.4);
        assert!(close(c.turned(true), b(0.4, 0.1, 0.4, 0.3)));
        assert!(close(c.turned(true).turned(false), c));
        let four = c.turned(true).turned(true).turned(true).turned(true);
        assert!(close(four, c));
        assert!(close(c.mirrored(true), b(0.6, 0.2, 0.3, 0.4)));
        assert!(close(c.mirrored(true).mirrored(true), c));
        assert!(close(c.mirrored(false), b(0.1, 0.4, 0.3, 0.4)));
        // Mirroring leans a straightened picture the other way.
        let mut shape = Shape::new(10, 10);
        shape.edit.straighten = 120;
        let flipped = shape.flip(true);
        assert_eq!(flipped.edit.straighten, -120);
        assert!(flipped.edit.flip_h);
        assert_eq!(flipped.flip(true), shape);
        // A turn after a flip swaps which way it's mirrored, as the server
        // applies the flips after the turn.
        let turned = Shape::new(10, 10).flip(true).turn(true);
        assert!(!turned.edit.flip_h && turned.edit.flip_v);
        assert_eq!(turned.edit.rotate, 90);
    }

    #[test]
    fn straightening_stays_in_tenths_within_the_limit() {
        let shape = Shape::new(10, 10);
        assert_eq!(shape.straightened(1.26).edit.straighten, 13);
        assert_eq!(shape.straightened(-3.0).degrees(), -3.0);
        assert_eq!(shape.straightened(90.0).edit.straighten, MAX_STRAIGHTEN);
        assert_eq!(shape.straightened(-90.0).edit.straighten, -MAX_STRAIGHTEN);
        assert_eq!(
            shape.straightened(f64::NAN).edit.straighten,
            -MAX_STRAIGHTEN
        );
    }

    #[test]
    fn crop_and_turn_make_the_request() {
        // A 400 × 300 picture turned right: the crop is in the turned
        // picture's pixels, 300 wide and 400 tall.
        let shape = Shape::new(400, 300).turn(true);
        let mut cropped = shape.clone();
        cropped.crop = b(0.0, 0.5, 1.0, 0.5);
        let e = cropped.request(400, 300);
        assert_eq!(e.rotate, 90);
        assert_eq!(
            e.crop,
            Some(Crop {
                x: 0,
                y: 200,
                width: 300,
                height: 200
            })
        );
        assert!(e.check(EditKind::Image, 400, 300, None).is_ok());
        assert_eq!(
            request(EditKind::Image, &cropped, ("", ""), (400, 300), None),
            Ok(e)
        );
        // Nothing changed, or turned all the way round: nothing to save.
        let untouched = Shape::new(400, 300);
        assert_eq!(
            request(EditKind::Image, &untouched, ("", ""), (400, 300), None),
            Err(Problem::NoChange)
        );
        let round = untouched.turn(true).turn(true).turn(true).turn(true);
        assert_eq!(
            request(EditKind::Image, &round, ("", ""), (400, 300), None),
            Err(Problem::NoChange)
        );
        // Both flips are a half turn, which is a change.
        let half = untouched.flip(true).flip(false);
        let e = request(EditKind::Image, &half, ("", ""), (400, 300), None).unwrap();
        assert!(e.flip_h && e.flip_v && e.crop.is_none());
        // The original isn't binned by the request itself: saving decides.
        assert!(!e.bin_original);
    }

    #[test]
    fn undo_and_redo() {
        let a = Shape::new(10, 10);
        let b = a.turn(true);
        let c = b.flip(true);
        let mut h = History::default();
        assert!(!h.can_undo() && !h.can_redo());
        h.record(a.clone());
        h.record(b.clone());
        assert!(h.can_undo());
        assert_eq!(h.undo(&c), Some(b.clone()));
        assert!(h.can_redo());
        assert_eq!(h.undo(&b), Some(a.clone()));
        assert_eq!(h.undo(&a), None);
        assert_eq!(h.redo(&a), Some(b.clone()));
        // A new change forgets what was undone.
        h.record(b.clone());
        assert!(!h.can_redo());
        assert_eq!(h.redo(&c), None);
        for _ in 0..MAX_HISTORY + 20 {
            h.record(a.clone());
        }
        assert_eq!(h.past.len(), MAX_HISTORY);
    }

    #[test]
    fn turns_the_short_way() {
        assert_eq!(spun(0, 90), 90);
        assert_eq!(spun(270, 0), 360);
        assert_eq!(spun(360, 270), 270);
        assert_eq!(spun(-90, 180), -180);
        assert_eq!(spun(0, 180), 180);
    }

    #[test]
    fn fits_the_frame_and_previews_turns() {
        assert_eq!(frame_size(400.0, 200.0, (840.0, 840.0)), (800.0, 400.0));
        assert_eq!(frame_size(200.0, 400.0, (840.0, 440.0)), (200.0, 400.0));
        let mut e = MediaEdit::default();
        let style = picture_style(&e, 0, (400.0, 200.0), 400, 200);
        assert!(
            style.starts_with("width: 400.00px; height: 200.00px"),
            "{style}"
        );
        e.turn(true);
        e.flip_h = true;
        e.straighten = -50;
        let style = picture_style(&e, 90, (100.0, 200.0), 400, 200);
        assert!(
            style.starts_with("width: 200.00px; height: 100.00px"),
            "{style}"
        );
        assert!(style.contains("rotate(-5.0deg)"), "{style}");
        assert!(style.ends_with("scale(-1, 1) rotate(90deg)"), "{style}");
        assert_eq!(
            b(0.1, 0.2, 0.3, 0.4).style(),
            "left: 10.000%; top: 20.000%; width: 30.000%; height: 40.000%"
        );
    }

    #[test]
    fn crops_in_pixels() {
        assert_eq!(CropBox::FULL.pixels(40, 30), None);
        assert_eq!(
            b(0.25, 0.0, 0.5, 1.0).pixels(40, 30),
            Some(Crop {
                x: 10,
                y: 0,
                width: 20,
                height: 30
            })
        );
        // Always at least one pixel, and inside the picture.
        let c = b(0.999, 0.999, 0.0001, 0.0001).pixels(10, 10).unwrap();
        assert!(c.width >= 1 && c.x + c.width <= 10, "{c:?}");
        assert!(c.height >= 1 && c.y + c.height <= 10, "{c:?}");
        // What the server accepts.
        let e = MediaEdit {
            crop: Some(c),
            ..MediaEdit::default()
        };
        assert!(e.check(EditKind::Image, 10, 10, None).is_ok());
    }

    #[test]
    fn reads_trim_times() {
        assert_eq!(parse_seconds("1.5"), Some(1_500));
        assert_eq!(parse_seconds(" 1,25 "), Some(1_250));
        assert_eq!(parse_seconds("0:01.5"), Some(1_500));
        assert_eq!(parse_seconds("1:02:03"), Some(3_723_000));
        assert_eq!(parse_seconds(""), None);
        assert_eq!(parse_seconds("-1"), None);
        assert_eq!(parse_seconds("abc"), None);
        assert_eq!(parse_seconds("inf"), None);
        assert_eq!(parse_seconds("1e300"), None);
        assert_eq!(seconds_text(1_500), "1.5");
        assert_eq!(seconds_text(0), "0.0");
        assert_eq!(seconds_text(-40), "0.0");
        assert_eq!(seconds_text(61_049), "61.0");
    }

    #[test]
    fn the_trim_fields_make_the_request() {
        let trim = |s, e| {
            request(
                EditKind::Video,
                &Shape::new(1, 1),
                (s, e),
                (1280, 720),
                Some(6_000),
            )
        };
        assert_eq!(
            trim("1.0", "3.0").map(|e| e.trim),
            Ok(Some(Trim {
                start_ms: 1_000,
                end_ms: 3_000
            }))
        );
        // The whole video is no change; past the end is the end.
        assert_eq!(trim("0", "6.0"), Err(Problem::NoChange));
        assert_eq!(trim("0", "99"), Err(Problem::NoChange));
        assert_eq!(trim("2", "1"), Err(Problem::BadTime));
        assert_eq!(trim("x", "1"), Err(Problem::BadTime));
        assert_eq!(trim("1.0", "1.05"), Err(Problem::TooShort));
        // Without a length there's nothing to trim against.
        assert_eq!(
            request(EditKind::Video, &Shape::new(1, 1), ("1", "2"), (1, 1), None),
            Err(Problem::BadTime)
        );
        // A length the fields can't show to the millisecond: its own text is
        // still the end, so opening the editor is no change.
        let odd = 6_033;
        assert_eq!(seconds_text(odd), "6.0");
        assert_eq!(
            trim_of("0.0", &seconds_text(odd), odd).map(|t| t.end_ms),
            Some(odd)
        );
        assert_eq!(trim_of("0.0", "5.9", odd).map(|t| t.end_ms), Some(5_900));
        assert_eq!(trim_shown("x", "y", odd), (0, odd));
        assert_eq!(trim_shown("2", "1", odd), (2_000, 2_000));
        assert_eq!(trim_shown("1", "6.0", odd), (1_000, odd));
    }

    #[test]
    fn trim_handles_stay_in_the_video_and_apart() {
        let d = 6_000;
        // The start can't pass the end, less the shortest clip.
        assert_eq!(
            moved_trim(0, 3_000, TrimEnd::Start, 2_500, d),
            (2_500, 3_000)
        );
        assert_eq!(
            moved_trim(0, 3_000, TrimEnd::Start, 5_000, d),
            (2_900, 3_000)
        );
        assert_eq!(
            moved_trim(1_000, 3_000, TrimEnd::Start, -400, d),
            (0, 3_000)
        );
        // Nor the end the start.
        assert_eq!(
            moved_trim(1_000, 3_000, TrimEnd::End, 500, d),
            (1_000, 1_100)
        );
        assert_eq!(moved_trim(1_000, 3_000, TrimEnd::End, 9_000, d), (1_000, d));
        // The other end, when out of the video, is brought back in.
        assert_eq!(moved_trim(9_000, 9_500, TrimEnd::End, 9_900, d), (5_900, d));
        assert_eq!(moved_trim(0, 9_500, TrimEnd::Start, 100, d), (100, d));
        // A video shorter than the shortest clip can't be trimmed.
        assert_eq!(moved_trim(10, 40, TrimEnd::Start, 20, 80), (0, 80));
    }

    #[test]
    fn the_timeline_reads_the_pointer_and_the_keys() {
        // 600 pixels for 6 seconds: a pixel is 10 ms, on tenths.
        assert_eq!(time_at(100.0, 100.0, 600.0, 6_000), 0);
        assert_eq!(time_at(203.0, 100.0, 600.0, 6_000), 1_000);
        assert_eq!(time_at(206.0, 100.0, 600.0, 6_000), 1_100);
        assert_eq!(time_at(50.0, 100.0, 600.0, 6_000), 0);
        assert_eq!(time_at(900.0, 100.0, 600.0, 6_000), 6_000);
        // The last bit is the very end, even when that isn't on a tenth.
        assert_eq!(time_at(699.0, 100.0, 600.0, 6_033), 6_033);
        assert_eq!(time_at(690.0, 100.0, 600.0, 6_033), 5_900);
        assert_eq!(trim_key("ArrowRight", false, 1_000, 6_000), Some(1_100));
        assert_eq!(trim_key("ArrowLeft", true, 1_000, 6_000), Some(0));
        assert_eq!(trim_key("PageUp", false, 1_000, 6_000), Some(2_000));
        assert_eq!(trim_key("Home", false, 1_000, 6_000), Some(0));
        assert_eq!(trim_key("End", false, 1_000, 6_000), Some(6_000));
        assert_eq!(trim_key("x", false, 1_000, 6_000), None);
        assert_eq!(share(1_500, 6_000), 0.25);
        assert_eq!(share(9_000, 6_000), 1.0);
        assert_eq!(share(-9, 6_000), 0.0);
    }

    #[test]
    fn the_editor_is_offered_where_the_server_can_edit() {
        let post = |kind: &str, mime: &str| -> Post {
            serde_json::from_value(serde_json::json!({
                "id": 5, "kind": kind, "mime": mime, "file_size": 1,
                "width": 1280, "height": 720, "duration_ms": 6000, "sha256": "ab",
                "rating": "safe", "score": 0, "my_vote": 0, "source": null, "tags": [],
                "created_at": "2026-01-01T00:00:00Z", "deleted": false,
                "file_url": "/media/original/ab.png?exp=1&sig=2",
            }))
            .unwrap()
        };
        assert_eq!(
            offer(&post("image", "image/png"), false),
            Some(EditKind::Image)
        );
        assert_eq!(offer(&post("image", "image/avif"), true), None);
        assert_eq!(offer(&post("animation", "image/gif"), true), None);
        let video = post("video", "video/mp4");
        assert_eq!(offer(&video, true), Some(EditKind::Video));
        // Trimming needs ffmpeg on the server, and the video's length.
        assert_eq!(offer(&video, false), None);
        let mut endless = video.clone();
        endless.duration_ms = None;
        assert_eq!(offer(&endless, true), None);
        let mut sizeless = post("image", "image/jpeg");
        sizeless.height = None;
        assert_eq!(offer(&sizeless, true), None);
        let mut binned = post("image", "image/png");
        binned.deleted = true;
        assert_eq!(offer(&binned, true), None);
    }

    #[test]
    fn keys_undo_redo_save_and_close() {
        let k = |key, ctrl, shift| command(key, ctrl, shift, false);
        assert_eq!(k("z", true, false), Some(Command::Undo));
        // Shift makes the letter a capital in some browsers.
        assert_eq!(k("Z", true, true), Some(Command::Redo));
        assert_eq!(k("y", true, false), Some(Command::Redo));
        assert_eq!(k("s", true, false), Some(Command::Save));
        assert_eq!(k("Enter", true, false), Some(Command::Save));
        assert_eq!(k("Escape", false, false), Some(Command::Close));
        assert_eq!(k("z", false, false), None);
        assert_eq!(k("s", false, false), None);
        assert_eq!(k("Enter", false, false), None);
        // AltGr is Ctrl+Alt on Windows, for typing.
        assert_eq!(command("z", true, false, true), None);
    }

    #[test]
    fn the_tools_stand_beside_wide_windows_and_under_tall_ones() {
        assert_eq!(Layout::of(1280.0, 800.0), Layout::Beside);
        assert_eq!(Layout::of(390.0, 844.0), Layout::Under);
        // A phone on its side is wide enough.
        assert_eq!(Layout::of(844.0, 390.0), Layout::Beside);
        // A small window, or a square one, keeps them under.
        assert_eq!(Layout::of(600.0, 400.0), Layout::Under);
        assert_eq!(Layout::of(900.0, 900.0), Layout::Under);
        assert_eq!(Layout::of(f64::NAN, 400.0), Layout::Under);
        assert_eq!(Layout::Beside.as_str(), "beside");
        assert_eq!(Layout::Under.as_str(), "under");
    }
}
