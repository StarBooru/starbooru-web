//! The forum without a browser: a post's text cut where it names posts of
//! the booru, the pages of a list put together, when someone may post
//! again, who may do what with a thread or a post, the checks of what's
//! typed, and the words for when something was written.

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use starbooru_core::forum::{
    MAX_BODY_CHARS, MAX_CATEGORY_DESCRIPTION_CHARS, MAX_CATEGORY_NAME_CHARS, MAX_TITLE_CHARS,
    POST_INTERVAL_SECS, POSTS_PER_DAY, clean_body, clean_category_name, clean_description,
    clean_title,
};
use starbooru_core::{ForumError, ForumPost, ForumThread, UserCard, post_mentions};

/// A piece of a post's text: words as written, or a post it names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
    Text(String),
    /// `>>123` or `post #123`, as written, and the post's id.
    Mention(i64, String),
}

/// `text` cut where it names posts of the booru, through the server's own
/// reading (`starbooru_core::post_mentions`).
pub fn segments(text: &str) -> Vec<Segment> {
    let mut out = Vec::new();
    let mut at = 0;
    for m in post_mentions(text) {
        let (Some(before), Some(named)) = (text.get(at..m.start), text.get(m.start..m.end)) else {
            continue;
        };
        if !before.is_empty() {
            out.push(Segment::Text(before.to_owned()));
        }
        out.push(Segment::Mention(m.id, named.to_owned()));
        at = m.end;
    }
    match text.get(at..) {
        Some(rest) if !rest.is_empty() => out.push(Segment::Text(rest.to_owned())),
        _ => {}
    }
    out
}

/// The threads a list has once `more` came, in the server's order, each
/// once: a thread that moved up while the pages were read shows where it
/// came first.
pub fn merge_threads(have: &mut Vec<ForumThread>, more: Vec<ForumThread>) {
    let known: HashSet<i64> = have.iter().map(|t| t.id).collect();
    have.extend(more.into_iter().filter(|t| !known.contains(&t.id)));
}

/// The same for a thread's posts, oldest first. A post read again replaces
/// the copy kept, as it may have been edited or deleted since.
pub fn merge_posts(have: &mut Vec<ForumPost>, more: Vec<ForumPost>) {
    for post in more {
        match have.iter_mut().find(|p| p.id == post.id) {
            Some(kept) => *kept = post,
            None => have.push(post),
        }
    }
    have.sort_by_key(|p| (p.created_at, p.id));
}

/// The page after `page`, when there is one.
pub fn next_page(page: u32, has_more: bool) -> Option<u32> {
    has_more.then(|| page.saturating_add(1))
}

/// Seconds before the account may post again, from its last post (written
/// `last`), as the server counts them. Administrators don't wait.
pub fn wait_left(last: Option<DateTime<Utc>>, now: DateTime<Utc>, admin: bool) -> Option<u64> {
    if admin {
        return None;
    }
    let age = (now - last?).num_seconds();
    // A clock set back makes a post from the future: nothing to wait for.
    (0..POST_INTERVAL_SECS)
        .contains(&age)
        .then(|| u64::try_from(POST_INTERVAL_SECS - age).unwrap_or(1))
}

/// The newest post `me` wrote among `posts`.
pub fn latest_by(posts: &[ForumPost], me: &str) -> Option<DateTime<Utc>> {
    posts
        .iter()
        .filter(|p| is_by(p.author.as_ref(), me))
        .map(|p| p.created_at)
        .max()
}

/// Whether `card` is the account named `me`, whatever the case.
pub fn is_by(card: Option<&UserCard>, me: &str) -> bool {
    card.is_some_and(|c| c.name.eq_ignore_ascii_case(me))
}

/// What the end of a thread offers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reply {
    /// A box to write in.
    Open,
    /// Only administrators post in a locked thread.
    Locked,
    /// A guest is asked to sign in.
    SignIn,
}

pub fn reply(thread: &ForumThread, signed_in: bool, admin: bool) -> Reply {
    match (signed_in, thread.locked && !admin) {
        (false, _) => Reply::SignIn,
        (true, true) => Reply::Locked,
        (true, false) => Reply::Open,
    }
}

/// Its author edits a post, unless it's deleted or the thread locked.
pub fn may_edit(post: &ForumPost, me: Option<&str>, locked: bool) -> bool {
    !post.deleted && !locked && me.is_some_and(|me| is_by(post.author.as_ref(), me))
}

/// Its author or an administrator deletes it.
pub fn may_delete(post: &ForumPost, me: Option<&str>, admin: bool) -> bool {
    !post.deleted && (admin || me.is_some_and(|me| is_by(post.author.as_ref(), me)))
}

/// Its author renames a thread while it's open; an administrator always.
pub fn may_rename(thread: &ForumThread, me: Option<&str>, admin: bool) -> bool {
    admin || (!thread.locked && me.is_some_and(|me| is_by(thread.author.as_ref(), me)))
}

/// The positions two neighbouring categories take to trade places: the one
/// moving (at `mine`) and the one it passes (at `theirs`). Equal positions
/// sort by name, so they're pulled apart first.
pub fn swapped(mine: i32, theirs: i32, up: bool) -> (i32, i32) {
    if mine != theirs {
        return (theirs, mine);
    }
    let low = mine.saturating_sub(i32::from(!up));
    if up {
        (low, low.saturating_add(1))
    } else {
        (low.saturating_add(1), low)
    }
}

/// A thread's title, cleaned as the server cleans it.
pub fn check_title(typed: &str) -> Result<String, ForumError> {
    clean_title(typed)
}

/// A post's text, its lines kept.
pub fn check_body(typed: &str) -> Result<String, ForumError> {
    clean_body(typed)
}

/// A category's name and description.
pub fn check_category(name: &str, description: &str) -> Result<(String, String), ForumError> {
    Ok((clean_category_name(name)?, clean_description(description)?))
}

pub fn error_text(e: &ForumError) -> String {
    match e {
        ForumError::EmptyTitle => t!("forum-title-empty"),
        ForumError::TitleTooLong => t!("forum-title-long", max = MAX_TITLE_CHARS),
        ForumError::EmptyBody => t!("forum-body-empty"),
        ForumError::BodyTooLong => t!("forum-body-long", max = MAX_BODY_CHARS),
        ForumError::EmptyName => t!("forum-category-name-empty"),
        ForumError::NameTooLong => t!("forum-category-name-long", max = MAX_CATEGORY_NAME_CHARS),
        ForumError::DescriptionTooLong => t!(
            "forum-category-description-long",
            max = MAX_CATEGORY_DESCRIPTION_CHARS
        ),
        ForumError::TitleControlChar
        | ForumError::BodyControlChar
        | ForumError::NameControlChar
        | ForumError::DescriptionControlChar => t!("profile-bad-character"),
    }
}

/// What a post the server turned away as too soon says: how long until the
/// account may post again, in seconds, minutes or hours.
pub fn slow_down(retry_after: Option<u64>) -> String {
    match retry_after {
        None => t!("forum-slow-down"),
        Some(s) if s < 60 => t!("forum-slow-down-seconds", seconds = s.max(1)),
        // Past the 15 seconds between posts: the day's are used up.
        Some(s) if s < 3_600 => t!(
            "forum-slow-down-minutes",
            minutes = s.div_ceil(60),
            posts = POSTS_PER_DAY
        ),
        Some(s) => t!(
            "forum-slow-down-hours",
            hours = s.div_ceil(3_600),
            posts = POSTS_PER_DAY
        ),
    }
}

/// How the end of a typed post is counted against the limit: what's left,
/// once it's close.
pub fn chars_left(typed: &str, max: usize) -> Option<i64> {
    let used = typed.trim().chars().count();
    let left = i64::try_from(max).unwrap_or(i64::MAX) - i64::try_from(used).unwrap_or(i64::MAX);
    (used * 10 >= max * 9).then_some(left)
}

/// What's left of the limit, in words: "12 characters left", or once past
/// it "3 characters too many".
pub fn left_words(left: i64) -> String {
    if left >= 0 {
        t!("profile-chars-left", count = left)
    } else {
        t!("profile-chars-over", count = left.saturating_neg())
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::from_timestamp(1_800_000_000, 0).unwrap()
    }

    fn card(name: &str) -> UserCard {
        UserCard {
            name: name.into(),
            display_name: None,
            avatar_url: None,
            avatar_small_url: None,
        }
    }

    fn post(id: i64, by: Option<&str>, secs_ago: i64) -> ForumPost {
        ForumPost {
            id,
            thread_id: 1,
            author: by.map(card),
            body: "hi".into(),
            created_at: now() - TimeDelta::seconds(secs_ago),
            edited_at: None,
            deleted: false,
        }
    }

    fn thread(id: i64, locked: bool) -> ForumThread {
        ForumThread {
            id,
            category_id: 1,
            title: "t".into(),
            author: Some(card("ann")),
            created_at: now(),
            last_activity_at: now() - TimeDelta::minutes(5),
            posts: 3,
            pinned: false,
            locked,
        }
    }

    #[test]
    fn a_post_is_cut_where_it_names_posts() {
        assert_eq!(
            segments("See >>12, and post #7."),
            [
                Segment::Text("See ".into()),
                Segment::Mention(12, ">>12".into()),
                Segment::Text(", and ".into()),
                Segment::Mention(7, "post #7".into()),
                Segment::Text(".".into()),
            ]
        );
        assert_eq!(segments(">>5"), [Segment::Mention(5, ">>5".into())]);
        assert_eq!(
            segments("ciò >>5 è"),
            [
                Segment::Text("ciò ".into()),
                Segment::Mention(5, ">>5".into()),
                Segment::Text(" è".into()),
            ]
        );
        // Not ids, as the server reads them.
        assert_eq!(
            segments("repost #4 >>x"),
            [Segment::Text("repost #4 >>x".into())]
        );
        assert!(segments("").is_empty());
    }

    #[test]
    fn pages_join_without_twice_the_same() {
        let mut have = vec![thread(1, false), thread(2, false)];
        merge_threads(&mut have, vec![thread(2, false), thread(3, false)]);
        let ids: Vec<i64> = have.iter().map(|t| t.id).collect();
        assert_eq!(ids, [1, 2, 3]);
        let mut posts = vec![post(1, Some("ann"), 30), post(2, Some("bob"), 20)];
        let mut gone = post(2, None, 20);
        gone.deleted = true;
        merge_posts(&mut posts, vec![post(3, Some("ann"), 10), gone]);
        let ids: Vec<i64> = posts.iter().map(|p| p.id).collect();
        assert_eq!(ids, [1, 2, 3]);
        assert!(posts.get(1).is_some_and(|p| p.deleted));
        assert_eq!(next_page(1, true), Some(2));
        assert_eq!(next_page(1, false), None);
        assert_eq!(next_page(u32::MAX, true), Some(u32::MAX));
    }

    #[test]
    fn the_wait_after_a_post_counts_down_from_fifteen_seconds() {
        let at = |secs| Some(now() - TimeDelta::seconds(secs));
        assert_eq!(wait_left(at(0), now(), false), Some(15));
        assert_eq!(wait_left(at(3), now(), false), Some(12));
        assert_eq!(wait_left(at(14), now(), false), Some(1));
        assert_eq!(wait_left(at(15), now(), false), None);
        assert_eq!(wait_left(None, now(), false), None);
        // From the future, or an administrator's: no wait.
        assert_eq!(wait_left(at(-30), now(), false), None);
        assert_eq!(wait_left(at(3), now(), true), None);
        let posts = [
            post(1, Some("Ann"), 40),
            post(2, Some("bob"), 2),
            post(3, Some("ann"), 9),
        ];
        assert_eq!(latest_by(&posts, "ann"), at(9));
        assert_eq!(latest_by(&posts, "eve"), None);
    }

    #[test]
    fn who_may_do_what() {
        let open = thread(1, false);
        let locked = thread(1, true);
        assert_eq!(reply(&open, true, false), Reply::Open);
        assert_eq!(reply(&locked, true, false), Reply::Locked);
        assert_eq!(reply(&locked, true, true), Reply::Open);
        assert_eq!(reply(&open, false, false), Reply::SignIn);
        let mine = post(1, Some("ann"), 0);
        assert!(may_edit(&mine, Some("ANN"), false));
        assert!(!may_edit(&mine, Some("ann"), true));
        assert!(!may_edit(&mine, Some("bob"), false));
        assert!(!may_edit(&mine, None, false));
        assert!(may_delete(&mine, Some("ann"), false));
        assert!(may_delete(&mine, Some("bob"), true));
        assert!(!may_delete(&mine, Some("bob"), false));
        let mut gone = mine.clone();
        gone.deleted = true;
        assert!(!may_edit(&gone, Some("ann"), false) && !may_delete(&gone, Some("ann"), true));
        // An account deleted leaves its posts to no one.
        assert!(!may_edit(&post(2, None, 0), Some("ann"), false));
        assert!(may_rename(&open, Some("ann"), false));
        assert!(!may_rename(&locked, Some("ann"), false));
        assert!(may_rename(&locked, Some("bob"), true));
        assert!(!may_rename(&open, Some("bob"), false));
    }

    #[test]
    fn categories_trade_places() {
        assert_eq!(swapped(2, 1, true), (1, 2));
        assert_eq!(swapped(1, 2, false), (2, 1));
        // Equal: the one going up ends before, the one going down after.
        let (me, them) = swapped(4, 4, true);
        assert!(me < them);
        let (me, them) = swapped(4, 4, false);
        assert!(me > them);
        assert_eq!(swapped(i32::MAX, i32::MAX, false).0, i32::MAX);
    }

    #[test]
    fn what_is_typed_is_checked_as_the_server_checks_it() {
        assert_eq!(check_title("  Best   tags "), Ok("Best tags".into()));
        assert_eq!(check_title(" "), Err(ForumError::EmptyTitle));
        assert_eq!(check_body(" \n "), Err(ForumError::EmptyBody));
        assert_eq!(check_body("\r\n# Hi\r\n"), Ok("# Hi".into()));
        assert_eq!(
            check_category(" Art ", " Pictures "),
            Ok(("Art".into(), "Pictures".into()))
        );
        assert_eq!(check_category("", "x"), Err(ForumError::EmptyName));
        assert!(error_text(&ForumError::TitleTooLong).contains("120"));
        assert!(
            error_text(&ForumError::BodyTooLong).contains("10000")
                || error_text(&ForumError::BodyTooLong).contains("10,000")
        );
    }

    #[test]
    fn a_post_turned_away_says_when_to_try_again() {
        assert_eq!(slow_down(Some(12)), "You can post again in 12 seconds.");
        assert_eq!(slow_down(Some(0)), "You can post again in 1 second.");
        assert_eq!(
            slow_down(Some(61)),
            "That's 100 posts today. You can post again in 2 minutes."
        );
        assert_eq!(
            slow_down(Some(3 * 3_600 + 5)),
            "That's 100 posts today. You can post again in 4 hours."
        );
        assert!(slow_down(None).contains("again"));
    }

    #[test]
    fn the_count_of_whats_left_shows_near_the_limit() {
        assert_eq!(chars_left("abc", 10), None);
        assert_eq!(chars_left(&"x".repeat(9), 10), Some(1));
        assert_eq!(chars_left(&"x".repeat(12), 10), Some(-2));
        assert_eq!(left_words(1), "1 character left");
        assert_eq!(left_words(0), "0 characters left");
        assert_eq!(left_words(-2), "2 characters too many");
    }
}
