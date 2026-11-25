//! The people in a post without a browser: each person as the editor holds
//! them, what saving sends, the franchises the server filled in, how the
//! slots are grouped, and how people of the same kind are numbered.

use std::collections::BTreeMap;

use starbooru_core::tag::TagError;
use starbooru_core::{
    FaceBox, Figure, FigureError, FigureKind, FiguresUpdate, MAX_FIGURES, Post, Slot,
    clean_figures, normalize_tag,
};

use super::tags::BadTag;

/// A person as the editor holds them: each box as typed, and where their
/// face is.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Draft {
    pub kind: FigureKind,
    pub character: String,
    pub cosplay: String,
    pub franchise: String,
    pub slots: BTreeMap<Slot, String>,
    pub face: Option<FaceBox>,
}

impl Draft {
    pub fn of(figure: &Figure) -> Self {
        Self {
            kind: figure.kind,
            character: figure.character.clone().unwrap_or_default(),
            cosplay: figure.cosplay.clone().unwrap_or_default(),
            franchise: figure.franchise.clone().unwrap_or_default(),
            face: figure.face,
            slots: figure
                .tags
                .iter()
                .map(|(slot, tags)| (*slot, tags.join(" ")))
                .collect(),
        }
    }

    /// Nothing typed and no face: the person the editor starts with, or one
    /// added and left empty. Their kind alone says nothing.
    pub fn is_blank(&self) -> bool {
        let tidy = self.tidy();
        tidy == Self {
            kind: self.kind,
            ..Self::default()
        }
    }

    /// As typed, without the spaces that say nothing: names trimmed, one
    /// space between words, empty slots left out.
    fn tidy(&self) -> Self {
        Self {
            kind: self.kind,
            character: self.character.trim().to_owned(),
            cosplay: self.cosplay.trim().to_owned(),
            franchise: self.franchise.trim().to_owned(),
            face: self.face,
            slots: self
                .slots
                .iter()
                .map(|(slot, text)| (*slot, text.split_whitespace().collect::<Vec<_>>().join(" ")))
                .filter(|(_, text)| !text.is_empty())
                .collect(),
        }
    }
}

/// Whether the people typed differ from the `saved` ones, so leaving would
/// lose them. The empty person the editor starts with, when there's no one
/// yet, isn't a change.
pub fn people_changed(drafts: &[Draft], saved: &[Figure]) -> bool {
    let typed: Vec<Draft> = drafts.iter().map(Draft::tidy).collect();
    let before: Vec<Draft> = saved.iter().map(|f| Draft::of(f).tidy()).collect();
    let untouched =
        saved.is_empty() && matches!(typed.as_slice(), [only] if *only == Draft::default());
    typed != before && !untouched
}

/// One of a person's boxes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Character,
    Cosplay,
    Franchise,
    Slot(Slot),
}

/// Why the people can't be saved as they are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PeopleError {
    /// A word in one of person `person`'s boxes can't be a tag.
    Tag {
        person: usize,
        field: Field,
        bad: BadTag,
    },
    /// Person `person` has more tags than one person may.
    TooManyTags { person: usize },
    /// Person `person`'s face box isn't inside the picture.
    BadFace { person: usize },
    /// More people than a post may have.
    TooMany,
}

/// A box that holds one name, such as the character: `None` when empty.
fn one_name(text: &str) -> Result<Option<String>, TagError> {
    match normalize_tag(text.trim()) {
        Ok(name) => Ok(Some(name)),
        Err(TagError::Empty) => Ok(None),
        Err(e) => Err(e),
    }
}

/// A slot's tags: one per word, normalised, each once.
fn slot_tags(text: &str) -> Result<Vec<String>, BadTag> {
    let mut out: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match normalize_tag(word) {
            Ok(tag) if !out.contains(&tag) => out.push(tag),
            Ok(_) | Err(TagError::Empty) => {}
            Err(error) => {
                return Err(BadTag {
                    word: word.to_owned(),
                    error,
                });
            }
        }
    }
    Ok(out)
}

/// What saving `drafts` sends: names normalised, empty boxes left out, and
/// each person cleaned as the server cleans them (a tag in its first slot
/// only, no count tags, no category prefixes).
pub fn figures_update(drafts: &[Draft]) -> Result<FiguresUpdate, PeopleError> {
    if drafts.len() > MAX_FIGURES {
        return Err(PeopleError::TooMany);
    }
    let mut figures = Vec::with_capacity(drafts.len());
    for (person, draft) in drafts.iter().enumerate() {
        let name = |text: &str, field: Field| {
            one_name(text).map_err(|error| PeopleError::Tag {
                person,
                field,
                bad: BadTag {
                    word: text.trim().to_owned(),
                    error,
                },
            })
        };
        let character = name(&draft.character, Field::Character)?;
        let cosplay = name(&draft.cosplay, Field::Cosplay)?;
        let franchise = name(&draft.franchise, Field::Franchise)?;
        let mut tags = BTreeMap::new();
        for (slot, text) in &draft.slots {
            let found = slot_tags(text).map_err(|bad| PeopleError::Tag {
                person,
                field: Field::Slot(*slot),
                bad,
            })?;
            if !found.is_empty() {
                tags.insert(*slot, found);
            }
        }
        let figure = Figure {
            kind: draft.kind,
            character,
            cosplay,
            franchise,
            tags,
            face: draft.face,
            face_url: None,
        };
        // One at a time, so a refusal can say whose.
        match clean_figures(vec![figure]) {
            Ok(clean) => figures.extend(clean),
            Err(FigureError::TooManyTags) => return Err(PeopleError::TooManyTags { person }),
            Err(FigureError::BadFace) => return Err(PeopleError::BadFace { person }),
            Err(FigureError::TooMany) => return Err(PeopleError::TooMany),
            // It only runs `normalize_tag` again, which every word passed above.
            Err(FigureError::Tag(error)) => {
                return Err(PeopleError::Tag {
                    person,
                    field: Field::Character,
                    bad: BadTag {
                        word: draft.character.trim().to_owned(),
                        error,
                    },
                });
            }
        }
    }
    Ok(FiguresUpdate { figures })
}

/// The franchises the server filled in on saving, person by person: theirs
/// where `sent` gave the same person none and `saved` has one. The server
/// keeps the people in the order they were sent.
pub fn filled_in(sent: &[Figure], saved: &[Figure]) -> Vec<Option<String>> {
    saved
        .iter()
        .enumerate()
        .map(|(i, after)| {
            let before = sent.get(i)?;
            let same = before.kind == after.kind
                && before.character == after.character
                && before.cosplay == after.cosplay;
            (same && before.franchise.is_none())
                .then(|| after.franchise.clone())
                .flatten()
        })
        .collect()
}

/// Whether the server has yet to cut out a face of `post`'s people: it does
/// a moment after saving, so the post is asked for again until then.
pub fn faces_pending(post: &Post) -> bool {
    post.figures
        .iter()
        .any(|f| f.face.is_some() && f.face_url.is_none())
}

/// The two groups of slots in the editor: what a person wears, then how
/// they look.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotGroup {
    Clothing,
    Looks,
}

/// The slots in each group, in the order the server lists them.
pub fn slot_groups() -> [(SlotGroup, Vec<Slot>); 2] {
    let (clothing, looks): (Vec<Slot>, Vec<Slot>) =
        Slot::ALL.into_iter().partition(|s| s.is_clothing());
    [(SlotGroup::Clothing, clothing), (SlotGroup::Looks, looks)]
}

/// Numbers people within their kind when there are several of it: `[1, 0,
/// 2]` for girl, boy, girl, where `0` is the only one of a kind.
pub fn numbers(kinds: &[FigureKind]) -> Vec<usize> {
    kinds
        .iter()
        .enumerate()
        .map(|(i, kind)| {
            let same = |k: &&FigureKind| *k == kind;
            if kinds.iter().filter(same).count() > 1 {
                kinds.iter().take(i + 1).filter(same).count()
            } else {
                0
            }
        })
        .collect()
}

/// Whether `post` says who is in it. A post from a list leaves its people
/// out, and its tags' details with them; one without tags has no one.
pub fn people_known(post: &Post) -> bool {
    !post.tag_details.is_empty() || post.tags.is_empty()
}

/// What the viewer shows of People.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeopleShown {
    /// The card, or its editor in its place.
    Card,
    /// People went off, in another tab, while the editor held changes: it
    /// stays, saying so, until they're saved or cancelled.
    Kept,
    Hidden,
}

/// What the viewer shows, from the setting and whether the editor is open.
/// With People off, the editor is only still open if it held changes
/// (`closes_when_off`).
pub fn people_shown(on: bool, editing: bool) -> PeopleShown {
    match (on, editing) {
        (true, _) => PeopleShown::Card,
        (false, true) => PeopleShown::Kept,
        (false, false) => PeopleShown::Hidden,
    }
}

/// Whether turning People off closes the editor: only when closing loses
/// nothing.
pub fn closes_when_off(editing: bool, unsaved: bool) -> bool {
    editing && !unsaved
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft(kind: FigureKind, slots: &[(Slot, &str)]) -> Draft {
        Draft {
            kind,
            slots: slots.iter().map(|(s, t)| (*s, (*t).to_owned())).collect(),
            ..Draft::default()
        }
    }

    fn face(x: u16, y: u16, w: u16, h: u16) -> FaceBox {
        FaceBox { x, y, w, h }
    }

    #[test]
    fn a_figure_becomes_a_draft_and_back() {
        let figure = Figure {
            kind: FigureKind::Boy,
            character: Some("link".into()),
            franchise: Some("the_legend_of_zelda".into()),
            tags: BTreeMap::from([(Slot::Hair, vec!["blonde_hair".into(), "ponytail".into()])]),
            face: Some(face(412, 96, 130, 180)),
            ..Figure::default()
        };
        let d = Draft::of(&figure);
        assert_eq!(
            d.slots.get(&Slot::Hair).map(String::as_str),
            Some("blonde_hair ponytail")
        );
        assert_eq!(d.character, "link");
        assert_eq!(d.franchise, "the_legend_of_zelda");
        // What the server sent comes back as it was.
        assert_eq!(
            figures_update(&[d]),
            Ok(FiguresUpdate {
                figures: vec![figure.clone()]
            })
        );
        // The crop's address is the server's: it never goes back.
        let shown = Figure {
            face_url: Some("/media/face/x.jpg".into()),
            ..figure.clone()
        };
        assert_eq!(
            figures_update(&[Draft::of(&shown)]).map(|u| u.figures),
            Ok(vec![figure])
        );
    }

    #[test]
    fn the_franchise_and_face_always_go_and_empty_means_none() {
        let mut d = draft(FigureKind::Girl, &[]);
        d.character = "saber".into();
        d.franchise = "  Fate (series) ".into();
        let sent = figures_update(std::slice::from_ref(&d)).map(|u| u.figures);
        let franchise = sent
            .as_ref()
            .ok()
            .and_then(|f| f.first())
            .and_then(|f| f.franchise.clone());
        assert_eq!(franchise.as_deref(), Some("fate_(series)"));
        // Cleared, it's null: the server takes that as "none", not "as before".
        d.franchise = " ".into();
        let json = figures_update(&[d]).map(|u| serde_json::to_value(u).unwrap_or_default());
        let first = json.ok().and_then(|j| j["figures"].get(0).cloned());
        let first = first.unwrap_or_default();
        assert!(
            first
                .get("franchise")
                .is_some_and(serde_json::Value::is_null)
        );
        assert!(first.get("face").is_some_and(serde_json::Value::is_null));
        assert!(first.get("face_url").is_none());
        // A franchise that can't be a tag says where it is.
        let mut bad = draft(FigureKind::Boy, &[]);
        bad.franchise = "-fate".into();
        assert!(matches!(
            figures_update(&[bad]),
            Err(PeopleError::Tag {
                person: 0,
                field: Field::Franchise,
                ..
            })
        ));
        // A box out of the picture is turned down before it's sent.
        let mut out = draft(FigureKind::Girl, &[]);
        out.face = Some(face(900, 0, 200, 10));
        assert_eq!(
            figures_update(&[out]),
            Err(PeopleError::BadFace { person: 0 })
        );
    }

    #[test]
    fn a_face_without_its_crop_is_still_being_cut() {
        let with = |face: Option<FaceBox>, url: Option<&str>| Figure {
            face,
            face_url: url.map(str::to_owned),
            ..Figure::default()
        };
        let post = |figures: Vec<Figure>| -> Post {
            let mut post: Post = serde_json::from_value(serde_json::json!({
                "id": 1, "kind": "image", "mime": "image/png", "file_size": 1,
                "width": 1, "height": 1, "sha256": "x", "rating": "safe", "score": 0,
                "my_vote": 0, "source": null, "tags": [],
                "created_at": "2026-01-01T00:00:00Z", "deleted": false,
                "file_url": "/media/x", "thumb_url": null,
            }))
            .unwrap();
            post.figures = figures;
            post
        };
        assert!(!faces_pending(&post(Vec::new())));
        assert!(!faces_pending(&post(vec![with(None, None)])));
        let done = with(Some(face(1, 1, 10, 10)), Some("/media/face/a.jpg"));
        assert!(!faces_pending(&post(vec![done.clone()])));
        assert!(faces_pending(&post(vec![
            done,
            with(Some(face(1, 1, 10, 10)), None)
        ])));
    }

    #[test]
    fn the_server_filling_in_a_franchise_is_told_apart() {
        let person = |character: &str, franchise: Option<&str>| Figure {
            kind: FigureKind::Girl,
            character: Some(character.into()),
            franchise: franchise.map(str::to_owned),
            ..Figure::default()
        };
        let sent = [
            person("saber", None),
            person("rin", Some("fate")),
            person("miku", None),
        ];
        let saved = [
            person("saber", Some("fate")),
            person("rin", Some("fate")),
            person("miku", None),
            person("extra", Some("x")),
        ];
        assert_eq!(
            filled_in(&sent, &saved),
            [Some("fate".into()), None, None, None]
        );
        // Someone else in that place isn't the same person.
        assert_eq!(
            filled_in(&[person("rin", None)], &[person("saber", Some("fate"))]),
            [None]
        );
        assert!(filled_in(&sent, &[]).is_empty());
    }

    #[test]
    fn empty_boxes_go_and_tags_are_normalised() {
        let mut d = draft(
            FigureKind::Girl,
            &[
                (Slot::Shoes, "  Boots   sandals boots "),
                (Slot::Pose, " "),
                (Slot::Hair, "Blue Hair"),
                (Slot::Other, "_ __"),
            ],
        );
        d.character = "  Hatsune Miku ".into();
        d.cosplay = "   ".into();
        let sent = figures_update(&[d]).map(|u| u.figures);
        assert_eq!(
            sent,
            Ok(vec![Figure {
                kind: FigureKind::Girl,
                character: Some("hatsune_miku".into()),
                tags: BTreeMap::from([
                    (Slot::Shoes, vec!["boots".into(), "sandals".into()]),
                    (Slot::Hair, vec!["blue".into(), "hair".into()]),
                ]),
                ..Figure::default()
            }])
        );
        // No one at all clears the people.
        assert_eq!(figures_update(&[]), Ok(FiguresUpdate::default()));
    }

    #[test]
    fn saving_cleans_as_the_server_does() {
        let mut d = draft(
            FigureKind::Other,
            &[(Slot::Head, "hat 1girl"), (Slot::Body, "artist:dress hat")],
        );
        d.cosplay = "Saber_(cosplay)".into();
        let sent = figures_update(&[d]).map(|u| u.figures);
        assert_eq!(
            sent,
            Ok(vec![Figure {
                kind: FigureKind::Other,
                cosplay: Some("saber".into()),
                // The count tag goes, the prefix goes, and `hat` stays in its first slot.
                tags: BTreeMap::from([
                    (Slot::Head, vec!["hat".into()]),
                    (Slot::Body, vec!["dress".into()]),
                ]),
                ..Figure::default()
            }])
        );
    }

    #[test]
    fn a_bad_word_says_whose_and_where() {
        let ok = draft(FigureKind::Girl, &[]);
        let bad = draft(FigureKind::Boy, &[(Slot::Eyes, "red_eyes -closed")]);
        assert_eq!(
            figures_update(&[ok.clone(), bad]),
            Err(PeopleError::Tag {
                person: 1,
                field: Field::Slot(Slot::Eyes),
                bad: BadTag {
                    word: "-closed".into(),
                    error: TagError::BadPrefix("-closed".into())
                },
            })
        );
        let mut who = ok.clone();
        who.cosplay = format!(" {} ", "x".repeat(129));
        assert!(matches!(
            figures_update(&[who]),
            Err(PeopleError::Tag { person: 0, field: Field::Cosplay, bad }) if bad.error == TagError::TooLong
        ));
        let many: Vec<String> = (0..=starbooru_core::MAX_FIGURE_TAGS)
            .map(|i| format!("t{i}"))
            .collect();
        let crowded = draft(FigureKind::Girl, &[(Slot::Other, &many.join(" "))]);
        assert_eq!(
            figures_update(&[ok.clone(), crowded]),
            Err(PeopleError::TooManyTags { person: 1 })
        );
        assert_eq!(
            figures_update(&vec![ok; MAX_FIGURES + 1]),
            Err(PeopleError::TooMany)
        );
    }

    #[test]
    fn only_other_people_are_unsaved_changes() {
        let link = Figure {
            kind: FigureKind::Boy,
            character: Some("link".into()),
            franchise: Some("zelda".into()),
            tags: BTreeMap::from([(Slot::Hair, vec!["blonde_hair".into(), "ponytail".into()])]),
            face: Some(face(10, 10, 100, 100)),
            ..Figure::default()
        };
        let saved = [link.clone()];
        // As the editor holds them: every slot, most of them empty.
        let mut typed = Draft::of(&link);
        for slot in Slot::ALL {
            typed.slots.entry(slot).or_default();
        }
        assert!(!people_changed(std::slice::from_ref(&typed), &saved));
        // Spaces that say nothing.
        typed.character = " link ".into();
        typed
            .slots
            .insert(Slot::Hair, " blonde_hair\n ponytail ".into());
        assert!(!people_changed(std::slice::from_ref(&typed), &saved));
        // A tag, a name, a kind, or a person more or fewer.
        let mut more = typed.clone();
        more.slots.insert(Slot::Eyes, "blue_eyes".into());
        assert!(people_changed(&[more], &saved));
        let mut named = typed.clone();
        named.cosplay = "zelda".into();
        assert!(people_changed(&[named], &saved));
        let mut girl = typed.clone();
        girl.kind = FigureKind::Girl;
        assert!(people_changed(&[girl], &saved));
        // A franchise or a face box, moved, cleared or not.
        let mut series = typed.clone();
        series.franchise = " zelda ".into();
        assert!(!people_changed(std::slice::from_ref(&series), &saved));
        series.franchise.clear();
        assert!(people_changed(&[series], &saved));
        let mut moved = typed.clone();
        moved.face = Some(face(11, 10, 100, 100));
        assert!(people_changed(&[moved], &saved));
        let mut faceless = typed.clone();
        faceless.face = None;
        assert!(people_changed(&[faceless], &saved));
        assert!(people_changed(&[typed.clone(), Draft::default()], &saved));
        assert!(people_changed(&[], &saved));
        // No one yet: the empty person the editor starts with is no change.
        assert!(!people_changed(&[Draft::default()], &[]));
        assert!(!people_changed(&[], &[]));
        assert!(people_changed(&[typed], &[]));
        assert!(people_changed(
            &[draft(FigureKind::Girl, &[(Slot::Hair, "red_hair")])],
            &[]
        ));
    }

    #[test]
    fn a_person_with_nothing_typed_is_blank() {
        assert!(Draft::default().is_blank());
        let mut d = draft(FigureKind::Other, &[(Slot::Hair, "  "), (Slot::Eyes, "")]);
        d.character = " ".into();
        assert!(d.is_blank());
        let mut named = d.clone();
        named.franchise = "zelda".into();
        assert!(!named.is_blank());
        let mut tagged = d.clone();
        tagged.slots.insert(Slot::Hair, "red_hair".into());
        assert!(!tagged.is_blank());
        let mut marked = d;
        marked.face = Some(face(1, 1, 10, 10));
        assert!(!marked.is_blank());
    }

    #[test]
    fn clothing_comes_first_then_looks() {
        let [(first, clothing), (second, looks)] = slot_groups();
        assert_eq!(first, SlotGroup::Clothing);
        assert_eq!(
            clothing,
            [
                Slot::Head,
                Slot::Body,
                Slot::Bottom,
                Slot::Shoes,
                Slot::Wear
            ]
        );
        assert_eq!(second, SlotGroup::Looks);
        assert_eq!(
            looks,
            [
                Slot::Shape,
                Slot::Eyes,
                Slot::Skin,
                Slot::Hair,
                Slot::Pose,
                Slot::Other
            ]
        );
    }

    #[test]
    fn numbers_count_within_each_kind() {
        use FigureKind::{Boy, Girl, Other};
        assert_eq!(numbers(&[Girl, Boy, Girl]), [1, 0, 2]);
        assert_eq!(numbers(&[Other]), [0]);
        assert_eq!(numbers(&[Boy, Boy, Boy]), [1, 2, 3]);
        assert!(numbers(&[]).is_empty());
    }

    #[test]
    fn only_a_whole_post_says_who_is_in_it() {
        let mut post: Post = serde_json::from_value(serde_json::json!({
            "id": 45, "kind": "image", "mime": "image/png", "file_size": 1,
            "width": 1, "height": 1, "sha256": "x", "rating": "safe", "score": 0,
            "my_vote": 0, "source": null, "tags": [],
            "created_at": "2026-01-01T00:00:00Z", "deleted": false,
            "file_url": "/media/x", "thumb_url": null,
        }))
        .unwrap();
        assert!(people_known(&post));
        post.tags = vec!["1girl".into()];
        assert!(!people_known(&post));
        post.tag_details = vec![starbooru_core::PostTag {
            name: "1girl".into(),
            category: starbooru_core::TagCategory::General,
            section: None,
        }];
        assert!(people_known(&post));
    }

    #[test]
    fn turning_people_off_never_loses_an_edit() {
        assert_eq!(people_shown(true, false), PeopleShown::Card);
        assert_eq!(people_shown(true, true), PeopleShown::Card);
        assert_eq!(people_shown(false, false), PeopleShown::Hidden);
        // Off in another tab: an editor with nothing to lose closes, one
        // with changes stays until they're saved or cancelled.
        assert!(closes_when_off(true, false));
        assert!(!closes_when_off(true, true));
        assert!(!closes_when_off(false, false));
        assert_eq!(people_shown(false, true), PeopleShown::Kept);
        // Saved or cancelled, the editor closes and the card goes.
        assert_eq!(people_shown(false, false), PeopleShown::Hidden);
    }
}
