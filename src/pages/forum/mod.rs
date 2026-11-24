//! The forum of a public server, in Upload's place in the navigation: its
//! categories at `/forum` (`index.rs`, where administrators also manage
//! them), a category's threads at `/forum/c/{id}` (`category.rs`), and a
//! thread at `/forum/t/{id}` (`thread.rs`), with the box to write in
//! (`compose.rs`). While the forum is open the top bar's search finds its
//! threads (`?q=`). Guests read everything, and are asked to sign in where
//! they'd write. A private server has no forum: the pages say so.

mod category;
mod compose;
mod index;
mod thread;

use std::cell::RefCell;

use chrono::{DateTime, Local, Utc};
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{ForumCategory, Role, UserCard};

pub use self::category::ForumCategoryPage;
pub use self::index::Forum;
pub use self::thread::ForumThreadPage;
use crate::components::avatar::{Size, UserLink};
use crate::components::banner::Spinner;
use crate::components::icons::{Ico, Icon};
use crate::logic::library::ago;
use crate::logic::viewer::when;
use crate::{api, instance, store};

thread_local! {
    // Reference counted, as they outlive every page: the categories, and
    // whose they are.
    static CATEGORIES: ArcRwSignal<Option<(String, Vec<ForumCategory>)>> = ArcRwSignal::new(None);
    /// When the account signed in last posted, as far as this tab knows,
    /// for the wait before the next post.
    static LAST_POST: RefCell<Option<(String, DateTime<Utc>)>> = const { RefCell::new(None) };
}

/// Whose an answer is: the account, or the guests, on the server.
fn owner() -> String {
    let who = store::owner();
    if who.is_empty() {
        instance::guest_server()
            .map(|s| format!("{s}|guest"))
            .unwrap_or_default()
    } else {
        who
    }
}

/// The forum's categories, once asked. Read inside a view, it follows each
/// answer.
pub(crate) fn categories() -> Option<Vec<ForumCategory>> {
    let owner = owner();
    CATEGORIES.with(|c| {
        c.with(|c| {
            c.as_ref()
                .filter(|(of, _)| *of == owner)
                .map(|(_, list)| list.clone())
        })
    })
}

fn set_categories(list: Vec<ForumCategory>) {
    let owner = owner();
    CATEGORIES.with(|c| c.set(Some((owner, list))));
}

/// Asks for the categories again. A failure comes back as words.
pub(crate) async fn load_categories() -> Result<Vec<ForumCategory>, api::Refused> {
    let list = api::attempt(|c| async move { c.forum().await }).await?;
    set_categories(list.clone());
    Ok(list)
}

/// The name of category `id`, once the categories are known.
fn category_name(id: i64) -> Option<String> {
    categories()?
        .into_iter()
        .find(|c| c.id == id)
        .map(|c| c.name)
}

/// The account posted at `at`.
fn posted(at: DateTime<Utc>) {
    let owner = store::owner();
    LAST_POST.with_borrow_mut(|last| {
        let newer = last
            .as_ref()
            .is_none_or(|(of, was)| *of != owner || *was < at);
        if newer && !owner.is_empty() {
            *last = Some((owner, at));
        }
    });
}

/// When the account last posted, as far as this tab knows.
fn last_post() -> Option<DateTime<Utc>> {
    let owner = store::owner();
    LAST_POST.with_borrow(|last| {
        last.as_ref()
            .filter(|(of, _)| *of == owner)
            .map(|(_, at)| *at)
    })
}

/// The name signed in, and whether it's an administrator's.
fn me() -> (Option<String>, bool) {
    let session = store::session();
    let admin = session.as_ref().is_some_and(|s| s.user.role == Role::Admin);
    (session.map(|s| s.user.username), admin)
}

/// Whether the error code says the server has no forum.
fn is_off(code: Option<&str>) -> bool {
    code == Some("forum_off")
}

/// What stands in for the forum on a private server.
#[component]
fn NoForum() -> impl IntoView {
    view! {
        <div class="card empty">
            <Ico icon=Icon::Forum size=48 />
            <h2>{move || t!("forum-off")}</h2>
            <p class="muted">{move || t!("forum-off-body")}</p>
            <a class="btn" href="/">
                {move || t!("notfound-back")}
            </a>
        </div>
    }
}

/// The forum's pages, once the server has said it's public; the note that
/// there is none on a private one.
#[component]
fn ForumOnly(children: ChildrenFn) -> impl IntoView {
    let shown = Memo::new(|_| instance::known().then(instance::public));
    move || match shown.get() {
        None => view! {
            <div class="sentinel">
                <Spinner />
            </div>
        }
        .into_any(),
        Some(true) => children().into_any(),
        Some(false) => view! { <NoForum /> }.into_any(),
    }
}

/// Who wrote something.
#[component]
fn Author(card: Option<UserCard>, #[prop(optional)] size: Size) -> impl IntoView {
    view! { <UserLink card=card size=size /> }
}

/// A moment as "5 minutes ago", with the date and time on hover, and as a
/// `<time>` for screen readers and copying.
#[component]
fn Ago(at: DateTime<Utc>) -> impl IntoView {
    let exact = when(&at, &Local);
    let iso = at.to_rfc3339();
    view! {
        <time datetime=iso title=exact>
            {move || ago(at, Utc::now())}
        </time>
    }
}

/// Asks for the categories when the page opens, unless known.
fn ask_categories() {
    if untrack(categories).is_none() {
        spawn_local(async move {
            let _ = load_categories().await;
        });
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_forum_styles_keep_to_the_tokens_and_the_motion_settings() {
        let css = include_str!("../../../style/pages/forum.css");
        assert!(!css.contains("backdrop-filter"));
        assert!(css.contains("@media (prefers-reduced-motion: reduce)"));
        assert!(css.contains(r#".app[data-effects="lite"]"#));
        let page = include_str!("../../../index.html");
        assert!(page.contains(r#"<link data-trunk rel="css" href="style/pages/forum.css">"#));
    }

    #[test]
    fn a_post_is_drawn_from_its_markdown_with_its_mentions_linked() {
        let thread = include_str!("thread.rs");
        assert!(thread.contains("<Markdown text=body mentions=true />"));
        let compose = include_str!("compose.rs");
        assert!(compose.contains("<Markdown text=text.get() mentions=true />"));
    }
}
