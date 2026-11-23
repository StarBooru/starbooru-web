//! The pages this tab went through inside the app, for the Back button of a
//! page under a tab (`components/back.rs`). The browser won't say where its
//! history leads, so the app follows the router and tells a step back from a
//! step forward the way the router does: back is to the page before.

/// About as many as the browser keeps.
const MAX: usize = 50;

/// A page in the trail: its path and query, and how far down it was when a
/// link on it was followed.
#[derive(Clone, Debug, PartialEq)]
pub struct Spot {
    pub at: String,
    pub y: f64,
}

/// Notes that the tab is now at `at` (a path with its query).
pub fn step(trail: &mut Vec<Spot>, at: &str) {
    if trail.last().is_some_and(|last| last.at == at) {
        return;
    }
    let before = trail.len().checked_sub(2).and_then(|i| trail.get(i));
    if before.is_some_and(|before| before.at == at) {
        trail.pop();
        return;
    }
    trail.push(Spot {
        at: at.to_owned(),
        y: 0.0,
    });
    if trail.len() > MAX {
        trail.remove(0);
    }
}

/// Notes how far down the page the tab is on has been scrolled.
pub fn scrolled(trail: &mut [Spot], y: f64) {
    if let Some(last) = trail.last_mut() {
        last.y = y;
    }
}

/// Where a step back through history leads, while that stays in the app. A
/// page opened from a link elsewhere, or as an installed app's first page,
/// has nothing before it.
pub fn before(trail: &[Spot]) -> Option<&Spot> {
    trail.len().checked_sub(2).and_then(|i| trail.get(i))
}

/// The path alone, which is what the browser's location compares.
pub fn path_of(at: &str) -> &str {
    at.split(['?', '#']).next().unwrap_or(at)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walk(paths: &[&str]) -> Vec<Spot> {
        let mut trail = Vec::new();
        for path in paths {
            step(&mut trail, path);
        }
        trail
    }

    fn ats(trail: &[Spot]) -> Vec<&str> {
        trail.iter().map(|s| s.at.as_str()).collect()
    }

    #[test]
    fn the_first_page_has_nothing_to_go_back_to() {
        assert_eq!(before(&walk(&["/trash"])), None);
        assert_eq!(before(&walk(&[])), None);
    }

    #[test]
    fn a_page_opened_from_another_goes_back_to_it() {
        let trail = walk(&["/settings", "/trash"]);
        assert_eq!(before(&trail).map(|s| s.at.as_str()), Some("/settings"));
        // The query counts: one tag's page leads to another's.
        let trail = walk(&["/tag?name=a", "/tag?name=b"]);
        assert_eq!(before(&trail).map(|s| s.at.as_str()), Some("/tag?name=a"));
    }

    #[test]
    fn going_back_forgets_the_page_left() {
        // Settings, Trash, a post, then the browser's Back to Trash.
        let trail = walk(&["/settings", "/trash", "/post/5", "/trash"]);
        assert_eq!(ats(&trail), ["/settings", "/trash"]);
        // Back again, to the first page.
        let trail = walk(&["/trash", "/settings", "/trash"]);
        assert_eq!(ats(&trail), ["/trash"]);
        assert_eq!(before(&trail), None);
    }

    #[test]
    fn the_same_address_again_is_no_step() {
        assert_eq!(ats(&walk(&["/help", "/help"])), ["/help"]);
    }

    #[test]
    fn a_page_keeps_how_far_down_it_was_left() {
        let mut trail = walk(&["/settings"]);
        scrolled(&mut trail, 1200.0);
        step(&mut trail, "/trash");
        scrolled(&mut trail, 40.0);
        assert_eq!(before(&trail).map(|s| s.y), Some(1200.0));
        // Back, and on again: the page is new, from the top.
        step(&mut trail, "/settings");
        step(&mut trail, "/help");
        assert_eq!(trail.last().map(|s| s.y), Some(0.0));
        assert_eq!(before(&trail).map(|s| s.y), Some(1200.0));
    }

    #[test]
    fn it_keeps_the_latest_pages() {
        let paths: Vec<String> = (0..60).map(|i| format!("/post/{i}")).collect();
        let refs: Vec<&str> = paths.iter().map(String::as_str).collect();
        let trail = walk(&refs);
        assert_eq!(trail.len(), MAX);
        assert_eq!(trail.last().map(|s| s.at.as_str()), Some("/post/59"));
    }

    #[test]
    fn the_path_leaves_the_query_out() {
        assert_eq!(path_of("/tag?name=a"), "/tag");
        assert_eq!(path_of("/privacy#cookies"), "/privacy");
        assert_eq!(path_of("/settings"), "/settings");
    }
}
