//! Editing your own profile, on its page and in Settings › Account: the
//! picture, picked from a file and cut to a circle in the crop picker, or
//! removed; the display name; and the text about you. Public servers only:
//! a private one shows nobody a profile.

use leptos::ev::SubmitEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use send_wrapper::SendWrapper;
use starbooru_core::Profile;
use starbooru_core::profile::{MAX_ABOUT_CHARS, MAX_DISPLAY_NAME_CHARS};
use web_sys::File;

use super::crop::CropPicker;
use crate::components::avatar::{Avatar, Size};
use crate::components::banner::ErrorBanner;
use crate::components::banner::Spinner;
use crate::components::confirm::ConfirmButton;
use crate::components::icons::{Ico, Icon};
use crate::components::snackbar::notify;
use crate::logic::forum::{chars_left, left_words};
use crate::logic::profile::{
    Crop, check_about, check_name, check_picture, error_text, picture_problem_text, update,
};
use crate::{api, files, instance, me, route, store};

/// A file picked for the picture, and the address the page shows it at.
#[derive(Clone)]
struct Picked {
    file: SendWrapper<File>,
    url: String,
}

/// The form, starting from `profile`. `on_saved` hears each change the
/// server kept.
#[component]
pub fn ProfileForm(profile: Profile, on_saved: Callback<Profile>) -> impl IntoView {
    let saved = RwSignal::new(profile.clone());
    let name = RwSignal::new(profile.display_name.clone().unwrap_or_default());
    let about = RwSignal::new(profile.about.clone().unwrap_or_default());
    let picked = RwSignal::new(None::<Picked>);
    let crop = RwSignal::new(None::<Crop>);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let done = RwSignal::new(false);
    let input: NodeRef<html::Input> = NodeRef::new();
    let public = Memo::new(|_| instance::public());

    let name_problem =
        Memo::new(move |_| name.with(|n| check_name(n).err().map(|e| error_text(&e))));
    let about_problem =
        Memo::new(move |_| about.with(|a| check_about(a).err().map(|e| error_text(&e))));
    let request = Memo::new(move |_| {
        let about = public.get().then(|| about.get());
        saved.with(|s| update(s, &name.read(), about.as_deref()))
    });
    let kept = move |p: Profile| {
        saved.try_set(p.clone());
        me::set(p.clone());
        on_saved.try_run(p);
    };

    let forget_pick = move || {
        if let Some(p) = picked.try_update(Option::take).flatten() {
            files::release(&p.url);
        }
        crop.try_set(None);
    };
    on_cleanup(forget_pick);
    let pick = move |_| {
        let Some(el) = input.get_untracked() else {
            return;
        };
        let file = el.files().and_then(|l| l.get(0));
        // The same file picked again still fires.
        el.set_value("");
        let Some(file) = file else {
            return;
        };
        error.set(None);
        if let Err(problem) = check_picture(file.size(), &file.type_()) {
            error.set(Some(picture_problem_text(problem)));
            return;
        }
        forget_pick();
        if let Some(url) = files::object_url(&file) {
            picked.set(Some(Picked {
                file: SendWrapper::new(file),
                url,
            }));
        }
    };
    let choose = move |_| {
        if let Some(el) = input.get_untracked() {
            el.click();
        }
    };
    let use_picture = move |_| {
        let (Some(p), Some(square)) = (picked.get_untracked(), crop.get_untracked()) else {
            return;
        };
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        let square = square.square();
        spawn_local(async move {
            let size = p.file.size().max(0.0) as u64;
            let read = files::read_range(&p.file, 0, size).await;
            let result = match read {
                Ok(bytes) => {
                    let bytes = bytes.to_vec();
                    api::call(move |c| async move { c.set_avatar(bytes, Some(square)).await }).await
                }
                Err(why) => Err(untrack(|| t!("upload-unreadable", error = why))),
            };
            busy.try_set(false);
            match result {
                Ok(profile) => {
                    forget_pick();
                    kept(profile);
                    notify(untrack(|| t!("profile-picture-saved")));
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    };
    let remove = Callback::new(move |()| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            let result = api::call(|c| async move { c.remove_avatar().await }).await;
            busy.try_set(false);
            match result {
                Ok(profile) => {
                    kept(profile);
                    notify(untrack(|| t!("profile-picture-removed")));
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    });
    let save = move |e: SubmitEvent| {
        e.prevent_default();
        let Some(change) = request.get_untracked() else {
            return;
        };
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        done.set(false);
        spawn_local(async move {
            let result = api::call(move |c| async move { c.update_profile(&change).await }).await;
            busy.try_set(false);
            match result {
                Ok(profile) => {
                    name.try_set(profile.display_name.clone().unwrap_or_default());
                    about.try_set(profile.about.clone().unwrap_or_default());
                    kept(profile);
                    done.try_set(true);
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    };

    let user = Signal::derive(move || saved.with(|s| s.name.clone()));
    let display = Signal::derive(move || saved.with(|s| s.display_name.clone()));
    let picture = Signal::derive(move || saved.with(|s| s.avatar_url.clone()));
    let has_picture = Memo::new(move |_| saved.with(|s| s.avatar_url.is_some()));
    let left = move || about.with(|a| chars_left(a, MAX_ABOUT_CHARS));
    let invalid = |bad: bool| if bad { "true" } else { "false" };

    let picture_part = move || {
        match picked.get() {
        Some(p) => view! {
            <div class="picture-pick">
                <CropPicker src=p.url crop=crop />
                <div class="row-end">
                    <button type="button" class="btn text" disabled=move || busy.get() on:click=move |_| forget_pick()>
                        {move || t!("cancel")}
                    </button>
                    <button
                        type="button"
                        class="btn"
                        disabled=move || busy.get() || crop.with(Option::is_none)
                        on:click=use_picture
                    >
                        <Ico icon=Icon::Check />
                        {move || if busy.get() { t!("profile-picture-sending") } else { t!("profile-picture-use") }}
                    </button>
                </div>
            </div>
        }
        .into_any(),
        None => view! {
            <div class="picture-row">
                <Avatar name=user display=display url=picture size=Size::Huge />
                <div class="picture-actions">
                    <button type="button" class="btn tonal" disabled=move || busy.get() on:click=choose>
                        <Ico icon=Icon::Edit />
                        {move || if has_picture.get() { t!("profile-picture-change") } else { t!("profile-picture-add") }}
                    </button>
                    <Show when=move || has_picture.get()>
                        <ConfirmButton
                            icon=Icon::Delete
                            title=Signal::derive(|| t!("profile-picture-remove"))
                            label=Signal::derive(|| t!("profile-picture-remove"))
                            text=Signal::derive(|| t!("profile-picture-remove"))
                            small=true
                            disabled=busy
                            on_confirm=remove
                        />
                    </Show>
                    <small class="muted">{move || t!("profile-picture-hint")}</small>
                </div>
            </div>
        }
        .into_any(),
    }
    };

    view! {
        <div class="profile-form">
            <input
                node_ref=input
                type="file"
                class="visually-hidden"
                tabindex="-1"
                aria-hidden="true"
                accept="image/jpeg,image/png,image/webp,image/gif"
                on:change=pick
            />
            {picture_part}
            {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
            <form on:submit=save on:input=move |_| done.set(false)>
                <label class="field">
                    <span>{move || t!("profile-display-name")}</span>
                    <input
                        maxlength=MAX_DISPLAY_NAME_CHARS
                        autocomplete="nickname"
                        placeholder=move || saved.with(|s| s.name.clone())
                        aria-invalid=move || invalid(name_problem.with(Option::is_some))
                        bind:value=name
                    />
                    {move || name_problem.get().map(|p| view! { <small class="field-error">{p}</small> })}
                    <small class="muted field-hint">{move || t!("profile-display-name-hint")}</small>
                </label>
                <Show when=move || public.get()>
                    <label class="field">
                        <span>{move || t!("profile-about")}</span>
                        <textarea
                            class="profile-about-box"
                            rows="4"
                            maxlength=MAX_ABOUT_CHARS
                            aria-invalid=move || invalid(about_problem.with(Option::is_some))
                            prop:value=move || about.get()
                            on:input=move |e| about.set(event_target_value(&e))
                        ></textarea>
                        {move || about_problem.get().map(|p| view! { <small class="field-error">{p}</small> })}
                        {move || {
                            left()
                                .map(|n| view! { <small class="muted field-hint">{left_words(n)}</small> })
                        }}
                    </label>
                </Show>
                <div class="row-end">
                    <Show when=move || done.get()>
                        <span class="muted saved-note" role="status">
                            <Ico icon=Icon::Check size=18 />
                            {move || t!("admin-saved")}
                        </span>
                    </Show>
                    <button
                        class="btn"
                        type="submit"
                        disabled=move || busy.get() || request.with(Option::is_none)
                    >
                        {move || t!("profile-save")}
                    </button>
                </div>
            </form>
        </div>
    }
}

/// Settings › Account's card for the profile, with the way to see it as
/// others do.
#[component]
pub fn ProfileSettings() -> impl IntoView {
    me::ask();
    let loaded = Memo::new(|_| me::profile().is_some());
    let name = move || {
        store::session()
            .map(|s| s.user.username)
            .unwrap_or_default()
    };
    view! {
        <section class="card" id="profile">
            <div class="card-head">
                <h3 class="card-title">{move || t!("settings-profile")}</h3>
                <a class="btn text small" href=move || route::profile(&name())>
                    {move || t!("account-see-profile")}
                </a>
            </div>
            <Show
                when=move || loaded.get()
                fallback=|| view! { <div class="sentinel"><Spinner /></div> }
            >
                // The form starts from the profile as it was; what it saves
                // comes back to it.
                {move || {
                    me::profile_untracked()
                        .map(|p| view! { <ProfileForm profile=p on_saved=Callback::new(|_| {}) /> })
                }}
            </Show>
        </section>
    }
}
