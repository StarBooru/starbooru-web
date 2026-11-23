//! The booru browser's rules that need no browser: the sites a search goes
//! to and how an address names them, the text an address carries, the sites
//! the user added and the accounts an older app kept on the device. What a
//! search found is in `results`, one post's page in `post`.

pub mod post;
pub mod results;

use starbooru_core::remote::{RemoteError, clean_base};
use starbooru_core::saved::MAX_SEARCH_NAME_CHARS;
use starbooru_core::{BooruApi, BooruSite, MAX_SEARCH_SITES, RemoteSite, SITES, SaveBooruAccount};

use crate::store::SavedBooru;

/// Sites added by their address that are remembered.
pub const MAX_SAVED: usize = 20;

/// The kind a stored name stands for. `None` for one this version doesn't
/// know, which a newer one may have written.
pub fn api_of(name: &str) -> Option<BooruApi> {
    BooruApi::ALL.into_iter().find(|api| api.as_str() == name)
}

/// An address as the server keeps it.
pub fn tidy(base: &str) -> String {
    base.trim().trim_end_matches('/').to_owned()
}

// ---------- the sites the user added ----------

/// The accounts an older app kept on this device, to move to the server.
pub fn kept_on_device(saved: &[SavedBooru]) -> Vec<SaveBooruAccount> {
    saved
        .iter()
        .filter(|s| !s.login.trim().is_empty() && !s.api_key.trim().is_empty())
        .filter_map(|s| {
            Some(SaveBooruAccount {
                api: api_of(&s.api)?,
                base: s.base.clone(),
                login: s.login.clone(),
                api_key: s.api_key.clone(),
            })
        })
        .collect()
}

/// Whether a key kept on this device comes off it after it was offered to
/// the server. `refused` is `None` when the server took it, else the
/// refusal's status, `None` when there was none (the network, or being told
/// to wait). Trouble that may pass keeps the key for the next try: no
/// status, a 5xx, 401 and 429. Any other refusal would come again, so the
/// key goes.
pub fn drop_moved_key(refused: Option<Option<u16>>) -> bool {
    match refused {
        None => true,
        Some(None | Some(401 | 429)) => false,
        Some(Some(status)) => (400..500).contains(&status),
    }
}

/// Takes the account for `base` off this device.
pub fn forget_on_device(saved: &mut [SavedBooru], base: &str) {
    for site in saved.iter_mut().filter(|s| tidy(&s.base) == tidy(base)) {
        site.login.clear();
        site.api_key.clear();
    }
}

/// Adds `site` after the others, so its chip comes last. One already there
/// by that address stays where it is and only takes the new kind. The oldest
/// make room.
pub fn remember(saved: &mut Vec<SavedBooru>, site: SavedBooru) {
    match saved.iter_mut().find(|s| tidy(&s.base) == tidy(&site.base)) {
        Some(known) => known.api = site.api,
        None => saved.push(site),
    }
    let extra = saved.len().saturating_sub(MAX_SAVED);
    saved.drain(..extra);
}

// ---------- addresses ----------

/// Text as it goes into one of the browser's addresses. The reference's
/// router read `&`, `%` and `+` in a value back as something else, and a
/// Gelbooru post's page has `&` in it, as some tags do; a comma parts the
/// sites. Those go in as `~` and their code in hex, so links made by the
/// reference still open here.
pub fn to_address(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '~' | '%' | '&' | '#' | '+' | ',' => out.push_str(&format!("~{:02X}", u32::from(c))),
            _ => out.push(c),
        }
    }
    out
}

/// The text an address carries (`to_address`). A `~` that isn't followed by
/// a code stays as it is.
pub fn from_address(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('~') {
        let (before, after) = rest.split_at(at);
        out.push_str(before);
        let code = after
            .get(1..3)
            .filter(|hex| hex.bytes().all(|b| b.is_ascii_hexdigit()));
        match code
            .and_then(|hex| u8::from_str_radix(hex, 16).ok())
            .filter(u8::is_ascii)
        {
            Some(byte) => {
                out.push(char::from(byte));
                rest = after.get(3..).unwrap_or_default();
            }
            None => {
                out.push('~');
                rest = after.get(1..).unwrap_or_default();
            }
        }
    }
    out.push_str(rest);
    out
}

/// What is wrong with a site or account typed in, in words.
pub fn remote_error_text(error: &RemoteError) -> String {
    match error {
        RemoteError::BadBase => t!("boorus-bad-address"),
        RemoteError::NoAccount => t!("boorus-need-account"),
        RemoteError::TooLong => t!("boorus-too-long"),
        other => other.to_string(),
    }
}

/// The name shown for a site's login: Gelbooru signs in by user ID.
pub fn login_label(api: BooruApi) -> String {
    if api == BooruApi::Gelbooru {
        t!("boorus-user-id")
    } else {
        t!("boorus-login")
    }
}

/// The name shown for a site's key: Moebooru takes a password hash.
pub fn key_label(api: BooruApi) -> String {
    if api == BooruApi::Moebooru {
        t!("boorus-password-hash")
    } else {
        t!("boorus-key")
    }
}

/// A search as the sites get it: single spaces, none at the ends.
pub fn tidy_query(q: &str) -> String {
    q.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ---------- sites ----------

/// A site the browser can search.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Site {
    pub name: String,
    pub site: RemoteSite,
    /// Added by its address, so it can be taken away again.
    pub custom: bool,
}

/// A preset as a site to search.
fn site_of_preset(preset: &BooruSite) -> RemoteSite {
    RemoteSite {
        api: preset.api,
        base: preset.base.to_owned(),
    }
}

/// What stands for a preset in an address: its name in small letters.
fn preset_key(site: &BooruSite) -> String {
    site.name.to_lowercase()
}

/// The preset a site is, if it is one: the same kind at the same address.
pub fn preset_of(site: &RemoteSite) -> Option<BooruSite> {
    let base = tidy(&site.base);
    SITES
        .into_iter()
        .find(|s| s.api == site.api && s.base == base)
}

/// What stands for a site in an address: a preset's key, or `api|base`.
pub fn site_key(site: &RemoteSite) -> String {
    match preset_of(site) {
        Some(preset) => preset_key(&preset),
        // A comma parts one site from the next: `to_address` takes it out.
        None => format!("{}|{}", site.api.as_str(), to_address(&tidy(&site.base))),
    }
}

/// The site a key stands for. `None` for one this version can't read.
fn site_of(key: &str) -> Option<RemoteSite> {
    let key = key.trim();
    let name = key.to_lowercase();
    if let Some(preset) = SITES.into_iter().find(|s| preset_key(s) == name) {
        return Some(site_of_preset(&preset));
    }
    let (api, base) = key.split_once('|')?;
    Some(RemoteSite {
        api: api_of(api)?,
        base: clean_base(&from_address(base)).ok()?,
    })
}

/// The sites an address names (`s`), each once, as many as a search goes to.
pub fn parse_sites(s: &str) -> Vec<RemoteSite> {
    let mut sites: Vec<RemoteSite> = Vec::new();
    for site in s.split(',').filter_map(site_of) {
        if sites.len() < MAX_SEARCH_SITES && !sites.contains(&site) {
            sites.push(site);
        }
    }
    sites
}

/// `sites` as an address names them.
pub fn sites_param(sites: &[RemoteSite]) -> String {
    sites.iter().map(site_key).collect::<Vec<_>>().join(",")
}

/// The sites a search goes to: the address's, else the ones chosen last
/// time (`last`, as keys), else the first preset.
pub fn chosen_sites(s: &str, last: &[String]) -> Vec<RemoteSite> {
    let some = |sites: Vec<RemoteSite>| (!sites.is_empty()).then_some(sites);
    some(parse_sites(s))
        .or_else(|| some(parse_sites(&last.join(","))))
        .unwrap_or_else(|| SITES.first().map(site_of_preset).into_iter().collect())
}

/// The site whose tags complete what is typed in the search box: the first
/// of the chosen ones.
pub fn completing_site(s: &str, last: &[String]) -> Option<RemoteSite> {
    chosen_sites(s, last).into_iter().next()
}

/// An address without its scheme, as a site's name when it has no other.
pub fn host_of(base: &str) -> String {
    let base = tidy(base);
    base.strip_prefix("https://")
        .or_else(|| base.strip_prefix("http://"))
        .unwrap_or(&base)
        .to_owned()
}

/// Every site there is a chip for: the presets, the ones the user added, and
/// any other the address names (a saved search's, say).
pub fn all_sites(added: &[SavedBooru], chosen: &[RemoteSite]) -> Vec<Site> {
    let mut sites: Vec<Site> = SITES
        .into_iter()
        .map(|preset| Site {
            name: preset.name.to_owned(),
            site: site_of_preset(&preset),
            custom: false,
        })
        .collect();
    let added = added.iter().filter_map(|saved| {
        Some(RemoteSite {
            api: api_of(&saved.api)?,
            base: clean_base(&saved.base).ok()?,
        })
    });
    let others = added
        .map(|site| (site, true))
        .chain(chosen.iter().cloned().map(|site| (site, false)));
    for (site, custom) in others {
        if !sites.iter().any(|known| known.site == site) {
            sites.push(Site {
                name: host_of(&site.base),
                site,
                custom,
            });
        }
    }
    sites
}

/// The chosen sites after a click on `site`'s chip, in the order of the
/// chips. One always stays on, and no more go on than a search takes.
pub fn toggled(chosen: &[RemoteSite], site: &RemoteSite, order: &[Site]) -> Vec<RemoteSite> {
    let mut next = chosen.to_vec();
    match next.iter().position(|s| s == site) {
        Some(at) if next.len() > 1 => {
            next.remove(at);
        }
        Some(_) => {}
        None if next.len() < MAX_SEARCH_SITES => next.push(site.clone()),
        None => {}
    }
    next.sort_by_key(|s| {
        order
            .iter()
            .position(|known| known.site == *s)
            .unwrap_or(usize::MAX)
    });
    next
}

/// The name of the site at `base`: a preset's, else its address.
pub fn site_name(base: &str) -> String {
    let base = tidy(base);
    SITES
        .into_iter()
        .find(|s| s.base == base)
        .map_or_else(|| host_of(&base), |s| s.name.to_owned())
}

/// Whether two searches go to the same sites.
pub fn same_sites(a: &[RemoteSite], b: &[RemoteSite]) -> bool {
    let keys = |sites: &[RemoteSite]| {
        let mut keys: Vec<String> = sites.iter().map(site_key).collect();
        keys.sort();
        keys
    };
    keys(a) == keys(b)
}

/// The name a search is saved under: the search itself, or its sites when it
/// is the newest posts.
pub fn search_name(q: &str, sites: &[RemoteSite]) -> String {
    let name = if q.is_empty() {
        sites
            .iter()
            .map(|s| site_name(&s.base))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        q.to_owned()
    };
    name.chars().take(MAX_SEARCH_NAME_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved(api: &str, base: &str, login: &str) -> SavedBooru {
        SavedBooru {
            api: api.into(),
            base: base.into(),
            login: login.into(),
            api_key: if login.is_empty() {
                String::new()
            } else {
                format!("{login}-key")
            },
        }
    }

    pub(super) fn site(api: BooruApi, base: &str) -> RemoteSite {
        RemoteSite {
            api,
            base: base.into(),
        }
    }

    pub(super) fn preset(name: &str) -> RemoteSite {
        let found = SITES.into_iter().find(|s| s.name == name).unwrap();
        site(found.api, found.base)
    }

    #[test]
    fn remembers_added_sites_by_address() {
        let mut list = vec![saved("danbooru", "https://a.org", "a")];
        remember(&mut list, saved("gelbooru", "https://b.org", "b"));
        // Added again as another kind: the same place, and what an older
        // app kept with it stays until it has moved to the server.
        remember(&mut list, saved("blombooru", "https://a.org/", ""));
        assert_eq!(list.len(), 2);
        assert_eq!(
            (
                list[0].api.as_str(),
                list[0].base.as_str(),
                list[0].login.as_str()
            ),
            ("blombooru", "https://a.org", "a")
        );
        assert_eq!(list[1].base, "https://b.org", "the latest comes last");
        for i in 0..MAX_SAVED + 5 {
            remember(
                &mut list,
                saved("danbooru", &format!("https://{i}.org"), ""),
            );
        }
        assert_eq!(list.len(), MAX_SAVED);
        assert_eq!(
            list[MAX_SAVED - 1].base,
            format!("https://{}.org", MAX_SAVED + 4)
        );
        assert!(list.iter().all(|s| s.base != "https://a.org"));
    }

    #[test]
    fn accounts_kept_on_the_device_move_to_the_server() {
        let mut list = vec![
            saved("danbooru", "https://a.org", "a"),
            saved("gelbooru", "https://b.org", ""),
            // A kind this version doesn't know can't be moved.
            saved("szurubooru", "https://c.org", "c"),
        ];
        let moving = kept_on_device(&list);
        assert_eq!(moving.len(), 1);
        assert_eq!(
            (moving[0].base.as_str(), moving[0].api_key.as_str()),
            ("https://a.org", "a-key")
        );
        forget_on_device(&mut list, "https://a.org/");
        assert_eq!((list[0].login.as_str(), list[0].api_key.as_str()), ("", ""));
        assert!(kept_on_device(&list).is_empty());
        // The site itself is still remembered.
        assert_eq!(list[0].base, "https://a.org");
        assert_eq!(tidy(" https://a.org// "), "https://a.org");
        assert_eq!(api_of("e621"), Some(BooruApi::E621));
        assert_eq!(api_of("szurubooru"), None);
    }

    #[test]
    fn a_moved_key_stays_on_the_device_only_for_trouble_that_passes() {
        // Taken by the server.
        assert!(drop_moved_key(None));
        // No answer, a server in trouble, a session that ended, a wait.
        for kept in [None, Some(500), Some(502), Some(503), Some(401), Some(429)] {
            assert!(!drop_moved_key(Some(kept)), "{kept:?}");
        }
        // Refusals that would only come again: a bad key, too many accounts,
        // URL import turned off.
        for gone in [400, 403, 404, 409, 413, 422] {
            assert!(drop_moved_key(Some(Some(gone))), "{gone}");
        }
    }

    #[test]
    fn what_an_address_carries_comes_back_as_it_went_in() {
        for text in [
            "",
            "blue_sky rating:safe",
            "https://gelbooru.com/index.php?page=post&s=view&id=7#comments",
            "panty_&_stocking_with_garterbelt 100% c++ ~either a,b",
            "già_visto ★ 日本語",
            "~7E ~zz ~",
        ] {
            let safe = to_address(text);
            assert!(!safe.contains(['&', '%', '#', '+', ',']), "{safe}");
            assert_eq!(from_address(&safe), text);
        }
        assert_eq!(to_address("a&b"), "a~26b");
        // Text nobody encoded reads as it is.
        assert_eq!(from_address("blue_sky ~a"), "blue_sky ~a");
        assert_eq!(from_address("~"), "~");
        // A code past ASCII isn't one of ours.
        assert_eq!(from_address("~E2"), "~E2");
        assert_eq!(tidy_query("  a   b "), "a b");
    }

    #[test]
    fn sites_go_into_an_address_and_come_back() {
        for preset in SITES {
            let site = site(preset.api, preset.base);
            let key = site_key(&site);
            assert_eq!(key, preset.name.to_lowercase());
            assert!(!key.contains([',', '|', ' ']), "{key}");
            assert_eq!(site_of(&key), Some(site));
        }
        assert_eq!(site_key(&preset("yande.re")), "yande.re");
        // A trailing slash is the same site, and so is another case.
        assert_eq!(
            site_key(&site(BooruApi::Danbooru, "https://danbooru.donmai.us/")),
            "danbooru"
        );
        assert_eq!(site_of(" Danbooru "), Some(preset("Danbooru")));

        // One added by its address, and one whose address has a comma.
        let nas = site(BooruApi::Blombooru, "http://nas:8000/booru");
        assert_eq!(site_key(&nas), "blombooru|http://nas:8000/booru");
        let odd = site(BooruApi::Danbooru, "https://x.org/a,b&c");
        assert_eq!(site_key(&odd), "danbooru|https://x.org/a~2Cb~26c");
        let all = vec![preset("Danbooru"), nas.clone(), odd.clone(), preset("e926")];
        let s = sites_param(&all);
        assert_eq!(
            s,
            "danbooru,blombooru|http://nas:8000/booru,danbooru|https://x.org/a~2Cb~26c,e926"
        );
        assert_eq!(parse_sites(&s), all);
        // A preset's address under another kind is a site of its own.
        let other = site(BooruApi::Danbooru, "https://safebooru.org");
        assert_eq!(site_key(&other), "danbooru|https://safebooru.org");
        assert_eq!(site_of(&site_key(&other)), Some(other));
    }

    #[test]
    fn an_address_names_each_site_once_and_only_ones_it_can_read() {
        assert!(parse_sites("").is_empty());
        assert_eq!(
            parse_sites(
                "danbooru,,nonsense,szurubooru|https://s.org,danbooru|not-an-address,e926,danbooru"
            ),
            [preset("Danbooru"), preset("e926")]
        );
        let many: Vec<String> = (0..MAX_SEARCH_SITES + 3)
            .map(|i| format!("danbooru|https://{i}.org"))
            .collect();
        assert_eq!(parse_sites(&many.join(",")).len(), MAX_SEARCH_SITES);
    }

    #[test]
    fn a_search_goes_to_the_sites_chosen_last_unless_the_address_says() {
        let first = SITES.first().unwrap();
        assert_eq!(chosen_sites("", &[]), [site(first.api, first.base)]);
        let last = vec!["e926".to_owned(), "gelbooru".to_owned()];
        assert_eq!(
            chosen_sites("", &last),
            [preset("e926"), preset("Gelbooru")]
        );
        assert_eq!(chosen_sites("konachan", &last), [preset("Konachan")]);
        // Nothing readable in the address, or in what was kept.
        assert_eq!(
            chosen_sites("??", &last),
            [preset("e926"), preset("Gelbooru")]
        );
        assert_eq!(
            chosen_sites("", &["gone".to_owned()]),
            [site(first.api, first.base)]
        );
    }

    #[test]
    fn the_first_chosen_site_completes_tags() {
        assert_eq!(
            completing_site("gelbooru,danbooru", &[]),
            Some(preset("Gelbooru"))
        );
        assert_eq!(
            completing_site("", &["yande.re".to_owned(), "danbooru".to_owned()]),
            Some(preset("yande.re"))
        );
        assert_eq!(completing_site("", &[]), Some(preset("Danbooru")));
    }

    #[test]
    fn chips_are_the_presets_then_the_added_sites() {
        let added = vec![
            saved("danbooru", "http://nas:8000/", ""),
            // The same as a preset, an unknown kind, and a bad address.
            saved("gelbooru", "https://gelbooru.com", ""),
            saved("szurubooru", "https://s.org", ""),
            saved("danbooru", "nas", ""),
        ];
        let linked = site(BooruApi::E621, "https://e621.net");
        let chips = all_sites(&added, &[preset("Danbooru"), linked.clone()]);
        assert_eq!(chips.len(), SITES.len() + 2);
        assert!(chips.iter().take(SITES.len()).all(|c| !c.custom));
        let nas = &chips[SITES.len()];
        assert_eq!((nas.name.as_str(), nas.custom), ("nas:8000", true));
        assert_eq!(nas.site, site(BooruApi::Danbooru, "http://nas:8000"));
        // A site only the address names has a chip, with nothing to remove.
        let other = &chips[SITES.len() + 1];
        assert_eq!((other.name.as_str(), other.custom), ("e621.net", false));
        assert_eq!(other.site, linked);
        assert_eq!(site_name("https://yande.re/"), "yande.re");
        assert_eq!(site_name("http://127.0.0.1:9001"), "127.0.0.1:9001");
        assert_eq!(preset_of(&preset("e926")).map(|p| p.name), Some("e926"));
        assert_eq!(
            preset_of(&site(BooruApi::Danbooru, "https://e926.net")),
            None
        );
    }

    #[test]
    fn a_chip_turns_its_site_on_or_off_and_one_stays_on() {
        let chips = all_sites(&[], &[]);
        let (dan, gel, e9) = (preset("Danbooru"), preset("Gelbooru"), preset("e926"));
        let one = vec![dan.clone()];
        assert_eq!(toggled(&one, &dan, &chips), one, "the last one stays");
        // In the order of the chips, whatever the order of the clicks.
        let two = toggled(&one, &e9, &chips);
        assert_eq!(two, [dan.clone(), e9.clone()]);
        let three = toggled(&two, &gel, &chips);
        assert_eq!(three, [dan.clone(), gel.clone(), e9.clone()]);
        assert_eq!(toggled(&three, &dan, &chips), [gel, e9]);
        // No more than a search goes to.
        let full: Vec<RemoteSite> = (0..MAX_SEARCH_SITES)
            .map(|i| site(BooruApi::Danbooru, &format!("https://{i}.org")))
            .collect();
        assert_eq!(toggled(&full, &dan, &chips).len(), MAX_SEARCH_SITES);
    }

    #[test]
    fn a_site_or_account_typed_in_says_what_is_wrong() {
        assert_eq!(
            remote_error_text(&RemoteError::BadBase),
            "Enter the site’s address, like https://booru.example."
        );
        assert_eq!(
            remote_error_text(&RemoteError::NoAccount),
            "Enter both the username (or user ID) and the key."
        );
        assert_eq!(
            remote_error_text(&RemoteError::TooLong),
            "That’s longer than a site takes."
        );
        assert_eq!(login_label(BooruApi::Gelbooru), "User ID");
        assert_eq!(login_label(BooruApi::Danbooru), "Username");
        assert_eq!(key_label(BooruApi::Moebooru), "Password hash");
        assert_eq!(key_label(BooruApi::E621), "API key");
    }

    #[test]
    fn a_saved_search_is_named_and_matched_by_its_sites() {
        let (dan, gel) = (preset("Danbooru"), preset("Gelbooru"));
        assert_eq!(
            search_name("blue_sky rating:safe", std::slice::from_ref(&dan)),
            "blue_sky rating:safe"
        );
        assert_eq!(
            search_name("", &[dan.clone(), gel.clone()]),
            "Danbooru, Gelbooru"
        );
        let long = "x".repeat(MAX_SEARCH_NAME_CHARS + 10);
        assert_eq!(
            search_name(&long, &[]).chars().count(),
            MAX_SEARCH_NAME_CHARS
        );
        assert!(same_sites(
            &[dan.clone(), gel.clone()],
            &[gel.clone(), dan.clone()]
        ));
        assert!(!same_sites(std::slice::from_ref(&dan), &[dan.clone(), gel]));
    }
}
