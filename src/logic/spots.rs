//! Which places the gallery remembers (`store::Spot`): a saved search's for as
//! long as it stays saved, and the latest of the rest.

use std::collections::HashSet;

use starbooru_core::saved::MAX_SAVED_SEARCHES;

use super::saved::search_key;
use crate::store::Spot;

/// Searches the gallery remembers besides the saved ones.
pub const MAX_SPOTS: usize = 30;

/// Saved searches it remembers, at most: as many as the server saves.
pub const MAX_SAVED_SPOTS: usize = MAX_SAVED_SEARCHES;

/// Trims `spots`, the latest first. A saved search's spot stays however many
/// others come after it, and the rest keep the latest `MAX_SPOTS` among
/// themselves. `saved` holds the account's saved searches, `None` while
/// they aren't known: then any spot may be one, so only what's past both
/// limits together goes. A search written two ways stays once, as it came
/// last.
pub fn keep(spots: &mut Vec<Spot>, saved: Option<&[String]>) {
    let saved: Option<HashSet<String>> =
        saved.map(|list| list.iter().map(String::as_str).map(search_key).collect());
    let mut seen = HashSet::new();
    let (mut unknown, mut pinned, mut others) = (0_usize, 0_usize, 0_usize);
    spots.retain(|spot| {
        let key = search_key(&spot.q);
        let (count, limit) = match &saved {
            None => (&mut unknown, MAX_SPOTS + MAX_SAVED_SPOTS),
            Some(saved) if saved.contains(&key) => (&mut pinned, MAX_SAVED_SPOTS),
            Some(_) => (&mut others, MAX_SPOTS),
        };
        if !seen.insert(key) {
            return false;
        }
        *count += 1;
        *count <= limit
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spot(q: &str) -> Spot {
        Spot {
            q: q.into(),
            post: Some(7),
            count: 60,
            ..Spot::default()
        }
    }

    /// `n` searches, the latest first, after `older` ones.
    fn after(n: usize, older: &[&str]) -> Vec<Spot> {
        let mut spots: Vec<Spot> = (0..n).rev().map(|i| spot(&format!("tag{i}"))).collect();
        spots.extend(older.iter().map(|q| spot(q)));
        spots
    }

    fn queries(spots: &[Spot]) -> Vec<&str> {
        spots.iter().map(|s| s.q.as_str()).collect()
    }

    fn saved(list: &[&str]) -> Vec<String> {
        list.iter().map(|q| (*q).to_owned()).collect()
    }

    #[test]
    fn a_saved_search_keeps_its_place_however_many_come_after() {
        let mut spots = after(MAX_SPOTS + 10, &["fractal", "cat"]);
        keep(&mut spots, Some(&saved(&["fractal"])));
        assert_eq!(spots.len(), MAX_SPOTS + 1);
        assert_eq!(spots.last().map(|s| s.q.as_str()), Some("fractal"));
        assert_eq!(spots.last().and_then(|s| s.post), Some(7));
        assert!(!queries(&spots).contains(&"cat"));
        // The latest of the rest, in order.
        assert_eq!(spots.first().map(|s| s.q.as_str()), Some("tag39"));
        assert!(queries(&spots).contains(&"tag10"));
        assert!(!queries(&spots).contains(&"tag9"));
        // Trimming again changes nothing.
        let before = spots.clone();
        keep(&mut spots, Some(&saved(&["fractal"])));
        assert_eq!(spots, before);
    }

    #[test]
    fn a_saved_search_is_known_however_it_was_written() {
        let mut spots = after(MAX_SPOTS + 5, &["order:score  Fractal", "rating:x  cat"]);
        keep(
            &mut spots,
            Some(&saved(&["fractal order:score", "rating:x cat"])),
        );
        let kept = queries(&spots);
        assert!(kept.contains(&"order:score  Fractal"));
        // A search the parser turns down is matched by its spaces alone.
        assert!(kept.contains(&"rating:x  cat"));
        assert_eq!(spots.len(), MAX_SPOTS + 2);
    }

    #[test]
    fn a_forgotten_search_ages_out_like_any_other() {
        // Forgotten while still among the latest: it stays for now.
        let mut spots = after(10, &["fractal"]);
        keep(&mut spots, Some(&[]));
        assert!(queries(&spots).contains(&"fractal"));
        // Past the latest searches: it goes.
        let mut spots = after(MAX_SPOTS, &["fractal"]);
        keep(&mut spots, Some(&[]));
        assert_eq!(spots.len(), MAX_SPOTS);
        assert!(!queries(&spots).contains(&"fractal"));
    }

    #[test]
    fn nothing_saved_goes_while_the_saved_searches_are_unknown() {
        let mut spots = after(MAX_SPOTS + 10, &["fractal"]);
        keep(&mut spots, None);
        assert_eq!(spots.len(), MAX_SPOTS + 11);
        // Still within one limit for everything.
        let mut spots = after(MAX_SPOTS + MAX_SAVED_SPOTS + 10, &[]);
        keep(&mut spots, None);
        assert_eq!(spots.len(), MAX_SPOTS + MAX_SAVED_SPOTS);
        assert_eq!(spots.first().map(|s| s.q.as_str()), Some("tag239"));
    }

    #[test]
    fn saved_searches_have_a_limit_too() {
        let mut spots = after(MAX_SAVED_SPOTS + 50, &[]);
        let all: Vec<String> = spots.iter().map(|s| s.q.clone()).collect();
        keep(&mut spots, Some(&all));
        assert_eq!(spots.len(), MAX_SAVED_SPOTS);
        // The latest stay.
        assert_eq!(spots.first().map(|s| s.q.as_str()), Some("tag249"));
        assert_eq!(spots.last().map(|s| s.q.as_str()), Some("tag50"));
    }

    #[test]
    fn a_search_written_two_ways_stays_once() {
        let mut spots = vec![
            spot("cat  dog"),
            spot("tag1"),
            spot("dog cat"),
            spot("Cat dog"),
        ];
        keep(&mut spots, Some(&[]));
        assert_eq!(queries(&spots), ["cat  dog", "tag1"]);
        let mut spots = vec![spot("cat dog"), spot("dog cat")];
        keep(&mut spots, None);
        assert_eq!(queries(&spots), ["cat dog"]);
    }
}
