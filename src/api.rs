//! The way to the server: one [`Client`] for the signed-in session, and what
//! to say when a request fails. Pages make their requests through [`call`],
//! so a session the server no longer knows signs out in one place.

use std::cell::RefCell;
use std::future::Future;

use chrono::{DateTime, Utc};
use starbooru_client::{Client, ClientError};
use starbooru_core::ServerInfo;

use crate::store::{self, Session};

thread_local! {
    // One client per session, so its connections are reused.
    static CLIENT: RefCell<Option<(String, String, Client)>> = const { RefCell::new(None) };
    static SERVER_INFO: RefCell<Kept<ServerInfo>> = const { RefCell::new(Kept::new()) };
}

/// The client for the signed-in session, or `None` when no one is.
pub fn client() -> Option<Client> {
    let session = store::session_untracked()?;
    client_for(&session)
}

fn client_for(session: &Session) -> Option<Client> {
    CLIENT.with_borrow_mut(|cache| {
        if let Some((server, token, client)) = cache.as_ref()
            && *server == session.server
            && *token == session.token
        {
            return Some(client.clone());
        }
        let client = Client::new(&session.server)
            .ok()?
            .with_token(Some(session.token.clone()));
        *cache = Some((
            session.server.clone(),
            session.token.clone(),
            client.clone(),
        ));
        Some(client)
    })
}

/// Drops the client and what it was told, at sign-out.
pub fn forget() {
    CLIENT.with_borrow_mut(|cache| *cache = None);
    SERVER_INFO.with_borrow_mut(|kept| kept.0 = None);
}

/// Runs `request` with the session's client. A failure comes back as words
/// to show; when the server no longer knows the session, the app signs out
/// and the sign-in screen says why.
pub async fn call<T, F, Fut>(request: F) -> Result<T, String>
where
    F: FnOnce(Client) -> Fut,
    Fut: Future<Output = Result<T, ClientError>>,
{
    let Some(session) = store::session_untracked() else {
        return Err(t!("error-session-expired"));
    };
    let Some(client) = client_for(&session) else {
        return Err(t!("error-bad-address"));
    };
    request(client).await.map_err(|e| {
        if e.is_unauthorized() {
            expire(&session.token);
        }
        error_text(&e, &session.server)
    })
}

/// Signs out after a 401, unless the session it was for has gone already:
/// an answer that comes late mustn't end the next one.
fn expire(token: &str) {
    let current = store::session_untracked().is_some_and(|s| s.token == token);
    if current {
        store::set_sign_in_note(t!("error-session-expired"));
        store::forget_session();
    }
}

/// What a failed request means for the person using the app.
#[derive(Debug, PartialEq, Eq)]
enum Trouble<'a> {
    SignedOut,
    Unreachable,
    BadAddress,
    NotAServer,
    /// Minutes to wait, when the server said.
    RateLimited(Option<u64>),
    /// The server's own words.
    Said(&'a str),
    Other,
}

fn trouble(e: &ClientError) -> Trouble<'_> {
    match e {
        ClientError::Api { status: 401, .. } => Trouble::SignedOut,
        // No JSON in the answer: a proxy in front of a server that's down,
        // or some other site at that address.
        ClientError::Api {
            status: 502..=504,
            error,
        } if error.code == "http_error" => Trouble::Unreachable,
        ClientError::Api { status: 404, error } if error.code == "http_error" => {
            Trouble::NotAServer
        }
        ClientError::Api { error, .. } => Trouble::Said(&error.message),
        ClientError::Http(e) if e.is_decode() => Trouble::NotAServer,
        ClientError::Http(_) => Trouble::Unreachable,
        ClientError::InvalidUrl(_) => Trouble::BadAddress,
        // As the client words it: a minute at least.
        ClientError::RateLimited { retry_after } => {
            Trouble::RateLimited(retry_after.map(|s| s.div_ceil(60).max(1)))
        }
        ClientError::TooLarge { .. } | ClientError::Io(_) | ClientError::Read(_) => Trouble::Other,
    }
}

/// A failed request in words the user can act on. `server` is the address it
/// went to.
pub fn error_text(e: &ClientError, server: &str) -> String {
    let server = server.to_owned();
    match trouble(e) {
        Trouble::SignedOut => t!("error-session-expired"),
        Trouble::Unreachable => t!("error-unreachable", server = server),
        Trouble::BadAddress => t!("error-bad-address"),
        Trouble::NotAServer => t!("error-not-a-server", server = server),
        Trouble::RateLimited(None) => t!("error-rate-limited"),
        Trouble::RateLimited(Some(minutes)) => t!("error-rate-limited-minutes", minutes = minutes),
        Trouble::Said(message) => message.to_owned(),
        Trouble::Other => e.to_string(),
    }
}

/// What the server offers (`GET /info`), asked at most every two minutes.
pub async fn server_info() -> Option<ServerInfo> {
    let session = store::session_untracked()?;
    let owner = owner(&session);
    if let Some(known) = SERVER_INFO.with_borrow(|kept| kept.get(&owner, Utc::now())) {
        return Some(known);
    }
    let info = call(|c| async move { c.info().await }).await.ok()?;
    SERVER_INFO.with_borrow_mut(|kept| kept.put(owner, Utc::now(), info.clone()));
    Some(info)
}

/// Whose an answer is: one account on one server.
fn owner(session: &Session) -> String {
    format!("{}|{}", session.server, session.user.id)
}

/// How long an answer about the server itself is good for: long enough for
/// one page after another, short enough that a change made elsewhere shows
/// soon.
const KEPT_SECONDS: i64 = 120;

/// An answer that rarely changes, with whose it is and when it came.
struct Kept<T>(Option<(String, DateTime<Utc>, T)>);

impl<T: Clone> Kept<T> {
    const fn new() -> Self {
        Self(None)
    }

    fn get(&self, owner: &str, now: DateTime<Utc>) -> Option<T> {
        let (whose, at, value) = self.0.as_ref()?;
        let age = (now - *at).num_seconds();
        // A clock set back makes an answer from the future: ask again.
        (whose == owner && (0..KEPT_SECONDS).contains(&age)).then(|| value.clone())
    }

    fn put(&mut self, owner: String, now: DateTime<Utc>, value: T) {
        self.0 = Some((owner, now, value));
    }
}

#[cfg(test)]
mod tests {
    use starbooru_core::ApiError;

    use super::*;

    fn api(status: u16, code: &str, message: &str) -> ClientError {
        ClientError::Api {
            status,
            error: ApiError {
                code: code.into(),
                message: message.into(),
            },
        }
    }

    /// A request that never left: what the browser reports for a server
    /// that isn't there.
    fn network() -> ClientError {
        let e = reqwest::Client::new()
            .get("http://exa mple.invalid")
            .build()
            .unwrap_err();
        ClientError::Http(e)
    }

    #[test]
    fn each_failure_means_something_to_the_user() {
        assert_eq!(trouble(&api(401, "unauthorized", "x")), Trouble::SignedOut);
        assert_eq!(trouble(&network()), Trouble::Unreachable);
        assert_eq!(
            trouble(&api(502, "http_error", "server answered 502")),
            Trouble::Unreachable
        );
        assert_eq!(
            trouble(&api(404, "http_error", "server answered 404")),
            Trouble::NotAServer
        );
        assert_eq!(
            trouble(&ClientError::InvalidUrl("x".into())),
            Trouble::BadAddress
        );
        // The server's JSON errors keep its words, whatever the status.
        assert_eq!(
            trouble(&api(404, "not_found", "no such post")),
            Trouble::Said("no such post")
        );
        assert_eq!(
            trouble(&api(503, "storage_full", "disk full")),
            Trouble::Said("disk full")
        );
        assert_eq!(
            trouble(&api(409, "username_taken", "username taken")),
            Trouble::Said("username taken")
        );
    }

    #[test]
    fn a_rate_limit_says_how_long_to_wait() {
        let minutes = |retry_after| match trouble(&ClientError::RateLimited { retry_after }) {
            Trouble::RateLimited(minutes) => minutes,
            other => panic!("{other:?}"),
        };
        assert_eq!(minutes(None), None);
        assert_eq!(minutes(Some(0)), Some(1));
        assert_eq!(minutes(Some(45)), Some(1));
        assert_eq!(minutes(Some(61)), Some(2));
        assert_eq!(minutes(Some(900)), Some(15));
    }

    #[test]
    fn the_words_name_the_server_and_the_wait() {
        let server = "https://booru.example";
        assert_eq!(
            error_text(&network(), server),
            "Can't reach the server at https://booru.example. Check the address and your connection, then try again."
        );
        assert_eq!(
            error_text(&api(401, "unauthorized", "x"), server),
            "Your session has expired. Sign in again."
        );
        assert_eq!(
            error_text(
                &ClientError::RateLimited {
                    retry_after: Some(30)
                },
                server
            ),
            "Too many attempts. Try again in a minute."
        );
        assert_eq!(
            error_text(
                &ClientError::RateLimited {
                    retry_after: Some(600)
                },
                server
            ),
            "Too many attempts. Try again in 10 minutes."
        );
        assert_eq!(
            error_text(&ClientError::RateLimited { retry_after: None }, server),
            "Too many requests. Try again later."
        );
        assert_eq!(
            error_text(&api(409, "c", "username taken"), server),
            "username taken"
        );
    }

    #[test]
    fn an_answer_is_kept_two_minutes_for_one_account() {
        let now = Utc::now();
        let mut kept = Kept::new();
        assert_eq!(kept.get("a|1", now), None::<u8>);
        kept.put("a|1".into(), now, 7u8);
        assert_eq!(kept.get("a|1", now), Some(7));
        let later = |s| now + chrono::Duration::seconds(s);
        assert_eq!(kept.get("a|1", later(KEPT_SECONDS - 1)), Some(7));
        assert_eq!(kept.get("a|1", later(KEPT_SECONDS)), None);
        // Another server or account, or a clock that went backwards.
        assert_eq!(kept.get("b|1", now), None);
        assert_eq!(kept.get("a|1", later(-5)), None);
    }

    #[test]
    fn the_client_is_the_sessions() {
        let user = serde_json::from_value(serde_json::json!({
            "id": 3, "username": "ada", "role": "user",
            "created_at": "2026-01-02T03:04:05Z",
        }))
        .unwrap();
        let session = Session {
            server: "https://booru.example/".into(),
            token: "t1".into(),
            user,
        };
        let client = client_for(&session).unwrap();
        assert_eq!(client.base_url(), "https://booru.example");
        assert_eq!(client.token(), Some("t1"));
        let other = Session {
            token: "t2".into(),
            ..session.clone()
        };
        assert_eq!(client_for(&other).unwrap().token(), Some("t2"));
        assert_eq!(owner(&session), "https://booru.example/|3");
        let nowhere = Session {
            server: "not an address".into(),
            ..session
        };
        assert!(client_for(&nowhere).is_none());
    }
}
