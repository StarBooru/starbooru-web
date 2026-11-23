//! The search box's rules that need no browser: the word at the caret, in
//! the search and in a tag box, and what completing it does, the answers kept
//! for it, which answer is still wanted, the keys of its list, the empty
//! box's typing demo, and the words of the correction banner.

use chrono::{DateTime, Utc};
use starbooru_core::{RemoteTag, Tag, TagCategory, TagCorrection, split_category};

// ---------- the word at the caret ----------

/// The tag being typed where the caret is. `start..end` is the whole tag in
/// the search, in bytes, without what leads it (`-`, `{`) or a `}` closing
/// a group; `typed` is its part before the caret, which the suggestions
/// start with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Word {
    pub start: usize,
    pub end: usize,
    pub typed: String,
}

/// Where the word around `caret` starts and ends in `value`, in bytes: words
/// are what's between spaces and line breaks.
fn around(value: &str, caret: usize) -> Option<(usize, usize)> {
    let before = value.get(..caret)?;
    let after = value.get(caret..)?;
    let start = before
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_whitespace())
        .map_or(0, |(i, c)| i + c.len_utf8());
    let end = caret + after.find(char::is_whitespace).unwrap_or(after.len());
    Some((start, end))
}

/// The tag under the caret, `caret` bytes into `value`, if there is one to
/// complete. A `key:value` term, a word in the quotes of `text:"…"` and the
/// space after a word have none.
pub fn word_at(value: &str, caret: usize) -> Option<Word> {
    let (start, end) = around(value, caret)?;
    if in_quotes(value.get(..start)?) {
        return None;
    }
    let word = value.get(start..end)?;
    let lead = word.len() - word.trim_start_matches(['-', '{']).len();
    let body = word.trim_end_matches('}').len().max(lead);
    let (tag_start, tag_end) = (start + lead, start + body);
    // In what leads the tag, or past the brace that closes its group.
    if !(tag_start..=tag_end).contains(&caret) {
        return None;
    }
    let tag = value.get(tag_start..tag_end)?;
    let typed = value.get(tag_start..caret)?;
    if typed.is_empty() || tag.contains([':', '"', '{', '}']) {
        return None;
    }
    Some(Word {
        start: tag_start,
        end: tag_end,
        typed: typed.to_owned(),
    })
}

/// Whether `text` ends inside the quotes of a `text:"…"` term, as
/// `starbooru_core`'s parser splits a search: the quote closes at a word
/// that ends with `"`.
fn in_quotes(text: &str) -> bool {
    let mut open = false;
    for word in text.split_whitespace() {
        if open {
            open = !word.ends_with('"');
            continue;
        }
        let lower = word.to_ascii_lowercase();
        let body = lower.strip_prefix('-').unwrap_or(&lower);
        if let Some(rest) = body.strip_prefix("text:\"") {
            open = rest.is_empty() || !rest.ends_with('"');
        }
    }
    open
}

/// The tag under the caret in a tag box, where every word is a tag. A
/// category prefix (`artist:`) stays in front and the rest is completed; a
/// word with any other colon in it has nothing to complete.
pub fn tag_word_at(value: &str, caret: usize) -> Option<Word> {
    let (start, end) = around(value, caret)?;
    let word = value.get(start..end)?;
    let (_, tag) = split_category(word);
    let tag_start = start + (word.len() - tag.len());
    // In the prefix itself, nothing is typed yet.
    if caret < tag_start || tag.contains(':') {
        return None;
    }
    let typed = value.get(tag_start..caret)?;
    (!typed.is_empty()).then(|| Word {
        start: tag_start,
        end,
        typed: typed.to_owned(),
    })
}

/// `value` with `word` replaced by `tag`, and where the caret goes: after a
/// space that ends the tag, or before the `}` that closes its group. `None`
/// when `word` isn't in `value` any more.
pub fn complete(value: &str, word: &Word, tag: &str) -> Option<(String, usize)> {
    let head = value.get(..word.start)?;
    let tail = value.get(word.end..)?;
    let mut out = String::with_capacity(value.len() + tag.len() + 1);
    out.push_str(head);
    out.push_str(tag);
    let mut caret = out.len();
    match tail.chars().next() {
        None => {
            out.push(' ');
            caret += 1;
        }
        Some(c) if c.is_whitespace() => caret += c.len_utf8(),
        Some(_) => {}
    }
    out.push_str(tail);
    Some((out, caret))
}

/// The byte offset in `value` of a caret `units` UTF-16 code units in, as the
/// browser counts them. Past the end, the end.
pub fn byte_index(value: &str, units: usize) -> usize {
    let mut counted = 0;
    for (i, c) in value.char_indices() {
        if counted >= units {
            return i;
        }
        counted += c.len_utf16();
    }
    value.len()
}

/// The browser's caret for `byte` bytes into `value`: UTF-16 code units.
pub fn utf16_index(value: &str, byte: usize) -> usize {
    let upto = value.get(..byte.min(value.len())).unwrap_or(value);
    upto.encode_utf16().count()
}

// ---------- suggestions ----------

/// How many suggestions the list shows.
pub const SUGGESTIONS: u32 = 8;

/// How long the typing has to pause before the server is asked, in
/// milliseconds. A word typed in one go then costs one request.
pub const TYPING_PAUSE_MS: i32 = 150;
/// The same for another booru's tags, which the server asks that site for:
/// longer, as each site only takes so many requests.
pub const REMOTE_PAUSE_MS: i32 = 250;

/// How many answers are kept, and for how long: tags and their counts change.
const KEPT_ANSWERS: usize = 48;
const KEPT_SECONDS: i64 = 60;

/// Recent suggestions by what was typed, so taking a letter back or typing a
/// word again shows them at once and asks nothing.
#[derive(Debug)]
pub struct Answers<T> {
    /// Whose they are: another account sees other tags.
    owner: String,
    /// The latest last.
    answers: Vec<(String, DateTime<Utc>, Vec<T>)>,
}

impl<T> Default for Answers<T> {
    fn default() -> Self {
        Self {
            owner: String::new(),
            answers: Vec::new(),
        }
    }
}

impl<T: Clone> Answers<T> {
    pub fn get(&self, owner: &str, typed: &str, now: DateTime<Utc>) -> Option<Vec<T>> {
        if self.owner != owner {
            return None;
        }
        self.answers
            .iter()
            .find(|(was, at, _)| {
                was == typed && (0..KEPT_SECONDS).contains(&(now - *at).num_seconds())
            })
            .map(|(.., found)| found.clone())
    }

    pub fn put(&mut self, owner: &str, typed: String, found: Vec<T>, now: DateTime<Utc>) {
        if self.owner != owner {
            owner.clone_into(&mut self.owner);
            self.answers.clear();
        }
        self.answers.retain(|(was, ..)| *was != typed);
        self.answers.push((typed, now, found));
        let extra = self.answers.len().saturating_sub(KEPT_ANSWERS);
        self.answers.drain(..extra);
    }
}

/// Numbers the requests, so an answer that comes after a newer question is
/// dropped.
#[derive(Debug, Default)]
pub struct Latest(u64);

impl Latest {
    /// A new question, which makes every earlier one stale.
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(1);
        self.0
    }

    /// Whether question `n` is still the latest.
    pub fn is(&self, n: u64) -> bool {
        self.0 == n
    }
}

/// A tag offered under the search box.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hint {
    pub name: String,
    /// The class that colours it by category; none for a general tag.
    pub class: &'static str,
    /// How many posts carry it, when that is known.
    pub count: Option<u64>,
}

impl From<Tag> for Hint {
    fn from(tag: Tag) -> Self {
        Self {
            class: category_class(tag.category),
            count: u64::try_from(tag.post_count).ok(),
            name: tag.name,
        }
    }
}

/// Another booru's tag, as that site writes it.
impl From<RemoteTag> for Hint {
    fn from(tag: RemoteTag) -> Self {
        Self {
            class: category_class(tag.category),
            count: tag.count,
            name: tag.name,
        }
    }
}

/// The class that colours a tag's name by its category; none for a general
/// tag.
pub fn category_class(category: TagCategory) -> &'static str {
    match category {
        TagCategory::General => "",
        TagCategory::Artist => "cat-artist",
        TagCategory::Character => "cat-character",
        TagCategory::Copyright => "cat-copyright",
        TagCategory::Meta => "cat-meta",
    }
}

/// A count in few characters, for sites with tags on millions of posts:
/// `1234` → `1.2k`, `4217021` → `4.2M`.
pub fn short_count(n: u64) -> String {
    let short = |unit: u64, letter: &str| {
        let (whole, tenth) = (n / unit, (n % unit).saturating_mul(10) / unit);
        if whole >= 100 {
            format!("{whole}{letter}")
        } else {
            format!("{whole}.{tenth}{letter}")
        }
    };
    match n {
        ..1_000 => n.to_string(),
        1_000..1_000_000 => short(1_000, "k"),
        _ => short(1_000_000, "M"),
    }
}

/// What the list still shows while the server is asked about a longer
/// word: the hints that start with it.
pub fn still_fitting(hints: &mut Vec<Hint>, typed: &str) {
    let typed = typed.to_lowercase();
    hints.retain(|h| h.name.starts_with(&typed));
}

/// What a key does in the search box while it has suggestions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListKey {
    /// Highlight this suggestion.
    Move(usize),
    /// Put this suggestion in place of the word.
    Accept(usize),
    Close,
    /// Not the list's: the box or the page has it.
    Pass,
}

/// Key `key` (as `KeyboardEvent.key` names it), with the list `open`, `count`
/// suggestions and suggestion `active` highlighted. The arrows go round;
/// Enter and Tab take the highlighted one, and without one Enter searches.
pub fn list_key(key: &str, open: bool, active: Option<usize>, count: usize) -> ListKey {
    let Some(last) = count.checked_sub(1).filter(|_| open) else {
        return ListKey::Pass;
    };
    let active = active.filter(|i| *i <= last);
    match key {
        "ArrowDown" => ListKey::Move(active.map_or(0, |i| if i >= last { 0 } else { i + 1 })),
        "ArrowUp" => ListKey::Move(active.map_or(last, |i| i.checked_sub(1).unwrap_or(last))),
        "Enter" | "Tab" => active.map_or(ListKey::Pass, ListKey::Accept),
        "Escape" => ListKey::Close,
        _ => ListKey::Pass,
    }
}

/// The correction banner's list: `fractl → fractal, blu → blue`.
pub fn corrections(list: &[TagCorrection]) -> String {
    list.iter()
        .map(|c| format!("{} → {}", c.from, c.to))
        .collect::<Vec<_>>()
        .join(", ")
}

// ---------- the typing demo ----------

/// The demo's line split in two: the part of the example tag `tag` written
/// in the quiet colour, and the rest, which is what the example teaches.
pub fn demo_split<'a>(tag: &str, text: &'a str) -> (&'a str, &'a str) {
    let plain = tag.starts_with(text) || text.starts_with(tag);
    if !plain {
        return ("", text);
    }
    let quiet = tag.len().min(text.len());
    match (text.get(..quiet), text.get(quiet..)) {
        (Some(base), Some(term)) => (base, term),
        _ => ("", text),
    }
}

/// How many characters `a` and `b` start with in common: the demo rubs out
/// back to there before it types the next example.
pub fn common_prefix(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count()
}

/// How long the demo pauses after typing character `c`, given a random
/// number in `0..1`: a beat longer after a space or a colon, as a person
/// thinks of the next part.
pub fn key_delay(c: char, random: f64) -> u32 {
    let thought = if c.is_whitespace() || c == ':' {
        150
    } else {
        0
    };
    // `as` saturates: a NaN or a number out of range stays in bounds.
    let jitter = (random.clamp(0.0, 1.0) * 70.0) as u32;
    40 + jitter + thought
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    /// The word at `|` in `marked`.
    fn at(marked: &str) -> Option<String> {
        let caret = marked.find('|').unwrap();
        let value = marked.replace('|', "");
        word_at(&value, caret).map(|w| w.typed)
    }

    /// Accepting `tag` at `|` in `marked`, with the caret shown as `|`.
    fn accept(marked: &str, tag: &str) -> String {
        let caret = marked.find('|').unwrap();
        let value = marked.replace('|', "");
        let word = word_at(&value, caret).unwrap();
        let (mut out, at) = complete(&value, &word, tag).unwrap();
        out.insert(at, '|');
        out
    }

    #[test]
    fn the_word_at_the_end_is_completed() {
        assert_eq!(at("fra|").as_deref(), Some("fra"));
        assert_eq!(at("blue_sky fra|").as_deref(), Some("fra"));
        assert_eq!(accept("fra|", "fractal"), "fractal |");
        assert_eq!(accept("blue_sky fra|", "fractal"), "blue_sky fractal |");
        for nothing in ["|", "   |", "fractal |", "-|", "{|", "-{|"] {
            assert_eq!(at(nothing), None, "{nothing}");
        }
    }

    #[test]
    fn a_minus_and_a_group_stay_in_front_of_the_tag() {
        assert_eq!(at("night -clo|").as_deref(), Some("clo"));
        assert_eq!(accept("-fra|", "fractal"), "-fractal |");
        assert_eq!(at("{girl boo|").as_deref(), Some("boo"));
        assert_eq!(at("-{bo|").as_deref(), Some("bo"));
        assert_eq!(accept("-{bo|", "boots"), "-{boots |");
        // Inside a closed group the brace stays, and the caret before it.
        assert_eq!(at("{girl boo|}").as_deref(), Some("boo"));
        assert_eq!(accept("{girl boo|}", "boots"), "{girl boots|}");
        assert_eq!(at("{girl boots}|"), None);
        // On the lead itself there's nothing typed yet.
        assert_eq!(at("|-fra"), None);
        assert_eq!(at("-|fra"), None);
    }

    #[test]
    fn filters_and_quoted_text_get_no_suggestions() {
        for term in [
            "rating:s|",
            "order:sc|",
            "-rating:e|",
            "width:>19|",
            "artist:lav|",
            "text:\"no par|",
            "text:\"no par|king\"",
            "cat text:hel|",
        ] {
            assert_eq!(at(term), None, "{term}");
        }
        // Words after a closed quote are tags again; inside an open one, text.
        assert_eq!(at("text:\"no parking\" fra|").as_deref(), Some("fra"));
        assert_eq!(at("text:\"one\" fra|").as_deref(), Some("fra"));
        assert_eq!(at("-text:\"no parking fra|"), None);
        assert_eq!(at("TEXT:\"no fra|"), None);
    }

    #[test]
    fn the_caret_in_the_middle_completes_its_own_word() {
        assert_eq!(at("fra| blue_sky").as_deref(), Some("fra"));
        assert_eq!(accept("fra| blue_sky", "fractal"), "fractal |blue_sky");
        // Halfway into a word: what's before the caret is looked up, and the
        // whole word is replaced.
        assert_eq!(at("blue_sky fra|ctl order:score").as_deref(), Some("fra"));
        assert_eq!(
            accept("blue_sky fra|ctl order:score", "fractal"),
            "blue_sky fractal |order:score"
        );
        assert_eq!(at("|fra"), None);
        assert_eq!(accept("f|ra  cat", "fractal"), "fractal | cat");
    }

    #[test]
    fn letters_outside_ascii_are_counted_as_the_browser_counts_them() {
        let value = "猫 😀 fra";
        // The browser says the caret is 7 units in: 猫 is one, 😀 two.
        assert_eq!(value.encode_utf16().count(), 8);
        let caret = byte_index(value, 7);
        assert_eq!(value.get(caret..), Some("a"));
        assert_eq!(utf16_index(value, caret), 7);
        assert_eq!(utf16_index(value, value.len()), 8);
        assert_eq!(byte_index(value, 999), value.len());
        assert_eq!(utf16_index(value, 999), 8);
        assert_eq!(
            word_at(value, value.len()).map(|w| w.typed).as_deref(),
            Some("fra")
        );
        // A word after a wide space, and a caret off a character's boundary.
        assert_eq!(
            word_at("猫\u{3000}fr", "猫\u{3000}fr".len())
                .map(|w| w.typed)
                .as_deref(),
            Some("fr")
        );
        assert_eq!(word_at("猫", 1), None);
        assert_eq!(word_at("cat", 9), None);
    }

    /// The tag box's word at `|` in `marked`, as `(typed, start)`.
    fn in_box(marked: &str) -> Option<(String, usize)> {
        let caret = marked.find('|').unwrap();
        let value = marked.replace('|', "");
        tag_word_at(&value, caret).map(|w| (w.typed, w.start))
    }

    /// Accepting `tag` at `|` in a tag box.
    fn accept_in_box(marked: &str, tag: &str) -> String {
        let caret = marked.find('|').unwrap();
        let value = marked.replace('|', "");
        let word = tag_word_at(&value, caret).unwrap();
        let (mut out, at) = complete(&value, &word, tag).unwrap();
        out.insert(at, '|');
        out
    }

    #[test]
    fn a_tag_box_completes_each_word_and_keeps_a_category_prefix() {
        assert_eq!(in_box("blue_sk|"), Some(("blue_sk".into(), 0)));
        assert_eq!(in_box("cat blu| dog"), Some(("blu".into(), 4)));
        // Lines are words apart too.
        assert_eq!(in_box("cat\nchar:ri|"), Some(("ri".into(), 9)));
        assert_eq!(in_box("artist:mi|"), Some(("mi".into(), 7)));
        assert_eq!(accept_in_box("artist:mi|", "mika"), "artist:mika |");
        assert_eq!(
            accept_in_box("cat blu| dog", "blue_sky"),
            "cat blue_sky |dog"
        );
        assert_eq!(
            accept_in_box("cat\nblu|\ndog", "blue_sky"),
            "cat\nblue_sky\n|dog"
        );
        // Halfway into a word, as in the search.
        assert_eq!(in_box("cat bl|u dog"), Some(("bl".into(), 4)));
        // A minus or a brace is part of the tag here: there are no groups.
        assert_eq!(in_box("{fra|"), Some(("{fra".into(), 0)));
        for nothing in [
            "|",
            "cat |",
            "|cat dog",
            "cat | dog",
            "artist:|",
            "arti|st:mi",
            "rating:s|",
            "a:b:c|",
        ] {
            assert_eq!(in_box(nothing), None, "{nothing}");
        }
    }

    #[test]
    fn a_word_that_moved_is_not_replaced() {
        let word = Word {
            start: 4,
            end: 20,
            typed: "x".into(),
        };
        assert_eq!(complete("cat", &word, "xylophone"), None);
    }

    fn tag(name: &str, category: TagCategory, count: i32) -> Tag {
        Tag {
            id: 1,
            name: name.into(),
            category,
            post_count: count,
        }
    }

    #[test]
    fn a_hint_shows_its_category_and_count() {
        assert_eq!(
            Hint::from(tag("lavfi", TagCategory::Artist, 35)),
            Hint {
                name: "lavfi".into(),
                class: "cat-artist",
                count: Some(35)
            }
        );
        assert_eq!(
            Hint::from(tag("fractal", TagCategory::General, 8)).class,
            ""
        );
        assert_eq!(Hint::from(tag("x", TagCategory::Meta, -1)).count, None);
        let remote = |category, count| RemoteTag {
            name: "hatsune_miku".into(),
            category,
            count,
        };
        assert_eq!(
            Hint::from(remote(TagCategory::Character, Some(4_217_021))),
            Hint {
                name: "hatsune_miku".into(),
                class: "cat-character",
                count: Some(4_217_021)
            }
        );
        assert_eq!(Hint::from(remote(TagCategory::General, None)).count, None);
        // The colours are in the stylesheet, for the list under the box.
        let css = include_str!("../../style/shell.css");
        for class in ["cat-artist", "cat-character", "cat-copyright", "cat-meta"] {
            assert!(css.contains(&format!(
                ".suggestions .{class} {{ color: var(--{class}); }}"
            )));
        }
    }

    #[test]
    fn counts_stay_short() {
        assert_eq!(short_count(0), "0");
        assert_eq!(short_count(999), "999");
        assert_eq!(short_count(1_000), "1.0k");
        assert_eq!(short_count(1_234), "1.2k");
        assert_eq!(short_count(123_456), "123k");
        assert_eq!(short_count(4_217_021), "4.2M");
        assert_eq!(short_count(u64::MAX), "18446744073709M");
    }

    #[test]
    fn hints_that_still_fit_stay_while_the_server_is_asked() {
        let hint = |name: &str| Hint {
            name: name.into(),
            class: "",
            count: None,
        };
        let mut hints = vec![hint("fractal"), hint("frame"), hint("fox")];
        still_fitting(&mut hints, "FRA");
        assert_eq!(hints, [hint("fractal"), hint("frame")]);
        still_fitting(&mut hints, "fract");
        assert_eq!(hints, [hint("fractal")]);
    }

    #[test]
    fn an_answer_is_kept_for_a_minute_and_for_its_account() {
        let now = Utc::now();
        let mut kept = Answers::default();
        kept.put("a|1", "blu".into(), vec!["blue_sky"], now);
        assert_eq!(kept.get("a|1", "blu", now), Some(vec!["blue_sky"]));
        assert_eq!(kept.get("a|1", "blue", now), None);
        assert_eq!(kept.get("b|2", "blu", now), None);
        let later = now + Duration::seconds(KEPT_SECONDS);
        assert_eq!(kept.get("a|1", "blu", later), None);
        // A clock set back asks again.
        assert_eq!(kept.get("a|1", "blu", now - Duration::seconds(5)), None);
        // Another account's first answer empties what the last one saw.
        kept.put("b|2", "x".into(), Vec::new(), now);
        assert_eq!(kept.get("a|1", "blu", now), None);
        assert_eq!(kept.get("b|2", "x", now), Some(Vec::new()));
    }

    #[test]
    fn old_answers_make_room() {
        let now = Utc::now();
        let mut kept = Answers::<u8>::default();
        for i in 0..KEPT_ANSWERS + 10 {
            kept.put("a|1", format!("t{i}"), Vec::new(), now);
            // Asked again: one entry, not two.
            kept.put("a|1", format!("t{i}"), Vec::new(), now);
        }
        assert_eq!(kept.answers.len(), KEPT_ANSWERS);
        assert!(kept.get("a|1", "t0", now).is_none());
        let newest = format!("t{}", KEPT_ANSWERS + 9);
        assert!(kept.get("a|1", &newest, now).is_some());
    }

    #[test]
    fn only_the_latest_question_gets_its_answer() {
        let mut latest = Latest::default();
        let first = latest.next();
        assert!(latest.is(first));
        let second = latest.next();
        assert!(!latest.is(first), "a late answer for an older word");
        assert!(latest.is(second));
        let mut wrapped = Latest(u64::MAX);
        assert!(wrapped.is(u64::MAX));
        assert_eq!(wrapped.next(), 0);
    }

    #[test]
    fn the_arrows_go_round_and_enter_takes_the_highlighted_one() {
        use ListKey::*;
        assert_eq!(list_key("ArrowDown", true, None, 3), Move(0));
        assert_eq!(list_key("ArrowDown", true, Some(0), 3), Move(1));
        assert_eq!(list_key("ArrowDown", true, Some(2), 3), Move(0));
        assert_eq!(list_key("ArrowUp", true, None, 3), Move(2));
        assert_eq!(list_key("ArrowUp", true, Some(0), 3), Move(2));
        assert_eq!(list_key("ArrowUp", true, Some(2), 3), Move(1));
        assert_eq!(list_key("Enter", true, Some(1), 3), Accept(1));
        assert_eq!(list_key("Tab", true, Some(2), 3), Accept(2));
        assert_eq!(list_key("Escape", true, None, 3), Close);
        // Nothing highlighted: Enter searches and Tab moves on.
        assert_eq!(list_key("Enter", true, None, 3), Pass);
        assert_eq!(list_key("Tab", true, None, 3), Pass);
        assert_eq!(list_key("a", true, Some(1), 3), Pass);
    }

    #[test]
    fn a_closed_or_empty_list_leaves_the_keys_alone() {
        for key in ["ArrowDown", "ArrowUp", "Enter", "Tab", "Escape"] {
            assert_eq!(list_key(key, false, Some(1), 3), ListKey::Pass, "{key}");
            assert_eq!(list_key(key, true, Some(0), 0), ListKey::Pass, "{key}");
        }
        // A highlight left from a longer list never points past the end.
        assert_eq!(list_key("Enter", true, Some(7), 3), ListKey::Pass);
        assert_eq!(list_key("ArrowDown", true, Some(7), 3), ListKey::Move(0));
    }

    #[test]
    fn the_banner_lists_each_correction() {
        let fix = |from: &str, to: &str| TagCorrection {
            from: from.into(),
            to: to.into(),
        };
        assert_eq!(corrections(&[fix("fractl", "fractal")]), "fractl → fractal");
        assert_eq!(
            corrections(&[fix("fractl", "fractal"), fix("blu_sky", "blue_sky")]),
            "fractl → fractal, blu_sky → blue_sky"
        );
        assert_eq!(corrections(&[]), "");
    }

    #[test]
    fn the_demo_colours_what_the_example_teaches() {
        let tag = "ishtar_(fate)";
        assert_eq!(demo_split(tag, "ish"), ("ish", ""));
        assert_eq!(demo_split(tag, tag), (tag, ""));
        assert_eq!(demo_split(tag, "ishtar_(fate) smile"), (tag, " smile"));
        // An example that starts some other way is all coloured.
        assert_eq!(demo_split(tag, "{ishtar"), ("", "{ishtar"));
        assert_eq!(demo_split(tag, ""), ("", ""));
    }

    #[test]
    fn the_demo_rubs_out_only_what_differs() {
        assert_eq!(common_prefix("cat rating:safe", "cat rating:q"), 11);
        assert_eq!(common_prefix("", "cat"), 0);
        assert_eq!(common_prefix("猫x", "猫y"), 1);
        assert_eq!(key_delay('a', 0.0), 40);
        assert_eq!(key_delay('a', 1.0), 110);
        assert_eq!(key_delay(' ', 0.0), 190);
        assert_eq!(key_delay(':', 0.5), 225);
        assert_eq!(key_delay('a', f64::NAN), 40);
        assert_eq!(key_delay('a', 9.0), 110);
    }
}
