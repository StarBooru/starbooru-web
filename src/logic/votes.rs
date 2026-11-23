//! Votes: up, down or none, and the score that goes with them, so the viewer
//! can show a vote before the server has it and take it back if it fails.

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

#[cfg(test)]
mod tests {
    use super::*;

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
