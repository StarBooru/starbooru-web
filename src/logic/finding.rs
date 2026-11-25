//! Finding the people in a picture with the server's People tool, without a
//! browser: how the people found join those in the editor, what an answer
//! comes to, a failure in words, and when the viewer waits for the people
//! the server looks for by itself after an upload.

use std::collections::BTreeSet;

use chrono::{DateTime, TimeDelta, Utc};
use starbooru_core::{
    FaceBox, Figure, ImportJob, ImportRequest, JobState, MediaKind, Post, cosplay_name,
    normalize_tag,
};

use super::faces::same_face;
use super::people::{Draft, people_known};

/// Someone in the editor who was found again: person `person` (their place
/// among the editor's people) as they become, with the box found for them
/// and its preview.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paired {
    pub person: usize,
    pub draft: Draft,
    pub preview: Option<(FaceBox, String)>,
}

/// What the people found do to the editor's.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Joined {
    /// People already there who were found again, with what was found of
    /// them in their empty fields.
    pub paired: Vec<Paired>,
    /// Everyone else found, to come after the people there.
    pub added: Vec<Figure>,
}

/// Who someone is by name, as tags: their character and whom they dress
/// as. `None` for someone with neither.
fn named(
    character: Option<&str>,
    cosplay: Option<&str>,
) -> Option<(Option<String>, Option<String>)> {
    let tag = |text: Option<&str>| text.and_then(|t| normalize_tag(t).ok());
    let character = tag(character);
    let cosplay = tag(cosplay).map(|c| cosplay_name(&c).to_owned());
    (character.is_some() || cosplay.is_some()).then_some((character, cosplay))
}

fn draft_named(d: &Draft) -> Option<(Option<String>, Option<String>)> {
    named(Some(&d.character), Some(&d.cosplay))
}

/// `draft` with what was found of them: the box when they have none, and
/// each empty field filled. What was typed stays.
fn filled_from(draft: &Draft, found: &Figure) -> Draft {
    let mut out = draft.clone();
    if out.face.is_none() {
        out.face = found.face;
    }
    let fill = |mine: &mut String, theirs: Option<&String>| {
        if let Some(theirs) = theirs.filter(|_| mine.trim().is_empty()) {
            mine.clone_from(theirs);
        }
    };
    fill(&mut out.character, found.character.as_ref());
    fill(&mut out.cosplay, found.cosplay.as_ref());
    fill(&mut out.franchise, found.franchise.as_ref());
    for (slot, tags) in &found.tags {
        let text = out.slots.entry(*slot).or_default();
        if text.trim().is_empty() && !tags.is_empty() {
            *text = tags.join(" ");
        }
    }
    out
}

/// How the people found, left to right, join the editor's `have`. Someone
/// whose face is over a face already marked is left out. Each of the rest
/// is first matched with someone there without a box: the same character
/// and cosplay, or, when neither is named, the first of the same kind not
/// matched yet. They get the box found for them and their empty fields
/// filled. Only those left over are added, at most `room` of them. The
/// empty person the editor starts with matches no one.
pub fn join_found(have: &[Draft], found: Vec<Figure>, room: usize) -> Joined {
    let known: Vec<FaceBox> = have.iter().filter_map(|d| d.face).collect();
    let mut taken = BTreeSet::new();
    let mut joined = Joined::default();
    let mut left = Vec::new();
    for figure in found {
        if figure
            .face
            .is_some_and(|face| known.iter().any(|k| same_face(*k, face)))
        {
            continue;
        }
        let who = named(figure.character.as_deref(), figure.cosplay.as_deref());
        let matched = have.iter().enumerate().find(|(i, d)| {
            let free = d.face.is_none() && !d.is_blank() && !taken.contains(i);
            free && match (&who, draft_named(d)) {
                (Some(found), Some(theirs)) => *found == theirs,
                (None, None) => d.kind == figure.kind,
                _ => false,
            }
        });
        match matched {
            Some((person, draft)) => {
                taken.insert(person);
                let after = filled_from(draft, &figure);
                // Found again with nothing new: they're there already.
                if after != *draft {
                    joined.paired.push(Paired {
                        person,
                        draft: after,
                        preview: figure.face.zip(figure.face_url),
                    });
                }
            }
            None => left.push(figure),
        }
    }
    joined.added = left.into_iter().take(room).collect();
    joined
}

/// What an answer comes to, for the line under the button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// This many people joined the editor or were found again.
    Added(usize),
    /// Everyone found was there already.
    NoneNew,
    /// No one in the picture.
    Nobody,
}

/// `found` people in the answer, of whom `added` joined the editor or were
/// found again there.
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
    /// The tool is busy with other pictures, uploads included: soon.
    Busy,
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
            Trouble::Starting | Trouble::Busy | Trouble::Provider(_) | Trouble::Other
        )
    }
}

/// A failed `POST /posts/{id}/find-people`, from its status, error code and
/// the server's message. The 400s share a code, so their words tell them
/// apart; a message the app doesn't know is a picture it can't read.
pub fn trouble(status: Option<u16>, code: Option<&str>, message: &str) -> Trouble {
    let said = message.to_ascii_lowercase();
    match (status, code) {
        (Some(409), Some("ai_busy")) => Trouble::Busy,
        (Some(409), Some("ai_unavailable")) if said.contains("memory") => Trouble::NoMemory,
        (Some(409), Some("ai_unavailable")) => Trouble::Starting,
        (Some(400), _) if said.contains("turned off") => Trouble::Off,
        (Some(400), _) => Trouble::NoPicture,
        (Some(424), _) => Trouble::Provider(message.trim().trim_end_matches('.').to_owned()),
        _ => Trouble::Other,
    }
}

/// A post younger than this, without people, may still get them from the
/// server, which tags each upload first and then finds people one picture
/// at a time.
pub const FRESH_SECONDS: i64 = 180;

/// The first looks at a new post, each in ms after the one before: at 3, 8
/// and 20 seconds after it opens.
const FIRST_LOOKS_MS: [i32; 3] = [3_000, 5_000, 12_000];
/// Then one every this many ms, while the post is fresh.
const LATER_LOOK_MS: i32 = 10_000;

/// How long the viewer waits before its next look at a new post after
/// `looked` looks, or `None` to stop: the first three whatever the post's
/// `age` now, then one every ten seconds that comes before it's
/// [`FRESH_SECONDS`] old. A batch of uploads, or a model still starting,
/// takes a while.
pub fn next_look(looked: usize, age: TimeDelta) -> Option<i32> {
    if let Some(wait) = FIRST_LOOKS_MS.get(looked) {
        return Some(*wait);
    }
    let then = age + TimeDelta::milliseconds(i64::from(LATER_LOOK_MS));
    (then < TimeDelta::seconds(FRESH_SECONDS)).then_some(LATER_LOOK_MS)
}

/// Whether the viewer should wait for the people of `post`: the People tool
/// is on (`tool`), the server looks for them in this upload (`looked_for`:
/// not when its uploader turned the AI off), and it's a whole still
/// picture, live, without people, uploaded less than three minutes ago. A
/// server clock a little ahead of the browser's counts as new too.
pub fn waits_for_people(post: &Post, now: DateTime<Utc>, tool: bool, looked_for: bool) -> bool {
    let age = (now - post.created_at).num_seconds();
    tool && looked_for
        && post.kind == MediaKind::Image
        && !post.deleted
        && post.figures.is_empty()
        && people_known(post)
        && (-FRESH_SECONDS..FRESH_SECONDS).contains(&age)
}

/// The uploads of this visit that the server won't look for people in, as
/// their uploader turned the AI off for them (`autotag: false` skips both):
/// their posts, and the imports that have yet to make one.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Unlooked {
    posts: BTreeSet<i64>,
    jobs: BTreeSet<i64>,
}

impl Unlooked {
    /// A file sent with `autotag` became post `id`.
    pub fn uploaded(&mut self, id: i64, autotag: Option<bool>) {
        if autotag == Some(false) {
            self.posts.insert(id);
        }
    }

    /// The server took `requests` as `jobs`: an import is its address's.
    pub fn queued(&mut self, requests: &[ImportRequest], jobs: &[ImportJob]) {
        for job in jobs {
            let off = requests
                .iter()
                .any(|r| r.url == job.url && r.autotag == Some(false));
            if off {
                self.jobs.insert(job.id);
            }
        }
    }

    /// The imports as the server has them now: one done gives its post, and
    /// one that ended otherwise is forgotten.
    pub fn imports(&mut self, now: &[ImportJob]) {
        for job in now.iter().filter(|j| !j.state.is_active()) {
            if !self.jobs.remove(&job.id) {
                continue;
            }
            if let (JobState::Done, Some(post)) = (job.state, job.post_id) {
                self.posts.insert(post);
            }
        }
    }

    /// Whether the server looks for people in post `id`, as far as this
    /// visit knows.
    pub fn looked_for(&self, id: i64) -> bool {
        !self.posts.contains(&id)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use starbooru_core::{FigureKind, Rating, Slot};

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

    fn found_as(kind: FigureKind, character: Option<&str>, x: u16) -> Figure {
        Figure {
            kind,
            character: character.map(str::to_owned),
            face: Some(at(x, 100)),
            face_url: Some(format!("data:image/jpeg;base64,{x}")),
            ..Figure::default()
        }
    }

    /// Someone typed in the editor, without a box.
    fn typed(kind: FigureKind, character: &str) -> Draft {
        Draft {
            kind,
            character: character.into(),
            ..Draft::default()
        }
    }

    fn boxed(face: FaceBox) -> Draft {
        Draft {
            face: Some(face),
            ..Draft::default()
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
        let have = [boxed(at(100, 100)), Draft::default(), boxed(at(600, 100))];
        let answer = vec![
            found(Some(at(110, 120))),
            found(Some(at(350, 100))),
            found(None),
            found(Some(at(620, 90))),
            found(Some(at(800, 500))),
        ];
        let joined = join_found(&have, answer, 20);
        assert!(joined.paired.is_empty());
        let faces: Vec<_> = joined.added.iter().map(|f| f.face).collect();
        assert_eq!(faces, [Some(at(350, 100)), None, Some(at(800, 500))]);
        // The crop's preview comes along for the editor's disc.
        assert!(joined.added.iter().all(|f| f.face_url.is_some()));
        // No room past the post's limit; the first ones come.
        let few = join_found(&[], vec![found(Some(at(0, 0))), found(Some(at(500, 0)))], 1);
        assert_eq!(few.added.len(), 1);
        assert_eq!(few.added.first().and_then(|f| f.face), Some(at(0, 0)));
        assert_eq!(join_found(&have, Vec::new(), 20), Joined::default());
    }

    #[test]
    fn someone_typed_without_a_face_gets_the_one_found_for_them() {
        use FigureKind::{Boy, Girl};
        // Saber typed by hand, no box: found again, she isn't added twice.
        let have = [typed(Girl, "saber")];
        let joined = join_found(&have, vec![found_as(Girl, Some("saber"), 300)], 20);
        assert!(joined.added.is_empty(), "{joined:?}");
        let [paired] = joined.paired.as_slice() else {
            panic!("{joined:?}")
        };
        assert_eq!(paired.person, 0);
        assert_eq!(paired.draft.face, Some(at(300, 100)));
        assert_eq!(paired.draft.character, "saber");
        assert_eq!(
            paired.preview,
            Some((at(300, 100), "data:image/jpeg;base64,300".into()))
        );
        // Typed as words, or as another kind: the name is who they are.
        let have = [typed(Boy, " Saber ")];
        let joined = join_found(&have, vec![found_as(Girl, Some("saber"), 300)], 20);
        assert_eq!(joined.paired.len(), 1);
        assert_eq!(
            joined.paired.first().map(|p| p.draft.kind),
            Some(Boy),
            "the kind typed stays"
        );
        // Someone else named: a new person.
        let joined = join_found(
            &[typed(Girl, "rin")],
            vec![found_as(Girl, Some("saber"), 300)],
            20,
        );
        assert!(joined.paired.is_empty());
        assert_eq!(joined.added.len(), 1);
    }

    #[test]
    fn a_cosplay_must_match_too() {
        let mut rin_as_saber = typed(FigureKind::Girl, "rin");
        rin_as_saber.cosplay = "Saber_(cosplay)".into();
        let mut found_rin = found_as(FigureKind::Girl, Some("rin"), 200);
        found_rin.cosplay = Some("saber".into());
        let plain_rin = found_as(FigureKind::Girl, Some("rin"), 600);
        let joined = join_found(&[rin_as_saber], vec![plain_rin, found_rin], 20);
        // The plain Rin isn't her; the one dressed as Saber is.
        assert_eq!(joined.paired.len(), 1);
        assert_eq!(
            joined.paired.first().and_then(|p| p.draft.face),
            Some(at(200, 100))
        );
        assert_eq!(
            joined.added.iter().map(|f| f.face).collect::<Vec<_>>(),
            [Some(at(600, 100))]
        );
    }

    #[test]
    fn people_without_names_pair_by_kind_left_to_right() {
        use FigureKind::{Boy, Girl, Other};
        let mut tagged_girl = typed(Girl, "");
        tagged_girl.slots.insert(Slot::Hair, "red_hair".into());
        let mut second_girl = typed(Girl, "");
        second_girl.slots.insert(Slot::Eyes, "blue_eyes".into());
        let mut boy = typed(Boy, "");
        boy.slots.insert(Slot::Head, "hat".into());
        let have = [tagged_girl, boy, second_girl];
        let answer = vec![
            found_as(Girl, None, 100),
            found_as(Other, None, 300),
            found_as(Girl, None, 500),
            found_as(Girl, None, 700),
            found_as(Boy, None, 900),
        ];
        let joined = join_found(&have, answer, 20);
        let pairs: Vec<(usize, Option<FaceBox>)> = joined
            .paired
            .iter()
            .map(|p| (p.person, p.draft.face))
            .collect();
        // The girls in order, then the boy; an "other" is no one's.
        assert_eq!(
            pairs,
            [
                (0, Some(at(100, 100))),
                (2, Some(at(500, 100))),
                (1, Some(at(900, 100)))
            ]
        );
        let added: Vec<_> = joined.added.iter().map(|f| f.face).collect();
        assert_eq!(added, [Some(at(300, 100)), Some(at(700, 100))]);
        // A name on one side only isn't a match.
        let joined = join_found(&[typed(Girl, "saber")], vec![found_as(Girl, None, 100)], 20);
        assert!(joined.paired.is_empty());
        let joined = join_found(&have, vec![found_as(Girl, Some("saber"), 100)], 20);
        assert!(joined.paired.is_empty());
    }

    #[test]
    fn only_empty_fields_are_filled() {
        let mut saber = typed(FigureKind::Girl, "saber");
        saber.franchise = String::new();
        saber.slots.insert(Slot::Hair, "blonde_hair".into());
        saber.slots.insert(Slot::Eyes, " ".into());
        let mut answer = found_as(FigureKind::Girl, Some("saber"), 400);
        answer.franchise = Some("fate_(series)".into());
        answer.tags = BTreeMap::from([
            (Slot::Hair, vec!["short_hair".into()]),
            (Slot::Eyes, vec!["green_eyes".into()]),
            (Slot::Body, vec!["armor".into(), "dress".into()]),
        ]);
        let joined = join_found(&[saber.clone()], vec![answer], 20);
        let after = joined.paired.first().map(|p| p.draft.clone()).unwrap();
        assert_eq!(after.franchise, "fate_(series)");
        // What was typed stays; the empty slots get the AI's.
        assert_eq!(
            after.slots.get(&Slot::Hair).map(String::as_str),
            Some("blonde_hair")
        );
        assert_eq!(
            after.slots.get(&Slot::Eyes).map(String::as_str),
            Some("green_eyes")
        );
        assert_eq!(
            after.slots.get(&Slot::Body).map(String::as_str),
            Some("armor dress")
        );
        // A franchise typed stays too.
        saber.franchise = "fate_zero".into();
        let mut again = found_as(FigureKind::Girl, Some("saber"), 400);
        again.franchise = Some("fate_(series)".into());
        let joined = join_found(&[saber], vec![again], 20);
        assert_eq!(
            joined.paired.first().map(|p| p.draft.franchise.as_str()),
            Some("fate_zero")
        );
    }

    #[test]
    fn each_person_is_found_once() {
        // Two Sabers found, one typed: the second is someone new.
        let have = [typed(FigureKind::Girl, "saber")];
        let answer = vec![
            found_as(FigureKind::Girl, Some("saber"), 100),
            found_as(FigureKind::Girl, Some("saber"), 600),
        ];
        let joined = join_found(&have, answer, 20);
        assert_eq!(joined.paired.len(), 1);
        assert_eq!(joined.added.len(), 1);
        // Someone with a box already is matched by the box, not the name.
        let mut marked = typed(FigureKind::Girl, "saber");
        marked.face = Some(at(100, 100));
        let joined = join_found(
            &[marked],
            vec![found_as(FigureKind::Girl, Some("saber"), 105)],
            20,
        );
        assert_eq!(joined, Joined::default());
        // The empty person the editor starts with isn't anyone.
        let joined = join_found(
            &[Draft::default()],
            vec![found_as(FigureKind::Girl, None, 100)],
            20,
        );
        assert!(joined.paired.is_empty());
        assert_eq!(joined.added.len(), 1);
        // Pairing takes no room; only those added need it.
        let joined = join_found(
            &have,
            vec![
                found_as(FigureKind::Girl, Some("saber"), 100),
                found_as(FigureKind::Boy, None, 600),
            ],
            0,
        );
        assert_eq!((joined.paired.len(), joined.added.len()), (1, 0));
        // Found again with nothing new to give: no change, and no one added.
        let mut whole = typed(FigureKind::Girl, "saber");
        let bare = Figure {
            kind: FigureKind::Girl,
            character: Some("saber".into()),
            ..Figure::default()
        };
        assert_eq!(
            join_found(&[whole.clone()], vec![bare.clone()], 20),
            Joined::default()
        );
        whole.franchise = "x".into();
        assert_eq!(join_found(&[whole], vec![bare], 20), Joined::default());
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
                409,
                "ai_busy",
                "the people tool is busy with other pictures; try again later"
            ),
            Trouble::Busy
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
        assert!(Trouble::Busy.retries());
        assert!(Trouble::Provider(String::new()).retries());
        assert!(Trouble::Other.retries());
        assert!(!Trouble::Off.retries());
        assert!(!Trouble::NoPicture.retries());
        assert!(!Trouble::NoMemory.retries());
    }

    #[test]
    fn a_new_picture_without_people_is_waited_for() {
        let now = Utc::now();
        assert!(waits_for_people(&post(5, now), now, true, true));
        assert!(waits_for_people(&post(179, now), now, true, true));
        // The server's clock a little ahead.
        assert!(waits_for_people(&post(-30, now), now, true, true));
        // Too old, the tool is off, or the uploader turned the AI off.
        assert!(!waits_for_people(&post(180, now), now, true, true));
        assert!(!waits_for_people(&post(3600, now), now, true, true));
        assert!(!waits_for_people(&post(5, now), now, false, true));
        assert!(!waits_for_people(&post(5, now), now, true, false));
        // People already, a video or a GIF, the recycle bin.
        let mut marked = post(5, now);
        marked.figures = vec![found(Some(at(0, 0)))];
        assert!(!waits_for_people(&marked, now, true, true));
        for kind in [MediaKind::Video, MediaKind::Animation] {
            let mut moving = post(5, now);
            moving.kind = kind;
            assert!(!waits_for_people(&moving, now, true, true), "{kind:?}");
        }
        let mut binned = post(5, now);
        binned.deleted = true;
        assert!(!waits_for_people(&binned, now, true, true));
        // A copy from a list, which leaves the people out.
        let mut listed = post(5, now);
        listed.tags = vec!["photo".into()];
        assert!(!waits_for_people(&listed, now, true, true));
    }

    /// When the looks come, in seconds after the viewer opens a post that
    /// was `age` seconds old.
    fn looks(age: i64) -> Vec<i64> {
        let (mut at_ms, mut out) = (0_i64, Vec::new());
        while let Some(wait) = next_look(
            out.len(),
            TimeDelta::seconds(age) + TimeDelta::milliseconds(at_ms),
        ) {
            at_ms += i64::from(wait);
            out.push(at_ms / 1000);
            assert!(out.len() < 100, "{out:?}");
        }
        out
    }

    #[test]
    fn the_viewer_looks_at_three_eight_and_twenty_seconds_then_every_ten() {
        // Opened at once: every ten seconds until it's three minutes old.
        let mut expected = vec![3, 8, 20];
        expected.extend((30..=170).step_by(10));
        assert_eq!(looks(5), expected);
        // Opened late, it still gets its first three looks.
        assert_eq!(looks(170), [3, 8, 20]);
        assert_eq!(looks(150), [3, 8, 20]);
        assert_eq!(looks(130), [3, 8, 20, 30, 40]);
        // A server clock ahead makes it younger.
        assert_eq!(looks(-20).last(), Some(&190));
    }

    fn job(id: i64, url: &str, state: JobState, post_id: Option<i64>) -> ImportJob {
        ImportJob {
            id,
            url: url.into(),
            source: None,
            tags: Vec::new(),
            rating: Rating::Safe,
            state,
            post_id,
            error: None,
            created_at: Utc::now(),
            finished_at: None,
            progress: None,
        }
    }

    fn request(url: &str, autotag: Option<bool>) -> ImportRequest {
        ImportRequest {
            url: url.into(),
            tags: Vec::new(),
            rating: Rating::Safe,
            source: None,
            autotag,
        }
    }

    #[test]
    fn uploads_with_the_ai_off_are_not_waited_for() {
        let mut off = Unlooked::default();
        off.uploaded(10, Some(false));
        off.uploaded(11, Some(true));
        off.uploaded(12, None);
        assert!(!off.looked_for(10));
        assert!(off.looked_for(11) && off.looked_for(12) && off.looked_for(99));
        // Imports: by their address, until the server says which post.
        off.queued(
            &[
                request("https://a/1", Some(false)),
                request("https://a/2", None),
            ],
            &[
                job(1, "https://a/1", JobState::Queued, None),
                job(2, "https://a/2", JobState::Queued, None),
            ],
        );
        off.imports(&[job(1, "https://a/1", JobState::Running, None)]);
        assert!(off.looked_for(40));
        off.imports(&[
            job(1, "https://a/1", JobState::Done, Some(40)),
            job(2, "https://a/2", JobState::Done, Some(41)),
        ]);
        assert!(!off.looked_for(40));
        assert!(off.looked_for(41));
        // A duplicate is a post from before, and a failure makes none.
        off.queued(
            &[
                request("https://a/3", Some(false)),
                request("https://a/4", Some(false)),
            ],
            &[
                job(3, "https://a/3", JobState::Queued, None),
                job(4, "https://a/4", JobState::Queued, None),
            ],
        );
        off.imports(&[
            job(3, "https://a/3", JobState::Duplicate, Some(5)),
            job(4, "https://a/4", JobState::Failed, None),
        ]);
        assert!(off.looked_for(5));
        assert!(off.jobs.is_empty());
    }
}
