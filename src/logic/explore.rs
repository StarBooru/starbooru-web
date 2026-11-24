//! Explore's rules that need no browser: the window it asks for, the
//! windows a public server's Trending offers and how they read, the searches
//! its sections lead to, what a shuffle searches, and the years of the posts
//! added on this day.

use chrono::{DateTime, Datelike, Utc};
use starbooru_core::{DEFAULT_EXPLORE_DAYS, MAX_EXPLORE_DAYS, Order, with_order};

use super::tags::include;

/// The window a private server is asked for, in days. Only Trending and
/// Popular tags read it, and a private server leaves both empty, so the
/// shortest window leaves it the least to count.
pub const DAYS: u32 = 1;

/// The windows a public server's Trending offers, in days.
pub const WINDOWS: [u32; 4] = [1, 7, 30, 365];

/// The window in an address's `days`: one the server takes, or a week.
pub fn window(days: Option<&str>) -> u32 {
    days.and_then(|d| d.trim().parse::<u32>().ok())
        .filter(|d| (1..=MAX_EXPLORE_DAYS).contains(d))
        .unwrap_or(DEFAULT_EXPLORE_DAYS)
}

/// The window to ask for: the address's on a public server, else the
/// shortest.
pub fn days_asked(public: bool, days: Option<&str>) -> u32 {
    if public { window(days) } else { DAYS }
}

/// How a window reads: "today", "this week", … or so many days.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Span {
    Today,
    Week,
    Month,
    Year,
    Days(u32),
}

pub fn span(days: u32) -> Span {
    match days {
        1 => Span::Today,
        7 => Span::Week,
        30 => Span::Month,
        365 => Span::Year,
        n => Span::Days(n),
    }
}

/// The gallery search a popular tag leads to: Explore's search narrowed to
/// it.
pub fn tag_search(q: &str, tag: &str) -> String {
    include(&tidy(q), tag)
}

/// Random picks a shuffle asks for: as many as the server sends at first.
pub const RANDOM_COUNT: u32 = 12;

/// A search as Explore reads and sends it, the spaces in it collapsed.
pub fn tidy(q: &str) -> String {
    q.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// What a shuffle of the random picks searches: Explore's search, in random
/// order.
pub fn random_query(q: &str) -> String {
    with_order(&tidy(q), Order::parse("random"))
}

/// The year a post was added in, as the server counts "on this day": in UTC,
/// so a post added late on New Year's Eve west of Greenwich keeps its year.
pub fn year_added(at: &DateTime<Utc>) -> i32 {
    at.year()
}

/// Whole years from `at` to `now`.
pub fn years_ago(at: &DateTime<Utc>, now: &DateTime<Utc>) -> i32 {
    let (then, today) = (at.date_naive(), now.date_naive());
    let mut years = today.year() - then.year();
    if (today.month(), today.day()) < (then.month(), then.day()) {
        years -= 1;
    }
    years.max(0)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use starbooru_core::MAX_EXPLORE_DAYS;

    use super::*;

    fn utc(y: i32, m: u32, d: u32, h: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, 0, 0).single().unwrap()
    }

    #[test]
    fn a_private_server_is_asked_for_the_shortest_window() {
        assert_eq!(DAYS, 1);
        assert!((1..=MAX_EXPLORE_DAYS).contains(&DAYS));
        assert_eq!(days_asked(false, Some("30")), DAYS);
        assert_eq!(days_asked(true, Some("30")), 30);
        assert_eq!(days_asked(true, None), DEFAULT_EXPLORE_DAYS);
    }

    #[test]
    fn the_window_comes_from_the_address_or_is_a_week() {
        assert_eq!(window(None), 7);
        assert_eq!(window(Some("30")), 30);
        assert_eq!(window(Some(" 365 ")), 365);
        for bad in ["0", "366", "-1", "week", ""] {
            assert_eq!(window(Some(bad)), DEFAULT_EXPLORE_DAYS, "{bad}");
        }
        for days in WINDOWS {
            assert_eq!(window(Some(&days.to_string())), days);
        }
    }

    #[test]
    fn each_window_reads_as_a_span() {
        assert_eq!(span(1), Span::Today);
        assert_eq!(span(7), Span::Week);
        assert_eq!(span(30), Span::Month);
        assert_eq!(span(365), Span::Year);
        assert_eq!(span(14), Span::Days(14));
    }

    #[test]
    fn a_popular_tag_searches_within_explores_search() {
        assert_eq!(tag_search("", "fractal"), "fractal");
        assert_eq!(
            tag_search(" rating:safe ", "fractal"),
            "rating:safe fractal"
        );
        // A tag left out by the search is wanted instead, not both.
        assert_eq!(tag_search("cat -fractal", "fractal"), "cat fractal");
    }

    #[test]
    fn a_shuffle_searches_within_explores_search() {
        assert_eq!(random_query(""), "order:random");
        assert_eq!(
            random_query("  fractal  -abstract "),
            "fractal -abstract order:random"
        );
        // Its own order gives way to the shuffle.
        assert_eq!(random_query("cat order:score"), "cat order:random");
        assert_eq!(tidy("  a   b "), "a b");
    }

    #[test]
    fn a_memory_is_dated_as_the_server_counts_this_day_in_utc() {
        let now = utc(2026, 11, 23, 3);
        // Late on the 22nd in New York is already the 23rd in UTC, so the
        // server lists it today, and its year is the UTC one.
        let new_york = chrono::FixedOffset::west_opt(5 * 3600).unwrap();
        let evening = new_york
            .with_ymd_and_hms(2023, 11, 22, 21, 0, 0)
            .single()
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!((evening.month(), evening.day()), (now.month(), now.day()));
        assert_eq!(year_added(&evening), 2023);
        assert_eq!(years_ago(&evening, &now), 3);
        let eve = new_york
            .with_ymd_and_hms(2024, 12, 31, 22, 0, 0)
            .single()
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(
            year_added(&eve),
            2025,
            "the server's year, not the local one"
        );
    }

    #[test]
    fn years_ago_counts_whole_years() {
        let now = utc(2026, 11, 23, 12);
        assert_eq!(years_ago(&utc(2024, 11, 23, 0), &now), 2);
        assert_eq!(years_ago(&utc(2025, 11, 24, 0), &now), 0);
        assert_eq!(years_ago(&utc(2020, 2, 29, 0), &now), 6);
        assert_eq!(years_ago(&utc(2030, 1, 1, 0), &now), 0, "a clock set back");
    }
}
