//! The gallery's rules that need no browser: what a search's title and sort
//! are, rewriting the search for a sort or a colour, how long a video runs,
//! the shuffle each `order:random` search keeps, and the results kept while
//! the app runs.

use std::collections::HashSet;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use starbooru_core::color::ColorTerm;
use starbooru_core::{Order, Post, SortKey, parse_query, with_color, with_order};

/// Results come in pages of this many.
pub const PAGE_SIZE: u32 = 60;

/// The sort menu's entries, as `order:` values.
pub const SORTS: [&str; 11] = [
    "new", "old", "score", "updated", "size", "size_asc", "pixels", "longside", "tags", "tags_asc",
    "random",
];

/// The sort menu's entry for search `q`: `new` unless it asks for another
/// order the menu has.
pub fn current_sort(q: &str) -> &'static str {
    let order = parse_query(q).map(|p| p.order).unwrap_or_default();
    SORTS
        .into_iter()
        .find(|v| Order::parse(v) == Some(order))
        .unwrap_or("new")
}

/// Search `q` sorted by menu entry `value`. The default order leaves no
/// `order:` term behind.
pub fn sorted(q: &str, value: &str) -> String {
    with_order(q, Order::parse(value))
}

/// The colour search `q` asks for, if any.
pub fn current_color(q: &str) -> Option<ColorTerm> {
    parse_query(q)
        .ok()
        .and_then(|p| p.colors.into_iter().next())
}

/// Search `q` for colour `value`, a name or hex digits, or for any colour.
pub fn colored(q: &str, value: Option<&str>) -> String {
    with_color(q, value)
}

/// The page's title for search `q`. The sort menu shows the order, so the
/// title leaves it out.
pub fn title(q: &str) -> String {
    with_order(q, None)
}

/// The tag a search for one plain tag is for, whose page the gallery links
/// to.
pub fn single_tag(title: &str) -> Option<&str> {
    let plain =
        !title.is_empty() && !title.contains([' ', ':', '{', '}']) && !title.starts_with('-');
    plain.then_some(title)
}

/// `order:random` comes out different each time, so it has no spot to go back to.
pub fn is_random(q: &str) -> bool {
    parse_query(q).is_ok_and(|p| p.order.key == SortKey::Random)
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

/// A post's main colour for its tile, if it is `#rrggbb` and nothing else: it
/// goes into a style attribute as it comes from the server.
pub fn tint(palette: &[String]) -> Option<&str> {
    let colour = palette.first()?;
    let hex = colour.strip_prefix('#')?;
    (hex.len() == 6 && hex.bytes().all(|b| b.is_ascii_hexdigit())).then_some(colour.as_str())
}

const MAX_SEEDS: usize = 16;

/// The seed each `order:random` search was last shuffled with, the latest
/// first. Paging and the viewer send the same one, so they follow the order
/// the gallery shows.
#[derive(Debug, Default)]
pub struct Seeds(Vec<(String, i64)>);

impl Seeds {
    pub fn get(&self, q: &str) -> Option<i64> {
        self.0.iter().find(|(k, _)| k == q).map(|(_, s)| *s)
    }

    pub fn put(&mut self, q: &str, seed: i64) {
        self.0.retain(|(k, _)| k != q);
        self.0.insert(0, (q.to_owned(), seed));
        self.0.truncate(MAX_SEEDS);
    }
}

/// A seed for a new shuffle, from the clock and a count of seeds so far. It
/// only has to shuffle differently, not be hard to guess. Mixed (splitmix64)
/// so seeds a moment apart don't come out alike.
pub fn mix_seed(nanos: i64, count: u64) -> i64 {
    let mut x = u64::from_ne_bytes(nanos.to_ne_bytes()) ^ count.rotate_left(32);
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    i64::from_ne_bytes((x ^ (x >> 31)).to_ne_bytes())
}

/// The first page as it is now, followed by the rest of what was loaded. `None`
/// if nothing changed. With no overlap, or no further pages, the rest is gone.
pub fn merge_head(
    current: &[Arc<Post>],
    head: Vec<Post>,
    has_more: bool,
) -> Option<Vec<Arc<Post>>> {
    let same = current
        .iter()
        .map(|p| p.id)
        .take(head.len())
        .eq(head.iter().map(|p| p.id));
    if same && (has_more || head.len() == current.len()) {
        return None;
    }
    let ids: HashSet<i64> = head.iter().map(|p| p.id).collect();
    let rest = current
        .iter()
        .rposition(|p| ids.contains(&p.id))
        .filter(|_| has_more)
        .and_then(|last| current.get(last + 1..))
        .unwrap_or_default()
        .to_vec();
    Some(head.into_iter().map(Arc::new).chain(rest).collect())
}

/// Results kept while the app runs, so coming back to a search shows it as it
/// was without loading it again.
#[derive(Clone, Debug)]
pub struct List {
    pub q: String,
    /// Whose results they are: one account on one server.
    pub owner: String,
    pub posts: Vec<Arc<Post>>,
    pub next_page: u32,
    pub has_more: bool,
    pub loaded_at: DateTime<Utc>,
    /// What `order:random` was shuffled with, so the list stays as it was.
    pub seed: Option<i64>,
}

const MAX_LISTS: usize = 4;
/// Signed media links last at least six hours (`docs/architecture.md`).
const MAX_LIST_HOURS: i64 = 4;

#[derive(Debug, Default)]
pub struct Lists(Vec<List>);

impl Lists {
    /// The results kept for search `q` and this account, unless they're old
    /// enough for their media links to run out.
    pub fn find(&self, q: &str, owner: &str, now: DateTime<Utc>) -> Option<List> {
        self.0
            .iter()
            .find(|l| {
                let age = (now - l.loaded_at).num_hours();
                l.q == q && l.owner == owner && (0..MAX_LIST_HOURS).contains(&age)
            })
            .cloned()
    }

    pub fn keep(&mut self, list: List) {
        self.0.retain(|l| l.q != list.q);
        self.0.insert(0, list);
        self.0.truncate(MAX_LISTS);
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

/// How many posts going back to a spot loads, at most, before it settles for
/// the scroll position. Only their details: pictures load near the screen.
pub const MAX_RESTORE: u32 = 100_000;
/// Going back to a spot asks for this many pages of results at a time.
pub const RESTORE_PAGES: u32 = 3;

/// The page number and size of the next request on the way back to a spot,
/// after `done` of them: pages of `RESTORE_PAGES` ordinary ones.
pub fn restore_request(done: u32) -> (u32, u32) {
    (done.saturating_add(1), PAGE_SIZE * RESTORE_PAGES)
}

/// The ordinary page that comes after `done` requests of `restore_request`.
pub fn page_after_restore(done: u32) -> u32 {
    done.saturating_mul(RESTORE_PAGES).saturating_add(1)
}

/// How long the page has to be still before a tile under the pointer lights
/// up again, in milliseconds.
pub const LIGHT_MS: i64 = 120;
/// How long the page has to be still before the gallery remembers its spot.
pub const SPOT_MS: i64 = 200;

/// What to do once the page has been still for `still` milliseconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rest {
    /// Light the tile under the pointer again (it was `moving`).
    pub calm: bool,
    /// Remember the spot.
    pub report: bool,
    /// Look again after this many milliseconds.
    pub wait: Option<i64>,
}

/// One timer that looks at how long ago the page last moved, in place of one
/// set and cleared on every scroll event.
pub fn rest(still: i64, moving: bool) -> Rest {
    let calm = moving && still >= LIGHT_MS;
    if still >= SPOT_MS {
        return Rest {
            calm,
            report: true,
            wait: None,
        };
    }
    let next = if still < LIGHT_MS { LIGHT_MS } else { SPOT_MS };
    Rest {
        calm,
        report: false,
        wait: Some(next - still.max(0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_light_and_the_spot_wait_for_the_page_to_stop() {
        assert_eq!(
            rest(0, true),
            Rest {
                calm: false,
                report: false,
                wait: Some(120)
            }
        );
        assert_eq!(
            rest(130, true),
            Rest {
                calm: true,
                report: false,
                wait: Some(70)
            }
        );
        assert_eq!(
            rest(250, true),
            Rest {
                calm: true,
                report: true,
                wait: None
            }
        );
        // Already calm: nothing to light again.
        assert!(!rest(250, false).calm);
        // A clock that went back waits the whole time again.
        assert_eq!(rest(-5, true).wait, Some(120));
    }

    fn post(id: i64) -> Post {
        serde_json::from_value(serde_json::json!({
            "id": id, "kind": "image", "mime": "image/png", "file_size": 1,
            "width": 1, "height": 1, "sha256": "", "rating": "general", "score": 0,
            "my_vote": 0, "source": null, "tags": [],
            "created_at": "2026-01-01T00:00:00Z", "deleted": false,
            "file_url": "", "thumb_url": null,
        }))
        .unwrap()
    }

    fn merged(current: &[i64], head: &[i64], has_more: bool) -> Option<Vec<i64>> {
        let current: Vec<Arc<Post>> = current.iter().copied().map(post).map(Arc::new).collect();
        let head = head.iter().copied().map(post).collect();
        merge_head(&current, head, has_more).map(|l| l.iter().map(|p| p.id).collect())
    }

    #[test]
    fn nothing_new_changes_nothing() {
        assert_eq!(merged(&[5, 4, 3, 2, 1], &[5, 4], true), None);
        assert_eq!(merged(&[5, 4], &[5, 4], false), None);
    }

    #[test]
    fn new_posts_go_on_top_of_what_was_loaded() {
        assert_eq!(
            merged(&[5, 4, 3, 2, 1], &[7, 6], true),
            Some(vec![7, 6]),
            "no overlap: a whole page newer, so the rest is out of step",
        );
        assert_eq!(
            merged(&[5, 4, 3, 2, 1], &[6, 5], true),
            Some(vec![6, 5, 4, 3, 2, 1]),
        );
    }

    #[test]
    fn posts_gone_from_the_first_page_go() {
        assert_eq!(
            merged(&[5, 4, 3, 2, 1], &[5, 3], true),
            Some(vec![5, 3, 2, 1]),
        );
        // No further pages: the first page is the whole list.
        assert_eq!(merged(&[5, 4, 3], &[5, 4], false), Some(vec![5, 4]));
        assert_eq!(merged(&[5, 4, 3], &[], false), Some(vec![]));
    }

    #[test]
    fn video_lengths_read_as_on_a_player() {
        assert_eq!(format_duration(0).as_deref(), Some("0:00"));
        assert_eq!(format_duration(1_499).as_deref(), Some("0:01"));
        assert_eq!(format_duration(1_500).as_deref(), Some("0:02"));
        assert_eq!(format_duration(65_000).as_deref(), Some("1:05"));
        assert_eq!(format_duration(599_000).as_deref(), Some("9:59"));
        assert_eq!(format_duration(3_600_000).as_deref(), Some("1:00:00"));
        assert_eq!(format_duration(3_725_000).as_deref(), Some("1:02:05"));
        assert_eq!(format_duration(-1), None);
        assert!(format_duration(i64::MAX).is_some());
    }

    #[test]
    fn only_a_plain_colour_tints_a_tile() {
        let one = |c: &str| vec![c.to_owned()];
        assert_eq!(tint(&one("#41ab78")), Some("#41ab78"));
        assert_eq!(tint(&one("#41AB78")), Some("#41AB78"));
        assert_eq!(tint(&[]), None);
        for bad in [
            "41ab78",
            "#41ab7",
            "#41ab788",
            "#41ab7g",
            "red",
            "#fff; background: url(//x)",
            "#àààààà",
            "",
        ] {
            assert_eq!(tint(&one(bad)), None, "{bad}");
        }
    }

    #[test]
    fn the_sort_menu_shows_the_searchs_order() {
        assert_eq!(current_sort(""), "new");
        assert_eq!(current_sort("cat"), "new");
        assert_eq!(current_sort("cat order:score"), "score");
        assert_eq!(current_sort("order:oldest cat"), "old");
        assert_eq!(current_sort("sort:size_asc"), "size_asc");
        assert_eq!(current_sort("order:random"), "random");
        // An order the menu doesn't list, or a search that doesn't parse.
        assert_eq!(current_sort("order:score_asc"), "new");
        assert_eq!(current_sort("cat {"), "new");
        for value in SORTS {
            assert_eq!(current_sort(&sorted("cat", value)), value, "{value}");
        }
    }

    #[test]
    fn a_sort_rewrites_only_the_order() {
        assert_eq!(sorted("cat dog", "score"), "cat dog order:score");
        assert_eq!(sorted("cat order:score dog", "old"), "cat dog order:old");
        assert_eq!(sorted("cat sort:size", "random"), "cat order:random");
        // The default order needs no term.
        assert_eq!(sorted("cat order:score", "new"), "cat");
        assert_eq!(sorted("", "new"), "");
        assert_eq!(sorted("", "tags_asc"), "order:tags_asc");
    }

    #[test]
    fn a_colour_rewrites_only_the_colour() {
        assert_eq!(colored("cat", Some("red")), "cat color:red");
        assert_eq!(colored("cat color:red", Some("2060e0")), "cat color:2060e0");
        assert_eq!(colored("colour:blue cat", None), "cat");
        assert_eq!(
            current_color("cat color:red").map(|c| c.raw),
            Some("red".into())
        );
        assert_eq!(
            current_color("color:2060E0").map(|c| c.rgb),
            Some([0x20, 0x60, 0xe0])
        );
        assert_eq!(current_color("cat"), None);
    }

    #[test]
    fn the_title_leaves_out_the_order_and_names_a_single_tag() {
        assert_eq!(title("cat order:score"), "cat");
        assert_eq!(title("order:random"), "");
        assert_eq!(single_tag(&title("blue_sky order:old")), Some("blue_sky"));
        for not_one in [
            "",
            "cat dog",
            "rating:safe",
            "-cat",
            "artist:lavfi",
            "{cat}",
        ] {
            assert_eq!(single_tag(not_one), None, "{not_one}");
        }
    }

    #[test]
    fn random_order_has_no_spot() {
        assert!(is_random("cat order:random"));
        assert!(is_random("sort:random"));
        assert!(!is_random("cat"));
        assert!(!is_random("order:new"));
        assert!(!is_random("order:nonsense"));
    }

    #[test]
    fn each_search_keeps_its_shuffle_and_old_ones_drop_off() {
        let mut seeds = Seeds::default();
        assert_eq!(seeds.get("order:random"), None);
        seeds.put("order:random", 7);
        seeds.put("cat order:random", 9);
        assert_eq!(seeds.get("order:random"), Some(7));
        seeds.put("order:random", 8);
        assert_eq!(seeds.get("order:random"), Some(8));
        assert_eq!(seeds.get("cat order:random"), Some(9));
        for i in 0..MAX_SEEDS {
            seeds.put(&format!("t{i} order:random"), 1);
        }
        assert_eq!(seeds.get("order:random"), None, "the oldest went");
        assert_eq!(seeds.get("t0 order:random"), Some(1));
    }

    #[test]
    fn seeds_a_moment_apart_come_out_unalike() {
        let seeds: HashSet<i64> = (0..100).map(|i| mix_seed(1_000_000 + i, 0)).collect();
        assert_eq!(seeds.len(), 100);
        let counted: HashSet<i64> = (0..100).map(|i| mix_seed(5, i)).collect();
        assert_eq!(counted.len(), 100);
    }

    fn list(q: &str, owner: &str, at: DateTime<Utc>) -> List {
        List {
            q: q.into(),
            owner: owner.into(),
            posts: vec![Arc::new(post(1))],
            next_page: 2,
            has_more: true,
            loaded_at: at,
            seed: None,
        }
    }

    #[test]
    fn kept_results_are_for_their_account_and_go_stale() {
        let now = Utc::now();
        let mut lists = Lists::default();
        lists.keep(list("cat", "a|1", now));
        assert!(lists.find("cat", "a|1", now).is_some());
        assert!(lists.find("cat", "a|2", now).is_none(), "another account");
        assert!(lists.find("dog", "a|1", now).is_none());
        let later = now + chrono::Duration::hours(MAX_LIST_HOURS);
        assert!(
            lists.find("cat", "a|1", later).is_none(),
            "links running out"
        );
        let earlier = now - chrono::Duration::hours(1);
        assert!(
            lists.find("cat", "a|1", earlier).is_none(),
            "a clock set back"
        );
        for q in ["a", "b", "c", "d"] {
            lists.keep(list(q, "a|1", now));
        }
        assert!(
            lists.find("cat", "a|1", now).is_none(),
            "only the latest few"
        );
        assert!(lists.find("a", "a|1", now).is_some());
        lists.clear();
        assert!(lists.find("d", "a|1", now).is_none());
    }

    #[test]
    fn going_back_loads_in_big_pages_and_carries_on_after_them() {
        assert_eq!(restore_request(0), (1, 180));
        assert_eq!(restore_request(1), (2, 180));
        assert_eq!(restore_request(2), (3, 180));
        // Two requests of three pages each: the next ordinary page is the 7th.
        assert_eq!(page_after_restore(0), 1);
        assert_eq!(page_after_restore(2), 7);
        assert_eq!(page_after_restore(u32::MAX), u32::MAX);
    }
}
