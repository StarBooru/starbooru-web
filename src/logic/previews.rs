//! A search's first post, as a small picture of what the search finds: the
//! Tags page shows one beside each tag, and saved searches one each. They are
//! kept for a while per account, so scrolling back or coming again asks the
//! server nothing.

use chrono::{DateTime, Utc};
use starbooru_core::Post;

use super::gallery::tint;
use super::saved::tidy;

/// How long a preview is kept. The server's signed addresses last hours.
const KEPT_MINUTES: i64 = 30;
/// How many are kept at most, the oldest going first.
const KEPT: usize = 600;

/// What a search's first post looks like.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preview {
    pub id: i64,
    /// The thumbnail's address on the server, if the post has one.
    pub thumb: Option<String>,
    /// The picture's main colour, shown while the thumbnail comes.
    pub tint: Option<String>,
}

impl Preview {
    pub fn of(post: &Post) -> Self {
        Self {
            id: post.id,
            thumb: post.thumb_url.clone(),
            tint: tint(&post.palette).map(str::to_owned),
        }
    }
}

/// Previews by search, for one account.
#[derive(Debug, Default)]
pub struct Previews {
    owner: String,
    /// The latest last. `None` is a search that finds nothing.
    kept: Vec<(String, DateTime<Utc>, Option<Preview>)>,
}

impl Previews {
    /// What search `q` showed, if it was asked lately: `Some(None)` when it
    /// found nothing.
    pub fn get(&self, owner: &str, q: &str, now: DateTime<Utc>) -> Option<Option<Preview>> {
        if self.owner != owner {
            return None;
        }
        let key = tidy(q);
        self.kept
            .iter()
            .find(|(was, at, _)| *was == key && fresh(*at, now))
            .map(|(.., found)| found.clone())
    }

    pub fn put(&mut self, owner: &str, q: &str, found: Option<Preview>, now: DateTime<Utc>) {
        if self.owner != owner {
            owner.clone_into(&mut self.owner);
            self.kept.clear();
        }
        let key = tidy(q);
        self.kept
            .retain(|(was, at, _)| *was != key && fresh(*at, now));
        self.kept.push((key, now, found));
        let extra = self.kept.len().saturating_sub(KEPT);
        self.kept.drain(..extra);
    }

    /// Forgets them all, after a change that can move posts between searches.
    pub fn clear(&mut self) {
        self.kept.clear();
    }
}

fn fresh(at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    (0..KEPT_MINUTES).contains(&(now - at).num_minutes())
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    fn preview(id: i64) -> Option<Preview> {
        Some(Preview {
            id,
            thumb: Some(format!("/media/thumb/{id}.webp")),
            tint: None,
        })
    }

    #[test]
    fn a_preview_is_kept_for_a_while_for_its_account() {
        let now = Utc::now();
        let mut kept = Previews::default();
        assert_eq!(kept.get("me", "cat", now), None);
        kept.put("me", "cat", preview(1), now);
        kept.put("me", "nothing_here", None, now);
        assert_eq!(kept.get("me", "cat", now), Some(preview(1)));
        // However the search was spaced.
        assert_eq!(kept.get("me", "  cat ", now), Some(preview(1)));
        // Known to find nothing is not the same as not known.
        assert_eq!(kept.get("me", "nothing_here", now), Some(None));
        assert_eq!(kept.get("you", "cat", now), None);
        let later = now + Duration::minutes(KEPT_MINUTES);
        assert_eq!(kept.get("me", "cat", later), None);
        // Another account starts over.
        kept.put("you", "dog", preview(2), now);
        assert_eq!(kept.get("me", "cat", now), None);
        assert_eq!(kept.get("you", "dog", now), Some(preview(2)));
        kept.clear();
        assert_eq!(kept.get("you", "dog", now), None);
    }

    #[test]
    fn the_oldest_go_first_and_a_search_is_kept_once() {
        let now = Utc::now();
        let mut kept = Previews::default();
        for i in 0..=KEPT {
            kept.put("me", &format!("t{i}"), preview(1), now);
        }
        assert_eq!(kept.kept.len(), KEPT);
        assert_eq!(kept.get("me", "t0", now), None);
        assert!(kept.get("me", "t1", now).is_some());
        kept.put("me", "t1", preview(9), now);
        assert_eq!(kept.kept.len(), KEPT);
        assert_eq!(kept.get("me", "t1", now), Some(preview(9)));
    }
}
