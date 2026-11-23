//! The posts opened in the viewer, which Explore lists: the latest first,
//! each once, and the oldest dropped past the cap.

/// Posts the history keeps.
pub const MAX_OPENED: usize = 60;

/// Posts Explore shows at first, and again with each Load more. The server
/// has no way to ask for several posts by id, so each is a request.
pub const SHOWN: usize = 12;

/// Post `id` was opened: it goes first, out of where it was. Says whether
/// the list changed.
pub fn open(ids: &mut Vec<i64>, id: i64) -> bool {
    if ids.first() == Some(&id) {
        return false;
    }
    ids.retain(|i| *i != id);
    ids.insert(0, id);
    ids.truncate(MAX_OPENED);
    true
}

/// Ids as a list keeps them: real ones, each once where it first comes, and
/// no more than the cap.
pub fn tidy(ids: impl IntoIterator<Item = i64>) -> Vec<i64> {
    let mut kept: Vec<i64> = Vec::new();
    for id in ids {
        if kept.len() == MAX_OPENED {
            break;
        }
        if id > 0 && !kept.contains(&id) {
            kept.push(id);
        }
    }
    kept
}

/// Takes `gone` out of the list. Says whether any were in it.
pub fn drop_ids(ids: &mut Vec<i64>, gone: &[i64]) -> bool {
    let before = ids.len();
    ids.retain(|id| !gone.contains(id));
    ids.len() != before
}

/// What an Undo of a clear puts back: what was opened since first, then
/// what was cleared.
pub fn put_back(now: &[i64], cleared: &[i64]) -> Vec<i64> {
    tidy(now.iter().chain(cleared).copied())
}

/// The ids among the first `wanted` that still need asking for, in order.
pub fn to_fetch(ids: &[i64], wanted: usize, done: impl Fn(i64) -> bool) -> Vec<i64> {
    ids.iter()
        .take(wanted)
        .copied()
        .filter(|id| !done(*id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_latest_comes_first_and_each_post_once() {
        let mut ids = Vec::new();
        assert!(open(&mut ids, 1));
        assert!(open(&mut ids, 2));
        assert!(open(&mut ids, 3));
        assert_eq!(ids, [3, 2, 1]);
        // Opened again: to the front, not twice.
        assert!(open(&mut ids, 1));
        assert_eq!(ids, [1, 3, 2]);
        // Already first: nothing to save.
        assert!(!open(&mut ids, 1));
        assert_eq!(ids, [1, 3, 2]);
    }

    #[test]
    fn the_oldest_drop_off_past_the_cap() {
        let mut ids = Vec::new();
        for id in 1..=(MAX_OPENED as i64 + 5) {
            open(&mut ids, id);
        }
        assert_eq!(ids.len(), MAX_OPENED);
        assert_eq!(ids.first(), Some(&(MAX_OPENED as i64 + 5)));
        assert_eq!(ids.last(), Some(&6));
        // A post far down comes back to the front without growing the list.
        open(&mut ids, 6);
        assert_eq!((ids.len(), ids.first()), (MAX_OPENED, Some(&6)));
    }

    #[test]
    fn a_stored_list_is_tidied() {
        assert_eq!(tidy([5, 3, 5, 0, -2, 3, 9]), [5, 3, 9]);
        assert_eq!(tidy(1..=200).len(), MAX_OPENED);
        assert_eq!(tidy([]), Vec::<i64>::new());
    }

    #[test]
    fn gone_posts_leave_the_list() {
        let mut ids = vec![4, 3, 2, 1];
        assert!(drop_ids(&mut ids, &[3, 1, 99]));
        assert_eq!(ids, [4, 2]);
        assert!(!drop_ids(&mut ids, &[7]));
        assert!(!drop_ids(&mut ids, &[]));
    }

    #[test]
    fn undo_puts_the_cleared_posts_back_after_newer_ones() {
        assert_eq!(put_back(&[], &[3, 2, 1]), [3, 2, 1]);
        // Opened meanwhile, maybe in another tab: those lead, once each.
        assert_eq!(put_back(&[7, 2], &[3, 2, 1]), [7, 2, 3, 1]);
        let full: Vec<i64> = (1..=MAX_OPENED as i64).collect();
        assert_eq!(put_back(&[100], &full).len(), MAX_OPENED);
        assert_eq!(put_back(&[100], &full).first(), Some(&100));
    }

    #[test]
    fn only_the_shown_posts_not_yet_had_are_asked_for() {
        let ids = [9, 8, 7, 6, 5];
        assert_eq!(to_fetch(&ids, 3, |_| false), [9, 8, 7]);
        assert_eq!(to_fetch(&ids, 3, |id| id == 8), [9, 7]);
        assert_eq!(to_fetch(&ids, 10, |id| id > 6), [6, 5]);
        assert!(to_fetch(&ids, 0, |_| false).is_empty());
        assert!(to_fetch(&[], SHOWN, |_| false).is_empty());
    }
}
