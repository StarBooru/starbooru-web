//! Filled, rounded icons as inline SVG on a 24 by 24 grid, from the reference
//! app (sources in ATTRIBUTION.md). A new icon is a variant and its path data:
//! take it from the reference's `icons.rs` when it has one.

use leptos::prelude::*;

#[allow(dead_code, reason = "the pages still to come use the rest")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Add,
    ArrowBack,
    Check,
    ChevronLeft,
    ChevronRight,
    Close,
    Delete,
    Download,
    Edit,
    Explore,
    Gallery,
    Help,
    /// A tag.
    Label,
    Logout,
    Restore,
    Search,
    Share,
    Star,
    StarOutline,
    ThumbDown,
    ThumbUp,
    /// Settings.
    Tune,
    Upload,
}

/// How an icon's path data is painted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    Fill(&'static str),
    EvenOdd(&'static str),
    /// Round-ended strokes of this width.
    Stroke(&'static str, &'static str),
}

impl Icon {
    fn shape(self) -> Shape {
        match self {
            Icon::Add => Shape::EvenOdd(
                "M12 3.5A1.5 1.5 0 0 1 13.5 5v5.5H19a1.5 1.5 0 0 1 0 3h-5.5V19a1.5 1.5 0 0 1-3 0v-5.5H5a1.5 1.5 0 0 1 0-3h5.5V5A1.5 1.5 0 0 1 12 3.5",
            ),
            Icon::ArrowBack => Shape::Stroke("3", "M20 12H5m6.657-6.657L5 12l6.657 6.657"),
            Icon::Check => Shape::EvenOdd(
                "M21.546 5.112a1.5 1.5 0 0 1 0 2.121L10.232 18.547a1.5 1.5 0 0 1-2.121 0L2.454 12.89a1.5 1.5 0 1 1 2.121-2.121l4.596 4.596L19.424 5.112a1.5 1.5 0 0 1 2.122 0",
            ),
            Icon::ChevronLeft => Shape::Stroke("3", "M14.657 17.657L9 12l5.657-5.657"),
            Icon::ChevronRight => Shape::Stroke("3", "M9.343 6.343L15 12l-5.657 5.657"),
            Icon::Close => Shape::EvenOdd(
                "M4.575 4.576a1.5 1.5 0 0 1 2.122 0L12 9.88l5.303-5.303a1.5 1.5 0 0 1 2.122 2.121l-5.304 5.304l5.304 5.303a1.5 1.5 0 1 1-2.122 2.121L12 14.122l-5.303 5.303a1.5 1.5 0 1 1-2.122-2.12L9.88 12L4.575 6.697a1.5 1.5 0 0 1 0-2.12",
            ),
            Icon::Delete => Shape::EvenOdd(
                "M7.823 3.368A2 2 0 0 1 9.721 2h4.558a2 2 0 0 1 1.898 1.368L16.72 5H20a1 1 0 1 1 0 2h-.064l-.814 12.2A3 3 0 0 1 16.13 22H7.87a3 3 0 0 1-2.993-2.8L4.064 7H4a1 1 0 0 1 0-2h3.28zM14.613 5H9.387l.334-1h4.558zM9 10a1 1 0 0 1 1 1v6a1 1 0 1 1-2 0v-6a1 1 0 0 1 1-1m6 0a1 1 0 0 1 1 1v6a1 1 0 1 1-2 0v-6a1 1 0 0 1 1-1",
            ),
            Icon::Download => Shape::Stroke(
                "3",
                "M20 16v4a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1v-4m8-13v12m-4.243-3.586L12 15.657l4.243-4.243",
            ),
            Icon::Edit => Shape::Fill(
                "M11.4001 18.1612L11.4001 18.1612L18.796 10.7653C17.7894 10.3464 16.5972 9.6582 15.4697 8.53068C14.342 7.40298 13.6537 6.21058 13.2348 5.2039L5.83882 12.5999L5.83879 12.5999C5.26166 13.1771 4.97307 13.4657 4.7249 13.7838C4.43213 14.1592 4.18114 14.5653 3.97634 14.995C3.80273 15.3593 3.67368 15.7465 3.41556 16.5208L2.05445 20.6042C1.92743 20.9852 2.0266 21.4053 2.31063 21.6894C2.59466 21.9734 3.01478 22.0726 3.39584 21.9456L7.47918 20.5844C8.25351 20.3263 8.6407 20.1973 9.00498 20.0237C9.43469 19.8189 9.84082 19.5679 10.2162 19.2751C10.5343 19.0269 10.823 18.7383 11.4001 18.1612ZM20.8482 8.71306C22.3839 7.17735 22.3839 4.68748 20.8482 3.15178C19.3125 1.61607 16.8226 1.61607 15.2869 3.15178L14.3999 4.03882C14.4121 4.0755 14.4246 4.11268 14.4377 4.15035C14.7628 5.0875 15.3763 6.31601 16.5303 7.47002C17.6843 8.62403 18.9128 9.23749 19.85 9.56262C19.8875 9.57563 19.9245 9.58817 19.961 9.60026L20.8482 8.71306Z",
            ),
            Icon::Explore => Shape::Fill(
                "M12 2C6.477 2 2 6.477 2 12s4.477 10 10 10s10-4.477 10-10S17.523 2 12 2M9.879 9.879c1.414-1.414 6.01-2.475 6.364-2.121c.352.357-.708 4.95-2.122 6.363s-6.01 2.475-6.363 2.121s.707-4.949 2.12-6.363M12 11a1 1 0 1 0 0 2a1 1 0 0 0 0-2",
            ),
            Icon::Gallery => Shape::EvenOdd(
                "M5 3a3 3 0 0 0-3 3v10a2 2 0 0 0 2 2V6a1 1 0 0 1 1-1h14a2 2 0 0 0-2-2zm0 5a2 2 0 0 1 2-2h13a2 2 0 0 1 2 2v11.333a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2zm15 0H7v8.848L10.848 13a1.25 1.25 0 0 1 1.768 0l3.241 3.24l.884-.883a1.25 1.25 0 0 1 1.768 0L20 16.848zm-2 3a1.5 1.5 0 1 1-3 0a1.5 1.5 0 0 1 3 0",
            ),
            Icon::Help => Shape::Fill(
                "M12 2c5.523 0 10 4.477 10 10s-4.477 10-10 10S2 17.523 2 12S6.477 2 12 2m0 14a1 1 0 1 0 0 2h.002a1 1 0 0 0 0-2zm0-9.5a3.625 3.625 0 0 0-3.625 3.625a1 1 0 1 0 2 0a1.625 1.625 0 1 1 2.23 1.509c-.675.27-1.605.963-1.605 2.116V14a1 1 0 1 0 2 0v-.246a.2.2 0 0 1 .043-.063a.8.8 0 0 1 .306-.2A3.626 3.626 0 0 0 12 6.5",
            ),
            Icon::Label => Shape::Fill(
                "M10.537 2.164a3 3 0 0 1 2.393.866l7.823 7.824a3 3 0 0 1 0 4.242l-5.657 5.657a3 3 0 0 1-4.242 0L3.03 12.93a3 3 0 0 1-.866-2.393l.472-5.185a3 3 0 0 1 2.716-2.716zm.316 5.862a2 2 0 1 0-2.828 2.827a2 2 0 0 0 2.828-2.827",
            ),
            Icon::Logout => Shape::EvenOdd(
                "M3.5 6A3.5 3.5 0 0 1 7 2.5h5a1.5 1.5 0 0 1 0 3H7a.5.5 0 0 0-.5.5v12a.5.5 0 0 0 .5.5h5a1.5 1.5 0 0 1 0 3H7A3.5 3.5 0 0 1 3.5 18zm12.44 2.11a1.5 1.5 0 0 1 2.12 0l2.829 2.83a1.5 1.5 0 0 1 0 2.12l-2.828 2.83a1.5 1.5 0 1 1-2.122-2.122l.268-.268H12a1.5 1.5 0 0 1 0-3h4.207l-.268-.268a1.5 1.5 0 0 1 0-2.121",
            ),
            Icon::Restore => Shape::EvenOdd(
                "M13.941 19.244a7.5 7.5 0 0 1-7.534-2.245A1.5 1.5 0 1 0 4.172 19a10.5 10.5 0 0 0 10.545 3.142c5.602-1.501 8.926-7.259 7.425-12.86c-1.5-5.602-7.258-8.926-12.86-7.425a10.5 10.5 0 0 0-7.784 10.06c-.01 1.314 1.449 2.04 2.492 1.337l2.674-1.801c1.466-.988.483-3.274-1.243-2.889l-.1.022a7.5 7.5 0 1 1 8.62 10.658",
            ),
            Icon::Search => Shape::Fill(
                "M10.5 2a8.5 8.5 0 0 1 8.5 8.5a8.46 8.46 0 0 1-1.826 5.26l3.654 3.654a1.001 1.001 0 0 1-1.414 1.415l-3.654-3.655A8.46 8.46 0 0 1 10.5 19a8.5 8.5 0 0 1 0-17m0 2a6.5 6.5 0 1 0 0 13a6.5 6.5 0 0 0 0-13zM5.75 10.5A4.75 4.75 0 0 1 10.5 5.75a.75.75 0 0 1 0 1.5A3.25 3.25 0 0 0 7.25 10.5a.75.75 0 0 1-1.5 0z",
            ),
            Icon::Share => Shape::Fill(
                "M5.74 11.28L16.24 5.78L17.26 7.72L6.76 13.22zM6.87 11.34L14.99 16.84L13.76 18.66L5.63 13.16zM12 6.75a4.75 4.75 0 1 1 9.5 0a4.75 4.75 0 1 1 -9.5 0zM2.5 12.25a3.75 3.75 0 1 1 7.5 0a3.75 3.75 0 1 1 -7.5 0zM11.125 17.75a3.25 3.25 0 1 1 6.5 0a3.25 3.25 0 1 1 -6.5 0z",
            ),
            Icon::Star => Shape::Fill(
                "M10.92 2.37a1.25 1.25 0 0 1 2.16 0l2.795 4.8l5.428 1.175a1.25 1.25 0 0 1 .667 2.054l-3.7 4.142l.56 5.525a1.25 1.25 0 0 1-1.748 1.27L12 19.096l-5.082 2.24a1.25 1.25 0 0 1-1.747-1.27l.559-5.525l-3.7-4.142a1.25 1.25 0 0 1 .667-2.054L8.125 7.17z",
            ),
            Icon::StarOutline => Shape::Stroke(
                "2",
                "M11.784 2.87a.25.25 0 0 1 .432 0l2.961 5.085a.25.25 0 0 0 .164.119l5.75 1.245a.25.25 0 0 1 .134.41l-3.92 4.388a.25.25 0 0 0-.063.192l.593 5.854a.25.25 0 0 1-.35.254l-5.384-2.373a.25.25 0 0 0-.202 0l-5.384 2.373a.25.25 0 0 1-.35-.254l.593-5.854a.25.25 0 0 0-.062-.192L2.776 9.73a.25.25 0 0 1 .133-.411l5.75-1.245a.25.25 0 0 0 .164-.119z",
            ),
            Icon::ThumbDown => Shape::Fill(
                "M3 16q-.8 0-1.4-.6T1 14v-2q0-.175.037-.375t.113-.375l3-7.05q.225-.5.75-.85T6 3h8q.825 0 1.413.587T16 5v10.175q0 .4-.162.763t-.438.637l-5.425 5.4q-.375.35-.887.425t-.988-.175t-.687-.7t-.088-.925L8.45 16zM20 3q.825 0 1.413.588T22 5v9q0 .825-.587 1.413T20 16t-1.412-.587T18 14V5q0-.825.588-1.412T20 3",
            ),
            Icon::ThumbUp => Shape::Fill(
                "M21 8q.8 0 1.4.6T23 10v2q0 .175-.038.375t-.112.375l-3 7.05q-.225.5-.75.85T18 21h-8q-.825 0-1.412-.587T8 19V8.825q0-.4.163-.762t.437-.638l5.425-5.4q.375-.35.888-.425t.987.175t.688.7t.087.925L15.55 8zM4 21q-.825 0-1.412-.587T2 19v-9q0-.825.588-1.412T4 8t1.413.588T6 10v9q0 .825-.587 1.413T4 21",
            ),
            Icon::Tune => Shape::Fill(
                "M11.288 20.713Q11 20.425 11 20v-4q0-.425.288-.712T12 15t.713.288T13 16v1h7q.425 0 .713.288T21 18t-.288.713T20 19h-7v1q0 .425-.288.713T12 21t-.712-.288M4 19q-.425 0-.712-.288T3 18t.288-.712T4 17h4q.425 0 .713.288T9 18t-.288.713T8 19zm3.288-4.288Q7 14.425 7 14v-1H4q-.425 0-.712-.288T3 12t.288-.712T4 11h3v-1q0-.425.288-.712T8 9t.713.288T9 10v4q0 .425-.288.713T8 15t-.712-.288M12 13q-.425 0-.712-.288T11 12t.288-.712T12 11h8q.425 0 .713.288T21 12t-.288.713T20 13zm3.288-4.288Q15 8.425 15 8V4q0-.425.288-.712T16 3t.713.288T17 4v1h3q.425 0 .713.288T21 6t-.288.713T20 7h-3v1q0 .425-.288.713T16 9t-.712-.288M4 7q-.425 0-.712-.288T3 6t.288-.712T4 5h8q.425 0 .713.288T13 6t-.288.713T12 7z",
            ),
            Icon::Upload => {
                Shape::Stroke("3", "M5 3.5h14M12 20.5V9.5M7.757 13.743 12 9.5l4.243 4.243")
            }
        }
    }
}

/// An icon, `size` pixels square, in the current colour. Decorative: the
/// button or link it sits in carries the words.
#[component]
pub fn Ico(icon: Icon, #[prop(default = 24)] size: u32) -> impl IntoView {
    let path = match icon.shape() {
        Shape::Fill(d) => view! { <path d=d /> }.into_any(),
        Shape::EvenOdd(d) => view! { <path fill-rule="evenodd" d=d /> }.into_any(),
        Shape::Stroke(width, d) => view! {
            <path
                fill="none"
                stroke="currentColor"
                stroke-width=width
                stroke-linecap="round"
                stroke-linejoin="round"
                d=d
            />
        }
        .into_any(),
    };
    view! {
        <svg
            class="icon"
            width=size
            height=size
            viewBox="0 0 24 24"
            fill="currentColor"
            aria-hidden="true"
        >
            {path}
        </svg>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Icon; 23] = [
        Icon::Add,
        Icon::ArrowBack,
        Icon::Check,
        Icon::ChevronLeft,
        Icon::ChevronRight,
        Icon::Close,
        Icon::Delete,
        Icon::Download,
        Icon::Edit,
        Icon::Explore,
        Icon::Gallery,
        Icon::Help,
        Icon::Label,
        Icon::Logout,
        Icon::Restore,
        Icon::Search,
        Icon::Share,
        Icon::Star,
        Icon::StarOutline,
        Icon::ThumbDown,
        Icon::ThumbUp,
        Icon::Tune,
        Icon::Upload,
    ];

    #[test]
    fn every_icon_has_well_formed_path_data() {
        for icon in ALL {
            let d = match icon.shape() {
                Shape::Fill(d) | Shape::EvenOdd(d) | Shape::Stroke(_, d) => d,
            };
            assert!(d.starts_with(['M', 'm']), "{icon:?} must start with a move");
            assert!(
                d.chars()
                    .all(|c| c.is_ascii_alphanumeric() || " .,-".contains(c)),
                "{icon:?} has a character that is not path data",
            );
        }
    }
}
