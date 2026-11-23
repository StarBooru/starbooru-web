//! Looking a post's picture up on other sites (`components/reverse.rs`):
//! which copy goes, and how each site gets it. A site that fetches the picture by
//! its address needs this server on the internet; one that takes an upload
//! doesn't.

use std::net::{Ipv4Addr, Ipv6Addr};

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use starbooru_core::{MediaKind, Post};

/// A site that finds where a picture came from, or pictures like it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Site {
    Lens,
    Iqdb,
    SauceNao,
    TraceMoe,
}

impl Site {
    pub const ALL: [Site; 4] = [Site::Lens, Site::Iqdb, Site::SauceNao, Site::TraceMoe];
}

/// How a site gets the picture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Way {
    /// Sent as a file, from a form posted into a new tab as the site's own
    /// page would.
    Upload {
        action: &'static str,
        field: &'static str,
    },
    /// The site opens with the picture's address, and fetches it from there.
    Address(String),
    /// The site opens, and the picture goes on the clipboard to paste there.
    Paste(&'static str),
    /// The site opens, and the picture is saved to upload there.
    Save(&'static str),
}

impl Way {
    /// Whether the picture itself has to be at hand first.
    pub fn needs_file(&self) -> bool {
        !matches!(self, Way::Address(_))
    }
}

/// What the browser can do.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Browser {
    /// It can put a picture on the clipboard: over https, or on this computer.
    pub clipboard: bool,
    /// A touch screen and no mouse, where pasting a picture into a page is
    /// hard to find.
    pub touch: bool,
}

/// Google's page for searching by a picture, which takes a paste or a file.
const LENS_PAGE: &str = "https://www.google.com/?olud";
const TRACE_PAGE: &str = "https://trace.moe/";

/// How `site` gets the picture at `picture`, an absolute address.
/// `reachable` is whether sites on the internet can fetch from there.
pub fn way(site: Site, picture: &str, reachable: bool, browser: Browser) -> Way {
    let by_hand = |page| {
        if browser.clipboard && !browser.touch {
            Way::Paste(page)
        } else {
            Way::Save(page)
        }
    };
    match site {
        // Not an upload: Google's consent cookie isn't sent with a form
        // posted from another site, so in the EU an upload lands on the
        // consent page and the picture is lost. A link gets through.
        Site::Lens if reachable => Way::Address(format!(
            "https://lens.google.com/uploadbyurl?url={}",
            encode(picture)
        )),
        Site::Lens => by_hand(LENS_PAGE),
        Site::Iqdb => Way::Upload {
            action: "https://iqdb.org/",
            field: "file",
        },
        Site::SauceNao => Way::Upload {
            action: "https://saucenao.com/search.php",
            field: "file",
        },
        Site::TraceMoe if reachable => {
            Way::Address(format!("https://trace.moe/?url={}", encode(picture)))
        }
        // Its page has no form to post to; it takes an address, a drop or
        // a paste.
        Site::TraceMoe => by_hand(TRACE_PAGE),
    }
}

/// What a query value keeps as it is, as `encodeURIComponent` does.
const COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

fn encode(s: &str) -> String {
    utf8_percent_encode(s, COMPONENT).to_string()
}

/// The largest file sent: iqdb's limit, the lowest of the four.
pub const MAX_BYTES: u64 = 8 * 1024 * 1024;
/// iqdb's longest side.
const MAX_SIDE: i32 = 7500;

/// The copy to send, as the server-relative address the API gave: the
/// thumbnail, which is a frame for a video and the first one for a GIF,
/// and plenty to search by. Without one, a picture's viewing copy, or its
/// file when every site takes it. `None` when there's nothing to send, as
/// for a video without a thumbnail.
pub fn picture(p: &Post) -> Option<&str> {
    if let Some(thumb) = p.thumb_url.as_deref().filter(|u| file_type(u).is_some()) {
        return Some(thumb);
    }
    if p.kind == MediaKind::Video {
        return None;
    }
    if let Some(sample) = p.sample_url.as_deref().filter(|u| file_type(u).is_some()) {
        return Some(sample);
    }
    let small = u64::try_from(p.file_size).is_ok_and(|size| size <= MAX_BYTES);
    let fits = p.width.max(p.height).is_none_or(|side| side <= MAX_SIDE);
    (small && fits && file_type(&p.file_url).is_some()).then_some(p.file_url.as_str())
}

/// Whether `p` can be looked up: not from the recycle bin, and only with a
/// picture to send.
pub fn offered(p: &Post) -> bool {
    !p.deleted && picture(p).is_some()
}

/// The type and extension of the picture at `url`, from its path, for the
/// formats all four sites read.
pub fn file_type(url: &str) -> Option<(&'static str, &'static str)> {
    let path = url.split(['?', '#']).next().unwrap_or_default();
    let (_, ext) = path.rsplit('/').next()?.rsplit_once('.')?;
    match ext.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => Some(("image/jpeg", "jpg")),
        "png" => Some(("image/png", "png")),
        "gif" => Some(("image/gif", "gif")),
        _ => None,
    }
}

/// The name the copy is sent or saved under. Not the original's: its hash
/// would say more than the site needs.
pub fn file_name(post_id: i64, url: &str) -> String {
    let ext = file_type(url).map_or("jpg", |(_, ext)| ext);
    format!("starbooru-{post_id}.{ext}")
}

/// The keys that paste, from the browser's user agent.
pub fn paste_keys(user_agent: &str) -> &'static str {
    let apple = ["Macintosh", "Mac OS X", "iPhone", "iPad"]
        .iter()
        .any(|s| user_agent.contains(s));
    if apple { "⌘V" } else { "Ctrl+V" }
}

/// Whether sites on the internet can fetch `url`. Not from this computer, a
/// home or office network, or a private network such as Tailscale's. When
/// it's unclear, not: the other ways work from anywhere.
pub fn reachable(url: &str) -> bool {
    let Some((scheme, rest)) = url.split_once("://") else {
        return false;
    };
    if !matches!(scheme.to_ascii_lowercase().as_str(), "http" | "https") {
        return false;
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    if let Some(inner) = host.strip_prefix('[') {
        let ip = inner.split(']').next().unwrap_or_default();
        return ip.parse().is_ok_and(public_v6);
    }
    let name = host.split(':').next().unwrap_or_default();
    let name = name.trim_end_matches('.').to_ascii_lowercase();
    if let Ok(ip) = name.parse::<Ipv4Addr>() {
        return public_v4(ip);
    }
    public_name(&name)
}

fn public_v4(ip: Ipv4Addr) -> bool {
    let [a, b, ..] = ip.octets();
    // 100.64.0.0/10: carrier-grade NAT, and Tailscale's addresses.
    let shared = a == 100 && (64..128).contains(&b);
    // 198.18.0.0/15, for testing networks.
    let benchmark = a == 198 && (b == 18 || b == 19);
    !(ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.is_documentation()
        || shared
        || benchmark
        || a == 0
        || a >= 240)
}

fn public_v6(ip: Ipv6Addr) -> bool {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return public_v4(v4);
    }
    let [first, second, ..] = ip.segments();
    // fc00::/7, unique local (Tailscale's among them); fe80::/10, link-local;
    // 2001:db8::/32, documentation.
    let unique_local = first & 0xfe00 == 0xfc00;
    let link_local = first & 0xffc0 == 0xfe80;
    let documentation = first == 0x2001 && second == 0x0db8;
    !(ip.is_loopback() || ip.is_unspecified() || unique_local || link_local || documentation)
}

/// Names only a local network or a VPN knows. Tailscale's `ts.net` names
/// can be public through Funnel, but mostly aren't.
const LOCAL_SUFFIXES: [&str; 13] = [
    "localhost",
    "local",
    "lan",
    "home",
    "internal",
    "intranet",
    "private",
    "corp",
    "home.arpa",
    "ts.net",
    "test",
    "invalid",
    "example",
];

fn public_name(name: &str) -> bool {
    if name.is_empty() || !name.contains('.') {
        return false;
    }
    !LOCAL_SUFFIXES.iter().any(|suffix| {
        name == *suffix
            || name
                .strip_suffix(suffix)
                .is_some_and(|head| head.ends_with('.'))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn post(kind: MediaKind) -> Post {
        let mut p: Post = serde_json::from_value(serde_json::json!({
            "id": 1, "kind": "image", "mime": "image/png", "file_size": 1_626_018,
            "width": 1600, "height": 1200, "sha256": "ab", "rating": "safe", "score": 0,
            "my_vote": 0, "source": null, "tags": [],
            "created_at": "2026-01-01T00:00:00Z", "deleted": false,
            "file_url": "/media/original/ab.png?exp=1&sig=2",
            "thumb_url": "/media/thumb/ab.jpg?exp=1&sig=3",
            "sample_url": "/media/sample/ab.jpg?exp=1&sig=4",
        }))
        .unwrap();
        p.kind = kind;
        p
    }

    const HOME: &str = "http://192.168.1.20:8080/media/thumb/ab.jpg?exp=1&sig=3";
    const PUBLIC: &str = "https://booru.example.org/media/thumb/ab.jpg?exp=1&sig=3";
    const DESK: Browser = Browser {
        clipboard: true,
        touch: false,
    };

    #[test]
    fn the_thumbnail_goes_and_a_video_needs_one() {
        let image = post(MediaKind::Image);
        assert_eq!(picture(&image), Some("/media/thumb/ab.jpg?exp=1&sig=3"));
        // No thumbnail: the viewing copy, then the file if iqdb takes it.
        let mut bare = image.clone();
        bare.thumb_url = None;
        assert_eq!(picture(&bare), Some("/media/sample/ab.jpg?exp=1&sig=4"));
        bare.sample_url = None;
        assert_eq!(picture(&bare), Some("/media/original/ab.png?exp=1&sig=2"));
        let mut big = bare.clone();
        big.file_size = 9 * 1024 * 1024;
        assert_eq!(picture(&big), None);
        let mut huge = bare.clone();
        huge.width = Some(8000);
        assert_eq!(picture(&huge), None);
        let mut webp = bare.clone();
        webp.file_url = "/media/original/ab.webp?exp=1".into();
        assert_eq!(picture(&webp), None);
        // A GIF without a thumbnail goes whole when it's small.
        let mut gif = post(MediaKind::Animation);
        gif.thumb_url = None;
        gif.sample_url = None;
        gif.file_url = "/media/original/ab.gif?exp=1".into();
        assert_eq!(picture(&gif), Some("/media/original/ab.gif?exp=1"));
        // A video: its thumbnail is a frame, and its file never goes.
        let mut video = post(MediaKind::Video);
        assert_eq!(picture(&video), Some("/media/thumb/ab.jpg?exp=1&sig=3"));
        video.thumb_url = None;
        assert_eq!(picture(&video), None);
        assert!(!offered(&video));
    }

    #[test]
    fn nothing_from_the_recycle_bin() {
        let mut p = post(MediaKind::Image);
        assert!(offered(&p));
        p.deleted = true;
        assert!(!offered(&p));
    }

    #[test]
    fn file_types_come_from_the_path() {
        assert_eq!(
            file_type("/media/thumb/ab.jpg?exp=1&sig=2.png"),
            Some(("image/jpeg", "jpg"))
        );
        assert_eq!(file_type("/x/a.JPEG"), Some(("image/jpeg", "jpg")));
        assert_eq!(file_type("/x/a.png#b"), Some(("image/png", "png")));
        assert_eq!(file_type("/x/a.gif"), Some(("image/gif", "gif")));
        assert_eq!(file_type("/x/a.webp"), None);
        assert_eq!(file_type("/x.jpg/a"), None);
        assert_eq!(file_type(""), None);
    }

    #[test]
    fn the_copy_is_named_after_the_post() {
        assert_eq!(
            file_name(7, "/media/thumb/ab.jpg?sig=x.png"),
            "starbooru-7.jpg"
        );
        assert_eq!(file_name(7, "/media/original/ab.gif"), "starbooru-7.gif");
        assert_eq!(file_name(7, ""), "starbooru-7.jpg");
    }

    #[test]
    fn paste_is_command_v_on_apple_devices() {
        let mac = "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15";
        assert_eq!(paste_keys(mac), "⌘V");
        let linux = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/131.0";
        assert_eq!(paste_keys(linux), "Ctrl+V");
        assert_eq!(paste_keys(""), "Ctrl+V");
    }

    #[test]
    fn upload_sites_take_the_file_from_anywhere() {
        for reachable in [true, false] {
            assert_eq!(
                way(Site::Iqdb, HOME, reachable, DESK),
                Way::Upload {
                    action: "https://iqdb.org/",
                    field: "file"
                }
            );
            assert_eq!(
                way(Site::SauceNao, HOME, reachable, Browser::default()),
                Way::Upload {
                    action: "https://saucenao.com/search.php",
                    field: "file"
                }
            );
        }
    }

    #[test]
    fn a_reachable_server_gives_lens_and_trace_moe_the_address() {
        let lens = way(Site::Lens, PUBLIC, true, DESK);
        assert_eq!(
            lens,
            Way::Address(
                "https://lens.google.com/uploadbyurl?url=https%3A%2F%2Fbooru.example.org%2Fmedia%2Fthumb%2Fab.jpg%3Fexp%3D1%26sig%3D3".into()
            )
        );
        assert!(!lens.needs_file());
        let Way::Address(trace) = way(Site::TraceMoe, PUBLIC, true, Browser::default()) else {
            panic!("not an address");
        };
        // The signature stays part of the picture's address, not trace.moe's.
        assert!(trace.starts_with("https://trace.moe/?url=https%3A%2F%2F"));
        assert!(!trace.contains("&sig"));
    }

    #[test]
    fn an_unreachable_server_hands_the_picture_over_by_hand() {
        assert_eq!(way(Site::Lens, HOME, false, DESK), Way::Paste(LENS_PAGE));
        assert_eq!(
            way(Site::TraceMoe, HOME, false, DESK),
            Way::Paste("https://trace.moe/")
        );
        // No clipboard for pictures, as on plain http, or a phone: saved.
        let plain = Browser {
            clipboard: false,
            touch: false,
        };
        let phone = Browser {
            clipboard: true,
            touch: true,
        };
        for browser in [plain, phone] {
            assert_eq!(
                way(Site::TraceMoe, HOME, false, browser),
                Way::Save(TRACE_PAGE)
            );
            assert_eq!(way(Site::Lens, HOME, false, browser), Way::Save(LENS_PAGE));
        }
        assert!(Way::Paste(TRACE_PAGE).needs_file());
        assert!(Way::Save(TRACE_PAGE).needs_file());
    }

    #[test]
    fn home_servers_are_not_reachable() {
        for url in [
            "http://localhost:8080/media/x.jpg",
            "http://app.localhost/x",
            "http://127.0.0.1:8088/media/x.jpg",
            "http://10.0.0.5/x",
            "http://172.16.4.1/x",
            "http://192.168.1.20:8080/x",
            "http://169.254.0.1/x",
            "http://100.101.102.103/x",
            "http://0.0.0.0/x",
            "http://[::1]:8080/x",
            "http://[fd7a:115c:a1e0::1]/x",
            "http://[fe80::1]/x",
            "http://[::ffff:192.168.0.1]/x",
            "http://nas/x",
            "http://nas.local:8080/x",
            "http://booru.lan/x",
            "http://booru.home.arpa/x",
            "https://box.tail1234.ts.net/x",
            "http://user:pw@192.168.1.2/x",
            "http://booru.local./x",
            "ftp://example.org/x",
            "/media/thumb/x.jpg",
            "",
        ] {
            assert!(!reachable(url), "{url}");
        }
    }

    #[test]
    fn public_servers_are_reachable() {
        for url in [
            "https://booru.example.org/media/x.jpg",
            "http://203.0.114.7:8080/x",
            "https://8.8.8.8/x",
            "https://[2a00:1450:4001::1]/x",
            "HTTPS://Booru.Example.Org/x",
            "https://user@booru.example.org/x",
            "https://local.example.org/x",
            "https://homes.org/x",
        ] {
            assert!(reachable(url), "{url}");
        }
    }
}
