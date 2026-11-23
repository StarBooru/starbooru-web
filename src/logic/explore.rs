//! Explore's rules that need no browser: the window it asks for, what a
//! shuffle searches, and the years of the posts added on this day.

use chrono::{DateTime, Datelike, Utc};
use starbooru_core::{Order, with_order};

/// The window sent with every request, in days. Only Trending and Popular
/// tags read it, and a private booru shows neither, so the shortest window
/// leaves the server the least to count.
pub const DAYS: u32 = 1;

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
    fn the_window_is_the_shortest_the_server_takes() {
        assert_eq!(DAYS, 1);
        assert!((1..=MAX_EXPLORE_DAYS).contains(&DAYS));
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
