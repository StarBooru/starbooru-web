//! The server's import jobs, as the app follows them: how often to ask about
//! them, the list after an answer, and when the ones started here are all
//! finished.

use std::collections::{HashMap, HashSet};

use starbooru_core::{ImportJob, JobState};

/// Seconds between asking while a download is going (its progress shows),
/// while some wait, and otherwise.
pub const POLL_RUNNING: u32 = 1;
pub const POLL_BUSY: u32 = 2;
pub const POLL_IDLE: u32 = 60;

/// How often to ask for the job list, given what is on it.
pub fn poll_every(jobs: &[ImportJob]) -> u32 {
    if jobs.iter().any(|j| j.state == JobState::Running) {
        POLL_RUNNING
    } else if jobs.iter().any(|j| j.state.is_active()) {
        POLL_BUSY
    } else {
        POLL_IDLE
    }
}

/// `old` with `new` jobs added or put in place, newest first.
pub fn merge(old: &[ImportJob], new: Vec<ImportJob>) -> Vec<ImportJob> {
    let mut by_id: HashMap<i64, ImportJob> = old.iter().map(|j| (j.id, j.clone())).collect();
    for job in new {
        by_id.insert(job.id, job);
    }
    let mut list: Vec<ImportJob> = by_id.into_values().collect();
    list.sort_by_key(|j| std::cmp::Reverse(j.id));
    list
}

/// How the watched jobs ended.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ended {
    pub done: usize,
    pub duplicate: usize,
    pub failed: usize,
}

/// Adds `now`'s waiting and running jobs to `watched`. Once none of them is
/// left going: how the watched ones ended, and `watched` starts over.
pub fn finished(watched: &mut HashSet<i64>, now: &[ImportJob]) -> Option<Ended> {
    watched.extend(now.iter().filter(|j| j.state.is_active()).map(|j| j.id));
    if watched.is_empty() || now.iter().any(|j| j.state.is_active()) {
        return None;
    }
    let states: HashMap<i64, JobState> = now.iter().map(|j| (j.id, j.state)).collect();
    let mut ended = Ended::default();
    for id in watched.drain() {
        match states.get(&id) {
            Some(JobState::Done) => ended.done += 1,
            Some(JobState::Duplicate) => ended.duplicate += 1,
            Some(JobState::Failed) => ended.failed += 1,
            // Cancelled, or cleared elsewhere.
            _ => {}
        }
    }
    (ended != Ended::default()).then_some(ended)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use starbooru_core::Rating;

    use super::*;

    fn job(id: i64, state: JobState) -> ImportJob {
        ImportJob {
            id,
            url: format!("https://example.org/{id}.png"),
            source: None,
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

    #[test]
    fn asks_more_often_while_a_download_shows_its_progress() {
        assert_eq!(poll_every(&[]), POLL_IDLE);
        assert_eq!(poll_every(&[job(1, JobState::Done)]), POLL_IDLE);
        assert_eq!(poll_every(&[job(1, JobState::Queued)]), POLL_BUSY);
        let jobs = [job(1, JobState::Queued), job(2, JobState::Running)];
        assert_eq!(poll_every(&jobs), POLL_RUNNING);
    }

    #[test]
    fn merge_replaces_by_id_newest_first() {
        let old = [job(1, JobState::Running), job(2, JobState::Queued)];
        let list = merge(&old, vec![job(2, JobState::Done), job(3, JobState::Queued)]);
        let ids: Vec<i64> = list.iter().map(|j| j.id).collect();
        assert_eq!(ids, [3, 2, 1]);
        assert_eq!(list[1].state, JobState::Done);
    }

    #[test]
    fn counts_only_when_everything_finished() {
        let mut watched = HashSet::from([3]);
        let first = [
            job(1, JobState::Running),
            job(2, JobState::Queued),
            job(3, JobState::Failed),
            job(4, JobState::Done),
        ];
        assert_eq!(finished(&mut watched, &first), None);
        let still = [job(1, JobState::Done), job(2, JobState::Running)];
        assert_eq!(finished(&mut watched, &still), None);
        let now = [
            job(1, JobState::Done),
            job(2, JobState::Duplicate),
            job(3, JobState::Failed),
            job(4, JobState::Done),
        ];
        assert_eq!(
            finished(&mut watched, &now),
            Some(Ended {
                done: 1,
                duplicate: 1,
                failed: 1
            })
        );
        assert!(watched.is_empty());
        // Nothing was going since: nothing to say.
        assert_eq!(finished(&mut watched, &now), None);
        // A watched job cancelled or cleared elsewhere isn't counted.
        let mut watched = HashSet::from([9]);
        assert_eq!(finished(&mut watched, &[]), None);
    }
}
