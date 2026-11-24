//! The account's saved searches, the gallery's and the boorus': each found by
//! its search however that was written, sorted the way the user chose, and
//! labelled by what sets it apart from the ones like it.

use std::fmt::Debug;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use starbooru_core::saved::MAX_SEARCH_NAME_CHARS;
use starbooru_core::{RemoteSite, SavedRemoteSearch, SavedSearch, parse_query};

use super::boorus::{same_sites, search_name, site_name};

/// A search as it is saved: the server collapses the spaces too.
pub fn tidy(q: &str) -> String {
    q.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// What `q` finds, as a string to compare: two ways of writing one search
/// give the same key. Spaces, the order of the terms, a tag's case or
/// category and a repeated term don't change it. A search the parser turns
/// down is its tidy form.
pub fn search_key(q: &str) -> String {
    let Ok(mut p) = parse_query(q) else {
        return tidy(q);
    };
    for figure in p.figures.iter_mut().chain(p.exclude_figures.iter_mut()) {
        figure.tags.sort();
    }
    sort(&mut p.include);
    sort(&mut p.exclude);
    sort(&mut p.ratings);
    sort(&mut p.exclude_ratings);
    sort(&mut p.text);
    sort(&mut p.exclude_text);
    sort(&mut p.figures);
    sort(&mut p.exclude_figures);
    sort(&mut p.dates);
    sort(&mut p.exclude_dates);
    sort(&mut p.sizes);
    sort(&mut p.exclude_sizes);
    sort(&mut p.colors);
    sort(&mut p.exclude_colors);
    format!("{p:?}")
}

/// The terms in one order, whatever order they were typed in. Not all of
/// them can be compared, but all can be printed.
fn sort<T: Debug>(terms: &mut [T]) {
    terms.sort_by_cached_key(|t| format!("{t:?}"));
}

/// The saved search that runs `q`, on `sites` for a booru search, if any.
/// The boorus' own syntax isn't ours: theirs match as typed.
pub fn find(list: &[Item], q: &str, sites: &[RemoteSite]) -> Option<i64> {
    let key = search_key(q);
    list.iter()
        .find(|s| match s.kind {
            Kind::Gallery => search_key(&s.query) == key,
            Kind::Boorus => tidy(&s.query) == tidy(q) && same_sites(&s.sites, sites),
        })
        .map(|s| s.id)
}

/// The name a search is saved under: the search itself, cut to the longest
/// name the server takes.
pub fn name_for(q: &str) -> String {
    let name: String = tidy(q).chars().take(MAX_SEARCH_NAME_CHARS).collect();
    name.trim_end().to_owned()
}

/// Puts `saved` in the list, in its place by name, in place of one with the
/// same id.
pub fn add(list: &mut Vec<Item>, saved: Item) {
    list.retain(|s| s.id != saved.id);
    let key = saved.name.to_lowercase();
    let at = list.partition_point(|s| s.name.to_lowercase() <= key);
    list.insert(at, saved);
}

/// Takes saved search `id` out of the list, giving it back.
pub fn remove(list: &mut Vec<Item>, id: i64) -> Option<Item> {
    let at = list.iter().position(|s| s.id == id)?;
    Some(list.remove(at))
}

// ---------- both kinds, as the lists show them ----------

/// Whose saved searches: the gallery's, or the booru browser's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Gallery,
    Boorus,
}

/// A saved search of either kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub kind: Kind,
    pub id: i64,
    pub name: String,
    pub query: String,
    /// Where a booru search goes; none for the gallery's.
    pub sites: Vec<RemoteSite>,
    pub created_at: DateTime<Utc>,
}

impl From<SavedSearch> for Item {
    fn from(s: SavedSearch) -> Self {
        Self {
            kind: Kind::Gallery,
            id: s.id,
            name: s.name,
            query: s.query,
            sites: Vec::new(),
            created_at: s.created_at,
        }
    }
}

impl From<SavedRemoteSearch> for Item {
    fn from(s: SavedRemoteSearch) -> Self {
        Self {
            kind: Kind::Boorus,
            id: s.id,
            name: s.name,
            query: s.query,
            sites: s.sites,
            created_at: s.created_at,
        }
    }
}

impl Item {
    /// Whether the user gave it a name of its own, rather than the one it
    /// was saved under.
    pub fn named(&self) -> bool {
        let given = match self.kind {
            Kind::Gallery => name_for(&self.query),
            Kind::Boorus => search_name(&self.query, &self.sites),
        };
        tidy(&self.name) != given
    }

    /// Its sites by name, for a booru search.
    pub fn site_names(&self) -> String {
        self.sites
            .iter()
            .map(|s| site_name(&s.base))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// How many saved searches the search box offers while it's empty.
pub const OFFERED: usize = 8;
/// How many the row above a grid shows; the rest are in the full list.
pub const IN_ROW: usize = 12;
/// How many uses are remembered for sorting by last use.
const RECENT: usize = 200;

/// How the saved searches are sorted, everywhere they show.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    /// The ones used last first, then the newest.
    #[default]
    Recent,
    Newest,
    Name,
    /// The order the user put them in.
    Mine,
}

impl Sort {
    pub const ALL: [Sort; 4] = [Sort::Recent, Sort::Newest, Sort::Name, Sort::Mine];

    pub fn as_str(self) -> &'static str {
        match self {
            Sort::Recent => "recent",
            Sort::Newest => "newest",
            Sort::Name => "name",
            Sort::Mine => "mine",
        }
    }

    pub fn of(s: &str) -> Option<Sort> {
        Sort::ALL.into_iter().find(|o| o.as_str() == s)
    }
}

/// What the app keeps about one kind's saved searches, by id: the order the
/// user put them in, and the ones used last, the latest first.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Placed {
    pub order: Vec<i64>,
    pub recent: Vec<i64>,
}

impl Placed {
    /// Saved search `id` was just used.
    pub fn used(&mut self, id: i64) {
        self.recent.retain(|r| *r != id);
        self.recent.insert(0, id);
        self.recent.truncate(RECENT);
    }

    /// Saved search `old` is now `new`, as renaming does: it keeps its places.
    pub fn renamed(&mut self, old: i64, new: i64) {
        self.order = replace_id(&self.order, old, new);
        self.recent = replace_id(&self.recent, old, new);
    }
}

fn newest_first(a: &Item, b: &Item) -> std::cmp::Ordering {
    b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id))
}

/// `list` sorted by `sort`. In the user's own order, ones saved since they
/// last moved any come first, newest first, so a new one is in sight.
pub fn sorted(list: &[Item], sort: Sort, placed: &Placed) -> Vec<Item> {
    let mut rest: Vec<Item> = list.to_vec();
    rest.sort_by(newest_first);
    let pick = |ids: &[i64], rest: &mut Vec<Item>| -> Vec<Item> {
        let out: Vec<Item> = ids
            .iter()
            .filter_map(|id| rest.iter().find(|s| s.id == *id))
            .cloned()
            .collect();
        rest.retain(|s| !ids.contains(&s.id));
        out
    };
    match sort {
        Sort::Newest => rest,
        Sort::Name => {
            rest.sort_by(|a, b| {
                a.name
                    .to_lowercase()
                    .cmp(&b.name.to_lowercase())
                    .then(a.id.cmp(&b.id))
            });
            rest
        }
        Sort::Recent => {
            let mut out = pick(&placed.recent, &mut rest);
            out.append(&mut rest);
            out
        }
        Sort::Mine => {
            let mut ordered = pick(&placed.order, &mut rest);
            rest.append(&mut ordered);
            rest
        }
    }
}

/// Whether `item` has every word of `filter` in its name, its search or its
/// sites' names.
pub fn matches(item: &Item, filter: &str) -> bool {
    let within = format!("{} {} {}", item.name, item.query, item.site_names()).to_lowercase();
    filter
        .split_whitespace()
        .all(|word| within.contains(&word.to_lowercase()))
}

// ---------- what sets each one apart ----------

/// The terms of search `q`, as typed: a quoted text or a `{…}` figure is one
/// term, spaces and all.
pub fn terms(q: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut term = String::new();
    let (mut quoted, mut depth) = (false, 0_u32);
    for c in q.chars() {
        match c {
            '"' => {
                quoted = !quoted;
                term.push(c);
            }
            '{' if !quoted => {
                depth = depth.saturating_add(1);
                term.push(c);
            }
            '}' if !quoted => {
                depth = depth.saturating_sub(1);
                term.push(c);
            }
            c if c.is_whitespace() => {
                if quoted || depth > 0 {
                    if !term.ends_with(' ') {
                        term.push(' ');
                    }
                } else if !term.is_empty() {
                    out.push(std::mem::take(&mut term));
                }
            }
            c => term.push(c),
        }
    }
    if !term.is_empty() {
        out.push(term);
    }
    out
}

/// A saved search's label: what sets it apart first, then what it shares
/// with the one most like it, which shows dimmed. `rest` is empty when
/// nothing is shared, or the name is the user's own.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Label {
    pub lead: String,
    pub rest: String,
}

/// Fewest terms two searches share before the shared ones step back. One
/// term in common isn't enough to call them alike.
const ALIKE: usize = 2;

/// For each search of `searches` (their terms), the terms that set it apart
/// and the ones it shares with the search most like it, each in the order
/// typed. Only a search that adds something to that one is split: the one
/// the others build on, and one with nothing of its own, show whole.
pub fn differences(searches: &[Vec<String>]) -> Vec<(Vec<String>, Vec<String>)> {
    let lower: Vec<Vec<String>> = searches
        .iter()
        .map(|t| t.iter().map(|w| w.to_lowercase()).collect())
        .collect();
    lower
        .iter()
        .enumerate()
        .map(|(i, mine)| {
            let shared_with =
                |other: &Vec<String>| mine.iter().filter(|t| other.contains(t)).count();
            let best = lower
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, other)| (shared_with(other), other))
                .filter(|(n, _)| *n >= ALIKE && *n < mine.len())
                .fold(
                    None::<(usize, &Vec<String>)>,
                    |best, (n, other)| match best {
                        Some((b, _)) if b >= n => best,
                        _ => Some((n, other)),
                    },
                );
            let typed = searches.get(i).cloned().unwrap_or_default();
            match best {
                Some((_, other)) => typed
                    .into_iter()
                    .partition(|t| !other.contains(&t.to_lowercase())),
                None => (typed, Vec::new()),
            }
        })
        .collect()
}

/// The labels of `list`, in its order. The searches named after themselves
/// are told apart by their terms; a name of the user's own shows as it is.
pub fn labels(list: &[Item]) -> Vec<Label> {
    let plain: Vec<usize> = (0..list.len())
        .filter(|i| {
            list.get(*i)
                .is_some_and(|s| !s.named() && !s.query.trim().is_empty())
        })
        .collect();
    let searches: Vec<Vec<String>> = plain
        .iter()
        .filter_map(|i| list.get(*i).map(|s| terms(&s.query)))
        .collect();
    let split = differences(&searches);
    list.iter()
        .enumerate()
        .map(|(i, s)| {
            match plain
                .iter()
                .position(|p| *p == i)
                .and_then(|at| split.get(at))
            {
                Some((lead, rest)) => Label {
                    lead: lead.join(" "),
                    rest: rest.join(" "),
                },
                None => Label {
                    lead: s.name.clone(),
                    rest: String::new(),
                },
            }
        })
        .collect()
}

// ---------- moving and renaming ----------

/// The ids of `shown` with `id` moved to place `to`, or `None` when it
/// isn't there or is there already.
pub fn move_to(shown: &[i64], id: i64, to: usize) -> Option<Vec<i64>> {
    let from = shown.iter().position(|s| *s == id)?;
    let to = to.min(shown.len().saturating_sub(1));
    if from == to {
        return None;
    }
    let mut out = shown.to_vec();
    let moved = out.remove(from);
    out.insert(to, moved);
    Some(out)
}

/// Where a search dragged from place `from` ends up when dropped before the
/// one at place `before` (the length of the list for after the last).
pub fn landing(from: usize, before: usize) -> usize {
    if from < before { before - 1 } else { before }
}

/// The ids of `shown` with `id` one place earlier (`later` false) or later.
pub fn step(shown: &[i64], id: i64, later: bool) -> Option<Vec<i64>> {
    let from = shown.iter().position(|s| *s == id)?;
    let to = if later {
        from.checked_add(1).filter(|t| *t < shown.len())?
    } else {
        from.checked_sub(1)?
    };
    move_to(shown, id, to)
}

/// `order` with saved search `old` become `new`, as renaming does.
pub fn replace_id(order: &[i64], old: i64, new: i64) -> Vec<i64> {
    order
        .iter()
        .map(|id| if *id == old { new } else { *id })
        .collect()
}

/// How a saved search is renamed, the server having no rename of its own: a
/// copy under the new name, then the old one goes. Only the case changing
/// clashes with the old name, which then goes first; the boorus' searches
/// may share a name, so theirs never clash.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rename {
    SaveFirst,
    RemoveFirst,
}

/// How to rename `old` to `typed`, or `None` when there's nothing to do.
pub fn rename(kind: Kind, old: &str, typed: &str) -> Option<Rename> {
    let new = tidy(typed);
    if new.is_empty() || new == old {
        None
    } else if kind == Kind::Gallery && new.to_lowercase() == old.to_lowercase() {
        Some(Rename::RemoveFirst)
    } else {
        Some(Rename::SaveFirst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Duration;
    use starbooru_core::BooruApi;

    fn ids(list: &[Item]) -> Vec<i64> {
        list.iter().map(|s| s.id).collect()
    }

    /// Item `id`, saved `id` minutes after the first.
    fn item(id: i64, name: &str, query: &str) -> Item {
        let first = DateTime::<Utc>::UNIX_EPOCH;
        Item::from(SavedSearch {
            id,
            name: name.into(),
            query: query.into(),
            created_at: first + Duration::minutes(id),
        })
    }

    fn remote(id: i64, name: &str, query: &str, sites: &[&str]) -> Item {
        Item::from(SavedRemoteSearch {
            id,
            name: name.into(),
            query: query.into(),
            sites: sites
                .iter()
                .map(|base| RemoteSite {
                    api: BooruApi::Danbooru,
                    base: (*base).into(),
                })
                .collect(),
            created_at: DateTime::<Utc>::UNIX_EPOCH,
        })
    }

    #[test]
    fn each_sort_puts_them_in_its_order() {
        let list = vec![
            item(1, "eel", "eel"),
            item(2, "Cat", "cat"),
            item(3, "dog", "dog"),
            item(4, "abstract", "abstract"),
        ];
        let none = Placed::default();
        assert_eq!(ids(&sorted(&list, Sort::Newest, &none)), [4, 3, 2, 1]);
        assert_eq!(ids(&sorted(&list, Sort::Name, &none)), [4, 2, 3, 1]);
        // Never used: the newest first.
        assert_eq!(ids(&sorted(&list, Sort::Recent, &none)), [4, 3, 2, 1]);
        assert_eq!(ids(&sorted(&list, Sort::Mine, &none)), [4, 3, 2, 1]);
        let mut placed = Placed {
            order: vec![1, 9, 3],
            recent: Vec::new(),
        };
        placed.used(2);
        placed.used(1);
        placed.used(2);
        assert_eq!(placed.recent, [2, 1]);
        assert_eq!(ids(&sorted(&list, Sort::Recent, &placed)), [2, 1, 4, 3]);
        // Ids of searches since removed are passed over, and ones the user
        // hasn't placed yet come first.
        assert_eq!(ids(&sorted(&list, Sort::Mine, &placed)), [4, 2, 1, 3]);
        placed.renamed(1, 8);
        assert_eq!(placed.order, [8, 9, 3]);
        assert_eq!(placed.recent, [2, 8]);
        for id in 0..300 {
            placed.used(id);
        }
        assert_eq!(placed.recent.len(), RECENT);
        assert_eq!(placed.recent.first(), Some(&299));
        for sort in Sort::ALL {
            assert_eq!(Sort::of(sort.as_str()), Some(sort));
        }
        assert_eq!(Sort::of("best"), None);
    }

    #[test]
    fn a_filter_finds_a_search_by_any_of_its_words() {
        let mine = item(1, "Wallpapers", "highres horizontal_image -lowres");
        assert!(matches(&mine, "wall"));
        assert!(matches(&mine, "  HORIZONTAL  wall "));
        assert!(matches(&mine, ""));
        assert!(!matches(&mine, "wall cat"));
        let theirs = remote(2, "fox", "fox solo", &["https://e926.net"]);
        assert!(matches(&theirs, "e926 solo"));
        assert!(!matches(&theirs, "danbooru"));
    }

    #[test]
    fn terms_keep_quotes_and_figures_whole() {
        assert_eq!(
            terms("  cat  text:\"two   words\" {girl  boots} -dog "),
            ["cat", "text:\"two words\"", "{girl boots}", "-dog"]
        );
        assert!(terms("   ").is_empty());
    }

    fn split(list: &[&str]) -> Vec<(String, String)> {
        let searches: Vec<Vec<String>> = list.iter().map(|q| terms(q)).collect();
        differences(&searches)
            .into_iter()
            .map(|(lead, rest)| (lead.join(" "), rest.join(" ")))
            .collect()
    }

    #[test]
    fn alike_searches_lead_with_what_sets_them_apart() {
        let pairs = split(&[
            "rating:safe fractal",
            "rating:safe fractal monochrome",
            "rating:safe Fractal mandelbrot",
            "mandelbrot rating:safe fractal highres",
            "game_of_life glider",
            "fractal",
            "rating:safe fractal",
        ]);
        let want = [
            // What the others build on shows whole, and so does its double.
            ("rating:safe fractal", ""),
            ("monochrome", "rating:safe fractal"),
            // In the order typed, whatever the case.
            ("mandelbrot", "rating:safe Fractal"),
            // Set against the one most like it.
            ("highres", "mandelbrot rating:safe fractal"),
            // One term in common isn't enough.
            ("game_of_life glider", ""),
            ("fractal", ""),
            ("rating:safe fractal", ""),
        ];
        for (got, (lead, rest)) in pairs.iter().zip(want) {
            assert_eq!((got.0.as_str(), got.1.as_str()), (lead, rest));
        }
        assert_eq!(pairs.len(), want.len());
        // Alone, or with nothing alike, nothing steps back.
        assert_eq!(split(&["a b c"]), [("a b c".into(), String::new())]);
        assert!(split(&[]).is_empty());
    }

    #[test]
    fn a_name_of_the_users_own_shows_as_it_is() {
        let list = vec![
            item(1, "hd test_pattern", "hd test_pattern"),
            item(2, "hd test_pattern smpte", "hd  test_pattern smpte"),
            item(3, "Wallpapers", "hd test_pattern color_bars"),
            remote(4, "Safebooru", "", &["https://safebooru.org"]),
            remote(5, "fox", "fox", &["https://e926.net"]),
            remote(6, "fox solo", "fox solo", &["https://e926.net"]),
        ];
        assert!(!list.iter().any(|s| s.named() && s.id != 3));
        assert!(list.get(2).is_some_and(Item::named));
        let got: Vec<(String, String)> = labels(&list)
            .into_iter()
            .map(|l| (l.lead, l.rest))
            .collect();
        assert_eq!(
            got,
            [
                ("hd test_pattern".into(), String::new()),
                ("smpte".into(), "hd test_pattern".into()),
                // Not set against: its terms don't show.
                ("Wallpapers".into(), String::new()),
                ("Safebooru".into(), String::new()),
                ("fox".into(), String::new()),
                ("fox solo".into(), String::new()),
            ]
        );
        assert_eq!(
            list.get(3).map(Item::site_names).as_deref(),
            Some("Safebooru")
        );
    }

    #[test]
    fn a_search_moves_within_the_list() {
        let shown = [1, 2, 3, 4];
        assert_eq!(move_to(&shown, 1, 2), Some(vec![2, 3, 1, 4]));
        assert_eq!(move_to(&shown, 4, 0), Some(vec![4, 1, 2, 3]));
        assert_eq!(move_to(&shown, 3, 99), Some(vec![1, 2, 4, 3]));
        assert_eq!(move_to(&shown, 2, 1), None);
        assert_eq!(move_to(&shown, 9, 0), None);
        assert_eq!(step(&shown, 2, false), Some(vec![2, 1, 3, 4]));
        assert_eq!(step(&shown, 2, true), Some(vec![1, 3, 2, 4]));
        assert_eq!(step(&shown, 1, false), None);
        assert_eq!(step(&shown, 4, true), None);
        assert_eq!(step(&[], 4, true), None);
        // Dragged from the front to before the fourth, it's third; dragged
        // back to before the first, it's first; after the last, it's last.
        assert_eq!(move_to(&shown, 1, landing(0, 3)), Some(vec![2, 3, 1, 4]));
        assert_eq!(move_to(&shown, 3, landing(2, 0)), Some(vec![3, 1, 2, 4]));
        assert_eq!(move_to(&shown, 2, landing(1, 4)), Some(vec![1, 3, 4, 2]));
        assert_eq!(move_to(&shown, 2, landing(1, 2)), None);
        assert_eq!(replace_id(&[3, 1, 2], 1, 8), [3, 8, 2]);
        assert_eq!(replace_id(&[3, 2], 1, 8), [3, 2]);
    }

    #[test]
    fn renaming_clashes_only_with_its_own_name() {
        let gallery = Kind::Gallery;
        assert_eq!(
            rename(gallery, "fractal", "  Fractals "),
            Some(Rename::SaveFirst)
        );
        assert_eq!(
            rename(gallery, "fractal", "Fractal"),
            Some(Rename::RemoveFirst)
        );
        assert_eq!(rename(gallery, "fractal", " fractal "), None);
        assert_eq!(rename(gallery, "fractal", "   "), None);
        // The boorus' searches may share a name.
        assert_eq!(rename(Kind::Boorus, "fox", "Fox"), Some(Rename::SaveFirst));
    }

    fn saved(id: i64, name: &str, query: &str) -> Item {
        item(id, name, query)
    }

    fn names(list: &[Item]) -> Vec<&str> {
        list.iter().map(|s| s.name.as_str()).collect()
    }

    #[test]
    fn a_search_is_found_however_it_was_spaced() {
        let list = vec![
            saved(1, "cat", "cat"),
            saved(2, "blue sky", "blue_sky -cloud"),
        ];
        assert_eq!(find(&list, "  blue_sky   -cloud ", &[]), Some(2));
        assert_eq!(find(&list, "-cloud Blue_Sky", &[]), Some(2));
        assert_eq!(find(&list, "cat", &[]), Some(1));
        assert_eq!(find(&list, "dog", &[]), None);
        assert_eq!(find(&list, "blue_sky", &[]), None);
        assert_eq!(find(&[], "cat", &[]), None);
        // A booru search is its search and its sites.
        let theirs = [remote(3, "fox", "fox  solo", &["https://e926.net"])];
        let e926 = |base: &str| RemoteSite {
            api: BooruApi::Danbooru,
            base: base.into(),
        };
        assert_eq!(
            find(&theirs, "fox solo", &[e926("https://e926.net")]),
            Some(3)
        );
        assert_eq!(find(&theirs, "solo fox", &[e926("https://e926.net")]), None);
        assert_eq!(find(&theirs, "fox solo", &[e926("https://e621.net")]), None);
    }

    fn same_search(a: &str, b: &str) -> bool {
        search_key(a) == search_key(b)
    }

    #[test]
    fn one_search_written_two_ways_is_the_same() {
        for (a, b) in [
            ("fractal  order:score", "fractal order:score"),
            // The sort and colour menus put their terms last.
            ("order:score fractal", " fractal   order:score "),
            ("color:red width:>100 cat", "cat width:>100 color:red"),
            ("fractal -abstract rating:s", "rating:s -abstract fractal"),
            ("Fractal", "fractal"),
            ("artist:lavfi", "lavfi"),
            ("{girl boots} cat", "cat {boots girl}"),
            ("cat cat", "cat"),
            ("order:top", "order:score"),
            ("cat text:\"two words\"", "text:\"two   words\" cat"),
        ] {
            assert!(same_search(a, b), "{a} / {b}");
        }
        for (a, b) in [
            ("cat", "dog"),
            ("cat", "-cat"),
            ("cat dog", "cat"),
            // The last sort typed is the one that counts.
            ("order:score order:new", "order:new order:score"),
            ("text:\"a b\"", "text:a text:b"),
            ("{girl boots}", "{girl} {boots}"),
            ("fractal", "exact:true fractal"),
            ("", "cat"),
        ] {
            assert!(!same_search(a, b), "{a} / {b}");
        }
        // One the parser turns down still matches its tidy form, and only that.
        assert!(same_search("rating:x  cat", " rating:x cat"));
        assert!(!same_search("rating:x cat", "cat rating:x"));
        assert!(same_search("", "   "));
    }

    #[test]
    fn the_list_stays_in_order_of_name() {
        let mut list = Vec::new();
        add(&mut list, saved(1, "fractal", "fractal"));
        add(&mut list, saved(2, "Abstract", "abstract"));
        add(&mut list, saved(3, "game_of_life", "game_of_life"));
        assert_eq!(names(&list), ["Abstract", "fractal", "game_of_life"]);
        // Saved again: one entry, in its new place.
        add(&mut list, saved(1, "zebra", "zebra"));
        assert_eq!(names(&list), ["Abstract", "game_of_life", "zebra"]);
        assert_eq!(remove(&mut list, 3).map(|s| s.id), Some(3));
        assert_eq!(names(&list), ["Abstract", "zebra"]);
        assert!(remove(&mut list, 3).is_none());
        // Undo puts it back where it was.
        add(&mut list, saved(3, "game_of_life", "game_of_life"));
        assert_eq!(names(&list), ["Abstract", "game_of_life", "zebra"]);
    }

    #[test]
    fn a_search_is_named_after_itself_within_the_limit() {
        assert_eq!(name_for("  fractal   order:score "), "fractal order:score");
        let long = "blue_sky_over_the_sea  rating:safe order:score  -abstract  fractal  mountain";
        let name = name_for(long);
        assert!(name.chars().count() <= MAX_SEARCH_NAME_CHARS);
        assert!(!name.ends_with(' '));
        assert!(starbooru_core::clean_saved_search(&name, long).is_ok());
        let wide = "猫".repeat(80);
        assert_eq!(name_for(&wide).chars().count(), MAX_SEARCH_NAME_CHARS);
    }
}
