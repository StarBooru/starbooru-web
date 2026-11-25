//! The collection's people, its characters, as Explore and the people page
//! show them, without a browser: a tag's name in words, the page's groups by
//! franchise and the counts on their headings, what a page asks for, and
//! how long a page is kept for coming back to it.

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use starbooru_core::{FranchisePage, PeopleOrder, PeopleQuery, Person};

/// People asked for at a time on the people page.
pub const PAGE: u32 = 60;
/// Franchises asked for, for the headings' counts: all a page can hold.
pub const FRANCHISES: u32 = 200;
/// How long the people page is kept to come back to.
const KEPT_MINUTES: i64 = 10;

/// The orders the people page offers, the grouped one first.
pub const ORDERS: [PeopleOrder; 3] = [
    PeopleOrder::Franchise,
    PeopleOrder::Posts,
    PeopleOrder::Name,
];

/// A tag's name in words, as the app shows names: `hatsune_miku` reads
/// "hatsune miku". A name of underscores alone stays as it is.
pub fn spoken(name: &str) -> String {
    let words: Vec<&str> = name.split('_').filter(|w| !w.is_empty()).collect();
    if words.is_empty() {
        name.to_owned()
    } else {
        words.join(" ")
    }
}

/// Votes as a badge says them: `+12`, or `-3` for a character voted down.
pub fn signed(votes: i64) -> String {
    if votes > 0 {
        format!("+{votes}")
    } else {
        votes.to_string()
    }
}

/// One page of `GET /people` for the people page.
pub fn query(q: &str, order: PeopleOrder, page: u32) -> PeopleQuery {
    PeopleQuery {
        q: q.split_whitespace().collect::<Vec<_>>().join(" "),
        franchise: None,
        order,
        page,
        limit: PAGE,
    }
}

/// A page more of people after `list`: each person once, where they came
/// first, as one who moved up a page since would come twice.
pub fn joined(list: &mut Vec<Person>, more: Vec<Person>) {
    for person in more {
        if !list.iter().any(|p| p.name == person.name) {
            list.push(person);
        }
    }
}

/// The people of one franchise in a row, or of none.
#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub franchise: Option<String>,
    pub people: Vec<Person>,
}

/// People as `order=franchise` lists them, a group for each franchise in a
/// row. The server puts those without one last.
pub fn grouped(people: &[Person]) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    for person in people {
        match groups.last_mut() {
            Some(last) if last.franchise == person.franchise => last.people.push(person.clone()),
            _ => groups.push(Group {
                franchise: person.franchise.clone(),
                people: vec![person.clone()],
            }),
        }
    }
    groups
}

/// How many people each franchise has, and how many have none, as
/// `GET /franchises` counts them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Counts {
    by_name: HashMap<String, i64>,
    without: i64,
}

impl Counts {
    pub fn of(page: &FranchisePage) -> Self {
        Self {
            by_name: page
                .franchises
                .iter()
                .map(|f| (f.name.clone(), f.people))
                .collect(),
            without: page.without,
        }
    }

    /// The count on a franchise's heading: the server's for everyone, or
    /// those listed under it when a search narrows the page (the server
    /// counts franchises by their own names) or the server didn't say.
    pub fn heading(&self, franchise: Option<&str>, listed: usize, narrowed: bool) -> usize {
        if narrowed {
            return listed;
        }
        let known = match franchise {
            Some(name) => self.by_name.get(name).copied(),
            None => Some(self.without),
        };
        known
            .and_then(|n| usize::try_from(n).ok())
            .map_or(listed, |n| n.max(listed))
    }
}

/// The people page as it was left: what it listed for a search and an
/// order, how far it had gone and where it was scrolled.
#[derive(Clone, Debug, PartialEq)]
pub struct Listed {
    pub owner: String,
    pub q: String,
    pub order: PeopleOrder,
    pub people: Vec<Person>,
    pub next: u32,
    pub has_more: bool,
    pub counts: Option<Counts>,
    pub scroll: f64,
    pub at: DateTime<Utc>,
}

impl Listed {
    /// Whether it is the page `owner` asks for now with `q` and `order`, and
    /// recent enough to show.
    pub fn fits(&self, owner: &str, q: &str, order: PeopleOrder, now: DateTime<Utc>) -> bool {
        let age = now - self.at;
        self.owner == owner
            && self.q == q
            && self.order == order
            && age >= Duration::zero()
            && age < Duration::minutes(KEPT_MINUTES)
    }
}

#[cfg(test)]
mod tests {
    use starbooru_core::Franchise;

    use super::*;

    fn person(name: &str, franchise: Option<&str>) -> Person {
        Person {
            name: name.into(),
            posts: 1,
            franchise: franchise.map(str::to_owned),
            ..Person::default()
        }
    }

    #[test]
    fn a_name_reads_as_words() {
        assert_eq!(spoken("hatsune_miku"), "hatsune miku");
        assert_eq!(spoken("saber_(fate)"), "saber (fate)");
        assert_eq!(spoken("_a__b_"), "a b");
        assert_eq!(spoken("__"), "__");
        assert_eq!(spoken("link"), "link");
    }

    #[test]
    fn votes_say_which_way() {
        assert_eq!(signed(12), "+12");
        assert_eq!(signed(0), "0");
        assert_eq!(signed(-3), "-3");
    }

    #[test]
    fn a_page_asks_for_the_search_and_order_chosen() {
        let q = query("  hatsune   miku ", PeopleOrder::Name, 3);
        assert_eq!(q.q, "hatsune miku");
        assert_eq!(q.order, PeopleOrder::Name);
        assert_eq!((q.page, q.limit), (3, PAGE));
        assert_eq!(q.franchise, None);
        assert_eq!(ORDERS.first(), Some(&PeopleOrder::Franchise));
    }

    #[test]
    fn pages_join_without_repeats() {
        let mut list = vec![person("a", None), person("b", None)];
        joined(&mut list, vec![person("b", Some("x")), person("c", None)]);
        let names: Vec<&str> = list.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["a", "b", "c"]);
        assert_eq!(list.get(1).and_then(|p| p.franchise.clone()), None);
    }

    #[test]
    fn people_group_by_franchise_in_the_servers_order() {
        let people = [
            person("shinji", Some("eva")),
            person("asuka", Some("eva")),
            person("saber", Some("fate")),
            person("nobody", None),
            person("someone", None),
        ];
        let groups = grouped(&people);
        let heads: Vec<(Option<&str>, usize)> = groups
            .iter()
            .map(|g| (g.franchise.as_deref(), g.people.len()))
            .collect();
        assert_eq!(heads, [(Some("eva"), 2), (Some("fate"), 1), (None, 2)]);
        assert!(grouped(&[]).is_empty());
    }

    #[test]
    fn a_heading_counts_the_franchises_people() {
        let page = FranchisePage {
            franchises: vec![Franchise {
                name: "eva".into(),
                people: 7,
                posts: 40,
            }],
            without: 3,
            ..FranchisePage::default()
        };
        let counts = Counts::of(&page);
        assert_eq!(counts.heading(Some("eva"), 2, false), 7);
        assert_eq!(counts.heading(None, 1, false), 3);
        // Not among the franchises counted: those listed.
        assert_eq!(counts.heading(Some("fate"), 4, false), 4);
        // Narrowed by a search: those listed, whatever the server counted.
        assert_eq!(counts.heading(Some("eva"), 2, true), 2);
        // Never fewer than are on the page.
        assert_eq!(counts.heading(Some("eva"), 9, false), 9);
    }

    #[test]
    fn a_page_is_kept_for_the_same_reader_search_and_order_a_while() {
        let now = Utc::now();
        let listed = Listed {
            owner: "s|1".into(),
            q: "mi".into(),
            order: PeopleOrder::Franchise,
            people: Vec::new(),
            next: 2,
            has_more: true,
            counts: None,
            scroll: 120.0,
            at: now,
        };
        assert!(listed.fits("s|1", "mi", PeopleOrder::Franchise, now));
        assert!(listed.fits(
            "s|1",
            "mi",
            PeopleOrder::Franchise,
            now + Duration::minutes(9)
        ));
        assert!(!listed.fits(
            "s|1",
            "mi",
            PeopleOrder::Franchise,
            now + Duration::minutes(10)
        ));
        assert!(!listed.fits("s|2", "mi", PeopleOrder::Franchise, now));
        assert!(!listed.fits("s|1", "m", PeopleOrder::Franchise, now));
        assert!(!listed.fits("s|1", "mi", PeopleOrder::Name, now));
        // A clock set back.
        assert!(!listed.fits(
            "s|1",
            "mi",
            PeopleOrder::Franchise,
            now - Duration::minutes(1)
        ));
    }
}
