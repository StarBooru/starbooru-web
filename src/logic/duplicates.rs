//! The duplicate finder's rules: the matching levels, how alike a group
//! reads, which copy to keep, what resolving a group sends, and what puts the
//! kept post back on Undo.

use std::fmt::Display;

use chrono::{DateTime, TimeZone, Utc};
use starbooru_core::{
    DEFAULT_DUPLICATE_DISTANCE, DuplicateGroup, MAX_DUPLICATE_DISTANCE, MAX_DUPLICATE_GROUP, Post,
    PostUpdate, ResolveDuplicates,
};

use super::viewer::dimensions;

/// The matching levels, as the largest distance in bits between two copies.
pub const LEVELS: [u32; 3] = [2, DEFAULT_DUPLICATE_DISTANCE, MAX_DUPLICATE_DISTANCE];

/// Groups asked for at first, and how many more each time.
pub const PAGE: u32 = 30;
/// The most groups the server lists at once.
pub const MAX_LIMIT: u32 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Strict,
    Normal,
    Loose,
}

/// The level a distance belongs to.
pub fn level(distance: u32) -> Level {
    if distance <= 2 {
        Level::Strict
    } else if distance <= DEFAULT_DUPLICATE_DISTANCE {
        Level::Normal
    } else {
        Level::Loose
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Likeness {
    Identical,
    NearlyIdentical,
    VerySimilar,
    Similar,
}

/// How alike a group looks, from the largest distance linking it.
pub fn likeness(distance: u32) -> Likeness {
    match distance {
        0 => Likeness::Identical,
        1..=2 => Likeness::NearlyIdentical,
        3..=6 => Likeness::VerySimilar,
        _ => Likeness::Similar,
    }
}

/// How many of a group's posts are listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Summary {
    /// Every one, so many.
    All(u32),
    /// The first `shown` of `size`: the server lists 50 at most.
    Some { shown: usize, size: u32 },
}

pub fn summary(group: &DuplicateGroup) -> Summary {
    let shown = group.posts.len();
    if usize::try_from(group.size).is_ok_and(|size| size > shown) {
        Summary::Some {
            shown,
            size: group.size,
        }
    } else {
        Summary::All(group.size)
    }
}

/// What a group is known by while it's on the page: its first post.
pub fn group_key(group: &DuplicateGroup) -> Option<i64> {
    group.posts.first().map(|p| p.id)
}

fn pixels(p: &Post) -> i64 {
    let side = |s: Option<i32>| i64::from(s.unwrap_or(0).max(0));
    side(p.width).saturating_mul(side(p.height))
}

/// The copy worth keeping: the most pixels, then the largest file, then the
/// oldest. The server lists a group in this order, but the choice doesn't
/// rely on that.
pub fn suggested_keeper(posts: &[Post]) -> Option<i64> {
    posts
        .iter()
        .max_by(|a, b| {
            pixels(a)
                .cmp(&pixels(b))
                .then(a.file_size.cmp(&b.file_size))
                .then(b.created_at.cmp(&a.created_at))
                .then(b.id.cmp(&a.id))
        })
        .map(|p| p.id)
}

/// What resolving the group of `ids` sends: keep `keep`, bin the rest.
/// `None` when `keep` isn't in the group or nothing else is.
pub fn resolve_request(ids: &[i64], keep: i64, merge_tags: bool) -> Option<ResolveDuplicates> {
    if !ids.contains(&keep) {
        return None;
    }
    let mut remove: Vec<i64> = Vec::new();
    for &id in ids {
        if id != keep && !remove.contains(&id) {
            remove.push(id);
        }
    }
    remove.truncate(MAX_DUPLICATE_GROUP);
    (!remove.is_empty()).then_some(ResolveDuplicates {
        keep,
        remove,
        merge_tags,
    })
}

/// What puts the kept post back as it was before the merge: `before` as the
/// group listed it, `after` as the resolve returned it. `None` when the merge
/// changed nothing.
pub fn unmerge(before: &Post, after: &Post) -> Option<PostUpdate> {
    let set = |tags: &[String]| {
        let mut tags = tags.to_vec();
        tags.sort_unstable();
        tags
    };
    let tags = (set(&before.tags) != set(&after.tags)).then(|| before.tags.clone());
    let has = |p: &Post| p.source.as_deref().is_some_and(|s| !s.trim().is_empty());
    // The merge only fills a missing source; an empty one clears it.
    let source = (!has(before) && has(after)).then(String::new);
    (tags.is_some() || source.is_some()).then_some(PostUpdate {
        tags,
        rating: None,
        source,
    })
}

/// `1400 × 900`, or `None` when the server doesn't know.
pub fn size_of(p: &Post) -> Option<String> {
    matches!((p.width, p.height), (Some(w), Some(h)) if w > 0 && h > 0)
        .then(|| dimensions(p.width, p.height))
}

/// The day a moment fell on in `zone`.
pub fn day<Tz: TimeZone>(at: &DateTime<Utc>, zone: &Tz) -> String
where
    Tz::Offset: Display,
{
    at.with_timezone(zone).format("%Y-%m-%d").to_string()
}

/// How many groups to ask for after "Load more".
pub fn more(limit: u32) -> u32 {
    limit.saturating_add(PAGE).min(MAX_LIMIT)
}

/// Whether the server has groups past the `shown` ones that can still be
/// asked for.
pub fn has_more(shown: usize, total: u64, limit: u32) -> bool {
    u64::try_from(shown).unwrap_or(u64::MAX) < total && limit < MAX_LIMIT
}

#[cfg(test)]
mod tests {
    use chrono::FixedOffset;
    use serde_json::json;

    use super::*;

    fn post(id: i64, width: i32, height: i32, file_size: i64, created: &str) -> Post {
        serde_json::from_value(json!({
            "id": id, "kind": "image", "mime": "image/png", "file_size": file_size,
            "width": width, "height": height, "sha256": "ab", "rating": "safe",
            "score": 0, "my_vote": 0, "source": null, "tags": ["cat"],
            "created_at": created, "deleted": false,
            "file_url": "/media/original/ab.png", "thumb_url": null,
        }))
        .unwrap()
    }

    const EARLY: &str = "2026-01-01T00:00:00Z";
    const LATE: &str = "2026-06-01T00:00:00Z";

    #[test]
    fn the_levels_cover_the_servers_range() {
        assert!(LEVELS.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(LEVELS[1], DEFAULT_DUPLICATE_DISTANCE);
        assert_eq!(LEVELS[2], MAX_DUPLICATE_DISTANCE);
        assert_eq!(level(LEVELS[0]), Level::Strict);
        assert_eq!(level(LEVELS[1]), Level::Normal);
        assert_eq!(level(LEVELS[2]), Level::Loose);
        assert_eq!(level(0), Level::Strict);
    }

    #[test]
    fn a_group_reads_by_its_largest_distance() {
        assert_eq!(likeness(0), Likeness::Identical);
        assert_eq!(likeness(1), Likeness::NearlyIdentical);
        assert_eq!(likeness(2), Likeness::NearlyIdentical);
        assert_eq!(likeness(3), Likeness::VerySimilar);
        assert_eq!(likeness(6), Likeness::VerySimilar);
        assert_eq!(likeness(7), Likeness::Similar);
        assert_eq!(likeness(10), Likeness::Similar);
    }

    #[test]
    fn a_group_says_when_only_some_of_it_is_listed() {
        let posts = vec![post(1, 10, 10, 1, EARLY), post(2, 10, 10, 1, EARLY)];
        let group = |size| DuplicateGroup {
            distance: 0,
            size,
            posts: posts.clone(),
        };
        assert_eq!(summary(&group(2)), Summary::All(2));
        assert_eq!(summary(&group(60)), Summary::Some { shown: 2, size: 60 });
        assert_eq!(group_key(&group(2)), Some(1));
        let empty = DuplicateGroup {
            distance: 0,
            size: 0,
            posts: Vec::new(),
        };
        assert_eq!(group_key(&empty), None);
    }

    #[test]
    fn the_keeper_has_the_most_pixels_then_the_largest_file_then_is_oldest() {
        // More pixels beat a bigger file.
        let posts = [
            post(1, 900, 900, 9_000_000, EARLY),
            post(2, 1400, 1400, 20_000, LATE),
        ];
        assert_eq!(suggested_keeper(&posts), Some(2));
        // The same shape: the larger file.
        let posts = [
            post(3, 1400, 1400, 20_000, EARLY),
            post(4, 1400, 1400, 30_000, LATE),
        ];
        assert_eq!(suggested_keeper(&posts), Some(4));
        // The same file size too: the older one, then the lower id.
        let posts = [
            post(6, 1400, 1400, 30_000, LATE),
            post(5, 1400, 1400, 30_000, EARLY),
        ];
        assert_eq!(suggested_keeper(&posts), Some(5));
        let posts = [
            post(8, 1400, 1400, 30_000, EARLY),
            post(7, 1400, 1400, 30_000, EARLY),
        ];
        assert_eq!(suggested_keeper(&posts), Some(7));
        // A post of unknown size counts as none.
        let mut unknown = post(9, 1, 1, 99_000_000, EARLY);
        unknown.width = None;
        let posts = [unknown, post(10, 10, 10, 1, LATE)];
        assert_eq!(suggested_keeper(&posts), Some(10));
        assert_eq!(suggested_keeper(&[]), None);
    }

    #[test]
    fn resolving_keeps_one_and_bins_the_rest() {
        let req = resolve_request(&[14, 11, 12], 11, false).unwrap();
        assert_eq!(req.keep, 11);
        assert_eq!(req.remove, vec![14, 12]);
        assert!(!req.merge_tags);
        assert!(resolve_request(&[14, 11], 14, true).unwrap().merge_tags);
        // It goes to the server as the API has it.
        assert_eq!(
            serde_json::to_value(resolve_request(&[5, 81], 5, true).unwrap()).unwrap(),
            json!({"keep": 5, "remove": [81], "merge_tags": true})
        );
        // Each post once.
        assert_eq!(
            resolve_request(&[1, 2, 2, 1], 1, false).unwrap().remove,
            vec![2]
        );
        // Not a post of the group, or nothing to bin.
        assert!(resolve_request(&[1, 2], 3, false).is_none());
        assert!(resolve_request(&[1], 1, false).is_none());
        assert!(resolve_request(&[], 1, false).is_none());
        // No more than the server takes.
        let ids: Vec<i64> = (0..80).collect();
        assert_eq!(
            resolve_request(&ids, 0, false).unwrap().remove.len(),
            MAX_DUPLICATE_GROUP
        );
    }

    #[test]
    fn undo_takes_back_what_the_merge_added() {
        let before = post(5, 10, 10, 1, EARLY);
        // Nothing merged: nothing to put back.
        assert!(unmerge(&before, &before).is_none());
        let mut after = before.clone();
        after.tags = vec!["cat".into(), "dog".into()];
        let update = unmerge(&before, &after).unwrap();
        assert_eq!(update.tags, Some(vec!["cat".to_owned()]));
        assert_eq!(update.source, None);
        assert_eq!(update.rating, None);
        // The same tags in another order are the same tags.
        let mut reordered = before.clone();
        reordered.tags = vec!["b".into(), "a".into()];
        let mut sorted = before.clone();
        sorted.tags = vec!["a".into(), "b".into()];
        assert!(unmerge(&reordered, &sorted).is_none());
        // A source it took is cleared; one it had stays.
        let mut sourced = before.clone();
        sourced.source = Some("https://example.com/a".into());
        let update = unmerge(&before, &sourced).unwrap();
        assert_eq!(update.tags, None);
        assert_eq!(update.source.as_deref(), Some(""));
        assert!(unmerge(&sourced, &sourced).is_none());
        let mut blank = before.clone();
        blank.source = Some("  ".into());
        assert_eq!(
            unmerge(&blank, &sourced).and_then(|u| u.source).as_deref(),
            Some("")
        );
    }

    #[test]
    fn each_copy_says_its_shape_and_day() {
        assert_eq!(
            size_of(&post(1, 1400, 900, 1, EARLY)).as_deref(),
            Some("1400 × 900")
        );
        let mut unknown = post(1, 0, 900, 1, EARLY);
        assert_eq!(size_of(&unknown), None);
        unknown.height = None;
        assert_eq!(size_of(&unknown), None);
        let at: DateTime<Utc> = "2026-11-22T23:39:00Z".parse().unwrap();
        assert_eq!(day(&at, &Utc), "2026-11-22");
        let rome = FixedOffset::east_opt(3600).unwrap();
        assert_eq!(day(&at, &rome), "2026-11-23");
    }

    #[test]
    fn more_groups_come_a_page_at_a_time_up_to_the_servers_most() {
        assert_eq!(more(PAGE), 60);
        assert_eq!(more(90), MAX_LIMIT);
        assert_eq!(more(MAX_LIMIT), MAX_LIMIT);
        assert!(has_more(30, 45, 30));
        assert!(!has_more(30, 30, 30));
        // After one was resolved, the rest of the page still shows.
        assert!(has_more(29, 44, 30));
        // The server lists no more than 100.
        assert!(!has_more(100, 140, MAX_LIMIT));
    }
}
