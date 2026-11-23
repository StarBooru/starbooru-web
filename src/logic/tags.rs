//! Tags without a browser: the tag editor's text and its "did you mean", who
//! may change a post or a tag, the Tags page's requests and filter, and the
//! searches a tag's menu leads to.

use starbooru_core::tag::{MAX_TAGS_PER_POST, TagError};
use starbooru_core::{
    Role, Tag, TagCategory, TagCorrection, TagOrder, TagSuggestion, User, normalize_tag,
    split_category,
};

use super::search::Hint;

// ---------- the tag editor's text ----------

/// A word of the tag editor that can't be a tag. `word` is empty when the
/// trouble is the number of tags.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BadTag {
    pub word: String,
    pub error: TagError,
}

/// The tags in the editor's text: one per word, normalised as the server
/// does, each once, in the order typed. A word of underscores alone is no
/// tag and goes.
pub fn parse_tags(text: &str) -> Result<Vec<String>, BadTag> {
    let mut out: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        let tag = match normalize_tag(word) {
            Ok(tag) => tag,
            Err(TagError::Empty) => continue,
            Err(error) => {
                return Err(BadTag {
                    word: word.to_owned(),
                    error,
                });
            }
        };
        if out.contains(&tag) {
            continue;
        }
        if out.len() >= MAX_TAGS_PER_POST {
            return Err(BadTag {
                word: String::new(),
                error: TagError::TooMany,
            });
        }
        out.push(tag);
    }
    Ok(out)
}

/// Whether a word of the editor's text is the tag `name`.
pub fn is_tag(word: &str, name: &str) -> bool {
    normalize_tag(word).is_ok_and(|w| w == name)
}

/// `text` with `name` added at the end, unless it's there already.
pub fn with_tag(text: &str, name: &str) -> String {
    if text.split_whitespace().any(|w| is_tag(w, name)) {
        return text.to_owned();
    }
    let mut out = text.trim_end().to_owned();
    if !out.is_empty() {
        out.push(' ');
    }
    out.push_str(name);
    out
}

/// `text` with the tag `from` replaced by `to`, and `to` there only once.
pub fn replace_tag(text: &str, from: &str, to: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for word in text.split_whitespace() {
        let word = if is_tag(word, from) { to } else { word };
        if !out.contains(&word) {
            out.push(word);
        }
    }
    out.join(" ")
}

/// The words of `text` that aren't among the post's `saved` tags: the ones
/// worth a "did you mean".
pub fn new_words(text: &str, saved: &[String]) -> String {
    text.split_whitespace()
        .filter(|word| !saved.iter().any(|s| is_tag(word, s)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The corrections still worth showing: those whose unknown tag is still in
/// the text.
pub fn shown_fixes(fixes: &[TagCorrection], text: &str) -> Vec<TagCorrection> {
    fixes
        .iter()
        .filter(|fix| text.split_whitespace().any(|w| is_tag(w, &fix.from)))
        .cloned()
        .collect()
}

/// The AI's suggestions that aren't in the text yet.
pub fn untyped(found: &[TagSuggestion], text: &str) -> Vec<TagSuggestion> {
    found
        .iter()
        .filter(|s| !text.split_whitespace().any(|w| is_tag(w, &s.name)))
        .cloned()
        .collect()
}

/// The tags to offer for the word being typed: not the ones in the text
/// already, with or without a category prefix.
pub fn offers(text: &str, hints: Vec<Hint>) -> Vec<Hint> {
    let have: Vec<String> = text
        .split_whitespace()
        .filter_map(|w| normalize_tag(w).ok())
        .map(|w| split_category(&w).1.to_owned())
        .collect();
    hints
        .into_iter()
        .filter(|h| !have.contains(&h.name))
        .collect()
}

// ---------- who may change what ----------

/// Whether `user` may change a post's tags and rating. The server lets
/// anyone signed in edit a post that isn't in the recycle bin.
pub fn may_edit_post(user: Option<&User>, deleted: bool) -> bool {
    user.is_some() && !deleted
}

/// A tag's category and its place in the tree: anyone signed in.
pub fn may_edit_tag(user: Option<&User>) -> bool {
    user.is_some()
}

/// Aliases and implications, which change every post with the tag:
/// administrators only.
pub fn may_relate_tags(user: Option<&User>) -> bool {
    user.is_some_and(|u| u.role == Role::Admin)
}

/// Whether making `typed` an alias of the tag with id `this` merges another
/// tag into it, for good: `found` is what the server has under that name,
/// and it's a tag of its own, not an alias of one.
pub fn merges(found: &Tag, typed: &str, this: i64) -> bool {
    found.id != this && normalize_tag(typed).is_ok_and(|name| name == found.name)
}

/// Whether the page asked for as `asked` shows another tag, because `asked`
/// is one of its aliases.
pub fn by_alias(asked: &str, shown: &str) -> bool {
    !normalize_tag(asked).is_ok_and(|name| name == shown)
}

// ---------- the Tags page ----------

/// Tags in a page of the list.
pub const TAG_PAGE: u32 = 60;

/// Pages in a row that may come without a tag to show (all of another
/// category) before the list waits for a press of Load more.
pub const QUIET_PAGES: u32 = 5;

/// One request for the tag list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagListRequest {
    /// Names and aliases starting with this; empty for every tag in use.
    pub prefix: String,
    pub order: TagOrder,
    /// From 1.
    pub page: u32,
    pub limit: u32,
}

/// The request for page `page` of the list filtered by `filter`. The
/// server reads the prefix as a tag, so `Blue sky` finds `blue_sky`; a minus
/// in front would only be dropped.
pub fn tag_list_request(filter: &str, order: TagOrder, page: u32) -> TagListRequest {
    TagListRequest {
        prefix: filter.trim().trim_start_matches('-').trim().to_owned(),
        order,
        page: page.max(1),
        limit: TAG_PAGE,
    }
}

/// Whether the list shows `tag` with the category chip `category` on; `None`
/// is All.
pub fn shows(tag: &Tag, category: Option<TagCategory>) -> bool {
    category.is_none_or(|c| tag.category == c)
}

/// After a page that added `shown` rows to the list, the pages in a row
/// that added none, and whether to go on to the next page by itself while
/// the end of the list is in sight.
pub fn go_on(quiet: u32, shown: usize) -> (u32, bool) {
    let quiet = if shown == 0 {
        quiet.saturating_add(1)
    } else {
        0
    };
    (quiet, quiet < QUIET_PAGES)
}

// ---------- a tag's menu ----------

/// The words of a search, a quoted phrase (`text:"two words"`) staying one.
pub fn words(q: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = None;
    let mut quoted = false;
    for (i, c) in q.char_indices() {
        if c == '"' {
            quoted = !quoted;
        }
        if c.is_whitespace() && !quoted {
            if let Some(from) = start.take() {
                out.extend(q.get(from..i));
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(from) = start {
        out.extend(q.get(from..));
    }
    out
}

/// `q` without `name`, whether it was wanted or left out, and then `word` at
/// the end.
fn with_word(q: &str, name: &str, word: &str) -> String {
    let excluded = format!("-{name}");
    let mut kept: Vec<&str> = words(q)
        .into_iter()
        .filter(|w| *w != name && *w != excluded)
        .collect();
    kept.push(word);
    kept.join(" ")
}

/// The search narrowed to posts that also have `name`.
pub fn include(q: &str, name: &str) -> String {
    with_word(q, name, name)
}

/// The search without the posts that have `name`.
pub fn exclude(q: &str, name: &str) -> String {
    with_word(q, name, &format!("-{name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(role: Role) -> User {
        User {
            id: 1,
            username: "ada".into(),
            role,
            created_at: chrono::Utc::now(),
        }
    }

    fn tag(name: &str, category: TagCategory) -> Tag {
        Tag {
            id: 1,
            name: name.into(),
            category,
            post_count: 3,
        }
    }

    fn hint(name: &str) -> Hint {
        Hint {
            name: name.into(),
            class: "",
            count: Some(3),
        }
    }

    #[test]
    fn the_text_becomes_tags_once_each_in_the_order_typed() {
        assert_eq!(
            parse_tags("  Blue_Sky fractal\nblue_sky  ARTIST:Lavfi\tfractal "),
            Ok(vec![
                "blue_sky".to_owned(),
                "fractal".to_owned(),
                "artist:lavfi".to_owned()
            ])
        );
        assert_eq!(parse_tags(""), Ok(Vec::new()));
        assert_eq!(parse_tags(" \n "), Ok(Vec::new()));
        // Underscores alone are no tag.
        assert_eq!(parse_tags("_ cat __"), Ok(vec!["cat".to_owned()]));
        // Repeated underscores collapse, as on the server.
        assert_eq!(parse_tags("a__b"), Ok(vec!["a_b".to_owned()]));
    }

    #[test]
    fn a_word_that_cant_be_a_tag_is_named() {
        assert_eq!(
            parse_tags("cat -dog"),
            Err(BadTag {
                word: "-dog".into(),
                error: TagError::BadPrefix("-dog".into())
            })
        );
        let long = "x".repeat(129);
        assert_eq!(
            parse_tags(&format!("cat {long}")),
            Err(BadTag {
                word: long,
                error: TagError::TooLong
            })
        );
        let many: Vec<String> = (0..=MAX_TAGS_PER_POST).map(|i| format!("t{i}")).collect();
        assert_eq!(
            parse_tags(&many.join(" ")).map_err(|e| e.error),
            Err(TagError::TooMany)
        );
        // The same tag over and over is still one.
        let again = vec!["t"; MAX_TAGS_PER_POST + 5].join(" ");
        assert_eq!(parse_tags(&again), Ok(vec!["t".to_owned()]));
    }

    #[test]
    fn a_fix_replaces_the_tag_and_an_addition_comes_once() {
        assert_eq!(replace_tag("blnde cat", "blnde", "blonde"), "blonde cat");
        assert_eq!(replace_tag("Blnde  cat ", "blnde", "blonde"), "blonde cat");
        assert_eq!(
            replace_tag("blnde blonde cat", "blnde", "blonde"),
            "blonde cat"
        );
        assert_eq!(replace_tag("cat", "blnde", "blonde"), "cat");
        assert_eq!(with_tag("", "sky"), "sky");
        assert_eq!(with_tag("cat  ", "sky"), "cat sky");
        assert_eq!(with_tag("cat sky", "sky"), "cat sky");
        assert_eq!(with_tag("cat Sky", "sky"), "cat Sky");
    }

    #[test]
    fn only_new_words_are_checked_and_only_fixes_still_wanted_shown() {
        let saved = ["cat".to_owned(), "sky".to_owned()];
        assert_eq!(new_words("cat  blnde Sky dgo ", &saved), "blnde dgo");
        assert_eq!(new_words("cat sky", &saved), "");
        let fix = |from: &str, to: &str| TagCorrection {
            from: from.into(),
            to: to.into(),
        };
        let fixes = [fix("fractl", "fractal"), fix("blu_sky", "blue_sky")];
        // One typo mended by hand, the other still there in capitals.
        assert_eq!(
            shown_fixes(&fixes, "fractal Blu_Sky"),
            [fix("blu_sky", "blue_sky")]
        );
        assert!(shown_fixes(&fixes, "").is_empty());
    }

    #[test]
    fn suggestions_already_typed_are_left_out() {
        let s = |name: &str| TagSuggestion {
            name: name.into(),
            category: TagCategory::General,
            exists: true,
        };
        let left = untyped(&[s("sky"), s("cloud"), s("sun")], "Sky  sun_");
        assert_eq!(left, [s("cloud")]);
    }

    #[test]
    fn tags_already_in_the_box_are_not_offered() {
        let left = offers(
            "artist:mika Blu",
            vec![hint("mika"), hint("blue_sky"), hint("blu")],
        );
        let names: Vec<&str> = left.iter().map(|h| h.name.as_str()).collect();
        assert_eq!(names, ["blue_sky"]);
    }

    #[test]
    fn anyone_signed_in_edits_a_live_post_and_admins_relate_tags() {
        let admin = user(Role::Admin);
        let member = user(Role::User);
        assert!(may_edit_post(Some(&member), false));
        assert!(may_edit_post(Some(&admin), false));
        // In the recycle bin, the server says to restore it first.
        assert!(!may_edit_post(Some(&admin), true));
        assert!(!may_edit_post(None, false));
        assert!(may_edit_tag(Some(&member)));
        assert!(!may_edit_tag(None));
        assert!(may_relate_tags(Some(&admin)));
        assert!(!may_relate_tags(Some(&member)));
        assert!(!may_relate_tags(None));
    }

    #[test]
    fn an_alias_that_is_a_tag_already_is_a_merge() {
        let other = Tag {
            id: 9,
            ..tag("fractals", TagCategory::General)
        };
        assert!(merges(&other, " Fractals ", 1));
        // Under that name the server found this tag, or another through one
        // of its aliases: nothing merges.
        assert!(!merges(&other, "fractals", 9));
        assert!(!merges(&other, "fractal_art", 1));
        assert!(!merges(&other, "-", 1));
        assert!(!by_alias("Blue sky", "blue_sky"));
        assert!(by_alias("sky_blue", "blue_sky"));
    }

    #[test]
    fn the_tags_page_asks_for_a_page_by_prefix_and_order() {
        assert_eq!(
            tag_list_request("", TagOrder::Count, 1),
            TagListRequest {
                prefix: String::new(),
                order: TagOrder::Count,
                page: 1,
                limit: TAG_PAGE
            }
        );
        let r = tag_list_request("  Blue sky ", TagOrder::Name, 3);
        assert_eq!(
            (r.prefix.as_str(), r.order, r.page),
            ("Blue sky", TagOrder::Name, 3)
        );
        assert_eq!(tag_list_request("- cat", TagOrder::Count, 0).prefix, "cat");
        assert_eq!(tag_list_request("x", TagOrder::Count, 0).page, 1);
    }

    #[test]
    fn the_category_chips_filter_what_came_and_stop_asking_in_time() {
        let artist = tag("lavfi", TagCategory::Artist);
        assert!(shows(&artist, None));
        assert!(shows(&artist, Some(TagCategory::Artist)));
        assert!(!shows(&artist, Some(TagCategory::Meta)));
        assert_eq!(go_on(3, 12), (0, true));
        assert_eq!(go_on(0, 0), (1, true));
        assert_eq!(go_on(QUIET_PAGES - 1, 0), (QUIET_PAGES, false));
        assert_eq!(go_on(u32::MAX, 0), (u32::MAX, false));
    }

    #[test]
    fn a_quoted_phrase_is_one_word() {
        assert_eq!(
            words("  blue  text:\"two words\" -red "),
            ["blue", "text:\"two words\"", "-red"]
        );
        assert!(words("").is_empty());
        // A quote that never closes runs to the end.
        assert_eq!(words("a \"b c"), ["a", "\"b c"]);
    }

    #[test]
    fn a_tag_narrows_or_leaves_out_of_the_search() {
        assert_eq!(include("", "sky"), "sky");
        assert_eq!(include("blue order:new", "sky"), "blue order:new sky");
        // Left out before: now wanted.
        assert_eq!(include("blue -sky", "sky"), "blue sky");
        assert_eq!(exclude("blue sky", "sky"), "blue -sky");
        assert_eq!(exclude("", "sky"), "-sky");
        assert_eq!(exclude("-sky blue", "sky"), "blue -sky");
        // In a phrase, the word is the phrase's.
        assert_eq!(exclude("text:\"a sky\"", "sky"), "text:\"a sky\" -sky");
        assert_eq!(include("skyline", "sky"), "skyline sky");
    }
}
