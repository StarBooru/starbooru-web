//! The administration without a browser: its tabs, storage limits typed in
//! GB, what the lists of accounts and backups say of each, the upload chart,
//! the checks of the server settings and the backup schedule, and the AI
//! panel's rules and requests.

use std::fmt::Display;

use chrono::{DateTime, TimeZone, Utc};
use starbooru_core::admin::{MAX_BACKUP_DAYS, MAX_BACKUP_KEEP, MAX_TRASH_DAYS};
use starbooru_core::ai::MAX_AI_FIELD_CHARS;
use starbooru_core::{
    AdminUser, AiAdminStatus, AiGeneralUpdate, AiMode, AiModelOption, AiPreset, AiTool,
    AiToolSettings, AiToolStatus, AiToolUpdate, Backup, BackupSchedule, DayCount, Hardware,
    ServerSettings, clean_general_update, clean_settings, clean_tool_update,
};

use super::library::ago;
use super::upload::human_size;
use super::viewer::when;
use crate::route;

// ---------- tabs ----------

/// The administration's tabs, as the reference had them. The AI tools stand
/// under Server.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Overview,
    Accounts,
    Backups,
    Server,
}

impl Tab {
    pub const ALL: [Tab; 4] = [Tab::Overview, Tab::Accounts, Tab::Backups, Tab::Server];

    pub fn as_str(self) -> &'static str {
        match self {
            Tab::Overview => "overview",
            Tab::Accounts => "accounts",
            Tab::Backups => "backups",
            Tab::Server => "server",
        }
    }

    /// The tab `?tab=` names; the overview for none or one we don't have.
    pub fn parse(value: Option<&str>) -> Tab {
        Tab::ALL
            .into_iter()
            .find(|t| value.is_some_and(|v| v.trim() == t.as_str()))
            .unwrap_or_default()
    }

    pub fn href(self) -> String {
        match self {
            Tab::Overview => route::ADMIN.to_owned(),
            other => format!("{}?tab={}", route::ADMIN, other.as_str()),
        }
    }
}

// ---------- the overview ----------

/// A size from the server, which counts in `u64`.
pub fn size(bytes: u64) -> String {
    human_size(bytes)
}

/// When the server started: "3 minutes ago", "yesterday", or its date.
pub fn started(at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    ago(at, now)
}

/// A day of the chart, as its axis and tooltips show it: `22/11`.
pub fn day_label(day: &DayCount) -> String {
    day.day.format("%d/%m").to_string()
}

/// One day's bar: its height as a share of the busiest day's, in percent.
#[derive(Clone, Debug, PartialEq)]
pub struct Bar {
    pub label: String,
    pub posts: u64,
    pub height: f64,
}

/// The uploads of the last days, as bars, with the first and last day for
/// the axis.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Chart {
    pub total: u64,
    pub bars: Vec<Bar>,
    pub first: String,
    pub last: String,
}

pub fn chart(days: &[DayCount]) -> Chart {
    // At least one, so an empty month draws flat bars instead of dividing by 0.
    let most = days.iter().map(|d| d.posts).max().unwrap_or(0).max(1);
    Chart {
        total: days
            .iter()
            .fold(0_u64, |sum, d| sum.saturating_add(d.posts)),
        bars: days
            .iter()
            .map(|d| Bar {
                label: day_label(d),
                posts: d.posts,
                height: d.posts as f64 / most as f64 * 100.0,
            })
            .collect(),
        first: days.first().map(day_label).unwrap_or_default(),
        last: days.last().map(day_label).unwrap_or_default(),
    }
}

// ---------- accounts ----------

const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
/// Largest storage limit the field takes, in GB.
pub const MAX_QUOTA_GB: f64 = 1_000_000.0;

/// A storage limit that isn't a size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BadQuota;

/// A storage limit as typed: GB unless it says otherwise (`5`, `2,5`,
/// `5 GB`, `500 MB`, `1 TB`). Empty is no limit.
pub fn parse_quota(text: &str) -> Result<Option<u64>, BadQuota> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    let lower = text.to_ascii_lowercase();
    let number_end = lower
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == ','))
        .unwrap_or(lower.len());
    let (number, unit) = lower.split_at(number_end);
    let scale = match unit.trim() {
        "" | "g" | "gb" | "gib" => 1.0,
        "m" | "mb" | "mib" => 1.0 / 1024.0,
        "k" | "kb" | "kib" => 1.0 / (1024.0 * 1024.0),
        "t" | "tb" | "tib" => 1024.0,
        _ => return Err(BadQuota),
    };
    let gb = number
        .replace(',', ".")
        .parse::<f64>()
        .map_err(|_| BadQuota)?
        * scale;
    if !gb.is_finite() || !(0.0..=MAX_QUOTA_GB).contains(&gb) {
        return Err(BadQuota);
    }
    Ok(Some((gb * GIB).round() as u64))
}

/// A limit in GB for the field: `1.5 GB`, `10 GB`, `0.25 GB`.
pub fn quota_text(bytes: u64) -> String {
    let text = format!("{:.2}", bytes as f64 / GIB);
    format!("{} GB", text.trim_end_matches('0').trim_end_matches('.'))
}

/// "3 posts · 1.5 KB of 1.0 GB", or without the limit when there is none.
pub fn usage_line(user: &AdminUser) -> String {
    let used = match user.quota_bytes {
        Some(quota) => t!("usage-of", used = size(user.bytes), quota = size(quota)),
        None => size(user.bytes),
    };
    format!("{} · {used}", t!("posts-count", count = user.posts))
}

/// What an account's row says under its name, in pieces that don't break
/// across lines: "3 posts · 1.5 KB", "active 3 minutes ago", "joined
/// 2026-11-22", the date as the clock in `zone` shows it.
pub fn account_parts<Tz: TimeZone>(user: &AdminUser, now: DateTime<Utc>, zone: &Tz) -> [String; 3]
where
    Tz::Offset: Display,
{
    let active = match user.last_active {
        Some(at) => t!("admin-active", when = ago(at, now)),
        None => t!("admin-never-active"),
    };
    let joined = user.created_at.with_timezone(zone).format("%Y-%m-%d");
    let joined = t!("admin-joined", date = joined.to_string());
    [usage_line(user), active, joined]
}

// ---------- server settings ----------

/// What the server settings' form says of what's typed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SettingsCheck {
    pub name_ok: bool,
    pub days_ok: bool,
    /// The settings to send, cleaned as the server cleans them, once every
    /// field is right.
    pub request: Option<ServerSettings>,
}

/// The name is 1–64 characters once its spaces are collapsed; the recycle
/// bin keeps posts 0–3650 days.
pub fn check_settings(
    name: &str,
    allow_registration: bool,
    trash_days: &str,
    auto_meta_tags: bool,
) -> SettingsCheck {
    let days = trash_days
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|d| *d <= MAX_TRASH_DAYS);
    let settings = |trash_days| ServerSettings {
        name: name.to_owned(),
        allow_registration,
        trash_days,
        auto_meta_tags,
    };
    // The days don't matter to the name's check.
    let name_ok = clean_settings(&settings(0)).is_ok();
    SettingsCheck {
        name_ok,
        days_ok: days.is_some(),
        request: days.and_then(|d| clean_settings(&settings(d)).ok()),
    }
}

// ---------- backups ----------

/// The command shown under "Restoring".
pub const RESTORE_COMMAND: &str =
    "pg_restore --clean --if-exists --no-owner -d \"$DATABASE_URL\" starbooru-….dump";

/// When a backup was made, as the clock in `zone` showed it.
pub fn backup_time<Tz: TimeZone>(backup: &Backup, zone: &Tz) -> String
where
    Tz::Offset: Display,
{
    when(&backup.created_at, zone)
}

/// "80.0 KB · starbooru-auto-20261122-233957.dump".
pub fn backup_line(backup: &Backup) -> String {
    format!("{} · {}", size(backup.bytes), backup.name)
}

/// What the schedule's form says of what's typed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScheduleCheck {
    pub every_ok: bool,
    pub keep_ok: bool,
    pub schedule: Option<BackupSchedule>,
}

/// Every 0–365 days (0 is off), keeping 1–100.
pub fn check_schedule(every: &str, keep: &str) -> ScheduleCheck {
    let every = every
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|d| *d <= MAX_BACKUP_DAYS);
    let keep = keep
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|k| (1..=MAX_BACKUP_KEEP).contains(k));
    let schedule = every
        .zip(keep)
        .map(|(every_days, keep)| BackupSchedule { every_days, keep })
        .filter(|s| s.check().is_ok());
    ScheduleCheck {
        every_ok: every.is_some(),
        keep_ok: keep.is_some(),
        schedule,
    }
}

// ---------- AI ----------

/// How often the AI panel looks while something changes by itself, and how
/// many of those steps it waits otherwise: 15 s.
pub const BUSY_POLL_MS: i32 = 1500;
pub const IDLE_ROUNDS: u32 = 10;

/// An answer time: `0.8 s`, `12 s`, `2 min`.
pub fn duration(ms: u32) -> String {
    if ms < 10_000 {
        format!("{:.1} s", f64::from(ms) / 1000.0)
    } else if ms < 90_000 {
        format!("{} s", ms / 1000)
    } else {
        format!("{} min", ms.saturating_add(30_000) / 60_000)
    }
}

/// `≈` marks a time estimated from how fast this machine is.
pub fn model_time(m: &AiModelOption) -> String {
    let time = duration(m.est_ms);
    if m.measured {
        time
    } else {
        format!("≈ {time}")
    }
}

pub fn percent(done: u64, total: u64) -> u64 {
    done.saturating_mul(100) / total.max(1)
}

/// One line about the machine: board or processor, threads, memory, GPUs.
pub fn hardware_line(hw: &Hardware) -> String {
    let mut parts = Vec::new();
    if let Some(name) = hw.board.as_ref().or(hw.cpu.as_ref()) {
        parts.push(name.clone());
    }
    parts.push(t!("admin-ai-hw-threads", count = hw.threads));
    if hw.ram_total > 0 {
        parts.push(t!("admin-ai-hw-memory", total = size(hw.ram_total)));
    }
    parts.extend(hw.gpus.iter().map(|gpu| gpu.name.clone()));
    parts.join(" · ")
}

/// The server's reason as a sentence: a capital first and a full stop.
pub fn sentence(text: &str) -> String {
    let text = text.trim();
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out: String = first.to_uppercase().chain(chars).collect();
    if !out.ends_with(['.', '!', '?']) {
        out.push('.');
    }
    out
}

/// A saved tool's setup, as the starting point of a change. The key stays
/// as it is.
pub fn update_of(s: &AiToolSettings) -> AiToolUpdate {
    AiToolUpdate {
        mode: s.mode,
        preset: s.preset,
        model: s.model.clone(),
        api_url: s.api_url.clone(),
        api_model: s.api_model.clone(),
        api_key: None,
    }
}

/// The preset in effect: Balanced stands in for one this machine can't meet.
pub fn effective_preset(tool: &AiToolStatus) -> AiPreset {
    let met = tool
        .presets
        .iter()
        .any(|p| p.preset == tool.settings.preset && p.available);
    if met {
        tool.settings.preset
    } else {
        AiPreset::Balanced
    }
}

/// The model a tool in local mode is meant to run: the admin's, or the
/// preset's.
pub fn wanted_model(tool: &AiToolStatus) -> Option<&AiModelOption> {
    let id = if tool.settings.model.is_empty() {
        let preset = effective_preset(tool);
        tool.presets
            .iter()
            .find(|p| p.preset == preset)
            .and_then(|p| p.model.as_deref())
            .or(tool.active.as_deref())?
    } else {
        tool.settings.model.as_str()
    };
    tool.models.iter().find(|m| m.id == id)
}

/// Whether something in the panel is about to change by itself: the
/// runtime starting, or a download.
pub fn busy(s: &AiAdminStatus) -> bool {
    let starting = !s.runtime.ready && s.runtime.reason.is_none();
    let downloading = s
        .tools
        .iter()
        .any(|t| t.models.iter().any(|m| m.progress.is_some()));
    starting || downloading
}

/// What picking a mode does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModeChoice {
    Send(AiToolUpdate),
    /// The API needs an address and a model before it can be saved.
    OpenForm,
}

pub fn choose_mode(settings: &AiToolSettings, mode: AiMode) -> ModeChoice {
    if mode == AiMode::Api && (settings.api_url.is_empty() || settings.api_model.is_empty()) {
        return ModeChoice::OpenForm;
    }
    ModeChoice::Send(AiToolUpdate {
        mode,
        ..update_of(settings)
    })
}

/// A preset replaces a model picked by hand.
pub fn preset_request(settings: &AiToolSettings, preset: AiPreset) -> AiToolUpdate {
    AiToolUpdate {
        preset,
        model: String::new(),
        ..update_of(settings)
    }
}

pub fn model_request(settings: &AiToolSettings, id: &str) -> AiToolUpdate {
    AiToolUpdate {
        model: id.to_owned(),
        ..update_of(settings)
    }
}

pub fn clear_key_request(settings: &AiToolSettings) -> AiToolUpdate {
    AiToolUpdate {
        api_key: Some(String::new()),
        ..update_of(settings)
    }
}

/// Why the API's form can't be saved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApiFormError {
    NoAddress,
    BadAddress,
    NoModel,
    /// Too long, or with a control character.
    Unusable,
}

/// The API's form as the request that saves it, with the tool in API mode.
/// An empty key keeps the one saved.
pub fn api_request(
    tool: AiTool,
    settings: &AiToolSettings,
    url: &str,
    model: &str,
    key: &str,
) -> Result<AiToolUpdate, ApiFormError> {
    let url = url.trim();
    if url.is_empty() {
        return Err(ApiFormError::NoAddress);
    }
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(ApiFormError::BadAddress);
    }
    if model.trim().is_empty() {
        return Err(ApiFormError::NoModel);
    }
    let key = key.trim();
    let update = AiToolUpdate {
        mode: AiMode::Api,
        api_url: url.to_owned(),
        api_model: model.to_owned(),
        api_key: (!key.is_empty()).then(|| key.to_owned()),
        ..update_of(settings)
    };
    clean_tool_update(tool, &update).map_err(|_| ApiFormError::Unusable)
}

pub fn api_form_error_text(e: ApiFormError) -> String {
    match e {
        ApiFormError::NoAddress => t!("admin-ai-error-url"),
        ApiFormError::BadAddress => t!("admin-ai-error-url-scheme"),
        ApiFormError::NoModel => t!("admin-ai-error-model"),
        ApiFormError::Unusable => t!("admin-ai-error-long", max = MAX_AI_FIELD_CHARS),
    }
}

/// The language new comics are translated into, as typed: a code such as
/// `en` or `pt-BR`, or empty for none. `None` when it isn't one.
pub fn translate_request(autotag: bool, typed: &str) -> Option<AiGeneralUpdate> {
    clean_general_update(&AiGeneralUpdate {
        autotag,
        auto_translate: typed.to_owned(),
    })
    .ok()
}

#[cfg(test)]
mod tests {
    use chrono::{FixedOffset, NaiveDate, TimeDelta};
    use starbooru_core::{AiPresetOption, AiRuntimeStatus, Role};

    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::from_timestamp(1_800_000_000, 0).unwrap()
    }

    fn user() -> AdminUser {
        AdminUser {
            id: 2,
            username: "someone".into(),
            role: Role::User,
            created_at: now() - TimeDelta::days(40),
            disabled: false,
            posts: 3,
            bytes: 1536,
            quota_bytes: None,
            last_active: None,
        }
    }

    #[test]
    fn each_tab_has_an_address_of_its_own() {
        assert_eq!(Tab::parse(None), Tab::Overview);
        assert_eq!(Tab::parse(Some("nonsense")), Tab::Overview);
        for tab in Tab::ALL {
            let href = tab.href();
            let value = href.split_once("?tab=").map(|(_, v)| v);
            assert_eq!(Tab::parse(value), tab, "{href}");
        }
        assert_eq!(Tab::Overview.href(), "/admin");
        assert_eq!(Tab::Server.href(), "/admin?tab=server");
    }

    #[test]
    fn a_limit_is_typed_in_gb_or_with_its_unit() {
        assert_eq!(parse_quota(""), Ok(None));
        assert_eq!(parse_quota("  "), Ok(None));
        assert_eq!(parse_quota("1"), Ok(Some(1 << 30)));
        assert_eq!(parse_quota("0,5"), Ok(Some(1 << 29)));
        assert_eq!(parse_quota("0"), Ok(Some(0)));
        assert_eq!(parse_quota("5 GB"), Ok(Some(5 << 30)));
        assert_eq!(parse_quota("5gb"), Ok(Some(5 << 30)));
        assert_eq!(parse_quota("5 GiB"), Ok(Some(5 << 30)));
        assert_eq!(parse_quota("500 MB"), Ok(Some(500 << 20)));
        assert_eq!(parse_quota("1 TB"), Ok(Some(1 << 40)));
        assert_eq!(parse_quota("2,5 tb"), Ok(Some(5 << 39)));
        for bad in [
            "-1", "lots", "NaN", "inf", "1e300", "5 PB", "5 G B", "1.2.3", "GB",
        ] {
            assert_eq!(parse_quota(bad), Err(BadQuota), "{bad}");
        }
        assert_eq!(parse_quota("1000001"), Err(BadQuota));
        assert_eq!(quota_text(1 << 30), "1 GB");
        assert_eq!(quota_text(3 << 29), "1.5 GB");
        assert_eq!(quota_text(10 << 30), "10 GB");
        assert_eq!(quota_text(1 << 28), "0.25 GB");
        assert_eq!(quota_text(0), "0 GB");
        for text in ["2.5 GB", "0.25 GB", "100 GB"] {
            let bytes = parse_quota(text).unwrap().unwrap();
            assert_eq!(quota_text(bytes), text);
        }
    }

    #[test]
    fn an_account_says_what_it_uses_and_when_it_was_seen() {
        let mut user = user();
        assert_eq!(usage_line(&user), "3 posts · 1.5 KB");
        user.quota_bytes = Some(1 << 30);
        assert_eq!(usage_line(&user), "3 posts · 1.5 KB of 1.0 GB");
        let utc = FixedOffset::east_opt(0).unwrap();
        let joined = (now() - TimeDelta::days(40)).format("%Y-%m-%d").to_string();
        let line = |user: &AdminUser, zone| account_parts(user, now(), zone).join(" · ");
        assert_eq!(
            line(&user, &utc),
            format!("3 posts · 1.5 KB of 1.0 GB · never active · joined {joined}")
        );
        user.last_active = Some(now() - TimeDelta::minutes(3));
        assert!(line(&user, &utc).contains(" · active 3 minutes ago · "));
        // The date is the admin's own day: 20:00 UTC is the next morning in Tokyo.
        user.created_at = DateTime::from_timestamp(1_800_000_000 - 12 * 3600, 0).unwrap();
        let tokyo = FixedOffset::east_opt(9 * 3600).unwrap();
        let text = line(&user, &tokyo);
        assert!(text.ends_with("joined 2027-01-15"), "{text}");
    }

    #[test]
    fn the_start_reads_as_a_time_ago() {
        let start = |minutes| now() - TimeDelta::minutes(minutes);
        assert_eq!(started(start(3), now()), "3 minutes ago");
        assert_eq!(started(start(11 * 60 + 40), now()), "11 hours ago");
        assert_eq!(started(start(36 * 60), now()), "yesterday");
        assert_eq!(started(start(3 * 24 * 60), now()), "3 days ago");
        // A clock a little ahead of the server's.
        assert_eq!(started(now() + TimeDelta::seconds(20), now()), "just now");
    }

    #[test]
    fn the_chart_scales_to_the_busiest_day() {
        let day = |d, posts| DayCount {
            day: NaiveDate::from_ymd_opt(2026, 11, d).unwrap(),
            posts,
        };
        let c = chart(&[day(21, 0), day(22, 67), day(23, 0)]);
        assert_eq!(c.total, 67);
        assert_eq!(c.first, "21/11");
        assert_eq!(c.last, "23/11");
        let heights: Vec<f64> = c.bars.iter().map(|b| b.height).collect();
        assert_eq!(heights, [0.0, 100.0, 0.0]);
        assert_eq!(c.bars.get(1).map(|b| b.label.as_str()), Some("22/11"));
        // A quiet month, or none at all, draws flat.
        let quiet = chart(&[day(1, 0), day(2, 0)]);
        assert!(quiet.bars.iter().all(|b| b.height == 0.0));
        assert_eq!(chart(&[]), Chart::default());
        // Huge counts don't overflow.
        assert_eq!(chart(&[day(1, u64::MAX), day(2, 5)]).total, u64::MAX);
    }

    #[test]
    fn the_settings_form_checks_what_the_server_checks() {
        let ok = check_settings("  My   booru ", true, " 30 ", false);
        assert!(ok.name_ok && ok.days_ok);
        assert_eq!(
            ok.request,
            Some(ServerSettings {
                name: "My booru".into(),
                allow_registration: true,
                trash_days: 30,
                auto_meta_tags: false,
            })
        );
        assert!(check_settings("x", false, "0", true).request.is_some());
        assert!(check_settings("x", false, "3650", true).request.is_some());
        let long = check_settings(&"n".repeat(65), true, "0", true);
        assert!(!long.name_ok && long.days_ok && long.request.is_none());
        assert!(check_settings(&"é".repeat(64), true, "0", true).name_ok);
        let empty = check_settings("   ", true, "1", true);
        assert!(!empty.name_ok && empty.request.is_none());
        assert!(!check_settings("a\u{7}b", true, "1", true).name_ok);
        for days in ["3651", "-1", "", "ten", "1.5"] {
            let check = check_settings("x", true, days, true);
            assert!(
                check.name_ok && !check.days_ok && check.request.is_none(),
                "{days}"
            );
        }
    }

    #[test]
    fn the_schedule_form_checks_each_number() {
        let ok = check_schedule(" 1 ", "7");
        assert_eq!(
            ok.schedule,
            Some(BackupSchedule {
                every_days: 1,
                keep: 7
            })
        );
        assert!(check_schedule("0", "1").schedule.is_some());
        assert!(check_schedule("365", "100").schedule.is_some());
        let every = check_schedule("366", "7");
        assert!(!every.every_ok && every.keep_ok && every.schedule.is_none());
        let keep = check_schedule("1", "0");
        assert!(keep.every_ok && !keep.keep_ok && keep.schedule.is_none());
        assert!(!check_schedule("1", "101").keep_ok);
        assert!(!check_schedule("", "7").every_ok);
        assert!(!check_schedule("-1", "7").every_ok);
    }

    #[test]
    fn a_backup_says_when_and_how_big() {
        let backup = Backup {
            name: "starbooru-20270115-080000.dump".into(),
            bytes: 81_945,
            created_at: DateTime::from_timestamp(1_800_000_000, 0).unwrap(),
            automatic: false,
            url: String::new(),
        };
        assert_eq!(backup_time(&backup, &Utc), "2027-01-15 08:00");
        let rome = FixedOffset::east_opt(3600).unwrap();
        assert_eq!(backup_time(&backup, &rome), "2027-01-15 09:00");
        assert_eq!(
            backup_line(&backup),
            "80.0 KB · starbooru-20270115-080000.dump"
        );
    }

    #[test]
    fn answer_times_read_well() {
        assert_eq!(duration(816), "0.8 s");
        assert_eq!(duration(9_999), "10.0 s");
        assert_eq!(duration(16_928), "16 s");
        assert_eq!(duration(116_354), "2 min");
        // A corrupt timing doesn't overflow.
        assert_eq!(duration(u32::MAX), "71582 min");
        let model = |measured| AiModelOption {
            est_ms: 900,
            measured,
            ..AiModelOption::default()
        };
        assert_eq!(model_time(&model(true)), "0.9 s");
        assert_eq!(model_time(&model(false)), "≈ 0.9 s");
    }

    #[test]
    fn progress_never_divides_by_zero() {
        assert_eq!(percent(0, 0), 0);
        assert_eq!(percent(5, 10), 50);
        assert_eq!(percent(u64::MAX, 10), u64::MAX / 10);
    }

    #[test]
    fn the_machine_fits_on_one_line() {
        let mut hw = Hardware {
            cpu: Some("AMD Ryzen 9 5900X".into()),
            threads: 24,
            ram_total: 32 << 30,
            ..Hardware::default()
        };
        assert_eq!(
            hardware_line(&hw),
            "AMD Ryzen 9 5900X · 24 threads · 32.0 GB RAM"
        );
        hw.board = Some("Raspberry Pi 5 Model B".into());
        hw.ram_total = 0;
        assert_eq!(hardware_line(&hw), "Raspberry Pi 5 Model B · 24 threads");
    }

    #[test]
    fn the_servers_reason_reads_as_a_sentence() {
        assert_eq!(
            sentence("the AI runtime can't be downloaded: check the disk"),
            "The AI runtime can't be downloaded: check the disk."
        );
        assert_eq!(sentence(" Done. "), "Done.");
        assert_eq!(sentence("è già spento"), "È già spento.");
        assert_eq!(sentence(""), "");
    }

    #[test]
    fn the_runtime_starting_or_a_download_counts_as_busy() {
        let mut status = AiAdminStatus::default();
        // Not ready, and no reason why: it's starting.
        assert!(busy(&status));
        status.runtime = AiRuntimeStatus {
            ready: false,
            reason: Some("turned off".into()),
            gpu: None,
        };
        assert!(!busy(&status));
        status.tools.push(AiToolStatus {
            models: vec![AiModelOption {
                progress: Some(starbooru_core::AiInstallProgress { done: 1, total: 2 }),
                ..AiModelOption::default()
            }],
            ..AiToolStatus::default()
        });
        assert!(busy(&status));
    }

    #[test]
    fn balanced_stands_in_for_a_preset_out_of_reach() {
        let option = |preset, available| AiPresetOption {
            preset,
            available,
            model: available.then(|| preset.as_str().to_owned()),
            est_ms: None,
        };
        let model = |id: &str| AiModelOption {
            id: id.into(),
            ..AiModelOption::default()
        };
        let mut tool = AiToolStatus {
            models: vec![model("balanced"), model("accurate")],
            presets: vec![
                option(AiPreset::Fast, false),
                option(AiPreset::Balanced, true),
                option(AiPreset::Accurate, true),
            ],
            ..AiToolStatus::default()
        };
        tool.settings.preset = AiPreset::Accurate;
        assert_eq!(effective_preset(&tool), AiPreset::Accurate);
        assert_eq!(wanted_model(&tool).unwrap().id, "accurate");
        tool.settings.preset = AiPreset::Fast;
        assert_eq!(effective_preset(&tool), AiPreset::Balanced);
        assert_eq!(wanted_model(&tool).unwrap().id, "balanced");
        // A model picked by hand wins, while it's in the catalogue.
        tool.settings.model = "accurate".into();
        assert_eq!(wanted_model(&tool).unwrap().id, "accurate");
        tool.settings.model = "gone".into();
        assert_eq!(wanted_model(&tool), None);
    }

    fn saved_api() -> AiToolSettings {
        AiToolSettings {
            mode: AiMode::Off,
            preset: AiPreset::Fast,
            model: "wd-vit".into(),
            api_url: "http://127.0.0.1:11434/v1".into(),
            api_model: "llava".into(),
            api_key_set: true,
        }
    }

    #[test]
    fn picking_a_mode_keeps_the_rest_of_the_setup() {
        let none = AiToolSettings::default();
        // The API waits for its form when there's no address or model yet.
        assert_eq!(choose_mode(&none, AiMode::Api), ModeChoice::OpenForm);
        let ModeChoice::Send(local) = choose_mode(&none, AiMode::Local) else {
            panic!("local sends at once");
        };
        assert_eq!(local.mode, AiMode::Local);
        let saved = saved_api();
        let ModeChoice::Send(api) = choose_mode(&saved, AiMode::Api) else {
            panic!("a saved API sends at once");
        };
        assert_eq!(
            api,
            AiToolUpdate {
                mode: AiMode::Api,
                preset: AiPreset::Fast,
                model: "wd-vit".into(),
                api_url: "http://127.0.0.1:11434/v1".into(),
                api_model: "llava".into(),
                // The key stays as it is.
                api_key: None,
            }
        );
        let preset = preset_request(&saved, AiPreset::Accurate);
        assert_eq!(
            (preset.preset, preset.model.as_str()),
            (AiPreset::Accurate, "")
        );
        assert_eq!(model_request(&saved, "joytag").model, "joytag");
        assert_eq!(clear_key_request(&saved).api_key.as_deref(), Some(""));
    }

    #[test]
    fn the_api_form_becomes_its_request() {
        let none = AiToolSettings::default();
        let request = |url: &str, model: &str, key: &str| {
            api_request(AiTool::Tagging, &none, url, model, key)
        };
        let saved = request(" http://127.0.0.1:9/v1/ ", " fake-model ", "").unwrap();
        assert_eq!(saved.mode, AiMode::Api);
        assert_eq!(saved.api_url, "http://127.0.0.1:9/v1");
        assert_eq!(saved.api_model, "fake-model");
        // An empty key keeps the one saved; a typed one replaces it.
        assert_eq!(saved.api_key, None);
        let keyed = request("https://api.example/v1", "m", " sk-1 ").unwrap();
        assert_eq!(keyed.api_key.as_deref(), Some("sk-1"));
        assert_eq!(request("", "m", ""), Err(ApiFormError::NoAddress));
        assert_eq!(
            request("api.example/v1", "m", ""),
            Err(ApiFormError::BadAddress)
        );
        assert_eq!(
            request("ftp://api.example", "m", ""),
            Err(ApiFormError::BadAddress)
        );
        assert_eq!(
            request("https://api.example", "  ", ""),
            Err(ApiFormError::NoModel)
        );
        let long = "m".repeat(MAX_AI_FIELD_CHARS + 1);
        assert_eq!(
            request("https://api.example", &long, ""),
            Err(ApiFormError::Unusable)
        );
        assert!(api_form_error_text(ApiFormError::Unusable).contains("512"));
    }

    #[test]
    fn new_comics_are_translated_into_a_language_code_or_none() {
        assert_eq!(
            translate_request(true, " pt-BR "),
            Some(AiGeneralUpdate {
                autotag: true,
                auto_translate: "pt-BR".into()
            })
        );
        assert_eq!(
            translate_request(false, "").map(|u| u.auto_translate),
            Some(String::new())
        );
        assert_eq!(translate_request(false, "english please"), None);
    }
}
