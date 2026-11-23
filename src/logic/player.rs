//! The video player's rules that need no browser: the clock it shows, where
//! a press on the seek bar lands, what each key does, when its controls may
//! rest, what a tap does, the badge a seek shows, and whether a video loops
//! unless told otherwise.

use crate::logic::viewer::format_duration;

/// How far a key seeks, in seconds.
pub const STEP_SECONDS: i32 = 5;
/// The speeds offered, slowest first.
pub const SPEEDS: [f64; 6] = [0.5, 0.75, 1.0, 1.25, 1.5, 2.0];
/// Videos up to this long, in seconds, loop until told otherwise: in a booru
/// a short clip is usually made to loop, and a long one has an ending.
pub const LOOP_UP_TO: f64 = 60.0;
/// The controls rest after this long without input while the video plays,
/// as long as the viewer's own floating controls take in fullscreen.
pub const REST_MS: u64 = 2800;
/// After the mouse leaves the player.
pub const LEAVE_MS: u64 = 700;
/// How long a seek's badge stays; presses within it add up.
pub const BADGE_MS: f64 = 900.0;
/// What the clock shows for a length not known yet.
pub const UNKNOWN: &str = "–:––";

/// `m:ss`, or `h:mm:ss` from an hour, of the whole seconds gone. `None` for
/// nonsense.
pub fn clock(secs: f64) -> Option<String> {
    if !secs.is_finite() || secs < 0.0 {
        return None;
    }
    // Saturates on absurd values instead of wrapping.
    let whole = secs.floor() as i64;
    format_duration(whole.saturating_mul(1000))
}

/// The length of a video `duration` seconds long, to the nearest second.
fn length(duration: f64) -> Option<String> {
    if !duration.is_finite() || duration <= 0.0 {
        return None;
    }
    format_duration((duration * 1000.0).round() as i64)
}

/// The time shown now and the length: `0:03` and `0:06`. The length is a
/// dash until it's known, and at the very end the two read the same.
pub fn times(now: f64, duration: f64) -> (String, String) {
    let total = length(duration);
    let at_end = total.is_some() && now >= duration - 0.25;
    let shown = if at_end { total.clone() } else { clock(now) };
    (
        shown.unwrap_or_else(|| "0:00".to_owned()),
        total.unwrap_or_else(|| UNKNOWN.to_owned()),
    )
}

/// Whether `duration` seconds is a length the player can seek in.
pub fn known(duration: f64) -> bool {
    duration.is_finite() && duration > 0.0
}

/// The share of a bar from `left`, `width` wide, under a pointer at `x`:
/// 0 at its start, 1 at its end, and no further either way.
pub fn share_at(x: f64, left: f64, width: f64) -> f64 {
    let share = (x - left) / width;
    if width > 0.0 && share.is_finite() {
        share.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// The moment `share` of the way through a video `duration` long. `None`
/// while the length isn't known, when there's nowhere to go.
pub fn time_at(share: f64, duration: f64) -> Option<f64> {
    (known(duration) && share.is_finite()).then(|| share.clamp(0.0, 1.0) * duration)
}

/// How far through the video `now` is, 0 to 1.
pub fn played(now: f64, duration: f64) -> f64 {
    if known(duration) && now.is_finite() {
        (now / duration).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// `now` moved by `by` seconds, within the video.
pub fn stepped(now: f64, by: f64, duration: f64) -> Option<f64> {
    let to = now + by;
    (known(duration) && to.is_finite()).then(|| to.clamp(0.0, duration))
}

/// The parts loaded, as `(start, width)` shares of the bar, from the
/// browser's `(start, end)` ranges in seconds. Nonsense and empty ranges
/// are left out.
pub fn loaded(ranges: &[(f64, f64)], duration: f64) -> Vec<(f64, f64)> {
    if !known(duration) {
        return Vec::new();
    }
    ranges
        .iter()
        .filter(|(start, end)| start.is_finite() && end.is_finite() && end > start)
        .map(|&(start, end)| {
            let from = (start / duration).clamp(0.0, 1.0);
            let to = (end / duration).clamp(0.0, 1.0);
            (from, to - from)
        })
        .filter(|(_, width)| *width > 0.0)
        .collect()
}

/// Whether a video `duration` seconds long loops unless told otherwise.
pub fn loops(duration: f64) -> bool {
    known(duration) && duration <= LOOP_UP_TO
}

/// A speed as the menu writes it, without the zeros a person wouldn't:
/// `0.5`, `1`, `1.25`, with `mark` before the fraction.
pub fn speed_text(rate: f64, mark: char) -> String {
    let rate = if rate.is_finite() && rate > 0.0 {
        rate
    } else {
        1.0
    };
    let text = format!("{rate:.2}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    text.replace('.', mark.encode_utf8(&mut [0; 4]))
}

/// The volume to unmute to: a slider left at nothing would stay silent.
pub fn unmuted_volume(volume: f64) -> f64 {
    if volume.is_finite() && volume >= 0.05 {
        volume.min(1.0)
    } else {
        0.5
    }
}

/// The volume as a whole percentage.
pub fn percent(volume: f64) -> u8 {
    if volume.is_finite() {
        (volume.clamp(0.0, 1.0) * 100.0).round() as u8
    } else {
        0
    }
}

// ---------- keys ----------

/// What a key does to the video.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Act {
    TogglePlay,
    /// Seconds back or forward.
    Seek(i32),
    /// To a share of the way through, 0 to 1.
    Jump(f64),
    ToggleMute,
    Fullscreen,
}

/// A key pressed while the player takes keys, and where the focus was.
#[derive(Clone, Copy, Debug, Default)]
pub struct Press<'a> {
    /// `KeyboardEvent.key`.
    pub key: &'a str,
    pub shift: bool,
    /// The focus is on the seek bar.
    pub on_seek: bool,
    /// The focus is on a button or a link, which Space presses.
    pub on_button: bool,
}

/// What `p` does. The arrows step between posts in the viewer, so they seek
/// only with Shift or on the seek bar; J and L always do.
pub fn act(p: Press<'_>) -> Option<Act> {
    let step = STEP_SECONDS;
    Some(match p.key {
        " " if !p.on_button => Act::TogglePlay,
        "k" | "K" => Act::TogglePlay,
        "j" | "J" => Act::Seek(-step),
        "l" | "L" => Act::Seek(step),
        "ArrowLeft" if p.shift || p.on_seek => Act::Seek(-step),
        "ArrowRight" if p.shift || p.on_seek => Act::Seek(step),
        "ArrowDown" if p.on_seek => Act::Seek(-step),
        "ArrowUp" if p.on_seek => Act::Seek(step),
        "Home" if p.on_seek => Act::Jump(0.0),
        "End" if p.on_seek => Act::Jump(1.0),
        "m" | "M" => Act::ToggleMute,
        "f" | "F" => Act::Fullscreen,
        digit => {
            let mut chars = digit.chars();
            let tenth = chars.next().and_then(|c| c.to_digit(10))?;
            if chars.next().is_some() {
                return None;
            }
            Act::Jump(f64::from(tenth) / 10.0)
        }
    })
}

// ---------- the controls ----------

/// What keeps the controls up while the video plays.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Held {
    pub paused: bool,
    /// A finger or the mouse is on the seek bar.
    pub scrubbing: bool,
    /// A key has put the focus on them.
    pub focused: bool,
    /// The speed menu is open.
    pub menu: bool,
    /// The mouse rests on them.
    pub hovered: bool,
    pub failed: bool,
}

/// Whether the controls may rest now: only while the video plays, and
/// nothing holds them up.
pub fn may_rest(h: Held) -> bool {
    !(h.paused || h.scrubbing || h.focused || h.menu || h.hovered || h.failed)
}

/// What a finger's tap on the picture does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tap {
    Play,
    ShowControls,
    HideControls,
}

/// A tap plays a paused video, and otherwise brings the controls up or puts
/// them away: they're the way to pause.
pub fn tap(paused: bool, controls_shown: bool) -> Tap {
    match (paused, controls_shown) {
        (true, _) => Tap::Play,
        (false, false) => Tap::ShowControls,
        (false, true) => Tap::HideControls,
    }
}

/// The badge a seek shows: how far, and when it last changed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Badge {
    pub seconds: i32,
    pub at: f64,
}

/// The badge after seeking `by` seconds at `now` (ms): presses the same way
/// while the badge still shows add up, and the other way starts again.
pub fn badge_after(before: Option<Badge>, by: i32, now: f64) -> Badge {
    let adds = before.filter(|b| {
        let gap = now - b.at;
        (0.0..BADGE_MS).contains(&gap) && b.seconds.signum() == by.signum()
    });
    Badge {
        seconds: adds.map_or(by, |b| b.seconds.saturating_add(by)),
        at: now,
    }
}

/// Why a video didn't play.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    /// The connection dropped on the way.
    Network,
    /// The browser can't decode it.
    Format,
    /// Never came: an expired link, a file gone, or a type the browser
    /// doesn't take, which it can't tell apart.
    Missing,
}

/// The failure for `MediaError.code`.
pub fn failure(code: u16) -> Failure {
    match code {
        2 => Failure::Network,
        3 => Failure::Format,
        _ => Failure::Missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_counts_whole_seconds() {
        assert_eq!(clock(0.0).as_deref(), Some("0:00"));
        assert_eq!(clock(5.99).as_deref(), Some("0:05"));
        assert_eq!(clock(65.2).as_deref(), Some("1:05"));
        assert_eq!(clock(3723.0).as_deref(), Some("1:02:03"));
        assert_eq!(clock(-1.0), None);
        assert_eq!(clock(f64::NAN), None);
        assert_eq!(clock(f64::INFINITY), None);
        assert!(clock(1e300).is_some(), "absurd but not a crash");
    }

    #[test]
    fn the_time_and_the_length_agree_at_the_end() {
        assert_eq!(times(3.4, 6.0), ("0:03".into(), "0:06".into()));
        assert_eq!(times(65.0, 200.0), ("1:05".into(), "3:20".into()));
        // Just short of a length that rounds up: both say the length.
        assert_eq!(times(5.9, 5.98), ("0:06".into(), "0:06".into()));
        assert_eq!(times(6.0, 6.0), ("0:06".into(), "0:06".into()));
        // Not known yet.
        assert_eq!(times(0.0, f64::NAN), ("0:00".into(), UNKNOWN.into()));
        assert_eq!(
            times(f64::NAN, f64::INFINITY),
            ("0:00".into(), UNKNOWN.into())
        );
        assert_eq!(times(2.0, 0.0), ("0:02".into(), UNKNOWN.into()));
    }

    #[test]
    fn a_press_on_the_bar_lands_within_it() {
        assert_eq!(share_at(150.0, 100.0, 200.0), 0.25);
        assert_eq!(share_at(50.0, 100.0, 200.0), 0.0);
        assert_eq!(share_at(900.0, 100.0, 200.0), 1.0);
        assert_eq!(share_at(150.0, 100.0, 0.0), 0.0);
        assert_eq!(share_at(f64::NAN, 100.0, 200.0), 0.0);
        assert_eq!(share_at(150.0, 100.0, -5.0), 0.0);
        assert_eq!(time_at(0.25, 200.0), Some(50.0));
        assert_eq!(time_at(1.5, 200.0), Some(200.0));
        assert_eq!(time_at(-1.0, 200.0), Some(0.0));
        assert_eq!(time_at(0.5, f64::NAN), None, "nowhere to go yet");
        assert_eq!(time_at(f64::NAN, 200.0), None);
        assert_eq!(played(50.0, 200.0), 0.25);
        assert_eq!(played(500.0, 200.0), 1.0);
        assert_eq!(played(5.0, f64::NAN), 0.0);
    }

    #[test]
    fn seeking_stays_within_the_video() {
        assert_eq!(stepped(10.0, 5.0, 60.0), Some(15.0));
        assert_eq!(stepped(3.0, -5.0, 60.0), Some(0.0));
        assert_eq!(stepped(58.0, 5.0, 60.0), Some(60.0));
        assert_eq!(stepped(3.0, 5.0, f64::NAN), None);
        assert_eq!(stepped(f64::NAN, 5.0, 60.0), None);
    }

    #[test]
    fn the_loaded_parts_are_shares_of_the_bar() {
        assert_eq!(
            loaded(&[(0.0, 2.0), (4.0, 8.0)], 8.0),
            [(0.0, 0.25), (0.5, 0.5)]
        );
        // Past the end, backwards, empty or nonsense.
        assert_eq!(loaded(&[(6.0, 12.0)], 8.0), [(0.75, 0.25)]);
        assert!(loaded(&[(3.0, 2.0), (2.0, 2.0), (f64::NAN, 4.0)], 8.0).is_empty());
        assert!(loaded(&[(9.0, 12.0)], 8.0).is_empty());
        assert!(loaded(&[(0.0, 2.0)], f64::NAN).is_empty());
    }

    #[test]
    fn short_videos_loop() {
        assert!(loops(6.0));
        assert!(loops(LOOP_UP_TO));
        assert!(!loops(LOOP_UP_TO + 0.5));
        assert!(!loops(f64::NAN), "unknown");
        assert!(!loops(f64::INFINITY), "a stream");
        assert!(!loops(0.0));
    }

    #[test]
    fn speeds_read_without_extra_zeros() {
        assert_eq!(speed_text(1.0, '.'), "1");
        assert_eq!(speed_text(0.5, '.'), "0.5");
        assert_eq!(speed_text(1.25, ','), "1,25");
        assert_eq!(speed_text(2.0, ','), "2");
        assert_eq!(speed_text(f64::NAN, '.'), "1");
        assert_eq!(speed_text(-2.0, '.'), "1");
        let shown: Vec<String> = SPEEDS.iter().map(|s| speed_text(*s, '.')).collect();
        assert_eq!(shown, ["0.5", "0.75", "1", "1.25", "1.5", "2"]);
    }

    #[test]
    fn unmuting_is_never_silent() {
        assert_eq!(unmuted_volume(0.7), 0.7);
        assert_eq!(unmuted_volume(0.0), 0.5);
        assert_eq!(unmuted_volume(0.01), 0.5);
        assert_eq!(unmuted_volume(f64::NAN), 0.5);
        assert_eq!(unmuted_volume(3.0), 1.0);
        assert_eq!(percent(0.704), 70);
        assert_eq!(percent(2.0), 100);
        assert_eq!(percent(f64::NAN), 0);
    }

    fn press(key: &str) -> Press<'_> {
        Press {
            key,
            ..Press::default()
        }
    }

    #[test]
    fn the_arrows_stay_the_viewers_unless_shifted_or_on_the_bar() {
        assert_eq!(act(press("ArrowLeft")), None);
        assert_eq!(act(press("ArrowRight")), None);
        let shifted = |key| Press {
            shift: true,
            ..press(key)
        };
        assert_eq!(act(shifted("ArrowLeft")), Some(Act::Seek(-5)));
        assert_eq!(act(shifted("ArrowRight")), Some(Act::Seek(5)));
        let on_seek = |key| Press {
            on_seek: true,
            ..press(key)
        };
        assert_eq!(act(on_seek("ArrowLeft")), Some(Act::Seek(-5)));
        assert_eq!(act(on_seek("ArrowUp")), Some(Act::Seek(5)));
        assert_eq!(act(on_seek("Home")), Some(Act::Jump(0.0)));
        assert_eq!(act(on_seek("End")), Some(Act::Jump(1.0)));
        assert_eq!(act(press("Home")), None);
        assert_eq!(act(press("ArrowUp")), None, "the page scrolls");
        assert_eq!(act(press("j")), Some(Act::Seek(-5)));
        assert_eq!(act(press("L")), Some(Act::Seek(5)));
    }

    #[test]
    fn space_presses_a_focused_button_instead() {
        assert_eq!(act(press(" ")), Some(Act::TogglePlay));
        let on_button = Press {
            on_button: true,
            ..press(" ")
        };
        assert_eq!(act(on_button), None);
        assert_eq!(
            act(Press {
                on_button: true,
                ..press("k")
            }),
            Some(Act::TogglePlay)
        );
        assert_eq!(act(press("m")), Some(Act::ToggleMute));
        assert_eq!(act(press("F")), Some(Act::Fullscreen));
        assert_eq!(act(press("e")), None, "the tag editor's");
        assert_eq!(act(press("Escape")), None);
    }

    #[test]
    fn digits_jump_by_tenths() {
        assert_eq!(act(press("0")), Some(Act::Jump(0.0)));
        assert_eq!(act(press("4")), Some(Act::Jump(0.4)));
        assert_eq!(act(press("9")), Some(Act::Jump(0.9)));
        // On a keyboard that needs Shift for them.
        assert_eq!(
            act(Press {
                shift: true,
                ..press("7")
            }),
            Some(Act::Jump(0.7))
        );
        assert_eq!(act(press("10")), None);
        assert_eq!(act(press("F1")), None);
        assert_eq!(act(press("")), None);
        assert_eq!(act(press("٣")), None, "only plain digits");
    }

    #[test]
    fn the_controls_rest_only_while_playing_and_free() {
        assert!(may_rest(Held::default()));
        for held in [
            Held {
                paused: true,
                ..Held::default()
            },
            Held {
                scrubbing: true,
                ..Held::default()
            },
            Held {
                focused: true,
                ..Held::default()
            },
            Held {
                menu: true,
                ..Held::default()
            },
            Held {
                hovered: true,
                ..Held::default()
            },
            Held {
                failed: true,
                ..Held::default()
            },
        ] {
            assert!(!may_rest(held), "{held:?}");
        }
    }

    #[test]
    fn a_tap_plays_or_shows_or_hides() {
        assert_eq!(tap(true, true), Tap::Play);
        assert_eq!(tap(true, false), Tap::Play);
        assert_eq!(tap(false, false), Tap::ShowControls);
        assert_eq!(tap(false, true), Tap::HideControls);
    }

    #[test]
    fn seeks_the_same_way_add_up_on_the_badge() {
        let first = badge_after(None, -5, 1000.0);
        assert_eq!(first.seconds, -5);
        let second = badge_after(Some(first), -5, 1400.0);
        assert_eq!(second.seconds, -10);
        // The other way starts again.
        assert_eq!(badge_after(Some(second), 5, 1500.0).seconds, 5);
        // Gone by then.
        assert_eq!(badge_after(Some(second), -5, 1400.0 + BADGE_MS).seconds, -5);
        // A clock set back.
        assert_eq!(badge_after(Some(second), -5, 900.0).seconds, -5);
        assert_eq!(badge_after(Some(second), -5, 1450.0).at, 1450.0);
    }

    #[test]
    fn a_failure_says_why() {
        assert_eq!(failure(2), Failure::Network);
        assert_eq!(failure(3), Failure::Format);
        assert_eq!(failure(4), Failure::Missing);
        assert_eq!(failure(1), Failure::Missing);
        assert_eq!(failure(0), Failure::Missing);
    }
}
