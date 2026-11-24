//! Votes: up, down or none, or a number of stars, and the tallies that go
//! with them, so the viewer can show a vote before the server has it and
//! take it back if it fails.

use starbooru_core::Post;

use super::stars::{liked, restarred};

/// The vote after pressing up (`1`) or down (`-1`): pressing the vote already
/// cast takes it back.
pub fn next_vote(current: i16, pressed: i16) -> i16 {
    let pressed = pressed.signum();
    if current.signum() == pressed {
        0
    } else {
        pressed
    }
}

/// The score once vote `old` is replaced by `new`.
pub fn rescored(score: i32, old: i16, new: i16) -> i32 {
    score
        .saturating_sub(i32::from(old.signum()))
        .saturating_add(i32::from(new.signum()))
}

/// A vote to send: a like (1), a dislike (-1) or none (0), or on a server
/// that counts stars, 1 to 5 of them or none (0).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ballot {
    Like(i16),
    Stars(u8),
}

/// What a post shows of its votes: enough to show a vote at once, and to
/// put back what was there if the server turns it down.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tally {
    pub score: i32,
    pub my_vote: i16,
    pub stars: Option<f64>,
    pub votes: u32,
    pub my_stars: u8,
}

impl Tally {
    pub fn of(post: &Post) -> Tally {
        Tally {
            score: post.score,
            my_vote: post.my_vote,
            stars: post.stars,
            votes: post.votes,
            my_stars: post.my_stars,
        }
    }

    pub fn put(self, post: &mut Post) {
        post.score = self.score;
        post.my_vote = self.my_vote;
        post.stars = self.stars;
        post.votes = self.votes;
        post.my_stars = self.my_stars;
    }

    /// The tally once `ballot` takes the place of the vote in it.
    pub fn after(self, ballot: Ballot) -> Tally {
        match ballot {
            Ballot::Like(new) => Tally {
                score: rescored(self.score, self.my_vote, new),
                my_vote: new.signum(),
                ..self
            },
            Ballot::Stars(new) => {
                let (stars, votes) = restarred(self.stars, self.votes, self.my_stars, new);
                let (score, my_vote) = liked(self.score, self.my_stars, new);
                Tally {
                    score,
                    my_vote,
                    stars,
                    votes,
                    my_stars: new,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ballot_shows_at_once_in_the_tally() {
        let likes = Tally {
            score: 3,
            my_vote: 0,
            stars: None,
            votes: 0,
            my_stars: 0,
        };
        assert_eq!(likes.after(Ballot::Like(1)).score, 4);
        assert_eq!(likes.after(Ballot::Like(1)).my_vote, 1);
        assert_eq!(likes.after(Ballot::Like(-1)).after(Ballot::Like(0)), likes);
        // 13 stars over 3 votes, 2 of them 4 or more.
        let stars = Tally {
            score: 2,
            my_vote: 0,
            stars: Some(4.33),
            votes: 3,
            my_stars: 0,
        };
        let given = stars.after(Ballot::Stars(5));
        assert_eq!(
            given,
            Tally {
                score: 3,
                my_vote: 1,
                stars: Some(4.5),
                votes: 4,
                my_stars: 5,
            }
        );
        let back = given.after(Ballot::Stars(0));
        assert_eq!((back.score, back.my_vote, back.votes), (2, 0, 3));
        assert_eq!(back.stars, Some(4.33));
    }

    #[test]
    fn a_tally_goes_into_a_post_and_comes_back() {
        let mut post: Post = serde_json::from_value(serde_json::json!({
            "id": 1, "kind": "image", "mime": "image/png", "file_size": 1, "width": 1,
            "height": 1, "sha256": "00", "rating": "safe", "score": 1, "my_vote": 1,
            "stars": 4.0, "votes": 1, "my_stars": 4, "source": null, "tags": [],
            "created_at": "2026-11-24T00:00:00Z", "deleted": false, "file_url": "/media/x",
            "thumb_url": null
        }))
        .unwrap();
        let tally = Tally::of(&post);
        assert_eq!(
            (tally.stars, tally.votes, tally.my_stars),
            (Some(4.0), 1, 4)
        );
        tally.after(Ballot::Stars(0)).put(&mut post);
        assert_eq!(
            (post.score, post.my_vote, post.votes, post.my_stars),
            (0, 0, 0, 0)
        );
        assert_eq!(post.stars, None);
    }

    #[test]
    fn pressing_a_vote_casts_it_or_takes_it_back() {
        assert_eq!(next_vote(0, 1), 1);
        assert_eq!(next_vote(0, -1), -1);
        assert_eq!(next_vote(1, 1), 0);
        assert_eq!(next_vote(-1, -1), 0);
        // The other way turns it round.
        assert_eq!(next_vote(1, -1), -1);
        assert_eq!(next_vote(-1, 1), 1);
        // Odd values count by their sign.
        assert_eq!(next_vote(5, 1), 0);
        assert_eq!(next_vote(0, 3), 1);
    }

    #[test]
    fn the_score_follows_the_vote() {
        assert_eq!(rescored(3, 0, 1), 4);
        assert_eq!(rescored(3, 1, 0), 2);
        assert_eq!(rescored(3, 1, -1), 1);
        assert_eq!(rescored(-2, -1, 1), 0);
        assert_eq!(rescored(0, 0, 0), 0);
        assert_eq!(rescored(i32::MAX, 0, 1), i32::MAX);
        assert_eq!(rescored(i32::MIN, 1, 0), i32::MIN);
        // Undoing a press comes back to where it started.
        for (score, old) in [(0, 0), (7, 1), (-3, -1)] {
            for pressed in [1, -1] {
                let new = next_vote(old, pressed);
                let back = rescored(rescored(score, old, new), new, old);
                assert_eq!(back, score);
            }
        }
    }
}
