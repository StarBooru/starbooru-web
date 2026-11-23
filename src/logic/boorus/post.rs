//! One post on another booru: what saves it here, whether its site is to be
//! asked about it alone, what its page shows on the stage, and its tags by
//! category.

use starbooru_core::tag::MAX_TAGS_PER_POST;
use starbooru_core::{BooruApi, ImportRequest, RemotePost, TagCategory, guarded_by};

use crate::logic::tags::{BadTag, parse_tags};

/// A post's own tags with the user's extra ones first, so they survive the
/// per-post limit.
pub fn import_tags(post: &[String], extra: &str) -> Result<Vec<String>, BadTag> {
    let mut tags = parse_tags(extra)?;
    for tag in post {
        if tags.len() >= MAX_TAGS_PER_POST {
            break;
        }
        if !tags.contains(tag) {
            tags.push(tag.clone());
        }
    }
    Ok(tags)
}

/// What saves `post` here: its file, with its page as the source, its tags
/// and its rating. A post whose site showed no file (Gelbooru without a key)
/// is imported by its page, and the server finds the file.
pub fn import_request(post: &RemotePost, extra: &str) -> Result<ImportRequest, BadTag> {
    let tags = import_tags(&post.import_tags(), extra)?;
    let by_page = post.file_url.trim().is_empty();
    Ok(ImportRequest {
        url: if by_page {
            post.page_url.clone()
        } else {
            post.file_url.clone()
        },
        tags,
        rating: post.rating,
        source: (!by_page).then(|| post.page_url.clone()),
        autotag: None,
    })
}

/// Whether the site is to be asked about the post by itself: nothing is known
/// of it, it came without its file, or it is on a Gelbooru-kind site and came
/// without its tags' categories.
pub fn wants_full(post: Option<&RemotePost>, api: Option<BooruApi>) -> bool {
    post.is_none_or(|post| {
        post.file_url.trim().is_empty()
            || (api == Some(BooruApi::Gelbooru) && post.groups.is_empty())
    })
}

/// The post to show: what its site said when asked for it alone, else what
/// the search brought. The search's address for its page stays, as that is
/// what its import is found by, and so does what the answer left out.
pub fn merged(kept: Option<&RemotePost>, looked: Option<&RemotePost>) -> Option<RemotePost> {
    let (Some(kept), Some(looked)) = (kept, looked) else {
        return looked.or(kept).cloned();
    };
    let mut post = looked.clone();
    post.page_url = kept.page_url.clone();
    post.post_id = post.post_id.or(kept.post_id);
    if post.preview_url.is_empty() {
        post.preview_url = kept.preview_url.clone();
    }
    if post.groups.is_empty() {
        post.groups = kept.groups.clone();
    }
    Some(post)
}

/// The address of what the stage shows. A picture shows its sample, a copy
/// sized for a screen. A video plays its file, as its sample is often a
/// still, unless the file's site only answers its own pages: then the copy
/// behind the server is all a page of ours can show.
pub fn stage_source(post: &RemotePost) -> Option<&str> {
    fn filled(url: &str) -> Option<&str> {
        Some(url).filter(|url| !url.trim().is_empty())
    }
    let sample = post.sample_url.as_deref().and_then(filled);
    let file = filled(&post.file_url).filter(|url| guarded_by(url).is_none());
    if post.is_video() {
        file.or(sample)
    } else {
        sample.or(file).or_else(|| filled(&post.preview_url))
    }
}

/// The post's tags under their categories, in the viewer's order: artists,
/// copyrights, characters, the rest, meta. Each group by name.
pub fn grouped(post: &RemotePost) -> Vec<(TagCategory, Vec<String>)> {
    const ORDER: [TagCategory; 5] = [
        TagCategory::Artist,
        TagCategory::Copyright,
        TagCategory::Character,
        TagCategory::General,
        TagCategory::Meta,
    ];
    ORDER
        .into_iter()
        .filter_map(|category| {
            let mut tags: Vec<String> = post
                .tags
                .iter()
                .filter(|tag| post.groups.category(tag) == category)
                .cloned()
                .collect();
            tags.sort();
            (!tags.is_empty()).then_some((category, tags))
        })
        .collect()
}

/// The picture's size for the stage, when the site gave one.
pub fn stage_size(post: &RemotePost) -> Option<(i32, i32)> {
    let side = |n: u32| i32::try_from(n).ok().filter(|n| *n > 0);
    side(post.width).zip(side(post.height))
}

#[cfg(test)]
mod tests {
    use starbooru_core::Rating;

    use super::*;
    use crate::logic::boorus::results::tests::{A, post as listed};

    fn post() -> RemotePost {
        RemotePost {
            id: 7,
            site: "https://d.org".into(),
            page_url: "https://d.org/posts/7".into(),
            preview_url: "https://cdn.d.org/360x360/a.jpg".into(),
            sample_url: Some("https://cdn.d.org/sample/a.jpg".into()),
            file_url: "https://cdn.d.org/original/a.png".into(),
            tags: vec![
                "sky".into(),
                "someone".into(),
                "miku".into(),
                "vocaloid".into(),
                "highres".into(),
                "cloud".into(),
            ],
            groups: Default::default(),
            rating: Rating::Safe,
            width: 1200,
            height: 800,
            md5: None,
            source: None,
            score: None,
            created_at: None,
            file_size: None,
            post_id: None,
        }
    }

    #[test]
    fn extra_tags_come_first_and_fit() {
        let post = vec!["sky".to_owned(), "sea".to_owned()];
        assert_eq!(
            import_tags(&post, "From_Danbooru sky").unwrap(),
            ["from_danbooru", "sky", "sea"]
        );
        let full: Vec<String> = (0..MAX_TAGS_PER_POST).map(|i| format!("t{i}")).collect();
        let tags = import_tags(&full, "mine").unwrap();
        assert_eq!(tags.len(), MAX_TAGS_PER_POST);
        assert_eq!(tags[0], "mine");
        let bad = import_tags(&post, "fine -dog").unwrap_err();
        assert_eq!(bad.word, "-dog");
    }

    #[test]
    fn a_post_is_imported_by_its_file_or_else_by_its_page() {
        let mut p = listed(A, 7, None);
        p.groups.put(TagCategory::Artist, "someone");
        let request = import_request(&p, "mine").unwrap();
        assert_eq!(request.url, "https://a.org/file/7.jpg");
        assert_eq!(request.source.as_deref(), Some("https://a.org/posts/7"));
        assert_eq!(request.rating, Rating::Sensitive);
        // The user's tags first, then the post's with their categories.
        assert_eq!(request.tags, ["mine", "artist:someone", "sky"]);
        assert_eq!(request.autotag, None);

        // Gelbooru without a key shows no file: the server reads the page.
        p.file_url = String::new();
        let request = import_request(&p, "").unwrap();
        assert_eq!(request.url, "https://a.org/posts/7");
        assert_eq!(request.source, None);
        assert_eq!(request.tags, ["artist:someone", "sky"]);

        assert!(import_request(&p, "-dog").is_err());
    }

    #[test]
    fn a_post_is_asked_for_alone_only_when_something_is_missing() {
        let mut p = post();
        p.groups.put(TagCategory::Artist, "someone");
        assert!(!wants_full(Some(&p), Some(BooruApi::Danbooru)));
        assert!(!wants_full(Some(&p), Some(BooruApi::Gelbooru)));
        assert!(!wants_full(Some(&p), None));
        // Opened from a link: nothing is known of it.
        assert!(wants_full(None, None));
        // Gelbooru's API names no categories; its page does.
        p.groups = Default::default();
        assert!(wants_full(Some(&p), Some(BooruApi::Gelbooru)));
        assert!(!wants_full(Some(&p), Some(BooruApi::Moebooru)));
        // And without a key it shows no file.
        p.file_url = " ".into();
        assert!(wants_full(Some(&p), Some(BooruApi::Danbooru)));
    }

    #[test]
    fn the_answer_fills_in_what_the_search_left_out() {
        let mut kept = post();
        kept.file_url = String::new();
        kept.post_id = Some(4);
        kept.page_url = "https://d.org/index.php?page=post&s=view&id=7".into();
        kept.groups.put(TagCategory::Artist, "someone");
        let mut looked = post();
        looked.preview_url = String::new();
        let both = merged(Some(&kept), Some(&looked)).unwrap();
        assert_eq!(both.file_url, "https://cdn.d.org/original/a.png");
        // Its import is found by the address the search gave.
        assert_eq!(both.page_url, kept.page_url);
        assert_eq!(both.post_id, Some(4));
        assert_eq!(both.preview_url, kept.preview_url);
        assert_eq!(both.groups.artist, ["someone"]);
        assert_eq!(merged(Some(&kept), None), Some(kept.clone()));
        assert_eq!(merged(None, Some(&looked)), Some(looked));
        assert_eq!(merged(None, None), None);
    }

    #[test]
    fn the_stage_shows_the_sample_or_plays_the_file() {
        let mut p = post();
        assert_eq!(stage_source(&p), Some("https://cdn.d.org/sample/a.jpg"));
        p.sample_url = None;
        assert_eq!(stage_source(&p), Some("https://cdn.d.org/original/a.png"));
        // No file yet: the thumbnail, until the site is asked.
        p.file_url = String::new();
        assert_eq!(stage_source(&p), Some("https://cdn.d.org/360x360/a.jpg"));
        p.preview_url = String::new();
        assert_eq!(stage_source(&p), None);

        // A video plays its file; its sample is often a still.
        let mut v = post();
        v.file_url = "https://cdn.d.org/original/a.webm".into();
        assert_eq!(stage_source(&v), Some("https://cdn.d.org/original/a.webm"));
        // Gelbooru's file servers refuse a page of ours: the copy behind the
        // server is the one to show.
        let relayed = "/api/v1/remote/media?u=x&exp=1&sig=ab";
        v.file_url = "https://video-cdn3.gelbooru.com/images/aa/bb/a.mp4".into();
        v.sample_url = Some(relayed.into());
        assert_eq!(stage_source(&v), Some(relayed));
        let mut g = post();
        g.file_url = "https://img4.gelbooru.com/images/aa/bb/a.png".into();
        g.sample_url = None;
        assert_eq!(stage_source(&g), Some("https://cdn.d.org/360x360/a.jpg"));
    }

    #[test]
    fn tags_go_under_their_categories() {
        let mut p = post();
        assert_eq!(grouped(&p).len(), 1, "no categories: one group");
        assert_eq!(grouped(&p)[0].0, TagCategory::General);
        p.groups.put(TagCategory::Artist, "someone");
        p.groups.put(TagCategory::Character, "miku");
        p.groups.put(TagCategory::Copyright, "vocaloid");
        p.groups.put(TagCategory::Meta, "highres");
        let groups = grouped(&p);
        let order: Vec<TagCategory> = groups.iter().map(|(c, _)| *c).collect();
        assert_eq!(
            order,
            [
                TagCategory::Artist,
                TagCategory::Copyright,
                TagCategory::Character,
                TagCategory::General,
                TagCategory::Meta
            ]
        );
        assert_eq!(groups[3].1, ["cloud", "sky"], "by name");
        p.tags.clear();
        assert!(grouped(&p).is_empty());
    }

    #[test]
    fn the_stage_is_sized_only_from_a_size_the_site_gave() {
        let mut p = post();
        assert_eq!(stage_size(&p), Some((1200, 800)));
        p.height = 0;
        assert_eq!(stage_size(&p), None);
        p.height = 800;
        p.width = u32::MAX;
        assert_eq!(stage_size(&p), None);
    }
}
