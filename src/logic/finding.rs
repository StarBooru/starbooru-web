//! Finding the people in a picture with the server's People tool, without a
//! browser: which of the people found are new to the editor, what an answer
//! comes to, a failure in words, and when the viewer waits for the people
//! the server looks for by itself after an upload.

use chrono::{DateTime, Utc};
use starbooru_core::{FaceBox, Figure, MediaKind, Post};

use super::faces::same_face;
use super::people::people_known;

/// The people found that the editor doesn't have yet, left to right as
/// they came: those whose face is over the face of someone in `have` are
/// left out, and at most `room` come. Someone found without a face can't be
/// told apart, and comes.
pub fn new_people(have: &[Option<FaceBox>], found: Vec<Figure>, room: usize) -> Vec<Figure> {
    let known: Vec<FaceBox> = have.iter().flatten().copied().collect();
    found
        .into_iter()
        .filter(|f| {
            f.face
                .is_none_or(|face| !known.iter().any(|k| same_face(*k, face)))
        })
        .take(room)
        .collect()
}

/// What an answer comes to, for the line under the button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// This many people joined the editor.
    Added(usize),
    /// Everyone found was there already.
    NoneNew,
    /// No one in the picture.
    Nobody,
}

/// `found` people in the answer, of whom `added` joined the editor.
pub fn outcome(found: usize, added: usize) -> Outcome {
    match (found, added) {
        (0, _) => Outcome::Nobody,
        (_, 0) => Outcome::NoneNew,
        (_, added) => Outcome::Added(added),
    }
}

/// Why finding people failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Trouble {
    /// The models are starting or downloading: soon.
    Starting,
    /// The machine can't run them.
    NoMemory,
    /// An administrator turned the tool off.
    Off,
    /// The post has no picture the server can read.
    NoPicture,
    /// The AI provider failed, in the server's words.
    Provider(String),
    /// Anything else: the words the app has for any failed request.
    Other,
}

impl Trouble {
    /// Whether trying again may work.
    pub fn retries(&self) -> bool {
        matches!(
            self,
            Trouble::Starting | Trouble::Provider(_) | Trouble::Other
        )
    }
}

/// A failed `POST /posts/{id}/find-people`, from its status, error code and
/// the server's message. The 400s share a code, so their words tell them
/// apart; a message the app doesn't know is a picture it can't read.
pub fn trouble(status: Option<u16>, code: Option<&str>, message: &str) -> Trouble {
    let said = message.to_ascii_lowercase();
    match (status, code) {
        (Some(409), Some("ai_unavailable")) if said.contains("memory") => Trouble::NoMemory,
        (Some(409), Some("ai_unavailable")) => Trouble::Starting,
        (Some(400), _) if said.contains("turned off") => Trouble::Off,
        (Some(400), _) => Trouble::NoPicture,
        (Some(424), _) => Trouble::Provider(message.trim().trim_end_matches('.').to_owned()),
        _ => Trouble::Other,
    }
}

/// A post younger than this, without people, may still get them from the
/// server, which looks for them a few seconds after an upload.
pub const FRESH_SECONDS: i64 = 120;

/// How long the viewer waits before each look at a new post, in ms: at 3, 8
/// and 20 seconds after it opens.
pub const CHECKS_MS: [i32; 3] = [3_000, 5_000, 12_000];

/// Whether the viewer should wait for the people of `post`: the People tool
/// is on (`tool`), and it's a whole still picture, live, without people,
/// uploaded less than two minutes ago. A server clock a little ahead of the
/// browser's counts as new too.
pub fn waits_for_people(post: &Post, now: DateTime<Utc>, tool: bool) -> bool {
    let age = (now - post.created_at).num_seconds();
    tool && post.kind == MediaKind::Image
        && !post.deleted
        && post.figures.is_empty()
        && people_known(post)
        && (-FRESH_SECONDS..FRESH_SECONDS).contains(&age)
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;
    use starbooru_core::FigureKind;

    use super::*;

    fn at(x: u16, y: u16) -> FaceBox {
        FaceBox {
            x,
            y,
            w: 100,
            h: 100,
        }
    }

    fn found(face: Option<FaceBox>) -> Figure {
        Figure {
            kind: FigureKind::Other,
            face,
            face_url: Some("data:image/jpeg;base64,/9j/".into()),
            ..Figure::default()
        }
    }

    fn post(age_seconds: i64, now: DateTime<Utc>) -> Post {
        let created = now - TimeDelta::seconds(age_seconds);
        let mut post: Post = serde_json::from_value(serde_json::json!({
            "id": 70, "kind": "image", "mime": "image/jpeg", "file_size": 1,
            "width": 1000, "height": 800, "sha256": "x", "rating": "safe", "score": 0,
            "my_vote": 0, "source": null, "tags": [], "created_at": created,
            "deleted": false, "file_url": "/media/x", "thumb_url": null,
        }))
        .unwrap();
        post.created_at = created;
        post
    }

    #[test]
    fn faces_already_there_are_left_out() {
        let have = [Some(at(100, 100)), None, Some(at(600, 100))];
        let answer = vec![
            found(Some(at(110, 120))),
            found(Some(at(350, 100))),
            found(None),
            found(Some(at(620, 90))),
            found(Some(at(800, 500))),
        ];
        let added = new_people(&have, answer, 20);
        let faces: Vec<_> = added.iter().map(|f| f.face).collect();
        assert_eq!(faces, [Some(at(350, 100)), None, Some(at(800, 500))]);
        // The crop's preview comes along for the editor's disc.
        assert!(added.iter().all(|f| f.face_url.is_some()));
        // No room past the post's limit; the first ones come.
        let few = new_people(&[], vec![found(Some(at(0, 0))), found(Some(at(500, 0)))], 1);
        assert_eq!(few.len(), 1);
        assert_eq!(few.first().and_then(|f| f.face), Some(at(0, 0)));
        assert!(new_people(&have, Vec::new(), 20).is_empty());
    }

    #[test]
    fn an_answer_says_what_changed() {
        assert_eq!(outcome(0, 0), Outcome::Nobody);
        assert_eq!(outcome(4, 0), Outcome::NoneNew);
        assert_eq!(outcome(4, 2), Outcome::Added(2));
    }

    #[test]
    fn each_failure_has_its_reason() {
        let t = |status, code, message| trouble(Some(status), Some(code), message);
        assert_eq!(
            t(409, "ai_unavailable", "the AI models are still starting"),
            Trouble::Starting
        );
        assert_eq!(
            t(
                409,
                "ai_unavailable",
                "the AI model is still downloading; try again in a moment"
            ),
            Trouble::Starting
        );
        assert_eq!(
            t(
                409,
                "ai_unavailable",
                "this machine doesn't have enough memory for this AI feature"
            ),
            Trouble::NoMemory
        );
        assert_eq!(
            t(
                400,
                "bad_request",
                "this AI feature is turned off on this server"
            ),
            Trouble::Off
        );
        assert_eq!(
            t(
                400,
                "bad_request",
                "this post has no picture to find people in"
            ),
            Trouble::NoPicture
        );
        assert_eq!(
            t(424, "fetch_failed", "can't reach the AI provider."),
            Trouble::Provider("can't reach the AI provider".into())
        );
        assert_eq!(t(404, "not_found", "not found"), Trouble::Other);
        assert_eq!(t(409, "duplicate", "x"), Trouble::Other);
        assert_eq!(trouble(None, None, "offline"), Trouble::Other);
        assert!(Trouble::Starting.retries());
        assert!(Trouble::Provider(String::new()).retries());
        assert!(Trouble::Other.retries());
        assert!(!Trouble::Off.retries());
        assert!(!Trouble::NoPicture.retries());
        assert!(!Trouble::NoMemory.retries());
    }

    #[test]
    fn a_new_picture_without_people_is_waited_for() {
        let now = Utc::now();
        assert!(waits_for_people(&post(5, now), now, true));
        assert!(waits_for_people(&post(119, now), now, true));
        // The server's clock a little ahead.
        assert!(waits_for_people(&post(-30, now), now, true));
        // Too old, or the tool is off.
        assert!(!waits_for_people(&post(120, now), now, true));
        assert!(!waits_for_people(&post(3600, now), now, true));
        assert!(!waits_for_people(&post(5, now), now, false));
        // People already, a video or a GIF, the recycle bin.
        let mut marked = post(5, now);
        marked.figures = vec![found(Some(at(0, 0)))];
        assert!(!waits_for_people(&marked, now, true));
        for kind in [MediaKind::Video, MediaKind::Animation] {
            let mut moving = post(5, now);
            moving.kind = kind;
            assert!(!waits_for_people(&moving, now, true), "{kind:?}");
        }
        let mut binned = post(5, now);
        binned.deleted = true;
        assert!(!waits_for_people(&binned, now, true));
        // A copy from a list, which leaves the people out.
        let mut listed = post(5, now);
        listed.tags = vec!["photo".into()];
        assert!(!waits_for_people(&listed, now, true));
    }

    #[test]
    fn the_looks_come_at_three_eight_and_twenty_seconds() {
        let mut at = 0;
        let times: Vec<i32> = CHECKS_MS
            .iter()
            .map(|wait| {
                at += wait;
                at
            })
            .collect();
        assert_eq!(times, [3_000, 8_000, 20_000]);
    }
}
