//! Administration › Overview: what the server holds, the uploads of the last
//! 30 days, and how the server runs.

use chrono::Utc;
use leptos::prelude::*;
use starbooru_core::ServerStats;

use super::Waiting;
use crate::api;
use crate::components::banner::ErrorBanner;
use crate::logic::admin::chart;
use crate::logic::library::ago;
use crate::logic::viewer::human_size;

#[component]
pub(super) fn Overview() -> impl IntoView {
    let stats = LocalResource::new(|| api::call(|c| async move { c.admin_stats().await }));
    let retry = Callback::new(move |()| stats.refetch());
    move || match stats.get() {
        None => view! { <Waiting /> }.into_any(),
        Some(Err(message)) => view! { <ErrorBanner message=message on_retry=retry /> }.into_any(),
        Some(Ok(s)) => view! { <Stats stats=s /> }.into_any(),
    }
}

#[component]
fn Stats(stats: ServerStats) -> impl IntoView {
    let s = stats;
    let tiles: [(fn() -> String, String); 8] = [
        (|| t!("admin-posts"), s.posts().to_string()),
        (|| t!("admin-images"), s.images.to_string()),
        (|| t!("admin-animations"), s.animations.to_string()),
        (|| t!("admin-videos"), s.videos.to_string()),
        (|| t!("admin-bin"), s.bin_posts.to_string()),
        (|| t!("admin-tags"), s.tags.to_string()),
        (|| t!("admin-accounts"), s.users.to_string()),
        (|| t!("admin-media"), human_size(s.media_bytes)),
    ];
    let tiles = tiles
        .into_iter()
        .map(|(label, value)| {
            view! {
                <div class="stat">
                    <span class="stat-value">{value}</span>
                    <span class="stat-label">{label}</span>
                </div>
            }
        })
        .collect_view();
    let days = chart(&s.uploads_by_day);
    let total = days.total;
    let bars = days
        .bars
        .into_iter()
        .map(|b| {
            view! {
                <div
                    class="bar"
                    class:empty=b.posts == 0
                    style=format!("height: {:.1}%", b.height)
                    title=format!("{}: {}", b.label, b.posts)
                ></div>
            }
        })
        .collect_view();
    let on_off = |on: bool| if on { t!("admin-on") } else { t!("admin-off") };
    let (thumbs, import, since) = (s.video_thumbnails, s.url_import, s.started_at);
    let running = move || ago(since, Utc::now());
    view! {
        <div class="stat-grid">{tiles}</div>
        <section class="card">
            <h2 class="card-title">{move || t!("admin-uploads-title")}</h2>
            <p class="muted hint">{move || t!("admin-uploads-total", count = total)}</p>
            <div class="bars" role="img" aria-label=move || t!("admin-uploads-total", count = total)>
                {bars}
            </div>
            <div class="bars-axis muted">
                <span>{days.first}</span>
                <span>{days.last}</span>
            </div>
        </section>
        <section class="card">
            <h2 class="card-title">{move || t!("admin-server-title")}</h2>
            <dl class="meta">
                <dt>{move || t!("settings-server-version")}</dt>
                <dd class="break">{s.version}</dd>
                <dt>{move || t!("admin-running")}</dt>
                <dd>{running}</dd>
                <dt>{move || t!("admin-upload-limit")}</dt>
                <dd>{human_size(s.max_upload_bytes)}</dd>
                <dt>{move || t!("admin-database")}</dt>
                <dd>{human_size(s.database_bytes)}</dd>
                <dt>{move || t!("admin-bin-size")}</dt>
                <dd>{human_size(s.bin_bytes)}</dd>
                <dt>{move || t!("admin-disabled-accounts")}</dt>
                <dd>{s.disabled_users}</dd>
                <dt>{move || t!("admin-video-thumbs")}</dt>
                <dd>{move || on_off(thumbs)}</dd>
                <dt>{move || t!("admin-url-import")}</dt>
                <dd>{move || on_off(import)}</dd>
            </dl>
        </section>
    }
}
