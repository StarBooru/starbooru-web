//! Share links without a browser: the lifetimes a new link can have and the
//! request they make, the address to give out, what the list of links says
//! of each, and how the public page lays out what a link shows.

use chrono::{DateTime, Local, Utc};
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
    use chrono::TimeDelta;
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
