#![forbid(unsafe_code)]
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod account;
mod desktop;
mod game;
mod look;
mod servers;
mod strings;
mod ui;

use game::{Config, Session, StartError};
use iced::{Event, Subscription, Task, Theme, event, keyboard, widget::image, window};
use kr_tunnel::launch_tls;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    fs,
    net::TcpStream,
    path::Path,
    time::{Duration, Instant},
};
use strings::{Copy, Language};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Mode {
    #[default]
    #[serde(alias = "online")]
    Servers,
    OfflineFriends,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Screen {
    Settings,
    Help,
    Menu,
    Register,
    Change,
    Delete,
    Recover,
    AccountResult,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Server {
    Checking,
    Ready(u32),
    Offline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Files {
    Checking,
    Ready,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayState {
    Soon,
    Checking,
    Starting,
    Running,
    Error,
    WaitingForServer,
    Ready,
}

#[derive(Clone)]
enum Message {
    Language(Language),
    Mode(Mode),
    Select(servers::Id),
    Open(Screen),
    Dismiss,
    Tab(bool),
    Login(String),
    DisplayName(String),
    Password(Zeroizing<String>),
    Current(Zeroizing<String>),
    Code(Zeroizing<String>),
    Submit,
    AccountDone(Result<Option<Zeroizing<String>>, ()>),
    CopyCode,
    Tick,
    Resize(iced::Size),
    Status(servers::Id, Server),
    CheckFiles(bool),
    Retry,
    RepairFromHome,
    Prepared(Result<bool, ()>, bool),
    Play,
    Started(Result<Session, StartError>),
    Desktop(desktop::Action),
    DesktopDone(desktop::Action, bool),
    Close,
}

#[derive(Default)]
struct Form {
    login: String,
    display_name: String,
    password: Zeroizing<String>,
    current: Zeroizing<String>,
    code: Zeroizing<String>,
}

impl Form {
    fn error(&self, screen: Screen) -> Option<Copy> {
        let login = self.login.trim();
        let password = |value: &str| {
            (12..=128).contains(&value.len())
                && value.bytes().all(|b| b == b' ' || b.is_ascii_graphic())
        };
        if !(3..=32).contains(&login.len())
            || !login
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Some(Copy::LoginHint);
        }
        if screen == Screen::Register && self.display_name.trim().is_empty() {
            return Some(Copy::NameHint);
        }
        if matches!(screen, Screen::Change | Screen::Delete) && !password(&self.current) {
            return Some(Copy::PasswordHint);
        }
        if screen == Screen::Recover {
            let code = self.code.trim();
            if code.len() != 64 || !code.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Some(Copy::CodeHint);
            }
        }
        if screen != Screen::Delete && !password(&self.password) {
            return Some(Copy::PasswordHint);
        }
        None
    }
}

struct Player {
    config: Option<Config>,
    art: Option<image::Handle>,
    language: Language,
    mode: Mode,
    screen: Option<Screen>,
    form: Form,
    submitting: bool,
    recovery: Option<Zeroizing<String>>,
    catalog: Vec<servers::Entry>,
    server_prefs: servers::Preferences,
    server: Server,
    probing: bool,
    files: Files,
    starting: bool,
    session: Option<Session>,
    closing: bool,
    launch_error: Option<Copy>,
    preference_error: bool,
    modal_notice: Option<(Copy, bool)>,
    tick: usize,
    size: iced::Size,
    paint: u8,
}

fn play_state(
    mode: Mode,
    files: Files,
    server: Server,
    starting: bool,
    running: bool,
    failure: bool,
) -> PlayState {
    if mode == Mode::OfflineFriends {
        PlayState::Soon
    } else if running {
        PlayState::Running
    } else if starting {
        PlayState::Starting
    } else if files == Files::Checking {
        PlayState::Checking
    } else if files == Files::Error || failure {
        PlayState::Error
    } else if matches!(server, Server::Ready(_)) {
        PlayState::Ready
    } else {
        PlayState::WaitingForServer
    }
}

fn preference<T: Default + DeserializeOwned>(state: Option<&Path>, file: &str) -> T {
    state
        .and_then(|p| fs::read(p.join(file)).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn save_preference(state: Option<&Path>, file: &str, value: impl Serialize) -> bool {
    state.is_none_or(|p| {
        serde_json::to_vec(&value).is_ok_and(|bytes| fs::write(p.join(file), bytes).is_ok())
    })
}

const ACCOUNT_LOGIN: &str = "account-login";
const WINDOW_SIZE: iced::Size = iced::Size::new(1100.0, 820.0);
const MINIMUM: iced::Size = iced::Size::new(880.0, 660.0);

fn fitted_window(monitor: iced::Size) -> iced::Size {
    let scale = (monitor.width / WINDOW_SIZE.width)
        .min((monitor.height - 72.0) / WINDOW_SIZE.height)
        .clamp(0.4, 1.0);
    iced::Size::new(
        (WINDOW_SIZE.width * scale).round(),
        (WINDOW_SIZE.height * scale).round(),
    )
}

fn fit_monitor() -> Task<Message> {
    window::latest().and_then(|id| {
        window::monitor_size(id).and_then(move |monitor| {
            let size = fitted_window(monitor);
            Task::batch([
                window::set_min_size(
                    id,
                    Some(iced::Size::new(
                        MINIMUM.width.min(size.width),
                        MINIMUM.height.min(size.height),
                    )),
                ),
                window::resize(id, size),
            ])
        })
    })
}

fn probe(config: &Config) -> Server {
    let start = Instant::now();
    let result = (|| -> std::io::Result<bool> {
        let timeout = Duration::from_secs(2);
        let socket = TcpStream::connect_timeout(&config.server.into(), timeout)?;
        let mut stream = launch_tls::connect(socket, fs::read(&config.certificate)?, timeout)?;
        launch_tls::write_frame(
            &mut stream,
            b"{\"op\":\"status\"}\n",
            launch_tls::REQUEST_LIMIT,
        )?;
        let reply: serde_json::Value =
            serde_json::from_slice(&launch_tls::read_frame(&mut stream, 256)?)
                .map_err(std::io::Error::other)?;
        Ok(reply["ok"] == true && reply["ready"] == true)
    })();
    if matches!(result, Ok(true)) {
        Server::Ready(u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX))
    } else {
        Server::Offline
    }
}

impl Player {
    fn boot() -> (Self, Task<Message>) {
        let preview = std::env::args_os()
            .nth(1)
            .is_some_and(|arg| arg == "--preview");
        let config = if preview { None } else { Config::load().ok() };
        let state = config.as_ref().map(|c| c.state.as_path());
        let language = preference(state, "language.json");
        let mode = preference(state, "mode.json");
        let catalog: Vec<_> = config
            .as_ref()
            .and_then(|cfg| servers::Entry::bundled(cfg).ok())
            .into_iter()
            .collect();
        let mut server_prefs: servers::Preferences = preference(state, "servers.json");
        if !catalog
            .iter()
            .any(|entry| Some(&entry.id) == server_prefs.selected.as_ref())
        {
            server_prefs.selected = catalog.first().map(|entry| entry.id.clone());
        }
        let art = look::decode_art(config.as_ref().and_then(|c| {
            kr_player::launcher_art(&c.bundle, &c.manifest_sha256)
                .ok()
                .flatten()
        }));
        let mut player = Self {
            config,
            art,
            language,
            mode,
            screen: None,
            form: Form::default(),
            submitting: false,
            recovery: None,
            catalog,
            server_prefs,
            server: Server::Checking,
            probing: false,
            files: Files::Error,
            starting: false,
            session: None,
            closing: false,
            launch_error: None,
            preference_error: false,
            modal_notice: None,
            tick: 0,
            size: WINDOW_SIZE,
            paint: 0,
        };
        let check = player.check_files(false);
        let status = player.check_server();
        (player, Task::batch([check, status, fit_monitor()]))
    }

    fn play_state(&self) -> PlayState {
        play_state(
            self.mode,
            self.files,
            self.server,
            self.starting,
            self.session.is_some(),
            self.launch_error.is_some(),
        )
    }

    fn selected(&self) -> Option<&servers::Entry> {
        let selected = self.server_prefs.selected.as_ref()?;
        self.catalog.iter().find(|entry| &entry.id == selected)
    }

    fn selected_config(&self) -> Option<Config> {
        Some(self.config.clone()?.with_route(&self.selected()?.route))
    }

    fn save_servers(&mut self) {
        self.preference_error = !save_preference(
            self.config.as_ref().map(|c| c.state.as_path()),
            "servers.json",
            &self.server_prefs,
        );
    }

    fn check_server(&mut self) -> Task<Message> {
        if !self.probing
            && let Some((c, id)) = self
                .selected_config()
                .zip(self.server_prefs.selected.clone())
        {
            self.probing = true;
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || probe(&c))
                        .await
                        .unwrap_or(Server::Offline)
                },
                move |status| Message::Status(id.clone(), status),
            );
        }
        if self.selected().is_none() {
            self.server = Server::Offline;
        }
        Task::none()
    }

    fn check_files(&mut self, repair: bool) -> Task<Message> {
        if self.files == Files::Checking || self.starting || self.session.is_some() {
            return Task::none();
        }
        let Some(c) = self.config.clone() else {
            return Task::none();
        };
        self.files = Files::Checking;
        self.launch_error = None;
        self.modal_notice = None;
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    kr_player::maintain_install(
                        &c.bundle,
                        &c.manifest_sha256,
                        &c.installation,
                        repair,
                    )
                    .map_err(|_| ())
                })
                .await
                .unwrap_or(Err(()))
            },
            move |result| Message::Prepared(result, repair),
        )
    }

    fn show(&mut self, screen: Option<Screen>) {
        self.screen = screen;
        self.form = Form::default();
        if matches!(
            screen,
            Some(Screen::Register | Screen::Change | Screen::Delete | Screen::Recover)
        ) && let Some(id) = &self.server_prefs.selected
            && let Some(login) = self.server_prefs.login(id)
        {
            self.form.login = login.to_owned();
        }
        self.recovery = None;
        self.modal_notice = None;
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        if !matches!(message, Message::Tick) {
            self.paint = self.paint.wrapping_add(1);
        }
        match message {
            Message::Resize(size) => self.size = size,
            Message::Language(language) => {
                self.language = language;
                self.preference_error = !save_preference(
                    self.config.as_ref().map(|c| c.state.as_path()),
                    "language.json",
                    language,
                );
            }
            Message::Mode(mode)
                if self.screen.is_none() && !self.starting && self.session.is_none() =>
            {
                self.mode = mode;
                self.preference_error = !save_preference(
                    self.config.as_ref().map(|c| c.state.as_path()),
                    "mode.json",
                    mode,
                );
            }
            Message::Select(id)
                if self.mode == Mode::Servers
                    && !self.starting
                    && self.session.is_none()
                    && self.screen.is_none()
                    && self.catalog.iter().any(|entry| entry.id == id) =>
            {
                if self.server_prefs.selected.as_ref() != Some(&id) {
                    self.server_prefs.selected = Some(id);
                    self.server = Server::Checking;
                    self.probing = false;
                    self.launch_error = None;
                    self.save_servers();
                    return self.check_server();
                }
            }
            Message::Open(screen) if !self.submitting => {
                self.show(Some(screen));
                if matches!(
                    screen,
                    Screen::Register | Screen::Change | Screen::Delete | Screen::Recover
                ) {
                    return iced::widget::operation::focus(ACCOUNT_LOGIN);
                }
            }
            Message::Tab(back) if self.screen.is_some() && !self.submitting => {
                return if back {
                    iced::widget::operation::focus_previous()
                } else {
                    iced::widget::operation::focus_next()
                };
            }
            Message::Dismiss if !self.submitting => self.show(None),
            Message::Login(login) => self.form.login = login,
            Message::DisplayName(name) => self.form.display_name = name,
            Message::Password(password) => self.form.password = password,
            Message::Current(current) => self.form.current = current,
            Message::Code(code) => self.form.code = code,
            Message::Submit if !self.submitting => {
                if let (
                    Some(c),
                    Some(
                        screen @ (Screen::Register
                        | Screen::Change
                        | Screen::Delete
                        | Screen::Recover),
                    ),
                ) = (self.selected_config(), self.screen)
                    && matches!(self.server, Server::Ready(_))
                    && self.form.error(screen).is_none()
                {
                    self.submitting = true;
                    self.modal_notice = None;
                    let form = std::mem::take(&mut self.form);
                    self.form.login.clone_from(&form.login);
                    self.form.display_name.clone_from(&form.display_name);
                    return Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || account::account(&c, screen, form))
                                .await
                                .unwrap_or(Err(()))
                        },
                        Message::AccountDone,
                    );
                }
            }
            Message::AccountDone(result) => {
                self.submitting = false;
                match result {
                    Ok(code) => {
                        if let Some(id) = self.server_prefs.selected.clone() {
                            self.server_prefs.remember(
                                id,
                                self.form.login.trim(),
                                self.screen == Some(Screen::Delete),
                            );
                            self.save_servers();
                        }
                        self.screen = Some(Screen::AccountResult);
                        self.form = Form::default();
                        self.recovery = code;
                        self.modal_notice = Some((Copy::AccountOk, false));
                    }
                    Err(()) => self.modal_notice = Some((Copy::AccountBad, true)),
                }
            }
            Message::CopyCode => {
                if let Some(code) = &self.recovery {
                    self.modal_notice = Some((Copy::Copied, false));
                    return iced::clipboard::write(code.to_string());
                }
            }
            Message::CheckFiles(repair) => return self.check_files(repair),
            Message::RepairFromHome if self.screen.is_none() => {
                self.show(Some(Screen::Settings));
                return self.check_files(true);
            }
            Message::Retry
                if self.screen.is_none() && self.play_state() == PlayState::WaitingForServer =>
            {
                if self.server == Server::Offline {
                    self.server = Server::Checking;
                    return self.check_server();
                }
            }
            Message::Retry if self.screen.is_none() && self.play_state() == PlayState::Error => {
                if self.files == Files::Error || matches!(self.launch_error, Some(Copy::FilesBad)) {
                    return self.check_files(false);
                }
                self.launch_error = None;
                return self.update(Message::Play);
            }
            Message::Prepared(result, requested_repair) => {
                self.files = if result.is_ok() {
                    Files::Ready
                } else {
                    Files::Error
                };
                if requested_repair {
                    self.modal_notice = Some(match result {
                        Ok(true) => (Copy::FilesRepaired, false),
                        Ok(false) => (Copy::FilesOk, false),
                        Err(()) => (Copy::FilesBad, true),
                    });
                }
            }
            Message::Status(id, status) if self.server_prefs.selected.as_ref() == Some(&id) => {
                self.server = status;
                self.probing = false;
            }
            Message::Play if self.screen.is_none() && self.play_state() == PlayState::Ready => {
                if let Some(c) = self.selected_config() {
                    self.starting = true;
                    let language = self.language;
                    return Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || game::start(c, language))
                                .await
                                .unwrap_or(Err(StartError::Launch))
                        },
                        Message::Started,
                    );
                }
            }
            Message::Started(result) => {
                self.starting = false;
                match result {
                    Ok(session) => {
                        self.session = Some(session);
                        if self.closing {
                            self.stop_game();
                        }
                    }
                    Err(error) => {
                        self.launch_error = Some(match error {
                            StartError::Integrity => Copy::FilesBad,
                            StartError::WritableFiles => Copy::WriteBad,
                            StartError::Launch => Copy::StartBad,
                        })
                    }
                }
            }
            Message::Desktop(action) => {
                if let Some(c) = self.config.clone() {
                    self.modal_notice = None;
                    return Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || desktop::run(c, action).is_ok())
                                .await
                                .unwrap_or(false)
                        },
                        move |ok| Message::DesktopDone(action, ok),
                    );
                }
            }
            Message::DesktopDone(_action, ok) => {
                if !ok && matches!(_action, desktop::Action::Release) {
                    self.show(Some(Screen::Help));
                    self.modal_notice = Some((Copy::ReleaseError, true));
                } else if !ok {
                    self.modal_notice = Some((Copy::OpenBad, true));
                }
                #[cfg(windows)]
                if ok && matches!(_action, desktop::Action::Shortcut) {
                    self.modal_notice = Some((Copy::ShortcutOk, false));
                }
            }
            Message::Close => {
                self.closing = true;
                self.stop_game();
                if !self.starting && self.session.is_none() && self.files != Files::Checking {
                    return iced::exit();
                }
            }
            Message::Tick => {
                self.tick = self.tick.wrapping_add(1);
                let finished = self.session.as_ref().and_then(|s| s.0.lock().ok()?.poll());
                if let Some(end) = finished {
                    self.session = None;
                    if !self.closing {
                        self.launch_error = match end {
                            game::End::SessionEnded => Some(Copy::SessionEnded),
                            game::End::Failed => Some(Copy::StartBad),
                            game::End::Stopped => None,
                        };
                    }
                }
                if self.closing
                    && self.session.is_none()
                    && !self.starting
                    && self.files != Files::Checking
                {
                    return iced::exit();
                }
                if self.tick.is_multiple_of(20) {
                    return self.check_server();
                }
            }
            _ => {}
        }
        Task::none()
    }

    fn stop_game(&self) {
        if let Some(session) = &self.session
            && let Ok(mut owned) = session.0.lock()
        {
            owned.request_stop();
        }
    }
}

fn keys(event: Event, status: event::Status, _: window::Id) -> Option<Message> {
    if let Event::Keyboard(keyboard::Event::KeyPressed {
        key: keyboard::Key::Named(key),
        modifiers,
        ..
    }) = event
    {
        use keyboard::key::Named;
        return match key {
            Named::Enter if modifiers.is_empty() && status == event::Status::Ignored => {
                Some(Message::Play)
            }
            Named::Escape if modifiers.is_empty() => Some(Message::Dismiss),
            Named::Tab if modifiers.is_empty() || modifiers == keyboard::Modifiers::SHIFT => {
                Some(Message::Tab(modifiers.shift()))
            }
            _ => None,
        };
    }
    None
}

fn main() -> iced::Result {
    iced::application(Player::boot, Player::update, Player::view)
        .title(|player: &Player| player.language.get(Copy::Title).to_owned())
        .theme(Theme::Dark)
        .style(|player: &Player, theme: &Theme| {
            look::surface(theme, player.screen.is_some(), player.paint)
        })
        .window(window::Settings {
            size: WINDOW_SIZE,
            min_size: Some(MINIMUM),
            position: window::Position::SpecificWith(|_, monitor| {
                let size = fitted_window(monitor);
                iced::Point::new(
                    (monitor.width - size.width) / 2.0,
                    (monitor.height - size.height) / 2.0,
                )
            }),
            exit_on_close_request: false,
            ..Default::default()
        })
        .subscription(|_| {
            Subscription::batch([
                iced::time::every(Duration::from_millis(250)).map(|_| Message::Tick),
                window::close_requests().map(|_| Message::Close),
                window::resize_events().map(|(_, size)| Message::Resize(size)),
                event::listen_with(keys),
            ])
        })
        .run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_fits_short_monitors_without_changing_aspect() {
        assert_eq!(fitted_window(iced::Size::new(1600.0, 1000.0)), WINDOW_SIZE);
        let small = fitted_window(iced::Size::new(1024.0, 768.0));
        assert_eq!(small.height, 696.0);
        assert!(
            (small.width / small.height - WINDOW_SIZE.width / WINDOW_SIZE.height).abs() < 0.002
        );
    }

    #[test]
    fn play_is_enabled_only_for_ready_online_files_and_server() {
        let states = [
            (
                Mode::Servers,
                Files::Ready,
                Server::Ready(4),
                false,
                false,
                false,
                PlayState::Ready,
            ),
            (
                Mode::OfflineFriends,
                Files::Ready,
                Server::Ready(4),
                false,
                false,
                false,
                PlayState::Soon,
            ),
            (
                Mode::Servers,
                Files::Checking,
                Server::Ready(4),
                false,
                false,
                false,
                PlayState::Checking,
            ),
            (
                Mode::Servers,
                Files::Error,
                Server::Ready(4),
                false,
                false,
                false,
                PlayState::Error,
            ),
            (
                Mode::Servers,
                Files::Ready,
                Server::Offline,
                false,
                false,
                false,
                PlayState::WaitingForServer,
            ),
            (
                Mode::Servers,
                Files::Ready,
                Server::Ready(4),
                true,
                false,
                false,
                PlayState::Starting,
            ),
            (
                Mode::Servers,
                Files::Ready,
                Server::Ready(4),
                false,
                true,
                false,
                PlayState::Running,
            ),
            (
                Mode::Servers,
                Files::Ready,
                Server::Ready(4),
                false,
                false,
                true,
                PlayState::Error,
            ),
        ];
        for (mode, files, server, starting, running, error, expected) in states {
            assert_eq!(
                play_state(mode, files, server, starting, running, error),
                expected
            );
        }
    }

    #[test]
    fn remembers_both_choices_and_defaults_on_missing_or_bad_state() {
        let path = std::env::temp_dir().join(format!(
            "kr-player-choice-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir(&path).unwrap();
        let root = Some(path.as_path());
        assert_eq!(preference::<Language>(root, "language.json"), Language::Vi);
        assert!(save_preference(root, "mode.json", Mode::OfflineFriends));
        assert!(save_preference(root, "language.json", Language::Ko));
        assert_eq!(preference::<Mode>(root, "mode.json"), Mode::OfflineFriends);
        assert_eq!(preference::<Language>(root, "language.json"), Language::Ko);
        fs::write(path.join("mode.json"), b"bad JSON").unwrap();
        assert_eq!(preference::<Mode>(root, "mode.json"), Mode::Servers);
        fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn account_form_supplies_inline_errors_until_the_request_is_complete() {
        let mut form = Form::default();
        assert!(matches!(
            form.error(Screen::Register),
            Some(Copy::LoginHint)
        ));
        form.login = "fixture_player".into();
        assert!(matches!(form.error(Screen::Register), Some(Copy::NameHint)));
        form.display_name = "Bạn ơi".into();
        assert!(matches!(
            form.error(Screen::Register),
            Some(Copy::PasswordHint)
        ));
        form.password = Zeroizing::new("fixture-only-password".into());
        assert!(form.error(Screen::Register).is_none());
        assert!(matches!(form.error(Screen::Recover), Some(Copy::CodeHint)));
        form.code = Zeroizing::new("ab".repeat(32));
        assert!(form.error(Screen::Recover).is_none());
    }
}
