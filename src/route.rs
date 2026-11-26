//! The app's addresses. They keep the reference's paths, so links people saved
//! still open; `app.rs` maps each to its page. Building and reading them is
//! plain Rust here, tested on the host.

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

/// The navigation's items, five at most (the reference's design rules): a new
/// page goes under one of these. A private server has [`Tab::ALL`], or for
/// a guest [`Tab::GUEST`] and a way to sign in after them. A public one has
/// [`Tab::PUBLIC`], the forum in Upload's place, and the account button
/// beside them, which holds Upload and Settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Gallery,
    Explore,
    Tags,
    Forum,
    Upload,
    Settings,
}

impl Tab {
    pub const ALL: [Tab; 5] = [
        Tab::Gallery,
        Tab::Explore,
        Tab::Tags,
        Tab::Upload,
        Tab::Settings,
    ];

    /// What a guest may use: Upload needs an account.
    pub const GUEST: [Tab; 4] = [Tab::Gallery, Tab::Explore, Tab::Tags, Tab::Settings];

    /// A public server's, for anyone: the account button comes after them.
    pub const PUBLIC: [Tab; 4] = [Tab::Gallery, Tab::Explore, Tab::Tags, Tab::Forum];

    /// The items of the navigation on a public or a private server, a
    /// guest's or an account's.
    pub fn shown(public: bool, guest: bool) -> &'static [Tab] {
        match (public, guest) {
            (true, _) => &Tab::PUBLIC,
            (false, true) => &Tab::GUEST,
            (false, false) => &Tab::ALL,
        }
    }

    /// Where the item leads.
    pub fn href(self) -> &'static str {
        match self {
            Tab::Gallery => "/",
            Tab::Explore => "/explore",
            Tab::Tags => "/tags",
            Tab::Forum => FORUM,
            Tab::Upload => "/upload",
            Tab::Settings => "/settings",
        }
    }
}

/// The item a page is under, so the navigation always says where you are.
/// `None` for pages outside the shell and for addresses that lead nowhere.
pub fn tab_of(path: &str) -> Option<Tab> {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    match segments.as_slice() {
        [] | ["post", _] => Some(Tab::Gallery),
        // The booru browser is the other side of Explore, and everyone in
        // the collection one of its sections.
        ["explore"] | ["boorus"] | ["boorus", "post"] | ["people"] => Some(Tab::Explore),
        ["tags"] | ["tag"] => Some(Tab::Tags),
        ["forum"] | ["forum", "c" | "t", _] => Some(Tab::Forum),
        ["upload"] => Some(Tab::Upload),
        ["settings"] | ["settings", _] | ["trash"] | ["duplicates"] | ["help"] | ["admin"] => {
            Some(Tab::Settings)
        }
        _ => None,
    }
}

/// Where in the frame a page belongs: under an item of the navigation, or
/// on a public server under the account button, with Upload, Settings and
/// the account's own profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Tab(Tab),
    Account,
}

/// The place of `path`, on a public server or not, for the account named
/// `me` when someone is signed in.
pub fn place_of(path: &str, public: bool, me: Option<&str>) -> Option<Place> {
    if let Some(name) = profile_name(path) {
        let mine = me.is_some_and(|me| me.eq_ignore_ascii_case(&name));
        return (public && mine).then_some(Place::Account);
    }
    match tab_of(path)? {
        Tab::Upload | Tab::Settings if public => Some(Place::Account),
        Tab::Forum if !public => None,
        tab => Some(Place::Tab(tab)),
    }
}

/// The username in a profile's address, `/u/{name}`.
pub fn profile_name(path: &str) -> Option<String> {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    match segments.as_slice() {
        ["u", name] => Some(
            percent_encoding::percent_decode_str(name)
                .decode_utf8_lossy()
                .into_owned(),
        ),
        _ => None,
    }
}

/// The profile of the account named `name`.
pub fn profile(name: &str) -> String {
    format!("/u/{}", utf8_percent_encode(name, PATH_SEGMENT))
}

/// What a path segment keeps as it is: a username's letters and signs.
const PATH_SEGMENT: &AsciiSet = &NON_ALPHANUMERIC.remove(b'-').remove(b'_').remove(b'.');

/// The forum's categories, or with `q` the threads found in all of them.
pub fn forum(q: &str) -> String {
    let q = q.trim();
    if q.is_empty() {
        FORUM.to_owned()
    } else {
        format!("{FORUM}?q={}", query_value(q))
    }
}

/// The threads of category `id`, or with `q` those found in it.
pub fn forum_category(id: i64, q: &str) -> String {
    let q = q.trim();
    if q.is_empty() {
        format!("{FORUM}/c/{id}")
    } else {
        format!("{FORUM}/c/{id}?q={}", query_value(q))
    }
}

/// Thread `id`.
pub fn forum_thread(id: i64) -> String {
    format!("{FORUM}/t/{id}")
}

/// Post `post` in thread `thread`, by its anchor.
pub fn forum_post(thread: i64, post: i64) -> String {
    format!("{FORUM}/t/{thread}#{}", forum_anchor(post))
}

/// The id of a forum post's element, for links to it.
pub fn forum_anchor(post: i64) -> String {
    format!("p{post}")
}

/// What the forum's pages are: its categories, one category, a thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForumPage {
    Index,
    Category(i64),
    Thread(i64),
}

/// The forum's page at `path`, where the search box searches the forum.
pub fn forum_page(path: &str) -> Option<ForumPage> {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let id = |s: &str| s.parse::<i64>().ok();
    match segments.as_slice() {
        ["forum"] => Some(ForumPage::Index),
        ["forum", "c", c] => id(c).map(ForumPage::Category),
        ["forum", "t", t] => id(t).map(ForumPage::Thread),
        _ => None,
    }
}

/// What a value in a query string keeps as it is: letters, digits and the
/// signs a search is mostly made of, so the address stays readable.
const QUERY_VALUE: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~')
    .remove(b':')
    .remove(b' ');

/// `s` written for a query string. A space is `+`, as in a form, and a `+`
/// typed in the search is `%2B`.
pub fn query_value(s: &str) -> String {
    utf8_percent_encode(s, QUERY_VALUE)
        .to_string()
        .replace(' ', "+")
}

/// The gallery for search `q`: everything when it's empty.
pub fn gallery(q: &str) -> String {
    let q = q.trim();
    if q.is_empty() {
        "/".to_owned()
    } else {
        format!("/?q={}", query_value(q))
    }
}

/// The gallery for search `q` as typed, with no unknown tag corrected: the
/// correction banner's way back.
pub fn exact(q: &str) -> String {
    gallery(&starbooru_core::with_exact(q))
}

/// Explore narrowed by search `q`.
pub fn explore(q: &str) -> String {
    let q = q.trim();
    if q.is_empty() {
        EXPLORE.to_owned()
    } else {
        format!("{EXPLORE}?q={}", query_value(q))
    }
}

/// Explore narrowed by search `q`, with Trending over a window of `days`,
/// as a public server has it. The usual week leaves the address without one.
pub fn explore_over(q: &str, days: u32) -> String {
    let to = explore(q);
    if days == starbooru_core::DEFAULT_EXPLORE_DAYS {
        to
    } else {
        let sep = if to.contains('?') { '&' } else { '?' };
        format!("{to}{sep}days={days}")
    }
}

/// Whether `path` is Explore's collection side, where the search box
/// narrows Explore.
pub fn is_explore(path: &str) -> bool {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    segments.as_slice() == ["explore"]
}

/// The Tags page filtered by `q`, as the search box filters it there.
pub fn tags(q: &str) -> String {
    let q = q.trim();
    if q.is_empty() {
        TAGS.to_owned()
    } else {
        format!("{TAGS}?q={}", query_value(q))
    }
}

/// Whether `path` is the Tags page, where the search box filters the tags.
pub fn is_tags(path: &str) -> bool {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    segments.as_slice() == ["tags"]
}

/// The people page, narrowed to names holding `q`, as the search box
/// narrows it there.
pub fn people(q: &str) -> String {
    let q = q.trim();
    if q.is_empty() {
        PEOPLE.to_owned()
    } else {
        format!("{PEOPLE}?q={}", query_value(q))
    }
}

/// Whether `path` is the people page, where the search box filters the
/// people by name.
pub fn is_people(path: &str) -> bool {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    segments.as_slice() == ["people"]
}

/// Post `id`, opened from the gallery for search `q`, so the viewer can step
/// through the same results. `seed` is the shuffle of an `order:random` search.
pub fn post(id: i64, q: &str, seed: Option<i64>) -> String {
    let mut to = format!("/post/{id}");
    let q = q.trim();
    if !q.is_empty() {
        to.push_str("?q=");
        to.push_str(&query_value(q));
    }
    if let Some(seed) = seed {
        to.push(if q.is_empty() { '?' } else { '&' });
        to.push_str(&format!("seed={seed}"));
    }
    to
}

/// Whether the page at `path` shows a search of its own in `?q=`: the
/// gallery and the posts opened from it.
pub fn has_search(path: &str) -> bool {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    matches!(segments.as_slice(), [] | ["post", _])
}

/// The page of tag `name`.
pub fn tag(name: &str) -> String {
    format!("/tag?name={}", query_value(name))
}

/// What the booru browser's `u` and `s` keep as they are besides: the
/// slashes of an address and the signs that part the sites.
const ADDRESS_VALUE: &AsciiSet = &QUERY_VALUE.remove(b'/').remove(b',').remove(b'|');

/// The query string of the booru browser's pages: `pairs` with an empty
/// value left out, each written as `to_address` writes it (`q` and `u`) or
/// as `sites_param` did (`s`).
fn booru_query(pairs: &[(&str, &str)]) -> String {
    let parts: Vec<String> = pairs
        .iter()
        .filter(|(_, value)| !value.is_empty())
        .map(|(name, value)| {
            let set = if *name == "q" {
                QUERY_VALUE
            } else {
                ADDRESS_VALUE
            };
            let value = match *name {
                "s" => (*value).to_owned(),
                _ => crate::logic::boorus::to_address(value),
            };
            let encoded = utf8_percent_encode(&value, set)
                .to_string()
                .replace(' ', "+");
            format!("{name}={encoded}")
        })
        .collect();
    if parts.is_empty() {
        String::new()
    } else {
        format!("?{}", parts.join("&"))
    }
}

/// The booru browser for search `q` on the sites `s` names
/// (`logic::boorus::sites_param`).
pub fn boorus(q: &str, s: &str) -> String {
    format!("{BOORUS}{}", booru_query(&[("q", q.trim()), ("s", s)]))
}

/// The post at page `u` on another booru, opened from search `q` on the
/// sites `s`.
pub fn booru_post(u: &str, q: &str, s: &str) -> String {
    format!(
        "{BOORUS}/post{}",
        booru_query(&[("u", u), ("q", q.trim()), ("s", s)])
    )
}

/// Explore's collection side.
pub const EXPLORE: &str = "/explore";
/// The home page, when the server has one on.
pub const HOME: &str = "/home";
/// The booru browser.
pub const BOORUS: &str = "/boorus";

/// The list of tags.
pub const TAGS: &str = "/tags";
/// Everyone in the collection, by franchise.
pub const PEOPLE: &str = "/people";
/// The Upload page.
pub const UPLOAD: &str = "/upload";
/// The search cheat sheet.
pub const HELP: &str = "/help";
/// The recycle bin.
pub const TRASH: &str = "/trash";
/// The duplicate finder.
pub const DUPLICATES: &str = "/duplicates";
/// Settings: its list of groups, each group at `/settings/<group>`
/// (`logic::settings::Group`).
pub const SETTINGS: &str = "/settings";

/// Whether `path` is Settings, its list or one of its groups.
pub fn is_settings(path: &str) -> bool {
    path.strip_prefix(SETTINGS)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

/// Settings at `path`, its list or a group, searched for `q`, as the
/// search box searches it there.
pub fn settings_search(path: &str, q: &str) -> String {
    let path = if is_settings(path) { path } else { SETTINGS };
    let q = q.trim();
    if q.is_empty() {
        path.to_owned()
    } else {
        format!("{path}?q={}", query_value(q))
    }
}

/// The Upload page narrowed to the files and addresses holding `q`.
pub fn upload(q: &str) -> String {
    let q = q.trim();
    if q.is_empty() {
        UPLOAD.to_owned()
    } else {
        format!("{UPLOAD}?q={}", query_value(q))
    }
}

/// Whether `path` is the Upload page, where the search box narrows the
/// tiles.
pub fn is_upload(path: &str) -> bool {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    segments.as_slice() == ["upload"]
}

/// Whether `path` is the Settings page proper, where the search box finds
/// settings: not the pages it leads to.
pub fn is_settings_page(path: &str) -> bool {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    matches!(segments.as_slice(), ["settings"] | ["settings", _])
}
/// The administration.
pub const ADMIN: &str = "/admin";
/// The forum of a public server.
pub const FORUM: &str = "/forum";
/// The privacy policy and the terms, outside the shell.
pub const PRIVACY: &str = "/privacy";
pub const TERMS: &str = "/terms";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upload_and_settings_carry_their_filter() {
        assert_eq!(upload(""), "/upload");
        assert_eq!(upload(" IMG 2024 "), "/upload?q=IMG+2024");
        assert_eq!(settings_search("/settings", "dark"), "/settings?q=dark");
        assert_eq!(
            settings_search("/settings/viewing", " "),
            "/settings/viewing"
        );
        assert_eq!(
            settings_search("/settings/viewing", "a&b"),
            "/settings/viewing?q=a%26b"
        );
        assert_eq!(settings_search("/trash", "x"), "/settings?q=x");
        assert!(is_upload("/upload") && is_upload("/upload/") && !is_upload("/uploads"));
        for path in ["/settings", "/settings/", "/settings/viewing"] {
            assert!(is_settings_page(path), "{path}");
        }
        for path in ["/trash", "/admin", "/settings/a/b", "/settingsx"] {
            assert!(!is_settings_page(path), "{path}");
        }
    }

    #[test]
    fn a_post_carries_the_search_it_was_opened_from() {
        assert_eq!(post(12, "", None), "/post/12");
        assert_eq!(post(12, " cat dog ", None), "/post/12?q=cat+dog");
        assert_eq!(
            post(5, "order:random", Some(-42)),
            "/post/5?q=order:random&seed=-42"
        );
        assert_eq!(post(5, "", Some(9)), "/post/5?seed=9");
        assert_eq!(tag("blue_sky"), "/tag?name=blue_sky");
        assert_eq!(tag("a&b"), "/tag?name=a%26b");
        assert_eq!(tab_of(&post(5, "cat", None)), Some(Tab::Gallery));
    }

    #[test]
    fn explore_carries_its_search() {
        assert_eq!(explore(""), "/explore");
        assert_eq!(explore("  "), "/explore");
        assert_eq!(explore("fractal -abstract"), "/explore?q=fractal+-abstract");
        assert_eq!(explore(" fractal "), "/explore?q=fractal");
        assert_eq!(explore("a&b"), "/explore?q=a%26b");
        // A public server's Trending keeps its window.
        assert_eq!(explore_over("", 7), "/explore");
        assert_eq!(explore_over("", 30), "/explore?days=30");
        assert_eq!(explore_over(" fractal ", 1), "/explore?q=fractal&days=1");
        assert_eq!(explore_over("a&b", 365), "/explore?q=a%26b&days=365");
        for path in ["/explore", "/explore/", "explore"] {
            assert!(is_explore(path), "{path}");
        }
        for path in ["/", "/explorer", "/explore/x", "/boorus", "/post/1"] {
            assert!(!is_explore(path), "{path}");
        }
    }

    #[test]
    fn the_tags_page_carries_its_filter() {
        assert_eq!(tags(""), "/tags");
        assert_eq!(tags(" fra "), "/tags?q=fra");
        assert_eq!(tags("a&b"), "/tags?q=a%26b");
        for path in ["/tags", "/tags/", "tags"] {
            assert!(is_tags(path), "{path}");
        }
        for path in ["/tag", "/", "/tagsx", "/tags/x", "/explore"] {
            assert!(!is_tags(path), "{path}");
        }
    }

    #[test]
    fn the_people_page_carries_its_filter_under_explore() {
        assert_eq!(people(""), "/people");
        assert_eq!(people(" hatsune miku "), "/people?q=hatsune+miku");
        assert_eq!(people("a&b"), "/people?q=a%26b");
        for path in ["/people", "/people/", "people"] {
            assert!(is_people(path), "{path}");
            assert_eq!(tab_of(path), Some(Tab::Explore), "{path}");
        }
        for path in ["/peoples", "/people/x", "/", "/explore"] {
            assert!(!is_people(path), "{path}");
        }
        assert!(!has_search("/people"));
    }

    #[test]
    fn the_gallery_and_its_posts_carry_a_search() {
        for path in ["/", "", "/post/12"] {
            assert!(has_search(path), "{path}");
        }
        for path in ["/tags", "/tag", "/settings", "/post", "/explore"] {
            assert!(!has_search(path), "{path}");
        }
    }

    #[test]
    fn a_page_under_a_tab_keeps_that_tab_selected() {
        for (path, tab) in [
            ("/", Tab::Gallery),
            ("", Tab::Gallery),
            ("/post/12", Tab::Gallery),
            ("/explore", Tab::Explore),
            ("/explore/", Tab::Explore),
            ("/boorus", Tab::Explore),
            ("/boorus/post", Tab::Explore),
            ("/tags", Tab::Tags),
            ("/tag", Tab::Tags),
            ("/upload", Tab::Upload),
            ("/settings", Tab::Settings),
            ("/settings/appearance", Tab::Settings),
            ("/trash", Tab::Settings),
            ("/duplicates", Tab::Settings),
            ("/help", Tab::Settings),
            ("/admin", Tab::Settings),
        ] {
            assert_eq!(tab_of(path), Some(tab), "{path}");
        }
        for path in [
            "/oidc",
            "/s/abc",
            "/privacy",
            "/terms",
            "/nowhere",
            "/post",
            "/post/1/2",
            "/settings/a/b",
        ] {
            assert_eq!(tab_of(path), None, "{path}");
        }
    }

    #[test]
    fn settings_is_its_list_and_its_groups() {
        for path in ["/settings", "/settings/", "/settings/keys"] {
            assert!(is_settings(path), "{path}");
        }
        for path in ["/", "/settingsx", "/trash", "/admin/settings"] {
            assert!(!is_settings(path), "{path}");
        }
    }

    #[test]
    fn a_guest_has_no_upload_and_room_to_sign_in() {
        assert_eq!(Tab::shown(false, false), Tab::ALL);
        assert_eq!(Tab::shown(false, true), Tab::GUEST);
        assert!(!Tab::GUEST.contains(&Tab::Upload));
        // The sign-in item comes after them, within the five.
        assert!(Tab::GUEST.len() < 5);
    }

    #[test]
    fn a_public_server_has_the_forum_in_uploads_place() {
        for guest in [false, true] {
            let tabs = Tab::shown(true, guest);
            assert_eq!(tabs, Tab::PUBLIC);
            // Upload and Settings are under the account button after them.
            assert!(!tabs.contains(&Tab::Upload) && !tabs.contains(&Tab::Settings));
            assert!(tabs.len() < 5);
        }
        let private: Vec<_> = Tab::ALL
            .iter()
            .position(|t| *t == Tab::Upload)
            .into_iter()
            .collect();
        let public: Vec<_> = Tab::PUBLIC
            .iter()
            .position(|t| *t == Tab::Forum)
            .into_iter()
            .collect();
        assert_eq!(private, public);
        // A private server's stays as it was.
        assert!(!Tab::ALL.contains(&Tab::Forum) && !Tab::GUEST.contains(&Tab::Forum));
    }

    #[test]
    fn upload_settings_and_your_profile_are_under_the_account_on_a_public_server() {
        let me = Some("Ann");
        for path in [
            "/upload",
            "/settings",
            "/settings/account",
            "/admin",
            "/trash",
            "/u/ann",
        ] {
            assert_eq!(place_of(path, true, me), Some(Place::Account), "{path}");
        }
        assert_eq!(place_of("/u/bob", true, me), None);
        assert_eq!(place_of("/u/ann", true, None), None);
        assert_eq!(
            place_of("/forum/t/3", true, me),
            Some(Place::Tab(Tab::Forum))
        );
        assert_eq!(place_of("/", true, me), Some(Place::Tab(Tab::Gallery)));
        // A private server keeps them in the navigation.
        assert_eq!(
            place_of("/upload", false, me),
            Some(Place::Tab(Tab::Upload))
        );
        assert_eq!(
            place_of("/settings", false, me),
            Some(Place::Tab(Tab::Settings))
        );
        assert_eq!(place_of("/u/ann", false, me), None);
        assert_eq!(place_of("/forum", false, me), None);
    }

    #[test]
    fn every_tab_leads_to_a_page_under_it() {
        assert!(Tab::ALL.len() <= 5, "five items at most");
        for tab in Tab::ALL.into_iter().chain(Tab::PUBLIC) {
            assert_eq!(tab_of(tab.href()), Some(tab), "{tab:?}");
        }
    }

    #[test]
    fn the_forum_and_profiles_have_addresses_of_their_own() {
        assert_eq!(forum(""), "/forum");
        assert_eq!(forum(" sky  tags "), "/forum?q=sky++tags");
        assert_eq!(forum_category(2, ""), "/forum/c/2");
        assert_eq!(forum_category(2, "a&b"), "/forum/c/2?q=a%26b");
        assert_eq!(forum_thread(7), "/forum/t/7");
        assert_eq!(forum_post(7, 12), "/forum/t/7#p12");
        assert_eq!(forum_page("/forum"), Some(ForumPage::Index));
        assert_eq!(forum_page("/forum/c/2"), Some(ForumPage::Category(2)));
        assert_eq!(forum_page("/forum/t/7"), Some(ForumPage::Thread(7)));
        for path in ["/forum/c/x", "/forum/t", "/forums", "/", "/forum/x/1"] {
            assert_eq!(forum_page(path), None, "{path}");
        }
        for path in ["/forum", "/forum/c/2", "/forum/t/7"] {
            assert_eq!(tab_of(path), Some(Tab::Forum), "{path}");
        }
        assert_eq!(profile("ann.b-c_d"), "/u/ann.b-c_d");
        assert_eq!(profile("a b"), "/u/a%20b");
        assert_eq!(profile_name(&profile("a b")).as_deref(), Some("a b"));
        assert_eq!(profile_name("/u/ann"), Some("ann".into()));
        assert_eq!(profile_name("/u"), None);
        assert_eq!(profile_name("/u/a/b"), None);
        assert_eq!(tab_of("/u/ann"), None);
    }

    #[test]
    fn a_search_goes_into_the_address_readably() {
        assert_eq!(gallery(""), "/");
        assert_eq!(gallery("   "), "/");
        assert_eq!(gallery("cat"), "/?q=cat");
        assert_eq!(
            gallery(" blue_sky  rating:safe "),
            "/?q=blue_sky++rating:safe"
        );
        assert_eq!(gallery("-{girl boots}"), "/?q=-%7Bgirl+boots%7D");
        // The signs that would end the value or change it are escaped.
        assert_eq!(query_value("a&b=c#d+e%f?"), "a%26b%3Dc%23d%2Be%25f%3F");
        assert_eq!(query_value("猫"), "%E7%8C%AB");
    }

    /// The value of `name` in the query of `address`, read back as a browser
    /// does (`URLSearchParams`).
    fn param(address: &str, name: &str) -> Option<String> {
        let (_, query) = address.split_once('?')?;
        query.split('&').find_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            let value = value.replace('+', " ");
            (key == name).then(|| {
                percent_encoding::percent_decode_str(&value)
                    .decode_utf8_lossy()
                    .into_owned()
            })
        })
    }

    #[test]
    fn the_booru_browser_carries_its_search_and_sites() {
        use crate::logic::boorus::{from_address, parse_sites, sites_param};
        assert_eq!(boorus("", ""), "/boorus");
        assert_eq!(
            boorus(" rating:g landscape ", "danbooru,e926"),
            "/boorus?q=rating:g+landscape&s=danbooru,e926"
        );
        assert_eq!(boorus("", "e926"), "/boorus?s=e926");
        assert_eq!(
            booru_post("https://danbooru.donmai.us/posts/7", "sky", "danbooru"),
            "/boorus/post?u=https://danbooru.donmai.us/posts/7&q=sky&s=danbooru"
        );
        let path = |to: String| to.split('?').next().unwrap_or_default().to_owned();
        assert_eq!(tab_of(&path(boorus("sky", ""))), Some(Tab::Explore));
        assert_eq!(tab_of(&path(booru_post("x", "", ""))), Some(Tab::Explore));
        assert!(!has_search("/boorus"));

        let sites = sites_param(&[
            starbooru_core::RemoteSite {
                api: starbooru_core::BooruApi::Moebooru,
                base: "https://yande.re".into(),
            },
            starbooru_core::RemoteSite {
                api: starbooru_core::BooruApi::Blombooru,
                base: "http://nas:8000/a,b&c%20d".into(),
            },
        ]);
        for text in [
            "",
            "blue_sky rating:safe",
            "https://gelbooru.com/index.php?page=post&s=view&id=7#comments",
            "panty_&_stocking_with_garterbelt 100% c++ ~either a,b",
            "già_visto ★ 日本語 (fate) ?=/",
            "~7E ~zz ~",
        ] {
            // Through the address and back, as a reload or a shared link does.
            let grid = boorus(text, &sites);
            let q = param(&grid, "q").unwrap_or_default();
            assert_eq!(from_address(&q), text.trim(), "{grid}");
            let s = param(&grid, "s").unwrap_or_default();
            assert_eq!(s, sites);
            assert_eq!(parse_sites(&s).len(), 2);
            let post = booru_post(text, text, &sites);
            let u = param(&post, "u").unwrap_or_default();
            assert_eq!(from_address(&u), text, "{post}");
            assert_eq!(param(&post, "s").as_deref(), Some(sites.as_str()));
            // Nothing in a value ends it or starts another.
            for address in [&grid, &post] {
                let (_, query) = address.split_once('?').unwrap_or_default();
                assert!(!query.contains('#'), "{address}");
                assert!(query.split('&').count() <= 3, "{address}");
            }
        }
        // A link from the reference, which encoded the same way.
        let old = "/boorus?q=panty_~26_stocking&s=danbooru";
        assert_eq!(from_address(&param(old, "q").unwrap()), "panty_&_stocking");
    }

    #[test]
    fn the_exact_search_is_the_same_search_uncorrected() {
        assert_eq!(exact("fractl"), "/?q=exact:true+fractl");
        assert_eq!(exact(" fractl  -blu "), "/?q=exact:true+fractl++-blu");
        // Once is enough, wherever it was typed.
        assert_eq!(exact("fractl exact:true"), "/?q=fractl+exact:true");
        // It goes first, so an open group can't take it in.
        assert_eq!(exact("{girl boo"), "/?q=exact:true+%7Bgirl+boo");
        let q = starbooru_core::with_exact("{girl boo");
        assert!(starbooru_core::parse_query(&q).is_ok_and(|p| p.exact));
    }
}
