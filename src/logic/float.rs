//! Where a list that floats by a field goes, such as a tag box's offers:
//! under the field, or over it when there's more room there and not enough
//! under, as wide as the field and kept on the screen.

/// Room around the list: from the field, and from the screen's edges.
pub const GAP: f64 = 6.0;
pub const EDGE: f64 = 8.0;
/// The least a list is cut down to, about one row of chips and a half.
const LEAST: f64 = 48.0;

/// A rectangle on the screen, in CSS pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn bottom(&self) -> f64 {
        self.top + self.height
    }
}

/// Where the list goes, for `position: fixed`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Float {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub max_height: f64,
    /// Over the field rather than under it.
    pub above: bool,
}

/// The list for `field` in the part of the screen in view, `view` (the
/// keyboard on a phone takes some), for a list `natural` pixels tall with
/// nothing cut.
pub fn float_by(field: Rect, view: Rect, natural: f64) -> Float {
    let natural = if natural.is_finite() {
        natural.max(0.0)
    } else {
        0.0
    };
    let below = view.bottom() - field.bottom() - GAP - EDGE;
    let over = field.top - view.top - GAP - EDGE;
    let above = below < natural && over > below;
    let room = if above { over } else { below };
    let max_height = room.max(LEAST);
    let tall = natural.min(max_height);
    let top = if above {
        field.top - GAP - tall
    } else {
        field.bottom() + GAP
    };
    let width = field.width.min(view.width - 2.0 * EDGE).max(0.0);
    let left = field
        .left
        .min(view.left + view.width - EDGE - width)
        .max(view.left + EDGE);
    Float {
        left,
        top,
        width,
        max_height,
        above,
    }
}

/// `view` less the bars fixed over it, `bars`, that are across from `field`
/// (the top bar, and on a phone the navigation at the bottom): one in the
/// top half takes the view's top down to its bottom, one in the lower half
/// its bottom up to its top.
pub fn clear_of(view: Rect, field: Rect, bars: &[Rect]) -> Rect {
    let middle = view.top + view.height / 2.0;
    let (mut top, mut bottom) = (view.top, view.bottom());
    for bar in bars {
        let across = bar.width > 0.0
            && bar.height > 0.0
            && bar.left < field.left + field.width
            && bar.left + bar.width > field.left;
        if !across {
            continue;
        }
        if bar.top + bar.height / 2.0 < middle {
            top = top.max(bar.bottom());
        } else {
            bottom = bottom.min(bar.top);
        }
    }
    Rect {
        top,
        height: (bottom - top).max(0.0),
        ..view
    }
}

/// Whether any of `field` is in `view`.
pub fn in_sight(field: Rect, view: Rect) -> bool {
    field.bottom() > view.top
        && field.top < view.bottom()
        && field.left + field.width > view.left
        && field.left < view.left + view.width
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEW: Rect = Rect {
        left: 0.0,
        top: 0.0,
        width: 390.0,
        height: 844.0,
    };

    fn field(top: f64) -> Rect {
        Rect {
            left: 28.0,
            top,
            width: 286.0,
            height: 44.0,
        }
    }

    #[test]
    fn it_goes_under_the_field_as_wide_as_it() {
        let f = float_by(field(300.0), VIEW, 120.0);
        assert!(!f.above);
        assert_eq!((f.left, f.width), (28.0, 286.0));
        assert_eq!(f.top, 344.0 + GAP);
        // All the room down to the edge, for a list that grows.
        assert_eq!(f.max_height, 844.0 - 344.0 - GAP - EDGE);
    }

    #[test]
    fn near_the_bottom_it_goes_over_the_field() {
        let f = float_by(field(760.0), VIEW, 120.0);
        assert!(f.above);
        assert_eq!(f.top, 760.0 - GAP - 120.0);
        assert_eq!(f.max_height, 760.0 - GAP - EDGE);
        // Short enough for the room under it, it stays under.
        assert!(!float_by(field(760.0), VIEW, 20.0).above);
    }

    #[test]
    fn a_tall_list_takes_the_bigger_side_and_scrolls() {
        // 456 under, 368 over: under, cut to the room there.
        let f = float_by(field(380.0), VIEW, 900.0);
        assert!(!f.above);
        assert_eq!(f.max_height, 844.0 - 424.0 - GAP - EDGE);
        let f = float_by(field(500.0), VIEW, 900.0);
        assert!(f.above);
        assert_eq!(f.top, 500.0 - GAP - f.max_height);
        assert!(f.top >= EDGE);
    }

    #[test]
    fn the_bars_across_from_the_field_take_their_room() {
        let top_bar = Rect {
            left: 0.0,
            top: 0.0,
            width: 390.0,
            height: 64.0,
        };
        let nav = Rect {
            left: 12.0,
            top: 772.0,
            width: 366.0,
            height: 60.0,
        };
        let view = clear_of(VIEW, field(300.0), &[top_bar, nav]);
        assert_eq!((view.top, view.bottom()), (64.0, 772.0));
        // A rail at the side, beside the field rather than across from it,
        // and a bar not shown, take nothing.
        let rail = Rect {
            left: 12.0,
            top: 240.0,
            width: 76.0,
            height: 380.0,
        };
        let hidden = Rect::default();
        let wide = Rect {
            left: 932.0,
            ..field(300.0)
        };
        let desk = Rect {
            width: 1280.0,
            height: 800.0,
            ..VIEW
        };
        assert_eq!(clear_of(desk, wide, &[rail, hidden]), desk);
        // Near the bottom of a phone, the list goes over the field rather
        // than under the navigation.
        let near = field(600.0);
        assert!(float_by(near, clear_of(VIEW, near, &[top_bar, nav]), 156.0).above);
        assert!(!float_by(near, VIEW, 156.0).above);
    }

    #[test]
    fn a_field_scrolled_away_is_out_of_sight() {
        assert!(in_sight(field(300.0), VIEW));
        assert!(in_sight(field(-40.0), VIEW));
        assert!(!in_sight(field(-44.0), VIEW));
        assert!(!in_sight(field(844.0), VIEW));
        assert!(in_sight(field(843.0), VIEW));
    }

    #[test]
    fn the_keyboard_and_the_edges_keep_it_in_view() {
        // The keyboard leaves the top 400 pixels.
        let view = Rect {
            height: 400.0,
            ..VIEW
        };
        let f = float_by(field(300.0), view, 120.0);
        assert!(f.above);
        assert!(f.top >= EDGE && f.top + 120.0 <= 300.0);
        // A field wider than the screen is cut to it.
        let wide = Rect {
            left: -20.0,
            width: 500.0,
            ..field(100.0)
        };
        let f = float_by(wide, VIEW, 50.0);
        assert_eq!((f.left, f.width), (EDGE, 390.0 - 2.0 * EDGE));
        // No room anywhere, or nothing measured: still a little, under it.
        let tiny = Rect {
            height: 60.0,
            ..VIEW
        };
        for natural in [120.0, f64::NAN] {
            let f = float_by(field(0.0), tiny, natural);
            assert!(!f.above);
            assert_eq!(f.max_height, LEAST);
        }
    }
}
