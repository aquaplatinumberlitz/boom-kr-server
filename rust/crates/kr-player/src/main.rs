#![forbid(unsafe_code)]
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use iced::{
    Background, Border, Color, Element, Length, Subscription, Task, Theme,
    widget::{Space, button, canvas, column, container, row, text, text_input},
};
use kr_tunnel::launch_tls;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    net::{SocketAddrV4, TcpStream, UdpSocket},
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use zeroize::Zeroizing;

const NAVY: Color = Color::from_rgb(0.035, 0.051, 0.10);
const PANEL: Color = Color::from_rgb(0.067, 0.09, 0.15);
const MINT: Color = Color::from_rgb(0.43, 0.98, 0.76);
const MUTED: Color = Color::from_rgb(0.56, 0.62, 0.73);

fn field_style(_: &Theme, status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: NAVY.into(),
        border: Border {
            color: if matches!(status, text_input::Status::Focused { .. }) {
                MINT
            } else {
                Color::from_rgb(0.18, 0.23, 0.31)
            },
            width: 1.0,
            radius: 10.0.into(),
        },
        icon: MUTED,
        placeholder: MUTED,
        value: Color::WHITE,
        selection: Color::from_rgba(0.43, 0.98, 0.76, 0.3),
    }
}

#[derive(Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Language {
    #[default]
    Vi,
    En,
    Ko,
}
impl Language {
    fn id(self) -> &'static str {
        match self {
            Self::Vi => "vi",
            Self::En => "en",
            Self::Ko => "ko",
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Vi => "Tiếng Việt",
            Self::En => "English",
            Self::Ko => "한국어",
        }
    }
    fn line(self, key: usize) -> &'static str {
        const VI: [&str; 18] = [
            "VÀO TRẬN.\nBÙNG NỔ.",
            "Một căn phòng. Những người bạn. Vô vàn ván mới.",
            "Chào mừng trở lại",
            "Đăng nhập và hẹn nhau trong phòng.",
            "Tài khoản",
            "Mật khẩu",
            "VÀO CHƠI  →",
            "Đang kiểm tra máy chủ",
            "Máy chủ sẵn sàng",
            "Chưa kết nối máy chủ",
            "Đang chuẩn bị trò chơi…",
            "Không thể kiểm tra bộ cài. Hãy kiểm tra lại gói trò chơi.",
            "Không thể đăng nhập hoặc mở trò chơi. Hãy thử lại.",
            "Trò chơi đang chạy",
            "Đang đóng trò chơi…",
            "DỪNG TRÒ CHƠI",
            "Không thể lưu lựa chọn ngôn ngữ.",
            "● CHƠI CÙNG NHAU",
        ];
        const EN: [&str; 18] = [
            "ONE MORE\nROUND.",
            "One room. Your friends. A whole new round.",
            "Welcome back",
            "Sign in. Meet your friends in the room.",
            "Account",
            "Password",
            "LET’S PLAY  →",
            "Checking server",
            "Server online",
            "Server unavailable",
            "Preparing your game…",
            "Game files could not be verified. Check your game package.",
            "Could not sign in or start the game. Please try again.",
            "Your game is running",
            "Closing your game…",
            "STOP GAME",
            "Could not save your language choice.",
            "● PLAY TOGETHER",
        ];
        const KO: [&str; 18] = [
            "다시 한 판,\n함께 즐겨요.",
            "하나의 방. 반가운 친구들. 새로운 한 판.",
            "다시 만나 반가워요",
            "로그인하고 친구들과 같은 방에서 만나요.",
            "계정",
            "비밀번호",
            "게임 시작  →",
            "서버 확인 중",
            "서버 접속 가능",
            "서버에 연결할 수 없음",
            "게임 준비 중…",
            "게임 파일을 확인할 수 없습니다. 설치 파일을 확인해 주세요.",
            "로그인하거나 게임을 시작할 수 없습니다. 다시 시도해 주세요.",
            "게임 실행 중",
            "게임 종료 중…",
            "게임 종료",
            "언어 설정을 저장할 수 없습니다.",
            "● 함께 즐겨요",
        ];
        match self {
            Self::Vi => VI[key],
            Self::En => EN[key],
            Self::Ko => KO[key],
        }
    }
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    server: SocketAddrV4,
    certificate: PathBuf,
    launcher: PathBuf,
    companion: PathBuf,
    bundle: PathBuf,
    manifest_sha256: String,
    installation: PathBuf,
    state: PathBuf,
    port_budget: usize,
    #[serde(default)]
    wine: Option<PathBuf>,
}
#[derive(Clone)]
enum Message {
    Language(Language),
    Login(String),
    Password(String),
    Tick,
    Status(bool),
    Prepared(bool),
    Play,
    Started(Result<Session, ()>),
    Close,
    Stop,
}
#[derive(Clone)]
struct Session(std::sync::Arc<std::sync::Mutex<OwnedSession>>);
struct OwnedSession {
    child: Child,
    cancel: PathBuf,
    heartbeat: PathBuf,
}
impl Drop for OwnedSession {
    fn drop(&mut self) {
        let _ = fs::write(&self.cancel, []);
    }
}
struct Player {
    config: Option<Config>,
    language: Language,
    login: String,
    password: Zeroizing<String>,
    online: Option<bool>,
    checking: bool,
    prepared: Option<bool>,
    starting: bool,
    session: Option<Session>,
    stopping: bool,
    closing: bool,
    notice: Option<usize>,
    ticks: usize,
}
fn probe(config: &Config) -> bool {
    let result = (|| -> std::io::Result<bool> {
        let socket = TcpStream::connect_timeout(&config.server.into(), Duration::from_secs(2))?;
        let mut stream = launch_tls::connect(
            socket,
            fs::read(&config.certificate)?,
            Duration::from_secs(2),
        )?;
        launch_tls::write_frame(
            &mut stream,
            b"{\"op\":\"status\"}\n",
            launch_tls::REQUEST_LIMIT,
        )?;
        let value: serde_json::Value =
            serde_json::from_slice(&launch_tls::read_frame(&mut stream, 256)?)
                .map_err(std::io::Error::other)?;
        Ok(value["ok"] == true && value["ready"] == true)
    })();
    result.unwrap_or(false)
}
fn start(
    config: Config,
    language: Language,
    login: String,
    password: Zeroizing<String>,
) -> Result<Session, ()> {
    kr_player::verify_language(
        &config.bundle,
        &config.manifest_sha256,
        &config.installation,
        language.id(),
    )
    .map_err(|_| ())?;
    let run = config.state.join(format!(
        "run-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ())?
            .as_nanos()
    ));
    fs::create_dir(&run).map_err(|_| ())?;
    let cancel = run.join("cancel");
    let heartbeat = run.join("heartbeat");
    fs::write(&heartbeat, b"0").map_err(|_| ())?;
    let game = config.installation.join(language.id());
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|_| ())?;
    socket.connect(config.server).map_err(|_| ())?;
    let reported = socket.local_addr().map_err(|_| ())?.ip();
    let native_program = config
        .wine
        .as_ref()
        .cloned()
        .unwrap_or_else(|| game.join("CA.exe"));
    let mut arguments = Vec::<String>::new();
    if config.wine.is_some() {
        arguments.push(game.join("CA.exe").to_string_lossy().into_owned());
    }
    arguments.push("multiserveridx:0".into());
    let value = serde_json::json!({"server":config.server,"certificate":config.certificate,"reported_login_ipv4":reported,
        "native_ip":"127.0.0.1","companion":config.companion,"native_program":native_program,"native_arguments":arguments,
        "native_directory":game,"port_budget":config.port_budget,"cancel_file":cancel,"owner_heartbeat":heartbeat});
    let path = run.join("launch.json");
    fs::write(&path, serde_json::to_vec(&value).map_err(|_| ())?).map_err(|_| ())?;
    #[derive(Serialize)]
    struct Credentials<'a> {
        login: &'a str,
        password: &'a str,
    }
    let credential = Zeroizing::new(
        serde_json::to_vec(&Credentials {
            login: &login,
            password: &password,
        })
        .map_err(|_| ())?,
    );
    let mut command = Command::new(config.launcher);
    command
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW; cancellation is the owned file seam.
    }
    if config.wine.is_some() {
        command.env(
            "WINEPREFIX",
            config.state.join(format!("wine-{}", language.id())),
        );
    }
    let child = command.spawn().map_err(|_| ())?;
    let mut owned = OwnedSession {
        child,
        cancel,
        heartbeat,
    };
    let result = owned.child.stdin.take().ok_or(())?.write_all(&credential);
    if result.is_err() {
        let _ = fs::write(&owned.cancel, []);
        return Err(());
    }
    Ok(Session(std::sync::Arc::new(std::sync::Mutex::new(owned))))
}
impl Player {
    fn boot() -> (Self, Task<Message>) {
        let preview = std::env::args_os()
            .nth(1)
            .is_some_and(|arg| arg == "--preview");
        let config = if preview { None } else { load_config().ok() };
        let language = config
            .as_ref()
            .and_then(|c| fs::read(c.state.join("language.json")).ok())
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        let task = config.clone().map_or_else(Task::none, |c| {
            Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        if c.installation.exists() {
                            kr_player::verify_install(
                                &c.bundle,
                                &c.manifest_sha256,
                                &c.installation,
                            )
                        } else {
                            kr_player::install(&c.bundle, &c.manifest_sha256, &c.installation)
                        }
                    })
                    .await
                    .is_ok_and(|result| result.is_ok())
                },
                Message::Prepared,
            )
        });
        (
            Self {
                notice: if !preview && config.is_none() {
                    Some(11)
                } else {
                    None
                },
                config,
                language,
                login: String::new(),
                password: Zeroizing::new(String::new()),
                online: None,
                checking: false,
                prepared: None,
                starting: false,
                session: None,
                stopping: false,
                closing: false,
                ticks: 0,
            },
            task,
        )
    }
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Language(language) => {
                self.language = language;
                if let Some(config) = &self.config
                    && fs::write(
                        config.state.join("language.json"),
                        serde_json::to_vec(&language).unwrap_or_default(),
                    )
                    .is_err()
                {
                    self.notice = Some(16);
                }
            }
            Message::Login(login) => self.login = login,
            Message::Password(password) => self.password = Zeroizing::new(password),
            Message::Prepared(ready) => {
                self.prepared = Some(ready);
                if !ready {
                    self.notice = Some(11);
                }
            }
            Message::Status(online) => {
                self.online = Some(online);
                self.checking = false;
            }
            Message::Play
                if self.prepared == Some(true)
                    && self.online == Some(true)
                    && self.session.is_none()
                    && !self.starting =>
            {
                if let Some(config) = self.config.clone() {
                    self.starting = true;
                    self.notice = None;
                    let password =
                        std::mem::replace(&mut self.password, Zeroizing::new(String::new()));
                    let login = self.login.clone();
                    let language = self.language;
                    return Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                start(config, language, login, password)
                            })
                            .await
                            .unwrap_or(Err(()))
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
                            return self.update(Message::Stop);
                        }
                    }
                    Err(()) => self.notice = Some(12),
                }
            }
            Message::Stop => {
                if let Some(session) = &self.session {
                    if let Ok(owned) = session.0.lock() {
                        let _ = fs::write(&owned.cancel, []);
                    }
                    self.stopping = true;
                }
            }
            Message::Close => {
                self.closing = true;
                if self.session.is_some() {
                    return self.update(Message::Stop);
                }
                if !self.starting {
                    return iced::exit();
                }
            }
            Message::Tick => {
                self.ticks += 1;
                if let Some(session) = &self.session
                    && let Ok(owned) = session.0.lock()
                {
                    let _ = fs::write(&owned.heartbeat, self.ticks.to_string());
                }
                let finished = self
                    .session
                    .as_ref()
                    .and_then(|s| s.0.lock().ok()?.child.try_wait().ok().flatten());
                if let Some(status) = finished {
                    self.session = None;
                    self.stopping = false;
                    if !status.success() && !self.closing {
                        self.notice = Some(12);
                    }
                }
                if self.closing && self.session.is_none() && !self.starting {
                    return iced::exit();
                }
                if self.ticks % 10 == 1
                    && !self.checking
                    && let Some(config) = self.config.clone()
                {
                    self.checking = true;
                    return Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || probe(&config))
                                .await
                                .unwrap_or(false)
                        },
                        Message::Status,
                    );
                }
            }
            _ => {}
        }
        Task::none()
    }
    fn view(&self) -> Element<'_, Message> {
        let language = self.language;
        let label = |key| text(language.line(key));
        let mut choices = row![].spacing(6);
        for candidate in [Language::Vi, Language::En, Language::Ko] {
            let selected = candidate == language;
            choices = choices.push(
                button(text(candidate.name()).size(13))
                    .padding([10, 14])
                    .style(move |_, _| button::Style {
                        background: Some(Background::Color(if selected { MINT } else { PANEL })),
                        text_color: if selected { NAVY } else { Color::WHITE },
                        border: Border::default().rounded(9),
                        ..Default::default()
                    })
                    .on_press(Message::Language(candidate)),
            );
        }
        let status_key = match self.online {
            None => 7,
            Some(true) => 8,
            Some(false) => 9,
        };
        let header = row![
            row![
                text("BOOM").size(29).color(Color::WHITE),
                text("PLAY").size(12).color(MINT)
            ]
            .spacing(9)
            .align_y(iced::Alignment::Center),
            Space::new().width(Length::Fill),
            choices
        ]
        .align_y(iced::Alignment::Center);
        let hero = column![
            label(17).size(12).color(MINT),
            label(0).size(52).color(Color::WHITE),
            label(1).size(15).color(MUTED),
            canvas::Canvas::new(Arcade).width(Length::Fill).height(220)
        ]
        .spacing(20)
        .width(Length::FillPortion(6));
        let busy = self.starting || self.session.is_some();
        let action = if self.session.is_some() { 15 } else { 6 };
        let mut play = button(label(action).size(18))
            .padding([17, 24])
            .width(Length::Fill)
            .style(|_, status| button::Style {
                background: Some(Background::Color(if status == button::Status::Disabled {
                    Color::from_rgb(0.19, 0.29, 0.29)
                } else {
                    MINT
                })),
                text_color: NAVY,
                border: Border::default().rounded(12),
                ..Default::default()
            });
        if self.session.is_some() && !self.stopping {
            play = play.on_press(Message::Stop);
        } else if !busy
            && self.prepared == Some(true)
            && self.online == Some(true)
            && !self.login.is_empty()
            && !self.password.is_empty()
        {
            play = play.on_press(Message::Play);
        }
        let notice = if let Some(key) = self.notice {
            key
        } else if self.stopping {
            14
        } else if self.starting {
            10
        } else if busy {
            13
        } else if self.prepared.is_none() {
            10
        } else {
            status_key
        };
        let form = column![
            label(2).size(29).color(Color::WHITE),
            label(3).size(13).color(MUTED),
            Space::new().height(10),
            label(4).size(12).color(MUTED),
            text_input(language.line(4), &self.login)
                .on_input(Message::Login)
                .style(field_style)
                .padding(13),
            label(5).size(12).color(MUTED),
            text_input(language.line(5), &self.password)
                .secure(true)
                .on_input(Message::Password)
                .style(field_style)
                .padding(13),
            Space::new().height(8),
            play,
            label(notice).size(12).color(MUTED)
        ]
        .spacing(13);
        let card = container(form)
            .padding(30)
            .width(Length::FillPortion(4))
            .style(|_| container::Style {
                background: Some(PANEL.into()),
                border: Border {
                    color: Color::from_rgb(0.15, 0.20, 0.28),
                    width: 1.0,
                    radius: 22.0.into(),
                },
                ..Default::default()
            });
        let body = row![hero, Space::new().width(32), card].align_y(iced::Alignment::Center);
        let footer = row![
            text("BOOM  /  PLAY").size(11).color(MUTED),
            Space::new().width(Length::Fill),
            label(status_key)
                .size(12)
                .color(if self.online == Some(true) {
                    MINT
                } else {
                    MUTED
                })
        ];
        container(
            column![
                header,
                Space::new().height(36),
                body,
                Space::new().height(24),
                footer
            ]
            .spacing(8),
        )
        .padding([30, 40])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style::default().background(NAVY))
        .into()
    }
}
struct Arcade;
impl canvas::Program<Message> for Arcade {
    type State = ();
    fn draw(
        &self,
        _: &(),
        renderer: &iced::Renderer,
        _: &Theme,
        bounds: iced::Rectangle,
        _: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        for x in 0..9 {
            for y in 0..4 {
                frame.fill_rectangle(
                    iced::Point::new(x as f32 * 48.0 + 8.0, y as f32 * 48.0 + 20.0),
                    iced::Size::new(36.0, 36.0),
                    Color::from_rgba(0.20, 0.31, 0.40, 0.25),
                );
            }
        }
        for (x, y, r, color) in [
            (100.0, 108.0, 63.0, MINT),
            (263.0, 135.0, 43.0, Color::from_rgb(0.64, 0.55, 1.0)),
            (335.0, 57.0, 25.0, Color::from_rgb(1.0, 0.84, 0.35)),
        ] {
            let center = iced::Point::new(x, y);
            frame.fill(
                &canvas::Path::circle(iced::Point::new(x, y + 10.0), r),
                Color::from_rgba(0.0, 0.0, 0.0, 0.28),
            );
            frame.fill(&canvas::Path::circle(center, r), color);
            frame.fill(
                &canvas::Path::circle(iced::Point::new(x - r * 0.27, y - r * 0.32), r * 0.19),
                Color::from_rgba(1.0, 1.0, 1.0, 0.55),
            );
            frame.fill_rectangle(
                iced::Point::new(x - r * 0.28, y + r * 0.1),
                iced::Size::new(r * 0.12, r * 0.27),
                NAVY,
            );
            frame.fill_rectangle(
                iced::Point::new(x + r * 0.16, y + r * 0.1),
                iced::Size::new(r * 0.12, r * 0.27),
                NAVY,
            );
        }
        vec![frame.into_geometry()]
    }
}
fn load_config() -> std::io::Result<Config> {
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or(std::env::current_exe()?.with_file_name("launcher.json"));
    let root = path
        .canonicalize()?
        .parent()
        .ok_or_else(|| std::io::Error::other("launcher config root"))?
        .to_owned();
    let mut config: Config =
        serde_json::from_slice(&fs::read(&path)?).map_err(std::io::Error::other)?;
    for path in [
        &mut config.certificate,
        &mut config.launcher,
        &mut config.companion,
        &mut config.bundle,
        &mut config.installation,
        &mut config.state,
    ] {
        if !path.is_absolute() {
            *path = root.join(&*path);
        }
    }
    if let Some(wine) = &mut config.wine {
        if !wine.is_absolute() {
            *wine = root.join(&*wine);
        }
        *wine = wine.canonicalize()?;
    }
    for path in [
        &mut config.certificate,
        &mut config.launcher,
        &mut config.companion,
        &mut config.bundle,
    ] {
        *path = path.canonicalize()?;
    }
    if !(26..=32768).contains(&config.port_budget)
        || config.server.port() == 0
        || config.server.ip().is_unspecified()
        || config.server.ip().is_multicast()
        || config.server.ip().is_broadcast()
    {
        return Err(std::io::Error::other("launcher bounds"));
    }
    fs::create_dir_all(&config.state)?;
    config.state = config.state.canonicalize()?;
    let parent = config
        .installation
        .parent()
        .ok_or_else(|| std::io::Error::other("installation parent"))?
        .canonicalize()?;
    config.installation = parent.join(
        config
            .installation
            .file_name()
            .ok_or_else(|| std::io::Error::other("installation name"))?,
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&config.state, fs::Permissions::from_mode(0o700))?;
    }
    Ok(config)
}
fn main() -> iced::Result {
    iced::application(Player::boot, Player::update, Player::view)
        .title("BOOM PLAY")
        .theme(Theme::Dark)
        .window_size((1000.0, 680.0))
        .centered()
        .resizable(false)
        .exit_on_close_request(false)
        .subscription(|_| {
            Subscription::batch([
                iced::time::every(Duration::from_millis(500)).map(|_| Message::Tick),
                iced::window::close_requests().map(|_| Message::Close),
            ])
        })
        .run()
}
