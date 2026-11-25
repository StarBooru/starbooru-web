//! The tag blacklist without a browser: what's typed into the list before
//! it's sent, which entries are ratings, which of a post's entries the
//! search it was opened from doesn't name, and a guest's list kept on the
//! device.

use serde::{Deserialize, Serialize};
use starbooru_core::blacklist::{BlacklistError, Entry, split_entries};
use starbooru_core::{PostTag, Rating, parse_query};

/// The most guest lists the device keeps, one per server.
pub const MAX_GUEST_LISTS: usize = 8;

/// What the box that adds turned into: tags for chips, ratings to hide.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Typed {
    pub tags: Vec<String>,
    pub ratings: Vec<Rating>,
}

/// A word that can't go on the list, as typed, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BadEntry {
    pub word: String,
    pub error: BlacklistError,
}

/// Reads what was typed in the box that adds, a word an entry, as the
/// server will: `Blue_Sky` is `blue_sky`, `artist:name` is `name`, and
/// `rating:e` hides explicit posts. Other search terms are refused.
pub fn read_typed(text: &str) -> Result<Typed, BadEntry> {
    let mut typed = Typed::default();
    for word in text.split_whitespace() {
        match Entry::parse(word) {
            Ok(Entry::Tag(tag)) => {
                if !typed.tags.contains(&tag) {
                    typed.tags.push(tag);
                }
            }
            Ok(Entry::Rating(rating)) => {
                if !typed.ratings.contains(&rating) {
                    typed.ratings.push(rating);
                }
            }
            Err(error) => {
                return Err(BadEntry {
                    word: word.to_owned(),
                    error,
                });
            }
        }
    }
    Ok(typed)
}

/// A list's tags, in its order, and its ratings.
pub fn split(entries: &[String]) -> (Vec<String>, Vec<Rating>) {
    split_entries(entries)
}

/// The list as it's sent and kept: its tags, then its ratings in their
/// usual order, each once.
pub fn join(tags: &[String], ratings: &[Rating]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for tag in tags {
        if !out.contains(tag) {
            out.push(tag.clone());
        }
    }
    out.extend(
        Rating::ALL
            .into_iter()
            .filter(|r| ratings.contains(r))
            .map(|r| Entry::Rating(r).as_entry()),
    );
    out
}

/// `entries` with `typed` added: new tags at the end of the tags, and its
/// ratings hidden too.
pub fn with_typed(entries: &[String], typed: &Typed) -> Vec<String> {
    let (mut tags, mut ratings) = split(entries);
    for tag in &typed.tags {
        if !tags.contains(tag) {
            tags.push(tag.clone());
        }
    }
    ratings.extend(typed.ratings.iter().copied());
    join(&tags, &ratings)
}

/// `entries` without `entry`, a tag or `rating:…`.
pub fn without(entries: &[String], entry: &str) -> Vec<String> {
    entries.iter().filter(|e| *e != entry).cloned().collect()
}

/// `entries` with `rating` hidden, or no longer hidden.
pub fn toggled(entries: &[String], rating: Rating) -> Vec<String> {
    let (tags, mut ratings) = split(entries);
    if ratings.contains(&rating) {
        ratings.retain(|r| *r != rating);
    } else {
        ratings.push(rating);
    }
    join(&tags, &ratings)
}

/// `entries` with the tag `from` in its place as `to`, as a "did you mean"
/// puts it.
pub fn replaced(entries: &[String], from: &str, to: &str) -> Vec<String> {
    let (tags, ratings) = split(entries);
    let tags: Vec<String> = tags
        .into_iter()
        .map(|t| if t == from { to.to_owned() } else { t })
        .collect();
    join(&tags, &ratings)
}

/// `entry` put back where it was in `before`, into the list as it is `now`:
/// Undo of a removal.
pub fn put_back(now: &[String], before: &[String], entry: &str) -> Vec<String> {
    if now.iter().any(|e| e == entry) {
        return now.to_vec();
    }
    let at = before.iter().position(|e| e == entry).unwrap_or(now.len());
    let mut out = now.to_vec();
    out.insert(at.min(out.len()), entry.to_owned());
    // Ratings stay after the tags.
    let (tags, ratings) = split(&out);
    join(&tags, &ratings)
}

/// The entries a post matched that the search it was opened from doesn't
/// name, so they still cover it. A search names a rating entry with the
/// same `rating:`, and a tag entry with the tag, or with a tag of the post
/// whose tree it heads (`details` has each tag's top). The server shows
/// such a post in that search's results; with one entry left unnamed it
/// wouldn't.
pub fn still_hidden(blacklisted: &[String], q: &str, details: &[PostTag]) -> Vec<String> {
    let Ok(search) = parse_query(q) else {
        return blacklisted.to_vec();
    };
    let names_tag = |entry: &str| {
        search.include.iter().any(|asked| {
            asked == entry
                || details
                    .iter()
                    .any(|t| t.name == *asked && t.section.as_deref() == Some(entry))
        })
    };
    blacklisted
        .iter()
        .filter(|entry| match Entry::parse(entry) {
            Ok(Entry::Rating(rating)) => !search.ratings.contains(&rating),
            Ok(Entry::Tag(tag)) => !names_tag(&tag),
            Err(_) => true,
        })
        .cloned()
        .collect()
}

/// A guest's own list on one server, kept on the device. Until one is
/// kept, the server's default applies.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GuestList {
    /// `server|guest`, as `guest_owner` makes it.
    pub owner: String,
    pub entries: Vec<String>,
}

/// Whose a guest's list is: the guests of one server.
pub fn guest_owner(server: &str) -> String {
    format!("{server}|guest")
}

/// The list kept for `owner`, if the guest has changed theirs.
pub fn guest_list<'a>(lists: &'a [GuestList], owner: &str) -> Option<&'a [String]> {
    lists
        .iter()
        .find(|l| l.owner == owner)
        .map(|l| l.entries.as_slice())
}

/// Keeps `entries` as `owner`'s list, first, or forgets it for `None`. The
/// oldest go past [`MAX_GUEST_LISTS`].
pub fn set_guest_list(lists: &mut Vec<GuestList>, owner: &str, entries: Option<Vec<String>>) {
    lists.retain(|l| l.owner != owner);
    if let Some(entries) = entries {
        lists.insert(
            0,
            GuestList {
                owner: owner.to_owned(),
                entries,
            },
        );
    }
    lists.truncate(MAX_GUEST_LISTS);
}

#[cfg(test)]
mod tests {
    use starbooru_core::TagCategory;

    use super::*;

    fn list(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_owned()).collect()
    }

    fn tag(name: &str, section: Option<&str>) -> PostTag {
        PostTag {
            name: name.into(),
            category: TagCategory::General,
            section: section.map(str::to_owned),
        }
    }

    #[test]
    fn a_typed_word_becomes_a_tag_or_a_rating() {
        let typed = read_typed("  Blue_Sky artist:Some_One rating:e cat cat RATING:Q ").unwrap();
        assert_eq!(typed.tags, ["blue_sky", "some_one", "cat"]);
        assert_eq!(typed.ratings, [Rating::Explicit, Rating::Questionable]);
        assert_eq!(read_typed("").unwrap(), Typed::default());
        // A search term isn't an entry, and says which word.
        let bad = read_typed("cat type:video").unwrap_err();
        assert_eq!(bad.word, "type:video");
        assert!(matches!(bad.error, BlacklistError::NotATag(_)));
        assert!(matches!(
            read_typed("-dog").unwrap_err().error,
            BlacklistError::Tag(_)
        ));
        assert!(matches!(
            read_typed("rating:x").unwrap_err().error,
            BlacklistError::Query(_)
        ));
    }

    #[test]
    fn tags_come_first_and_ratings_after_in_their_order() {
        let entries = list(&["cat", "rating:explicit", "dog", "rating:safe"]);
        let (tags, ratings) = split(&entries);
        assert_eq!(tags, ["cat", "dog"]);
        assert_eq!(ratings, [Rating::Explicit, Rating::Safe]);
        assert_eq!(
            join(&tags, &ratings),
            ["cat", "dog", "rating:safe", "rating:explicit"]
        );
        assert_eq!(join(&list(&["a", "a"]), &[]), ["a"]);
    }

    #[test]
    fn the_list_changes_a_chip_or_a_toggle_at_a_time() {
        let entries = list(&["cat", "rating:explicit"]);
        let typed = Typed {
            tags: list(&["dog", "cat"]),
            ratings: vec![Rating::Questionable],
        };
        assert_eq!(
            with_typed(&entries, &typed),
            ["cat", "dog", "rating:questionable", "rating:explicit"]
        );
        assert_eq!(without(&entries, "cat"), ["rating:explicit"]);
        assert_eq!(without(&entries, "rating:explicit"), ["cat"]);
        assert_eq!(toggled(&entries, Rating::Explicit), ["cat"]);
        assert_eq!(
            toggled(&entries, Rating::Safe),
            ["cat", "rating:safe", "rating:explicit"]
        );
        assert_eq!(
            replaced(&list(&["catt", "dog"]), "catt", "cat"),
            ["cat", "dog"]
        );
        // Corrected into a tag already there: once.
        assert_eq!(replaced(&list(&["catt", "cat"]), "catt", "cat"), ["cat"]);
    }

    #[test]
    fn undo_puts_an_entry_back_where_it_was() {
        let before = list(&["a", "b", "c", "rating:explicit"]);
        let now = without(&before, "b");
        assert_eq!(put_back(&now, &before, "b"), before);
        // Something added meanwhile stays.
        let now = list(&["a", "c", "d"]);
        assert_eq!(put_back(&now, &before, "b"), ["a", "b", "c", "d"]);
        assert_eq!(put_back(&before, &before, "b"), before);
        let now = without(&before, "rating:explicit");
        assert_eq!(put_back(&now, &before, "rating:explicit"), before);
    }

    #[test]
    fn a_search_that_names_an_entry_uncovers_it() {
        let flags = list(&["cat", "rating:explicit"]);
        let details = [tag("cat", None), tag("boots", Some("footwear"))];
        assert_eq!(still_hidden(&flags, "", &details), flags);
        // Only what the search names: the rest still covers it.
        assert_eq!(
            still_hidden(&flags, "cat sky", &details),
            ["rating:explicit"]
        );
        assert!(still_hidden(&flags, "cat rating:e", &details).is_empty());
        assert!(still_hidden(&flags, "rating:explicit artist:cat", &details).is_empty());
        // Leaving a tag out, or the rating, names nothing.
        assert_eq!(still_hidden(&flags, "-cat -rating:e", &details), flags);
        // A tag under the entry's tree, as the post's tags say.
        let footwear = list(&["footwear"]);
        assert!(still_hidden(&footwear, "boots", &details).is_empty());
        assert_eq!(still_hidden(&footwear, "sandals", &details), footwear);
        // A search that doesn't read names nothing.
        assert_eq!(still_hidden(&flags, "rating:zzz cat", &details), flags);
    }

    #[test]
    fn a_guest_keeps_a_list_per_server() {
        let mut lists = Vec::new();
        let a = guest_owner("https://a.example");
        assert_eq!(a, "https://a.example|guest");
        assert_eq!(guest_list(&lists, &a), None);
        set_guest_list(&mut lists, &a, Some(list(&["cat"])));
        assert_eq!(guest_list(&lists, &a), Some(list(&["cat"]).as_slice()));
        // An empty list hides nothing; the default doesn't come back.
        set_guest_list(&mut lists, &a, Some(Vec::new()));
        assert_eq!(guest_list(&lists, &a), Some(&[][..]));
        set_guest_list(&mut lists, &a, None);
        assert_eq!(guest_list(&lists, &a), None);
        for i in 0..MAX_GUEST_LISTS + 3 {
            set_guest_list(&mut lists, &guest_owner(&format!("s{i}")), Some(Vec::new()));
        }
        assert_eq!(lists.len(), MAX_GUEST_LISTS);
        assert_eq!(guest_list(&lists, &guest_owner("s0")), None);
        assert!(guest_list(&lists, &guest_owner("s10")).is_some());
    }
}
