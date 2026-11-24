//! The account button of a public server, beside the navigation: the
//! account's picture, or its initial, on a round button that opens a menu
//! with its profile, Upload, Settings, the administration for
//! administrators and a way out. A guest's says Sign in and opens the ways
//! to an account, with Settings. The pages under it (Upload, Settings, the
//! account's own profile) light its ring, as an item of the navigation
//! lights for its pages.

use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
use send_wrapper::SendWrapper;
use starbooru_core::Role;
use wasm_bindgen::JsCast;

use crate::components::avatar::{Avatar, Size};
use crate::components::icons::{Ico, Icon};
use crate::components::menu::{ActionMenu, MenuAt, MenuItem, beside};
use crate::logic::profile::shown_name;
use crate::{activity, dom, instance, me, route, store};

/// What the menu holds, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Profile,
    Upload,
    Settings,
    Admin,
    SignOut,
    SignIn,
    SignUp,
}

/// The menu's items: an account's, with the administration for
/// administrators, or a guest's, with making an account where the server
/// takes new ones.
pub fn items(guest: bool, admin: bool, registration: bool) -> Vec<Item> {
    if guest {
        let mut out = vec![Item::SignIn];
        if registration {
            out.push(Item::SignUp);
        }
        out.push(Item::Settings);
        out
    } else {
        let mut out = vec![Item::Profile, Item::Upload, Item::Settings];
        if admin {
            out.push(Item::Admin);
        }
        out.push(Item::SignOut);
        out
    }
}

/// Where the menu opens: out from the rail, beside the button, so it
/// covers the page rather than the navigation; above or below it elsewhere.
fn menu_at(button: &web_sys::Element) -> MenuAt {
    let r = button.get_bounding_client_rect();
    let (width, height) = dom::window_size();
    match rail_side(r.left(), r.right(), width) {
        Some(Side::Left) => MenuAt::beside(r.right() + GAP, r.bottom(), r.bottom(), width, height),
        Some(Side::Right) => MenuAt::beside(r.left() - GAP, r.bottom(), r.bottom(), width, height),
        None => beside(button),
    }
}

/// Space between the rail and a menu out of it.
const GAP: f64 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    Left,
    Right,
}

/// Which side's rail a button from `left` to `right` stands in, in a
/// window `width` wide: one hugging an edge, on a window wide enough for
/// the rail.
fn rail_side(left: f64, right: f64, width: f64) -> Option<Side> {
    const EDGE: f64 = 120.0;
    if !(761.0..1530.0).contains(&width) {
        return None;
    }
    if right < EDGE {
        Some(Side::Left)
    } else if left > width - EDGE {
        Some(Side::Right)
    } else {
        None
    }
}

/// `place` is the class that says where this copy shows: `account-top`
/// beside the top bar's navigation, `account-bottom` under the rail or
/// beside the bar at the bottom of a phone.
#[component]
pub fn AccountButton(place: &'static str, here: Signal<bool>, guest: bool) -> impl IntoView {
    let open = RwSignal::new(None::<MenuAt>);
    let user = Memo::new(|_| store::session().map(|s| s.user));
    let profile = Memo::new(|_| me::profile());
    if !guest {
        me::ask();
    }
    // Uploads and imports still going, as the Upload item showed them.
    let pending = Memo::new(|_| activity::pending());
    let name = Signal::derive(move || {
        user.with(|u| u.as_ref().map(|u| u.username.clone()))
            .unwrap_or_default()
    });
    let display =
        Signal::derive(move || profile.with(|p| p.as_ref().and_then(|p| p.display_name.clone())));
    let picture = Signal::derive(move || {
        profile.with(|p| {
            p.as_ref()
                .and_then(|p| p.avatar_small_url.clone().or(p.avatar_url.clone()))
        })
    });
    let label = move || {
        if guest {
            t!("guest-sign-in")
        } else {
            t!(
                "account-button",
                name = name.with(|n| display.with(|d| shown_name(n, d.as_deref())))
            )
        }
    };
    let toggle = move |e: MouseEvent| {
        if open.with_untracked(Option::is_some) {
            open.set(None);
            return;
        }
        if let Some(button) = e
            .current_target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
        {
            open.set(Some(menu_at(&button)));
        }
    };
    let badge = move || pending.get().min(99);
    let note = move || (pending.get() != 0).then(|| t!("activity-pending", count = pending.get()));
    let face = move || {
        if guest {
            view! {
                <span class="account-face">
                    <Ico icon=Icon::Login />
                </span>
                <span class="account-label">{move || t!("guest-sign-in")}</span>
            }
            .into_any()
        } else {
            view! {
                <span class="account-face">
                    <Avatar name=name display=display url=picture />
                </span>
            }
            .into_any()
        }
    };
    view! {
        <button
            type="button"
            class=format!("account-btn glass float {place}")
            class:guest=guest
            class:here=move || here.get()
            // The sheet of files dropped on the gallery comes out of the
            // button that holds Upload.
            class:nav-upload=!guest
            aria-haspopup="menu"
            aria-expanded=move || if open.with(Option::is_some) { "true" } else { "false" }
            aria-current=move || if here.get() { "page" } else { "false" }
            aria-label=label
            title=move || note().unwrap_or_else(label)
            on:click=toggle
        >
            {face}
            <Show when=move || !guest && pending.get() != 0>
                <span class="nav-badge" aria-hidden="true">
                    {badge}
                </span>
            </Show>
        </button>
        {move || {
            open.get()
                .map(|at| {
                    view! {
                        <AccountMenu
                            at=at
                            guest=guest
                            name=name
                            display=display
                            picture=picture
                            on_close=Callback::new(move |()| open.set(None))
                        />
                    }
                })
        }}
    }
}

#[component]
fn AccountMenu(
    at: MenuAt,
    guest: bool,
    name: Signal<String>,
    display: Signal<Option<String>>,
    picture: Signal<Option<String>>,
    on_close: Callback<()>,
) -> impl IntoView {
    let navigate = SendWrapper::new(use_navigate());
    let go = move |to: String| {
        on_close.run(());
        (*navigate)(&to, NavigateOptions::default());
    };
    let go = SendWrapper::new(go);
    let admin = untrack(|| store::session().is_some_and(|s| s.user.role == Role::Admin));
    let registration = untrack(instance::registration_open);
    let pending = untrack(activity::pending);
    let rows = items(guest, admin, registration)
        .into_iter()
        .map(|item| {
            let go = go.clone();
            let run = Callback::new(move |()| match item {
                Item::Profile => (*go)(route::profile(&name.get_untracked())),
                Item::Upload => (*go)(route::UPLOAD.to_owned()),
                Item::Settings => (*go)(route::SETTINGS.to_owned()),
                Item::Admin => (*go)(route::ADMIN.to_owned()),
                Item::SignOut => {
                    on_close.run(());
                    store::sign_out();
                }
                Item::SignIn => {
                    on_close.run(());
                    instance::ask_sign_in(None);
                }
                Item::SignUp => {
                    on_close.run(());
                    instance::ask_sign_up();
                }
            });
            match item {
                Item::Profile => view! {
                    <button type="button" role="menuitem" class="menu-item account-head" on:click=move |_| run.run(())>
                        <Avatar name=name display=display url=picture size=Size::Big />
                        <span class="account-names">
                            <strong>{move || name.with(|n| display.with(|d| shown_name(n, d.as_deref())))}</strong>
                            <small>"@" {move || name.get()}</small>
                            <small class="account-see">{move || t!("account-see-profile")}</small>
                        </span>
                    </button>
                }
                .into_any(),
                Item::Upload => {
                    let hint = (pending != 0).then(|| Signal::derive(move || t!("activity-pending", count = pending)));
                    match hint {
                        Some(hint) => view! {
                            <MenuItem icon=Icon::Upload label=Signal::derive(|| t!("nav-upload")) hint=hint on_click=run />
                        }
                        .into_any(),
                        None => view! {
                            <MenuItem icon=Icon::Upload label=Signal::derive(|| t!("nav-upload")) on_click=run />
                        }
                        .into_any(),
                    }
                }
                Item::Settings => view! {
                    <MenuItem icon=Icon::Tune label=Signal::derive(|| t!("nav-settings")) on_click=run />
                }
                .into_any(),
                Item::Admin => view! {
                    <MenuItem icon=Icon::Shield label=Signal::derive(|| t!("admin-title")) on_click=run />
                }
                .into_any(),
                Item::SignOut => view! {
                    <MenuItem icon=Icon::Logout label=Signal::derive(|| t!("settings-sign-out")) on_click=run />
                }
                .into_any(),
                Item::SignIn => view! {
                    <MenuItem
                        icon=Icon::Login
                        label=Signal::derive(|| t!("guest-sign-in"))
                        hint=Signal::derive(|| t!("account-guest-hint"))
                        on_click=run
                    />
                }
                .into_any(),
                Item::SignUp => view! {
                    <MenuItem icon=Icon::User label=Signal::derive(|| t!("guest-sign-up")) on_click=run />
                }
                .into_any(),
            }
        })
        .collect_view();
    let title = if guest {
        untrack(|| t!("account-guest-title"))
    } else {
        untrack(|| t!("account-title"))
    };
    view! {
        <ActionMenu at=at title=title on_close=on_close>
            {rows}
        </ActionMenu>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_a_rail_the_menu_opens_out_of_it() {
        assert_eq!(rail_side(12.0, 88.0, 1280.0), Some(Side::Left));
        assert_eq!(rail_side(1192.0, 1268.0, 1280.0), Some(Side::Right));
        // On a phone it's in the bar at the bottom; on a wide window, the top.
        assert_eq!(rail_side(318.0, 378.0, 390.0), None);
        assert_eq!(rail_side(1800.0, 1852.0, 1920.0), None);
        let at = MenuAt::beside(98.0, 630.0, 630.0, 1280.0, 800.0);
        let style = at.style();
        assert!(style.starts_with("left: 98px; bottom: 170px"), "{style}");
    }

    #[test]
    fn the_menu_holds_what_moved_out_of_the_navigation() {
        assert_eq!(
            items(false, false, true),
            [Item::Profile, Item::Upload, Item::Settings, Item::SignOut]
        );
        assert_eq!(
            items(false, true, false),
            [
                Item::Profile,
                Item::Upload,
                Item::Settings,
                Item::Admin,
                Item::SignOut
            ]
        );
        // A guest gets the ways to an account, and still Settings.
        assert_eq!(
            items(true, false, true),
            [Item::SignIn, Item::SignUp, Item::Settings]
        );
        assert_eq!(items(true, true, false), [Item::SignIn, Item::Settings]);
    }
}
