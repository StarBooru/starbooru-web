//! The viewer's rules that need no browser: what the details card says, the
//! name a download gets, how the tags group, where previous and next lead, the
//! picture's box before it loads, when the original is worth its weight, and
//! what a finger on the stage meant.

use std::fmt::Display;

use chrono::{DateTime, TimeZone, Utc};
use starbooru_core::{
    MediaKind, Neighbors, Post, PostTag, SAMPLE_SIZE, SECTIONS, TagCategory, with_color,
};

use crate::route;

// ---------- the details card ----------

/// `1536` → `1.5 KB`.
pub fn human_size(bytes: i64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes.max(0) as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    let name = UNITS.get(unit).copied().unwrap_or("B");
    if unit == 0 {
        format!("{value:.0} {name}")
    } else {
        format!("{value:.1} {name}")
    }
}

/// `m:ss`, or `h:mm:ss` from an hour. `None` for nonsense.
pub fn format_duration(ms: i64) -> Option<String> {
    if ms < 0 {
        return None;
    }
    let s = ms.saturating_add(500) / 1000;
    let (h, m, s) = (s / 3600, s / 60 % 60, s % 60);
    Some(if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    })
}

/// The gallery of posts in colour `hex` (`#f581fa`).
pub fn color_href(hex: &str) -> String {
    route::gallery(&with_color("", Some(hex.trim().trim_start_matches('#'))))
}

/// `1400 × 900`, or a dash when the server doesn't know.
pub fn dimensions(width: Option<i32>, height: Option<i32>) -> String {
    match (width, height) {
        (Some(w), Some(h)) if w > 0 && h > 0 => format!("{w} × {h}"),
        _ => "—".to_owned(),
    }
}

const TIME_FORMAT: &str = "%Y-%m-%d %H:%M";

/// A moment as the clock in `zone` showed it.
pub fn when<Tz: TimeZone>(at: &DateTime<Utc>, zone: &Tz) -> String
where
    Tz::Offset: Display,
{
    at.with_timezone(zone).format(TIME_FORMAT).to_string()
}

/// The date in a post's file. A picture's is the camera's own clock, stored as
/// if it were UTC because EXIF has no time zone, so it shows as it is; a
/// video's is a real moment, shown in `zone`.
pub fn file_date<Tz: TimeZone>(kind: MediaKind, at: &DateTime<Utc>, zone: &Tz) -> String
where
    Tz::Offset: Display,
{
    match kind {
        MediaKind::Video => when(at, zone),
        MediaKind::Image | MediaKind::Animation => at.naive_utc().format(TIME_FORMAT).to_string(),
    }
}

/// The source to link to, when it is a web address and nothing else.
pub fn source_link(source: &str) -> Option<&str> {
    let s = source.trim();
    let lower = s.get(..8).map(str::to_ascii_lowercase).unwrap_or_default();
    let web = lower.starts_with("http://") || lower.starts_with("https://");
    let rest = s.split_once("://").map_or("", |(_, rest)| rest);
    (web && !rest.is_empty() && !s.contains(char::is_whitespace)).then_some(s)
}

// ---------- downloads ----------

/// The name a post's original is saved under: `<sha256>.<ext>`, the extension
/// from the file's address, else from its type.
pub fn download_name(sha256: &str, file_url: &str, mime: &str) -> String {
    let clean = |s: &str| -> String {
        s.chars()
            .filter(char::is_ascii_alphanumeric)
            .collect::<String>()
            .to_ascii_lowercase()
    };
    let path = file_url.split(['?', '#']).next().unwrap_or_default();
    let file = path.rsplit('/').next().unwrap_or_default();
    let from_path = file.rsplit_once('.').map(|(_, ext)| ext);
    // `image/svg+xml` → `svg`.
    let from_mime = mime
        .split_once('/')
        .and_then(|(_, sub)| sub.split(['+', ';']).next());
    let ext = [from_path, from_mime]
        .into_iter()
        .flatten()
        .map(clean)
        .find(|e| !e.is_empty() && e.len() <= 8)
        .unwrap_or_else(|| "bin".to_owned());
    let stem = clean(sha256);
    let stem = if stem.is_empty() {
        "download".to_owned()
    } else {
        stem
    };
    format!("{stem}.{ext}")
}

/// `scheme://host[:port]` of an absolute address, in lower case and without the
/// scheme's own port. `None` for an address without a scheme or a host.
pub fn origin_of(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    let scheme_ok = !scheme.is_empty()
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c));
    if !scheme_ok {
        return None;
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    if host.is_empty() {
        return None;
    }
    let scheme = scheme.to_ascii_lowercase();
    let mut host = host.to_ascii_lowercase();
    let default_port = match scheme.as_str() {
        "http" => Some(":80"),
        "https" => Some(":443"),
        _ => None,
    };
    if let Some(short) = default_port.and_then(|port| host.strip_suffix(port)) {
        host = short.to_owned();
    }
    Some(format!("{scheme}://{host}"))
}

/// Whether `href` is on the page's own origin, where `<a download>` names the
/// file. Elsewhere the browser ignores the name, or opens the file instead.
pub fn same_origin(href: &str, page: &str) -> bool {
    matches!((origin_of(href), origin_of(page)), (Some(a), Some(b)) if a == b)
}

// ---------- tags ----------

/// A heading in the viewer's tag card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Group {
    Category(TagCategory),
    /// General tags under a section of the tag tree, such as `hair`.
    Section(String),
    /// General tags outside any tree.
    Other,
}

impl Group {
    /// Artist, copyright, character, the built-in sections in order, other
    /// sections by name, general tags without one, meta.
    fn rank(&self) -> (u8, usize, &str) {
        match self {
            Group::Category(TagCategory::Artist) => (0, 0, ""),
            Group::Category(TagCategory::Copyright) => (1, 0, ""),
            Group::Category(TagCategory::Character) => (2, 0, ""),
            Group::Section(name) => match SECTIONS.iter().position(|s| s == name) {
                Some(i) => (3, i, ""),
                None => (4, 0, name),
            },
            Group::Other | Group::Category(TagCategory::General) => (5, 0, ""),
            Group::Category(TagCategory::Meta) => (6, 0, ""),
        }
    }
}

/// A post's tags under their headings, each group sorted by name.
pub fn group_tags(tags: &[PostTag]) -> Vec<(Group, Vec<PostTag>)> {
    let mut groups: Vec<(Group, Vec<PostTag>)> = Vec::new();
    for tag in tags {
        let group = match (&tag.category, &tag.section) {
            (TagCategory::General, Some(section)) => Group::Section(section.clone()),
            (TagCategory::General, None) => Group::Other,
            (category, _) => Group::Category(*category),
        };
        match groups.iter_mut().find(|(g, _)| *g == group) {
            Some((_, list)) => list.push(tag.clone()),
            None => groups.push((group, vec![tag.clone()])),
        }
    }
    groups.sort_by(|a, b| a.0.rank().cmp(&b.0.rank()));
    for (_, list) in &mut groups {
        list.sort_by(|a, b| a.name.cmp(&b.name));
    }
    groups
}

/// Whether the groups need their headings: one saying "Other" says nothing.
pub fn headed(groups: &[(Group, Vec<PostTag>)]) -> bool {
    !matches!(groups, [(Group::Other, _)])
}

/// The chip class that colours a tag by its category.
pub fn category_class(category: TagCategory) -> &'static str {
    match category {
        TagCategory::General => "chip",
        TagCategory::Artist => "chip cat-artist",
        TagCategory::Character => "chip cat-character",
        TagCategory::Copyright => "chip cat-copyright",
        TagCategory::Meta => "chip cat-meta",
    }
}

// ---------- previous and next ----------

/// Presses of previous or next that can wait for the posts to be known.
/// Holding a key down shouldn't leave the viewer walking on long after it's
/// let go.
pub const MOST_QUEUED: i32 = 6;

/// The queue of presses after one more: the other way cancels one out.
pub fn queued(steps: i32, forward: bool) -> i32 {
    let step = if forward { 1 } else { -1 };
    steps.saturating_add(step).clamp(-MOST_QUEUED, MOST_QUEUED)
}

/// The address of the post before or after, in the same search.
pub fn neighbour_href(n: Neighbors, forward: bool, q: &str, seed: Option<i64>) -> Option<String> {
    let id = if forward { n.next } else { n.prev };
    id.map(|id| route::post(id, q, seed))
}

/// Whether Back can go back through history to the gallery, which then
/// shows itself as it was: only when the viewer was opened from the gallery's
/// results for this same search. Previous and next replace the address, so
/// the gallery is still the page before.
pub fn back_through_history(opened_from: Option<&str>, q: &str) -> bool {
    opened_from.is_some_and(|from| from.trim() == q.trim())
}

/// The entrance after previous or next: from the side the post came from.
pub fn enter_class(forward: Option<bool>) -> &'static str {
    match forward {
        Some(true) => "from-next",
        Some(false) => "from-prev",
        None => "",
    }
}

// ---------- the picture ----------

/// Sizes the picture's box to the fitted picture before it loads, so its
/// colour and thumbnail can stand in and nothing jumps when it arrives.
/// `--fit-h` is the stage's height. A picture isn't enlarged past its own
/// size, except where `--fit-cap` lifts that: fullscreen fits the display.
pub fn fit_style(size: Option<(i32, i32)>) -> String {
    match size {
        Some((w, h)) if w > 0 && h > 0 => format!(
            "; aspect-ratio: {w} / {h}; width: min(100%, calc(var(--fit-h) * {w} / {h}), var(--fit-cap, {w}px))"
        ),
        _ => String::new(),
    }
}

/// `url("…")` for inline CSS.
pub fn css_url(url: &str) -> String {
    format!(
        "url(\"{}\")",
        url.replace('\\', "\\\\").replace('"', "\\\"")
    )
}

/// Whether the sample, which fits in `SAMPLE_SIZE`², has every pixel the
/// screen can show of a picture of `natural` size fitted into a stage of
/// `stage` CSS pixels, at `dpr` device pixels per CSS pixel. The picture is
/// never enlarged past its natural CSS size, and the original can't show more
/// than its own pixels, so a sample of a small picture always is.
pub fn sample_is_sharp(natural: (i32, i32), stage: (f64, f64), dpr: f64) -> bool {
    let (w, h) = (f64::from(natural.0), f64::from(natural.1));
    if w <= 0.0 || h <= 0.0 || stage.0 <= 0.0 || stage.1 <= 0.0 {
        return true;
    }
    // Shares of the original's pixels: shown on screen, and in the sample.
    let shown = ((stage.0 / w).min(stage.1 / h).min(1.0) * dpr).min(1.0);
    let sample = (f64::from(SAMPLE_SIZE) / w.max(h)).min(1.0);
    sample >= shown
}

/// Whether the stage, once measured, shows more device pixels of this post
/// than its sample has: fullscreen on a big or dense screen.
pub fn needs_original(p: &Post, stage: Option<(f64, f64)>, dpr: Option<f64>) -> bool {
    let (Some(stage), Some(dpr)) = (stage, dpr) else {
        return false;
    };
    p.kind != MediaKind::Video
        && p.sample_url.is_some()
        && p.width
            .zip(p.height)
            .is_some_and(|size| !sample_is_sharp(size, stage, dpr))
}

/// What the stage shows: the sample when there is one, unless the original
/// is wanted.
pub fn shown_url(p: &Post, original: bool) -> &str {
    match &p.sample_url {
        Some(sample) if !original => sample,
        _ => &p.file_url,
    }
}

// ---------- fingers on the stage ----------

/// How far a finger travels before it counts as a swipe.
const SLOP: f64 = 10.0;

/// Which way a finger went: `None` while it's still within a few pixels,
/// then whether it went more sideways than up or down.
pub fn sideways(dx: f64, dy: f64) -> Option<bool> {
    let far = dx.hypot(dy);
    (far >= SLOP).then(|| dx.abs() > dy.abs())
}

/// How fast a finger was going as it let go, in pixels per millisecond, from
/// its last 100 ms of `(time, x)`.
pub fn speed(trail: &[(f64, f64)]) -> f64 {
    let Some(&(t1, x1)) = trail.last() else {
        return 0.0;
    };
    let (t0, x0) = trail
        .iter()
        .copied()
        .find(|(t, _)| t1 - t <= 100.0)
        .unwrap_or((t1, x1));
    if t1 > t0 { (x1 - x0) / (t1 - t0) } else { 0.0 }
}

/// Whether a swipe let go `dx` pixels across, moving at `speed`, turns to
/// another post on a stage `width` wide: `Some(true)` for the next (the
/// finger went left), `Some(false)` for the previous.
pub fn swipe_turns(dx: f64, speed: f64, width: f64) -> Option<bool> {
    let far = (width * 0.22).min(140.0);
    let flung = speed.abs() > 0.5 && speed.signum() == dx.signum() && dx.abs() > 30.0;
    (dx.abs() >= far || flung).then_some(dx < 0.0)
}

/// Where the picture follows the finger to: at the end of the list it gives
/// a little and no more.
pub fn resist(dx: f64, at_end: bool) -> f64 {
    if at_end { dx * 0.25 } else { dx }
}

/// A tap on the stage: when, and where.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tap {
    pub time: f64,
    pub x: f64,
    pub y: f64,
}

/// Whether `second` is the second tap of a double tap after `first`.
pub fn double_tap(first: Option<Tap>, second: Tap) -> bool {
    first.is_some_and(|f| {
        let gap = second.time - f.time;
        (0.0..=300.0).contains(&gap) && (second.x - f.x).hypot(second.y - f.y) <= 30.0
    })
}

#[cfg(test)]
mod tests {
    use chrono::FixedOffset;

    use super::*;

    fn tag(name: &str, category: TagCategory, section: Option<&str>) -> PostTag {
        PostTag {
            name: name.into(),
            category,
            section: section.map(str::to_owned),
        }
    }

    fn post() -> Post {
        serde_json::from_value(serde_json::json!({
            "id": 5, "kind": "image", "mime": "image/png", "file_size": 1336381,
            "width": 6000, "height": 4000, "sha256": "39fb", "rating": "safe", "score": 0,
            "my_vote": 0, "source": null, "tags": [],
            "created_at": "2026-01-01T00:00:00Z", "deleted": false,
            "file_url": "/media/original/39fb.png?exp=1&sig=2",
            "thumb_url": "/media/thumb/39fb.jpg?exp=1&sig=3",
            "sample_url": "/media/sample/39fb.jpg?exp=1&sig=4",
        }))
        .unwrap()
    }

    #[test]
    fn sizes_read_in_the_nearest_unit() {
        assert_eq!(human_size(0), "0 B");
        assert_eq!(human_size(1023), "1023 B");
        assert_eq!(human_size(1536), "1.5 KB");
        assert_eq!(human_size(5 * 1024 * 1024), "5.0 MB");
        assert_eq!(human_size(1_336_381), "1.3 MB");
        assert_eq!(human_size(-4), "0 B");
        assert!(human_size(i64::MAX).ends_with(" TB"));
        assert_eq!(dimensions(Some(1400), Some(900)), "1400 × 900");
        assert_eq!(dimensions(None, Some(900)), "—");
        assert_eq!(dimensions(Some(0), Some(900)), "—");
    }

    #[test]
    fn durations_read_like_a_player() {
        assert_eq!(format_duration(6000).as_deref(), Some("0:06"));
        assert_eq!(format_duration(0).as_deref(), Some("0:00"));
        // Rounded to the nearest second.
        assert_eq!(format_duration(59_499).as_deref(), Some("0:59"));
        assert_eq!(format_duration(59_500).as_deref(), Some("1:00"));
        assert_eq!(format_duration(754_000).as_deref(), Some("12:34"));
        assert_eq!(format_duration(3_723_000).as_deref(), Some("1:02:03"));
        assert_eq!(format_duration(-1), None);
        assert!(format_duration(i64::MAX).is_some());
    }

    #[test]
    fn a_swatch_searches_its_colour() {
        assert_eq!(color_href("#f581fa"), "/?q=color:f581fa");
        assert_eq!(color_href(" ae82e2"), "/?q=color:ae82e2");
    }

    #[test]
    fn times_show_in_the_users_zone_and_a_cameras_as_it_was() {
        let at: DateTime<Utc> = "2026-01-02T03:04:05Z".parse().unwrap();
        let rome = FixedOffset::east_opt(2 * 3600).unwrap();
        let utc = FixedOffset::east_opt(0).unwrap();
        assert_eq!(when(&at, &rome), "2026-01-02 05:04");
        assert_eq!(when(&at, &utc), "2026-01-02 03:04");
        // Across midnight, to the day before.
        let west = FixedOffset::west_opt(5 * 3600).unwrap();
        assert_eq!(when(&at, &west), "2026-01-01 22:04");
        // A camera's clock has no zone: shown as it was.
        assert_eq!(file_date(MediaKind::Image, &at, &rome), "2026-01-02 03:04");
        assert_eq!(
            file_date(MediaKind::Animation, &at, &rome),
            "2026-01-02 03:04"
        );
        assert_eq!(file_date(MediaKind::Video, &at, &rome), "2026-01-02 05:04");
    }

    #[test]
    fn only_a_web_address_is_a_link() {
        assert_eq!(
            source_link(" https://ffmpeg.org/ffmpeg-filters.html "),
            Some("https://ffmpeg.org/ffmpeg-filters.html")
        );
        assert_eq!(
            source_link("HTTP://Example.com"),
            Some("HTTP://Example.com")
        );
        for not_one in [
            "",
            "my camera",
            "javascript:alert(1)",
            "https://",
            "ftp://example.com",
            "https://a b",
            "data:text/html,x",
            "//example.com",
        ] {
            assert_eq!(source_link(not_one), None, "{not_one}");
        }
    }

    #[test]
    fn a_download_is_named_by_its_hash_and_type() {
        assert_eq!(
            download_name("39fb", "/media/original/39fb.png?exp=1&sig=2", "image/png"),
            "39fb.png"
        );
        // The file's own extension comes first.
        assert_eq!(
            download_name("ab", "/media/original/ab.jpg?exp=1", "image/jpeg"),
            "ab.jpg"
        );
        assert_eq!(
            download_name("ab", "/media/original/ab?exp=1", "image/jpeg"),
            "ab.jpeg"
        );
        assert_eq!(download_name("ab", "", "image/svg+xml"), "ab.svg");
        assert_eq!(download_name("ab", "", "video/mp4; codecs=x"), "ab.mp4");
        assert_eq!(download_name("ab", "", ""), "ab.bin");
        assert_eq!(download_name("ab", "/x/ab.PNG", ""), "ab.png");
        // Nothing that could lead out of the folder, or say nothing.
        assert_eq!(download_name("../../etc", "/a/b.p/n?g", "a/b"), "etc.b");
        assert_eq!(download_name("", "", ""), "download.bin");
        assert_eq!(
            download_name("ab", "/x/ab.thisistoolong", "image/gif"),
            "ab.gif"
        );
    }

    #[test]
    fn a_download_knows_when_its_name_would_be_ignored() {
        let page = "http://127.0.0.1:8088";
        assert!(same_origin(
            "http://127.0.0.1:8088/media/original/a.png?x=1",
            page
        ));
        assert!(same_origin("HTTP://LOCALHOST/a", "http://localhost:80"));
        assert!(same_origin(
            "https://booru.example:443/m",
            "https://booru.example"
        ));
        assert!(!same_origin("http://127.0.0.1:8080/media/a.png", page));
        assert!(!same_origin("https://127.0.0.1:8088/media/a.png", page));
        assert!(!same_origin(
            "https://evil.example@booru.example/",
            "https://evil.example"
        ));
        assert!(!same_origin("/media/a.png", page));
        assert!(!same_origin("http:///a", "http:///b"));
        assert_eq!(
            origin_of("https://user:pw@Booru.Example:8443/x?y#z").as_deref(),
            Some("https://booru.example:8443")
        );
        assert_eq!(
            origin_of("http://[::1]:80/x").as_deref(),
            Some("http://[::1]")
        );
        assert_eq!(origin_of("no scheme"), None);
    }

    #[test]
    fn tags_group_by_category_in_the_references_order() {
        // Post 5 of the test server.
        let tags = [
            tag("blue_sky", TagCategory::General, None),
            tag("ffmpeg", TagCategory::Copyright, None),
            tag("fractal", TagCategory::General, None),
            tag("hd", TagCategory::Meta, None),
            tag("lavfi", TagCategory::Artist, None),
            tag("mandelbrot", TagCategory::General, None),
            tag("sky_blue_hair", TagCategory::General, Some("hair")),
            tag("smile", TagCategory::General, Some("expression")),
            tag("1girl", TagCategory::General, Some("people")),
            tag("gear", TagCategory::General, Some("my_section")),
            tag("alice", TagCategory::Character, None),
        ];
        let groups = group_tags(&tags);
        let heads: Vec<Group> = groups.iter().map(|(g, _)| g.clone()).collect();
        assert_eq!(
            heads,
            [
                Group::Category(TagCategory::Artist),
                Group::Category(TagCategory::Copyright),
                Group::Category(TagCategory::Character),
                Group::Section("people".into()),
                Group::Section("hair".into()),
                Group::Section("expression".into()),
                Group::Section("my_section".into()),
                Group::Other,
                Group::Category(TagCategory::Meta),
            ]
        );
        let other: Vec<&str> = groups
            .iter()
            .find(|(g, _)| *g == Group::Other)
            .map(|(_, t)| t.iter().map(|t| t.name.as_str()).collect())
            .unwrap();
        assert_eq!(other, ["blue_sky", "fractal", "mandelbrot"]);
        assert!(headed(&groups));
        assert!(group_tags(&[]).is_empty());
    }

    #[test]
    fn plain_tags_alone_need_no_heading() {
        let tags = [
            tag("b", TagCategory::General, None),
            tag("a", TagCategory::General, None),
        ];
        let groups = group_tags(&tags);
        assert!(!headed(&groups));
        assert_eq!(groups.first().map(|(_, t)| t.len()), Some(2));
        assert_eq!(category_class(TagCategory::General), "chip");
        assert_eq!(category_class(TagCategory::Meta), "chip cat-meta");
    }

    #[test]
    fn no_press_is_lost_and_the_queue_stays_short() {
        assert_eq!(queued(0, true), 1);
        assert_eq!(queued(2, true), 3);
        assert_eq!(queued(0, false), -1);
        // The other way cancels one out.
        assert_eq!(queued(2, false), 1);
        // A key held down doesn't leave it walking on for long.
        assert_eq!(queued(MOST_QUEUED, true), MOST_QUEUED);
        assert_eq!(queued(-MOST_QUEUED, false), -MOST_QUEUED);
    }

    #[test]
    fn a_neighbour_keeps_the_search_and_its_shuffle() {
        let n = Neighbors {
            prev: Some(6),
            next: Some(4),
        };
        assert_eq!(
            neighbour_href(n, true, "", None).as_deref(),
            Some("/post/4")
        );
        assert_eq!(
            neighbour_href(n, false, " cat dog ", None).as_deref(),
            Some("/post/6?q=cat+dog")
        );
        assert_eq!(
            neighbour_href(n, true, "order:random", Some(-42)).as_deref(),
            Some("/post/4?q=order:random&seed=-42")
        );
        // The ends of the list lead nowhere.
        let first = Neighbors {
            prev: None,
            next: Some(2),
        };
        assert_eq!(neighbour_href(first, false, "cat", None), None);
        assert_eq!(neighbour_href(Neighbors::default(), true, "", None), None);
        assert!(back_through_history(Some("cat"), " cat "));
        assert!(back_through_history(Some(""), ""));
        assert!(!back_through_history(Some("cat"), "dog"));
        // Opened from a link or typed in: the gallery isn't behind it.
        assert!(!back_through_history(None, ""));
        assert_eq!(enter_class(Some(true)), "from-next");
        assert_eq!(enter_class(Some(false)), "from-prev");
        assert_eq!(enter_class(None), "");
    }

    #[test]
    fn sizes_the_box_before_loading() {
        assert_eq!(
            fit_style(Some((3600, 2400))),
            "; aspect-ratio: 3600 / 2400; width: min(100%, calc(var(--fit-h) * 3600 / 2400), var(--fit-cap, 3600px))"
        );
        // Unknown or nonsense sizes leave the box to the image.
        assert_eq!(fit_style(None), "");
        assert_eq!(fit_style(Some((0, 10))), "");
        assert_eq!(fit_style(Some((10, -1))), "");
    }

    #[test]
    fn quotes_css_urls() {
        assert_eq!(
            css_url("/media/thumb/a.jpg?x=1"),
            r#"url("/media/thumb/a.jpg?x=1")"#
        );
        assert_eq!(css_url(r#"a"b\c"#), r#"url("a\"b\\c")"#);
    }

    #[test]
    fn swaps_in_the_original_only_when_the_screen_has_more_pixels() {
        let photo = (6000, 4000);
        // A laptop window, and a phone held upright: the sample is enough.
        assert!(sample_is_sharp(photo, (900.0, 740.0), 1.0));
        assert!(sample_is_sharp(photo, (390.0, 520.0), 3.0));
        // Fullscreen on a 4K screen, or a dense laptop's: more than 2560 wide.
        assert!(!sample_is_sharp(photo, (3840.0, 2160.0), 1.0));
        assert!(!sample_is_sharp(photo, (1512.0, 982.0), 2.0));
        // Small enough that the sample keeps every pixel.
        assert!(sample_is_sharp((2400, 1600), (3840.0, 2160.0), 2.0));
        // Shown no bigger than itself: 3000 px shown at most 3000 device px.
        assert!(!sample_is_sharp((3000, 1000), (3000.0, 2000.0), 1.0));
        assert!(sample_is_sharp((3000, 1000), (1200.0, 2000.0), 2.0));
        // Not measured yet, or no size.
        assert!(sample_is_sharp(photo, (0.0, 0.0), 2.0));
        assert!(sample_is_sharp((0, 0), (900.0, 700.0), 1.0));

        let p = post();
        assert!(!needs_original(&p, Some((900.0, 740.0)), Some(1.0)));
        assert!(needs_original(&p, Some((3840.0, 2160.0)), Some(1.0)));
        assert!(!needs_original(&p, None, Some(1.0)));
        let video = Post {
            kind: MediaKind::Video,
            ..p.clone()
        };
        assert!(!needs_original(&video, Some((3840.0, 2160.0)), Some(1.0)));
        assert_eq!(shown_url(&p, false), "/media/sample/39fb.jpg?exp=1&sig=4");
        assert_eq!(shown_url(&p, true), "/media/original/39fb.png?exp=1&sig=2");
        let small = Post {
            sample_url: None,
            ..p
        };
        assert_eq!(
            shown_url(&small, false),
            "/media/original/39fb.png?exp=1&sig=2"
        );
    }

    #[test]
    fn a_finger_has_to_go_somewhere_first() {
        assert_eq!(sideways(3.0, 4.0), None);
        assert_eq!(sideways(12.0, 3.0), Some(true));
        assert_eq!(sideways(-3.0, 12.0), Some(false));
        assert_eq!(sideways(f64::NAN, 0.0), None);
    }

    #[test]
    fn a_swipe_turns_when_far_enough_or_flung() {
        // 22% of the stage, at most 140 px.
        assert_eq!(swipe_turns(-90.0, 0.0, 400.0), Some(true));
        assert_eq!(swipe_turns(90.0, 0.0, 400.0), Some(false));
        assert_eq!(swipe_turns(-80.0, 0.0, 400.0), None);
        assert_eq!(swipe_turns(-139.0, 0.0, 1200.0), None);
        assert_eq!(swipe_turns(-140.0, 0.0, 1200.0), Some(true));
        // Flicked: short, but fast and the same way.
        assert_eq!(swipe_turns(-40.0, -0.8, 1200.0), Some(true));
        assert_eq!(swipe_turns(-40.0, 0.8, 1200.0), None);
        assert_eq!(swipe_turns(-20.0, -2.0, 1200.0), None);
        assert_eq!(swipe_turns(f64::NAN, f64::NAN, 400.0), None);
        assert_eq!(resist(-100.0, true), -25.0);
        assert_eq!(resist(-100.0, false), -100.0);
    }

    #[test]
    fn the_speed_is_the_last_moments() {
        assert_eq!(speed(&[]), 0.0);
        assert_eq!(speed(&[(5.0, 10.0)]), 0.0);
        // Slow at first, then 60 px in the last 60 ms.
        let trail = [(0.0, 0.0), (90.0, 5.0), (140.0, 10.0), (200.0, 70.0)];
        assert_eq!(speed(&trail), 60.0 / 60.0);
    }

    #[test]
    fn two_quick_taps_in_one_place_are_a_double_tap() {
        let tap = |time, x, y| Tap { time, x, y };
        let first = Some(tap(1000.0, 100.0, 100.0));
        assert!(double_tap(first, tap(1250.0, 110.0, 95.0)));
        assert!(!double_tap(first, tap(1400.0, 100.0, 100.0)), "too slow");
        assert!(!double_tap(first, tap(1100.0, 160.0, 100.0)), "too far");
        assert!(!double_tap(None, tap(1100.0, 100.0, 100.0)));
        assert!(
            !double_tap(first, tap(900.0, 100.0, 100.0)),
            "a clock set back"
        );
    }
}
