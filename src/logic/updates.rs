//! Updates without a browser: when the shell's notice shows and which
//! version each account hid it for, what Administration › Updates offers,
//! what each step of an update is called, why a server can't update itself
//! and what to do instead, the hour of automatic updates on the server's
//! clock, and how the page tells the server came back after its restart.

use std::fmt::Display;

use chrono::{DateTime, FixedOffset, TimeDelta, TimeZone, Timelike, Utc};
use starbooru_core::{
    UpdateBlock, UpdateKind, UpdateMethod, UpdateRun, UpdateState, UpdateStatus, Version,
};

use super::admin::{Tab, sentence};
use super::viewer::when;
use crate::route;
use crate::store::DismissedUpdate;

/// How often the shell asks again while the app stays open. The server asks
/// GitHub every 6 hours.
pub const NOTICE_EVERY: TimeDelta = TimeDelta::hours(3);
/// How often the shell looks whether it's time to ask, or someone else
/// signed in.
pub const NOTICE_ROUND_MS: i32 = 60_000;
/// How often the page asks while an update runs.
pub const RUN_POLL_MS: i32 = 1500;
/// How often the page knocks while the server restarts.
pub const RESTART_POLL_MS: i32 = 2000;
/// A restart longer than this gets a word on the page.
pub const SLOW_RESTART: TimeDelta = TimeDelta::minutes(3);
/// How long "Updated" shows before the page reloads.
pub const RELOAD_AFTER_MS: i32 = 1800;
/// The server takes one "Check now" a minute.
pub const CHECK_GAP_SECS: u64 = 60;
/// Accounts whose hidden notice is remembered; the oldest goes first.
const MAX_DISMISSED: usize = 8;

/// Updating a container by hand, where its compose file is.
pub const UPDATE_COMMAND: &str = "docker compose pull && docker compose up -d";

// ---------- the notice ----------

/// The version the shell's notice is about: a newer release, unless this
/// account hid the notice for it or for a later one.
pub fn notice(status: &UpdateStatus, dismissed: Option<&str>) -> Option<String> {
    let latest = status.latest.as_ref().filter(|_| status.available)?;
    let hidden =
        dismissed.is_some_and(
            |d| match (Version::parse(d), Version::parse(&latest.version)) {
                (Some(d), Some(latest)) => d >= latest,
                _ => d == latest.version,
            },
        );
    (!hidden).then(|| latest.version.clone())
}

/// The version `owner` hid the notice for.
pub fn dismissed<'a>(list: &'a [DismissedUpdate], owner: &str) -> Option<&'a str> {
    list.iter()
        .find(|d| d.owner == owner)
        .map(|d| d.version.as_str())
}

/// Puts `version` as the one `owner` hid the notice for, or with `None`
/// forgets it. The latest change goes last.
pub fn set_dismissed(list: &mut Vec<DismissedUpdate>, owner: &str, version: Option<String>) {
    list.retain(|d| d.owner != owner);
    if let Some(version) = version {
        list.push(DismissedUpdate {
            owner: owner.to_owned(),
            version,
        });
    }
    let extra = list.len().saturating_sub(MAX_DISMISSED);
    list.drain(..extra);
}

/// Pages the notice keeps off: the viewers, and the page it leads to.
pub fn quiet_on(path: &str, tab: Option<&str>) -> bool {
    let path = path.trim_end_matches('/');
    path.starts_with("/post/")
        || path == "/boorus/post"
        || (path == route::ADMIN && Tab::parse(tab) == Tab::Updates)
}

/// Whether it's time to ask again, having asked at `asked`.
pub fn due(asked: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    asked.is_none_or(|at| now - at >= NOTICE_EVERY || at > now)
}

// ---------- what the page offers ----------

/// What the update card offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Offer {
    /// An update is under way.
    Running,
    /// "Update to …".
    Update(String),
    /// A newer version that has to be installed by hand.
    ByHand(String),
    /// The newest known version runs.
    UpToDate,
    /// Nothing known yet.
    Unknown,
}

pub fn offer(status: &UpdateStatus) -> Offer {
    if status.run.as_ref().is_some_and(|r| r.state.is_running()) {
        return Offer::Running;
    }
    match status.latest.as_ref() {
        Some(latest) if status.available && status.method.can_update => {
            Offer::Update(latest.version.clone())
        }
        Some(latest) if status.available => Offer::ByHand(latest.version.clone()),
        Some(_) => Offer::UpToDate,
        None => Offer::Unknown,
    }
}

/// Words for an update the server wouldn't start, by its error code, when
/// it's one the page knows.
pub fn install_refusal(code: Option<&str>) -> Option<String> {
    match code? {
        "up_to_date" => Some(t!("updates-error-up-to-date")),
        "cannot_update" => Some(t!("updates-error-cannot")),
        "backup_unavailable" => Some(t!("updates-error-no-backup")),
        "update_running" => Some(t!("updates-error-running")),
        _ => None,
    }
}

// ---------- the steps ----------

/// The steps of an update, in the order the server goes through them.
pub const STEPS: [UpdateState; 4] = [
    UpdateState::Checking,
    UpdateState::Downloading,
    UpdateState::BackingUp,
    UpdateState::Restarting,
];

/// How far a step is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mark {
    Done,
    Now,
    Next,
    /// Left out, as the backup when it was turned off.
    Skipped,
}

/// The steps a server of `kind` goes through. Swarm pulls the image itself
/// as it replaces the service, so there's no download of the server's own.
pub fn plan(kind: UpdateKind) -> Vec<UpdateState> {
    STEPS
        .into_iter()
        .filter(|s| !(kind == UpdateKind::Swarm && *s == UpdateState::Downloading))
        .collect()
}

/// Each step of `plan` and how far it is. `no_backup` says the backup was
/// turned off for this update, when this page started it; otherwise a backup
/// the run passed without making was left out.
pub fn steps(run: &UpdateRun, kind: UpdateKind, no_backup: bool) -> Vec<(UpdateState, Mark)> {
    let plan = plan(kind);
    let at = match run.state {
        UpdateState::Done | UpdateState::Failed => plan.len(),
        state => plan.iter().position(|s| *s == state).unwrap_or(0),
    };
    plan.iter()
        .enumerate()
        .map(|(i, step)| {
            let backup = *step == UpdateState::BackingUp;
            let mark = if backup && (no_backup || (i < at && run.backup.is_none())) {
                Mark::Skipped
            } else if i < at {
                Mark::Done
            } else if i == at {
                Mark::Now
            } else {
                Mark::Next
            };
            (*step, mark)
        })
        .collect()
}

/// A step's name.
pub fn state_label(state: UpdateState) -> String {
    match state {
        UpdateState::Checking => t!("updates-step-checking"),
        UpdateState::Downloading => t!("updates-step-downloading"),
        UpdateState::BackingUp => t!("updates-step-backing-up"),
        UpdateState::Restarting => t!("updates-step-restarting"),
        UpdateState::Done => t!("updates-step-done"),
        UpdateState::Failed => t!("updates-step-failed"),
    }
}

/// What a step means while it runs, for the update to `to`.
pub fn state_note(state: UpdateState, to: &str) -> String {
    let to = to.to_owned();
    match state {
        UpdateState::Checking => t!("updates-step-checking-note", version = to),
        UpdateState::Downloading => t!("updates-step-downloading-note", version = to),
        UpdateState::BackingUp => t!("updates-step-backing-up-note"),
        UpdateState::Restarting => t!("updates-step-restarting-note"),
        UpdateState::Done => t!("updates-step-done-note", version = to),
        UpdateState::Failed => t!("updates-step-failed-note"),
    }
}

// ---------- after the restart ----------

/// What the server says after its restart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Back {
    /// The new version answers.
    Updated,
    /// The update failed, and the old version runs again.
    Failed,
    /// Away, or still on its way.
    Waiting,
}

/// `version` is what `/info` answered, `status` the update status when it
/// could be read, and `to` the version being installed.
pub fn back(to: &str, version: Option<&str>, status: Option<&UpdateStatus>) -> Back {
    let same = |v: &str| match (Version::parse(v), Version::parse(to)) {
        (Some(a), Some(b)) => a == b,
        _ => v == to,
    };
    if version.is_some_and(same) {
        return Back::Updated;
    }
    match status.and_then(|s| s.run.as_ref()).filter(|r| same(&r.to)) {
        Some(run) if run.state == UpdateState::Done => Back::Updated,
        Some(run) if run.state == UpdateState::Failed => Back::Failed,
        _ => Back::Waiting,
    }
}

/// The last update's result, in `zone`: how it went and the line saying so.
pub fn run_result<Tz: TimeZone>(run: &UpdateRun, current: &str, zone: &Tz) -> Option<(bool, String)>
where
    Tz::Offset: Display,
{
    let at = when(&run.finished_at.unwrap_or(run.started_at), zone);
    let (from, to) = (run.from.clone(), run.to.clone());
    match run.state {
        UpdateState::Done if run.automatic => Some((
            true,
            t!("updates-result-done-auto", from = from, to = to, time = at),
        )),
        UpdateState::Done => Some((
            true,
            t!("updates-result-done", from = from, to = to, time = at),
        )),
        UpdateState::Failed => {
            let why = run
                .error
                .as_deref()
                .map_or_else(|| t!("updates-result-no-reason"), sentence);
            let current = current.to_owned();
            Some((
                false,
                if run.automatic {
                    t!(
                        "updates-result-failed-auto",
                        to = to,
                        time = at,
                        reason = why,
                        current = current
                    )
                } else {
                    t!(
                        "updates-result-failed",
                        to = to,
                        time = at,
                        reason = why,
                        current = current
                    )
                },
            ))
        }
        _ => None,
    }
}

/// A release's title, when it says more than its version: GitHub's
/// "StarBooru server 0.3.0" doesn't.
pub fn release_title(name: &str, version: &str) -> Option<String> {
    let name = name.trim();
    let bare = version.trim().trim_start_matches(['v', 'V']);
    let rest = name
        .to_lowercase()
        .replace(&bare.to_lowercase(), " ")
        .replace("starbooru", " ")
        .replace("server", " ")
        .replace("release", " ");
    let says_more = rest
        .split(|c: char| !c.is_alphanumeric())
        .any(|w| !w.is_empty() && w != "v");
    says_more.then(|| name.to_owned())
}

// ---------- updating by hand ----------

/// Why the server can't update itself, and what to do instead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ByHand {
    pub why: String,
    pub how: String,
    /// A command to copy.
    pub command: Option<&'static str>,
    /// Dokploy's Redeploy does it too.
    pub redeploy: bool,
    /// Giving the container the Docker socket would let it update itself.
    pub socket: bool,
}

pub fn by_hand(method: &UpdateMethod) -> ByHand {
    let compose = matches!(
        method.reason,
        Some(UpdateBlock::NoDockerSocket | UpdateBlock::DockerUnreachable)
    ) && method.kind == UpdateKind::Docker;
    let why = match method.reason {
        Some(UpdateBlock::NoDockerSocket) => t!("updates-why-no-docker-socket"),
        Some(UpdateBlock::DockerUnreachable) => t!("updates-why-docker-unreachable"),
        Some(UpdateBlock::UnknownImage) => t!("updates-why-unknown-image"),
        Some(UpdateBlock::RemovedOnStop) => t!("updates-why-removed-on-stop"),
        Some(UpdateBlock::DevelopmentBuild) => t!("updates-why-development-build"),
        Some(UpdateBlock::NotSupervised) => t!("updates-why-not-supervised"),
        Some(UpdateBlock::ReadOnlyFolder) => t!("updates-why-read-only-folder"),
        Some(UpdateBlock::Unsupported) => t!("updates-why-unsupported"),
        None => t!("updates-why-unknown"),
    };
    let how = match (method.kind, method.reason) {
        _ if compose => t!("updates-how-compose"),
        (UpdateKind::Swarm, _) => t!("updates-how-swarm"),
        (_, Some(UpdateBlock::UnknownImage)) => t!("updates-how-image"),
        (_, Some(UpdateBlock::RemovedOnStop)) => t!("updates-how-removed-on-stop"),
        (_, Some(UpdateBlock::DevelopmentBuild)) => t!("updates-how-source"),
        (_, Some(UpdateBlock::NotSupervised)) => t!("updates-how-systemd"),
        (_, Some(UpdateBlock::ReadOnlyFolder)) => t!("updates-how-folder"),
        _ => t!("updates-how-manual"),
    };
    ByHand {
        why,
        how,
        command: compose.then_some(UPDATE_COMMAND),
        redeploy: compose || method.kind == UpdateKind::Swarm,
        socket: method.reason == Some(UpdateBlock::NoDockerSocket),
    }
}

/// A container whose compose file names its version: the new version must
/// go there too, or a redeploy brings back the old one.
pub fn pinned_note(status: &UpdateStatus) -> Option<String> {
    if !status.method.pinned {
        return None;
    }
    Some(match status.latest.as_ref().filter(|_| status.available) {
        Some(latest) => t!("updates-pinned-before", version = latest.version.clone()),
        None => t!("updates-pinned-after", version = status.current.clone()),
    })
}

// ---------- the hour ----------

/// An hour of the day, as clocks show it: `04:00`.
pub fn hour_label(hour: u8) -> String {
    format!("{:02}:00", hour.min(23))
}

/// A time zone by its offset: `UTC`, `UTC+01:00`, `UTC−05:00`.
pub fn zone_label(offset: FixedOffset) -> String {
    let secs = offset.local_minus_utc();
    if secs == 0 {
        return "UTC".to_owned();
    }
    let sign = if secs < 0 { '−' } else { '+' };
    let minutes = secs.unsigned_abs() / 60;
    format!("UTC{sign}{:02}:{:02}", minutes / 60, minutes % 60)
}

/// `hour` on a clock at `from`, as a clock at `to` shows it at that moment:
/// `05:00`, or `09:30` across a half-hour zone. `None` when they agree.
pub fn hour_elsewhere(
    hour: u8,
    from: FixedOffset,
    to: FixedOffset,
    on: DateTime<Utc>,
) -> Option<String> {
    if from == to {
        return None;
    }
    let day = on.with_timezone(&from).date_naive();
    let there = day
        .and_hms_opt(u32::from(hour.min(23)), 0, 0)?
        .and_local_timezone(from)
        .single()?
        .with_timezone(&to);
    Some(format!("{:02}:{:02}", there.hour(), there.minute()))
}

/// The hours to pick from, `0` to `23`.
pub fn hours() -> impl Iterator<Item = u8> {
    0..24
}

// ---------- Check now ----------

/// Seconds to wait before checking again, from the server's `Retry-After`.
pub fn check_wait(retry_after: Option<u64>) -> u64 {
    retry_after
        .unwrap_or(CHECK_GAP_SECS)
        .clamp(1, CHECK_GAP_SECS)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use starbooru_core::{Release, UpdateSettings};

    use super::*;

    fn status(current: &str, latest: Option<&str>) -> UpdateStatus {
        let available = latest.is_some_and(|l| starbooru_core::is_newer(l, current));
        UpdateStatus {
            current: current.into(),
            latest: latest.map(|v| Release {
                version: v.into(),
                name: format!("StarBooru server {v}"),
                published_at: None,
                notes: String::new(),
                url: String::new(),
            }),
            available,
            checked_at: None,
            check_error: None,
            settings: UpdateSettings {
                check: true,
                auto: false,
                auto_hour: 4,
            },
            method: method(UpdateKind::Docker, None),
            server_time: Utc
                .with_ymd_and_hms(2026, 11, 24, 0, 0, 0)
                .unwrap()
                .fixed_offset(),
            run: None,
        }
    }

    fn method(kind: UpdateKind, reason: Option<UpdateBlock>) -> UpdateMethod {
        UpdateMethod {
            kind,
            can_update: reason.is_none(),
            reason,
            message: reason.map(|_| "in English".into()),
            image: None,
            pinned: false,
        }
    }

    fn run(state: UpdateState, backup: Option<&str>) -> UpdateRun {
        UpdateRun {
            state,
            from: "0.2.0".into(),
            to: "0.3.0".into(),
            automatic: false,
            started_at: Utc.with_ymd_and_hms(2026, 11, 24, 3, 0, 12).unwrap(),
            finished_at: None,
            error: None,
            backup: backup.map(str::to_owned),
        }
    }

    #[test]
    fn the_notice_shows_a_newer_version_until_it_is_hidden() {
        let s = status("0.2.0", Some("0.3.0"));
        assert_eq!(notice(&s, None), Some("0.3.0".into()));
        assert_eq!(notice(&s, Some("0.3.0")), None);
        assert_eq!(notice(&s, Some("v0.3.0")), None);
        // Hidden for an older one: a newer release shows again.
        assert_eq!(notice(&s, Some("0.2.5")), Some("0.3.0".into()));
        assert_eq!(notice(&s, Some("0.4.0")), None);
        // Whatever couldn't be read hides only that same text.
        assert_eq!(notice(&s, Some("main")), Some("0.3.0".into()));
        // Nothing newer, or nothing known.
        assert_eq!(notice(&status("0.3.0", Some("0.3.0")), None), None);
        assert_eq!(notice(&status("0.3.0", Some("0.2.0")), None), None);
        assert_eq!(notice(&status("0.2.0", None), None), None);
    }

    #[test]
    fn each_account_hides_its_own_notice() {
        let mut list = Vec::new();
        set_dismissed(&mut list, "a|1", Some("0.3.0".into()));
        set_dismissed(&mut list, "a|2", Some("0.2.0".into()));
        set_dismissed(&mut list, "a|1", Some("0.4.0".into()));
        assert_eq!(dismissed(&list, "a|1"), Some("0.4.0"));
        assert_eq!(dismissed(&list, "a|2"), Some("0.2.0"));
        assert_eq!(dismissed(&list, "b|1"), None);
        assert_eq!(list.len(), 2);
        set_dismissed(&mut list, "a|1", None);
        assert_eq!(dismissed(&list, "a|1"), None);
        // Only so many accounts are kept: the oldest goes.
        for i in 0..20 {
            set_dismissed(&mut list, &format!("s|{i}"), Some("1.0.0".into()));
        }
        assert_eq!(list.len(), MAX_DISMISSED);
        assert_eq!(dismissed(&list, "s|19"), Some("1.0.0"));
        assert_eq!(dismissed(&list, "s|0"), None);
    }

    #[test]
    fn the_notice_keeps_off_the_viewers_and_its_own_page() {
        for (path, tab) in [
            ("/post/12", None),
            ("/boorus/post", None),
            ("/admin", Some("updates")),
            ("/admin/", Some("updates")),
        ] {
            assert!(quiet_on(path, tab), "{path} {tab:?}");
        }
        for (path, tab) in [
            ("/", None),
            ("/settings", None),
            ("/admin", None),
            ("/admin", Some("server")),
            ("/boorus", None),
            ("/posts", Some("updates")),
        ] {
            assert!(!quiet_on(path, tab), "{path} {tab:?}");
        }
    }

    #[test]
    fn the_shell_asks_every_few_hours() {
        let now = Utc.with_ymd_and_hms(2026, 11, 24, 12, 0, 0).unwrap();
        assert!(due(None, now));
        assert!(!due(Some(now - TimeDelta::minutes(30)), now));
        assert!(due(Some(now - NOTICE_EVERY), now));
        // A clock that went back doesn't hold it off for hours.
        assert!(due(Some(now + TimeDelta::hours(1)), now));
    }

    #[test]
    fn the_card_offers_what_the_server_can_do() {
        assert_eq!(
            offer(&status("0.2.0", Some("0.3.0"))),
            Offer::Update("0.3.0".into())
        );
        let mut s = status("0.2.0", Some("0.3.0"));
        s.method = method(UpdateKind::Docker, Some(UpdateBlock::NoDockerSocket));
        assert_eq!(offer(&s), Offer::ByHand("0.3.0".into()));
        s.run = Some(run(UpdateState::Downloading, None));
        assert_eq!(offer(&s), Offer::Running);
        // A finished run leaves the offer as it was.
        s.run = Some(run(UpdateState::Failed, None));
        assert_eq!(offer(&s), Offer::ByHand("0.3.0".into()));
        assert_eq!(offer(&status("0.3.0", Some("0.3.0"))), Offer::UpToDate);
        assert_eq!(offer(&status("0.3.0", None)), Offer::Unknown);
    }

    #[test]
    fn a_refused_update_says_why_in_words() {
        for code in [
            "up_to_date",
            "cannot_update",
            "backup_unavailable",
            "update_running",
        ] {
            assert!(
                install_refusal(Some(code)).is_some_and(|w| !w.is_empty()),
                "{code}"
            );
        }
        assert!(
            install_refusal(Some("backup_unavailable"))
                .unwrap()
                .contains("Back up the database first")
        );
        assert_eq!(install_refusal(Some("internal")), None);
        assert_eq!(install_refusal(None), None);
    }

    #[test]
    fn steps_go_in_order_and_a_skipped_backup_says_so() {
        let marks = |r: &UpdateRun, no_backup| {
            steps(r, UpdateKind::Docker, no_backup)
                .into_iter()
                .map(|(_, m)| m)
                .collect::<Vec<_>>()
        };
        use Mark::{Done, Next, Now, Skipped};
        assert_eq!(
            marks(&run(UpdateState::Checking, None), false),
            [Now, Next, Next, Next]
        );
        assert_eq!(
            marks(&run(UpdateState::BackingUp, None), false),
            [Done, Done, Now, Next]
        );
        assert_eq!(
            marks(&run(UpdateState::Restarting, Some("b.dump")), false),
            [Done, Done, Done, Now]
        );
        // Passed without a backup, or turned off from the start.
        assert_eq!(
            marks(&run(UpdateState::Restarting, None), false),
            [Done, Done, Skipped, Now]
        );
        assert_eq!(
            marks(&run(UpdateState::Downloading, None), true),
            [Done, Now, Skipped, Next]
        );
        assert_eq!(
            marks(&run(UpdateState::Done, Some("b.dump")), false),
            [Done, Done, Done, Done]
        );
        // Swarm goes from getting ready straight to the backup.
        assert_eq!(
            steps(&run(UpdateState::BackingUp, None), UpdateKind::Swarm, false),
            [
                (UpdateState::Checking, Done),
                (UpdateState::BackingUp, Now),
                (UpdateState::Restarting, Next)
            ]
        );
        assert_eq!(plan(UpdateKind::Package).len(), 4);
        let names: Vec<_> = STEPS.iter().map(|s| state_label(*s)).collect();
        assert_eq!(
            names,
            ["Getting ready", "Downloading", "Backing up", "Restarting"]
        );
        assert!(state_note(UpdateState::Downloading, "0.3.0").contains("0.3.0"));
        assert!(!state_note(UpdateState::Restarting, "0.3.0").is_empty());
    }

    #[test]
    fn the_page_knows_when_the_new_version_is_back() {
        assert_eq!(back("0.3.0", None, None), Back::Waiting);
        assert_eq!(back("0.3.0", Some("0.2.0"), None), Back::Waiting);
        assert_eq!(back("0.3.0", Some("0.3.0"), None), Back::Updated);
        assert_eq!(back("0.3.0", Some("v0.3.0"), None), Back::Updated);
        let mut s = status("0.2.0", Some("0.3.0"));
        s.run = Some(run(UpdateState::Restarting, None));
        assert_eq!(back("0.3.0", Some("0.2.0"), Some(&s)), Back::Waiting);
        s.run = Some(run(UpdateState::Failed, None));
        assert_eq!(back("0.3.0", Some("0.2.0"), Some(&s)), Back::Failed);
        s.run = Some(run(UpdateState::Done, None));
        assert_eq!(back("0.3.0", None, Some(&s)), Back::Updated);
        // A run for another version says nothing of this one.
        assert_eq!(back("0.4.0", Some("0.2.0"), Some(&s)), Back::Waiting);
    }

    #[test]
    fn the_last_run_says_how_it_went() {
        let utc = FixedOffset::east_opt(0).unwrap();
        let mut done = run(UpdateState::Done, Some("b.dump"));
        done.finished_at = Some(Utc.with_ymd_and_hms(2026, 11, 24, 3, 1, 40).unwrap());
        assert_eq!(
            run_result(&done, "0.3.0", &utc),
            Some((
                true,
                "Updated from 0.2.0 to 0.3.0 on 2026-11-24 03:01.".into()
            ))
        );
        done.automatic = true;
        assert!(
            run_result(&done, "0.3.0", &utc)
                .unwrap()
                .1
                .contains("automatically")
        );
        let mut failed = run(UpdateState::Failed, None);
        failed.error = Some("the new container didn't become healthy".into());
        let (ok, line) = run_result(&failed, "0.2.0", &utc).unwrap();
        assert!(!ok);
        assert!(
            line.contains("The new container didn't become healthy."),
            "{line}"
        );
        assert!(line.contains("0.2.0"), "{line}");
        assert_eq!(
            run_result(&run(UpdateState::Restarting, None), "0.2.0", &utc),
            None
        );
    }

    #[test]
    fn a_release_title_shows_when_it_says_more_than_the_version() {
        assert_eq!(release_title("StarBooru server 0.3.0", "0.3.0"), None);
        assert_eq!(release_title("v0.3.0", "0.3.0"), None);
        assert_eq!(release_title("Release v0.3.0", "v0.3.0"), None);
        assert_eq!(release_title("  ", "0.3.0"), None);
        assert_eq!(
            release_title("0.3.0: Public mode", "0.3.0"),
            Some("0.3.0: Public mode".into())
        );
        assert_eq!(
            release_title("StarBooru 0.3.0 Comet", "0.3.0"),
            Some("StarBooru 0.3.0 Comet".into())
        );
    }

    #[test]
    fn each_reason_has_its_words_and_its_way_round() {
        let socket = by_hand(&method(
            UpdateKind::Docker,
            Some(UpdateBlock::NoDockerSocket),
        ));
        assert_eq!(socket.command, Some(UPDATE_COMMAND));
        assert!(socket.redeploy && socket.socket);
        let unreachable = by_hand(&method(
            UpdateKind::Docker,
            Some(UpdateBlock::DockerUnreachable),
        ));
        assert_eq!(unreachable.command, Some(UPDATE_COMMAND));
        assert!(!unreachable.socket);
        // `docker run --rm` and local images have no compose file to pull.
        for reason in [UpdateBlock::RemovedOnStop, UpdateBlock::UnknownImage] {
            let b = by_hand(&method(UpdateKind::Docker, Some(reason)));
            assert_eq!(b.command, None, "{reason:?}");
            assert!(!b.redeploy, "{reason:?}");
        }
        let dev = by_hand(&method(
            UpdateKind::Manual,
            Some(UpdateBlock::DevelopmentBuild),
        ));
        assert_eq!(dev.command, None);
        assert!(!dev.redeploy);
        let swarm = by_hand(&method(
            UpdateKind::Swarm,
            Some(UpdateBlock::DockerUnreachable),
        ));
        assert_eq!(swarm.command, None);
        assert!(swarm.redeploy);
        // Every reason says something of its own, never the server's English.
        let reasons = [
            UpdateBlock::NoDockerSocket,
            UpdateBlock::DockerUnreachable,
            UpdateBlock::UnknownImage,
            UpdateBlock::RemovedOnStop,
            UpdateBlock::DevelopmentBuild,
            UpdateBlock::NotSupervised,
            UpdateBlock::ReadOnlyFolder,
            UpdateBlock::Unsupported,
        ];
        let whys: std::collections::HashSet<_> = reasons
            .iter()
            .map(|r| by_hand(&method(UpdateKind::Package, Some(*r))).why)
            .collect();
        assert_eq!(whys.len(), reasons.len());
        assert!(!whys.contains("in English"));
        assert!(!by_hand(&method(UpdateKind::Manual, None)).why.is_empty());
    }

    #[test]
    fn a_pinned_version_asks_for_the_compose_file_to_change() {
        let mut s = status("0.2.0", Some("0.3.0"));
        assert_eq!(pinned_note(&s), None);
        s.method.pinned = true;
        let before = pinned_note(&s).unwrap();
        assert!(
            before.contains("0.3.0") && before.contains("STARBOORU_VERSION"),
            "{before}"
        );
        // Updated from here: the compose file still names the old one.
        let mut s = status("0.3.0", Some("0.3.0"));
        s.method.pinned = true;
        let after = pinned_note(&s).unwrap();
        assert!(after.contains("0.3.0") && after != before, "{after}");
    }

    #[test]
    fn the_hour_reads_on_the_servers_clock_and_here() {
        assert_eq!(hour_label(4), "04:00");
        assert_eq!(hour_label(23), "23:00");
        let utc = FixedOffset::east_opt(0).unwrap();
        let rome = FixedOffset::east_opt(3600).unwrap();
        let delhi = FixedOffset::east_opt(5 * 3600 + 1800).unwrap();
        let new_york = FixedOffset::west_opt(5 * 3600).unwrap();
        assert_eq!(zone_label(utc), "UTC");
        assert_eq!(zone_label(rome), "UTC+01:00");
        assert_eq!(zone_label(delhi), "UTC+05:30");
        assert_eq!(zone_label(new_york), "UTC−05:00");
        let on = Utc.with_ymd_and_hms(2026, 11, 24, 12, 0, 0).unwrap();
        assert_eq!(hour_elsewhere(4, utc, utc, on), None);
        assert_eq!(hour_elsewhere(4, utc, rome, on), Some("05:00".into()));
        assert_eq!(hour_elsewhere(4, rome, utc, on), Some("03:00".into()));
        assert_eq!(hour_elsewhere(4, utc, delhi, on), Some("09:30".into()));
        assert_eq!(hour_elsewhere(2, utc, new_york, on), Some("21:00".into()));
        assert_eq!(hours().count(), 24);
    }

    #[test]
    fn a_check_too_soon_waits_what_the_server_says() {
        assert_eq!(check_wait(Some(41)), 41);
        assert_eq!(check_wait(None), CHECK_GAP_SECS);
        assert_eq!(check_wait(Some(0)), 1);
        assert_eq!(check_wait(Some(3600)), CHECK_GAP_SECS);
    }
}
