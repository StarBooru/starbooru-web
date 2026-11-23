//! What a search on other boorus found: one page from each site as one
//! list, the same file on two sites shown once, how saving each stands, how
//! far each site has been read, how long the results are kept, and which
//! posts are picked.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use starbooru_core::{BooruApi, ImportJob, JobState, RemotePost, RemoteSite};

use super::tidy;

/// One page from each site as one list: the first post of each in turn, then
/// the second of each, and so on.
pub fn interleave(pages: Vec<Vec<RemotePost>>) -> Vec<RemotePost> {
    let longest = pages.iter().map(Vec::len).max().unwrap_or(0);
    let mut pages: Vec<_> = pages.into_iter().map(Vec::into_iter).collect();
    let mut out = Vec::new();
    for _ in 0..longest {
        out.extend(pages.iter_mut().filter_map(Iterator::next));
    }
    out
}

/// Adds `more` to `all`, leaving out posts already there. Says how many were
/// new.
pub fn append(all: &mut Vec<Arc<RemotePost>>, more: Vec<RemotePost>) -> usize {
    let mut seen: HashSet<String> = all.iter().map(|p| p.page_url.clone()).collect();
    let before = all.len();
    for post in more {
        if seen.insert(post.page_url.clone()) {
            all.push(Arc::new(post));
        }
    }
    all.len().saturating_sub(before)
}

/// The same file on another site, left out of the grid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Twin {
    /// The site's address.
    pub site: String,
    pub page_url: String,
}

/// A post in the grid, and where else its file is.
#[derive(Clone, Debug, PartialEq)]
pub struct Found {
    pub post: Arc<RemotePost>,
    pub also: Vec<Twin>,
}

/// The results as the grid shows them. With `hide`, a post whose file (by
/// its MD5) was already shown is left out and noted on the one that stays. A
/// post without an MD5 is never a duplicate.
pub fn shown(all: &[Arc<RemotePost>], hide: bool) -> Vec<Found> {
    let mut out: Vec<Found> = Vec::with_capacity(all.len());
    let mut by_md5: HashMap<&str, usize> = HashMap::new();
    for post in all {
        let md5 = post.md5.as_deref().filter(|md5| hide && !md5.is_empty());
        if let Some(first) = md5.and_then(|md5| by_md5.get(md5).copied()) {
            if let Some(kept) = out.get_mut(first) {
                kept.also.push(Twin {
                    site: post.site.clone(),
                    page_url: post.page_url.clone(),
                });
            }
            continue;
        }
        if let Some(md5) = md5 {
            by_md5.insert(md5, out.len());
        }
        out.push(Found {
            post: Arc::clone(post),
            also: Vec::new(),
        });
    }
    out
}

/// How saving a post here stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// Waiting or downloading, with how much is done when that is known.
    Busy(Option<u8>),
    /// In the collection: as which post, when that is known, and what the
    /// server said about it.
    Saved(Option<i64>, String),
    Failed(String),
}

/// Where `post` stands: saved before, or what its latest import says
/// (`jobs` by the page they came from, `logic::jobs::by_source`).
pub fn status_of(post: &RemotePost, jobs: &HashMap<String, ImportJob>) -> Option<Status> {
    let job = jobs.get(&post.page_url);
    let said = |job: &ImportJob| job.error.clone().unwrap_or_default();
    match job.map(|job| (job, job.state, job.post_id)) {
        Some((job, JobState::Queued | JobState::Running, _)) => {
            Some(Status::Busy(job.progress.and_then(|p| p.percent())))
        }
        Some((_, JobState::Done, Some(id))) => Some(Status::Saved(Some(id), String::new())),
        _ if post.post_id.is_some() => Some(Status::Saved(post.post_id, String::new())),
        // Imported and deleted since, or the file was already here.
        Some((job, JobState::Done | JobState::Duplicate, _)) => {
            Some(Status::Saved(None, said(job)))
        }
        Some((job, JobState::Failed, _)) => Some(Status::Failed(said(job))),
        None => None,
    }
}

/// How saving a post stands, in full words: what a screen reader hears with
/// its tile, which only shows a short mark.
pub fn status_note(status: &Status) -> String {
    match status {
        Status::Busy(Some(percent)) => format!("{} {percent}%", t!("upload-job-running")),
        Status::Busy(None) => t!("upload-job-running"),
        Status::Saved(Some(id), _) => t!("boorus-saved-as", id = *id),
        Status::Saved(None, note) if note.trim().is_empty() => t!("boorus-known"),
        Status::Saved(None, note) => format!("{}: {}", t!("boorus-known"), note.trim()),
        Status::Failed(reason) if reason.trim().is_empty() => t!("upload-failed"),
        Status::Failed(reason) => t!("boorus-save-failed", reason = reason.trim().to_owned()),
    }
}

/// The post here that `post` has become since it was listed, by its import.
fn newly_saved(post: &RemotePost, jobs: &HashMap<String, ImportJob>) -> Option<i64> {
    jobs.get(&post.page_url)
        .filter(|job| post.post_id.is_none() && job.state == JobState::Done)
        .and_then(|job| job.post_id)
}

/// Whether any of `all` was saved since by one of `jobs`.
pub fn any_newly_saved(all: &[Arc<RemotePost>], jobs: &HashMap<String, ImportJob>) -> bool {
    all.iter().any(|p| newly_saved(p, jobs).is_some())
}

/// Notes on each post the import that has finished since, so it stays saved
/// after the job is forgotten. Says whether anything changed.
pub fn mark_saved(all: &mut [Arc<RemotePost>], jobs: &HashMap<String, ImportJob>) -> bool {
    let mut changed = false;
    for post in all.iter_mut() {
        if let Some(id) = newly_saved(post, jobs) {
            Arc::make_mut(post).post_id = Some(id);
            changed = true;
        }
    }
    changed
}

/// Where the server passes on the pictures of sites that refuse other pages.
pub const RELAY_PATH: &str = "/api/v1/remote/media?";

/// The address to show one of a post's pictures from, or `None` for the
/// tile's colour instead. The server decides which pictures it passes on:
/// those come as `RELAY_PATH…` without its address, which `server` (the
/// server's base address) goes in front of, or already on it. Any other
/// picture comes straight from its site, so it must be `https:`.
pub fn picture_src(url: &str, server: &str) -> Option<String> {
    let url = url.trim();
    let server = server.trim_end_matches('/');
    if url.starts_with(RELAY_PATH) {
        return Some(format!("{server}{url}"));
    }
    let relayed = !server.is_empty()
        && url
            .strip_prefix(server)
            .is_some_and(|rest| rest.starts_with(RELAY_PATH));
    let https = url.split_once("://").is_some_and(|(scheme, rest)| {
        scheme.eq_ignore_ascii_case("https") && !rest.is_empty() && !rest.starts_with('/')
    });
    (relayed || https).then(|| url.to_owned())
}

/// A picture's size for the layout. A site that doesn't say gives 0.
pub fn size_of(post: &RemotePost) -> (Option<i32>, Option<i32>) {
    let side = |n: u32| i32::try_from(n).ok().filter(|n| *n > 0);
    (side(post.width), side(post.height))
}

/// How far one site has been read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SitePage {
    pub site: RemoteSite,
    /// The page to ask for next.
    pub next: u32,
    pub more: bool,
    /// What went wrong the last time it was asked.
    pub error: Option<String>,
}

impl SitePage {
    pub fn first(site: &RemoteSite) -> Self {
        Self {
            site: site.clone(),
            next: 1,
            more: true,
            error: None,
        }
    }

    /// Whether the next page is worth asking for.
    pub fn wanted(&self) -> bool {
        self.more && self.error.is_none()
    }

    /// What the site said, `page` being the posts it sent and `more` whether
    /// it has another page. A site that only repeats what is already there
    /// has no more to give.
    pub fn answered(&mut self, page: &[RemotePost], more: bool, known: &HashSet<String>) {
        let fresh = page.iter().any(|p| !known.contains(&p.page_url));
        self.more = more && fresh;
        self.next = self.next.saturating_add(1);
    }
}

/// How long the results of a search are kept while the app runs. The
/// pictures relayed by the server are behind signed addresses, good for six
/// hours at least.
pub const MAX_KEPT_HOURS: i64 = 4;

/// Whether results loaded at `loaded_at` are still good to show at `now`. A
/// clock set back makes them stale too.
pub fn fresh(loaded_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    let age = now - loaded_at;
    age.num_seconds() >= 0 && age.num_hours() < MAX_KEPT_HOURS
}

/// A kept post with its neighbours in the grid, for the post's own page.
#[derive(Clone, Debug, PartialEq)]
pub struct Around {
    pub found: Found,
    pub prev: Option<String>,
    pub next: Option<String>,
    /// The kind of site it is on.
    pub api: Option<BooruApi>,
}

/// The post at page `u` among the results: `list` as the grid shows them,
/// `all` as they came, from `sites`. A duplicate left out of the grid has no
/// neighbours.
pub fn around(
    list: &[Found],
    all: &[Arc<RemotePost>],
    sites: &[RemoteSite],
    u: &str,
) -> Option<Around> {
    let api_of_base = |base: &str| {
        sites
            .iter()
            .find(|s| tidy(&s.base) == tidy(base))
            .map(|s| s.api)
    };
    if let Some(at) = list.iter().position(|f| f.post.page_url == u) {
        let page = |i: Option<usize>| i.and_then(|i| list.get(i)).map(|f| f.post.page_url.clone());
        let found = list.get(at)?.clone();
        return Some(Around {
            api: api_of_base(&found.post.site),
            prev: page(at.checked_sub(1)),
            next: page(at.checked_add(1)),
            found,
        });
    }
    let post = all.iter().find(|p| p.page_url == u)?;
    Some(Around {
        api: api_of_base(&post.site),
        found: Found {
            post: Arc::clone(post),
            also: Vec::new(),
        },
        prev: None,
        next: None,
    })
}

/// The posts picked to save, by their page, and the page of the last one
/// clicked, where a Shift-click's range starts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Picks {
    pages: HashSet<String>,
    anchor: Option<String>,
}

impl Picks {
    pub fn contains(&self, page: &str) -> bool {
        self.pages.contains(page)
    }

    /// How many of `pages` are picked. The grid can change under a
    /// selection, so what it no longer shows doesn't count.
    pub fn count_in(&self, pages: &[String]) -> usize {
        pages.iter().filter(|p| self.pages.contains(*p)).count()
    }

    /// A click on the post at `index` of `list`, each post's page with
    /// whether it can be picked (not saved, nor on its way). With `shift`,
    /// every free post from the one clicked last to this one takes this
    /// one's new state. A last one the list no longer has starts no range.
    pub fn toggle(&mut self, list: &[(String, bool)], index: usize, shift: bool) {
        let Some((page, true)) = list.get(index) else {
            return;
        };
        let on = !self.pages.contains(page);
        let anchor = self
            .anchor
            .as_ref()
            .filter(|_| shift)
            .and_then(|last| list.iter().position(|(p, _)| p == last));
        match anchor {
            Some(anchor) => {
                let range = list
                    .get(anchor.min(index)..=anchor.max(index))
                    .unwrap_or_default();
                for (page, free) in range {
                    if !on {
                        self.pages.remove(page);
                    } else if *free {
                        self.pages.insert(page.clone());
                    }
                }
            }
            None => {
                if on {
                    self.pages.insert(page.clone());
                } else {
                    self.pages.remove(page);
                }
            }
        }
        self.anchor = Some(page.clone());
    }

    /// Whether every one of `free` is picked.
    pub fn all_of(&self, free: &[String]) -> bool {
        !free.is_empty() && free.iter().all(|p| self.pages.contains(p))
    }

    /// Every one of `free`, or none once they all are.
    pub fn toggle_all(&mut self, free: &[String]) {
        if self.all_of(free) {
            self.pages.clear();
        } else {
            self.pages.extend(free.iter().cloned());
        }
        self.anchor = None;
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
pub(super) mod tests {
    use chrono::Duration;
    use starbooru_core::{Rating, Transfer};

    use super::*;
    use crate::logic::boorus::tests::preset;
    use crate::logic::jobs::by_source;

    pub const A: &str = "https://a.org";
    pub const B: &str = "https://b.org";

    pub fn post(base: &str, id: u64, md5: Option<&str>) -> RemotePost {
        RemotePost {
            id,
            site: base.into(),
            page_url: format!("{base}/posts/{id}"),
            preview_url: format!("{base}/preview/{id}.jpg"),
            sample_url: None,
            file_url: format!("{base}/file/{id}.jpg"),
            tags: vec!["sky".into(), "someone".into()],
            groups: Default::default(),
            rating: Rating::Sensitive,
            width: 640,
            height: 480,
            md5: md5.map(str::to_owned),
            source: None,
            score: None,
            created_at: None,
            file_size: None,
            post_id: None,
        }
    }

    fn job(id: i64, state: JobState, page: &str) -> ImportJob {
        ImportJob {
            id,
            url: format!("{page}.jpg"),
            source: Some(page.to_owned()),
            tags: Vec::new(),
            rating: Rating::default(),
            state,
            post_id: None,
            error: None,
            created_at: Utc::now(),
            finished_at: None,
            progress: None,
        }
    }

    fn pages(all: &[Arc<RemotePost>]) -> Vec<&str> {
        all.iter().map(|p| p.page_url.as_str()).collect()
    }

    #[test]
    fn a_page_of_results_takes_a_post_from_each_site_in_turn() {
        let a = vec![post(A, 1, None), post(A, 2, None), post(A, 3, None)];
        let b = vec![post(B, 1, None)];
        let mixed = interleave(vec![a, Vec::new(), b]);
        let order: Vec<&str> = mixed.iter().map(|p| p.page_url.as_str()).collect();
        assert_eq!(
            order,
            [
                "https://a.org/posts/1",
                "https://b.org/posts/1",
                "https://a.org/posts/2",
                "https://a.org/posts/3"
            ]
        );
        assert!(interleave(Vec::new()).is_empty());

        // The next page goes after it, without what is already there.
        let mut all = Vec::new();
        assert_eq!(append(&mut all, mixed), 4);
        let more = vec![post(A, 3, None), post(A, 4, None), post(A, 4, None)];
        assert_eq!(append(&mut all, more), 1);
        assert_eq!(pages(&all).len(), 5);
        assert_eq!(pages(&all)[4], "https://a.org/posts/4");
    }

    #[test]
    fn the_same_file_on_two_sites_shows_once_when_asked() {
        let all: Vec<Arc<RemotePost>> = [
            post(A, 1, Some("aa")),
            post(B, 9, Some("bb")),
            post(A, 2, None),
            post(B, 8, Some("aa")),
            post(B, 7, None),
            post(A, 3, Some("bb")),
            post(A, 4, Some("")),
            post(B, 6, Some("")),
        ]
        .into_iter()
        .map(Arc::new)
        .collect();

        // As they came, when duplicates aren't hidden.
        let every = shown(&all, false);
        assert_eq!(every.len(), all.len());
        assert!(every.iter().all(|f| f.also.is_empty()));

        let once = shown(&all, true);
        let order: Vec<&str> = once.iter().map(|f| f.post.page_url.as_str()).collect();
        assert_eq!(
            order,
            [
                "https://a.org/posts/1",
                "https://b.org/posts/9",
                "https://a.org/posts/2",
                "https://b.org/posts/7",
                // No MD5, or an empty one, is never a duplicate.
                "https://a.org/posts/4",
                "https://b.org/posts/6"
            ]
        );
        // The one that stays knows where else the file is.
        assert_eq!(
            once[0].also,
            [Twin {
                site: B.into(),
                page_url: "https://b.org/posts/8".into()
            }]
        );
        assert_eq!(once[1].also[0].page_url, "https://a.org/posts/3");
        assert!(once[2].also.is_empty());
    }

    #[test]
    fn a_post_says_how_saving_it_stands() {
        let mut p = post(A, 1, None);
        let page = p.page_url.clone();
        let jobs = |job: ImportJob| by_source(&[job]);
        assert_eq!(status_of(&p, &HashMap::new()), None);

        let mut running = job(1, JobState::Running, &page);
        assert_eq!(
            status_of(&p, &jobs(running.clone())),
            Some(Status::Busy(None))
        );
        running.progress = Some(Transfer {
            done: 42,
            total: Some(100),
            speed: 0,
        });
        assert_eq!(status_of(&p, &jobs(running)), Some(Status::Busy(Some(42))));
        assert_eq!(
            status_of(&p, &jobs(job(1, JobState::Queued, &page))),
            Some(Status::Busy(None))
        );

        let mut done = job(1, JobState::Done, &page);
        done.post_id = Some(12);
        assert_eq!(
            status_of(&p, &jobs(done.clone())),
            Some(Status::Saved(Some(12), String::new()))
        );
        let mut failed = job(2, JobState::Failed, &page);
        failed.error = Some("the site answered 404".into());
        assert_eq!(
            status_of(&p, &jobs(failed.clone())),
            Some(Status::Failed("the site answered 404".into()))
        );
        let mut twin = job(3, JobState::Duplicate, &page);
        twin.error = Some("already here as post 5".into());
        assert_eq!(
            status_of(&p, &jobs(twin)),
            Some(Status::Saved(None, "already here as post 5".into()))
        );

        // A post the server already knows is saved, whatever an old job says.
        p.post_id = Some(30);
        assert_eq!(
            status_of(&p, &HashMap::new()),
            Some(Status::Saved(Some(30), String::new()))
        );
        assert_eq!(
            status_of(&p, &jobs(failed)),
            Some(Status::Saved(Some(30), String::new()))
        );
        // One imported by its page is found by that page.
        let mut by_page = job(4, JobState::Queued, &page);
        by_page.url = page.clone();
        by_page.source = None;
        assert_eq!(status_of(&p, &jobs(by_page)), Some(Status::Busy(None)));

        // Once done, the post keeps the note after the job is forgotten.
        let mut all = vec![Arc::new(post(A, 1, None)), Arc::new(post(A, 2, None))];
        assert!(any_newly_saved(&all, &jobs(done.clone())));
        assert!(mark_saved(&mut all, &jobs(done.clone())));
        assert_eq!((all[0].post_id, all[1].post_id), (Some(12), None));
        assert!(!mark_saved(&mut all, &jobs(done.clone())));
        assert!(!any_newly_saved(&all, &jobs(done)));
    }

    #[test]
    fn a_tile_says_in_words_how_saving_it_stands() {
        assert_eq!(status_note(&Status::Busy(None)), "Importing");
        assert_eq!(status_note(&Status::Busy(Some(42))), "Importing 42%");
        assert_eq!(
            status_note(&Status::Saved(Some(12), String::new())),
            "Saved here as post #12"
        );
        assert_eq!(
            status_note(&Status::Saved(None, " ".into())),
            "Already here"
        );
        assert_eq!(
            status_note(&Status::Saved(None, "already here as post 5".into())),
            "Already here: already here as post 5"
        );
        assert_eq!(status_note(&Status::Failed(String::new())), "Failed");
        assert_eq!(
            status_note(&Status::Failed("the site answered 404 ".into())),
            "Couldn't save it: the site answered 404"
        );
    }

    #[test]
    fn pictures_come_from_the_server_or_over_https() {
        let relayed = "/api/v1/remote/media?u=x&exp=1&sig=ab";
        // The server's relay gets its address in front, under a path too.
        assert_eq!(
            picture_src(relayed, "https://my.example").as_deref(),
            Some("https://my.example/api/v1/remote/media?u=x&exp=1&sig=ab")
        );
        assert_eq!(
            picture_src(relayed, "http://nas:8080/booru/").as_deref(),
            Some("http://nas:8080/booru/api/v1/remote/media?u=x&exp=1&sig=ab")
        );
        assert_eq!(picture_src(relayed, "").as_deref(), Some(relayed));
        // Already on the server's address, plain http included.
        let local = "http://127.0.0.1:8088/api/v1/remote/media?u=x";
        assert_eq!(
            picture_src(local, "http://127.0.0.1:8088").as_deref(),
            Some(local)
        );
        // Straight from the site, over https only.
        for open in [
            "https://cdn.d.org/360x360/a.jpg",
            " HTTPS://cdn.d.org/a.jpg ",
        ] {
            assert_eq!(
                picture_src(open, "http://127.0.0.1:8088").as_deref(),
                Some(open.trim())
            );
        }
        for refused in [
            "http://cdn.d.org/a.jpg",
            "//cdn.d.org/a.jpg",
            "data:image/png;base64,AAAA",
            "javascript:alert(1)",
            "/media/1.jpg?exp=1&sig=ab",
            "/api/v1/remote/mediax",
            "https:///a.jpg",
            "https://",
            "cdn.d.org/a.jpg",
            // The server's relay path, but on another host.
            "http://127.0.0.1:80889/api/v1/remote/media?u=x",
            "http://evil.example/api/v1/remote/media?u=x",
            "",
        ] {
            assert_eq!(
                picture_src(refused, "http://127.0.0.1:8088"),
                None,
                "{refused}"
            );
        }
    }

    #[test]
    fn a_picture_without_a_size_gets_a_square_tile() {
        let mut p = post(A, 1, None);
        assert_eq!(size_of(&p), (Some(640), Some(480)));
        p.width = 0;
        assert_eq!(size_of(&p), (None, Some(480)));
        p.height = u32::MAX;
        assert_eq!(size_of(&p), (None, None));
    }

    #[test]
    fn a_site_is_asked_again_only_while_it_has_more_and_answers() {
        let mut page = SitePage::first(&preset("Danbooru"));
        assert!(page.wanted());
        assert_eq!(page.next, 1);
        let known: HashSet<String> = HashSet::from(["https://a.org/posts/1".to_owned()]);
        page.answered(&[post(A, 2, None)], true, &known);
        assert_eq!((page.next, page.more), (2, true));
        // Only what is already there: nothing more to give.
        page.answered(&[post(A, 1, None)], true, &known);
        assert_eq!((page.next, page.more), (3, false));
        assert!(!page.wanted());
        let mut page = SitePage::first(&preset("Danbooru"));
        page.answered(&[post(A, 2, None)], false, &known);
        assert!(!page.wanted());
        let mut page = SitePage::first(&preset("Danbooru"));
        page.error = Some("that site is busy".into());
        assert!(!page.wanted());
    }

    #[test]
    fn results_are_kept_a_few_hours() {
        let now = Utc::now();
        assert!(fresh(now, now));
        assert!(fresh(now - Duration::minutes(239), now));
        assert!(!fresh(now - Duration::hours(MAX_KEPT_HOURS), now));
        assert!(!fresh(now - Duration::days(2), now));
        // Loaded "in the future": the clock went back.
        assert!(!fresh(now + Duration::minutes(5), now));
    }

    #[test]
    fn a_kept_post_knows_its_neighbours_in_the_grid() {
        let all: Vec<Arc<RemotePost>> = [
            post(A, 1, Some("aa")),
            post(B, 9, None),
            post(B, 8, Some("aa")),
            post(A, 2, None),
        ]
        .into_iter()
        .map(Arc::new)
        .collect();
        let sites = [
            RemoteSite {
                api: BooruApi::Danbooru,
                base: A.into(),
            },
            RemoteSite {
                api: BooruApi::Gelbooru,
                base: format!("{B}/"),
            },
        ];
        let list = shown(&all, true);
        let first = around(&list, &all, &sites, "https://a.org/posts/1").unwrap();
        assert_eq!(first.prev, None);
        assert_eq!(first.next.as_deref(), Some("https://b.org/posts/9"));
        assert_eq!(first.api, Some(BooruApi::Danbooru));
        assert_eq!(first.found.also.len(), 1);
        let middle = around(&list, &all, &sites, "https://b.org/posts/9").unwrap();
        assert_eq!(middle.prev.as_deref(), Some("https://a.org/posts/1"));
        assert_eq!(middle.next.as_deref(), Some("https://a.org/posts/2"));
        assert_eq!(middle.api, Some(BooruApi::Gelbooru));
        // The duplicate left out of the grid opens alone.
        let twin = around(&list, &all, &sites, "https://b.org/posts/8").unwrap();
        assert_eq!((twin.prev, twin.next), (None, None));
        assert!(around(&list, &all, &sites, "https://c.org/posts/1").is_none());
    }

    #[test]
    fn picks_take_ranges_and_leave_out_what_is_saved() {
        let list: Vec<(String, bool)> = (0..6).map(|i| (format!("p{i}"), i != 3)).collect();
        let pages: Vec<String> = list.iter().map(|(p, _)| p.clone()).collect();
        let count = |picks: &Picks| picks.count_in(&pages);
        let mut picks = Picks::default();
        picks.toggle(&list, 1, false);
        assert!(picks.contains("p1"));
        // Shift picks the range, but not the one already saved.
        picks.toggle(&list, 5, true);
        assert_eq!(count(&picks), 4);
        assert!(!picks.contains("p3"));
        // A saved one can't be clicked.
        picks.toggle(&list, 3, false);
        assert_eq!(count(&picks), 4);
        // Shift on a picked one takes the range off.
        picks.toggle(&list, 4, true);
        assert_eq!(count(&picks), 2);
        assert!(picks.contains("p1") && picks.contains("p2"));
        picks.toggle(&list, 0, false);
        let free: Vec<String> = list
            .iter()
            .filter(|(_, f)| *f)
            .map(|(p, _)| p.clone())
            .collect();
        assert!(!picks.all_of(&free));
        // From the one clicked last, whichever way.
        picks.toggle(&list, 5, true);
        assert!(picks.all_of(&free));
        picks.toggle_all(&free);
        assert_eq!(count(&picks), 0);
        picks.toggle_all(&free);
        assert!(picks.all_of(&free));
        picks.clear();
        assert!(!picks.all_of(&[]));
        picks.toggle(&list, 9, false);
        assert_eq!(count(&picks), 0);
    }

    #[test]
    fn a_range_starts_at_the_post_clicked_last_wherever_it_went() {
        let list = |pages: &[&str]| -> Vec<(String, bool)> {
            pages.iter().map(|p| ((*p).to_owned(), true)).collect()
        };
        let before = list(&["a", "b", "c", "d", "e"]);
        let mut picks = Picks::default();
        picks.toggle(&before, 3, false);
        // A twin hidden above it moves "d" up one place.
        let after = list(&["a", "c", "d", "e"]);
        picks.toggle(&after, 0, true);
        let pages: Vec<String> = after.iter().map(|(p, _)| p.clone()).collect();
        assert_eq!(picks.count_in(&pages), 3);
        assert!(["a", "c", "d"].iter().all(|p| picks.contains(p)));
        assert!(!picks.contains("e"));
        // The last one left the list: Shift picks just the one clicked.
        let mut picks = Picks::default();
        picks.toggle(&before, 1, false);
        picks.toggle(&after, 3, true);
        assert!(picks.contains("e") && !picks.contains("c") && !picks.contains("d"));
        // Picks the list no longer shows don't count.
        assert_eq!(picks.count_in(&pages), 1);
        assert_eq!(picks.count_in(&[]), 0);
    }
}
