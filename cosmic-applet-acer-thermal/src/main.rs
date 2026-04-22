// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    Core, Element, app,
    applet::menu_button,
    iced::{self, Length, Rectangle, Subscription, window},
    surface::action::{app_popup, destroy_popup},
    widget::{Space, column, divider, icon, row, text},
};
use serde::Deserialize;
use tokio::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt::init();
    let _ = tracing_log::LogTracer::init();
    tracing::info!("Starting acer thermal applet with version {VERSION}");
    cosmic::applet::run::<ThermalApplet>(())
}

#[derive(Debug, Clone, Deserialize)]
struct BackendState {
    current: String,
    profiles: Vec<Profile>,
}

#[derive(Debug, Clone, Deserialize)]
struct Profile {
    id: String,
    label: String,
    active: bool,
}

#[derive(Debug, Deserialize)]
struct BackendMode {
    mode: String,
}

#[derive(Debug, Deserialize)]
struct WaybarState {
    class: String,
}

struct ThermalApplet {
    core: Core,
    popup: Option<window::Id>,
    state: BackendState,
    pending: bool,
    error: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    CloseRequested(window::Id),
    Refresh,
    Surface(cosmic::surface::Action),
    Refreshed(Result<BackendState, String>),
    SelectProfile(String),
    ProfileSet(Result<(), String>),
}

impl Default for BackendState {
    fn default() -> Self {
        Self {
            current: "normal".to_string(),
            profiles: vec![
                Profile {
                    id: "quiet".to_string(),
                    label: "Quiet".to_string(),
                    active: false,
                },
                Profile {
                    id: "normal".to_string(),
                    label: "Normal".to_string(),
                    active: true,
                },
                Profile {
                    id: "performance".to_string(),
                    label: "Performance".to_string(),
                    active: false,
                },
                Profile {
                    id: "turbo".to_string(),
                    label: "Turbo".to_string(),
                    active: false,
                },
            ],
        }
    }
}

impl ThermalApplet {
    fn current_icon_name(&self) -> &'static str {
        match self.state.current.as_str() {
            "quiet" => "weather-clear-night-symbolic",
            "normal" => "preferences-system-power-symbolic",
            "performance" => "utilities-system-monitor-symbolic",
            "turbo" => "speedometer-symbolic",
            _ => "preferences-system-power-symbolic",
        }
    }

    fn current_label(&self) -> &str {
        self.state
            .profiles
            .iter()
            .find(|profile| profile.id == self.state.current)
            .map(|profile| profile.label.as_str())
            .unwrap_or("Normal")
    }

    fn refresh_task() -> app::Task<Message> {
        iced::Task::perform(fetch_state(), Message::Refreshed).map(cosmic::Action::App)
    }

    fn popup_content(&self) -> Element<'_, cosmic::Action<Message>> {
        let mut content = column![row![
            text::body("Fan profile"),
            Space::new().width(Length::Fill),
            text::caption(self.current_label()),
        ]]
        .padding([8, 12])
        .spacing(8);

        for profile in &self.state.profiles {
            let status = if profile.active { "Current" } else { "" };
            content = content.push(
                menu_button(row![
                    text::body(&profile.label),
                    Space::new().width(Length::Fill),
                    text::caption(status),
                ])
                .on_press_maybe(
                    (!self.pending).then(|| Message::SelectProfile(profile.id.clone())),
                ),
            );
        }

        if self.pending {
            content = content
                .push(divider::horizontal::default())
                .push(text::caption("Applying profile..."));
        }

        if let Some(error) = &self.error {
            content = content
                .push(divider::horizontal::default())
                .push(text::caption(error));
        }

        Element::from(self.core.applet.popup_container(content)).map(cosmic::Action::App)
    }
}

fn state_for_mode(current: &str) -> BackendState {
    let profiles = [
        ("quiet", "Quiet"),
        ("normal", "Normal"),
        ("performance", "Performance"),
        ("turbo", "Turbo"),
    ]
    .into_iter()
    .map(|(id, label)| Profile {
        id: id.to_string(),
        label: label.to_string(),
        active: id == current,
    })
    .collect();

    BackendState {
        current: current.to_string(),
        profiles,
    }
}

impl cosmic::Application for ThermalApplet {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "com.acer.CosmicAppletThermal";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: ()) -> (Self, app::Task<Self::Message>) {
        (
            Self {
                core,
                popup: None,
                state: BackendState::default(),
                pending: false,
                error: None,
            },
            Self::refresh_task(),
        )
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::CloseRequested(id))
    }

    fn subscription(&self) -> Subscription<Message> {
        cosmic::iced::time::every(std::time::Duration::from_secs(5)).map(|_| Message::Refresh)
    }

    fn update(&mut self, message: Message) -> app::Task<Message> {
        match message {
            Message::CloseRequested(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
            Message::Refresh => {
                return Self::refresh_task();
            }
            Message::Surface(action) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(action),
                ));
            }
            Message::Refreshed(result) => match result {
                Ok(state) => {
                    self.state = state;
                    self.pending = false;
                    self.error = None;
                }
                Err(err) => {
                    self.pending = false;
                    self.error = Some(err);
                }
            },
            Message::SelectProfile(profile) => {
                self.pending = true;
                self.error = None;
                self.state.current = profile.clone();
                for existing in &mut self.state.profiles {
                    existing.active = existing.id == profile;
                }
                return iced::Task::perform(set_profile(profile), Message::ProfileSet)
                    .map(cosmic::Action::App);
            }
            Message::ProfileSet(result) => match result {
                Ok(()) => {
                    return Self::refresh_task();
                }
                Err(err) => {
                    self.pending = false;
                    self.error = Some(err);
                }
            },
        }

        app::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let have_popup = self.popup;
        let content = row![icon(
            cosmic::widget::icon::from_name(self.current_icon_name())
                .size(self.core.applet.suggested_size(true).0)
                .symbolic(true)
                .into(),
        )];

        let button = self
            .core
            .applet
            .button_from_element(content, true)
            .on_press_down(Message::CloseRequested(window::Id::NONE))
            .on_press_with_rectangle(move |offset, bounds| {
                if let Some(id) = have_popup {
                    Message::Surface(destroy_popup(id))
                } else {
                    let _ = offset;
                    Message::Surface(app_popup::<ThermalApplet>(
                        move |state: &mut ThermalApplet| {
                            let popup_id = window::Id::unique();
                            state.popup = Some(popup_id);

                            let mut popup_settings = state.core.applet.get_popup_settings(
                                state.core.main_window_id().unwrap(),
                                popup_id,
                                None,
                                None,
                                None,
                            );

                            popup_settings.positioner.anchor_rect = Rectangle {
                                x: (bounds.x - offset.x) as i32,
                                y: (bounds.y - offset.y) as i32,
                                width: bounds.width as i32,
                                height: bounds.height as i32,
                            };

                            popup_settings
                        },
                        Some(Box::new(|state: &ThermalApplet| state.popup_content())),
                    ))
                }
            });

        Element::from(self.core.applet.applet_tooltip::<Message>(
            button,
            format!("Thermal profile: {}", self.current_label()),
            self.popup.is_some(),
            Message::Surface,
            None,
        ))
    }

    fn view_window(&self, _id: window::Id) -> Element<'_, Message> {
        self.popup_content().map(|action| match action {
            cosmic::Action::App(message) => message,
            cosmic::Action::Cosmic(_) => Message::CloseRequested(window::Id::NONE),
            cosmic::Action::None => Message::CloseRequested(window::Id::NONE),
        })
    }
}

async fn fetch_state() -> Result<BackendState, String> {
    let command = backend_command();
    let output = Command::new(&command)
        .arg("list")
        .arg("--json")
        .output()
        .await
        .map_err(|err| format!("Failed to run thermal backend: {err}"))?;

    if !output.status.success() {
        return fetch_legacy_state(&command).await;
    }

    serde_json::from_slice::<BackendState>(&output.stdout)
        .map_err(|err| format!("Invalid thermal backend JSON: {err}"))
}

async fn fetch_legacy_state(command: &str) -> Result<BackendState, String> {
    let get_output = Command::new(command)
        .arg("get")
        .output()
        .await
        .map_err(|err| format!("Failed to run thermal backend: {err}"))?;

    if get_output.status.success() {
        let mode = serde_json::from_slice::<BackendMode>(&get_output.stdout)
            .map_err(|err| format!("Invalid thermal backend JSON: {err}"))?;
        return Ok(state_for_mode(&mode.mode));
    }

    let waybar_output = Command::new(command)
        .arg("get-json")
        .output()
        .await
        .map_err(|err| format!("Failed to run thermal backend: {err}"))?;

    if waybar_output.status.success() {
        let state = serde_json::from_slice::<WaybarState>(&waybar_output.stdout)
            .map_err(|err| format!("Invalid Waybar JSON: {err}"))?;
        return Ok(state_for_mode(&state.class));
    }

    Err(format!(
        "Thermal backend does not support a readable state interface: get failed with {}, get-json failed with {}",
        output_status(&get_output.status),
        output_status(&waybar_output.status),
    ))
}

async fn set_profile(profile: String) -> Result<(), String> {
    let output = Command::new(backend_command())
        .arg("set")
        .arg(&profile)
        .output()
        .await
        .map_err(|err| format!("Failed to run thermal backend: {err}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Thermal backend exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn backend_command() -> String {
    std::env::var("ACER_THERMAL_CONTROL_CMD").unwrap_or_else(|_| "thermal-control.sh".to_string())
}

fn output_status(status: &std::process::ExitStatus) -> String {
    status
        .code()
        .map(|code| code.to_string())
        .unwrap_or_else(|| "signal".to_string())
}
