//! The upload queue as plain data: what was picked, what goes on it, and how
//! far each file has got. `activity.rs` keeps one at the app's root and does
//! the sending; this says what may happen next.
//!
//! An item is a draft until Upload is pressed. A file then waits its turn,
//! is sent, and ends done, a duplicate or failed; a failed or cancelled one
//! can go again. An address goes to the server's import queue instead, and
//! leaves the list once the server has it.

use starbooru_core::{ImportJob, ImportRequest, Rating, Transfer};

use super::filter::holds_all;
use super::tags::{BadTag, parse_tags};

/// Whether an import holds every one of `words`, in its address, its
/// source or its tags.
pub fn job_matches(job: &ImportJob, words: &[String]) -> bool {
    let mut places = vec![job.url.as_str(), job.source.as_deref().unwrap_or("")];
    places.extend(job.tags.iter().map(String::as_str));
    holds_all(words, &places)
}

/// How many files are sent at once. A big one goes in chunks, several at a
/// time already.
pub const AT_ONCE: usize = 2;

#[derive(Clone, Debug, PartialEq)]
pub enum Source {
    /// `mime` is the browser's type for it, `path` where it was in a folder
    /// (empty for a file picked alone).
    File {
        name: String,
        size: u64,
        mime: String,
        path: String,
    },
    Url(String),
}

/// What goes on everything sent together, from the Upload page's bar or the
/// sheet that drops open.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Batch {
    /// Tags for every item, before each one's own.
    pub tags: String,
    pub rating: Rating,
    /// Where the files come from; addresses are their own source.
    pub source: String,
    /// Whether the AI tags the new posts. `None` leaves it to the server.
    pub autotag: Option<bool>,
}

/// What an item is sent with, fixed when Upload is pressed.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub tags: Vec<String>,
    pub rating: Rating,
    pub source: Option<String>,
    pub autotag: Option<bool>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    /// Waiting for Upload; its tags can still change.
    Draft,
    /// Waiting its turn to be sent.
    Waiting,
    Sending,
    Done(i64),
    /// Already a post: that one, when the server said which.
    Duplicate(Option<i64>),
    Failed(String),
    /// Cancelled on its way.
    Stopped,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub key: u64,
    pub source: Source,
    /// Tags for this item only, on top of the shared ones.
    pub tags: String,
    /// Its own rating; `None` goes with the rest.
    pub rating: Option<Rating>,
    pub status: Status,
    pub plan: Option<Plan>,
    /// Which time it is being sent, so a cancelled attempt that ends late
    /// changes nothing.
    pub attempt: u32,
    /// How much the server has taken, once a chunk is through.
    pub progress: Option<Transfer>,
}

impl Item {
    pub fn name(&self) -> &str {
        match &self.source {
            Source::File { name, .. } => name,
            Source::Url(url) => url,
        }
    }

    /// The file's size; an address has none.
    pub fn size(&self) -> Option<u64> {
        match &self.source {
            Source::File { size, .. } => Some(*size),
            Source::Url(_) => None,
        }
    }

    /// Sent for and not finished.
    pub fn pending(&self) -> bool {
        matches!(self.status, Status::Waiting | Status::Sending)
    }

    /// Anything but what is on its way can be taken off the list.
    pub fn removable(&self) -> bool {
        self.status != Status::Sending
    }

    pub fn retryable(&self) -> bool {
        matches!(self.status, Status::Failed(_) | Status::Stopped) && self.plan.is_some()
    }
}

/// How sending a file ended.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    Posted(i64),
    Duplicate(Option<i64>),
    Failed(String),
}

/// How far the files sent together have got.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Overall {
    pub finished: usize,
    pub files: usize,
    pub transfer: Transfer,
}

/// An address to import, from its item's plan.
pub fn import_of(url: &str, plan: &Plan) -> ImportRequest {
    ImportRequest {
        url: url.to_owned(),
        tags: plan.tags.clone(),
        rating: plan.rating,
        source: plan.source.clone(),
        autotag: plan.autotag,
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Queue {
    items: Vec<Item>,
    next: u64,
}

impl Queue {
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn get(&self, key: u64) -> Option<&Item> {
        self.items.iter().find(|i| i.key == key)
    }

    fn get_mut(&mut self, key: u64) -> Option<&mut Item> {
        self.items.iter_mut().find(|i| i.key == key)
    }

    /// Whether something added now begins a new batch: nothing waits to be
    /// sent, so what's on the list belongs to one sent already.
    pub fn starts_batch(&self) -> bool {
        !self.items.iter().any(|i| i.status == Status::Draft)
    }

    /// Adds a draft and gives its key. The first of a new batch clears away
    /// what the last one left: finished uploads and failures.
    pub fn add(&mut self, source: Source, tags: String) -> u64 {
        if self.starts_batch() {
            self.items.retain(Item::pending);
        }
        let key = self.next;
        self.next = self.next.wrapping_add(1);
        self.items.push(Item {
            key,
            source,
            tags,
            rating: None,
            status: Status::Draft,
            plan: None,
            attempt: 0,
            progress: None,
        });
        key
    }

    pub fn set_tags(&mut self, key: u64, tags: String) {
        if let Some(item) = self.get_mut(key) {
            item.tags = tags;
        }
    }

    pub fn set_rating(&mut self, key: u64, rating: Option<Rating>) {
        if let Some(item) = self.get_mut(key) {
            item.rating = rating;
        }
    }

    /// Takes an item off the list, unless it is on its way.
    pub fn remove(&mut self, key: u64) -> bool {
        let before = self.items.len();
        self.items.retain(|i| i.key != key || !i.removable());
        self.items.len() != before
    }

    /// Empties the list, keeping what is on its way.
    pub fn clear(&mut self) {
        self.items.retain(|i| i.status == Status::Sending);
    }

    /// Takes the items in `keys` off the list, but for those on their way.
    pub fn clear_only(&mut self, keys: &[u64]) {
        self.items
            .retain(|i| i.status == Status::Sending || !keys.contains(&i.key));
    }

    /// The items that hold every one of `words`, in their name, their
    /// folder or their own tags. No words: all of them.
    pub fn matching(&self, words: &[String]) -> Vec<u64> {
        self.items
            .iter()
            .filter(|i| {
                let folder = match &i.source {
                    Source::File { path, .. } => path.as_str(),
                    Source::Url(_) => "",
                };
                holds_all(words, &[i.name(), folder, i.tags.as_str()])
            })
            .map(|i| i.key)
            .collect()
    }

    /// The drafts among `keys`.
    pub fn drafts_in(&self, keys: &[u64]) -> usize {
        self.items
            .iter()
            .filter(|i| i.status == Status::Draft && keys.contains(&i.key))
            .count()
    }

    /// Sends the drafts `wanted` picks with what `batch` puts on them: files
    /// wait their turn, and the addresses come back to be imported. A draft
    /// whose tags can't be tags fails, in `explain`'s words.
    pub fn start(
        &mut self,
        batch: &Batch,
        wanted: impl Fn(u64) -> bool,
        explain: impl Fn(&BadTag) -> String,
    ) -> Vec<(u64, ImportRequest)> {
        let source = Some(batch.source.trim().to_owned()).filter(|s| !s.is_empty());
        let mut imports = Vec::new();
        for item in &mut self.items {
            if item.status != Status::Draft || !wanted(item.key) {
                continue;
            }
            let tags = match parse_tags(&format!("{} {}", batch.tags, item.tags)) {
                Ok(tags) => tags,
                Err(e) => {
                    item.status = Status::Failed(explain(&e));
                    continue;
                }
            };
            let plan = Plan {
                tags,
                rating: item.rating.unwrap_or(batch.rating),
                // An import given a source of its own is downloaded as it
                // is, not looked up on its site: an address is its own.
                source: matches!(item.source, Source::File { .. })
                    .then(|| source.clone())
                    .flatten(),
                autotag: batch.autotag,
            };
            item.status = match &item.source {
                Source::File { .. } => Status::Waiting,
                Source::Url(url) => {
                    imports.push((item.key, import_of(url, &plan)));
                    Status::Sending
                }
            };
            item.plan = Some(plan);
        }
        imports
    }

    /// Files to send now, so that no more than `limit` are on their way, as
    /// their keys and which attempt each is.
    pub fn next(&mut self, limit: usize) -> Vec<(u64, u32)> {
        let sending = self
            .items
            .iter()
            .filter(|i| i.status == Status::Sending && matches!(i.source, Source::File { .. }))
            .count();
        let mut room = limit.saturating_sub(sending);
        let mut out = Vec::new();
        for item in &mut self.items {
            if room == 0 {
                break;
            }
            if item.status == Status::Waiting && matches!(item.source, Source::File { .. }) {
                item.status = Status::Sending;
                item.attempt = item.attempt.wrapping_add(1);
                item.progress = None;
                out.push((item.key, item.attempt));
                room -= 1;
            }
        }
        out
    }

    /// How much of attempt `attempt` the server has taken.
    pub fn sent(&mut self, key: u64, attempt: u32, progress: Transfer) {
        if let Some(item) = self.get_mut(key)
            && item.status == Status::Sending
            && item.attempt == attempt
        {
            item.progress = Some(progress);
        }
    }

    /// How attempt `attempt` ended. Says whether it still counted: one
    /// cancelled, or taken off the list, changes nothing.
    pub fn finish(&mut self, key: u64, attempt: u32, outcome: Outcome) -> bool {
        let Some(item) = self.get_mut(key) else {
            return false;
        };
        if item.status != Status::Sending || item.attempt != attempt {
            return false;
        }
        item.status = match outcome {
            Outcome::Posted(id) => Status::Done(id),
            Outcome::Duplicate(id) => Status::Duplicate(id),
            Outcome::Failed(message) => Status::Failed(message),
        };
        let whole = item.size();
        if let (Some(progress), Some(size)) = (&mut item.progress, whole)
            && matches!(item.status, Status::Done(_) | Status::Duplicate(_))
        {
            progress.done = size;
        }
        true
    }

    /// Stops a file on its way or waiting its turn.
    pub fn cancel(&mut self, key: u64) -> bool {
        match self.get_mut(key) {
            Some(item) if item.pending() && matches!(item.source, Source::File { .. }) => {
                item.status = Status::Stopped;
                item.progress = None;
                true
            }
            _ => false,
        }
    }

    /// Sends a failed or cancelled item again, with what it was sent with. A
    /// file waits its turn; an address comes back to be imported.
    pub fn retry(&mut self, key: u64) -> Option<Option<ImportRequest>> {
        let item = self.get_mut(key).filter(|i| i.retryable())?;
        let plan = item.plan.clone()?;
        item.progress = None;
        match &item.source {
            Source::File { .. } => {
                item.status = Status::Waiting;
                Some(None)
            }
            Source::Url(url) => {
                let request = import_of(url, &plan);
                item.status = Status::Sending;
                Some(Some(request))
            }
        }
    }

    /// The server has these addresses to import: their jobs show them now.
    pub fn imported(&mut self, keys: &[u64]) {
        self.items
            .retain(|i| !(keys.contains(&i.key) && i.status == Status::Sending));
    }

    /// The server took none of these addresses.
    pub fn import_failed(&mut self, keys: &[u64], message: &str) {
        for item in &mut self.items {
            if keys.contains(&item.key) && item.status == Status::Sending {
                item.status = Status::Failed(message.to_owned());
            }
        }
    }

    pub fn drafts(&self) -> usize {
        self.items
            .iter()
            .filter(|i| i.status == Status::Draft)
            .count()
    }

    /// How many are sent for and not finished.
    pub fn pending(&self) -> usize {
        self.items.iter().filter(|i| i.pending()).count()
    }

    /// The files sent together so far: how many are finished, and the bytes
    /// done of all of them. `None` once none is left to go.
    pub fn overall(&self) -> Option<Overall> {
        let files: Vec<&Item> = self
            .items
            .iter()
            .filter(|i| i.plan.is_some() && i.status != Status::Stopped)
            .filter(|i| matches!(i.source, Source::File { .. }))
            .collect();
        if !files.iter().any(|i| i.pending()) {
            return None;
        }
        let (mut finished, mut total, mut done, mut speed) = (0, 0u64, 0u64, 0u64);
        for item in &files {
            let size = item.size().unwrap_or(0);
            match item.status {
                Status::Done(_) | Status::Duplicate(_) => {
                    finished += 1;
                    total = total.saturating_add(size);
                    done = done.saturating_add(size);
                }
                Status::Failed(_) => finished += 1,
                Status::Sending => {
                    total = total.saturating_add(size);
                    if let Some(p) = item.progress {
                        done = done.saturating_add(p.done.min(size));
                        speed = speed.saturating_add(p.speed);
                    }
                }
                _ => total = total.saturating_add(size),
            }
        }
        Some(Overall {
            finished,
            files: files.len(),
            transfer: Transfer {
                done,
                total: Some(total),
                speed,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::filter::words;

    fn file(name: &str, size: u64) -> Source {
        Source::File {
            name: name.into(),
            size,
            mime: String::new(),
            path: String::new(),
        }
    }

    fn batch(tags: &str) -> Batch {
        Batch {
            tags: tags.into(),
            rating: Rating::Sensitive,
            source: " https://src.example/1 ".into(),
            autotag: Some(false),
        }
    }

    fn start_all(q: &mut Queue, b: &Batch) -> Vec<(u64, ImportRequest)> {
        q.start(b, |_| true, |bad| bad.word.clone())
    }

    fn status(q: &Queue, key: u64) -> Status {
        q.get(key).map(|i| i.status.clone()).unwrap()
    }

    #[test]
    fn a_draft_waits_its_turn_then_goes_and_ends_done() {
        let mut q = Queue::default();
        let a = q.add(file("a.png", 10), "cat".into());
        assert_eq!(status(&q, a), Status::Draft);
        assert_eq!(q.drafts(), 1);
        assert!(start_all(&mut q, &batch("sky")).is_empty());
        assert_eq!(status(&q, a), Status::Waiting);
        let plan = q.get(a).and_then(|i| i.plan.clone()).unwrap();
        assert_eq!(plan.tags, ["sky", "cat"]);
        assert_eq!(plan.rating, Rating::Sensitive);
        assert_eq!(plan.source.as_deref(), Some("https://src.example/1"));
        assert_eq!(plan.autotag, Some(false));
        assert_eq!(q.next(AT_ONCE), [(a, 1)]);
        assert_eq!(status(&q, a), Status::Sending);
        assert_eq!(q.pending(), 1);
        assert!(q.finish(a, 1, Outcome::Posted(70)));
        assert_eq!(status(&q, a), Status::Done(70));
        assert_eq!(q.pending(), 0);
    }

    #[test]
    fn a_filter_finds_items_by_name_folder_or_tags_and_clears_only_those() {
        let mut q = Queue::default();
        let beach = q.add(
            Source::File {
                name: "IMG_0001.jpg".into(),
                size: 10,
                mime: String::new(),
                path: "Holidays/IMG_0001.jpg".into(),
            },
            "sea".into(),
        );
        let cat = q.add(file("cat.png", 10), String::new());
        let link = q.add(
            Source::Url("https://example.com/a.gif".into()),
            String::new(),
        );
        let all = |q: &Queue, s: &str| q.matching(&words(s));
        assert_eq!(all(&q, ""), [beach, cat, link]);
        assert_eq!(all(&q, "holidays"), [beach]);
        assert_eq!(all(&q, "SEA img"), [beach]);
        assert_eq!(all(&q, "example gif"), [link]);
        assert!(all(&q, "dog").is_empty());
        assert_eq!(q.drafts_in(&[beach, link]), 2);
        // One on its way stays, cleared or not.
        start_all(&mut q, &batch(""));
        q.next(1);
        assert_eq!(status(&q, beach), Status::Sending);
        q.clear_only(&[beach, cat]);
        assert!(q.get(beach).is_some());
        assert!(q.get(cat).is_none());
        assert!(q.get(link).is_some());
        let job: ImportJob = serde_json::from_value(serde_json::json!({
            "id": 1, "url": "https://site.example/p/9", "source": null,
            "tags": ["blue_sky"], "rating": "safe", "state": "queued",
            "post_id": null, "error": null, "created_at": "2026-11-26T00:00:00Z",
            "finished_at": null
        }))
        .unwrap();
        assert!(job_matches(&job, &words("site blue")));
        assert!(!job_matches(&job, &words("cat")));
    }

    #[test]
    fn no_more_than_the_limit_go_at_once() {
        let mut q = Queue::default();
        let keys: Vec<u64> = (0..5)
            .map(|n| q.add(file(&format!("{n}.png"), 10), String::new()))
            .collect();
        start_all(&mut q, &batch(""));
        let first = q.next(AT_ONCE);
        assert_eq!(first.len(), AT_ONCE);
        // Nothing more while both are on their way.
        assert!(q.next(AT_ONCE).is_empty());
        let (key, attempt) = first[0];
        q.finish(key, attempt, Outcome::Duplicate(Some(3)));
        let then = q.next(AT_ONCE);
        assert_eq!(then.len(), 1);
        assert_eq!(then[0].0, keys[2]);
        assert_eq!(status(&q, key), Status::Duplicate(Some(3)));
    }

    #[test]
    fn a_failed_or_cancelled_file_can_go_again() {
        let mut q = Queue::default();
        let a = q.add(file("a.png", 10), String::new());
        start_all(&mut q, &batch(""));
        let [(_, first)] = q.next(1)[..] else {
            panic!()
        };
        q.finish(a, first, Outcome::Failed("network".into()));
        assert_eq!(status(&q, a), Status::Failed("network".into()));
        assert!(matches!(q.retry(a), Some(None)));
        assert_eq!(status(&q, a), Status::Waiting);
        let [(_, second)] = q.next(1)[..] else {
            panic!()
        };
        assert_eq!(second, first + 1);
        // Cancelled on its way: the late end of that attempt changes nothing,
        // even once it is sent again.
        assert!(q.cancel(a));
        assert_eq!(status(&q, a), Status::Stopped);
        assert!(!q.finish(a, second, Outcome::Posted(1)));
        assert!(matches!(q.retry(a), Some(None)));
        let [(_, third)] = q.next(1)[..] else {
            panic!()
        };
        assert!(!q.finish(a, second, Outcome::Posted(1)));
        assert_eq!(status(&q, a), Status::Sending);
        assert!(q.finish(a, third, Outcome::Posted(2)));
        assert_eq!(status(&q, a), Status::Done(2));
        // Nothing to retry once it's done, nor a draft never sent.
        assert!(q.retry(a).is_none());
        let b = q.add(file("b.png", 1), String::new());
        assert!(q.retry(b).is_none());
        assert!(!q.cancel(b));
    }

    #[test]
    fn what_is_on_its_way_stays_on_the_list() {
        let mut q = Queue::default();
        let a = q.add(file("a.png", 10), String::new());
        let b = q.add(file("b.png", 10), String::new());
        let c = q.add(file("c.png", 10), String::new());
        start_all(&mut q, &batch(""));
        q.next(1);
        assert!(!q.remove(a));
        assert!(q.remove(b));
        q.clear();
        assert_eq!(q.items().len(), 1);
        assert!(q.get(a).is_some() && q.get(c).is_none());
        // A sending file that goes is no longer counted when it ends.
        q.cancel(a);
        assert!(q.remove(a));
        assert!(!q.finish(a, 1, Outcome::Posted(1)));
    }

    #[test]
    fn a_new_batch_clears_what_the_last_one_left() {
        let mut q = Queue::default();
        let a = q.add(file("a.png", 10), String::new());
        let b = q.add(file("b.png", 10), String::new());
        let c = q.add(file("c.png", 10), String::new());
        start_all(&mut q, &batch(""));
        q.next(1);
        q.finish(a, 1, Outcome::Posted(1));
        q.next(1);
        assert_eq!(status(&q, b), Status::Sending);
        assert!(q.starts_batch());
        let d = q.add(file("d.png", 10), String::new());
        // The finished file goes; those still to send stay.
        let keys: Vec<u64> = q.items().iter().map(|i| i.key).collect();
        assert_eq!(keys, [b, c, d]);
        // Files still being picked belong to the batch being made.
        assert!(!q.starts_batch());
        q.add(file("e.png", 10), String::new());
        assert_eq!(q.items().len(), 4);
    }

    #[test]
    fn bad_tags_fail_their_file_and_its_own_rating_wins() {
        let mut q = Queue::default();
        let a = q.add(file("a.png", 10), "-bad".into());
        let b = q.add(file("b.png", 10), "fine".into());
        q.set_rating(b, Some(Rating::Explicit));
        q.set_tags(b, "fine better".into());
        start_all(&mut q, &batch(""));
        assert_eq!(status(&q, a), Status::Failed("-bad".into()));
        let plan = q.get(b).and_then(|i| i.plan.clone()).unwrap();
        assert_eq!(plan.rating, Rating::Explicit);
        assert_eq!(plan.tags, ["fine", "better"]);
    }

    #[test]
    fn only_the_wanted_drafts_go() {
        let mut q = Queue::default();
        let a = q.add(file("a.png", 10), String::new());
        let b = q.add(file("b.png", 10), String::new());
        q.start(&batch(""), |k| k == b, |bad| bad.word.clone());
        assert_eq!(status(&q, a), Status::Draft);
        assert_eq!(status(&q, b), Status::Waiting);
    }

    #[test]
    fn an_address_goes_to_the_import_queue_as_its_own_source() {
        let mut q = Queue::default();
        let u = q.add(
            Source::Url("https://cdn.example/a.png".into()),
            "own".into(),
        );
        let imports = start_all(&mut q, &batch("shared"));
        let [(key, request)] = &imports[..] else {
            panic!()
        };
        assert_eq!(*key, u);
        assert_eq!(request.url, "https://cdn.example/a.png");
        assert_eq!(request.tags, ["shared", "own"]);
        assert_eq!(request.rating, Rating::Sensitive);
        assert_eq!(request.source, None);
        assert_eq!(request.autotag, Some(false));
        // It isn't sent as a file.
        assert!(q.next(AT_ONCE).is_empty());
        assert_eq!(q.pending(), 1);
        q.import_failed(&[u], "turned off");
        assert_eq!(status(&q, u), Status::Failed("turned off".into()));
        let again = q.retry(u).flatten().unwrap();
        assert_eq!(again.url, request.url);
        q.imported(&[u]);
        assert!(q.get(u).is_none());
    }

    #[test]
    fn progress_counts_only_the_attempt_on_its_way() {
        let mut q = Queue::default();
        let a = q.add(file("a.mp4", 100), String::new());
        start_all(&mut q, &batch(""));
        q.next(1);
        let at = |done| Transfer {
            done,
            total: Some(100),
            speed: 10,
        };
        q.sent(a, 1, at(40));
        q.sent(a, 7, at(90));
        assert_eq!(q.get(a).and_then(|i| i.progress).map(|p| p.done), Some(40));
    }

    #[test]
    fn the_whole_batch_adds_up() {
        let mut q = Queue::default();
        let a = q.add(file("a.png", 100), String::new());
        let b = q.add(file("b.mp4", 300), String::new());
        let c = q.add(file("c.png", 600), String::new());
        let d = q.add(file("d.png", 50), String::new());
        assert_eq!(q.overall(), None);
        start_all(&mut q, &batch(""));
        q.next(2);
        q.finish(a, 1, Outcome::Posted(1));
        q.sent(
            b,
            1,
            Transfer {
                done: 150,
                total: Some(300),
                speed: 40,
            },
        );
        q.cancel(d);
        let all = q.overall().unwrap();
        assert_eq!(all.finished, 1);
        // The cancelled file isn't part of it any more.
        assert_eq!(all.files, 3);
        assert_eq!(all.transfer.done, 250);
        assert_eq!(all.transfer.total, Some(1000));
        assert_eq!(all.transfer.speed, 40);
        assert_eq!(all.transfer.percent(), Some(25));
        // A failure counts as finished, and its bytes no longer do.
        q.finish(b, 1, Outcome::Failed("x".into()));
        let all = q.overall().unwrap();
        assert_eq!((all.finished, all.transfer.total), (2, Some(700)));
        q.next(2);
        q.finish(c, 1, Outcome::Posted(3));
        assert_eq!(q.overall(), None);
    }

    #[test]
    fn a_finished_chunked_file_reads_as_whole() {
        let mut q = Queue::default();
        let a = q.add(file("a.mp4", 100), String::new());
        start_all(&mut q, &batch(""));
        q.next(1);
        q.sent(
            a,
            1,
            Transfer {
                done: 60,
                total: Some(100),
                speed: 1,
            },
        );
        q.finish(a, 1, Outcome::Posted(9));
        assert_eq!(q.get(a).and_then(|i| i.progress).map(|p| p.done), Some(100));
    }
}
