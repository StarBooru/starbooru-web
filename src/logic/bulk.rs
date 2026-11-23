//! Mass edits and the recycle bin without a browser: which posts are picked
//! and in what order, a Shift-click's range, the request an edit sends, what
//! to say after it, and what the bin says about purging.

use std::collections::BTreeSet;

use starbooru_core::{BulkEdit, BulkResult, MAX_BULK_POSTS, Rating, split_category};

use super::tags::{BadTag, parse_tags};

// ---------- the selection ----------

/// The posts picked in the gallery's selection mode, and the last one
/// clicked, where a Shift-click's range starts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    ids: BTreeSet<i64>,
    anchor: Option<i64>,
}

impl Selection {
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn contains(&self, id: i64) -> bool {
        self.ids.contains(&id)
    }

    pub fn ids(&self) -> &BTreeSet<i64> {
        &self.ids
    }

    /// A click on post `id`, of the posts `loaded` in the gallery's order.
    /// With `shift`, every post from the one clicked last to this one takes
    /// this one's new state; without, only this one changes. Nothing past
    /// `MAX_BULK_POSTS` is picked.
    pub fn toggle(&mut self, loaded: &[i64], id: i64, shift: bool) {
        let on = !self.ids.contains(&id);
        let ends = self.anchor.filter(|_| shift).and_then(|anchor| {
            let from = loaded.iter().position(|p| *p == anchor)?;
            let to = loaded.iter().position(|p| *p == id)?;
            Some((from, to))
        });
        match ends {
            Some((from, to)) => self.set_range(loaded, from, to, on),
            None => {
                self.set(id, on);
                self.anchor = Some(id);
            }
        }
    }

    /// Picks, or lets go of when `on` is false, every post of `loaded` from
    /// place `from` to place `to`, either way round, and a Shift-click's
    /// range goes on from `to`.
    pub fn set_range(&mut self, loaded: &[i64], from: usize, to: usize, on: bool) {
        let range = loaded.get(from.min(to)..=from.max(to)).unwrap_or_default();
        // From `from` outwards, so a full selection keeps the posts next to it.
        if from <= to {
            for p in range {
                self.set(*p, on);
            }
        } else {
            for p in range.iter().rev() {
                self.set(*p, on);
            }
        }
        if let Some(id) = loaded.get(to) {
            self.anchor = Some(*id);
        }
    }

    fn set(&mut self, id: i64, on: bool) {
        if !on {
            self.ids.remove(&id);
        } else if self.ids.len() < MAX_BULK_POSTS {
            self.ids.insert(id);
        }
    }

    /// Whether "select all" has nothing left to add: every loaded post is
    /// picked, or as many as one edit takes.
    pub fn is_full(&self, loaded: &[i64]) -> bool {
        self.ids.len() >= MAX_BULK_POSTS
            || (!loaded.is_empty() && loaded.iter().all(|id| self.ids.contains(id)))
    }

    /// Every loaded post, or none once that's done.
    pub fn toggle_all(&mut self, loaded: &[i64]) {
        if self.is_full(loaded) {
            self.ids.clear();
        } else {
            for id in loaded {
                self.set(*id, true);
            }
        }
        self.anchor = None;
    }

    /// Post `id` left the gallery.
    pub fn remove(&mut self, id: i64) {
        self.ids.remove(&id);
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// The picked ids in the order the gallery shows them, then any it no
/// longer shows.
pub fn in_gallery_order(loaded: &[i64], selected: &BTreeSet<i64>) -> Vec<i64> {
    let mut ids: Vec<i64> = loaded
        .iter()
        .copied()
        .filter(|id| selected.contains(id))
        .collect();
    ids.extend(selected.iter().filter(|id| !loaded.contains(id)));
    ids
}

/// Of `ids`, the ones a move to the recycle bin takes there: Undo brings
/// back only those, not posts that were in the bin already.
pub fn live_ones(ids: &[i64], in_bin: impl Fn(i64) -> bool) -> Vec<i64> {
    ids.iter().copied().filter(|id| !in_bin(*id)).collect()
}

// ---------- the request ----------

/// Why an edit can't be sent as it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditProblem {
    /// A word that can't be a tag.
    Bad(BadTag),
    /// The same tag both to add and to remove.
    Both(String),
    /// No tag to add or remove, and no rating.
    Nothing,
}

/// The edit of posts `ids`: the tags in `add` go on, those in `remove` come
/// off, each normalised as the server does, and `rating` replaces theirs.
pub fn bulk_edit(
    ids: Vec<i64>,
    add: &str,
    remove: &str,
    rating: Option<Rating>,
) -> Result<BulkEdit, EditProblem> {
    let add_tags = parse_tags(add).map_err(EditProblem::Bad)?;
    let remove_tags = parse_tags(remove).map_err(EditProblem::Bad)?;
    // `artist:mika` and `mika` are the same tag once it's on a post.
    let bare = |tag: &str| split_category(tag).1.to_owned();
    if let Some(both) = add_tags
        .iter()
        .map(|t| bare(t))
        .find(|a| remove_tags.iter().any(|r| bare(r) == *a))
    {
        return Err(EditProblem::Both(both));
    }
    if add_tags.is_empty() && remove_tags.is_empty() && rating.is_none() {
        return Err(EditProblem::Nothing);
    }
    Ok(BulkEdit {
        ids,
        add_tags,
        remove_tags,
        rating,
        delete: false,
    })
}

/// Moving posts `ids` to the recycle bin.
pub fn bulk_delete(ids: Vec<i64>) -> BulkEdit {
    BulkEdit {
        ids,
        delete: true,
        ..BulkEdit::default()
    }
}

// ---------- what to say after ----------

/// What was done to how many posts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Done {
    Moved(u64),
    Updated(u64),
}

/// Posts the server left alone, and why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skipped {
    /// Not the user's to move to the bin.
    Undeletable(u64),
    /// Missing, or in the bin and so not edited.
    Left(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Report {
    pub done: Done,
    pub skipped: Option<Skipped>,
}

/// The message for an edit's `result`, a move to the bin when `deleting`.
pub fn report(result: BulkResult, deleting: bool) -> Report {
    let done = if deleting {
        Done::Moved(result.updated)
    } else {
        Done::Updated(result.updated)
    };
    let skipped = match (result.skipped, deleting) {
        (0, _) => None,
        (n, true) => Some(Skipped::Undeletable(n)),
        (n, false) => Some(Skipped::Left(n)),
    };
    Report { done, skipped }
}

/// Whether a failed restore is worth telling about when Undo brings back a
/// mass move: a post the user may not delete was never moved, and the server
/// refuses to restore it all the same.
pub fn worth_telling(status: Option<u16>) -> bool {
    status != Some(403)
}

// ---------- the recycle bin ----------

/// Posts in a page of the bin.
pub const BIN_PAGE: u32 = 60;
/// Restoring everything reads the bin in pages this long.
pub const BIN_SWEEP: u32 = 200;
/// Pages of `BIN_SWEEP` read at most, so a huge bin can't keep it going.
pub const BIN_SWEEPS: u32 = 50;
/// Restores sent at once.
pub const RESTORES_AT_ONCE: usize = 8;

/// The page of the bin that holds the post after the `listed` ones, which
/// may hold some of them too.
pub fn bin_page(listed: usize) -> u32 {
    let per_page = usize::try_from(BIN_PAGE).unwrap_or(1).max(1);
    u32::try_from(listed / per_page)
        .unwrap_or(u32::MAX)
        .saturating_add(1)
}

/// What the bin says about purging, from the server's `trash_days`: unknown
/// when it couldn't be read (only administrators may), none at 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PurgeNote {
    Never,
    After(u32),
}

pub fn purge_note(trash_days: Option<u32>) -> Option<PurgeNote> {
    match trash_days? {
        0 => Some(PurgeNote::Never),
        days => Some(PurgeNote::After(days)),
    }
}

#[cfg(test)]
mod tests {
    use starbooru_core::tag::TagError;

    use super::*;

    fn picked(s: &Selection) -> Vec<i64> {
        s.ids().iter().copied().collect()
    }

    #[test]
    fn picks_in_the_order_the_gallery_shows() {
        let selected = BTreeSet::from([2, 5, 9, 40]);
        assert_eq!(in_gallery_order(&[9, 7, 5, 2], &selected), [9, 5, 2, 40]);
        assert!(in_gallery_order(&[1], &BTreeSet::new()).is_empty());
        assert_eq!(in_gallery_order(&[], &selected), [2, 5, 9, 40]);
    }

    #[test]
    fn a_click_toggles_one_post() {
        let loaded = [10, 9, 8, 7];
        let mut s = Selection::default();
        s.toggle(&loaded, 9, false);
        s.toggle(&loaded, 7, false);
        assert_eq!(picked(&s), [7, 9]);
        s.toggle(&loaded, 9, false);
        assert_eq!(picked(&s), [7]);
        // Shift without a post clicked before is a plain click.
        let mut fresh = Selection::default();
        fresh.toggle(&loaded, 8, true);
        assert_eq!(picked(&fresh), [8]);
    }

    #[test]
    fn shift_picks_from_the_last_click_to_this_one_either_way() {
        let loaded = [10, 9, 8, 7, 6, 5];
        let mut down = Selection::default();
        down.toggle(&loaded, 9, false);
        down.toggle(&loaded, 6, true);
        assert_eq!(picked(&down), [6, 7, 8, 9]);
        let mut up = Selection::default();
        up.toggle(&loaded, 6, false);
        up.toggle(&loaded, 9, true);
        assert_eq!(picked(&up), [6, 7, 8, 9]);
        // The range goes on from the post clicked last.
        up.toggle(&loaded, 5, true);
        assert_eq!(picked(&up), [5, 6, 7, 8, 9]);
    }

    #[test]
    fn shift_on_a_picked_post_lets_the_range_go() {
        let loaded = [10, 9, 8, 7, 6];
        let mut s = Selection::default();
        s.toggle_all(&loaded);
        s.toggle(&loaded, 9, false);
        s.toggle(&loaded, 9, false);
        s.toggle(&loaded, 6, true);
        assert_eq!(picked(&s), [10]);
    }

    #[test]
    fn a_range_from_a_post_no_longer_shown_is_one_click() {
        let mut s = Selection::default();
        s.toggle(&[1, 2, 3], 1, false);
        s.toggle(&[2, 3, 4], 4, true);
        assert_eq!(picked(&s), [1, 4]);
    }

    #[test]
    fn never_more_than_one_edit_takes() {
        let loaded: Vec<i64> = (0..600).collect();
        let mut s = Selection::default();
        s.toggle(&loaded, 0, false);
        s.toggle(&loaded, 599, true);
        assert_eq!(s.len(), MAX_BULK_POSTS);
        // Nearest the anchor first.
        assert!(s.contains(0) && s.contains(499) && !s.contains(500));
        assert!(s.is_full(&loaded));
        let mut back = Selection::default();
        back.toggle(&loaded, 599, false);
        back.toggle(&loaded, 0, true);
        assert!(back.contains(599) && back.contains(100) && !back.contains(99));
        let mut all = Selection::default();
        all.toggle_all(&loaded);
        assert_eq!(all.len(), MAX_BULK_POSTS);
        all.toggle(&loaded, 599, false);
        assert!(!all.contains(599));
    }

    #[test]
    fn select_all_then_none() {
        let loaded = [3, 2, 1];
        let mut s = Selection::default();
        assert!(!s.is_full(&loaded));
        assert!(!s.is_full(&[]));
        s.toggle(&loaded, 2, false);
        s.toggle_all(&loaded);
        assert_eq!(picked(&s), [1, 2, 3]);
        assert!(s.is_full(&loaded));
        s.toggle_all(&loaded);
        assert_eq!(s.len(), 0);
        s.toggle(&loaded, 3, false);
        s.toggle(&loaded, 1, false);
        s.remove(3);
        assert_eq!(picked(&s), [1]);
        s.clear();
        assert_eq!(s, Selection::default());
    }

    #[test]
    fn undo_brings_back_only_what_the_move_took() {
        let in_bin = |id: i64| id == 30;
        assert_eq!(live_ones(&[41, 30, 42], in_bin), [41, 42]);
        assert!(live_ones(&[30], in_bin).is_empty());
    }

    #[test]
    fn an_edit_normalises_its_tags() {
        let edit = bulk_edit(vec![1, 2], " Blue_Sky night\nNIGHT ", "Cloud", None).unwrap();
        assert_eq!(edit.ids, [1, 2]);
        assert_eq!(edit.add_tags, ["blue_sky", "night"]);
        assert_eq!(edit.remove_tags, ["cloud"]);
        assert_eq!(edit.rating, None);
        assert!(!edit.delete);
        let rated = bulk_edit(vec![3], "", "", Some(Rating::Explicit)).unwrap();
        assert!(rated.add_tags.is_empty() && rated.remove_tags.is_empty());
        assert_eq!(rated.rating, Some(Rating::Explicit));
        // A category prefix stays, for the server to read.
        let artist = bulk_edit(vec![3], "ARTIST:Lavfi", "", None).unwrap();
        assert_eq!(artist.add_tags, ["artist:lavfi"]);
    }

    #[test]
    fn an_edit_that_contradicts_itself_or_says_nothing_is_not_sent() {
        let problem = |add, remove| bulk_edit(vec![1], add, remove, None).err();
        assert_eq!(
            problem("sky cat", "Cat"),
            Some(EditProblem::Both("cat".into()))
        );
        assert_eq!(
            problem("artist:mika", "mika"),
            Some(EditProblem::Both("mika".into()))
        );
        assert_eq!(problem(" ", "\n"), Some(EditProblem::Nothing));
        assert_eq!(
            problem("sky", "-dog"),
            Some(EditProblem::Bad(BadTag {
                word: "-dog".into(),
                error: TagError::BadPrefix("-dog".into())
            }))
        );
        let moving = bulk_delete(vec![4, 5]);
        assert!(moving.delete && moving.add_tags.is_empty() && moving.rating.is_none());
        assert_eq!(moving.ids, [4, 5]);
    }

    #[test]
    fn the_report_says_what_was_done_and_what_was_left() {
        let r = |updated, skipped| BulkResult { updated, skipped };
        assert_eq!(
            report(r(3, 0), true),
            Report {
                done: Done::Moved(3),
                skipped: None
            }
        );
        assert_eq!(
            report(r(2, 1), true),
            Report {
                done: Done::Moved(2),
                skipped: Some(Skipped::Undeletable(1))
            }
        );
        assert_eq!(
            report(r(5, 0), false),
            Report {
                done: Done::Updated(5),
                skipped: None
            }
        );
        assert_eq!(
            report(r(0, 4), false),
            Report {
                done: Done::Updated(0),
                skipped: Some(Skipped::Left(4))
            }
        );
    }

    #[test]
    fn undo_keeps_quiet_about_posts_it_never_moved() {
        assert!(!worth_telling(Some(403)));
        assert!(worth_telling(Some(404)));
        assert!(worth_telling(Some(500)));
        assert!(worth_telling(None));
    }

    #[test]
    fn the_bin_reads_on_from_what_it_lists() {
        assert_eq!(bin_page(0), 1);
        assert_eq!(bin_page(59), 1);
        assert_eq!(bin_page(60), 2);
        // Two of 120 restored: the next post is the 59th of page 2 now.
        assert_eq!(bin_page(118), 2);
        assert_eq!(bin_page(120), 3);
    }

    #[test]
    fn the_bin_says_when_it_purges_if_the_server_says() {
        assert_eq!(purge_note(None), None);
        assert_eq!(purge_note(Some(0)), Some(PurgeNote::Never));
        assert_eq!(purge_note(Some(1)), Some(PurgeNote::After(1)));
        assert_eq!(purge_note(Some(30)), Some(PurgeNote::After(30)));
    }
}
