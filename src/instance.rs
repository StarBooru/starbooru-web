//! What the server in use is: private or public, likes or stars, and whether
//! people without an account may browse it. The views that follow the mode
//! read it here, so they change as soon as an answer to `/info` does: when
//! the shell starts, every two minutes as pages ask (`api::server_info`), and
//! right after an administrator switches. Until the server says, it counts
//! as private with likes.
//!
//! Without a session this is also where a guest's server is known, and
//! where a guest asks for the sign-in screen. A guest has no session and
//! nothing of an account: requests go out without a token.

use std::cell::Cell;

use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_client::Client;
use starbooru_core::{Mode, ScoreKind, ServerInfo};

use crate::logic::modes::Guests;
use crate::{api, store};

/// A guest asked for the sign-in screen: to make an account rather than
/// sign in, and why, when a press on something that needs one brought it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Asked {
    pub create: bool,
    pub why: Option<String>,
}

thread_local! {
    // Reference counted, as they outlive every page.
    static INFO: ArcRwSignal<Option<(String, ServerInfo)>> = ArcRwSignal::new(None);
    static GUESTS: ArcRwSignal<Guests> = ArcRwSignal::new(Guests::Asking);
    static GUEST_SERVER: ArcRwSignal<Option<String>> = ArcRwSignal::new(None);
    static ASKED: ArcRwSignal<Option<Asked>> = ArcRwSignal::new(None);
    /// The server in use was asked, whether or not it answered.
    static HEARD: ArcRwSignal<Option<String>> = ArcRwSignal::new(None);
    /// Bumped by each question about guests, so only the latest answer counts.
    static ASKING: Cell<u32> = const { Cell::new(0) };
}

/// The server the app talks to: the session's, or the guest's.
fn server_in_use() -> Option<String> {
    store::session()
        .map(|s| s.server)
        .or_else(|| GUEST_SERVER.with(|g| g.get()))
}

/// What the server in use said of itself, once it has. Read inside a view,
/// it follows each new answer.
pub fn info() -> Option<ServerInfo> {
    let server = server_in_use()?;
    INFO.with(|i| {
        i.with(|i| {
            i.as_ref()
                .filter(|(of, _)| *of == server)
                .map(|(_, info)| info.clone())
        })
    })
}

pub fn mode() -> Mode {
    info().map(|i| i.mode).unwrap_or_default()
}

pub fn public() -> bool {
    mode() == Mode::Public
}

/// The score kind in force: a public server counts likes, whatever is set.
pub fn score_kind() -> ScoreKind {
    info()
        .map(|i| i.score_kind.in_mode(i.mode))
        .unwrap_or_default()
}

pub fn stars() -> bool {
    score_kind() == ScoreKind::Stars
}

pub fn registration_open() -> bool {
    info().is_some_and(|i| i.registration_open)
}

/// Whether the server has its home page on: `None` until it has been asked.
/// A server older than the home page says nothing of it, and one that
/// didn't answer has none to show: both read as off.
pub fn home() -> Option<bool> {
    match info() {
        Some(i) => Some(i.home),
        None => known().then_some(false),
    }
}

/// Whether the server in use has been asked what it is, so a page that
/// differs by mode can show the right one at once. A server that didn't
/// answer counts as asked: its pages say why when they fail.
pub fn known() -> bool {
    let Some(server) = server_in_use() else {
        return false;
    };
    info().is_some() || HEARD.with(|h| h.with(|h| h.as_deref() == Some(server.as_str())))
}

fn heard(server: String) {
    HEARD.with(|h| {
        if h.with_untracked(|h| h.as_deref() != Some(server.as_str())) {
            h.set(Some(server));
        }
    });
}

/// Keeps what `server` said of itself. A guest whose server stops taking
/// guests gets the sign-in screen.
pub fn learn(server: &str, info: ServerInfo) {
    let welcome = info.guests;
    INFO.with(|i| {
        let same = i.with_untracked(|i| {
            i.as_ref()
                .is_some_and(|(of, known)| of == server && *known == info)
        });
        if !same {
            i.set(Some((server.to_owned(), info)));
        }
    });
    let guests_here = store::session_untracked().is_none()
        && GUEST_SERVER.with(|g| g.with_untracked(|g| g.as_deref() == Some(server)));
    if guests_here && !welcome {
        set_guests(Guests::No);
    }
}

/// Asks the server in use again, and tells the views what it says now.
pub fn refresh() {
    api::forget_server_answers();
    let asking = server_in_use_untracked();
    spawn_local(async move {
        let _ = api::server_info().await;
        if let Some(server) = asking {
            heard(server);
        }
    });
}

fn server_in_use_untracked() -> Option<String> {
    store::session_untracked()
        .map(|s| s.server)
        .or_else(|| GUEST_SERVER.with(|g| g.get_untracked()))
}

/// The same, with the settings an administrator just saved, so the views
/// change before the server has been asked again.
pub fn settled(mode: Mode, score_kind: ScoreKind) {
    INFO.with(|i| {
        i.update(|i| {
            if let Some((_, info)) = i {
                info.mode = mode;
                info.score_kind = score_kind;
                info.guests = mode.guests();
            }
        });
    });
    refresh();
}

/// The home page an administrator just turned on or off, so the brand and
/// the start follow before the server has been asked again.
pub fn settled_home(on: bool) {
    INFO.with(|i| {
        i.update(|i| {
            if let Some((_, info)) = i {
                info.home = on;
            }
        });
    });
    refresh();
}

/// Forgets what the server said, at sign-out.
pub fn forget() {
    INFO.with(|i| i.set(None));
    HEARD.with(|h| h.set(None));
}

fn set_guests(now: Guests) {
    GUESTS.with(|g| {
        let was = g.get_untracked();
        if was != now {
            // A guest turned away: the sign-in screen says why.
            if was == Guests::Welcome && now == Guests::No {
                store::set_sign_in_note(t!("guest-server-closed"));
            }
            g.set(now);
        }
    });
}

/// Whether the server lets people browse without an account. Read inside a
/// view, it follows the answer.
pub fn guests() -> Guests {
    GUESTS.with(|g| g.get())
}

/// Whether the app is browsing without an account. Read inside a view, it
/// follows sign-in and the server's answer.
pub fn guest() -> bool {
    !store::signed_in() && guests() == Guests::Welcome
}

/// The server a guest browses, outside a view.
pub fn guest_server() -> Option<String> {
    let welcome = GUESTS.with(|g| g.get_untracked()) == Guests::Welcome;
    GUEST_SERVER
        .with(|g| g.get_untracked())
        .filter(|_| welcome && store::session_untracked().is_none())
}

/// Asks the server the sign-in screen would start with whether it takes
/// guests. Until it answers, the shell waits; a server that doesn't answer
/// gets the sign-in screen, which says why.
pub fn ask_guests() {
    set_guests(Guests::Asking);
    probe(store::default_server());
}

/// A guest's request was turned away: the server may have stopped taking
/// guests. Asked again without leaving the page meanwhile.
pub fn recheck() {
    if let Some(server) = guest_server() {
        probe(server);
    }
}

fn probe(server: String) {
    let n = ASKING.with(|a| {
        let n = a.get().wrapping_add(1);
        a.set(n);
        n
    });
    if server.is_empty() {
        set_guests(Guests::No);
        return;
    }
    spawn_local(async move {
        let answer = match Client::new(&server) {
            Ok(client) => client.info().await.ok(),
            Err(_) => None,
        };
        if ASKING.with(Cell::get) != n || store::session_untracked().is_some() {
            return;
        }
        match answer {
            Some(info) if info.guests => {
                GUEST_SERVER.with(|g| g.set(Some(server.clone())));
                learn(&server, info);
                set_guests(Guests::Welcome);
            }
            Some(info) => {
                learn(&server, info);
                set_guests(Guests::No);
            }
            None => set_guests(Guests::No),
        }
    });
}

/// A guest asks for the sign-in screen, saying why when `why` does.
pub fn ask_sign_in(why: Option<String>) {
    ASKED.with(|a| a.set(Some(Asked { create: false, why })));
}

/// A guest asks for the sign-in screen, to make an account.
pub fn ask_sign_up() {
    ASKED.with(|a| {
        a.set(Some(Asked {
            create: true,
            why: None,
        }));
    });
}

/// What a guest asked for. Read inside a view, it follows the asking.
pub fn asked() -> Option<Asked> {
    ASKED.with(|a| a.get())
}

/// Back to browsing, or signed in: the sign-in screen goes.
pub fn stop_asking() {
    ASKED.with(|a| {
        if a.with_untracked(Option::is_some) {
            a.set(None);
        }
    });
}
