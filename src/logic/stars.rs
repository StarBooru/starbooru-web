//! Stars, on a server that counts them in place of likes: what a press on
//! the five stars does, where the keys move along them, how a post's
//! average reads, and the tallies to show before the server answers.

use starbooru_core::{MAX_STARS, like_of};

use super::votes::rescored;

/// The vote after pressing star `pressed` (1 to 5): pressing the one already
/// given takes the vote back.
pub fn next_stars(current: u8, pressed: u8) -> u8 {
    let pressed = pressed.clamp(1, MAX_STARS);
    if pressed == current { 0 } else { pressed }
}

/// The star the keyboard starts on: the one given, else the first.
pub fn tab_stop(mine: u8) -> u8 {
    mine.clamp(1, MAX_STARS)
}

/// The star a key moves to from star `at`; `None` for keys that don't move.
pub fn moved(key: &str, at: u8) -> Option<u8> {
    let at = at.clamp(1, MAX_STARS);
    match key {
        "ArrowRight" | "ArrowUp" => Some(at.saturating_add(1).min(MAX_STARS)),
        "ArrowLeft" | "ArrowDown" => Some(at.saturating_sub(1).max(1)),
        "Home" => Some(1),
        "End" => Some(MAX_STARS),
        _ => None,
    }
}

/// Whether star `star` is lit: up to the one pointed at, else up to the
/// vote given.
pub fn lit(star: u8, mine: u8, pointed: Option<u8>) -> bool {
    star <= pointed.unwrap_or(mine)
}

/// A post's average, to one decimal, with the decimal sign of the language.
pub fn average(stars: f64, comma: bool) -> String {
    let stars = if stars.is_finite() { stars } else { 0.0 };
    let text = format!("{:.1}", stars.clamp(0.0, f64::from(MAX_STARS)));
    if comma { text.replace('.', ",") } else { text }
}

/// A star's name: "4 of 5 stars".
pub fn star_label(star: u8) -> String {
    t!("stars-of", count = star.min(MAX_STARS))
}

/// What the row of stars says it holds: the vote given, or none yet.
pub fn mine_label(mine: u8) -> String {
    if mine == 0 {
        t!("stars-none-yours")
    } else {
        t!("stars-yours", stars = star_label(mine))
    }
}

/// The post's average and how many voted, in a sentence, or that no one has
/// yet.
pub fn summary(stars: Option<f64>, votes: u32, comma: bool) -> String {
    match stars.filter(|_| votes > 0) {
        Some(stars) => t!(
            "stars-summary",
            average = average(stars, comma),
            count = votes
        ),
        None => t!("stars-no-votes"),
    }
}

/// The tallies once vote `old` is replaced by `new`, to show before the
/// server answers: the average (`None` without votes) and the count. The
/// average comes to two decimals, so the sum is rounded back to whole stars.
pub fn restarred(stars: Option<f64>, votes: u32, old: u8, new: u8) -> (Option<f64>, u32) {
    let sum = (stars.unwrap_or(0.0) * f64::from(votes)).round().max(0.0) as i64;
    let sum = sum - i64::from(old.min(MAX_STARS)) + i64::from(new.min(MAX_STARS));
    let votes = votes
        .saturating_sub(u32::from(old > 0))
        .saturating_add(u32::from(new > 0));
    let average =
        (votes > 0 && sum > 0).then(|| (sum as f64 / f64::from(votes) * 100.0).round() / 100.0);
    (average, votes)
}

/// The score and the like a post shows once its stars go from `old` to
/// `new`: 4 or 5 count as a like, 1 or 2 as a dislike, as the server counts.
pub fn liked(score: i32, old: u8, new: u8) -> (i32, i16) {
    let (was, now) = (like_of(old), like_of(new));
    (rescored(score, was, now), now)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_press_gives_that_many_stars_or_takes_the_same_back() {
        assert_eq!(next_stars(0, 4), 4);
        assert_eq!(next_stars(4, 2), 2);
        assert_eq!(next_stars(2, 5), 5);
        assert_eq!(next_stars(4, 4), 0);
        assert_eq!(next_stars(5, 5), 0);
        // Out of range, as the nearest star.
        assert_eq!(next_stars(0, 9), 5);
        assert_eq!(next_stars(0, 0), 1);
        assert_eq!(next_stars(1, 0), 0);
    }

    #[test]
    fn the_keys_walk_along_the_stars_and_stop_at_the_ends() {
        assert_eq!(tab_stop(0), 1);
        assert_eq!(tab_stop(4), 4);
        assert_eq!(moved("ArrowRight", 3), Some(4));
        assert_eq!(moved("ArrowUp", 3), Some(4));
        assert_eq!(moved("ArrowLeft", 3), Some(2));
        assert_eq!(moved("ArrowDown", 3), Some(2));
        assert_eq!(moved("ArrowRight", 5), Some(5));
        assert_eq!(moved("ArrowLeft", 1), Some(1));
        assert_eq!(moved("Home", 4), Some(1));
        assert_eq!(moved("End", 2), Some(5));
        assert_eq!(moved("Enter", 2), None);
        assert_eq!(moved("a", 2), None);
    }

    #[test]
    fn pointing_lights_the_stars_up_to_it() {
        let row = |mine, pointed| (1..=5).map(|s| lit(s, mine, pointed)).collect::<Vec<_>>();
        assert_eq!(row(3, None), [true, true, true, false, false]);
        assert_eq!(row(3, Some(1)), [true, false, false, false, false]);
        assert_eq!(row(0, Some(5)), [true; 5]);
        assert_eq!(row(0, None), [false; 5]);
    }

    #[test]
    fn the_average_reads_to_one_decimal() {
        assert_eq!(average(4.33, false), "4.3");
        assert_eq!(average(4.35, false), "4.3");
        assert_eq!(average(4.36, true), "4,4");
        assert_eq!(average(5.0, false), "5.0");
        assert_eq!(average(9.0, false), "5.0");
        assert_eq!(average(f64::NAN, false), "0.0");
    }

    #[test]
    fn the_stars_say_what_they_are_in_words() {
        assert_eq!(star_label(4), "4 of 5 stars");
        assert_eq!(star_label(1), "1 of 5 stars");
        assert_eq!(mine_label(0), "You haven't rated this yet");
        assert_eq!(mine_label(4), "Your rating: 4 of 5 stars");
        assert_eq!(
            summary(Some(4.33), 3, false),
            "Average 4.3 of 5, from 3 votes"
        );
        assert_eq!(summary(Some(5.0), 1, true), "Average 5,0 of 5, from 1 vote");
        assert_eq!(summary(None, 0, false), "No votes yet");
        assert_eq!(summary(Some(4.0), 0, false), "No votes yet");
    }

    #[test]
    fn the_tallies_follow_the_vote_until_the_server_answers() {
        // 13 stars in 3 votes.
        assert_eq!(restarred(Some(4.33), 3, 0, 5), (Some(4.5), 4));
        assert_eq!(restarred(Some(4.33), 3, 4, 1), (Some(3.33), 3));
        assert_eq!(restarred(Some(4.33), 3, 5, 0), (Some(4.0), 2));
        assert_eq!(restarred(None, 0, 0, 3), (Some(3.0), 1));
        assert_eq!(restarred(Some(3.0), 1, 3, 0), (None, 0));
        // Nothing to take back from a post the page thinks has no votes.
        assert_eq!(restarred(None, 0, 2, 0), (None, 0));
    }

    #[test]
    fn stars_count_as_likes_in_the_score() {
        assert_eq!(liked(2, 0, 5), (3, 1));
        assert_eq!(liked(2, 5, 4), (2, 1));
        assert_eq!(liked(2, 5, 3), (1, 0));
        assert_eq!(liked(2, 3, 1), (1, -1));
        assert_eq!(liked(0, 1, 0), (1, 0));
    }
}
