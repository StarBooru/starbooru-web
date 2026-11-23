//! The server address on the sign-in screen: which one it starts with, and
//! how what was typed becomes the one the app keeps.

use std::net::Ipv4Addr;

/// The address the sign-in screen starts with: the server signed in to last,
/// else the one the build named (`STARBOORU_SERVER`), else the page's own
/// origin, as the web app is normally served by the server itself.
pub fn prefill(last: &str, built_in: Option<&str>, origin: &str) -> String {
    [last, built_in.unwrap_or_default(), origin]
        .into_iter()
        .map(str::trim)
        .find(|s| !s.is_empty())
        .unwrap_or_default()
        .to_owned()
}

/// `input` as the address the app keeps: trimmed, with no slash or API path
/// at its end. Without a scheme it gets `https://`, or `http://` for a machine
/// at home (an IP address, `localhost`, a name with no dot, a `.local` name),
/// where servers seldom have a certificate.
pub fn normalize_server(input: &str) -> String {
    let mut s = input.trim().trim_end_matches('/');
    // Pasted from the API's own address.
    if let Some(root) = s.strip_suffix("/api/v1") {
        s = root.trim_end_matches('/');
    }
    if s.is_empty() || s.contains("://") {
        return s.to_owned();
    }
    let scheme = if at_home(host(s)) { "http" } else { "https" };
    format!("{scheme}://{s}")
}

/// A kept address as the pages show it, without its scheme or a slash at
/// its end: `https://booru.example/` is `booru.example`.
pub fn shown(address: &str) -> &str {
    let address = address.trim();
    let bare = address.split_once("://").map_or(address, |(_, rest)| rest);
    bare.trim_end_matches('/')
}

/// The host of an address with no scheme: before its port and its path.
fn host(address: &str) -> &str {
    let authority = address.split(['/', '?', '#']).next().unwrap_or_default();
    if authority.starts_with('[') {
        // An IPv6 address, with its brackets.
        return authority.split_once(']').map_or(authority, |(h, _)| h);
    }
    authority.split(':').next().unwrap_or_default()
}

fn at_home(host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    host.starts_with('[')
        || host.parse::<Ipv4Addr>().is_ok()
        || host == "localhost"
        || !host.contains('.')
        || host.ends_with(".local")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_last_server_comes_first_then_the_builds_then_the_page() {
        let origin = "https://booru.example";
        assert_eq!(
            prefill("http://nas:8080", Some("http://dev:1"), origin),
            "http://nas:8080"
        );
        assert_eq!(prefill("", Some("http://dev:1"), origin), "http://dev:1");
        assert_eq!(prefill("  ", None, origin), origin);
        assert_eq!(prefill("", Some(""), origin), origin);
        assert_eq!(prefill("", None, ""), "");
    }

    #[test]
    fn a_typed_address_is_tidied() {
        assert_eq!(
            normalize_server("  https://booru.example/  "),
            "https://booru.example"
        );
        assert_eq!(
            normalize_server("https://booru.example/api/v1"),
            "https://booru.example"
        );
        assert_eq!(
            normalize_server("https://booru.example/api/v1/"),
            "https://booru.example"
        );
        assert_eq!(
            normalize_server("https://x.example/booru/"),
            "https://x.example/booru"
        );
        assert_eq!(
            normalize_server("HTTP://Booru.example"),
            "HTTP://Booru.example"
        );
        assert_eq!(normalize_server(""), "");
        assert_eq!(normalize_server(" / "), "");
    }

    #[test]
    fn an_address_shows_without_its_scheme() {
        assert_eq!(shown("https://booru.example/"), "booru.example");
        assert_eq!(shown(" http://127.0.0.1:8088 "), "127.0.0.1:8088");
        assert_eq!(shown("https://x.example/booru"), "x.example/booru");
        assert_eq!(shown("nas:8080"), "nas:8080");
        assert_eq!(shown(""), "");
    }

    #[test]
    fn an_address_without_a_scheme_gets_one() {
        assert_eq!(normalize_server("booru.example"), "https://booru.example");
        assert_eq!(
            normalize_server("booru.example:8443/x"),
            "https://booru.example:8443/x"
        );
        assert_eq!(normalize_server("127.0.0.1:8088"), "http://127.0.0.1:8088");
        assert_eq!(normalize_server("192.168.1.20"), "http://192.168.1.20");
        assert_eq!(normalize_server("localhost:8080/"), "http://localhost:8080");
        assert_eq!(normalize_server("[::1]:8080"), "http://[::1]:8080");
        assert_eq!(normalize_server("nas:8080"), "http://nas:8080");
        assert_eq!(normalize_server("NAS.local"), "http://NAS.local");
    }
}
