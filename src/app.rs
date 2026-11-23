//! The root of the app: the theme and the language from the preferences, and
//! which address shows which page.

use leptos::prelude::*;
use leptos_router::components::{ParentRoute, Route, Router, Routes};
use leptos_router::path;

use crate::components::snackbar::Snackbar;
use crate::i18n;
use crate::pages::NotFound;
use crate::pages::gallery::Gallery;
use crate::pages::help::Help;
use crate::pages::login::OidcReturn;
use crate::pages::post::PostView;
use crate::pages::shell::Shell;
use crate::pages::soon::{SettingsSoon, Soon, SoonAlone, TagSoon};
use crate::store;
use crate::theme;

#[component]
pub fn App() -> impl IntoView {
    let theme = Memo::new(|_| store::prefs().theme);
    let hue = Memo::new(|_| store::prefs().accent_hue);
    let picked = Memo::new(|_| store::prefs().language);
    let flush = Memo::new(|_| store::prefs().flush_tiles);
    // A language picked in Settings, else the browser's.
    Effect::new(move |_| {
        i18n::set_lang(i18n::resolve(picked.get(), &i18n::browser_languages()));
    });
    // The page itself says its language too, for screen readers and hyphens.
    Effect::new(|_| set_document_lang(i18n::lang().code()));
    Effect::new(move |_| {
        let (light, dark) = theme::bar_colors(theme.get(), hue.get());
        set_meta("theme-color-light", &light);
        set_meta("theme-color-dark", &dark);
    });
    view! {
        <div
            class="app"
            lang=move || i18n::lang().code()
            data-theme=move || theme.get().as_str()
            // Thumbnails edge to edge (style/pages/gallery.css).
            data-tiles=move || flush.get().then_some("flush")
            style=move || format!("--seed-hue: {}", hue.get())
        >
            <Router>
                <AppRoutes />
            </Router>
            <Snackbar />
        </div>
    }
}

/// Every address the app has, the reference's. A page still to be built
/// shows `Soon` with its name until it lands.
#[component]
fn AppRoutes() -> impl IntoView {
    view! {
        <Routes fallback=NotFound>
            // The sign-in pages, shared posts and the legal pages stand alone.
            <Route path=path!("/oidc") view=OidcReturn />
            <Route path=path!("/s/:token") view=|| view! { <SoonAlone title=|| t!("shared-title") /> } />
            <Route path=path!("/privacy") view=|| view! { <SoonAlone title=|| t!("privacy-title") /> } />
            <Route path=path!("/terms") view=|| view! { <SoonAlone title=|| t!("terms-title") /> } />
            <ParentRoute path=path!("") view=Shell>
                <Route path=path!("/") view=Gallery />
                <Route path=path!("/explore") view=|| view! { <Soon title=|| t!("nav-explore") /> } />
                <Route path=path!("/post/:id") view=PostView />
                <Route path=path!("/tags") view=|| view! { <Soon title=|| t!("nav-tags") /> } />
                <Route path=path!("/tag") view=TagSoon />
                <Route path=path!("/upload") view=|| view! { <Soon title=|| t!("nav-upload") /> } />
                <Route path=path!("/boorus") view=|| view! { <Soon title=|| t!("nav-boorus") /> } />
                <Route path=path!("/boorus/post") view=|| view! { <Soon title=|| t!("nav-boorus") /> } />
                <Route path=path!("/settings") view=SettingsSoon />
                <Route path=path!("/trash") view=|| view! { <Soon title=|| t!("trash-title") /> } />
                <Route path=path!("/duplicates") view=|| view! { <Soon title=|| t!("dup-title") /> } />
                <Route path=path!("/help") view=Help />
                <Route path=path!("/admin") view=|| view! { <Soon title=|| t!("admin-title") /> } />
            </ParentRoute>
        </Routes>
    }
}

/// `<html lang>`.
fn set_document_lang(code: &str) {
    if let Some(root) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let _ = root.set_attribute("lang", code);
    }
}

/// The content of the `<meta>` with this id in `index.html`.
fn set_meta(id: &str, content: &str) {
    if let Some(meta) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(id))
    {
        let _ = meta.set_attribute("content", content);
    }
}

#[cfg(test)]
mod tests {
    /// The body of the first rule in `css` for exactly `selector`.
    fn rule<'a>(css: &'a str, selector: &str) -> &'a str {
        let (_, body) = css
            .split_once(&format!("\n{selector} {{"))
            .unwrap_or_else(|| panic!("{selector}"));
        body.split_once('}').map_or(body, |(body, _)| body)
    }

    #[test]
    fn the_viewer_keeps_glass_to_its_floating_controls() {
        let post = include_str!("../style/pages/post.css");
        // Blur comes only from `.glass`, which the markup puts on the bar and
        // the stage's buttons.
        assert!(!post.contains("backdrop-filter"));
        let stage = include_str!("pages/post/stage.rs");
        let page = include_str!("pages/post/mod.rs");
        for class in [
            "stage-btn stage-back glass float",
            "stage-btn stage-prev glass float",
            "stage-btn stage-next glass float",
            "stage-tools glass float",
        ] {
            assert!(stage.contains(class), "{class}");
        }
        assert!(page.contains("post-bar glass float"));
        // Fullscreen is dark in both themes, glass included.
        let full = rule(post, ".stage.full");
        for token in [
            "background: var(--stage)",
            "--glass: var(--stage-glass)",
            "--on-surface: var(--on-stage)",
        ] {
            assert!(full.contains(token), "{token}");
        }
    }

    #[test]
    fn the_glow_moves_only_its_opacity_and_respects_reduced_effects() {
        let post = include_str!("../style/pages/post.css");
        assert!(rule(post, ".post-glow").contains("pointer-events: none"));
        assert!(rule(post, ".glow-layer").contains("transition: opacity"));
        assert!(rule(post, ".glow-layer.on").contains("opacity: var(--glow-a)"));
        assert!(rule(post, r#".app[data-effects="lite"] .post-glow"#).contains("display: none"));
        // Reduced motion leaves it unpainted (post/glow.rs), and the global rule
        // in base.css cuts the fades short.
        let glow = include_str!("pages/post/glow.rs");
        assert!(glow.contains("dom::reduced_motion()"));
    }

    #[test]
    fn the_stage_doesnt_move_with_the_pictures_shape() {
        let post = include_str!("../style/pages/post.css");
        let stage = rule(post, ".stage");
        assert!(stage.contains("height: var(--fit-h)"));
        assert!(rule(post, ".stage-tools .stage-btn").contains("width: var(--tool-w)"));
        assert!(rule(post, ".stage-zoom").contains("width: var(--zoom-w)"));
        assert!(rule(post, ".stage.full[data-rest]").contains("cursor: none"));
    }
    /// What `name: ` is set to in the first rule of `css` that starts with `rule`.
    fn token<'a>(css: &'a str, rule: &str, name: &str) -> &'a str {
        let (_, body) = css.split_once(rule).unwrap_or_else(|| panic!("{rule}"));
        let (body, _) = body.split_once('}').unwrap();
        let (_, value) = body
            .split_once(&format!("{name}: "))
            .unwrap_or_else(|| panic!("{name} in {rule}"));
        value.split_once(';').unwrap().0
    }

    #[test]
    fn the_loading_screen_is_in_the_apps_colours() {
        // It can't use the tokens, as it paints before the stylesheets come.
        let css = include_str!("../style/tokens.css");
        let page = include_str!("../index.html");
        for name in ["--canvas", "--primary-container", "--primary"] {
            assert_eq!(
                token(page, "#boot {", name),
                token(css, "\n.app {", name),
                "{name}, light"
            );
            let dark = token(css, r#".app[data-theme="dark"] {"#, name);
            assert_eq!(
                token(page, r#"#boot[data-theme="dark"] {"#, name),
                dark,
                "{name}, dark"
            );
            assert_eq!(
                token(page, r#"#boot:not([data-theme="light"]) {"#, name),
                dark,
                "{name}, dark by the system"
            );
            assert_eq!(
                token(css, r#".app[data-theme="system"] {"#, name),
                dark,
                "{name}, the system's dark"
            );
        }
        assert_eq!(token(page, "#boot {", "--h"), "285");
        assert!(css.contains("--h: var(--seed-hue, 285);"));
    }

    #[test]
    fn the_styles_load_in_order_and_take_the_loading_screen_away() {
        let page = include_str!("../index.html");
        let at = |s: &str| page.find(s).unwrap_or_else(|| panic!("{s}"));
        let tokens = at(r#"<link data-trunk rel="css" href="style/tokens.css">"#);
        let base = at(r#"<link data-trunk rel="css" href="style/base.css">"#);
        let shell = at(r#"<link data-trunk rel="css" href="style/shell.css">"#);
        let login = at(r#"<link data-trunk rel="css" href="style/pages/login.css">"#);
        let gallery = at(r#"<link data-trunk rel="css" href="style/pages/gallery.css">"#);
        let post = at(r#"<link data-trunk rel="css" href="style/pages/post.css">"#);
        let help = at(r#"<link data-trunk rel="css" href="style/pages/help.css">"#);
        assert!(
            tokens < base
                && base < shell
                && shell < login
                && login < gallery
                && gallery < post
                && post < help
        );
        // Empty, with not even a space in it, or `:empty` never matches.
        assert!(page.contains(r#"<div id="main"></div>"#));
        let base = include_str!("../style/base.css");
        assert!(base.contains(
            "#main:not(:empty) + #boot { opacity: 0; visibility: hidden; pointer-events: none;"
        ));
    }

    #[test]
    fn the_browser_bars_start_in_the_default_canvas() {
        let page = include_str!("../index.html");
        let (light, dark) = crate::theme::bar_colors(crate::store::Theme::System, 285);
        assert!(page.contains(&format!(
            r#"<meta id="theme-color-light" name="theme-color" content="{light}""#
        )));
        assert!(page.contains(&format!(
            r#"<meta id="theme-color-dark" name="theme-color" content="{dark}""#
        )));
    }

    #[test]
    fn the_glass_blurs_in_a_release_build() {
        // With both spellings, a minifier may keep only `-webkit-backdrop-filter`,
        // and Chromium shows no blur.
        for css in [
            include_str!("../style/tokens.css"),
            include_str!("../style/base.css"),
            include_str!("../style/shell.css"),
            include_str!("../style/pages/login.css"),
            include_str!("../style/pages/gallery.css"),
            include_str!("../style/pages/post.css"),
            include_str!("../style/pages/help.css"),
        ] {
            assert!(!css.contains("-webkit-backdrop-filter"));
        }
        assert!(include_str!("../style/tokens.css").contains("  backdrop-filter: blur("));
    }

    #[test]
    fn components_take_their_colours_from_the_tokens() {
        for (name, css) in [
            ("base", include_str!("../style/base.css")),
            ("shell", include_str!("../style/shell.css")),
            ("login", include_str!("../style/pages/login.css")),
            ("gallery", include_str!("../style/pages/gallery.css")),
            ("post", include_str!("../style/pages/post.css")),
            ("help", include_str!("../style/pages/help.css")),
        ] {
            // A colour worked out from the tokens is fine: `oklch(from var(--x) …)`,
            // or the strength meter's `oklch(var(--meter-l) …)`.
            let literal = css.match_indices("oklch(").any(|(i, _)| {
                !css.get(i + 6..).is_some_and(|rest| {
                    rest.starts_with("from var(--") || rest.starts_with("var(--")
                })
            });
            assert!(!literal, "{name}: a literal oklch() colour");
            for sign in ["rgb(", "rgba(", "hsl(", "#fff", "#000"] {
                assert!(!css.contains(sign), "{name}: {sign}");
            }
        }
    }
}
