//! Work that outlives a page: the upload queue and the server's import jobs.
//! Both live at the app's root, so moving to another page doesn't stop them.
//! Files are read from this browser, so their uploads stop with the tab, and
//! closing it asks first; imports run on the server and finish regardless.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::Utc;
use futures_util::future::{AbortHandle, Abortable};
use leptos::prelude::*;
use leptos::task::spawn_local;
use send_wrapper::SendWrapper;
use starbooru_client::FileUpload;
use starbooru_core::{
    ImportJob, ImportRequest, MAX_IMPORT_BATCH, Meter, Rating, Transfer, UPLOAD_CHUNK_SIZE,
};

use crate::api::{self, Refused};
use crate::components::snackbar::notify;
use crate::dom;
use crate::files::{self, Picked};
use crate::logic::finding::Unlooked;
use crate::logic::jobs::{Ended, finished, merge, poll_every};
use crate::logic::queue::{AT_ONCE, Batch, Outcome, Queue, Source, Status};
use crate::logic::upload::{Refusal, duplicate_post, human_size, refusal, too_big};
use crate::pages::{bad_tag_text, gallery};
use crate::store;

/// A file on its way, and how to stop it.
struct Flight {
    attempt: u32,
    /// Drops the request at once.
    abort: AbortHandle,
    /// Makes the next piece of the file fail to read, so the client gives up
    /// on its own and tells the server to drop the pieces it has.
    stop: Rc<Cell<bool>>,
    /// Sent in pieces, which dropping the request would leave on the server.
    chunked: bool,
}

impl Flight {
    /// Stops it. Says whether it's still winding down: a chunked one is,
    /// until the piece on its way is answered and the server told.
    fn stop(&self) -> bool {
        self.stop.set(true);
        if self.chunked {
            return true;
        }
        self.abort.abort();
        false
    }
}

thread_local! {
    // Reference counted, as they outlive every page.
    static QUEUE: ArcRwSignal<Queue> = ArcRwSignal::new(Queue::default());
    static JOBS: ArcRwSignal<Vec<ImportJob>> = ArcRwSignal::new(Vec::new());
    /// The picked files, by their item's key. Kept apart from the queue, as
    /// a browser handle isn't plain data.
    static FILES: RefCell<HashMap<u64, web_sys::File>> = RefCell::new(HashMap::new());
    /// The files on their way, by their item's key.
    static FLIGHTS: RefCell<HashMap<u64, Flight>> = RefCell::new(HashMap::new());
    /// Set to ask for the job list on the next check.
    static STALE: Cell<bool> = const { Cell::new(true) };
    /// Jobs seen going since the list was last still, for the notice when it is.
    static WATCHED: RefCell<HashSet<i64>> = RefCell::new(HashSet::new());
    /// How the files sent since the queue was last still ended.
    static RUN: Cell<Ended> = const { Cell::new(Ended { done: 0, duplicate: 0, failed: 0 }) };
    /// The uploads of this visit sent with the AI off, which the server
    /// finds no people in.
    static UNLOOKED: RefCell<Unlooked> = RefCell::new(Unlooked::default());
}

/// Whether the server looks for the people of post `id` by itself: not
/// when it was uploaded here with the AI off.
pub fn people_looked_for(id: i64) -> bool {
    UNLOOKED.with_borrow(|u| u.looked_for(id))
}

/// Reads the queue. Inside a view, it redraws when the queue changes.
pub fn with_queue<R>(f: impl FnOnce(&Queue) -> R) -> R {
    QUEUE.with(|q| q.with(f))
}

fn update(f: impl FnOnce(&mut Queue)) {
    QUEUE.with(|q| q.update(f));
}

/// Reads the import jobs, newest first. Inside a view, it redraws when they
/// change.
pub fn with_jobs<R>(f: impl FnOnce(&[ImportJob]) -> R) -> R {
    JOBS.with(|j| j.with(|j| f(j)))
}

/// The picked file behind item `key`, to show it.
pub fn file(key: u64) -> Option<web_sys::File> {
    FILES.with_borrow(|f| f.get(&key).cloned())
}

/// Drops the files of items no longer on the list.
fn forget_files() {
    let kept: HashSet<u64> =
        QUEUE.with(|q| q.with_untracked(|q| q.items().iter().map(|i| i.key).collect()));
    FILES.with_borrow_mut(|f| f.retain(|key, _| kept.contains(key)));
}

/// Adds a picked file as a draft, with `tags` of its own.
pub fn add_file(picked: Picked, tags: String) -> u64 {
    let Picked { file, path } = picked;
    let source = Source::File {
        name: file.name(),
        size: file.size().max(0.0) as u64,
        mime: file.type_(),
        path,
    };
    let key = add(source, tags);
    FILES.with_borrow_mut(|f| f.insert(key, file));
    key
}

/// Adds an address to import as a draft.
pub fn add_url(url: String) -> u64 {
    add(Source::Url(url), String::new())
}

/// The first of a new batch also clears away the imports the last one left.
fn add(source: Source, tags: String) -> u64 {
    let fresh = QUEUE.with(|q| q.with_untracked(Queue::starts_batch));
    if fresh && JOBS.with(|j| j.with_untracked(|j| j.iter().any(|j| !j.state.is_active()))) {
        clear_jobs_now();
    }
    let mut key = 0;
    update(|q| key = q.add(source, tags));
    if fresh {
        forget_files();
    }
    key
}

pub fn set_tags(key: u64, tags: String) {
    let same = QUEUE.with(|q| q.with_untracked(|q| q.get(key).is_some_and(|i| i.tags == tags)));
    if !same {
        update(|q| q.set_tags(key, tags));
    }
}

pub fn set_rating(key: u64, rating: Option<Rating>) {
    update(|q| q.set_rating(key, rating));
}

/// Takes an item off the list, unless it's on its way.
pub fn remove(key: u64) {
    update(|q| {
        q.remove(key);
    });
    forget_files();
}

/// Empties the list but for what is on its way, and the finished imports.
pub fn clear() {
    update(Queue::clear);
    forget_files();
    clear_jobs_now();
}

/// Sends the drafts, or only those in `keys`: files to the upload queue,
/// addresses to the server's imports.
pub fn start(batch: &Batch, keys: Option<&[u64]>) {
    let mut imports = Vec::new();
    update(|q| {
        imports = q.start(
            batch,
            |key| keys.is_none_or(|keys| keys.contains(&key)),
            bad_tag_text,
        );
    });
    if !imports.is_empty() {
        spawn_local(send_imports(imports));
    }
    pump();
}

/// Stops a file on its way, or waiting for its turn.
pub fn cancel(key: u64) {
    let mut stopped = false;
    update(|q| stopped = q.cancel(key));
    if stopped {
        FLIGHTS.with_borrow_mut(|f| {
            if f.get(&key).is_some_and(|flight| !flight.stop()) {
                f.remove(&key);
            }
        });
        pump();
    }
}

/// Stops every file on its way.
fn stop_all() {
    FLIGHTS.with_borrow_mut(|f| f.retain(|_, flight| flight.stop()));
}

/// How long signing out waits for the files sent in pieces to tell the
/// server to drop them, in tenths of a second.
const WIND_DOWN_TENTHS: u32 = 100;

/// Stops every file on its way, as at sign-out. The future ends once those
/// sent in pieces have told the server to drop them, which needs the
/// session, or after a while.
pub fn wind_down() -> impl Future<Output = ()> {
    stop_all();
    async {
        for _ in 0..WIND_DOWN_TENTHS {
            if FLIGHTS.with_borrow(HashMap::is_empty) {
                return;
            }
            dom::sleep(100).await;
        }
    }
}

/// Sends a failed or stopped item again.
pub fn retry(key: u64) {
    let mut again = None;
    update(|q| again = q.retry(key));
    match again {
        Some(Some(request)) => spawn_local(send_imports(vec![(key, request)])),
        Some(None) => pump(),
        None => {}
    }
}

/// How many uploads and imports are still to finish, for the navigation's
/// badge. Inside a view, it redraws when that changes.
pub fn pending() -> usize {
    let uploads = with_queue(Queue::pending);
    let imports = with_jobs(|j| j.iter().filter(|j| j.state.is_active()).count());
    uploads.saturating_add(imports)
}

/// Whether files are on their way, which closing the tab would stop.
fn sending_files() -> bool {
    QUEUE.with(|q| {
        q.with_untracked(|q| {
            q.items()
                .iter()
                .any(|i| i.pending() && matches!(i.source, Source::File { .. }))
        })
    })
}

/// Starts as many waiting files as may go at once.
fn pump() {
    let waiting = QUEUE.with(|q| {
        q.with_untracked(|q| {
            q.items()
                .iter()
                .any(|i| i.status == crate::logic::queue::Status::Waiting)
        })
    });
    if waiting {
        let mut next = Vec::new();
        update(|q| next = q.next(AT_ONCE));
        for (key, attempt) in next {
            spawn_local(send(key, attempt));
        }
    }
    if !QUEUE.with(|q| q.with_untracked(|q| q.items().iter().any(|i| i.pending()))) {
        ran();
    }
}

/// Everything sent has finished: says how it went, and shows the new posts.
fn ran() {
    let ended = RUN.replace(Ended::default());
    if ended == Ended::default() {
        return;
    }
    notify(untrack(|| summary(ended, true)));
    if ended.done > 0 {
        api::forget_tags();
        gallery::posts_arrived();
    }
}

/// Whether attempt `attempt` at item `key` is still the one on its way.
fn still_sending(key: u64, attempt: u32) -> bool {
    QUEUE.with(|q| {
        q.with_untracked(|q| {
            q.get(key)
                .is_some_and(|i| i.status == Status::Sending && i.attempt == attempt)
        })
    })
}

/// Takes attempt `attempt` at item `key` off the files on their way, unless
/// a later one has taken its place.
fn landed(key: u64, attempt: u32) {
    FLIGHTS.with_borrow_mut(|f| {
        if f.get(&key).is_some_and(|flight| flight.attempt == attempt) {
            f.remove(&key);
        }
    });
}

/// One file, from its first byte to the server's answer.
async fn send(key: u64, attempt: u32) {
    let item = QUEUE.with(|q| {
        q.with_untracked(|q| {
            let item = q.get(key)?;
            Some((item.name().to_owned(), item.size()?, item.plan.clone()?))
        })
    });
    let (Some((name, size, plan)), Some(file)) = (item, file(key)) else {
        end(key, attempt, Outcome::Failed(t!("upload-gone")));
        return;
    };
    // Stoppable from the start, so a cancel while the server is asked below
    // isn't lost.
    let (abort, registration) = AbortHandle::new_pair();
    let stop = Rc::new(Cell::new(false));
    let flight = Flight {
        attempt,
        abort,
        stop: Rc::clone(&stop),
        chunked: size > UPLOAD_CHUNK_SIZE,
    };
    FLIGHTS.with_borrow_mut(|f| f.insert(key, flight));
    let owner = store::owner();
    let limit = api::server_info().await.map_or(0, |i| i.max_upload_bytes);
    if stop.get() || !still_sending(key, attempt) {
        landed(key, attempt);
        return;
    }
    if too_big(size, limit) {
        landed(key, attempt);
        let max = human_size(limit);
        end(
            key,
            attempt,
            Outcome::Failed(t!("upload-too-big", max = max)),
        );
        return;
    }
    let autotag = plan.autotag;
    let upload = FileUpload {
        file_name: name.clone(),
        size,
        tags: plan.tags,
        rating: plan.rating,
        source: plan.source,
        autotag,
    };
    let reading = Rc::clone(&stop);
    let work = api::attempt(move |client| async move {
        let meter = RefCell::new(Meter::new(Utc::now()));
        let sent = |done: u64| {
            let speed = meter.borrow_mut().seen(done, Utc::now());
            let progress = Transfer {
                done,
                total: Some(size),
                speed,
            };
            update(|q| q.sent(key, attempt, progress));
        };
        let read = move |offset: u64, len: u64| {
            let file = file.clone();
            let stopped = reading.get();
            async move {
                // Never shown: a stopped file's outcome is dropped.
                if stopped {
                    return Err("stopped".to_owned());
                }
                files::read_range(&file, offset, len).await
            }
        };
        client.upload_file(upload, read, &sent).await
    });
    let result = Abortable::new(work, registration).await;
    landed(key, attempt);
    // Cancelled while sent whole: the queue already says so.
    let Ok(result) = result else {
        return;
    };
    if stop.get() {
        // The whole file was on the server before the stop came.
        if let Ok(post) = result
            && store::owner() == owner
        {
            notify(t!("upload-stopped-late", name = name, id = post.id));
            api::forget_tags();
            gallery::posts_arrived();
        }
        return;
    }
    let outcome = match result {
        Ok(post) => {
            UNLOOKED.with_borrow_mut(|u| u.uploaded(post.id, autotag));
            Outcome::Posted(post.id)
        }
        Err(refused) => outcome_of(refused, limit),
    };
    end(key, attempt, outcome);
}

/// What the server's refusal of a file means for it.
fn outcome_of(refused: Refused, limit: u64) -> Outcome {
    match refusal(refused.status, refused.code.as_deref()) {
        Refusal::Duplicate => Outcome::Duplicate(duplicate_post(&refused.message)),
        Refusal::TooBig if limit > 0 => {
            Outcome::Failed(t!("upload-too-big", max = human_size(limit)))
        }
        Refusal::Unsupported => Outcome::Failed(t!("upload-unsupported")),
        Refusal::TooBig | Refusal::Other => Outcome::Failed(refused.message),
    }
}

fn end(key: u64, attempt: u32, outcome: Outcome) {
    let tally = |mut e: Ended| {
        match &outcome {
            Outcome::Posted(_) => e.done += 1,
            Outcome::Duplicate(_) => e.duplicate += 1,
            Outcome::Failed(_) => e.failed += 1,
        }
        e
    };
    let mut counted = false;
    let next = tally(RUN.get());
    update(|q| counted = q.finish(key, attempt, outcome.clone()));
    if counted {
        RUN.set(next);
    }
    pump();
}

/// Queues addresses on the server. Those it takes leave the list and show
/// up as its jobs.
async fn send_imports(batch: Vec<(u64, ImportRequest)>) {
    let owner = store::owner();
    for chunk in batch.chunks(MAX_IMPORT_BATCH) {
        let keys: Vec<u64> = chunk.iter().map(|(k, _)| *k).collect();
        let requests: Vec<ImportRequest> = chunk.iter().map(|(_, r)| r.clone()).collect();
        let sent = requests.clone();
        let answer = api::call(move |c| async move { c.queue_imports(requests).await }).await;
        // Signed out meanwhile: the list was that account's, and has gone.
        if store::owner() != owner {
            return;
        }
        match answer {
            Ok(jobs) => {
                UNLOOKED.with_borrow_mut(|u| u.queued(&sent, &jobs));
                WATCHED.with_borrow_mut(|w| w.extend(jobs.iter().map(|j| j.id)));
                JOBS.with(|j| j.update(|list| *list = merge(list, jobs)));
                update(|q| q.imported(&keys));
                STALE.set(true);
            }
            Err(message) => update(|q| q.import_failed(&keys, &message)),
        }
    }
    pump();
}

/// Queues imports on the server for a page other than Upload (the booru
/// browser's selection). They show up as jobs here, on the Upload page and
/// in the navigation's badge. The words to show when the server refuses.
pub async fn queue_imports(requests: Vec<ImportRequest>) -> Result<(), String> {
    let owner = store::owner();
    for chunk in requests.chunks(MAX_IMPORT_BATCH) {
        let items = chunk.to_vec();
        let jobs = api::call(move |c| async move { c.queue_imports(items).await }).await?;
        if store::owner() != owner {
            return Ok(());
        }
        UNLOOKED.with_borrow_mut(|u| u.queued(chunk, &jobs));
        WATCHED.with_borrow_mut(|w| w.extend(jobs.iter().map(|j| j.id)));
        JOBS.with(|j| j.update(|list| *list = merge(list, jobs)));
    }
    STALE.set(true);
    Ok(())
}

/// Cancels an import that hasn't started.
pub fn cancel_job(id: i64) {
    let owner = store::owner();
    spawn_local(async move {
        let answer = api::call(move |c| async move { c.delete_import(id).await }).await;
        if store::owner() != owner {
            return;
        }
        match answer {
            Ok(()) => JOBS.with(|j| j.update(|list| list.retain(|j| j.id != id))),
            Err(message) => notify(message),
        }
    });
}

/// Takes the finished imports off the list at once, then off the server. On
/// a failure the next check brings them back.
pub fn clear_jobs_now() {
    let any = JOBS.with(|j| j.with_untracked(|j| j.iter().any(|j| !j.state.is_active())));
    if !any {
        return;
    }
    JOBS.with(|j| j.update(|list| list.retain(|j| j.state.is_active())));
    let owner = store::owner();
    spawn_local(async move {
        if let Err(message) = api::call(|c| async move { c.clear_imports().await }).await
            && store::owner() == owner
        {
            notify(message);
        }
    });
}

/// Asks for the job list on the next check.
pub fn refresh() {
    STALE.set(true);
}

async fn poll() {
    let owner = store::owner();
    // A failed check keeps the list it had; the next one tries again.
    let Ok(now) = api::call(|c| async move { c.imports().await }).await else {
        return;
    };
    // Signed out meanwhile: these are the last account's.
    if store::owner() != owner {
        return;
    }
    UNLOOKED.with_borrow_mut(|u| u.imports(&now));
    let ended = WATCHED.with_borrow_mut(|w| finished(w, &now));
    if let Some(ended) = ended {
        // Out of any view: the language is read, not followed.
        notify(untrack(|| summary(ended, false)));
        if ended.done > 0 {
            api::forget_tags();
            gallery::posts_arrived();
        }
    }
    JOBS.with(|j| {
        if j.with_untracked(|j| *j != now) {
            j.set(now);
        }
    });
}

/// What a finished run of uploads or imports says.
fn summary(ended: Ended, uploads: bool) -> String {
    let Ended {
        done,
        duplicate,
        failed,
    } = ended;
    let mut parts = Vec::new();
    if done > 0 || duplicate.saturating_add(failed) == 0 {
        parts.push(if uploads {
            t!("activity-uploaded", count = done)
        } else {
            t!("activity-imported", count = done)
        });
    }
    if duplicate > 0 {
        parts.push(t!("activity-known", count = duplicate));
    }
    if failed > 0 {
        parts.push(t!("activity-failed", count = failed));
    }
    parts.join(" · ")
}

/// Keeps the job list current while someone is signed in, and asks before a
/// tab with files on their way is closed. Mounted once, in the shell; when it
/// goes, at sign-out, the queue goes with it.
#[component]
pub fn Activity() -> impl IntoView {
    let alive = Arc::new(AtomicBool::new(true));
    {
        let alive = Arc::clone(&alive);
        spawn_local(async move {
            let mut waited = 0u32;
            while alive.load(Ordering::Relaxed) {
                let every = JOBS.with(|j| j.with_untracked(|j| poll_every(j)));
                if STALE.replace(false) || waited >= every {
                    waited = 0;
                    poll().await;
                }
                dom::sleep(1000).await;
                waited = waited.saturating_add(1);
            }
        });
    }
    let watch = web_sys::window().map(|window| {
        dom::listen(&window, "beforeunload", false, |e| {
            if sending_files() {
                e.prevent_default();
                // What older browsers look at.
                let _ = js_sys::Reflect::set(&e, &"returnValue".into(), &"".into());
            }
        })
    });
    let watch = SendWrapper::new(watch);
    on_cleanup(move || {
        alive.store(false, Ordering::Relaxed);
        if let Some(watch) = watch.take() {
            watch.stop();
        }
        // Sign-out stopped them already, unless the server ended the session.
        stop_all();
        QUEUE.with(|q| q.set(Queue::default()));
        JOBS.with(|j| j.set(Vec::new()));
        FILES.with_borrow_mut(HashMap::clear);
        WATCHED.with_borrow_mut(HashSet::clear);
        RUN.set(Ended::default());
        STALE.set(true);
        UNLOOKED.with_borrow_mut(|u| *u = Unlooked::default());
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_finished_run_says_how_it_went() {
        let e = |done, duplicate, failed| Ended {
            done,
            duplicate,
            failed,
        };
        assert_eq!(summary(e(3, 0, 0), true), "3 files uploaded");
        assert_eq!(summary(e(1, 0, 0), false), "1 post imported");
        assert_eq!(
            summary(e(2, 1, 1), true),
            "2 files uploaded · 1 already here · 1 failed"
        );
        assert_eq!(summary(e(0, 2, 0), true), "2 already here");
    }

    #[test]
    fn a_refusal_becomes_the_files_outcome() {
        let refused = |status, code: &str, message: &str| Refused {
            message: message.into(),
            missing: false,
            status: Some(status),
            code: Some(code.into()),
            retry_after: None,
        };
        assert_eq!(
            outcome_of(
                refused(409, "duplicate", "this file is already post #12"),
                0
            ),
            Outcome::Duplicate(Some(12))
        );
        assert_eq!(
            outcome_of(refused(413, "too_large", "too large"), 1 << 20),
            Outcome::Failed("Bigger than this server takes (1.0 MB at most).".into())
        );
        // Without the limit, the server's words.
        assert_eq!(
            outcome_of(refused(413, "too_large", "too large"), 0),
            Outcome::Failed("too large".into())
        );
        assert!(matches!(
            outcome_of(refused(415, "unsupported", "x"), 0),
            Outcome::Failed(m) if m.contains("image or a video")
        ));
        assert_eq!(
            outcome_of(refused(500, "internal", "oops"), 0),
            Outcome::Failed("oops".into())
        );
    }
}
