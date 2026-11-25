//! The viewer's People card: who is in the picture, each with their face,
//! the character, their franchise and whom they dress as, then their clothes
//! and looks by slot; and in its place the editor for them, where each
//! person's face is marked on the picture (`faces.rs`). Saving goes through
//! `PUT /posts/{id}/figures`: the server keeps the post's tags in step,
//! fills in the franchise of a character it knows, and cuts each face out a
//! moment later.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;

use leptos::ev::{MouseEvent, SubmitEvent};
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{
    FaceBox, Figure, FigureKind, MAX_FIGURE_TAGS, MAX_FIGURES, Post, Slot, TagSuggestions,
    cosplay_tag,
};
use wasm_bindgen::JsCast;

use super::faces::{Faces, Mark, reveal_stage};
use super::side::chip;
use crate::api;
use crate::components::banner::ErrorBanner;
use crate::components::icons::{Ico, Icon};
use crate::components::select::{Choice, Select};
use crate::components::tag_box::TagBox;
use crate::logic::faces::crop_view;
use crate::logic::people::{
    Draft, Field, PeopleError, SlotGroup, figures_update, filled_in, numbers, people_changed,
    people_known, slot_groups,
};
use crate::logic::viewer::css_url;
use crate::pages::{bad_tag_text, figure_kind_label};
use crate::route;

thread_local! {
    /// The franchises the server filled in when post `.0`'s people were
    /// last saved here, person by person.
    static FILLED: RefCell<Option<(i64, Vec<Option<String>>)>> = const { RefCell::new(None) };
}

/// The franchise the server filled in for person `i` of post `id`, if it
/// is still `franchise`.
fn filled(id: i64, i: usize, franchise: Option<&str>) -> Option<String> {
    FILLED.with_borrow(|f| {
        let (of, list) = f.as_ref()?;
        let was = list.get(i)?.as_deref()?;
        (*of == id && Some(was) == franchise).then(|| was.to_owned())
    })
}

fn slot_label(slot: Slot) -> String {
    match slot {
        Slot::Head => t!("slot-head"),
        Slot::Body => t!("slot-body"),
        Slot::Bottom => t!("slot-bottom"),
        Slot::Shoes => t!("slot-shoes"),
        Slot::Wear => t!("slot-wear"),
        Slot::Shape => t!("slot-shape"),
        Slot::Eyes => t!("slot-eyes"),
        Slot::Skin => t!("slot-skin"),
        Slot::Hair => t!("slot-hair"),
        Slot::Pose => t!("slot-pose"),
        Slot::Other => t!("slot-other"),
    }
}

fn group_label(group: SlotGroup) -> String {
    match group {
        SlotGroup::Clothing => t!("people-clothing"),
        SlotGroup::Looks => t!("people-looks"),
    }
}

/// A person's name on the card: their kind, numbered when the picture has
/// several of it ("Girl 2"). `number` is 0 for the only one.
fn person_label(kind: FigureKind, number: usize) -> String {
    if number == 0 {
        figure_kind_label(kind)
    } else {
        t!(
            "people-numbered",
            kind = figure_kind_label(kind),
            n = number
        )
    }
}

/// A person in a few words: who they are and whom they dress as.
fn person_summary(figure: &Figure, number: usize) -> String {
    let person = person_label(figure.kind, number);
    match (figure.character.clone(), figure.cosplay.clone()) {
        (None, None) => person,
        (Some(character), None) => {
            t!(
                "people-summary-character",
                person = person,
                character = character
            )
        }
        (None, Some(cosplay)) => t!("people-summary-cosplay", person = person, cosplay = cosplay),
        (Some(character), Some(cosplay)) => t!(
            "people-summary-both",
            person = person,
            character = character,
            cosplay = cosplay
        ),
    }
}

/// The id of one of person `key`'s boxes in the editor.
fn box_id(key: u64, field: Field) -> String {
    match field {
        Field::Character => format!("person-{key}-character"),
        Field::Cosplay => format!("person-{key}-cosplay"),
        Field::Franchise => format!("person-{key}-franchise"),
        Field::Slot(slot) => format!("person-{key}-{}", slot.as_str()),
    }
}

/// Puts the caret in the box with this id, once it's on the page.
fn focus_box(id: String) {
    request_animation_frame(move || {
        if let Some(el) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(&id))
            .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let _ = el.focus();
        }
    });
}

/// What the card needs of the picture for its faces.
#[derive(Clone, Copy)]
pub(super) struct Picture {
    /// The address the stage shows, for faces the server hasn't cut yet;
    /// `None` while the blacklist covers the post.
    pub src: Signal<Option<String>>,
    /// The picture's width and height.
    pub size: Signal<Option<(i32, i32)>>,
    /// Boxes can show on the stage: a still picture, not covered.
    pub boxes: Signal<bool>,
    /// The blacklist covers the post: no faces of it either.
    pub covered: Signal<bool>,
}

/// A person's face on a disc, numbered as their box on the picture is: the
/// server's crop, or before it has one the picture cut the same way, or
/// the person icon without a box. `marking` rings it while the picture
/// waits for the box to be drawn.
#[component]
fn FaceDisc(
    #[prop(into)] face: Signal<Option<FaceBox>>,
    #[prop(into)] crop: Signal<Option<String>>,
    picture: Picture,
    #[prop(into)] number: Signal<usize>,
    #[prop(optional, into)] marking: Signal<bool>,
) -> impl IntoView {
    let broken = RwSignal::new(None::<String>);
    let inner = move || {
        let crop = crop
            .get()
            .filter(|u| !picture.covered.get() && broken.with(|b| b.as_deref() != Some(u)));
        if let Some(url) = crop {
            let failed = url.clone();
            return view! {
                <img
                    src=api::media(&url)
                    alt=""
                    decoding="async"
                    draggable="false"
                    on:error=move |_| broken.set(Some(failed.clone()))
                />
            }
            .into_any();
        }
        let cut = face
            .get()
            .and_then(|f| crop_view(f, picture.size.get()))
            .zip(picture.src.get());
        match cut {
            Some((view, src)) => {
                let style = format!("background-image: {}; {}", css_url(&src), view.style());
                view! { <span class="face-crop" style=style></span> }.into_any()
            }
            None => view! { <Ico icon=Icon::User size=22 /> }.into_any(),
        }
    };
    view! {
        <span
            class="face-disc"
            class:empty=move || face.with(Option::is_none)
            class:marking=move || marking.get()
            aria-hidden="true"
        >
            {inner}
            <span class="face-num">{move || number.get()}</span>
        </span>
    }
}

/// The people of the post on the page, and the way to change them for
/// whoever `editable` says may. `editing` says the editor is open, which
/// `on_edit` asks for; `unsaved`, whether closing it would lose changes;
/// `kept`, that People went off with the editor holding changes. `faces`
/// is shared with the stage, which outlines whom the card points at.
#[component]
pub fn PeopleCard(
    #[prop(into)] post: Signal<Option<Arc<Post>>>,
    #[prop(into)] editable: Signal<bool>,
    #[prop(into)] editing: Signal<bool>,
    unsaved: RwSignal<bool>,
    #[prop(into)] kept: Signal<bool>,
    faces: Faces,
    picture: Picture,
    on_edit: Callback<()>,
    on_cancel: Callback<()>,
    on_saved: Callback<Post>,
) -> impl IntoView {
    // A post from a list leaves its people out: nothing to edit until the
    // whole of it comes.
    let known = Memo::new(move |_| post.with(|p| p.as_deref().is_some_and(people_known)));
    let figures = Memo::new(move |_| {
        post.with(|p| {
            p.as_ref()
                .map(|p| (p.id, p.figures.clone()))
                .unwrap_or_default()
        })
    });
    let start = move |_| {
        if editable.get_untracked() && known.get_untracked() {
            on_edit.run(());
        }
    };
    let none = move || figures.with(|(_, f)| f.is_empty());

    let body = move || {
        if !known.get() {
            return None;
        }
        let (id, figures) = figures.get();
        if figures.is_empty() {
            return Some(
                view! { <p class="muted hint">{move || t!("people-none")}</p> }.into_any(),
            );
        }
        let kinds: Vec<FigureKind> = figures.iter().map(|f| f.kind).collect();
        let numbers = numbers(&kinds);
        let list = figures
            .into_iter()
            .enumerate()
            .map(|(i, figure)| {
                let number = numbers.get(i).copied().unwrap_or(0);
                person(figure, id, i, number, faces, picture)
            })
            .collect_view();
        Some(view! { <div class="figures" data-press="tag">{list}</div> }.into_any())
    };
    let shown = move || {
        view! {
            <section class="card people" id="people">
                <div class="card-head">
                    <h2 class="card-title">{move || t!("people-title")}</h2>
                    <Show when=move || editable.get() && known.get()>
                        <button
                            type="button"
                            class="btn text small"
                            aria-label=move || (!none()).then(|| t!("people-edit-label"))
                            on:click=start
                        >
                            {move || {
                                if none() {
                                    view! {
                                        <Ico icon=Icon::Add size=18 />
                                        {t!("people-add")}
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <Ico icon=Icon::Edit size=18 />
                                        {t!("people-edit")}
                                    }
                                        .into_any()
                                }
                            }}
                        </button>
                    </Show>
                </div>
                {body}
            </section>
        }
    };
    move || {
        if editing.get() {
            let figures = post.get_untracked()?.figures.clone();
            let id = post.with_untracked(|p| p.as_ref().map(|p| p.id))?;
            Some(
                view! {
                    <PeopleEditor
                        id=id
                        figures=figures
                        unsaved=unsaved
                        kept=kept
                        faces=faces
                        picture=picture
                        on_cancel=on_cancel
                        on_saved=on_saved
                    />
                }
                .into_any(),
            )
        } else {
            Some(shown().into_any())
        }
    }
}

/// One person on the card: their face, kind, who they are, their franchise
/// and whom they dress as, then their tags slot by slot. Pointing at them
/// outlines their face on the picture, and their face's disc brings the
/// picture into sight.
fn person(
    figure: Figure,
    id: i64,
    index: usize,
    number: usize,
    faces: Faces,
    picture: Picture,
) -> impl IntoView {
    let kind = figure.kind;
    let key = u64::try_from(index).unwrap_or(u64::MAX);
    let summary = {
        let figure = figure.clone();
        move || person_summary(&figure, number)
    };
    let light = move || faces.lit.set(Some(key));
    let dim = move || {
        if faces.lit.get_untracked() == Some(key) {
            faces.lit.set(None);
        }
    };
    let face = figure.face;
    let crop = figure.face_url.clone();
    let disc = move || {
        let crop = crop.clone();
        let disc = view! {
            <FaceDisc
                face=Signal::derive(move || face)
                crop=Signal::derive(move || crop.clone())
                picture=picture
                number=Signal::derive(move || index + 1)
            />
        };
        if face.is_some() && picture.boxes.get() {
            let label = t!("people-face-show", person = person_label(kind, number));
            view! {
                <button
                    type="button"
                    class="face-press"
                    aria-label=label
                    aria-pressed=move || {
                        if faces.pinned.get() == Some(key) { "true" } else { "false" }
                    }
                    on:click=move |_| {
                        if faces.pinned.get_untracked() == Some(key) {
                            faces.pinned.set(None);
                        } else {
                            faces.pinned.set(Some(key));
                            reveal_stage();
                        }
                    }
                >
                    {disc}
                </button>
            }
            .into_any()
        } else {
            disc.into_any()
        }
    };
    let character = figure
        .character
        .clone()
        .map(|name| chip(name, "chip cat-character"));
    let franchise = figure.franchise.clone().map(|name| {
        // Not typed: the server knew the character's.
        let auto = filled(id, index, Some(&name)).is_some();
        let title = auto.then(|| t!("people-filled-title"));
        let (href, tag) = (route::gallery(&name), name.clone());
        view! {
            <a class="chip cat-copyright" class:filled=auto href=href data-tag=tag title=title>
                {name}
            </a>
        }
    });
    let cosplay = figure.cosplay.clone().map(|name| {
        let tag = cosplay_tag(&name);
        view! {
            <span class="muted">{move || t!("people-as")}</span>
            <a class="chip cat-character" href=route::gallery(&tag) data-tag=tag.clone()>
                {name}
            </a>
        }
    });
    let slots = (!figure.tags.is_empty()).then(|| {
        let rows = figure
            .tags
            .into_iter()
            .map(|(slot, tags)| {
                let chips = tags.into_iter().map(|t| chip(t, "chip")).collect_view();
                view! {
                    <dt>{move || slot_label(slot)}</dt>
                    <dd class="chips">{chips}</dd>
                }
            })
            .collect_view();
        view! { <dl class="figure-slots">{rows}</dl> }
    });
    view! {
        <section
            class="figure"
            class:lit=move || faces.lit.get().or(faces.pinned.get()) == Some(key)
            role="group"
            aria-label=summary
            on:mouseenter=move |_| light()
            on:mouseleave=move |_| dim()
            on:focusin=move |_| light()
            on:focusout=move |_| dim()
        >
            <div class="figure-head">
                {disc}
                <div class="figure-who">
                    <span class="figure-kind">{move || person_label(kind, number)}</span>
                    <div class="figure-names">{character} {franchise} {cosplay}</div>
                </div>
            </div>
            {slots}
        </section>
    }
}

/// A person in the editor: each box its own signal, for its tag box, and
/// their face's box, which the stage moves.
#[derive(Clone)]
struct Row {
    key: u64,
    kind: ArcRwSignal<FigureKind>,
    character: ArcRwSignal<String>,
    cosplay: ArcRwSignal<String>,
    franchise: ArcRwSignal<String>,
    slots: BTreeMap<Slot, ArcRwSignal<String>>,
    face: ArcRwSignal<Option<FaceBox>>,
    /// The box the server has for them and its crop, which shows while
    /// the box stays where it was.
    saved: Option<(FaceBox, String)>,
    /// The franchise the server filled in at the last save.
    filled: Option<String>,
}

impl Row {
    fn new(
        key: u64,
        draft: Draft,
        saved: Option<(FaceBox, String)>,
        filled: Option<String>,
    ) -> Self {
        let mut typed = draft.slots;
        Self {
            key,
            kind: ArcRwSignal::new(draft.kind),
            character: ArcRwSignal::new(draft.character),
            cosplay: ArcRwSignal::new(draft.cosplay),
            franchise: ArcRwSignal::new(draft.franchise),
            slots: Slot::ALL
                .into_iter()
                .map(|slot| {
                    let text = typed.remove(&slot).unwrap_or_default();
                    (slot, ArcRwSignal::new(text))
                })
                .collect(),
            face: ArcRwSignal::new(draft.face),
            saved,
            filled,
        }
    }

    fn draft(&self) -> Draft {
        Draft {
            kind: self.kind.get_untracked(),
            character: self.character.get_untracked(),
            cosplay: self.cosplay.get_untracked(),
            franchise: self.franchise.get_untracked(),
            slots: self
                .slots
                .iter()
                .map(|(slot, text)| (*slot, text.get_untracked()))
                .collect(),
            face: self.face.get_untracked(),
        }
    }

    /// Makes what's reading this follow every box of the row.
    fn track(&self) {
        self.kind.track();
        self.character.track();
        self.cosplay.track();
        self.franchise.track();
        self.face.track();
        for text in self.slots.values() {
            text.track();
        }
    }
}

/// The editor, in the card's place, for the post `id` whose people were
/// `figures` when editing began. `on_saved` gets the post as the server has
/// it after saving.
#[component]
fn PeopleEditor(
    id: i64,
    figures: Vec<Figure>,
    unsaved: RwSignal<bool>,
    kept: Signal<bool>,
    faces: Faces,
    picture: Picture,
    on_cancel: Callback<()>,
    on_saved: Callback<Post>,
) -> impl IntoView {
    let next_key = StoredValue::new(0_u64);
    // `None` once the editor has gone, as it may have by an answer.
    let make = move |draft: Draft, saved: Option<(FaceBox, String)>, filled: Option<String>| {
        let key = next_key.try_get_value()?;
        next_key.try_set_value(key.wrapping_add(1));
        Some(Row::new(key, draft, saved, filled))
    };
    // Someone to fill in at once when there's no one yet.
    let first: Vec<Row> = if figures.is_empty() {
        make(Draft::default(), None, None).into_iter().collect()
    } else {
        figures
            .iter()
            .enumerate()
            .filter_map(|(i, f)| {
                let saved = f.face.zip(f.face_url.clone());
                make(Draft::of(f), saved, filled(id, i, f.franchise.as_deref()))
            })
            .collect()
    };
    let rows = RwSignal::new(first);
    let saved = StoredValue::new(figures);
    let changed = Memo::new(move |_| {
        rows.with(|r| {
            r.iter().for_each(Row::track);
            let drafts: Vec<Draft> = r.iter().map(Row::draft).collect();
            saved.with_value(|s| people_changed(&drafts, s))
        })
    });
    Effect::new(move |_| {
        let now = changed.get();
        if unsaved.get_untracked() != now {
            unsaved.set(now);
        }
    });
    // The stage draws the boxes the rows hold.
    Effect::new(move |_| {
        let marks: Vec<Mark> = rows.with(|r| {
            r.iter()
                .map(|row| Mark {
                    key: row.key,
                    face: row.face.clone(),
                })
                .collect()
        });
        faces.marks.set(marks);
    });
    on_cleanup(move || {
        unsaved.try_set(false);
        faces.marks.try_set(Vec::new());
        faces.reset();
    });
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    // What can't be saved as typed, and the box it's in.
    let problem = RwSignal::new(None::<String>);
    let invalid = RwSignal::new(None::<(u64, Field)>);
    let status = LocalResource::new(api::ai_status);
    let ai = move || status.get().flatten().is_some_and(|s| s.tagging);
    let asking = RwSignal::new(false);
    let none_found = RwSignal::new(false);

    if let Some(key) = rows.with_untracked(|r| r.first().map(|row| row.key)) {
        focus_box(box_id(key, Field::Character));
    }

    // Person `key`'s heading, numbered among those of their kind.
    let label_of = move |key: u64| {
        rows.with(|r| {
            let kinds: Vec<FigureKind> = r.iter().map(|row| row.kind.get()).collect();
            let at = r.iter().position(|row| row.key == key)?;
            let number = numbers(&kinds).get(at).copied().unwrap_or(0);
            Some(person_label(*kinds.get(at)?, number))
        })
        .unwrap_or_default()
    };
    // Their place in the list, as the stage numbers their box.
    let place_of =
        move |key: u64| rows.with(|r| r.iter().position(|row| row.key == key).map_or(0, |i| i + 1));
    let key_at = move |person: usize| rows.with_untracked(|r| r.get(person).map(|row| row.key));

    let save = move |e: SubmitEvent| {
        e.prevent_default();
        if busy.get_untracked() {
            return;
        }
        faces.drawing.set(None);
        let drafts: Vec<Draft> = rows.with_untracked(|r| r.iter().map(Row::draft).collect());
        let update = match figures_update(&drafts) {
            Ok(update) => update,
            Err(refused) => {
                let whose =
                    |person: usize| untrack(|| key_at(person).map(label_of)).unwrap_or_default();
                let (mark, text) = match &refused {
                    PeopleError::Tag { person, field, bad } => {
                        (key_at(*person).map(|k| (k, *field)), bad_tag_text(bad))
                    }
                    PeopleError::TooManyTags { person } => {
                        let text = t!(
                            "people-error-many-tags",
                            person = whose(*person),
                            max = MAX_FIGURE_TAGS
                        );
                        (None, text)
                    }
                    PeopleError::BadFace { person } => {
                        (None, t!("people-error-face", person = whose(*person)))
                    }
                    PeopleError::TooMany => (None, t!("people-error-many", max = MAX_FIGURES)),
                };
                if let Some((key, field)) = mark {
                    focus_box(box_id(key, field));
                }
                invalid.set(mark);
                problem.set(Some(text));
                return;
            }
        };
        busy.set(true);
        error.set(None);
        let sent = update.figures.clone();
        spawn_local(async move {
            let result =
                api::call(move |c| async move { c.set_figures(id, update.figures).await }).await;
            busy.try_set(false);
            match result {
                Ok(saved) => {
                    let filled = filled_in(&sent, &saved.figures);
                    FILLED.with_borrow_mut(|f| *f = Some((id, filled)));
                    // Gone with its card if the page moved on meanwhile.
                    on_saved.try_run(saved);
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    };
    // Mended, perhaps: saving says again.
    let typed = move |_| {
        if invalid.with_untracked(Option::is_some) || problem.with_untracked(Option::is_some) {
            invalid.set(None);
            problem.set(None);
        }
    };

    let add = move |_| {
        let Some(row) = make(Draft::default(), None, None) else {
            return;
        };
        let key = row.key;
        let mut added = false;
        rows.update(|r| {
            if r.len() < MAX_FIGURES {
                r.push(row);
                added = true;
            }
        });
        if added {
            focus_box(box_id(key, Field::Character));
        }
    };
    let remove = Callback::new(move |key: u64| {
        rows.update(|r| r.retain(|row| row.key != key));
        if faces.drawing.get_untracked() == Some(key) {
            faces.drawing.set(None);
        }
        if faces.lit.get_untracked() == Some(key) {
            faces.lit.set(None);
        }
        if invalid.with_untracked(|i| i.is_some_and(|(k, _)| k == key)) {
            invalid.set(None);
            problem.set(None);
        }
    });

    // The AI's people take the place of those typed. Over changes not yet
    // saved, a first press asks and a second replaces them.
    let replacing = RwSignal::new(false);
    let suggest = move |e: MouseEvent| {
        if asking.get_untracked() {
            return;
        }
        if replacing.get_untracked() {
            // The second click of a double click is still the first press.
            if e.detail() > 1 {
                return;
            }
            replacing.set(false);
        } else if changed.get_untracked() {
            replacing.set(true);
            return;
        }
        asking.set(true);
        error.set(None);
        none_found.set(false);
        spawn_local(async move {
            let result = api::call(move |c| async move { c.suggest_tags(id).await }).await;
            asking.try_set(false);
            match result {
                Ok(TagSuggestions { figures, .. }) if figures.is_empty() => {
                    none_found.try_set(true);
                }
                Ok(TagSuggestions { figures, .. }) => {
                    // Cancelled while it asked: no editor to fill.
                    let Some(found) = figures
                        .iter()
                        .map(|f| make(Draft::of(f), None, None))
                        .collect::<Option<Vec<Row>>>()
                    else {
                        return;
                    };
                    invalid.try_set(None);
                    problem.try_set(None);
                    faces.reset();
                    rows.try_set(found);
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    };

    view! {
        <form class="card edit people-edit" id="people" on:submit=save on:input=typed>
            <h2 class="card-title">{move || t!("people-title")}</h2>
            <p class="muted hint">{move || t!("people-hint")}</p>
            <For
                each=move || rows.get()
                key=|row| row.key
                children=move |row: Row| {
                    let key = row.key;
                    view! {
                        <PersonFields
                            row=row
                            label=Signal::derive(move || label_of(key))
                            number=Signal::derive(move || place_of(key))
                            invalid=Signal::derive(move || {
                                invalid.get().filter(|(k, _)| *k == key).map(|(_, f)| f)
                            })
                            faces=faces
                            picture=picture
                            on_remove=remove
                        />
                    }
                }
            />
            // Room here for finding the people in the picture, once the
            // server can.
            <div class="people-tools">
                <button
                    type="button"
                    class="btn tonal"
                    disabled=move || rows.with(|r| r.len() >= MAX_FIGURES)
                    on:click=add
                >
                    <Ico icon=Icon::Add />
                    {move || t!("people-add")}
                </button>
                <Show when=ai>
                    <button
                        type="button"
                        class=move || if replacing.get() { "btn danger" } else { "btn text" }
                        disabled=move || asking.get()
                        on:click=suggest
                        on:blur=move |_| replacing.set(false)
                    >
                        {move || {
                            if asking.get() {
                                t!("ai-suggesting")
                            } else if replacing.get() {
                                t!("ai-suggest-people-replace")
                            } else {
                                t!("ai-suggest-people")
                            }
                        }}
                    </button>
                </Show>
            </div>
            <Show when=move || none_found.get()>
                <p class="muted hint">{move || t!("ai-no-people")}</p>
            </Show>
            {move || problem.get().map(|m| view! { <p class="field-error break" role="alert">{m}</p> })}
            {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
            // Beside the two buttons it's about.
            <Show when=move || kept.get()>
                <div class="banner info" role="status">
                    <span>{move || t!("people-turned-off")}</span>
                </div>
            </Show>
            <div class="row-end">
                <button class="btn text" type="button" on:click=move |_| on_cancel.run(())>
                    {move || t!("cancel")}
                </button>
                <button class="btn" type="submit" disabled=move || busy.get()>
                    <Ico icon=Icon::Check />
                    {move || t!("save")}
                </button>
            </div>
        </form>
    }
}

/// One person's boxes: their face, who they are and their franchise, their
/// kind and whom they dress as, and a tag box for each slot, clothing first.
/// The face is marked on the picture: its disc and button start a drag
/// there, and show the box once it's made.
#[component]
fn PersonFields(
    row: Row,
    #[prop(into)] label: Signal<String>,
    /// Their place in the list, the number on their box.
    #[prop(into)]
    number: Signal<usize>,
    /// The box that holds what can't be saved, if it's one of theirs.
    #[prop(into)]
    invalid: Signal<Option<Field>>,
    faces: Faces,
    picture: Picture,
    on_remove: Callback<u64>,
) -> impl IntoView {
    let key = row.key;
    let kind = RwSignal::from(row.kind.clone());
    let face = RwSignal::from(row.face.clone());
    let marked = move |field: Field| Signal::derive(move || invalid.get() == Some(field));
    let named = |field: Field,
                 text: &ArcRwSignal<String>,
                 class: &'static str,
                 title: fn() -> String| {
        let id = box_id(key, field);
        view! {
            <div class=format!("field {class}")>
                <label for=id.clone()>{title}</label>
                <TagBox id=id text=RwSignal::from(text.clone()) one_line=true invalid=marked(field) />
            </div>
        }
    };
    let character = named(
        Field::Character,
        &row.character,
        "named cat-character",
        || t!("people-character"),
    );
    let cosplay = named(Field::Cosplay, &row.cosplay, "named cat-character", || {
        t!("people-cosplay")
    });
    // The franchise, marked while it's still the one the server filled in.
    let franchise = {
        let id = box_id(key, Field::Franchise);
        let text = RwSignal::from(row.franchise.clone());
        let filled = row.filled.clone();
        let auto = Signal::derive(move || {
            filled
                .as_deref()
                .is_some_and(|f| text.with(|t| t.trim() == f))
        });
        view! {
            <div class="field named cat-copyright" class:filled=auto>
                <label for=id.clone()>
                    {move || t!("people-franchise")}
                    <Show when=move || auto.get()>
                        <span class="filled-note" title=move || t!("people-filled-title")>
                            {move || t!("people-filled")}
                        </span>
                    </Show>
                </label>
                <TagBox id=id text=text one_line=true invalid=marked(Field::Franchise) />
            </div>
        }
    };
    let groups = slot_groups()
        .into_iter()
        .map(|(group, slots)| {
            let fields = slots
                .into_iter()
                .filter_map(|slot| {
                    let text = RwSignal::from(row.slots.get(&slot)?.clone());
                    let id = box_id(key, Field::Slot(slot));
                    Some(view! {
                        <div class="field">
                            <label for=id.clone()>{move || slot_label(slot)}</label>
                            <TagBox
                                id=id
                                text=text
                                one_line=true
                                invalid=marked(Field::Slot(slot))
                            />
                        </div>
                    })
                })
                .collect_view();
            view! {
                <h3 class="slot-group">{move || group_label(group)}</h3>
                <div class="slot-grid">{fields}</div>
            }
        })
        .collect_view();

    // The face: drawn on the picture, then shown there and on the disc.
    let saved = row.saved.clone();
    let crop = Signal::derive(move || {
        let (was, url) = saved.clone()?;
        (face.get() == Some(was)).then_some(url)
    });
    let marking = Signal::derive(move || faces.drawing.get() == Some(key));
    let start_marking = move || {
        faces.lit.set(Some(key));
        faces.drawing.set(Some(key));
        reveal_stage();
    };
    let press_disc = move |_| {
        if !picture.boxes.get_untracked() {
            return;
        }
        if marking.get_untracked() {
            faces.drawing.set(None);
        } else if face.with_untracked(Option::is_some) {
            faces.lit.set(Some(key));
            reveal_stage();
        } else {
            start_marking();
        }
    };
    let face_tool = move || {
        if !picture.boxes.get() {
            return None;
        }
        Some(if marking.get() {
            view! {
                <button type="button" class="btn text small face-tool" on:click=move |_| faces.drawing.set(None)>
                    {move || t!("cancel")}
                </button>
            }
            .into_any()
        } else if face.with(Option::is_some) {
            view! {
                <button
                    type="button"
                    class="btn text small face-tool"
                    aria-label=move || t!("people-face-remove-named", person = label.get())
                    on:click=move |_| face.set(None)
                >
                    <Ico icon=Icon::Close size=18 />
                    {move || t!("people-face-remove")}
                </button>
            }
            .into_any()
        } else {
            view! {
                <button
                    type="button"
                    class="btn tonal small face-tool"
                    aria-label=move || t!("people-face-mark-named", person = label.get())
                    on:click=move |_| start_marking()
                >
                    <Ico icon=Icon::Face size=18 />
                    {move || t!("people-face-mark")}
                </button>
            }
            .into_any()
        })
    };
    let disc_label = move || {
        if marking.get() {
            t!("cancel")
        } else if face.with(Option::is_some) {
            t!("people-face-show", person = label.get())
        } else {
            t!("people-face-mark-named", person = label.get())
        }
    };
    view! {
        <fieldset
            class="figure-edit"
            class:lit=move || faces.lit.get() == Some(key)
            on:focusin=move |_| faces.lit.set(Some(key))
        >
            <legend>{move || label.get()}</legend>
            <div class="figure-row">
                <div class="figure-face">
                    <button
                        type="button"
                        class="face-press"
                        disabled=move || !picture.boxes.get()
                        aria-label=disc_label
                        on:click=press_disc
                    >
                        <FaceDisc
                            face=face
                            crop=crop
                            picture=picture
                            number=number
                            marking=marking
                        />
                    </button>
                    {face_tool}
                    <button
                        type="button"
                        class="icon-btn danger"
                        title=move || t!("people-remove")
                        aria-label=move || t!("people-remove-named", person = label.get())
                        on:click=move |_| on_remove.run(key)
                    >
                        <Ico icon=Icon::Delete />
                    </button>
                </div>
                {character}
                {franchise}
                <div class="field">
                    <span>{move || t!("people-kind")}</span>
                    <Select
                        class="boxed"
                        value=Signal::derive(move || kind.get().as_str().to_owned())
                        options=Signal::derive(|| {
                            FigureKind::ALL
                                .into_iter()
                                .map(|k| Choice::new(k.as_str(), figure_kind_label(k)))
                                .collect::<Vec<_>>()
                        })
                        on_change=Callback::new(move |v: String| {
                            if let Some(k) = FigureKind::parse(&v) {
                                kind.set(k);
                            }
                        })
                        label=Signal::derive(|| t!("people-kind"))
                    />
                </div>
                {cosplay}
            </div>
            {groups}
        </fieldset>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn figure(kind: FigureKind, character: Option<&str>, cosplay: Option<&str>) -> Figure {
        Figure {
            kind,
            character: character.map(str::to_owned),
            cosplay: cosplay.map(str::to_owned),
            ..Figure::default()
        }
    }

    #[test]
    fn a_person_is_named_by_kind_and_number() {
        assert_eq!(person_label(FigureKind::Girl, 0), "Girl");
        assert_eq!(person_label(FigureKind::Boy, 2), "Boy 2");
        assert_eq!(person_label(FigureKind::Other, 1), "Other 1");
    }

    #[test]
    fn the_summary_says_who_and_whom_they_dress_as() {
        let girl = |c, p| person_summary(&figure(FigureKind::Girl, c, p), 2);
        assert_eq!(girl(None, None), "Girl 2");
        assert_eq!(girl(Some("hatsune_miku"), None), "Girl 2: hatsune_miku");
        assert_eq!(girl(None, Some("saber")), "Girl 2, cosplaying saber");
        assert_eq!(girl(Some("rin"), Some("saber")), "Girl 2: rin as saber");
        let boy = figure(FigureKind::Boy, Some("link"), None);
        assert_eq!(person_summary(&boy, 0), "Boy: link");
    }

    #[test]
    fn every_slot_and_group_has_its_words() {
        for slot in Slot::ALL {
            assert!(!slot_label(slot).starts_with("slot-"), "{slot:?}");
        }
        for (group, _) in slot_groups() {
            assert!(!group_label(group).starts_with("people-"), "{group:?}");
        }
        assert_eq!(box_id(3, Field::Slot(Slot::Hair)), "person-3-hair");
        assert_eq!(box_id(3, Field::Cosplay), "person-3-cosplay");
        assert_eq!(box_id(3, Field::Franchise), "person-3-franchise");
    }

    #[test]
    fn a_franchise_the_server_filled_in_is_told_while_it_stays() {
        FILLED.with_borrow_mut(|f| *f = Some((7, vec![None, Some("fate".into())])));
        assert_eq!(filled(7, 1, Some("fate")).as_deref(), Some("fate"));
        // Changed since, another person or another post.
        assert_eq!(filled(7, 1, Some("fate_zero")), None);
        assert_eq!(filled(7, 0, Some("fate")), None);
        assert_eq!(filled(8, 1, Some("fate")), None);
        assert_eq!(filled(7, 1, None), None);
    }

    #[test]
    fn the_card_and_its_editor_send_the_franchise_and_face_and_light_the_stage() {
        let source = include_str!("people.rs");
        let (page, _) = source.split_once("#[cfg(test)]").unwrap_or_default();
        // Each row tracks both, so they count as changes and go with the rest.
        assert!(page.contains("self.franchise.track();"));
        assert!(page.contains("self.face.track();"));
        assert!(page.contains("face: self.face.get_untracked(),"));
        // Pointing at someone outlines their face.
        assert!(page.contains("on:mouseenter=move |_| light()"));
        assert!(page.contains("on:focusin=move |_| faces.lit.set(Some(key))"));
        assert!(page.contains("faces.marks.set(marks);"));
    }
}
