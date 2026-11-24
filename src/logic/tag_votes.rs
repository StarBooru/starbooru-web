//! Votes on a post's tags, on a public server, without a browser: what a
//! press sends, the score shown before the server answers, the post put
//! together from the answer, and the tags the answer took off.

use starbooru_core::{Post, TagVote};

use super::votes::next_vote;

/// What a press on up (`1`) or down (`-1`) sends for a tag voted `mine`:
/// the same press again takes the vote back.
pub fn next(mine: i16, pressed: i16) -> i16 {
    next_vote(mine, pressed)
}

/// The tag's votes once the caller's own becomes `mine`.
pub fn after(vote: TagVote, mine: i16) -> TagVote {
    let mine = mine.signum();
    TagVote {
        score: vote
            .score
            .saturating_sub(i32::from(vote.mine.signum()))
            .saturating_add(i32::from(mine)),
        mine,
    }
}

/// A tag's votes on `post`: none when it has none.
pub fn vote_of(post: &Post, tag: &str) -> TagVote {
    post.tag_votes.get(tag).copied().unwrap_or_default()
}

/// Whether the tag shows a score: once anyone has voted on it.
pub fn has_votes(post: &Post, tag: &str) -> bool {
    post.tag_votes.contains_key(tag)
}

/// The score as the tag shows it: `+2`, `−1` (a real minus) or `0`.
pub fn score_text(score: i32) -> String {
    match score {
        1.. => format!("+{score}"),
        0 => "0".to_owned(),
        _ => format!("\u{2212}{}", score.unsigned_abs()),
    }
}

/// The post from a vote's answer. It keeps what the answer may leave out
/// (the tags' groups, the people, the text read in the picture), for the
/// tags still on it.
pub fn merged(shown: &Post, answer: Post) -> Post {
    let mut post = answer;
    if post.tag_details.is_empty() {
        post.tag_details = shown
            .tag_details
            .iter()
            .filter(|t| post.tags.contains(&t.name))
            .cloned()
            .collect();
    }
    if post.figures.is_empty() {
        post.figures.clone_from(&shown.figures);
    }
    if post.ocr_text.is_none() {
        post.ocr_text.clone_from(&shown.ocr_text);
    }
    post
}

/// The tags `after` lost to the votes: on `before`, voted off now.
pub fn taken_off(before: &Post, after: &Post) -> Vec<String> {
    after
        .voted_off
        .iter()
        .filter(|t| before.tags.contains(t) && !after.tags.contains(t))
        .cloned()
        .collect()
}

/// `post` without `tags`, as it shows once they've left the card.
pub fn without(post: &Post, tags: &[String]) -> Post {
    let mut post = post.clone();
    post.tags.retain(|t| !tags.contains(t));
    post.tag_details.retain(|t| !tags.contains(&t.name));
    for tag in tags {
        post.tag_votes.remove(tag);
    }
    post
}

#[cfg(test)]
mod tests {
    use starbooru_core::{PostTag, TagCategory};

    use super::*;

    fn post() -> Post {
        let mut post: Post = serde_json::from_value(serde_json::json!({
            "id": 7, "kind": "image", "mime": "image/png", "file_size": 10,
            "width": 8, "height": 8, "duration_ms": null, "sha256": "ab",
            "rating": "safe", "score": 1, "my_vote": 0, "source": null,
            "tags": ["cat", "sky"], "created_at": "2026-11-24T00:00:00Z", "file_date": null,
            "deleted": false, "file_url": "/media/original/ab.png", "thumb_url": null,
            "sample_url": null,
            "tag_votes": { "sky": { "score": 2, "mine": 1 } }
        }))
        .unwrap();
        post.tag_details = ["cat", "sky"]
            .into_iter()
            .map(|name| PostTag {
                name: name.into(),
                category: TagCategory::General,
                section: None,
            })
            .collect();
        post
    }

    #[test]
    fn pressing_your_vote_again_takes_it_back() {
        assert_eq!(next(0, 1), 1);
        assert_eq!(next(1, 1), 0);
        assert_eq!(next(1, -1), -1);
        assert_eq!(next(-1, -1), 0);
        let vote = TagVote { score: 2, mine: 1 };
        assert_eq!(after(vote, 0), TagVote { score: 1, mine: 0 });
        assert_eq!(after(vote, -1), TagVote { score: 0, mine: -1 });
        assert_eq!(after(TagVote::default(), 1), TagVote { score: 1, mine: 1 });
        let edge = TagVote {
            score: i32::MIN,
            mine: 0,
        };
        assert_eq!(after(edge, -1).score, i32::MIN);
    }

    #[test]
    fn a_tag_shows_its_score_once_it_has_votes() {
        let p = post();
        assert!(has_votes(&p, "sky") && !has_votes(&p, "cat"));
        assert_eq!(vote_of(&p, "cat"), TagVote::default());
        assert_eq!(score_text(2), "+2");
        assert_eq!(score_text(0), "0");
        assert_eq!(score_text(-3), "\u{2212}3");
    }

    #[test]
    fn the_answer_keeps_what_it_leaves_out_and_says_what_came_off() {
        let before = post();
        let mut answer = before.clone();
        answer.tag_details.clear();
        answer.ocr_text = None;
        answer.tags = vec!["sky".into()];
        answer.voted_off = vec!["cat".into()];
        let mut shown = before.clone();
        shown.ocr_text = Some("hello".into());
        let post = merged(&shown, answer);
        let names: Vec<&str> = post.tag_details.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, ["sky"]);
        assert_eq!(post.ocr_text.as_deref(), Some("hello"));
        assert_eq!(taken_off(&before, &post), ["cat"]);
        // Off already before this vote: not news.
        assert!(taken_off(&post, &post).is_empty());
        let gone = without(&before, &["cat".into()]);
        assert_eq!(gone.tags, ["sky"]);
        assert_eq!(gone.tag_details.len(), 1);
    }
}
