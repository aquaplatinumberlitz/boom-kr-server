#![forbid(unsafe_code)]
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod game;
mod look;
use game::{Config, Session, StartError, start};
use iced::{
    Alignment, Color, Element, Font, Length, Subscription, Task, Theme,
    widget::{Space, button, canvas, column, container, row, stack, text, text_input},
};
use kr_tunnel::launch_tls;
use serde::{Deserialize, Serialize};
use std::{fs, net::TcpStream, time::Duration};
use zeroize::Zeroizing;

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
        const VI: [&str; 36] = [
            "VÀO TRẬN.\nBÙNG NỔ.",
            "Một căn phòng. Những người bạn. Vô vàn ván mới.",
            "Chào mừng bạn",
            "Đăng nhập hoặc chơi thử trong game.",
            "Tài khoản",
            "Mật khẩu",
            "Vào game  →",
            "Đang kiểm tra máy chủ",
            "Máy chủ sẵn sàng",
            "Chưa kết nối máy chủ",
            "Đang chuẩn bị trò chơi…",
            "Không thể kiểm tra bộ cài. Hãy kiểm tra lại gói trò chơi.",
            "Không thể mở trò chơi. Hãy thử lại.",
            "Trò chơi đang chạy",
            "Đang đóng trò chơi…",
            "DỪNG TRÒ CHƠI",
            "Không thể lưu lựa chọn ngôn ngữ.",
            "● CHƠI CÙNG NHAU",
            "Quản lý tài khoản",
            "Đăng ký",
            "Đổi mật khẩu",
            "Xoá tài khoản",
            "Khôi phục mật khẩu",
            "Quay lại",
            "Không thực hiện được. Kiểm tra thông tin và thử lại.",
            "Đã thực hiện.",
            "Tên hiển thị",
            "Mật khẩu hiện tại",
            "Mật khẩu mới",
            "Mã khôi phục một lần",
            "Lưu mã này ở nơi an toàn. Mã chỉ hiện một lần.",
            "Sao chép mã",
            "Mật khẩu: 12–128 ký tự ASCII in được.",
            "Xác nhận xoá tài khoản",
            "Không thể ghi tệp trò chơi. Kiểm tra quyền ghi của thư mục cài đặt.",
            "Phiên chơi đã kết thúc. Hãy mở game và đăng nhập lại.",
        ];
        const EN: [&str; 36] = [
            "ONE MORE\nROUND.",
            "One room. Your friends. A whole new round.",
            "Welcome",
            "Sign in or try the game from its login screen.",
            "Account",
            "Password",
            "Play game  →",
            "Checking server",
            "Server online",
            "Server unavailable",
            "Preparing your game…",
            "Game files could not be verified. Check your game package.",
            "Could not start the game. Please try again.",
            "Your game is running",
            "Closing your game…",
            "STOP GAME",
            "Could not save your language choice.",
            "● PLAY TOGETHER",
            "Manage accounts",
            "Register",
            "Change password",
            "Delete account",
            "Recover password",
            "Back",
            "Request failed. Check your details and try again.",
            "Done.",
            "Display name",
            "Current password",
            "New password",
            "One-time recovery code",
            "Keep this code somewhere safe. It is shown only once.",
            "Copy code",
            "Password: 12–128 printable ASCII characters.",
            "Confirm account deletion",
            "Could not write game files. Check write access to the installation folder.",
            "Your game session has ended. Open the game and sign in again.",
        ];
        const KO: [&str; 36] = [
            "다시 한 판,\n함께 즐겨요.",
            "하나의 방. 반가운 친구들. 새로운 한 판.",
            "환영합니다",
            "게임에서 로그인하거나 체험해 보세요.",
            "계정",
            "비밀번호",
            "게임 시작  →",
            "서버 확인 중",
            "서버 접속 가능",
            "서버에 연결할 수 없음",
            "게임 준비 중…",
            "게임 파일을 확인할 수 없습니다. 설치 파일을 확인해 주세요.",
            "게임을 시작할 수 없습니다. 다시 시도해 주세요.",
            "게임 실행 중",
            "게임 종료 중…",
            "게임 종료",
            "언어 설정을 저장할 수 없습니다.",
            "● 함께 즐겨요",
            "계정 관리",
            "가입",
            "비밀번호 변경",
            "계정 삭제",
            "비밀번호 복구",
            "뒤로",
            "처리할 수 없습니다. 정보를 확인한 후 다시 시도해 주세요.",
            "완료되었습니다.",
            "표시 이름",
            "현재 비밀번호",
            "새 비밀번호",
            "일회용 복구 코드",
            "이 코드를 안전한 곳에 보관하세요. 한 번만 표시됩니다.",
            "코드 복사",
            "비밀번호: 인쇄 가능한 ASCII 문자 12~128자.",
            "계정 삭제 확인",
            "게임 파일에 쓸 수 없습니다. 설치 폴더의 쓰기 권한을 확인해 주세요.",
            "게임 세션이 종료되었습니다. 게임을 열고 다시 로그인해 주세요.",
        ];
        match self {
            Self::Vi => VI[key],
            Self::En => EN[key],
            Self::Ko => KO[key],
        }
    }
}
#[derive(Clone)]
enum Message {
    Language(Language),
    Account(Screen),
    Login(String),
    DisplayName(String),
    Password(Zeroizing<String>),
    Current(Zeroizing<String>),
    Code(Zeroizing<String>),
    Submit,
    AccountDone(Result<Option<Zeroizing<String>>, ()>),
    CopyCode,
    Tick,
    Status(bool),
    Prepared(bool),
    Play,
    Started(Result<Session, StartError>),
    Close,
    Stop,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Home,
    Menu,
    Register,
    Change,
    Delete,
    Recover,
}
#[derive(Default)]
struct Form {
    login: String,
    display_name: String,
    password: Zeroizing<String>,
    current: Zeroizing<String>,
    code: Zeroizing<String>,
}
struct Player {
    config: Option<Config>,
    language: Language,
    screen: Screen,
    form: Form,
    submitting: bool,
    recovery: Option<Zeroizing<String>>,
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
fn account(config: &Config, screen: Screen, form: Form) -> Result<Option<Zeroizing<String>>, ()> {
    #[derive(Serialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    enum Action<'a> {
        Register {
            display_name: &'a str,
            password: &'a str,
        },
        Change {
            current: &'a str,
            password: &'a str,
        },
        Delete {
            current: &'a str,
        },
        Recover {
            code: &'a str,
            password: &'a str,
        },
    }
    #[derive(Serialize)]
    struct Request<'a> {
        op: &'static str,
        login: &'a str,
        action: Action<'a>,
    }
    #[derive(Deserialize)]
    struct Response {
        ok: bool,
        recovery_code: Option<String>,
    }
    let action = match screen {
        Screen::Register => Action::Register {
            display_name: &form.display_name,
            password: &form.password,
        },
        Screen::Change => Action::Change {
            current: &form.current,
            password: &form.password,
        },
        Screen::Delete => Action::Delete {
            current: &form.current,
        },
        Screen::Recover => Action::Recover {
            code: form.code.trim(),
            password: &form.password,
        },
        _ => return Err(()),
    };
    let mut data = Zeroizing::new(
        serde_json::to_vec(&Request {
            op: "account",
            login: form.login.trim(),
            action,
        })
        .map_err(|_| ())?,
    );
    data.push(b'\n');
    let timeout = Duration::from_secs(3);
    let socket = TcpStream::connect_timeout(&config.server.into(), timeout).map_err(|_| ())?;
    let mut tls = launch_tls::connect(
        socket,
        fs::read(&config.certificate).map_err(|_| ())?,
        timeout,
    )
    .map_err(|_| ())?;
    launch_tls::write_frame(&mut tls, &data, launch_tls::REQUEST_LIMIT).map_err(|_| ())?;
    drop(data);
    let frame = launch_tls::read_frame(&mut tls, launch_tls::RESPONSE_LIMIT).map_err(|_| ())?;
    let response: Response = serde_json::from_slice(&frame).map_err(|_| ())?;
    let code = response.recovery_code.map(Zeroizing::new);
    if response.ok
        && code.as_ref().is_some_and(|s| {
            s.len() != 64
                || !s
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
    {
        return Err(());
    }
    let code_matches = match screen {
        Screen::Register | Screen::Recover => code.is_some(),
        Screen::Change => true,
        _ => code.is_none(),
    };
    if response.ok && code_matches {
        Ok(code)
    } else {
        Err(())
    }
}
impl Player {
    fn boot() -> (Self, Task<Message>) {
        let preview = std::env::args_os()
            .nth(1)
            .is_some_and(|arg| arg == "--preview");
        let config = if preview { None } else { Config::load().ok() };
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
                screen: Screen::Home,
                form: Form::default(),
                submitting: false,
                recovery: None,
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
            Message::Account(screen) if !self.submitting => {
                self.form = Form::default();
                self.recovery = None;
                self.notice = None;
                self.screen = screen;
            }
            Message::Login(login) => self.form.login = login,
            Message::DisplayName(name) => self.form.display_name = name,
            Message::Password(password) => self.form.password = password,
            Message::Current(current) => self.form.current = current,
            Message::Code(code) => self.form.code = code,
            Message::Submit if !self.submitting && self.ready_to_submit() => {
                if let Some(config) = self.config.clone() {
                    self.submitting = true;
                    self.notice = None;
                    let screen = self.screen;
                    let form = std::mem::take(&mut self.form);
                    return Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || account(&config, screen, form))
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
                        self.screen = Screen::Menu;
                        self.notice = Some(25);
                        self.recovery = code;
                    }
                    Err(()) => self.notice = Some(24),
                }
            }
            Message::CopyCode => {
                if let Some(code) = &self.recovery {
                    return iced::clipboard::write(code.to_string());
                }
            }
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
                    let language = self.language;
                    return Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || start(config, language))
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
                            return self.update(Message::Stop);
                        }
                    }
                    Err(error) => {
                        self.notice = Some(match error {
                            StartError::Integrity => 11,
                            StartError::WritableFiles => 34,
                            StartError::Launch => 12,
                        })
                    }
                }
            }
            Message::Stop => {
                if let Some(session) = &self.session {
                    if let Ok(mut owned) = session.0.lock() {
                        owned.request_stop();
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
                let finished = self.session.as_ref().and_then(|s| s.0.lock().ok()?.poll());
                if let Some(status) = finished {
                    self.session = None;
                    if !self.closing && !self.stopping {
                        self.notice = match status {
                            game::End::SessionEnded => Some(35),
                            game::End::Failed => Some(12),
                            game::End::Stopped => self.notice,
                        };
                    }
                    self.stopping = false;
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
    fn ready_to_submit(&self) -> bool {
        let form = &self.form;
        let password = |value: &str| (12..=128).contains(&value.len());
        self.online == Some(true)
            && (3..=32).contains(&form.login.trim().len())
            && match self.screen {
                Screen::Register => {
                    !form.display_name.trim().is_empty() && password(&form.password)
                }
                Screen::Change => password(&form.current) && password(&form.password),
                Screen::Delete => password(&form.current),
                Screen::Recover => form.code.trim().len() == 64 && password(&form.password),
                _ => false,
            }
    }
    fn panel<'a>(
        &'a self,
        play: Element<'a, Message>,
        notice: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let l = self.language;
        let title = |key| {
            text(l.line(key))
                .size(28)
                .font(look::HEAVY)
                .color(Color::WHITE)
        };
        let small = |key| text(l.line(key)).size(14).color(look::MUTED);
        let menu_button = |key| {
            button(
                text(l.line(key))
                    .size(14)
                    .font(look::BOLD)
                    .center()
                    .width(Length::Fill),
            )
            .padding([8, 12])
            .width(Length::Fill)
            .style(look::segment(false))
        };
        let navigate = |key, page| {
            menu_button(key).on_press_maybe((!self.submitting).then_some(Message::Account(page)))
        };
        let field = |key, value, secret, change: fn(String) -> Message| {
            text_input(l.line(key), value)
                .secure(secret)
                .size(15)
                .padding([9, 12])
                .style(look::field)
                .on_input_maybe((!self.submitting).then_some(change))
        };
        let body = match self.screen {
            Screen::Home => column![
                title(2),
                small(3),
                Space::new().height(10),
                play,
                navigate(18, Screen::Menu),
                notice,
            ],
            Screen::Menu => {
                if let Some(code) = &self.recovery {
                    column![
                        title(29),
                        small(30),
                        text(format!("{}\n{}", &code[..32], &code[32..]))
                            .size(13)
                            .font(Font::MONOSPACE)
                            .color(Color::WHITE),
                        menu_button(31).on_press(Message::CopyCode),
                        navigate(23, Screen::Home),
                    ]
                } else {
                    column![
                        title(18),
                        navigate(19, Screen::Register),
                        navigate(20, Screen::Change),
                        navigate(21, Screen::Delete),
                        navigate(22, Screen::Recover),
                        navigate(23, Screen::Home),
                        notice,
                    ]
                }
            }
            screen => {
                let key = match screen {
                    Screen::Register => 19,
                    Screen::Change => 20,
                    Screen::Delete => 21,
                    Screen::Recover => 22,
                    _ => unreachable!(),
                };
                let mut form = column![
                    title(key),
                    field(4, &self.form.login, false, Message::Login)
                ];
                if screen == Screen::Register {
                    form = form.push(field(
                        26,
                        &self.form.display_name,
                        false,
                        Message::DisplayName,
                    ));
                }
                if matches!(screen, Screen::Change | Screen::Delete) {
                    form = form.push(field(27, &self.form.current, true, |s| {
                        Message::Current(Zeroizing::new(s))
                    }));
                }
                if screen == Screen::Recover {
                    form = form.push(field(29, &self.form.code, true, |s| {
                        Message::Code(Zeroizing::new(s))
                    }));
                }
                if screen != Screen::Delete {
                    form = form
                        .push(field(
                            if screen == Screen::Register { 5 } else { 28 },
                            &self.form.password,
                            true,
                            |s| Message::Password(Zeroizing::new(s)),
                        ))
                        .push(small(32));
                }
                form.push(
                    button(
                        text(l.line(if screen == Screen::Delete { 33 } else { key }))
                            .center()
                            .width(Length::Fill),
                    )
                    .padding([9, 12])
                    .width(Length::Fill)
                    .style(look::play(false))
                    .on_press_maybe(
                        (!self.submitting && self.ready_to_submit()).then_some(Message::Submit),
                    ),
                )
                .push(navigate(23, Screen::Menu))
                .push(notice)
            }
        };
        body.spacing(9).into()
    }
    fn view(&self) -> Element<'_, Message> {
        let language = self.language;
        let label = |key| text(language.line(key));
        let mut choices = row![].spacing(2);
        for candidate in [Language::Vi, Language::En, Language::Ko] {
            let selected = candidate == language;
            choices = choices.push(
                button(text(candidate.name()).size(13).font(if selected {
                    look::BOLD
                } else {
                    Font::DEFAULT
                }))
                .padding([8, 16])
                .style(look::segment(selected))
                .on_press(Message::Language(candidate)),
            );
        }
        let header = row![
            canvas::Canvas::new(look::Mark).width(34).height(36),
            text("BOOM").size(28).font(look::HEAVY).color(Color::WHITE),
            container(text("PLAY").size(11).font(look::BOLD))
                .padding([3, 8])
                .style(look::brand_tag),
            Space::new().width(Length::Fill),
            container(choices).padding(4).style(look::segmented)
        ]
        .spacing(10)
        .align_y(Alignment::Center);
        let mut headline = column![];
        for (i, line) in language.line(0).split('\n').enumerate() {
            headline = headline.push(
                text(line)
                    .size(58)
                    .font(look::HEAVY)
                    .line_height(1.08)
                    .color(if i == 0 { Color::WHITE } else { look::SUN }),
            );
        }
        let hero = column![
            container(label(17).size(12).font(look::BOLD).color(look::SUN))
                .padding([6, 14])
                .style(look::pill(look::SUN)),
            Space::new().height(4),
            headline,
            label(1).size(16).color(look::MUTED),
            canvas::Canvas::new(look::Arcade { tick: self.ticks })
                .width(Length::Fill)
                .height(236)
        ]
        .spacing(14)
        .width(Length::Fill);
        let status_key = match self.online {
            None => 7,
            Some(true) => 8,
            Some(false) => 9,
        };
        let busy = self.starting || self.session.is_some();
        let stop = self.session.is_some();
        let mut play = button(
            label(if stop { 15 } else { 6 })
                .size(17)
                .font(look::HEAVY)
                .width(Length::Fill)
                .center(),
        )
        .padding([17, 24])
        .width(Length::Fill)
        .style(look::play(stop));
        if stop && !self.stopping {
            play = play.on_press(Message::Stop);
        } else if !busy && self.prepared == Some(true) && self.online == Some(true) {
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
        // Trạng thái máy chủ đã có ở chân trang; dưới nút chỉ hiện tiến trình hoặc lỗi.
        let notice = if notice == status_key {
            text(" ").size(13)
        } else {
            label(notice)
                .size(13)
                .color(if matches!(notice, 11 | 12 | 16 | 24 | 34) {
                    look::CORAL
                } else {
                    look::MUTED
                })
        };
        let card = container(self.panel(play.into(), notice.into()))
            .padding(26)
            .width(410)
            .style(look::card);
        let body = row![hero, Space::new().width(36), card].align_y(Alignment::Center);
        let tint = match self.online {
            None => look::SUN,
            Some(true) => look::MINT,
            Some(false) => look::CORAL,
        };
        let footer = row![
            text("BOOM  /  PLAY").size(11).color(look::MUTED),
            Space::new().width(Length::Fill),
            container(
                row![
                    container(Space::new().width(8).height(8)).style(look::dot(tint)),
                    label(status_key).size(12).color(Color::WHITE)
                ]
                .spacing(8)
                .align_y(Alignment::Center)
            )
            .padding([6, 14])
            .style(look::pill(tint))
        ]
        .align_y(Alignment::Center);
        let content = container(column![
            header,
            container(body).center_y(Length::Fill),
            footer
        ])
        .padding([26, 44])
        .width(Length::Fill)
        .height(Length::Fill);
        container(stack![
            canvas::Canvas::new(look::Backdrop)
                .width(Length::Fill)
                .height(Length::Fill),
            content
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(look::window)
        .into()
    }
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
