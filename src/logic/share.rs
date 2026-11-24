//! Share links without a browser: the lifetimes a new link can have, the
//! day picked on a calendar and the days it takes, the request they make,
//! the address to give out, what the list of links says of each, and how the
//! public page lays out what a link shows.

use chrono::{DateTime, Local, NaiveDate, TimeDelta, TimeZone, Utc};
use starbooru_client::Client;
use starbooru_core::share::{MAX_SHARE_DAYS, MAX_SHARE_POSTS, MAX_SHARE_TITLE_CHARS, share_path};
use starbooru_core::{MediaKind, NewShare, Share, ShareError, SharedView};

use super::library::ago;
use super::viewer::when;

/// Lifetimes offered for a new link, in days; `None` lasts until removed.
pub const LIFETIMES: [Option<u32>; 4] = [None, Some(1), Some(7), Some(30)];
/// The lifetime the sheet starts with.
pub const DEFAULT_LIFETIME: Option<u32> = Some(7);

/// A lifetime as the sheet's drop-down holds it: the days, or nothing.
pub fn lifetime_value(days: Option<u32>) -> String {
    days.map(|d| d.to_string()).unwrap_or_default()
}

/// The drop-down's value as the request's `days`. Anything that isn't a
/// lifetime the server takes means until removed.
pub fn lifetime_days(value: &str) -> Option<u32> {
    value
        .trim()
        .parse()
        .ok()
        .filter(|d| (1..=MAX_SHARE_DAYS).contains(d))
}

/// A lifetime in words: "Until removed", "7 days".
pub fn lifetime_label(days: Option<u32>) -> String {
    match days {
        None => t!("share-until-removed"),
        Some(d) => t!("share-days", count = d),
    }
}

/// When a link stops working, as the sheet says it: the moment on the local
/// clock, or that it never does.
pub fn ends_text(expires_at: Option<DateTime<Utc>>) -> String {
    match expires_at {
        Some(at) => t!("shares-expires-on", date = when(&at, &Local)),
        None => t!("shares-no-expiry"),
    }
}

/// When a new link of `days` would stop working, made `now`.
pub fn ends_after(days: Option<u32>, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    days.and_then(|d| after_days(now, d))
}

fn after_days(now: DateTime<Utc>, days: u32) -> Option<DateTime<Utc>> {
    now.checked_add_signed(TimeDelta::try_days(i64::from(days))?)
}

// ---------- a day picked on the calendar ----------

/// The drop-down's value for a day picked on the calendar.
pub const PICK_DAY: &str = "day";

/// Why a picked day can't make a link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickError {
    /// The field is empty or half typed.
    NoDay,
    /// Today or earlier.
    TooSoon,
    /// Further than a link can last.
    TooLate,
}

/// A day as a date field holds it: `2026-11-30`.
pub fn day_value(day: NaiveDate) -> String {
    day.format("%Y-%m-%d").to_string()
}

/// A date field's value as a day, when it holds a whole one.
pub fn parse_day(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").ok()
}

/// When `day` ends on the clock of `zone`: the first moment of the next.
fn day_end<Tz: TimeZone>(day: NaiveDate, zone: &Tz) -> Option<DateTime<Utc>> {
    let next = day.succ_opt()?;
    // Where the clocks skip midnight the day starts when they land. A
    // browser's clock gives a moment of the day before for a time that never
    // was, so each answer is read back on the clock.
    (0..4).find_map(|hour| {
        let at = zone
            .from_local_datetime(&next.and_hms_opt(hour, 0, 0)?)
            .earliest()?
            .with_timezone(&Utc);
        (at.with_timezone(zone).date_naive() == next).then_some(at)
    })
}

/// The days a link needs from `now` to work until the end of `day` on the
/// clock of `zone`. The server counts whole days from when it makes the
/// link, so this rounds up and the link runs on into the day after.
pub fn days_until<Tz: TimeZone>(
    day: NaiveDate,
    now: DateTime<Utc>,
    zone: &Tz,
) -> Result<u32, PickError> {
    if day <= now.with_timezone(zone).date_naive() {
        return Err(PickError::TooSoon);
    }
    let left = day_end(day, zone).ok_or(PickError::TooLate)? - now;
    let whole = left.num_days();
    let days = if TimeDelta::try_days(whole).is_some_and(|d| d < left) {
        whole.saturating_add(1)
    } else {
        whole
    };
    if days < 1 {
        return Err(PickError::TooSoon);
    }
    u32::try_from(days)
        .ok()
        .filter(|d| *d <= MAX_SHARE_DAYS)
        .ok_or(PickError::TooLate)
}

/// The date field's value as the request's `days`.
pub fn picked_days<Tz: TimeZone>(
    value: &str,
    now: DateTime<Utc>,
    zone: &Tz,
) -> Result<u32, PickError> {
    days_until(parse_day(value).ok_or(PickError::NoDay)?, now, zone)
}

/// The first day the calendar offers: tomorrow.
pub fn first_day<Tz: TimeZone>(now: DateTime<Utc>, zone: &Tz) -> Option<NaiveDate> {
    now.with_timezone(zone).date_naive().succ_opt()
}

/// The last day the calendar offers: the last a link of the longest
/// lifetime covers to its end.
pub fn last_day<Tz: TimeZone>(now: DateTime<Utc>, zone: &Tz) -> Option<NaiveDate> {
    let limit = after_days(now, MAX_SHARE_DAYS)?;
    let mut day = limit.with_timezone(zone).date_naive();
    // The day the limit falls in ends after it, so this is usually the one before.
    for _ in 0..3 {
        if days_until(day, now, zone).is_ok() {
            return Some(day);
        }
        day = day.pred_opt()?;
    }
    None
}

/// The day the calendar opens on: the last day `days` covers to its end (a
/// week's for a link until removed), so the link ends when it would have.
pub fn first_pick<Tz: TimeZone>(
    days: Option<u32>,
    now: DateTime<Utc>,
    zone: &Tz,
) -> Option<NaiveDate> {
    let ends = after_days(now, days.or(DEFAULT_LIFETIME)?)?;
    let day = ends.with_timezone(zone).date_naive().pred_opt()?;
    Some(day.max(first_day(now, zone)?).min(last_day(now, zone)?))
}

/// Why a picked day won't do, in words.
pub fn pick_error_text<Tz: TimeZone>(error: PickError, now: DateTime<Utc>, zone: &Tz) -> String {
    match error {
        PickError::NoDay => t!("share-day-missing"),
        PickError::TooSoon => t!("share-day-soon"),
        PickError::TooLate => t!(
            "share-day-late",
            max = MAX_SHARE_DAYS,
            last = last_day(now, zone).map(day_value).unwrap_or_default()
        ),
    }
}

/// The request for a link to `post_ids`, in that order, with the title as
/// typed and the lifetime chosen. Checked as the server would check it.
pub fn new_share(
    post_ids: Vec<i64>,
    title: &str,
    days: Option<u32>,
) -> Result<NewShare, ShareError> {
    NewShare {
        post_ids,
        title: Some(title.to_owned()),
        days,
    }
    .clean()
}

/// Why a link can't be made, in words.
pub fn share_error_text(error: &ShareError) -> String {
    match error {
        ShareError::NoPosts => t!("share-error-none"),
        ShareError::TooManyPosts => t!("share-error-many", max = MAX_SHARE_POSTS),
        ShareError::TitleTooLong => t!("share-error-title-long", max = MAX_SHARE_TITLE_CHARS),
        ShareError::ControlChar => t!("share-error-title-control"),
        ShareError::BadDays => t!("share-error-days", max = MAX_SHARE_DAYS),
    }
}

/// The address to give out: the page `/s/{token}` on the server, which
/// serves the web app that opens it.
pub fn share_url(client: &Client, token: &str) -> String {
    client.absolute(&share_path(token))
}

/// A post's own address, which a public server shares it by: its page on
/// the server, which serves the app to anyone.
pub fn post_url(client: &Client, id: i64) -> String {
    client.absolute(&format!("/post/{id}"))
}

/// The addresses of several posts, one a line, to copy or share at once.
pub fn post_urls(client: &Client, ids: &[i64]) -> String {
    ids.iter()
        .map(|&id| post_url(client, id))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether the link has stopped working by its date.
pub fn expired(share: &Share, now: DateTime<Utc>) -> bool {
    share.expires_at.is_some_and(|at| at <= now)
}

/// When a link stops working: "Never expires", "Expires in 3 hours",
/// "Expires in 6 days", past a month the moment itself, and once it has
/// "Expired" and when.
pub fn expiry_text(expires_at: Option<DateTime<Utc>>, now: DateTime<Utc>) -> String {
    let Some(at) = expires_at else {
        return t!("shares-no-expiry");
    };
    let left = (at - now).num_seconds();
    // Rounded to the nearest, so a week's link reads "7 days" when it's new.
    let nearest = |unit: i64| (left + unit / 2) / unit;
    match left {
        ..=0 => t!("shares-expired-on", date = when(&at, &Local)),
        1..3_600 => t!("shares-expires-minutes", count = nearest(60).max(1)),
        3_600..172_800 => t!("shares-expires-hours", count = nearest(3_600)),
        172_800..=2_678_400 => t!("shares-expires-days", count = nearest(86_400)),
        _ => t!("shares-expires-on", date = when(&at, &Local)),
    }
}

/// What the list says of a link: "Made 3 days ago · Opened twice · Expires
/// in 4 days".
pub fn share_status(share: &Share, now: DateTime<Utc>) -> String {
    let made = t!("shares-made", when = ago(share.created_at, now));
    let views = t!("shares-views", count = share.views);
    let expiry = expiry_text(share.expires_at, now);
    format!("{made} · {views} · {expiry}")
}

/// A link's name in the list: its title, or that it has none.
pub fn share_name(share: &Share) -> String {
    share
        .title
        .clone()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| t!("shares-untitled"))
}

// ---------- the public page ----------

/// How the public page shows what a link holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    /// Every post went to the recycle bin since.
    Empty,
    /// One post, on a stage of its own with zoom.
    One,
    /// Tiles that open a lightbox.
    Many,
}

pub fn layout(count: usize) -> Layout {
    match count {
        0 => Layout::Empty,
        1 => Layout::One,
        _ => Layout::Many,
    }
}

/// The page's heading: the link's title, or how many posts it holds.
pub fn heading(title: Option<&str>, count: usize) -> String {
    title
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map_or_else(|| t!("share-posts", count = count), str::to_owned)
}

/// The lightbox's next or previous place among `count`, if there is one.
pub fn step(at: usize, count: usize, forward: bool) -> Option<usize> {
    if forward {
        at.checked_add(1).filter(|n| *n < count)
    } else {
        at.checked_sub(1).filter(|n| *n < count)
    }
}

/// One post as the public page shows it, with its signed addresses made
/// absolute.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub kind: MediaKind,
    pub size: Option<(i32, i32)>,
    pub duration_ms: Option<i64>,
    pub file: String,
    pub sample: Option<String>,
    pub thumb: Option<String>,
}

impl Item {
    /// What to show: the smaller copy where there is one, unless the
    /// original is wanted. A video is always its file.
    pub fn shown(&self, original: bool) -> &str {
        match &self.sample {
            Some(sample) if !original && self.kind != MediaKind::Video => sample,
            _ => &self.file,
        }
    }
}

/// The posts of `view`, in the link's order, through `absolute`.
pub fn items(view: &SharedView, absolute: impl Fn(&str) -> String) -> Vec<Item> {
    view.posts
        .iter()
        .map(|p| Item {
            kind: p.kind,
            size: p.width.zip(p.height),
            duration_ms: p.duration_ms,
            file: absolute(&p.file_url),
            sample: p.sample_url.as_deref().map(&absolute),
            thumb: p.thumb_url.as_deref().map(&absolute),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::FixedOffset;
    use starbooru_core::SharedPost;

    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::from_timestamp(1_800_000_000, 0).unwrap()
    }

    fn share() -> Share {
        Share {
            id: 1,
            token: "0123456789abcdef0123456789abcdef".into(),
            title: None,
            posts: 3,
            cover_url: None,
            created_at: now() - TimeDelta::minutes(3),
            expires_at: None,
            views: 0,
            last_viewed_at: None,
        }
    }

    #[test]
    fn each_lifetime_becomes_the_requests_days() {
        for days in LIFETIMES {
            assert_eq!(lifetime_days(&lifetime_value(days)), days);
        }
        assert_eq!(lifetime_value(None), "");
        assert_eq!(lifetime_value(Some(7)), "7");
        assert_eq!(lifetime_days("30"), Some(30));
        // Nothing the server would refuse.
        assert_eq!(lifetime_days("0"), None);
        assert_eq!(lifetime_days("366"), None);
        assert_eq!(lifetime_days("soon"), None);
        assert_eq!(lifetime_label(None), "Until removed");
        assert_eq!(lifetime_label(Some(1)), "1 day");
        assert_eq!(lifetime_label(Some(30)), "30 days");
        assert_eq!(DEFAULT_LIFETIME, Some(7));
    }

    /// Rome in winter, an hour ahead of UTC.
    fn rome() -> FixedOffset {
        FixedOffset::east_opt(3_600).unwrap()
    }

    /// `y-m-d h:m:s` on the clock of `zone`.
    fn at(zone: &impl TimeZone, y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) -> DateTime<Utc> {
        zone.with_ymd_and_hms(y, m, d, h, min, s)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn a_picked_day_is_counted_in_whole_days_rounded_up() {
        let zone = rome();
        // An evening: tomorrow ends in 30 hours, so it takes two days.
        let now = at(&zone, 2026, 11, 23, 18, 0, 0);
        assert_eq!(days_until(day(2026, 11, 24), now, &zone), Ok(2));
        assert_eq!(first_day(now, &zone), Some(day(2026, 11, 24)));
        assert_eq!(days_until(day(2026, 12, 3), now, &zone), Ok(11));
        // Today and before can't be picked, however near the end.
        assert_eq!(
            days_until(day(2026, 11, 23), now, &zone),
            Err(PickError::TooSoon)
        );
        assert_eq!(
            days_until(day(2025, 1, 1), now, &zone),
            Err(PickError::TooSoon)
        );
        // As the field gives it, and what it can't.
        assert_eq!(picked_days("2026-12-03", now, &zone), Ok(11));
        assert_eq!(picked_days(" 2026-12-03 ", now, &zone), Ok(11));
        assert_eq!(picked_days("", now, &zone), Err(PickError::NoDay));
        assert_eq!(picked_days("2026-02-30", now, &zone), Err(PickError::NoDay));
        assert_eq!(picked_days("soon", now, &zone), Err(PickError::NoDay));
        assert_eq!(day_value(day(2027, 1, 5)), "2027-01-05");
        assert_eq!(parse_day("2027-01-05"), Some(day(2027, 1, 5)));
    }

    #[test]
    fn around_midnight_the_day_still_ends_in_time() {
        let zone = rome();
        // Half a minute before midnight, today is still today.
        let late = at(&zone, 2026, 11, 23, 23, 59, 30);
        assert_eq!(
            days_until(day(2026, 11, 23), late, &zone),
            Err(PickError::TooSoon)
        );
        assert_eq!(days_until(day(2026, 11, 24), late, &zone), Ok(2));
        // On the stroke of midnight two days are exactly enough.
        let midnight = at(&zone, 2026, 11, 24, 0, 0, 0);
        assert_eq!(
            days_until(day(2026, 11, 24), midnight, &zone),
            Err(PickError::TooSoon)
        );
        assert_eq!(days_until(day(2026, 11, 25), midnight, &zone), Ok(2));
        let after = midnight + TimeDelta::milliseconds(1);
        assert_eq!(days_until(day(2026, 11, 25), after, &zone), Ok(2));
        // Midnight in Rome is 23:00 the day before in UTC: the days are the
        // user's own.
        assert_eq!(first_day(midnight, &zone), Some(day(2026, 11, 25)));
        assert_eq!(first_day(midnight, &Utc), Some(day(2026, 11, 24)));
        assert_eq!(days_until(day(2026, 11, 24), midnight, &Utc), Ok(2));
    }

    #[test]
    fn the_server_ends_the_link_after_the_day_and_not_a_day_later() {
        // Every hour of a day, each sixth day of a year ahead: the server's
        // end (made now, plus the days) is past the day's end, and a day
        // less wouldn't be.
        for zone in [rome(), FixedOffset::west_opt(5 * 3_600).unwrap()] {
            for hour in 0..24 {
                let now = at(&zone, 2026, 11, 23, hour, 17, 41);
                for ahead in (1..365).step_by(6) {
                    let picked = day(2026, 11, 23) + TimeDelta::days(ahead);
                    let Ok(days) = days_until(picked, now, &zone) else {
                        continue;
                    };
                    let end = day_end(picked, &zone).unwrap();
                    assert!(now + TimeDelta::days(days.into()) >= end, "{now} {picked}");
                    assert!(
                        now + TimeDelta::days(i64::from(days) - 1) < end,
                        "{now} {picked}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_last_day_is_the_last_a_years_link_covers() {
        let zone = rome();
        let now = at(&zone, 2026, 11, 23, 18, 0, 0);
        let last = last_day(now, &zone).unwrap();
        assert_eq!(last, day(2027, 11, 22));
        assert_eq!(days_until(last, now, &zone), Ok(MAX_SHARE_DAYS));
        assert_eq!(
            days_until(day(2027, 11, 23), now, &zone),
            Err(PickError::TooLate)
        );
        assert_eq!(
            picked_days("2099-01-01", now, &zone),
            Err(PickError::TooLate)
        );
        assert_eq!(
            picked_days("+262142-12-31", now, &zone),
            Err(PickError::TooLate)
        );
        // At midnight the year's link ends just as that day does.
        let midnight = at(&zone, 2026, 11, 24, 0, 0, 0);
        let last = last_day(midnight, &zone).unwrap();
        assert_eq!(last, day(2027, 11, 23));
        assert_eq!(days_until(last, midnight, &zone), Ok(MAX_SHARE_DAYS));
    }

    #[test]
    fn the_calendar_opens_where_the_lifetime_would_have_ended() {
        let zone = rome();
        let now = at(&zone, 2026, 11, 23, 18, 0, 0);
        // A week covers through the 29th and ends on the 30th at six.
        let week = first_pick(Some(7), now, &zone).unwrap();
        assert_eq!(week, day(2026, 11, 29));
        assert_eq!(days_until(week, now, &zone), Ok(7));
        assert_eq!(first_pick(None, now, &zone), Some(week));
        let month = first_pick(Some(30), now, &zone).unwrap();
        assert_eq!(days_until(month, now, &zone), Ok(30));
        // A day covers no whole day, so the calendar starts on tomorrow.
        assert_eq!(first_pick(Some(1), now, &zone), first_day(now, &zone));
        assert_eq!(first_pick(Some(400), now, &zone), last_day(now, &zone));
    }

    /// Clocks that go from 23:59:59 straight to 01:00 on 29 March 2026, as
    /// some countries' do. Asked for a time that never was, chrono says so;
    /// a browser's clock answers with the summer's offset instead.
    #[derive(Clone, Copy, Debug)]
    struct SkipsMidnight {
        browser: bool,
    }

    impl SkipsMidnight {
        fn change() -> chrono::NaiveDateTime {
            day(2026, 3, 29).and_hms_opt(0, 0, 0).unwrap()
        }
        fn winter() -> FixedOffset {
            FixedOffset::east_opt(0).unwrap()
        }
        fn summer() -> FixedOffset {
            FixedOffset::east_opt(3_600).unwrap()
        }
    }

    impl TimeZone for SkipsMidnight {
        type Offset = FixedOffset;
        fn from_offset(_: &FixedOffset) -> Self {
            SkipsMidnight { browser: false }
        }
        fn offset_from_local_date(
            &self,
            local: &NaiveDate,
        ) -> chrono::MappedLocalTime<FixedOffset> {
            self.offset_from_local_datetime(&local.and_hms_opt(12, 0, 0).unwrap())
        }
        fn offset_from_local_datetime(
            &self,
            local: &chrono::NaiveDateTime,
        ) -> chrono::MappedLocalTime<FixedOffset> {
            let change = Self::change();
            let never_was = *local >= change && *local < change + TimeDelta::hours(1);
            if never_was && !self.browser {
                chrono::MappedLocalTime::None
            } else if *local < change {
                chrono::MappedLocalTime::Single(Self::winter())
            } else {
                chrono::MappedLocalTime::Single(Self::summer())
            }
        }
        fn offset_from_utc_date(&self, utc: &NaiveDate) -> FixedOffset {
            self.offset_from_utc_datetime(&utc.and_hms_opt(0, 0, 0).unwrap())
        }
        fn offset_from_utc_datetime(&self, utc: &chrono::NaiveDateTime) -> FixedOffset {
            if *utc < Self::change() {
                Self::winter()
            } else {
                Self::summer()
            }
        }
    }

    #[test]
    fn a_day_whose_next_skips_midnight_ends_when_the_clocks_land() {
        let landed = at(&Utc, 2026, 3, 29, 0, 0, 0);
        let now = at(&Utc, 2026, 3, 27, 23, 0, 0);
        for zone in [
            SkipsMidnight { browser: false },
            SkipsMidnight { browser: true },
        ] {
            assert_eq!(day_end(day(2026, 3, 28), &zone), Some(landed), "{zone:?}");
            // 25 hours left: one day would end the link an hour early.
            assert_eq!(days_until(day(2026, 3, 28), now, &zone), Ok(2), "{zone:?}");
        }
    }

    #[test]
    fn the_sheet_says_when_the_link_ends_and_what_is_wrong() {
        let zone = rome();
        let now = at(&zone, 2026, 11, 23, 18, 0, 0);
        assert_eq!(ends_after(None, now), None);
        assert_eq!(ends_text(None), "Never expires");
        let week = now + TimeDelta::days(7);
        assert_eq!(ends_after(Some(7), now), Some(week));
        assert_eq!(
            ends_text(Some(week)),
            format!("Expires {}", when(&week, &Local))
        );
        assert_eq!(
            pick_error_text(PickError::NoDay, now, &zone),
            "Pick the last day the link works."
        );
        assert_eq!(
            pick_error_text(PickError::TooSoon, now, &zone),
            "Pick tomorrow or a later day."
        );
        assert_eq!(
            pick_error_text(PickError::TooLate, now, &zone),
            "Pick 2027-11-22 or earlier: 365 days at most."
        );
        assert_ne!(PICK_DAY, lifetime_value(None));
        assert_eq!(lifetime_days(PICK_DAY), None);
    }

    #[test]
    fn the_request_keeps_the_order_and_the_choice() {
        let week = new_share(vec![5, 3, 5, 9], "  Trip  ", lifetime_days("7")).unwrap();
        assert_eq!(week.post_ids, [5, 3, 9]);
        assert_eq!(week.title.as_deref(), Some("Trip"));
        let json = serde_json::to_value(&week).unwrap();
        assert_eq!(json["days"], 7);
        assert_eq!(json["post_ids"], serde_json::json!([5, 3, 9]));
        // Until removed is no lifetime at all, and an empty title none.
        let kept = new_share(vec![5], " ", lifetime_days("")).unwrap();
        assert_eq!(kept.days, None);
        assert_eq!(kept.title, None);
        assert!(serde_json::to_value(&kept).unwrap()["days"].is_null());
        // What the server would refuse, in words.
        let none = new_share(Vec::new(), "", None).unwrap_err();
        assert_eq!(share_error_text(&none), "Pick at least one post to share.");
        let long = new_share(vec![1], &"x".repeat(101), None).unwrap_err();
        assert!(share_error_text(&long).contains("100"));
        let many = new_share((0..=500).collect(), "", None).unwrap_err();
        assert!(share_error_text(&many).contains("500"));
    }

    #[test]
    fn the_address_is_the_servers_page_for_the_token() {
        let token = "0123456789abcdef0123456789abcdef";
        let client = Client::new("https://booru.example/").unwrap();
        assert_eq!(
            share_url(&client, token),
            "https://booru.example/s/0123456789abcdef0123456789abcdef"
        );
        // A server under a path keeps it.
        let client = Client::new("http://nas.local:8080/booru").unwrap();
        assert_eq!(
            share_url(&client, token),
            "http://nas.local:8080/booru/s/0123456789abcdef0123456789abcdef"
        );
    }

    #[test]
    fn a_public_server_shares_a_post_by_its_own_address() {
        let client = Client::new("https://booru.example/").unwrap();
        assert_eq!(post_url(&client, 12), "https://booru.example/post/12");
        assert_eq!(
            post_urls(&client, &[3, 12]),
            "https://booru.example/post/3\nhttps://booru.example/post/12"
        );
        assert_eq!(post_urls(&client, &[]), "");
        let client = Client::new("http://nas.local:8080/booru").unwrap();
        assert_eq!(post_url(&client, 5), "http://nas.local:8080/booru/post/5");
    }

    #[test]
    fn the_list_says_when_a_link_expires() {
        let now = now();
        let at = |d: TimeDelta| expiry_text(Some(now + d), now);
        assert_eq!(expiry_text(None, now), "Never expires");
        assert_eq!(at(TimeDelta::seconds(20)), "Expires in 1 minute");
        assert_eq!(at(TimeDelta::minutes(42)), "Expires in 42 minutes");
        assert_eq!(at(TimeDelta::hours(1)), "Expires in 1 hour");
        // A day's link, just made.
        assert_eq!(
            at(TimeDelta::days(1) - TimeDelta::seconds(5)),
            "Expires in 24 hours"
        );
        assert_eq!(at(TimeDelta::hours(47)), "Expires in 47 hours");
        // A week's, just made, and one half gone.
        assert_eq!(
            at(TimeDelta::days(7) - TimeDelta::seconds(5)),
            "Expires in 7 days"
        );
        assert_eq!(at(TimeDelta::hours(84)), "Expires in 4 days");
        assert_eq!(at(TimeDelta::days(30)), "Expires in 30 days");
        // Past a month, and gone: the moment, on the local clock.
        let far = now + TimeDelta::days(200);
        assert_eq!(
            expiry_text(Some(far), now),
            format!("Expires {}", when(&far, &Local))
        );
        let gone = now - TimeDelta::days(1);
        assert_eq!(
            expiry_text(Some(gone), now),
            format!("Expired {}", when(&gone, &Local))
        );
        assert_eq!(
            expiry_text(Some(now), now),
            format!("Expired {}", when(&now, &Local))
        );
    }

    #[test]
    fn the_list_describes_each_link() {
        let now = now();
        let mut link = share();
        assert_eq!(
            share_status(&link, now),
            "Made 3 minutes ago · Never opened · Never expires"
        );
        assert!(!expired(&link, now));
        link.views = 1;
        link.expires_at = Some(now + TimeDelta::days(7));
        assert_eq!(
            share_status(&link, now),
            "Made 3 minutes ago · Opened once · Expires in 7 days"
        );
        link.views = 12;
        link.expires_at = Some(now - TimeDelta::hours(2));
        assert!(share_status(&link, now).contains("Opened 12 times"));
        assert!(expired(&link, now));
        assert_eq!(share_name(&link), "Untitled link");
        link.title = Some("Trip".into());
        assert_eq!(share_name(&link), "Trip");
    }

    #[test]
    fn one_post_gets_a_stage_and_several_get_tiles() {
        assert_eq!(layout(0), Layout::Empty);
        assert_eq!(layout(1), Layout::One);
        assert_eq!(layout(3), Layout::Many);
        assert_eq!(layout(500), Layout::Many);
        assert_eq!(heading(None, 1), "1 post");
        assert_eq!(heading(Some("  "), 3), "3 posts");
        assert_eq!(heading(Some(" Trip "), 3), "Trip");
    }

    #[test]
    fn the_lightbox_steps_within_the_posts() {
        assert_eq!(step(0, 3, true), Some(1));
        assert_eq!(step(2, 3, true), None);
        assert_eq!(step(0, 3, false), None);
        assert_eq!(step(2, 3, false), Some(1));
        assert_eq!(step(5, 3, false), None);
        assert_eq!(step(usize::MAX, 3, true), None);
    }

    #[test]
    fn the_page_shows_the_signed_addresses_it_was_given() {
        let post = |kind, sample: Option<&str>| SharedPost {
            kind,
            mime: "image/png".into(),
            width: Some(1200),
            height: Some(800),
            duration_ms: None,
            file_url: "/media/original/ab.png?exp=1&sig=f".into(),
            thumb_url: Some("/media/thumb/ab.webp?exp=1&sig=e".into()),
            sample_url: sample.map(str::to_owned),
        };
        let view = SharedView {
            title: None,
            server: "StarBooru".into(),
            expires_at: None,
            posts: vec![
                post(MediaKind::Image, Some("/media/sample/ab.webp?exp=1&sig=d")),
                post(MediaKind::Video, Some("/media/sample/cd.webp?exp=1&sig=c")),
                post(MediaKind::Animation, None),
            ],
        };
        let client = Client::new("https://booru.example").unwrap();
        let items = items(&view, |u| client.absolute(u));
        let [image, video, gif] = items.as_slice() else {
            panic!("{items:?}");
        };
        assert_eq!(image.size, Some((1200, 800)));
        assert_eq!(
            image.shown(false),
            "https://booru.example/media/sample/ab.webp?exp=1&sig=d"
        );
        assert_eq!(
            image.shown(true),
            "https://booru.example/media/original/ab.png?exp=1&sig=f"
        );
        assert_eq!(
            image.thumb.as_deref(),
            Some("https://booru.example/media/thumb/ab.webp?exp=1&sig=e")
        );
        // A video plays its file; a GIF without a smaller copy shows its own.
        assert_eq!(video.shown(false), video.file);
        assert_eq!(gif.shown(false), gif.file);
    }
}
