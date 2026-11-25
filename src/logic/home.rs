//! The home page without a browser: whether the app opens on it, how its
//! background lays the posts out in columns, and the administration's form
//! for it with the checks the server makes.

use starbooru_core::home::{
    HomeError, MAX_HOME_POSTS, MAX_HOME_QUERY_CHARS, MAX_HOME_SUBTITLE_CHARS, MAX_HOME_TITLE_CHARS,
    MIN_HOME_POSTS, clean_home,
};
use starbooru_core::{HomeSettings, names_order, parse_query};

use crate::pages::query_error_text;

// ---------- where the app opens ----------

/// Where the shell goes as it starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Start {
    /// Where it is.
    Stay,
    /// Whether the server has a home page isn't known yet: hold the page.
    Wait,
    /// The home page, in the address's place.
    Home,
    /// The gallery's last search, in the address's place.
    Search(String),
}

/// Where the app goes when the shell starts at `path`, with `?q=` or not.
/// `fresh` is the first page of a visit: the browser opened this address,
/// not a reload or Back, and nothing else has been shown since. `home` is
/// whether the server has a home page, `None` until it says. Only a bare
/// `/` moves: a link with a search, or any other page, opens as it is.
pub fn start(path: &str, has_q: bool, fresh: bool, home: Option<bool>, last: &str) -> Start {
    if !matches!(path, "" | "/") || has_q {
        return Start::Stay;
    }
    match (fresh, home) {
        (true, None) => Start::Wait,
        (true, Some(true)) => Start::Home,
        _ if last.trim().is_empty() => Start::Stay,
        _ => Start::Search(last.to_owned()),
    }
}

// ---------- the background ----------

/// The narrowest and widest a tile is drawn, as its height for a width of
/// 1: a panorama or a tall strip still makes a tile.
const RATIO_RANGE: (f64, f64) = (0.6, 1.6);
/// How long a column takes to move by its own width, in seconds.
const SECONDS_PER_WIDTH: f64 = 16.0;
/// The most times a column repeats its posts to fill the screen.
const MAX_REPEATS: usize = 6;

/// One tile of a column: the post's place in the list, and its height for
/// a width of 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tile {
    pub index: usize,
    pub ratio: f64,
}

/// A column of the background, moving up or down at the same pace as the
/// others, whatever its height.
#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    pub tiles: Vec<Tile>,
    /// For a width of 1, each tile's gap after it included.
    pub height: f64,
    pub up: bool,
    /// How long one loop takes.
    pub seconds: f64,
}

/// How many columns a window `width` pixels wide gets.
pub fn columns_for(width: f64) -> usize {
    let n = (width / 220.0).round();
    if n.is_finite() {
        (n as usize).clamp(3, 8)
    } else {
        3
    }
}

fn ratio(width: u32, height: u32) -> f64 {
    if width == 0 || height == 0 {
        return 1.0;
    }
    (f64::from(height) / f64::from(width)).clamp(RATIO_RANGE.0, RATIO_RANGE.1)
}

/// The posts of `sizes` (width, height) in `columns` columns, each into the
/// shortest so far, every column at least `least` tall for a width of 1
/// (repeating its posts, or borrowing others when there are fewer posts
/// than columns). `gap` is the space after each tile, for a width of 1.
/// Neighbouring columns move opposite ways. The page draws each column
/// twice over, so its loop has no seam.
pub fn background(sizes: &[(u32, u32)], columns: usize, least: f64, gap: f64) -> Vec<Column> {
    let columns = columns.max(1);
    let mut out: Vec<Column> = (0..columns)
        .map(|i| Column {
            tiles: Vec::new(),
            height: 0.0,
            up: i % 2 == 0,
            seconds: 0.0,
        })
        .collect();
    if sizes.is_empty() {
        return out;
    }
    let tile = |index: usize| {
        let (w, h) = sizes.get(index).copied().unwrap_or((1, 1));
        Tile {
            index,
            ratio: ratio(w, h),
        }
    };
    for index in 0..sizes.len() {
        let shortest = out.iter_mut().min_by(|a, b| a.height.total_cmp(&b.height));
        if let Some(column) = shortest {
            let t = tile(index);
            column.height += t.ratio + gap;
            column.tiles.push(t);
        }
    }
    for (i, column) in out.iter_mut().enumerate() {
        if column.tiles.is_empty() {
            let t = tile(i % sizes.len());
            column.height = t.ratio + gap;
            column.tiles.push(t);
        }
        let own = column.tiles.clone();
        let mut repeats = 1;
        while column.height < least && repeats < MAX_REPEATS {
            for t in &own {
                column.height += t.ratio + gap;
                column.tiles.push(*t);
            }
            repeats += 1;
        }
        column.seconds = column.height * SECONDS_PER_WIDTH;
    }
    out
}

// ---------- the administration's form ----------

/// A field of the home page's form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HomeField {
    Title,
    Subtitle,
    Query,
    Posts,
}

/// What the form says of each field, and the settings to send once every
/// one is right.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HomeCheck {
    pub title: Option<HomeError>,
    pub subtitle: Option<HomeError>,
    pub query: Option<HomeError>,
    pub posts: Option<HomeError>,
    pub request: Option<HomeSettings>,
}

/// Checks the form as the server will: the title one line of at most 64
/// characters (empty for the server's name), the line under it at most 140
/// (empty for none), the background's search at most 1000 characters that
/// read as a search of live posts (empty for the most popular rated safe),
/// and 12 to 60 pictures. Each field is checked on its own, so each says
/// what's wrong with it.
pub fn check_home(
    enabled: bool,
    title: &str,
    subtitle: &str,
    query: &str,
    posts: &str,
) -> HomeCheck {
    let count = posts.trim().parse::<u32>().unwrap_or(0);
    let settings = HomeSettings {
        enabled,
        title: Some(title.to_owned()),
        subtitle: Some(subtitle.to_owned()),
        query: query.to_owned(),
        posts: count,
    };
    let fine = HomeSettings::default();
    let alone = |only: HomeSettings| clean_home(&only).err();
    HomeCheck {
        title: alone(HomeSettings {
            title: settings.title.clone(),
            ..fine.clone()
        }),
        subtitle: alone(HomeSettings {
            subtitle: settings.subtitle.clone(),
            ..fine.clone()
        }),
        query: alone(HomeSettings {
            query: settings.query.clone(),
            ..fine.clone()
        }),
        posts: alone(HomeSettings {
            posts: count,
            ..fine
        }),
        request: clean_home(&settings).ok(),
    }
}

/// The field the server's refusal is about, from its words.
pub fn field_of(message: &str) -> Option<HomeField> {
    let message = message.trim();
    let is = |e: HomeError| message == e.to_string();
    if is(HomeError::TitleTooLong) || is(HomeError::ControlChar) {
        Some(HomeField::Title)
    } else if is(HomeError::SubtitleTooLong) {
        Some(HomeField::Subtitle)
    } else if is(HomeError::QueryTooLong)
        || is(HomeError::Deleted)
        || message.starts_with("the background's search")
    {
        Some(HomeField::Query)
    } else if is(HomeError::BadCount) {
        Some(HomeField::Posts)
    } else {
        None
    }
}

/// What's wrong with a field, in words.
pub fn home_error_text(e: &HomeError) -> String {
    match e {
        HomeError::TitleTooLong => t!("home-error-title", max = MAX_HOME_TITLE_CHARS),
        HomeError::SubtitleTooLong => t!("home-error-subtitle", max = MAX_HOME_SUBTITLE_CHARS),
        HomeError::ControlChar => t!("home-error-control"),
        HomeError::QueryTooLong => t!("home-error-query-long", max = MAX_HOME_QUERY_CHARS),
        HomeError::Deleted => t!("home-error-deleted"),
        HomeError::BadCount => t!(
            "home-error-count",
            min = MIN_HOME_POSTS,
            max = MAX_HOME_POSTS
        ),
        HomeError::Query(e) => query_error_text(e),
    }
}

/// The background's search as the gallery takes it, for the form's preview:
/// the admin's, with `rating:safe` unless it names a rating, and by score
/// unless it names an order, as the server makes it.
pub fn preview_query(query: &str) -> String {
    let query = query.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut terms = vec![query.clone()];
    let rated =
        parse_query(&query).is_ok_and(|q| !q.ratings.is_empty() || !q.exclude_ratings.is_empty());
    if !rated {
        terms.push("rating:safe".to_owned());
    }
    if !names_order(&query) {
        terms.push("order:score".to_owned());
    }
    terms
        .into_iter()
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use starbooru_core::Rating;
    use starbooru_core::home::background_query;

    use super::*;

    #[test]
    fn a_visit_that_starts_at_the_bare_address_opens_on_the_home_page() {
        assert_eq!(start("/", false, true, Some(true), "cat"), Start::Home);
        assert_eq!(start("", false, true, Some(true), ""), Start::Home);
        // Until the server says, the page holds.
        assert_eq!(start("/", false, true, None, "cat"), Start::Wait);
        // Without one, the gallery carries on from its last search.
        assert_eq!(
            start("/", false, true, Some(false), "cat"),
            Start::Search("cat".into())
        );
        assert_eq!(start("/", false, true, Some(false), " "), Start::Stay);
    }

    #[test]
    fn a_link_a_reload_or_any_later_page_keeps_its_place() {
        // A link to a search, or to another page.
        assert_eq!(start("/", true, true, Some(true), "cat"), Start::Stay);
        for path in ["/explore", "/post/3", "/home", "/settings"] {
            assert_eq!(start(path, false, true, Some(true), "cat"), Start::Stay);
        }
        // A reload of the gallery, Back into it, or the shell starting again
        // after signing in from a guest's page.
        assert_eq!(
            start("/", false, false, Some(true), "cat"),
            Start::Search("cat".into())
        );
        assert_eq!(start("/", false, false, Some(true), ""), Start::Stay);
        assert_eq!(start("/", false, false, None, ""), Start::Stay);
    }

    #[test]
    fn the_columns_follow_the_window() {
        assert_eq!(columns_for(390.0), 3);
        assert_eq!(columns_for(1280.0), 6);
        assert_eq!(columns_for(3840.0), 8);
        assert_eq!(columns_for(f64::NAN), 3);
        assert_eq!(columns_for(-5.0), 3);
    }

    #[test]
    fn each_post_goes_to_the_shortest_column() {
        // A tall one, then wide ones: the tall one's column waits.
        let sizes = [(100, 160), (200, 100), (200, 100), (200, 100)];
        let cols = background(&sizes, 2, 0.0, 0.0);
        let indexes: Vec<Vec<usize>> = cols
            .iter()
            .map(|c| c.tiles.iter().map(|t| t.index).collect())
            .collect();
        assert_eq!(indexes, [vec![0], vec![1, 2, 3]]);
        assert!((cols[0].height - 1.6).abs() < 1e-9);
        assert!((cols[1].height - 1.8).abs() < 1e-9);
        // Neighbours move opposite ways, each at the same pace.
        assert!(cols[0].up && !cols[1].up);
        assert!((cols[0].seconds / cols[0].height - SECONDS_PER_WIDTH).abs() < 1e-9);
    }

    #[test]
    fn odd_shapes_make_tiles_within_reason() {
        let cols = background(&[(4000, 100), (100, 4000), (0, 0)], 3, 0.0, 0.0);
        let ratios: Vec<f64> = cols.iter().map(|c| c.tiles[0].ratio).collect();
        assert_eq!(ratios, [0.6, 1.6, 1.0]);
    }

    #[test]
    fn short_columns_repeat_to_fill_the_screen() {
        let sizes = [(100, 100), (100, 100)];
        // Fewer posts than columns: the others are borrowed.
        let cols = background(&sizes, 4, 3.0, 0.1);
        assert_eq!(cols.len(), 4);
        for c in &cols {
            assert!(c.height >= 3.0, "{c:?}");
            assert!(c.tiles.len() <= MAX_REPEATS);
        }
        assert_eq!(cols[2].tiles[0].index, 0);
        assert_eq!(cols[3].tiles[0].index, 1);
        // No posts: empty columns, and nothing to repeat.
        assert!(
            background(&[], 3, 5.0, 0.1)
                .iter()
                .all(|c| c.tiles.is_empty())
        );
        // However short, the repeats stop.
        let cols = background(&[(100, 60)], 1, 1000.0, 0.0);
        assert_eq!(cols[0].tiles.len(), MAX_REPEATS);
    }

    #[test]
    fn the_form_checks_each_field_as_the_server_does() {
        let ok = check_home(true, "  My   booru ", "", " sky ", "24");
        assert_eq!(ok.title, None);
        assert_eq!(
            ok.request,
            Some(HomeSettings {
                enabled: true,
                title: Some("My booru".into()),
                subtitle: None,
                query: "sky".into(),
                posts: 24,
            })
        );
        // Empty fields are the defaults.
        let blank = check_home(false, "", " ", "", "36").request.unwrap();
        assert_eq!((blank.title, blank.subtitle), (None, None));
        let long = check_home(
            true,
            &"t".repeat(65),
            &"s".repeat(141),
            &"q".repeat(1001),
            "11",
        );
        assert_eq!(long.title, Some(HomeError::TitleTooLong));
        assert_eq!(long.subtitle, Some(HomeError::SubtitleTooLong));
        assert_eq!(long.query, Some(HomeError::QueryTooLong));
        assert_eq!(long.posts, Some(HomeError::BadCount));
        assert_eq!(long.request, None);
        assert!(
            check_home(true, &"é".repeat(64), "", "", "60")
                .request
                .is_some()
        );
        assert!(check_home(true, "", "", "", "61").posts.is_some());
        assert!(check_home(true, "", "", "", "twelve").posts.is_some());
        assert!(check_home(true, "", "", "", " 12 ").request.is_some());
        let bad = check_home(true, "a\u{202E}b", "", "rating:zzz", "12");
        assert_eq!(bad.title, Some(HomeError::ControlChar));
        assert!(matches!(bad.query, Some(HomeError::Query(_))));
        assert_eq!(
            check_home(true, "", "", "status:deleted", "12").query,
            Some(HomeError::Deleted)
        );
    }

    #[test]
    fn a_refusal_names_its_field() {
        for (e, field) in [
            (HomeError::TitleTooLong, HomeField::Title),
            (HomeError::ControlChar, HomeField::Title),
            (HomeError::SubtitleTooLong, HomeField::Subtitle),
            (HomeError::QueryTooLong, HomeField::Query),
            (HomeError::Deleted, HomeField::Query),
            (HomeError::BadCount, HomeField::Posts),
        ] {
            assert_eq!(field_of(&e.to_string()), Some(field), "{e}");
        }
        let query = check_home(true, "", "", "rating:zzz", "12").query.unwrap();
        assert_eq!(field_of(&query.to_string()), Some(HomeField::Query));
        assert_eq!(field_of("the name is too long"), None);
        // Every error has words of its own.
        for e in [
            HomeError::TitleTooLong,
            HomeError::SubtitleTooLong,
            HomeError::ControlChar,
            HomeError::QueryTooLong,
            HomeError::Deleted,
            HomeError::BadCount,
            query,
        ] {
            let text = home_error_text(&e);
            assert!(
                !text.starts_with("home-error-") && text != e.to_string(),
                "{e}"
            );
        }
    }

    #[test]
    fn the_preview_searches_what_the_server_would() {
        for typed in [
            "",
            "  sky ",
            "sky rating:q",
            "-rating:e cat",
            "order:random",
            "cat order:new",
        ] {
            let shown = parse_query(&preview_query(typed)).unwrap();
            let server = background_query(typed).unwrap();
            assert_eq!(shown.include, server.include, "{typed}");
            assert_eq!(shown.ratings, server.ratings, "{typed}");
            assert_eq!(shown.exclude_ratings, server.exclude_ratings, "{typed}");
            assert_eq!(shown.order, server.order, "{typed}");
        }
        assert_eq!(preview_query(""), "rating:safe order:score");
        assert_eq!(
            preview_query(" sky  cat "),
            "sky cat rating:safe order:score"
        );
        assert_eq!(preview_query("rating:e"), "rating:e order:score");
        assert_eq!(
            parse_query(&preview_query("")).unwrap().ratings,
            [Rating::Safe]
        );
    }
}
