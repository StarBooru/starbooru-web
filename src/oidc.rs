//! Signing in through the server's identity provider (OpenID Connect), and
//! linking it to the account signed in (Settings › Ways to sign in).
//!
//! The page leaves for the provider and comes back to `/oidc` with a code
//! (`pages/login.rs`, `OidcReturn`). The handle that code is worth nothing
//! without waits in `sessionStorage` meanwhile, for this tab only, with
//! whether it was a link.

use serde::{Deserialize, Serialize};
use starbooru_client::{Client, ClientError};
use starbooru_core::oidc::read_return;
use starbooru_core::{OidcError, OidcFinish};

use crate::{api, store};

/// What the page needs once it's back from the provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pending {
    pub server: String,
    pub handle: String,
    /// The provider's account goes to the one signed in, rather than
    /// signing in. The reference wrote this too.
    #[serde(default)]
    pub link: bool,
}

impl Pending {
    fn to_json(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }

    fn from_json(json: &str) -> Option<Pending> {
        serde_json::from_str(json).ok()
    }
}

/// Where the provider sends the browser back to: the app's own `/oidc`.
pub fn return_to(origin: &str) -> String {
    format!("{}/oidc", origin.trim_end_matches('/'))
}

/// Why a provider sign-in didn't end in a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Why {
    Denied,
    Failed,
    /// Too late, used already, or begun in another tab.
    Expired,
}

impl Why {
    pub fn text(self) -> String {
        match self {
            Why::Denied => t!("oidc-denied"),
            Why::Failed => t!("oidc-failed"),
            Why::Expired => t!("oidc-expired"),
        }
    }
}

/// What to do on `/oidc`.
#[derive(Debug, PartialEq, Eq)]
pub enum Back {
    /// Trade the code for a session on `server`, or for the provider's
    /// account on the one signed in when `link`.
    Finish {
        server: String,
        finish: OidcFinish,
        link: bool,
    },
    /// Say why it didn't work: on the sign-in screen, or after a link on the
    /// way back to Settings.
    Failed { why: Why, link: bool },
    /// Nothing came back: an address opened by hand.
    Nothing,
}

/// Reads the query the provider sent the browser back with, against the
/// sign-in this tab was waiting for.
pub fn back(query: &str, pending: Option<Pending>) -> Back {
    let link = pending.as_ref().is_some_and(|p| p.link);
    match (read_return(query), pending) {
        (None, _) => Back::Nothing,
        (Some(Err(OidcError::Denied)), _) => Back::Failed {
            why: Why::Denied,
            link,
        },
        (Some(Err(OidcError::Failed)), _) => Back::Failed {
            why: Why::Failed,
            link,
        },
        (Some(Ok(_)), None) => Back::Failed {
            why: Why::Expired,
            link,
        },
        (Some(Ok(code)), Some(pending)) => Back::Finish {
            server: pending.server,
            finish: OidcFinish {
                code,
                handle: pending.handle,
            },
            link,
        },
    }
}

/// Why `POST /auth/oidc/finish` failed, in words.
pub fn finish_error(e: &ClientError, server: &str) -> String {
    match e {
        // A code too late, used twice, or with the wrong handle.
        ClientError::Api { status: 400, .. } => Why::Expired.text(),
        e => api::error_text(e, server),
    }
}

/// Why `POST /auth/oidc/link` failed, from its status and the words for it.
pub fn link_error(status: Option<u16>, message: String) -> String {
    match status {
        Some(400) => Why::Expired.text(),
        _ => message,
    }
}

/// Starts a provider sign-in on `server`: the page then leaves for the
/// provider. An error says why it couldn't.
pub async fn begin(server: String) -> Result<(), String> {
    let client = Client::new(&server).map_err(|e| api::error_text(&e, &server))?;
    let origin = page::origin().ok_or_else(|| Why::Failed.text())?;
    let start = client
        .oidc_begin(&return_to(&origin))
        .await
        .map_err(|e| api::error_text(&e, &server))?;
    leave(
        &Pending {
            server,
            handle: start.handle,
            link: false,
        },
        &start.url,
    )
}

/// Starts linking the provider to the account signed in: the page leaves for
/// the provider and comes back to Settings. An error says why it couldn't.
pub async fn begin_link() -> Result<(), String> {
    let origin = page::origin().ok_or_else(|| Why::Failed.text())?;
    let server = store::session_untracked()
        .map(|s| s.server)
        .ok_or_else(|| t!("error-session-expired"))?;
    let back = return_to(&origin);
    let start = api::call(move |c| async move { c.oidc_begin_link(&back).await }).await?;
    leave(
        &Pending {
            server,
            handle: start.handle,
            link: true,
        },
        &start.url,
    )
}

/// Whether `url`, which the server gave, is a web address to leave for. A
/// hostile server could answer with a `javascript:` one, which would run on
/// this page.
fn is_web_address(url: &str) -> bool {
    // As the browser reads an address: tabs and line breaks anywhere in it
    // don't count, nor spaces and control characters in front.
    let read: String = url
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .collect();
    let start = read.trim_start_matches(|c: char| c <= ' ');
    let scheme = start.split_once(':').map(|(s, _)| s.to_ascii_lowercase());
    matches!(scheme.as_deref(), Some("http" | "https"))
}

/// Keeps what the way back needs and goes to the provider at `url`.
fn leave(pending: &Pending, url: &str) -> Result<(), String> {
    if !is_web_address(url) {
        return Err(Why::Failed.text());
    }
    let kept = pending.to_json().is_some_and(|json| page::keep(&json));
    if kept && page::go(url) {
        Ok(())
    } else {
        Err(Why::Failed.text())
    }
}

/// The sign-in this tab was waiting for, taken so it's used only once.
pub fn take() -> Option<Pending> {
    page::take().and_then(|json| Pending::from_json(&json))
}

#[cfg(target_arch = "wasm32")]
mod page {
    const KEY: &str = "starbooru-oidc";

    pub use crate::dom::origin;

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.session_storage().ok()?
    }

    pub fn keep(json: &str) -> bool {
        storage().is_some_and(|s| s.set_item(KEY, json).is_ok())
    }

    pub fn take() -> Option<String> {
        let storage = storage()?;
        let json = storage.get_item(KEY).ok()??;
        let _ = storage.remove_item(KEY);
        Some(json)
    }

    pub fn go(url: &str) -> bool {
        web_sys::window().is_some_and(|w| w.location().assign(url).is_ok())
    }
}

/// The host, where the tests run, has no page to leave.
#[cfg(not(target_arch = "wasm32"))]
mod page {
    pub fn origin() -> Option<String> {
        None
    }

    pub fn keep(_json: &str) -> bool {
        false
    }

    pub fn take() -> Option<String> {
        None
    }

    pub fn go(_url: &str) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use starbooru_core::ApiError;

    use super::*;

    fn pending() -> Pending {
        Pending {
            server: "https://booru.example".into(),
            handle: "h4ndle".into(),
            link: false,
        }
    }

    #[test]
    fn the_pending_sign_in_survives_the_trip() {
        let json = pending().to_json().unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json).unwrap(),
            serde_json::json!({"server": "https://booru.example", "handle": "h4ndle", "link": false})
        );
        assert_eq!(Pending::from_json(&json), Some(pending()));
        // What the reference app kept there reads too, and so does what an
        // earlier version did without `link`.
        assert_eq!(
            Pending::from_json(
                r#"{"server":"https://booru.example","handle":"h4ndle","link":true}"#
            )
            .map(|p| p.link),
            Some(true)
        );
        assert_eq!(
            Pending::from_json(r#"{"server":"https://booru.example","handle":"h4ndle"}"#),
            Some(pending())
        );
        assert_eq!(Pending::from_json(r#"{"server":"s"}"#), None);
        assert_eq!(Pending::from_json("not json"), None);
    }

    #[test]
    fn the_way_back_is_the_apps_oidc_page() {
        assert_eq!(
            return_to("http://127.0.0.1:8088"),
            "http://127.0.0.1:8088/oidc"
        );
        assert_eq!(
            return_to("https://booru.example/"),
            "https://booru.example/oidc"
        );
    }

    #[test]
    fn a_code_and_its_handle_finish_the_sign_in() {
        assert_eq!(
            back("?oidc_code=c0de&state=x", Some(pending())),
            Back::Finish {
                server: "https://booru.example".into(),
                finish: OidcFinish {
                    code: "c0de".into(),
                    handle: "h4ndle".into()
                },
                link: false,
            }
        );
        // A link comes back to the account signed in.
        let linking = Pending {
            link: true,
            ..pending()
        };
        assert!(matches!(
            back("?oidc_code=c0de", Some(linking)),
            Back::Finish { link: true, .. }
        ));
        // The code wins over an error, as the server never sends both.
        assert!(matches!(
            back("oidc_error=failed&oidc_code=c0de", Some(pending())),
            Back::Finish { .. }
        ));
    }

    #[test]
    fn what_went_wrong_is_said() {
        let failed = |why| Back::Failed { why, link: false };
        assert_eq!(
            back("?oidc_error=denied", Some(pending())),
            failed(Why::Denied)
        );
        assert_eq!(back("?oidc_error=failed", None), failed(Why::Failed));
        assert_eq!(back("?oidc_error=strange", None), failed(Why::Failed));
        // A code with no handle: begun in another tab, or used already.
        assert_eq!(back("?oidc_code=c0de", None), failed(Why::Expired));
        // A link that failed goes back to Settings to say so.
        let linking = Pending {
            link: true,
            ..pending()
        };
        assert_eq!(
            back("?oidc_error=denied", Some(linking)),
            Back::Failed {
                why: Why::Denied,
                link: true
            }
        );
        assert_eq!(back("", Some(pending())), Back::Nothing);
        assert_eq!(back("?oidc_code=", None), Back::Nothing);
        assert_eq!(back("?q=cat", None), Back::Nothing);
    }

    #[test]
    fn only_a_web_address_is_left_for() {
        for good in [
            "https://id.example/auth?x=1",
            "http://127.0.0.1:9000/auth",
            "HTTPS://ID.EXAMPLE/",
            "  https://id.example/",
        ] {
            assert!(is_web_address(good), "{good}");
        }
        for bad in [
            "javascript:alert(1)",
            "JavaScript:alert(1)",
            " \u{1}javascript:alert(1)",
            "java\tscript:alert(1)",
            "jav\nascript:alert(1)",
            "data:text/html,<script>alert(1)</script>",
            "vbscript:x",
            "//id.example/auth",
            "/auth",
            "",
            "https",
        ] {
            assert!(!is_web_address(bad), "{bad:?}");
        }
        // Refused before anything is kept or the page leaves.
        assert_eq!(
            leave(&pending(), "javascript:alert(1)"),
            Err(Why::Failed.text())
        );
    }

    #[test]
    fn a_stale_code_says_to_start_again() {
        let stale = ClientError::Api {
            status: 400,
            error: ApiError {
                code: "bad_request".into(),
                message: "unknown sign-in".into(),
            },
        };
        assert_eq!(
            finish_error(&stale, "https://booru.example"),
            "This sign-in has expired. Start again."
        );
        let closed = ClientError::Api {
            status: 403,
            error: ApiError {
                code: "forbidden".into(),
                message: "registration is closed".into(),
            },
        };
        assert_eq!(finish_error(&closed, "s"), "registration is closed");
        assert_eq!(
            link_error(Some(400), "x".into()),
            "This sign-in has expired. Start again."
        );
        assert_eq!(
            link_error(Some(409), "already linked to another account".into()),
            "already linked to another account"
        );
        assert_eq!(link_error(None, "offline".into()), "offline");
    }
}
