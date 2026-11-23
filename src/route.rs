//! The app's addresses. They keep the reference's paths, so links people saved
//! still open; `app.rs` maps each to its page. Building and reading them is
//! plain Rust here, tested on the host.

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

/// The navigation's items, five at most (the reference's design rules): a new
/// page goes under one of these.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Gallery,
    Explore,
    Tags,
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

    /// Where the item leads.
    pub fn href(self) -> &'static str {
        match self {
            Tab::Gallery => "/",
            Tab::Explore => "/explore",
            Tab::Tags => "/tags",
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
        // The booru browser is the other side of Explore.
        ["explore"] | ["boorus"] | ["boorus", "post"] => Some(Tab::Explore),
        ["tags"] | ["tag"] => Some(Tab::Tags),
        ["upload"] => Some(Tab::Upload),
        ["settings"] | ["trash"] | ["duplicates"] | ["help"] | ["admin"] => Some(Tab::Settings),
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

/// The list of tags.
pub const TAGS: &str = "/tags";
/// The Upload page.
pub const UPLOAD: &str = "/upload";
/// The search cheat sheet.
pub const HELP: &str = "/help";

#[cfg(test)]
mod tests {
    use super::*;

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
            "/settings/x",
        ] {
            assert_eq!(tab_of(path), None, "{path}");
        }
    }

    #[test]
    fn every_tab_leads_to_a_page_under_it() {
        assert!(Tab::ALL.len() <= 5, "five items at most");
        for tab in Tab::ALL {
            assert_eq!(tab_of(tab.href()), Some(tab), "{tab:?}");
        }
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
