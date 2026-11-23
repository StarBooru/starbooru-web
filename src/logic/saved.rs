//! The account's saved searches, as the gallery keeps them: in order of name,
//! each found by its search.

use starbooru_core::SavedSearch;
use starbooru_core::saved::MAX_SEARCH_NAME_CHARS;

/// A search as it is saved: the server collapses the spaces too.
pub fn tidy(q: &str) -> String {
    q.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The saved search that runs `q`, if any.
pub fn find(list: &[SavedSearch], q: &str) -> Option<i64> {
    let q = tidy(q);
    list.iter().find(|s| s.query == q).map(|s| s.id)
}

/// The name a search is saved under: the search itself, cut to the longest
/// name the server takes.
pub fn name_for(q: &str) -> String {
    let name: String = tidy(q).chars().take(MAX_SEARCH_NAME_CHARS).collect();
    name.trim_end().to_owned()
}

/// Puts `saved` in the list, in its place by name, in place of one with the
/// same id.
pub fn add(list: &mut Vec<SavedSearch>, saved: SavedSearch) {
    list.retain(|s| s.id != saved.id);
    let key = saved.name.to_lowercase();
    let at = list.partition_point(|s| s.name.to_lowercase() <= key);
    list.insert(at, saved);
}

/// Takes saved search `id` out of the list, giving it back.
pub fn remove(list: &mut Vec<SavedSearch>, id: i64) -> Option<SavedSearch> {
    let at = list.iter().position(|s| s.id == id)?;
    Some(list.remove(at))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved(id: i64, name: &str, query: &str) -> SavedSearch {
        SavedSearch {
            id,
            name: name.into(),
            query: query.into(),
            created_at: chrono::Utc::now(),
        }
    }

    fn names(list: &[SavedSearch]) -> Vec<&str> {
        list.iter().map(|s| s.name.as_str()).collect()
    }

    #[test]
    fn a_search_is_found_however_it_was_spaced() {
        let list = vec![
            saved(1, "cat", "cat"),
            saved(2, "blue sky", "blue_sky -cloud"),
        ];
        assert_eq!(find(&list, "  blue_sky   -cloud "), Some(2));
        assert_eq!(find(&list, "cat"), Some(1));
        assert_eq!(find(&list, "dog"), None);
        assert_eq!(find(&[], "cat"), None);
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
