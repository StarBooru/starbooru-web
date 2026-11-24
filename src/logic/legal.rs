//! The privacy policy's and the terms' way back (`pages/legal.rs`).

use super::settings::Group;
use crate::route;

/// Where Back leads: the group of Settings with the links to these pages, or
/// the sign-in screen, which the shell shows at any address while no one is
/// signed in.
pub fn back_href(signed_in: bool) -> &'static str {
    if signed_in { Group::About.href() } else { "/" }
}

/// Whether Back can go back through history, so the page it leads to comes
/// back as it was: only when this tab opened the legal page from there.
/// `opened_from` is the path of the page whose link was followed, and
/// `history_len` the tab's `history.length`.
pub fn back_through_history(opened_from: Option<&str>, signed_in: bool, history_len: u32) -> bool {
    let from_back = opened_from.is_some_and(|path| !signed_in || route::is_settings(path));
    from_back && history_len > 1
}

/// A click the app may take over: the main button, with no key held that
/// asks for a new tab or window.
pub fn plain_click(button: i16, modifier: bool) -> bool {
    button == 0 && !modifier
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn back_leads_to_settings_or_to_the_sign_in_screen() {
        assert_eq!(back_href(true), "/settings/about");
        assert_eq!(back_href(false), "/");
    }

    #[test]
    fn back_goes_through_history_only_to_the_page_it_names() {
        // From Settings, signed in.
        assert!(back_through_history(Some("/settings"), true, 3));
        assert!(back_through_history(Some("/settings/about"), true, 3));
        // From the sign-in card, wherever it showed.
        assert!(back_through_history(Some("/"), false, 2));
        assert!(back_through_history(Some("/explore"), false, 2));
        // Opened by its address, or as the tab's first page.
        assert!(!back_through_history(None, true, 5));
        assert!(!back_through_history(None, false, 5));
        assert!(!back_through_history(Some("/settings"), true, 1));
        // Signed in, from anywhere but Settings: the link goes there instead.
        assert!(!back_through_history(Some("/"), true, 4));
    }

    #[test]
    fn only_a_plain_click_is_taken_over() {
        assert!(plain_click(0, false));
        assert!(!plain_click(0, true));
        assert!(!plain_click(1, false));
    }
}
