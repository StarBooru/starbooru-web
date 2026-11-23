//! Signing in through the server's identity provider (OpenID Connect).
//!
//! The page leaves for the provider and comes back to `/oidc` with a code
//! (`pages/login.rs`, `OidcReturn`). The handle that code is worth nothing
//! without waits in `sessionStorage` meanwhile, for this tab only.

use serde::{Deserialize, Serialize};
use starbooru_client::{Client, ClientError};
use starbooru_core::oidc::read_return;
use starbooru_core::{OidcError, OidcFinish};

use crate::api;

/// What the page needs once it's back from the provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pending {
    pub server: String,
    pub handle: String,
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
    /// Trade the code for a session on `server`.
    Finish { server: String, finish: OidcFinish },
    /// Show the sign-in screen with why it didn't work.
    Failed(Why),
    /// Nothing came back: an address opened by hand.
    Nothing,
}

/// Reads the query the provider sent the browser back with, against the
/// sign-in this tab was waiting for.
pub fn back(query: &str, pending: Option<Pending>) -> Back {
    match (read_return(query), pending) {
        (None, _) => Back::Nothing,
        (Some(Err(OidcError::Denied)), _) => Back::Failed(Why::Denied),
        (Some(Err(OidcError::Failed)), _) => Back::Failed(Why::Failed),
        (Some(Ok(_)), None) => Back::Failed(Why::Expired),
        (Some(Ok(code)), Some(pending)) => Back::Finish {
            server: pending.server,
            finish: OidcFinish {
                code,
                handle: pending.handle,
            },
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

/// Starts a provider sign-in on `server`: the page then leaves for the
/// provider. An error says why it couldn't.
pub async fn begin(server: String) -> Result<(), String> {
    let client = Client::new(&server).map_err(|e| api::error_text(&e, &server))?;
    let origin = page::origin().ok_or_else(|| Why::Failed.text())?;
    let start = client
        .oidc_begin(&return_to(&origin))
        .await
        .map_err(|e| api::error_text(&e, &server))?;
    let pending = Pending {
        server,
        handle: start.handle,
    };
    let kept = pending.to_json().is_some_and(|json| page::keep(&json));
    if !kept {
        return Err(Why::Failed.text());
    }
    if page::go(&start.url) {
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

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.session_storage().ok()?
    }

    pub fn origin() -> Option<String> {
        web_sys::window()?.location().origin().ok()
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
        }
    }

    #[test]
    fn the_pending_sign_in_survives_the_trip() {
        let json = pending().to_json().unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json).unwrap(),
            serde_json::json!({"server": "https://booru.example", "handle": "h4ndle"})
        );
        assert_eq!(Pending::from_json(&json), Some(pending()));
        // What the reference app kept there reads too.
        assert_eq!(
            Pending::from_json(
                r#"{"server":"https://booru.example","handle":"h4ndle","link":false}"#
            ),
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
            }
        );
        // The code wins over an error, as the server never sends both.
        assert!(matches!(
            back("oidc_error=failed&oidc_code=c0de", Some(pending())),
            Back::Finish { .. }
        ));
    }

    #[test]
    fn what_went_wrong_is_said() {
        assert_eq!(
            back("?oidc_error=denied", Some(pending())),
            Back::Failed(Why::Denied)
        );
        assert_eq!(back("?oidc_error=failed", None), Back::Failed(Why::Failed));
        assert_eq!(back("?oidc_error=strange", None), Back::Failed(Why::Failed));
        // A code with no handle: begun in another tab, or used already.
        assert_eq!(back("?oidc_code=c0de", None), Back::Failed(Why::Expired));
        assert_eq!(back("", Some(pending())), Back::Nothing);
        assert_eq!(back("?oidc_code=", None), Back::Nothing);
        assert_eq!(back("?q=cat", None), Back::Nothing);
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
    }
}
