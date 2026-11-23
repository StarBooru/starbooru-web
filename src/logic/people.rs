//! The people in a post without a browser: each person as the editor holds
//! them, what saving sends, how the slots are grouped, and how people of the
//! same kind are numbered.

use std::collections::BTreeMap;

use starbooru_core::tag::TagError;
use starbooru_core::{
    Figure, FigureError, FigureKind, FiguresUpdate, MAX_FIGURES, Post, Slot, clean_figures,
    normalize_tag,
};

use super::tags::BadTag;

/// A person as the editor holds them: each box as typed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Draft {
    pub kind: FigureKind,
    pub character: String,
    pub cosplay: String,
    pub slots: BTreeMap<Slot, String>,
}

impl Draft {
    pub fn of(figure: &Figure) -> Self {
        Self {
            kind: figure.kind,
            character: figure.character.clone().unwrap_or_default(),
            cosplay: figure.cosplay.clone().unwrap_or_default(),
            slots: figure
                .tags
                .iter()
                .map(|(slot, tags)| (*slot, tags.join(" ")))
                .collect(),
        }
    }

    /// As typed, without the spaces that say nothing: names trimmed, one
    /// space between words, empty slots left out.
    fn tidy(&self) -> Self {
        Self {
            kind: self.kind,
            character: self.character.trim().to_owned(),
            cosplay: self.cosplay.trim().to_owned(),
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
            tags,
        };
        // One at a time, so a refusal can say whose.
        match clean_figures(vec![figure]) {
            Ok(clean) => figures.extend(clean),
            Err(FigureError::TooManyTags) => return Err(PeopleError::TooManyTags { person }),
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

    #[test]
    fn a_figure_becomes_a_draft_and_back() {
        let figure = Figure {
            kind: FigureKind::Boy,
            character: Some("link".into()),
            cosplay: None,
            tags: BTreeMap::from([(Slot::Hair, vec!["blonde_hair".into(), "ponytail".into()])]),
        };
        let d = Draft::of(&figure);
        assert_eq!(
            d.slots.get(&Slot::Hair).map(String::as_str),
            Some("blonde_hair ponytail")
        );
        assert_eq!(d.character, "link");
        assert_eq!(
            figures_update(&[d]),
            Ok(FiguresUpdate {
                figures: vec![figure]
            })
        );
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
                cosplay: None,
                tags: BTreeMap::from([
                    (Slot::Shoes, vec!["boots".into(), "sandals".into()]),
                    (Slot::Hair, vec!["blue".into(), "hair".into()]),
                ]),
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
                character: None,
                cosplay: Some("saber".into()),
                // The count tag goes, the prefix goes, and `hat` stays in its first slot.
                tags: BTreeMap::from([
                    (Slot::Head, vec!["hat".into()]),
                    (Slot::Body, vec!["dress".into()]),
                ]),
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
            cosplay: None,
            tags: BTreeMap::from([(Slot::Hair, vec!["blonde_hair".into(), "ponytail".into()])]),
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
}
