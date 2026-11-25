//! The Upload page's plain parts: which files are taken, the addresses a drop
//! or a paste carries, a folder's name as a tag, sizes and the words beside a
//! transfer's bar, and what the server's refusal of a file means.

use starbooru_core::{AiStatus, FORMATS, Transfer, normalize_tag, post_link};

use super::tags::{BadTag, parse_tags};
// Kept here too: the upload pages and the admin panel import it from here.
pub use super::viewer::human_size;

/// Names the server's file types go by, which save reading the file.
const MEDIA_EXTENSIONS: [&str; 16] = [
    "jpg", "jpeg", "jfif", "png", "gif", "webp", "bmp", "tif", "tiff", "avif", "jxl", "mp4", "m4v",
    "webm", "mkv", "mov",
];

const VIDEO_EXTENSIONS: [&str; 5] = ["mp4", "m4v", "webm", "mkv", "mov"];

/// What a file's name and the browser's type for it say about taking it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Take,
    Skip,
    /// Neither says: its first bytes do (`starbooru_core::is_media`).
    Look,
}

fn extension(name: &str) -> Option<&str> {
    let (stem, ext) = name.rsplit_once('.')?;
    (!stem.is_empty()).then_some(ext)
}

/// An image or video by its name. Hidden files (`.DS_Store`, `._x.jpg`) are not.
pub fn is_media_name(name: &str) -> bool {
    !name.starts_with('.')
        && extension(name)
            .is_some_and(|ext| MEDIA_EXTENSIONS.iter().any(|m| m.eq_ignore_ascii_case(ext)))
}

/// One of the server's formats by the type the browser gives a file.
fn is_media_type(mime: &str) -> bool {
    FORMATS
        .iter()
        .any(|(m, ..)| m.eq_ignore_ascii_case(mime.trim()))
}

/// Whether to take a file called `name`, of type `mime` and `size` bytes. A
/// name or a type can only say yes: a saved picture may be called just
/// `anime`, so the rest are looked into. The server checks every file again.
pub fn verdict(name: &str, mime: &str, size: u64) -> Verdict {
    if name.starts_with('.') || size == 0 {
        Verdict::Skip
    } else if is_media_name(name) || is_media_type(mime) {
        Verdict::Take
    } else {
        Verdict::Look
    }
}

/// Whether a file is a video, for a preview that plays its first frame.
pub fn is_video(name: &str, mime: &str) -> bool {
    mime.starts_with("video/")
        || extension(name)
            .is_some_and(|ext| VIDEO_EXTENSIONS.iter().any(|v| v.eq_ignore_ascii_case(ext)))
}

/// Names of the files left out, for a notice: the first few, and how many
/// more there were.
pub fn skipped_names(names: &[String]) -> (String, usize) {
    const SHOWN: usize = 3;
    let shown: Vec<&str> = names.iter().take(SHOWN).map(String::as_str).collect();
    (shown.join(", "), names.len().saturating_sub(SHOWN))
}

/// The name of the folder holding the file at `path` (`Summer 2024/a.jpg`,
/// or `/Summer 2024/a.jpg` from a dropped folder), as a tag: `summer_2024`.
pub fn folder_tag(path: &str) -> Option<String> {
    let (folder, _) = path.trim_end_matches('/').rsplit_once('/')?;
    let name = folder.rsplit('/').next()?;
    normalize_tag(name).ok()
}

/// What to say beside a transfer's bar: `42% · 1.2 MB/s`, or the bytes so
/// far when the size isn't known. The speed is left out until it is measured.
pub fn transfer_label(progress: &Transfer) -> String {
    let how_far = match progress.percent() {
        Some(percent) => format!("{percent}%"),
        None => human_size(progress.done),
    };
    if progress.speed == 0 {
        how_far
    } else {
        format!("{how_far} · {}/s", human_size(progress.speed))
    }
}

/// The `http(s)://` addresses in pasted or typed text, one per word, without
/// repeats.
pub fn parse_urls(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        let lower = word.to_ascii_lowercase();
        let long_enough = lower
            .split_once("://")
            .is_some_and(|(_, rest)| !rest.is_empty());
        if (lower.starts_with("http://") || lower.starts_with("https://"))
            && long_enough
            && !out.iter().any(|u| u == word)
        {
            out.push(word.to_owned());
        }
    }
    out
}

/// The address of the first picture in a piece of HTML, as a drag out of a
/// browser carries it.
fn img_src(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let tag = lower.find("<img")?;
    let end = tag.checked_add(lower.get(tag..)?.find('>')?)?;
    let attrs = html.get(tag..end)?;
    let at = attrs.to_ascii_lowercase().find("src=")?.checked_add(4)?;
    let value = attrs.get(at..)?.trim_start();
    let src = match value.chars().next()? {
        quote @ ('"' | '\'') => value.get(1..)?.split(quote).next()?,
        _ => value.split_whitespace().next()?,
    };
    let src = src.trim().replace("&amp;", "&");
    let src = match src.strip_prefix("//") {
        Some(rest) => format!("https://{rest}"),
        None => src,
    };
    parse_urls(&src).into_iter().next()
}

/// What to import for a link or a picture dragged out of a browser, from the
/// forms a drop carries it in. A link to a post on a booru is kept, as the
/// server finds the original and its tags from it; otherwise a dragged
/// picture counts before the link around it.
pub fn drop_urls(uri_list: &str, html: &str, text: &str) -> Vec<String> {
    let listed: String = uri_list
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join(" ");
    let mut links = parse_urls(&listed);
    if links.is_empty() {
        links = parse_urls(text);
    }
    let is_post = |url: &String| post_link(url, &[]).is_some();
    match (links.as_slice(), img_src(html)) {
        ([], Some(picture)) => vec![picture],
        ([link], Some(picture)) if !is_post(link) => vec![picture],
        _ => links,
    }
}

/// Whether a drop is taken as addresses and not as files. A browser may hand
/// over a copy of a dragged picture as a file; the address is better when it
/// leads to the post, with its original and its tags.
pub fn drop_is_urls(files: usize, urls: &[String]) -> bool {
    !urls.is_empty() && (files == 0 || urls.iter().any(|url| post_link(url, &[]).is_some()))
}

/// Whether an address ends like a picture's, so it can stand as its own
/// preview. A link to a page can't.
pub fn is_picture_url(url: &str) -> bool {
    let path = url.split(['?', '#']).next().unwrap_or_default();
    let Some((_, ext)) = path.rsplit_once('.') else {
        return false;
    };
    let ext = ext.to_ascii_lowercase();
    matches!(
        ext.as_str(),
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "avif" | "bmp"
    )
}

/// The site an address is on, as a tile's caption: `gelbooru.com`.
pub fn host_of(url: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    host.trim_start_matches("www.").to_owned()
}

/// A format's short name, as `PNG`, for one StarBooru keeps.
pub fn format_of(mime: &str) -> Option<String> {
    FORMATS
        .iter()
        .find(|(m, ..)| m.eq_ignore_ascii_case(mime.trim()))
        .map(|(_, ext, ..)| ext.to_ascii_uppercase())
}

/// The same from a name or an address that ends like a kept format's.
pub fn format_by_name(name: &str) -> Option<String> {
    let path = name.split(['?', '#']).next().unwrap_or_default();
    let (stem, ext) = path.rsplit_once('.')?;
    if stem.is_empty() || stem.ends_with('/') || ext.contains('/') {
        return None;
    }
    let ext = match ext.to_ascii_lowercase().as_str() {
        "jpeg" | "jpe" | "jfif" => "jpg".to_owned(),
        "tif" => "tiff".to_owned(),
        "m4v" => "mp4".to_owned(),
        other => other.to_owned(),
    };
    FORMATS
        .iter()
        .any(|(_, e, ..)| *e == ext)
        .then(|| ext.to_ascii_uppercase())
}

/// What's wrong with the tags of a batch, if anything: each file gets the
/// shared ones and its own.
pub fn tag_problem<'a>(shared: &str, own: impl IntoIterator<Item = &'a str>) -> Option<BadTag> {
    own.into_iter()
        .find_map(|tags| parse_tags(&format!("{shared} {tags}")).err())
}

/// Why the server turned a file away, as far as the app tells them apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The same file is already a post.
    Duplicate,
    /// Over the server's upload limit.
    TooBig,
    /// Not a picture or a video the server keeps.
    Unsupported,
    /// Anything else: the server's own words say what.
    Other,
}

/// What a failed upload's status and error code mean.
pub fn refusal(status: Option<u16>, code: Option<&str>) -> Refusal {
    match (status, code) {
        (_, Some("duplicate")) => Refusal::Duplicate,
        (Some(413), _) => Refusal::TooBig,
        (Some(415), _) => Refusal::Unsupported,
        _ => Refusal::Other,
    }
}

/// The post a duplicate is, from the server's words for it ("this file is
/// already post #12").
pub fn duplicate_post(message: &str) -> Option<i64> {
    let (_, rest) = message.split_once('#')?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok().filter(|id| *id > 0)
}

/// Whether a file of `size` bytes is over the server's limit. A limit of 0
/// is no limit.
pub fn too_big(size: u64, limit: u64) -> bool {
    limit > 0 && size > limit
}

/// What the upload's AI switch does, as its label says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiSwitch {
    /// The AI tags the uploads.
    Tags,
    /// It tags them and then finds their people: turned off, the server
    /// does neither.
    TagsAndPeople,
}

/// The switch `GET /ai` calls for, and whether it starts on, as the server
/// tags an upload whose uploader doesn't say. `None` when the server can't
/// tag. The server finds the people of an upload unless its uploader turns
/// the AI off, so only a switch that starts on can stop that, and says so;
/// one that starts off adds the tags, and turned off again leaves it to the
/// server ([`chosen`]).
pub fn ai_switch(status: &AiStatus) -> Option<(AiSwitch, bool)> {
    match (status.tagging, status.people && status.autotag) {
        (false, _) => None,
        (true, false) => Some((AiSwitch::Tags, status.autotag)),
        (true, true) => Some((AiSwitch::TagsAndPeople, true)),
    }
}

/// What an upload says of the AI with the switch `on`, when the server
/// would have it `server`: nothing when they agree, so the server decides.
pub fn chosen(on: bool, server: bool) -> Option<bool> {
    (on != server).then_some(on)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ai_switch_says_what_turning_it_off_stops() {
        let ai = |tagging, people, autotag| AiStatus {
            enabled: tagging || people,
            tagging,
            people,
            autotag,
            ..AiStatus::default()
        };
        assert_eq!(
            ai_switch(&ai(true, false, true)),
            Some((AiSwitch::Tags, true))
        );
        assert_eq!(
            ai_switch(&ai(true, false, false)),
            Some((AiSwitch::Tags, false))
        );
        // With the People tool, turning it off stops both, and it says so.
        assert_eq!(
            ai_switch(&ai(true, true, true)),
            Some((AiSwitch::TagsAndPeople, true))
        );
        // Off from the start, it only adds tags: the people are found
        // either way.
        assert_eq!(
            ai_switch(&ai(true, true, false)),
            Some((AiSwitch::Tags, false))
        );
        // Nothing to tag with: no switch, as before.
        assert_eq!(ai_switch(&ai(false, true, true)), None);
        assert_eq!(ai_switch(&ai(false, false, false)), None);
        assert_eq!(ai_switch(&AiStatus::default()), None);
        // The upload says only what differs from the server.
        assert_eq!(chosen(true, true), None);
        assert_eq!(chosen(false, true), Some(false));
        assert_eq!(chosen(true, false), Some(true));
        assert_eq!(chosen(false, false), None);
    }

    #[test]
    fn a_name_or_a_type_can_say_yes_and_the_rest_are_looked_into() {
        assert_eq!(verdict("a.JPG", "", 10), Verdict::Take);
        assert_eq!(verdict("clip.final.webm", "", 10), Verdict::Take);
        // The browser's type for a file with no extension.
        assert_eq!(verdict("anime", "image/webp", 10), Verdict::Take);
        assert_eq!(verdict("1000", "", 10), Verdict::Look);
        assert_eq!(verdict("notes.txt", "text/plain", 10), Verdict::Look);
        assert_eq!(verdict("saved.jpg_large", "", 10), Verdict::Look);
        // Hidden files and empty ones are never media.
        assert_eq!(verdict("._a.jpg", "image/jpeg", 10), Verdict::Skip);
        assert_eq!(verdict(".DS_Store", "", 10), Verdict::Skip);
        assert_eq!(verdict("a.png", "image/png", 0), Verdict::Skip);
        assert!(!is_media_name(".png"));
        assert!(!is_media_name("README"));
    }

    #[test]
    fn videos_are_told_by_type_or_name() {
        assert!(is_video("a.MOV", ""));
        assert!(is_video("clip", "video/webm"));
        assert!(!is_video("a.gif", "image/gif"));
    }

    #[test]
    fn a_notice_names_a_few_skipped_files_and_counts_the_rest() {
        let names: Vec<String> = ["a.txt", "b.pdf", "c.doc", "d.zip", "e"]
            .map(String::from)
            .into();
        assert_eq!(skipped_names(&names), ("a.txt, b.pdf, c.doc".into(), 2));
        assert_eq!(skipped_names(&names[..1]), ("a.txt".into(), 0));
        assert_eq!(skipped_names(&[]), (String::new(), 0));
    }

    #[test]
    fn tags_from_folder_names() {
        assert_eq!(
            folder_tag("Summer 2024/a.jpg").as_deref(),
            Some("summer_2024")
        );
        assert_eq!(folder_tag("/Pics/Cats/a.jpg").as_deref(), Some("cats"));
        assert_eq!(folder_tag("/Cats/a.jpg").as_deref(), Some("cats"));
        assert_eq!(folder_tag("a.jpg"), None);
        assert_eq!(folder_tag("/a.jpg"), None);
        assert_eq!(folder_tag(""), None);
        assert_eq!(folder_tag("-x/a.jpg"), None);
    }

    #[test]
    fn formats_sizes() {
        assert_eq!(human_size(0), "0 B");
        assert_eq!(human_size(1536), "1.5 KB");
        assert_eq!(human_size(5 * 1024 * 1024), "5.0 MB");
        assert!(human_size(u64::MAX).ends_with(" TB"));
    }

    #[test]
    fn says_how_far_and_how_fast() {
        let t = |done, total, speed| Transfer { done, total, speed };
        assert_eq!(transfer_label(&t(0, Some(1000), 0)), "0%");
        assert_eq!(
            transfer_label(&t(420, Some(1000), 1_288_490)),
            "42% · 1.2 MB/s"
        );
        assert_eq!(transfer_label(&t(1000, Some(1000), 512)), "100% · 512 B/s");
        assert_eq!(transfer_label(&t(3_565_158, None, 0)), "3.4 MB");
    }

    #[test]
    fn finds_urls_in_text() {
        assert_eq!(
            parse_urls(
                " https://a.example/1.png\nftp://x  HTTP://b.example/2 https://a.example/1.png word https://"
            ),
            ["https://a.example/1.png", "HTTP://b.example/2"]
        );
        assert!(parse_urls("").is_empty());
    }

    #[test]
    fn reads_what_a_browser_drag_carries() {
        let post = "https://gelbooru.com/index.php?page=post&s=view&id=15250239";
        let thumb = "https://img3.gelbooru.com/thumbnails/aa/bb/thumbnail_x.jpg";
        let wrapped =
            format!(r#"<meta charset="utf-8"><a href="{post}"><img alt="x" src="{thumb}"></a>"#);
        // A thumbnail in a list of posts: the post it links to.
        assert_eq!(drop_urls(post, &wrapped, post), [post]);
        // A picture inside any other link: the picture.
        let wrapped =
            format!("<a href='https://blog.example/p/1'><IMG SRC='{thumb}?a=1&amp;b=2' ></a>");
        assert_eq!(
            drop_urls("https://blog.example/p/1", &wrapped, ""),
            [format!("{thumb}?a=1&b=2")]
        );
        // A bare picture, as Firefox and Chromium list it.
        let alone = format!(r#"<img src="{thumb}" width="150">"#);
        assert_eq!(
            drop_urls(&format!("# a comment\r\n{thumb}\r\n"), &alone, thumb),
            [thumb]
        );
        assert_eq!(
            drop_urls("", "<img src=//cdn.example/a.png>", ""),
            ["https://cdn.example/a.png"]
        );
        assert_eq!(
            drop_urls("", "", " https://a.example/x.png "),
            ["https://a.example/x.png"]
        );
        // Nothing to import: files from a file manager, text, a picture inline.
        assert!(drop_urls("file:///home/me/a.png", "", "").is_empty());
        assert!(drop_urls("", "<p>hello</p>", "hello").is_empty());
        assert!(drop_urls("", r#"<img src="data:image/png;base64,AA">"#, "").is_empty());
        assert!(drop_urls("", "<img", "").is_empty());
    }

    #[test]
    fn an_address_beats_a_file_only_when_it_leads_to_the_post() {
        let post = ["https://danbooru.donmai.us/posts/1".to_owned()];
        let other = ["https://blog.example/a.png".to_owned()];
        assert!(drop_is_urls(0, &other));
        assert!(drop_is_urls(1, &post));
        assert!(!drop_is_urls(1, &other));
        assert!(!drop_is_urls(0, &[]));
        assert!(!drop_is_urls(2, &[]));
    }

    #[test]
    fn only_a_pictures_address_stands_as_its_own_preview() {
        assert!(is_picture_url("https://cdn.example/a/b.JPG?x=1"));
        assert!(is_picture_url("https://cdn.example/a.webp#top"));
        for other in [
            "https://gelbooru.com/index.php?page=post&s=view&id=7",
            "https://cdn.example/a.mp4",
            "https://cdn.example/folder.png/",
            "",
        ] {
            assert!(!is_picture_url(other), "{other}");
        }
        assert_eq!(
            host_of("https://www.gelbooru.com/index.php?page=post"),
            "gelbooru.com"
        );
        assert_eq!(host_of("http://127.0.0.1:8192/a.png"), "127.0.0.1:8192");
        assert_eq!(host_of("not an address"), "not an address");
    }

    #[test]
    fn a_format_is_named_by_its_content_or_its_name() {
        assert_eq!(format_of("image/png").as_deref(), Some("PNG"));
        assert_eq!(format_of("video/x-matroska").as_deref(), Some("MKV"));
        assert_eq!(format_of("application/pdf"), None);
        assert_eq!(format_by_name("Holiday.JPEG").as_deref(), Some("JPG"));
        assert_eq!(
            format_by_name("https://cdn.example/a/b.webp?x=1#y").as_deref(),
            Some("WEBP")
        );
        assert_eq!(format_by_name("scan.tif").as_deref(), Some("TIFF"));
        for none in [
            "anime",
            ".png",
            "notes.txt",
            "https://example.org/post/12",
            "https://example.org/v1.2/page",
        ] {
            assert_eq!(format_by_name(none), None, "{none}");
        }
    }

    #[test]
    fn a_batch_needs_good_tags_on_every_file() {
        assert!(tag_problem("sky blue", ["", "cloud"]).is_none());
        let long = "x".repeat(400);
        assert!(tag_problem(&long, ["cloud"]).is_some());
        assert!(tag_problem("sky", ["cloud", long.as_str()]).is_some());
        assert_eq!(
            tag_problem("-sky", [""]).map(|b| b.word).as_deref(),
            Some("-sky")
        );
        // A word of underscores is no tag, and is left out rather than refused.
        assert!(tag_problem("___", ["cloud"]).is_none());
        // Nothing to send, nothing wrong.
        assert!(tag_problem(&long, []).is_none());
    }

    #[test]
    fn a_refusal_is_told_by_its_status_and_code() {
        assert_eq!(refusal(Some(409), Some("duplicate")), Refusal::Duplicate);
        assert_eq!(refusal(Some(413), Some("too_large")), Refusal::TooBig);
        assert_eq!(refusal(Some(415), None), Refusal::Unsupported);
        assert_eq!(refusal(Some(409), Some("upload_gone")), Refusal::Other);
        assert_eq!(refusal(None, None), Refusal::Other);
    }

    #[test]
    fn a_duplicate_names_its_post() {
        assert_eq!(duplicate_post("this file is already post #12"), Some(12));
        assert_eq!(
            duplicate_post("this file is post #7, in the recycle bin: restore it from there"),
            Some(7)
        );
        assert_eq!(duplicate_post("this file was already uploaded"), None);
        assert_eq!(duplicate_post("post #0"), None);
        assert_eq!(duplicate_post("#"), None);
    }

    #[test]
    fn the_limit_is_the_servers_and_none_is_no_limit() {
        assert!(too_big(11, 10));
        assert!(!too_big(10, 10));
        assert!(!too_big(u64::MAX, 0));
    }
}
