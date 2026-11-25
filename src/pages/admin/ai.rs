//! Administration › Server › AI: for each tool, off, StarBooru's own models
//! or an API. The server looks after the models; this card says what's
//! wanted, and looks again every few seconds while a download runs.

use std::future::Future;

use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_client::{Client, ClientError};
use starbooru_core::ai::MAX_AI_FIELD_CHARS;
use starbooru_core::{
    AiAdminStatus, AiMode, AiPreset, AiTool, AiToolSettings, AiToolState, AiToolStatus,
};

use super::Waiting;
use crate::components::banner::ErrorBanner;
use crate::components::confirm::ConfirmButton;
use crate::components::icons::Icon;
use crate::logic::admin::{
    ApiFormError, BUSY_POLL_MS, IDLE_ROUNDS, ModeChoice, ModelState, api_form_error_text,
    api_request, busy, choose_mode, clear_key_request, duration, effective_preset, hardware_line,
    model_request, model_state, model_time, percent, preset_request, sentence, tools_in_order,
    translate_request, wanted_model,
};
use crate::logic::viewer::human_size;
use crate::{api, dom};

type Status = RwSignal<Option<AiAdminStatus>>;

fn tool_label(tool: AiTool) -> String {
    match tool {
        AiTool::Tagging => t!("admin-ai-tagging"),
        AiTool::People => t!("admin-ai-people"),
        AiTool::Translation => t!("admin-ai-translation"),
        AiTool::Comics => t!("admin-ai-comics"),
    }
}

/// What a tool does, in a line under its name.
fn tool_about(tool: AiTool) -> String {
    match tool {
        AiTool::Tagging => t!("admin-ai-tagging-about"),
        AiTool::People => t!("admin-ai-people-about"),
        AiTool::Translation => t!("admin-ai-translation-about"),
        AiTool::Comics => t!("admin-ai-comics-about"),
    }
}

fn mode_label(mode: AiMode) -> String {
    match mode {
        AiMode::Off => t!("admin-ai-mode-off"),
        AiMode::Local => t!("admin-ai-mode-local"),
        AiMode::Api => t!("admin-ai-mode-api"),
    }
}

fn preset_label(preset: AiPreset) -> String {
    match preset {
        AiPreset::Fast => t!("admin-ai-preset-fast"),
        AiPreset::Balanced => t!("admin-ai-preset-balanced"),
        AiPreset::Accurate => t!("admin-ai-preset-accurate"),
    }
}

/// One tool's change in flight: `working` while it goes, what the server says
/// afterwards in `status`, and a refusal in `problem`. `generation` is the
/// card's, bumped when a change goes and when its answer comes, so only the
/// newest answer, a look's or a change's, is shown.
#[derive(Clone, Copy)]
struct Changer {
    status: Status,
    generation: StoredValue<u64>,
    problem: RwSignal<Option<String>>,
    working: RwSignal<bool>,
}

impl Changer {
    fn run<F, Fut>(self, change: F)
    where
        F: FnOnce(Client) -> Fut + 'static,
        Fut: Future<Output = Result<AiAdminStatus, ClientError>> + 'static,
    {
        self.run_or(change, || {});
    }

    /// As `run`, with `undo` putting back what the page changed ahead of the
    /// server when the server refuses.
    fn run_or<F, Fut>(self, change: F, undo: impl FnOnce() + 'static)
    where
        F: FnOnce(Client) -> Fut + 'static,
        Fut: Future<Output = Result<AiAdminStatus, ClientError>> + 'static,
    {
        if self.working.get_untracked() {
            undo();
            return;
        }
        self.working.set(true);
        self.problem.set(None);
        let at = self.generation.get_value().wrapping_add(1);
        self.generation.set_value(at);
        spawn_local(async move {
            match api::call(change).await {
                Ok(s) => {
                    if self.generation.try_get_value() == Some(at) {
                        self.generation.set_value(at.wrapping_add(1));
                        self.status.try_set(Some(s));
                    }
                    // The viewer and the upload page offer the tools anew.
                    api::forget_server_answers();
                }
                Err(message) => {
                    self.problem.try_set(Some(message));
                    undo();
                }
            }
            self.working.try_set(false);
        });
    }
}

#[component]
pub(super) fn AiCard() -> impl IntoView {
    let status: Status = RwSignal::new(None);
    let error = RwSignal::new(None::<String>);
    let generation = StoredValue::new(0_u64);
    // Polls while the card is on the page: often during a download or
    // start-up, otherwise every 15 s. A look sent before a change and
    // answered after it is dropped, as it doesn't know of the change.
    spawn_local(async move {
        loop {
            let Some(at) = generation.try_get_value() else {
                return;
            };
            match api::call(|c| async move { c.admin_ai().await }).await {
                Ok(found) => {
                    if generation.try_get_value() == Some(at) {
                        if status.try_set(Some(found)).is_some() {
                            return;
                        }
                        error.try_set(None);
                    }
                }
                Err(message) => {
                    // A later look may work; what's on screen stays.
                    if status.try_with_untracked(Option::is_none) == Some(true) {
                        error.try_set(Some(message));
                    }
                }
            }
            // Short steps, so a download started here is followed at once.
            for _ in 0..IDLE_ROUNDS {
                dom::sleep(BUSY_POLL_MS).await;
                let failing = error.try_with_untracked(Option::is_some);
                let moving = status.try_with_untracked(|s| s.as_ref().is_some_and(busy));
                match (failing, moving) {
                    (Some(false), Some(true)) => break,
                    (Some(_), Some(_)) => {}
                    _ => return,
                }
            }
        }
    });
    let shown = move || status.with(Option::is_some);
    let machine = move || status.with(|s| s.as_ref().map(|s| hardware_line(&s.hardware)));
    let reason =
        Memo::new(move |_| status.with(|s| s.as_ref().and_then(|s| s.runtime.reason.clone())));
    let ready = Memo::new(move |_| status.with(|s| s.as_ref().is_some_and(|s| s.runtime.ready)));
    let tools = Memo::new(move |_| {
        status.with(|s| {
            tools_in_order(
                s.iter()
                    .flat_map(|s| s.tools.iter().map(|t| t.tool))
                    .collect(),
            )
        })
    });
    let waiting = move || error.with(Option::is_none).then(|| view! { <Waiting /> });
    let runtime = move || {
        reason.get().map(|why| {
            view! {
                <div class="banner warn ai-runtime" role="status">
                    <strong>{move || t!("admin-ai-unavailable")}</strong>
                    <span class="break">{sentence(&why)}</span>
                    <span>{move || t!("admin-ai-unavailable-api")}</span>
                </div>
            }
        })
    };

    view! {
        <section class="card">
            <h2 class="card-title">{move || t!("admin-ai-title")}</h2>
            {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
            <Show when=shown fallback=waiting>
                <p class="muted hint ai-machine">{machine}</p>
                {runtime}
                <For
                    each=move || tools.get()
                    key=|tool| *tool
                    children=move |tool| {
                        view! { <ToolBlock tool=tool status=status generation=generation ready=ready /> }
                    }
                />
            </Show>
        </section>
    }
}

#[component]
fn ToolBlock(
    tool: AiTool,
    status: Status,
    generation: StoredValue<u64>,
    ready: Memo<bool>,
) -> impl IntoView {
    let info = Memo::new(move |_| {
        status.with(|s| {
            s.iter()
                .flat_map(|s| s.tools.iter())
                .find(|t| t.tool == tool)
                .cloned()
                .unwrap_or_default()
        })
    });
    let settings = Memo::new(move |_| info.with(|t| t.settings.clone()));
    let autotag = Memo::new(move |_| status.with(|s| s.as_ref().is_some_and(|s| s.autotag)));
    let auto_translate = Memo::new(move |_| {
        status.with(|s| {
            s.as_ref()
                .map(|s| s.auto_translate.clone())
                .unwrap_or_default()
        })
    });
    let changer = Changer {
        status,
        generation,
        problem: RwSignal::new(None),
        working: RwSignal::new(false),
    };
    // The API form opens before anything is saved, since the API needs an
    // address first. What's typed survives the polling.
    let api_open = RwSignal::new(false);
    let shown = Memo::new(move |_| {
        if api_open.get() {
            AiMode::Api
        } else {
            settings.with(|s| s.mode)
        }
    });
    let url = RwSignal::new(String::new());
    let model = RwSignal::new(String::new());
    Effect::watch(
        move || settings.with(|s| (s.api_url.clone(), s.api_model.clone())),
        move |(saved_url, saved_model), _, _| {
            url.set(saved_url.clone());
            model.set(saved_model.clone());
        },
        true,
    );

    let pick = move |mode: AiMode| match choose_mode(&settings.get_untracked(), mode) {
        ModeChoice::OpenForm => api_open.set(true),
        ModeChoice::Send(update) => {
            api_open.set(false);
            if settings.with_untracked(|s| s.mode) != mode {
                changer.run(move |c| async move { c.set_ai_tool(tool, &update).await });
            }
        }
    };
    let modes = [AiMode::Off, AiMode::Local, AiMode::Api]
        .into_iter()
        .map(|mode| {
            let on = move || shown.get() == mode;
            // Local models need the runtime; the banner above says why not.
            let blocked = move || mode == AiMode::Local && !ready.get();
            view! {
                <button
                    type="button"
                    role="radio"
                    class="seg"
                    class:on=on
                    aria-checked=move || if on() { "true" } else { "false" }
                    disabled=move || changer.working.get() || blocked()
                    title=move || blocked().then(|| t!("admin-ai-unavailable"))
                    on:click=move |_| pick(mode)
                >
                    {move || mode_label(mode)}
                </button>
            }
        })
        .collect_view();
    let state = move || {
        let (state, progress) = info.with(|t| (t.state, wanted_model(t).and_then(|m| m.progress)));
        match state {
            AiToolState::Off => None,
            AiToolState::Ready => Some(view! {
                <span class="pill ok">{move || t!("admin-ai-state-ready")}</span>
            }.into_any()),
            AiToolState::Installing => Some(view! {
                <span class="pill busy" role="status">
                    {move || match progress {
                        Some(p) => t!("admin-ai-state-downloading", percent = percent(p.done, p.total)),
                        None => t!("admin-ai-state-preparing"),
                    }}
                </span>
            }.into_any()),
            AiToolState::Failed => Some(view! {
                <span class="pill bad">{move || t!("admin-ai-state-failed")}</span>
            }.into_any()),
        }
    };
    // The box ticks before the server answers; a refusal unticks it.
    let set_autotag = move |box_: web_sys::HtmlInputElement| {
        let on = box_.checked();
        let update = starbooru_core::AiGeneralUpdate {
            autotag: on,
            auto_translate: auto_translate.get_untracked(),
        };
        changer.run_or(
            move |c| async move { c.set_ai_general(&update).await },
            move || box_.set_checked(!on),
        );
    };
    let set_language = move |typed: String| match translate_request(autotag.get_untracked(), &typed)
    {
        Some(update) => changer.run(move |c| async move { c.set_ai_general(&update).await }),
        None => changer
            .problem
            .set(Some(untrack(|| t!("admin-ai-translate-invalid")))),
    };

    view! {
        <div class="ai-tool">
            <div class="ai-tool-head">
                <div class="ai-tool-name">
                    <h3>{move || tool_label(tool)}</h3>
                    <p class="muted">{move || tool_about(tool)}</p>
                </div>
                {state}
            </div>
            <div class="segmented" role="radiogroup" aria-label=move || tool_label(tool)>
                {modes}
            </div>
            {move || changer.problem.get().map(|message| view! { <ErrorBanner message=message /> })}
            <Show when=move || shown.get() == AiMode::Local>
                <LocalSetup tool=tool info=info settings=settings changer=changer />
            </Show>
            <Show when=move || shown.get() == AiMode::Api>
                <ApiSetup tool=tool settings=settings url=url model=model changer=changer />
            </Show>
            <Show when=move || tool == AiTool::Tagging && shown.get() != AiMode::Off>
                <label class="check">
                    <input
                        type="checkbox"
                        prop:checked=move || autotag.get()
                        disabled=move || changer.working.get()
                        on:change=move |e| set_autotag(event_target::<web_sys::HtmlInputElement>(&e))
                    />
                    <span>{move || t!("admin-ai-autotag")}</span>
                </label>
            </Show>
            <Show when=move || tool == AiTool::Comics && settings.with(|s| s.mode != AiMode::Off)>
                <label class="field">
                    <span>{move || t!("admin-ai-auto-translate")}</span>
                    <input
                        maxlength="35"
                        placeholder="en"
                        autocomplete="off"
                        spellcheck="false"
                        prop:value=move || auto_translate.get()
                        on:change=move |e| set_language(event_target_value(&e))
                    />
                    <small class="muted field-hint">{move || t!("admin-ai-auto-translate-hint")}</small>
                </label>
            </Show>
        </div>
    }
}

/// StarBooru's own models: a speed, the model it comes to, and the whole
/// catalogue to install from or remove.
#[component]
fn LocalSetup(
    tool: AiTool,
    info: Memo<AiToolStatus>,
    settings: Memo<AiToolSettings>,
    changer: Changer,
) -> impl IntoView {
    // Each button stays as the panel is read again, so the focus stays on it.
    let preset_list =
        Memo::new(move |_| info.with(|t| t.presets.iter().map(|p| p.preset).collect::<Vec<_>>()));
    let preset_button = move |choice: AiPreset| {
        let on = Memo::new(move |_| {
            info.with(|t| t.settings.model.is_empty() && effective_preset(t) == choice)
        });
        let available = Memo::new(move |_| {
            info.with(|t| t.presets.iter().any(|p| p.preset == choice && p.available))
        });
        let budget = choice.budget_ms() / 1000;
        view! {
            <button
                type="button"
                role="radio"
                class="seg"
                class:on=move || on.get()
                aria-checked=move || if on.get() { "true" } else { "false" }
                disabled=move || !available.get() || changer.working.get()
                on:click=move |_| {
                    let update = preset_request(&settings.get_untracked(), choice);
                    changer.run(move |c| async move { c.set_ai_tool(tool, &update).await });
                }
            >
                {move || preset_label(choice)}
                <small>{format!("{budget} s")}</small>
            </button>
        }
    };
    let unavailable = move || {
        let names: Vec<String> = info.with(|t| {
            t.presets
                .iter()
                .filter(|p| !p.available)
                .map(|p| preset_label(p.preset))
                .collect()
        });
        (!names.is_empty()).then(|| {
            view! {
                <p class="muted ai-note">{t!("admin-ai-presets-off", presets = names.join(", "))}</p>
            }
        })
    };
    let wanted = move || {
        info.with(|t| wanted_model(t).map(|m| format!("{} · {}", m.name, model_time(m))))
            .map(|line| view! { <p class="ai-note">{line}</p> })
    };
    let failure = move || {
        let (reason, retry) = info.with(|t| {
            let retry = wanted_model(t)
                .filter(|m| m.error.is_some())
                .map(|m| m.id.clone());
            (t.error.as_deref().map(sentence), retry)
        });
        reason.map(|reason| {
            let retry = retry.map(|id| {
                view! {
                    <button
                        class="btn text"
                        type="button"
                        on:click=move |_| {
                            let id = id.clone();
                            changer.run(move |c| async move { c.install_ai_model(&id).await });
                        }
                    >
                        {move || t!("retry")}
                    </button>
                }
            });
            view! {
                <div class="banner error" role="alert">
                    <span class="break">{reason}</span>
                    {retry}
                </div>
            }
        })
    };
    let model_ids = Memo::new(move |_| {
        info.with(|t| t.models.iter().map(|m| m.id.clone()).collect::<Vec<_>>())
    });

    view! {
        <div class="segmented" role="radiogroup" aria-label=move || t!("admin-ai-speed")>
            <For each=move || preset_list.get() key=|p| p.as_str() children=preset_button />
        </div>
        {unavailable}
        {wanted}
        {failure}
        // Stays put while the rows inside follow the downloads, so it doesn't
        // close by itself.
        <details class="ai-models">
            <summary>{move || t!("admin-ai-models")}</summary>
            <For
                each=move || model_ids.get()
                key=|id| id.clone()
                children=move |id| {
                    view! { <ModelRow tool=tool id=id info=info settings=settings changer=changer /> }
                }
            />
        </details>
    }
}

/// One model of the catalogue. The row stays as the panel is read again, and
/// its buttons are built anew only when what it offers changes, so the focus
/// stays where it was.
#[component]
fn ModelRow(
    tool: AiTool,
    id: String,
    info: Memo<AiToolStatus>,
    settings: Memo<AiToolSettings>,
    changer: Changer,
) -> impl IntoView {
    let model = Memo::new({
        let id = id.clone();
        move |_| {
            info.with(|t| t.models.iter().find(|m| m.id == id).cloned())
                .unwrap_or_default()
        }
    });
    let state = Memo::new({
        let id = id.clone();
        move |_| {
            let in_use = info.with(|t| t.active.as_deref() == Some(id.as_str()));
            model.with(|m| model_state(m, in_use))
        }
    });
    let name = Memo::new(move |_| model.with(|m| m.name.clone()));
    let done =
        Memo::new(move |_| model.with(|m| m.progress.map_or(0, |p| percent(p.done, p.total))));
    let details = move || {
        model.with(|m| {
            let fit = if m.fits {
                model_time(m)
            } else {
                t!("admin-ai-no-memory")
            };
            format!("{} · {fit}", human_size(m.size))
        })
    };
    let install = {
        let id = id.clone();
        move || {
            let id = id.clone();
            changer.run(move |c| async move { c.install_ai_model(&id).await });
        }
    };
    let uninstall = {
        let id = id.clone();
        move || {
            let id = id.clone();
            changer.run(move |c| async move { c.uninstall_ai_model(&id).await });
        }
    };
    let use_it = {
        let id = id.clone();
        move || {
            let update = model_request(&settings.get_untracked(), &id);
            changer.run(move |c| async move { c.set_ai_tool(tool, &update).await });
        }
    };
    let actions = move || match state.get() {
        ModelState::Downloading => view! {
            <span
                class="meter ai-meter"
                role="progressbar"
                aria-label=move || t!("admin-ai-downloading-named", name = name.get())
                aria-valuenow=move || done.get()
                aria-valuemin="0"
                aria-valuemax="100"
            >
                <span style=move || format!("width: {}%", done.get())></span>
            </span>
            <span class="muted" aria-hidden="true">{move || format!("{}%", done.get())}</span>
        }
        .into_any(),
        ModelState::InUse => {
            view! { <span class="pill ok">{move || t!("admin-ai-in-use")}</span> }.into_any()
        }
        ModelState::Installed => {
            let (use_it, uninstall) = (use_it.clone(), uninstall.clone());
            view! {
                <button class="btn tonal small" type="button" on:click=move |_| use_it()>
                    {move || t!("admin-ai-use")}
                </button>
                <ConfirmButton
                    icon=Icon::Delete
                    title=Signal::derive(|| t!("admin-ai-uninstall"))
                    label=Signal::derive(move || t!("admin-ai-uninstall-named", name = name.get()))
                    text=Signal::derive(|| t!("admin-ai-uninstall"))
                    small=true
                    disabled=changer.working
                    on_confirm=Callback::new(move |()| uninstall())
                />
            }
            .into_any()
        }
        ModelState::Installable { failed } => {
            let install = install.clone();
            view! {
                <button class="btn tonal small" type="button" on:click=move |_| install()>
                    {move || if failed { t!("retry") } else { t!("admin-ai-install") }}
                </button>
            }
            .into_any()
        }
        ModelState::TooBig => ().into_any(),
    };
    let accurate = move || model.with(|m| m.high_accuracy);
    let gpu = move || model.with(|m| m.gpu);
    let error = move || model.with(|m| m.error.as_deref().map(sentence));
    view! {
        <div class="ai-model">
            <div class="ai-model-text">
                <strong>{move || name.get()}</strong>
                <span class="muted">{details}</span>
                <Show when=accurate>
                    <span class="pill">{move || t!("admin-ai-high-accuracy")}</span>
                </Show>
                <Show when=gpu>
                    <span class="pill">{move || t!("admin-ai-gpu")}</span>
                </Show>
                {move || error().map(|reason| view! { <span class="ai-model-error">{reason}</span> })}
            </div>
            <div class="ai-model-actions">{actions}</div>
        </div>
    }
}

/// An OpenAI-compatible API: its address, the model and a key, with a test
/// once it's saved.
#[component]
fn ApiSetup(
    tool: AiTool,
    settings: Memo<AiToolSettings>,
    url: RwSignal<String>,
    model: RwSignal<String>,
    changer: Changer,
) -> impl IntoView {
    let key = RwSignal::new(String::new());
    let form_error = RwSignal::new(None::<ApiFormError>);
    let tested = RwSignal::new(None::<u32>);
    let test_error = RwSignal::new(None::<String>);
    let testing = RwSignal::new(false);
    let key_set = Memo::new(move |_| settings.with(|s| s.api_key_set));
    let saved = Memo::new(move |_| settings.with(|s| s.mode == AiMode::Api));

    let save = move |e: SubmitEvent| {
        e.prevent_default();
        let request = api_request(
            tool,
            &settings.get_untracked(),
            &url.get_untracked(),
            &model.get_untracked(),
            &key.get_untracked(),
        );
        match request {
            Err(why) => form_error.set(Some(why)),
            Ok(update) => {
                form_error.set(None);
                key.set(String::new());
                tested.set(None);
                test_error.set(None);
                changer.run(move |c| async move { c.set_ai_tool(tool, &update).await });
            }
        }
    };
    let clear_key = move |_| {
        let update = clear_key_request(&settings.get_untracked());
        changer.run(move |c| async move { c.set_ai_tool(tool, &update).await });
    };
    let test = move |_| {
        if testing.get_untracked() {
            return;
        }
        testing.set(true);
        tested.set(None);
        test_error.set(None);
        spawn_local(async move {
            match api::attempt(move |c| async move { c.test_ai_tool(tool).await }).await {
                Ok(r) => {
                    tested.try_set(Some(r.ms));
                }
                // The provider failed, in the server's words.
                Err(e) if e.status == Some(424) => {
                    let reason = e.message.trim_end_matches('.').to_owned();
                    test_error.try_set(Some(untrack(|| {
                        t!("admin-ai-test-failed", reason = reason)
                    })));
                }
                Err(e) => {
                    test_error.try_set(Some(e.message));
                }
            }
            testing.try_set(false);
        });
    };

    view! {
        // The app checks the address itself, in words.
        <form class="ai-api" novalidate on:submit=save on:input=move |_| form_error.set(None)>
            <label class="field">
                <span>{move || t!("admin-ai-url")}</span>
                <input
                    type="url"
                    inputmode="url"
                    autocomplete="off"
                    spellcheck="false"
                    placeholder="https://api.openai.com/v1"
                    maxlength=MAX_AI_FIELD_CHARS
                    aria-invalid=move || {
                        let bad = matches!(
                            form_error.get(),
                            Some(ApiFormError::NoAddress | ApiFormError::BadAddress)
                        );
                        if bad { "true" } else { "false" }
                    }
                    bind:value=url
                />
            </label>
            <label class="field">
                <span>{move || t!("admin-ai-model")}</span>
                <input
                    autocomplete="off"
                    spellcheck="false"
                    maxlength=MAX_AI_FIELD_CHARS
                    aria-invalid=move || {
                        if form_error.get() == Some(ApiFormError::NoModel) { "true" } else { "false" }
                    }
                    bind:value=model
                />
            </label>
            <label class="field">
                <span>{move || t!("admin-ai-key")}</span>
                // The API's key: the browser mustn't fill in the StarBooru
                // password it keeps.
                <input
                    type="password"
                    name="ai-api-key"
                    autocomplete="new-password"
                    maxlength=MAX_AI_FIELD_CHARS
                    placeholder=move || {
                        if key_set.get() { t!("admin-ai-key-set") } else { t!("admin-ai-key-none") }
                    }
                    bind:value=key
                />
            </label>
            {move || form_error.get().map(|why| view! { <ErrorBanner message=api_form_error_text(why) /> })}
            {move || test_error.get().map(|message| view! { <ErrorBanner message=message /> })}
            <div class="ai-api-actions">
                <button class="btn" type="submit" disabled=move || changer.working.get()>
                    {move || t!("save")}
                </button>
                <Show when=move || saved.get()>
                    <button class="btn tonal" type="button" disabled=move || testing.get() on:click=test>
                        {move || if testing.get() { t!("admin-ai-testing") } else { t!("admin-ai-test") }}
                    </button>
                </Show>
                <Show when=move || key_set.get()>
                    <button
                        class="btn text"
                        type="button"
                        disabled=move || changer.working.get()
                        on:click=clear_key
                    >
                        {move || t!("admin-ai-key-clear")}
                    </button>
                </Show>
                {move || {
                    tested
                        .get()
                        .map(|ms| {
                            view! {
                                <span class="muted" role="status">
                                    {move || t!("admin-ai-test-ok", time = duration(ms))}
                                </span>
                            }
                        })
                }}
            </div>
        </form>
    }
}
