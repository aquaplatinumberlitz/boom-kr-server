//! BoomServer.exe: unzip, name the server, press Run, copy the `S1-...` invite.
//! Game rules come from the bundle's prebuilt JSON; there is no game folder step.
use kr_service::self_host::{self, Action, Phase, Preferences, Settings, Status};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    ffi::{OsString, c_void},
    io::{self, Write as _},
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
    ptr::{null, null_mut},
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, watch};
use windows_sys::{
    Win32::{
        Foundation::*,
        Globalization::GetUserDefaultUILanguage,
        Graphics::Gdi::{
            CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, COLOR_BTNFACE, CreateFontW, DEFAULT_CHARSET,
            DeleteObject, FF_MODERN, FF_SWISS, FIXED_PITCH, FW_NORMAL, FW_SEMIBOLD,
            GetSysColorBrush, HDC, HFONT, InvalidateRect, OUT_DEFAULT_PRECIS, SetBkMode,
            SetTextColor, TRANSPARENT, VARIABLE_PITCH,
        },
        System::{
            Com::{
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
                CoTaskMemFree,
            },
            DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
            LibraryLoader::GetModuleHandleW,
            Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock},
            SystemInformation::{GetLocalTime, GetSystemDirectoryW},
            WindowsProgramming::MulDiv,
        },
        UI::{
            HiDpi::{AdjustWindowRectExForDpi, GetDpiForWindow, GetSystemMetricsForDpi},
            Input::KeyboardAndMouse::EnableWindow,
            Shell::*,
            WindowsAndMessaging::*,
        },
    },
    core::{GUID, HRESULT, PCWSTR, PWSTR},
};

// Controls.
const PRIMARY: i32 = 100;
const COPY: i32 = 102;
const BACKUP: i32 = 104;
const SETTINGS: i32 = 107;
const OPEN_LOG: i32 = 108;
// Settings and tray menu items.
const LANGUAGE: i32 = 200; // + 0 vi, 1 en, 2 ko
const BACKUP_FOLDER: i32 = 210;
const BACKUP_FOLDER_CLEAR: i32 = 211;
const DATA: i32 = 212;
const ROTATE: i32 = 213;
const TRAY_OPEN: i32 = 220;
const TRAY_COPY: i32 = 221;
const TRAY_EXIT: i32 = 222;

const TRAY: u32 = WM_APP + 1;
const CF_UNICODETEXT: u32 = 13;
// Win32_UI_Controls values, kept local instead of enabling the whole feature.
const SS_CENTER: u32 = 0x0001;
const SS_NOPREFIX: u32 = 0x0080;
const EM_LIMITTEXT: u32 = 0x00c5;
const IDI_APP: u16 = 1;
/// Logical client size at 100% scaling (96 DPI). The invite box spans the
/// full width and wraps, so the ~91-character `S1-...` code is never cut.
const CLIENT: (i32, i32) = (600, 306);
const STYLE: u32 = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
const STYLE_EX: u32 = WS_EX_CONTROLPARENT;
const COPIED_FOR: Duration = Duration::from_secs(2);
const LOG_LIMIT: u64 = 1024 * 1024;

const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    r as u32 | (g as u32) << 8 | (b as u32) << 16
}
const GREEN: COLORREF = rgb(0x1e, 0xa4, 0x46);
const AMBER: COLORREF = rgb(0xe0, 0x9a, 0x00);
const RED: COLORREF = rgb(0xd1, 0x34, 0x38);
const GRAY: COLORREF = rgb(0x8a, 0x8a, 0x8a);

type Text = [&'static str; 3];
const LANGUAGES: [&str; 3] = ["vi", "en", "ko"];
const RUNNING: Text = ["Đang chạy", "Running", "실행 중"];
const STOPPED: Text = ["Đã dừng", "Stopped", "중지됨"];
const STOPPING: Text = [
    "Đang dừng Server…",
    "Stopping the server…",
    "서버를 중지하는 중…",
];
const STARTING: Text = ["Đang khởi động", "Starting", "시작하는 중"];
const FAILED: Text = [
    "Không chạy được Server",
    "The server could not run",
    "서버를 실행할 수 없습니다",
];
const STEP: Text = ["Bước", "Step", "단계"];
const STEPS: [Text; 3] = [
    [
        "Kiểm tra gói và dữ liệu luật chơi",
        "Check the bundle and game data",
        "패키지와 게임 데이터 확인",
    ],
    [
        "Chuẩn bị tài khoản Host",
        "Prepare the Host account",
        "Host 계정 준비",
    ],
    ["Khởi động Server", "Start the server", "서버 시작"],
];
const LAN: Text = ["Chơi qua LAN", "LAN only", "LAN 전용"];
const RELAY_ON: Text = ["Đã kết nối relay", "Relay connected", "릴레이 연결됨"];
const RELAY_WAIT: Text = [
    "Đang kết nối relay…",
    "Connecting to relay…",
    "릴레이 연결 중…",
];
const NAME: Text = ["Tên Server", "Server name", "서버 이름"];
const INVITE: Text = ["Mã mời S1", "S1 invite", "S1 초대 코드"];
const COPY_TEXT: Text = ["Sao chép", "Copy", "복사"];
const COPIED: Text = ["Đã chép ✓", "Copied ✓", "복사됨 ✓"];
const RUN: Text = ["Chạy", "Run", "실행"];
const STOP: Text = ["Dừng", "Stop", "중지"];
const BACKUP_TEXT: Text = ["Sao lưu", "Back up", "백업"];
const SETTINGS_TEXT: Text = ["Cài đặt", "Settings", "설정"];
const LOG_TEXT: Text = ["Mở log", "Open log", "로그 열기"];
const IDLE_HINT: Text = [
    "Đặt tên Server rồi bấm Chạy. Mã mời S1 sẽ hiện ở trên khi Server sẵn sàng.",
    "Name the server and press Run. The S1 invite appears above once the server is ready.",
    "서버 이름을 정하고 실행을 누르세요. 서버가 준비되면 위에 S1 초대 코드가 나타납니다.",
];
const READY_HINT: Text = [
    "Gửi mã mời S1 cho bạn bè. Đóng cửa sổ thì Server vẫn chạy nền ở khay hệ thống.",
    "Send the S1 invite to your friends. Closing the window keeps the server running in the tray.",
    "친구에게 S1 초대 코드를 보내세요. 창을 닫아도 서버는 트레이에서 계속 실행됩니다.",
];
const BACKED_UP: Text = ["Đã sao lưu lúc", "Backed up at", "백업 시각"];
const BACKUP_FAILED: Text = ["Sao lưu lỗi", "Backup failed", "백업 실패"];
const LOG_HINT: Text = [
    "Chi tiết đầy đủ nằm trong Log. Bạn có thể bôi đen và sao chép nội dung ở đây.",
    "Full details are in the log. You can select and copy this text.",
    "자세한 내용은 로그에 있습니다. 이 내용을 선택해 복사할 수 있습니다.",
];
const TRAY_TITLE: Text = [
    "BoomServer vẫn đang chạy",
    "BoomServer is still running",
    "BoomServer가 계속 실행 중입니다",
];
const TRAY_BODY: Text = [
    "Server chạy nền ở khay hệ thống. Bấm biểu tượng để mở lại; chuột phải để sao chép mã mời hoặc Dừng & Thoát.",
    "The server keeps running in the tray. Click the icon to reopen it; right-click to copy the invite or Stop & Exit.",
    "서버가 트레이에서 계속 실행됩니다. 아이콘을 클릭해 다시 열고, 오른쪽 클릭으로 초대 코드 복사나 중지 후 종료를 할 수 있습니다.",
];

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
    // Read while painting, which can happen during an `App` update.
    static DOT: Cell<COLORREF> = const { Cell::new(GRAY) };
    static DOT_HWND: Cell<HWND> = const { Cell::new(null_mut()) };
}

#[derive(Clone, Copy)]
enum Font {
    Ui,
    Bold,
    Mono,
    Dot,
}

struct Fonts {
    ui: HFONT,
    bold: HFONT,
    mono: HFONT,
    dot: HFONT,
}

impl Fonts {
    unsafe fn new(dpi: u32) -> Self {
        unsafe {
            Self {
                ui: font(9, FW_NORMAL, "Segoe UI", dpi, false),
                bold: font(10, FW_SEMIBOLD, "Segoe UI", dpi, false),
                mono: font(10, FW_NORMAL, "Consolas", dpi, true),
                dot: font(15, FW_NORMAL, "Segoe UI", dpi, false),
            }
        }
    }

    fn get(&self, kind: Font) -> HFONT {
        match kind {
            Font::Ui => self.ui,
            Font::Bold => self.bold,
            Font::Mono => self.mono,
            Font::Dot => self.dot,
        }
    }

    unsafe fn delete(&self) {
        for font in [self.ui, self.bold, self.mono, self.dot] {
            if !font.is_null() {
                unsafe { DeleteObject(font) };
            }
        }
    }
}

unsafe fn font(points: i32, weight: u32, face: &str, dpi: u32, mono: bool) -> HFONT {
    let pitch = if mono {
        FIXED_PITCH | FF_MODERN
    } else {
        VARIABLE_PITCH | FF_SWISS
    };
    unsafe {
        CreateFontW(
            -MulDiv(points, dpi as i32, 72),
            0,
            0,
            0,
            weight as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            CLEARTYPE_QUALITY as u32,
            pitch as u32,
            text(face).as_ptr(),
        )
    }
}

struct Control {
    hwnd: HWND,
    rect: (i32, i32, i32, i32),
    font: Font,
}

struct App {
    root: PathBuf,
    state: PathBuf,
    settings: Option<Settings>,
    preferences: Preferences,
    lang: usize,
    run: Option<Run>,
    error: String,
    failed_phase: Phase,
    tray: bool,
    tray_tip: String,
    exit_after_stop: bool,
    dpi: u32,
    fonts: Option<Fonts>,
    icons: (HICON, HICON),
    controls: Vec<Control>,
    texts: HashMap<usize, Vec<u16>>,
    enabled: HashMap<usize, bool>,
    copied_at: Option<Instant>,
    backup_seen: u64,
    backup_note: Option<(Result<PathBuf, String>, String)>,
    logged_phase: Phase,
    name_label: HWND,
    name: HWND,
    status: HWND,
    counts: HWND,
    invite_label: HWND,
    invite: HWND,
    copy: HWND,
    primary: HWND,
    backup: HWND,
    settings_button: HWND,
    message: HWND,
    log_button: HWND,
}

struct Run {
    send: mpsc::Sender<Action>,
    status: watch::Receiver<Status>,
    done: std::sync::mpsc::Receiver<Result<(), String>>,
    stopping: bool,
}

fn text(s: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    s.as_ref().encode_wide().chain(Some(0)).collect()
}

fn system_language() -> usize {
    match unsafe { GetUserDefaultUILanguage() } & 0x03ff {
        0x2a => 0,
        0x12 => 2,
        _ => 1,
    }
}

fn now() -> SYSTEMTIME {
    let mut time: SYSTEMTIME = unsafe { std::mem::zeroed() };
    unsafe { GetLocalTime(&mut time) };
    time
}

fn local_time() -> String {
    let time = now();
    format!("{:02}:{:02}:{:02}", time.wHour, time.wMinute, time.wSecond)
}

fn local_stamp() -> String {
    let time = now();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        time.wYear, time.wMonth, time.wDay, time.wHour, time.wMinute, time.wSecond
    )
}

impl App {
    fn new() -> io::Result<Self> {
        let root = std::env::current_exe()?
            .parent()
            .ok_or_else(|| io::Error::other("bundle path"))?
            .to_owned();
        let local = std::env::var_os("LOCALAPPDATA")
            .ok_or_else(|| io::Error::other("LOCALAPPDATA missing"))?;
        let state = PathBuf::from(local).join("BoomServer");
        self_host::state_dir(&state)?;
        let settings = self_host::settings(&state)?;
        // A damaged preferences file must not block the server.
        let preferences = self_host::preferences(&state).unwrap_or_default();
        let lang = preferences
            .language
            .as_deref()
            .and_then(|code| LANGUAGES.iter().position(|l| *l == code))
            .unwrap_or_else(system_language);
        Ok(Self {
            root,
            state,
            settings,
            preferences,
            lang,
            run: None,
            error: String::new(),
            failed_phase: Phase::Idle,
            tray: false,
            tray_tip: String::new(),
            exit_after_stop: false,
            dpi: 96,
            fonts: None,
            icons: (null_mut(), null_mut()),
            controls: Vec::new(),
            texts: HashMap::new(),
            enabled: HashMap::new(),
            copied_at: None,
            backup_seen: 0,
            backup_note: None,
            logged_phase: Phase::Idle,
            name_label: null_mut(),
            name: null_mut(),
            status: null_mut(),
            counts: null_mut(),
            invite_label: null_mut(),
            invite: null_mut(),
            copy: null_mut(),
            primary: null_mut(),
            backup: null_mut(),
            settings_button: null_mut(),
            message: null_mut(),
            log_button: null_mut(),
        })
    }

    fn tr(&self, x: Text) -> &'static str {
        x[self.lang]
    }

    fn log_path(&self) -> PathBuf {
        self.state.join("boomserver.log")
    }

    /// Append one line to `boomserver.log` (rotated once it reaches 1 MiB).
    fn log(&self, line: &str) {
        let path = self.log_path();
        if std::fs::metadata(&path).is_ok_and(|meta| meta.len() > LOG_LIMIT) {
            let _ = std::fs::rename(&path, self.state.join("boomserver.old.log"));
        }
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(file, "{} {line}", local_stamp());
        }
    }

    fn scale(&self, value: i32) -> i32 {
        unsafe { MulDiv(value, self.dpi as i32, 96) }
    }

    unsafe fn add(
        &mut self,
        hwnd: HWND,
        class: &str,
        rect: (i32, i32, i32, i32),
        id: i32,
        style: u32,
        font: Font,
    ) -> HWND {
        let handle = unsafe {
            CreateWindowExW(
                0,
                text(class).as_ptr(),
                text("").as_ptr(),
                WS_CHILD | WS_VISIBLE | style,
                0,
                0,
                0,
                0,
                hwnd,
                id as isize as HMENU,
                GetModuleHandleW(null()),
                null(),
            )
        };
        self.controls.push(Control {
            hwnd: handle,
            rect,
            font,
        });
        handle
    }

    /// Position every control and font for the window's current DPI.
    unsafe fn layout(&mut self, hwnd: HWND) {
        unsafe {
            if let Some(old) = self.fonts.take() {
                old.delete();
            }
            let fonts = Fonts::new(self.dpi);
            for control in &self.controls {
                let (x, y, w, h) = control.rect;
                MoveWindow(
                    control.hwnd,
                    self.scale(x),
                    self.scale(y),
                    self.scale(w),
                    self.scale(h),
                    1,
                );
                SendMessageW(
                    control.hwnd,
                    WM_SETFONT,
                    fonts.get(control.font) as usize,
                    1,
                );
            }
            self.fonts = Some(fonts);
            self.load_icons(hwnd);
        }
    }

    unsafe fn load_icons(&mut self, hwnd: HWND) {
        unsafe {
            let instance = GetModuleHandleW(null());
            let big = app_icon(instance, GetSystemMetricsForDpi(SM_CXICON, self.dpi));
            let small = app_icon(instance, GetSystemMetricsForDpi(SM_CXSMICON, self.dpi));
            SendMessageW(hwnd, WM_SETICON, ICON_BIG as usize, big as isize);
            SendMessageW(hwnd, WM_SETICON, ICON_SMALL as usize, small as isize);
            for old in [self.icons.0, self.icons.1] {
                if !old.is_null() {
                    DestroyIcon(old);
                }
            }
            self.icons = (big, small);
            if self.tray {
                tray(hwnd, NIM_MODIFY, small, None);
            }
        }
    }

    unsafe fn build(&mut self, hwnd: HWND) {
        const EDIT: u32 = WS_BORDER | WS_TABSTOP | ES_AUTOHSCROLL as u32;
        const READONLY_MULTILINE: u32 = WS_BORDER | ES_MULTILINE as u32 | ES_READONLY as u32;
        let push = BS_PUSHBUTTON as u32 | WS_TABSTOP;
        unsafe {
            self.dpi = GetDpiForWindow(hwnd).max(96);
            // Block 1: status light, state and online/room counts.
            let dot = self.add(hwnd, "STATIC", (16, 10, 26, 34), 0, SS_CENTER, Font::Dot);
            DOT_HWND.set(dot);
            self.status = self.add(
                hwnd,
                "STATIC",
                (46, 12, 538, 22),
                0,
                SS_NOPREFIX,
                Font::Bold,
            );
            self.counts = self.add(hwnd, "STATIC", (46, 35, 538, 20), 0, SS_NOPREFIX, Font::Ui);
            // Block 2: server name.
            self.name_label = self.add(hwnd, "STATIC", (16, 71, 110, 20), 0, 0, Font::Ui);
            self.name = self.add(hwnd, "EDIT", (130, 67, 454, 26), 0, EDIT, Font::Ui);
            SendMessageW(self.name, EM_LIMITTEXT, 128, 0);
            // Block 3: invite in a full-width monospace box, with Copy.
            self.invite_label = self.add(hwnd, "STATIC", (16, 108, 330, 20), 0, 0, Font::Ui);
            self.copy = self.add(hwnd, "BUTTON", (464, 103, 120, 28), COPY, push, Font::Ui);
            self.invite = self.add(
                hwnd,
                "EDIT",
                (16, 135, 568, 46),
                0,
                READONLY_MULTILINE,
                Font::Mono,
            );
            // Block 4: Run/Stop · Back up · Settings.
            self.primary = self.add(hwnd, "BUTTON", (16, 193, 132, 32), PRIMARY, push, Font::Ui);
            self.backup = self.add(hwnd, "BUTTON", (156, 193, 132, 32), BACKUP, push, Font::Ui);
            self.settings_button = self.add(
                hwnd,
                "BUTTON",
                (296, 193, 132, 32),
                SETTINGS,
                push,
                Font::Ui,
            );
            // Feedback: steps, backup result or the full error text, with the log.
            self.message = self.add(
                hwnd,
                "EDIT",
                (16, 237, 440, 56),
                0,
                READONLY_MULTILINE | WS_VSCROLL,
                Font::Ui,
            );
            self.log_button = self.add(
                hwnd,
                "BUTTON",
                (464, 237, 120, 28),
                OPEN_LOG,
                push,
                Font::Ui,
            );
            let name = self
                .settings
                .as_ref()
                .map(|s| s.name.clone())
                .unwrap_or_else(|| "Boom Server".into());
            SetWindowTextW(self.name, text(&name).as_ptr());
            self.layout(hwnd);
            self.resize(hwnd);
            self.refresh(hwnd);
            if self.settings.is_some() {
                PostMessageW(hwnd, WM_COMMAND, PRIMARY as usize, 0);
            }
        }
    }

    /// Size the window so its client area is `CLIENT` logical pixels.
    unsafe fn resize(&self, hwnd: HWND) {
        unsafe {
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: self.scale(CLIENT.0),
                bottom: self.scale(CLIENT.1),
            };
            AdjustWindowRectExForDpi(&mut rect, STYLE, 0, STYLE_EX, self.dpi);
            SetWindowPos(
                hwnd,
                null_mut(),
                0,
                0,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }

    unsafe fn dpi_changed(&mut self, hwnd: HWND, dpi: u32, suggested: *const RECT) {
        unsafe {
            self.dpi = dpi.max(96);
            if let Some(rect) = suggested.as_ref() {
                SetWindowPos(
                    hwnd,
                    null_mut(),
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            self.layout(hwnd);
            self.resize(hwnd);
        }
    }

    /// Only touch a control when its text really changes (no 300 ms flicker).
    unsafe fn set(&mut self, hwnd: HWND, value: &str) {
        let wide = text(value);
        if self.texts.get(&(hwnd as usize)) != Some(&wide) {
            unsafe { SetWindowTextW(hwnd, wide.as_ptr()) };
            self.texts.insert(hwnd as usize, wide);
        }
    }

    unsafe fn enable(&mut self, hwnd: HWND, value: bool) {
        if self.enabled.get(&(hwnd as usize)) != Some(&value) {
            unsafe { EnableWindow(hwnd, i32::from(value)) };
            self.enabled.insert(hwnd as usize, value);
        }
    }

    unsafe fn start(&mut self, hwnd: HWND) -> io::Result<()> {
        let settings = Settings {
            name: unsafe { read(self.name) }
                .to_string_lossy()
                .trim()
                .to_owned(),
            // Bundled pack-data/ is the default; BOOM_GAME is a dev override.
            game: self_host::game_from_environment(),
        };
        self_host::save_settings(&self.state, &settings)?;
        let bind = self_host::bind_from_environment()?;
        let network = self_host::network_from_environment()?;
        if matches!(network, kr_p2p::self_host::Network::Public) {
            unsafe {
                self.firewall(hwnd, bind.port());
            }
        }
        let root = self.root.clone();
        let state = self.state.clone();
        let (send, recv) = mpsc::channel(4);
        let (status_tx, status) = watch::channel(Status::default());
        let (result, done) = std::sync::mpsc::sync_channel(1);
        self.log(&format!("start: server name {:?}", settings.name));
        std::thread::Builder::new()
            .name("boom-s1".into())
            .spawn(move || {
                let outcome = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(4)
                    .enable_all()
                    .build()
                    .and_then(|runtime| {
                        runtime.block_on(self_host::listen(
                            root, state, settings, bind, network, recv, status_tx,
                        ))
                    });
                let _ = result.send(outcome.map_err(|e| e.to_string()));
            })?;
        self.settings = self_host::settings(&self.state)?;
        self.error.clear();
        self.failed_phase = Phase::Idle;
        self.backup_seen = 0;
        self.logged_phase = Phase::Idle;
        self.run = Some(Run {
            send,
            status,
            done,
            stopping: false,
        });
        Ok(())
    }

    unsafe fn firewall(&self, hwnd: HWND, port: u16) {
        let marker = self.state.join("firewall-asked");
        if marker.exists() {
            return;
        }
        let question = match self.lang {
            0 => format!(
                "Cho phép người chơi kết nối thẳng tới Server qua UDP (Port {port}) trên mạng Private/LAN? Windows Firewall sẽ hỏi quyền Admin. Chọn Không thì Server vẫn chạy và dùng relay."
            ),
            2 => format!(
                "개인 네트워크(LAN)에서 UDP(Port {port})로 서버에 직접 연결하도록 허용할까요? Windows Firewall이 Admin 권한을 요청합니다. 아니요를 선택해도 서버는 relay로 실행됩니다."
            ),
            _ => format!(
                "Allow players to connect directly over UDP (Port {port}) on Private/LAN networks? Windows Firewall will ask for Admin permission. If you choose No, the server still runs through the relay."
            ),
        };
        let yes = unsafe {
            MessageBoxW(
                hwnd,
                text(question).as_ptr(),
                text("BoomServer").as_ptr(),
                MB_YESNO | MB_ICONQUESTION | MB_DEFBUTTON2,
            ) == IDYES
        };
        if yes
            && let Ok(program) = std::env::current_exe()
            && let Some(netsh) = unsafe { system_netsh() }
        {
            let rule = format!(
                "advfirewall firewall add rule name=\"BoomServer S1\" dir=in action=allow protocol=UDP localport={port} profile=private program=\"{}\"",
                program.display()
            );
            unsafe {
                ShellExecuteW(
                    hwnd,
                    text("runas").as_ptr(),
                    text(netsh.as_os_str()).as_ptr(),
                    text(rule).as_ptr(),
                    null(),
                    SW_HIDE,
                );
            }
        }
        let _ = std::fs::write(marker, b"asked\n");
    }

    fn step_text(&self, phase: Phase) -> String {
        let current = phase.step().unwrap_or(0);
        let mut lines = Vec::new();
        for (index, label) in STEPS.iter().enumerate() {
            let number = index as u8 + 1;
            let mark = if number < current {
                "✓"
            } else if number == current {
                "▶"
            } else {
                "·"
            };
            lines.push(format!(
                "{mark} {} {number}/{}: {}",
                self.tr(STEP),
                Phase::STEPS,
                self.tr(*label)
            ));
        }
        lines.join("\r\n")
    }

    fn backup_text(&self) -> Option<String> {
        let (result, at) = self.backup_note.as_ref()?;
        Some(match result {
            Ok(path) => format!(
                "{} {at}: {}\r\n{}",
                self.tr(BACKED_UP),
                path.file_name().map_or_else(
                    || path.display().to_string(),
                    |n| n.to_string_lossy().into()
                ),
                path.parent()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
            ),
            Err(error) => format!("{} ({at}): {error}", self.tr(BACKUP_FAILED)),
        })
    }

    unsafe fn refresh(&mut self, hwnd: HWND) {
        if let Some(run) = &self.run {
            let phase = run.status.borrow().phase;
            match run.done.try_recv() {
                Ok(result) => {
                    if let Err(error) = result {
                        self.failed_phase = phase;
                        self.error = error;
                        self.log(&format!("error: {}", self.error));
                    } else {
                        self.log("stopped");
                    }
                    self.run = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.failed_phase = phase;
                    self.error = "server thread ended unexpectedly".into();
                    self.log(&format!("error: {}", self.error));
                    self.run = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
        let status = self
            .run
            .as_ref()
            .map(|run| run.status.borrow().clone())
            .unwrap_or_default();
        if status.phase != self.logged_phase {
            self.logged_phase = status.phase;
            if let Some(step) = status.phase.step() {
                self.log(&format!(
                    "step {step}/{}: {}",
                    Phase::STEPS,
                    status.phase.label()
                ));
            } else if status.phase == Phase::Online {
                self.log("online");
            }
        }
        if status.backups != self.backup_seen {
            self.backup_seen = status.backups;
            if let Some(result) = status.backup.clone() {
                match &result {
                    Ok(path) => self.log(&format!("backup: {}", path.display())),
                    Err(error) => self.log(&format!("backup failed: {error}")),
                }
                self.backup_note = Some((result, local_time()));
            }
        }
        let stopping = self.run.as_ref().is_some_and(|run| run.stopping);
        let running = self.run.is_some();
        let (color, headline) = if !self.error.is_empty() {
            (RED, self.tr(FAILED).to_owned())
        } else if stopping {
            (AMBER, self.tr(STOPPING).to_owned())
        } else if running && !status.ready {
            let step = status.phase.step().unwrap_or(1);
            (
                AMBER,
                format!(
                    "{} ({} {step}/{})…",
                    self.tr(STARTING),
                    self.tr(STEP),
                    Phase::STEPS
                ),
            )
        } else if status.ready {
            let (color, path) = if !status.relay_expected {
                (GREEN, self.tr(LAN))
            } else if status.relay_online {
                (GREEN, self.tr(RELAY_ON))
            } else {
                (AMBER, self.tr(RELAY_WAIT))
            };
            (color, format!("{} · {path}", self.tr(RUNNING)))
        } else {
            (GRAY, self.tr(STOPPED).to_owned())
        };
        let message = if !self.error.is_empty() {
            let step = self
                .failed_phase
                .step()
                .map(|step| {
                    format!(
                        " — {} {step}/{}: {}",
                        self.tr(STEP),
                        Phase::STEPS,
                        self.tr(STEPS[usize::from(step) - 1])
                    )
                })
                .unwrap_or_default();
            format!(
                "{}{step}\r\n{}\r\n{}",
                self.tr(FAILED),
                self.error.replace('\n', "\r\n").replace("\r\r\n", "\r\n"),
                self.tr(LOG_HINT)
            )
        } else if running && !status.ready && !stopping {
            self.step_text(status.phase)
        } else if let Some(note) = self.backup_text() {
            note
        } else if status.ready {
            self.tr(READY_HINT).into()
        } else {
            self.tr(IDLE_HINT).into()
        };
        let copied = self.copied_at.is_some_and(|at| at.elapsed() < COPIED_FOR);
        let counts = match self.lang {
            0 => format!("Online: {} người · Phòng: {}", status.players, status.rooms),
            2 => format!("접속: {}명 · 방: {}", status.players, status.rooms),
            _ => format!("Online: {} · Rooms: {}", status.players, status.rooms),
        };
        unsafe {
            if DOT.get() != color {
                DOT.set(color);
                InvalidateRect(DOT_HWND.get(), null(), 1);
            }
            self.set(DOT_HWND.get(), "●");
            self.set(self.status, &headline);
            self.set(self.counts, &counts);
            self.set(self.name_label, self.tr(NAME));
            self.set(self.invite_label, self.tr(INVITE));
            self.set(self.invite, &status.invite);
            self.set(self.copy, self.tr(if copied { COPIED } else { COPY_TEXT }));
            self.set(self.primary, self.tr(if running { STOP } else { RUN }));
            self.set(self.backup, self.tr(BACKUP_TEXT));
            self.set(self.settings_button, self.tr(SETTINGS_TEXT));
            self.set(self.log_button, self.tr(LOG_TEXT));
            self.set(self.message, &message);
            self.enable(self.copy, status.ready);
            self.enable(self.backup, status.ready);
            self.enable(self.primary, !stopping);
            self.enable(self.name, !running);
            if self.tray {
                let tip = if status.ready {
                    format!("BoomServer · {}", self.tr(RUNNING))
                } else {
                    format!("BoomServer · {headline}")
                };
                if self.tray_tip != tip {
                    self.tray_tip = tip;
                    tray_tip(hwnd, &self.tray_tip);
                }
            }
        }
    }

    unsafe fn copy_invite(&mut self, hwnd: HWND) {
        if let Some(run) = &self.run {
            let status = run.status.borrow();
            if status.ready {
                unsafe { clipboard(hwnd, &status.invite) };
                drop(status);
                self.copied_at = Some(Instant::now());
            }
        }
    }

    fn stop(&mut self) -> bool {
        match &mut self.run {
            Some(run) if run.stopping => true,
            Some(run) => {
                if run.send.try_send(Action::Stop).is_ok() {
                    run.stopping = true;
                    self.log("stop requested");
                }
                true
            }
            None => false,
        }
    }

    fn save_preferences(&mut self) {
        if let Err(error) = self_host::save_preferences(&self.state, &self.preferences) {
            self.log(&format!("settings.json: {error}"));
        }
    }

    unsafe fn settings_menu(&mut self, hwnd: HWND) {
        unsafe {
            let menu = CreatePopupMenu();
            let languages = CreatePopupMenu();
            for (index, label) in ["Tiếng Việt", "English", "한국어"].iter().enumerate() {
                let checked = if index == self.lang { MF_CHECKED } else { 0 };
                AppendMenuW(
                    languages,
                    MF_STRING | checked,
                    (LANGUAGE + index as i32) as usize,
                    text(label).as_ptr(),
                );
            }
            let language = self.tr([
                "Ngôn ngữ / Language",
                "Language / Ngôn ngữ",
                "언어 / Language",
            ]);
            AppendMenuW(menu, MF_POPUP, languages as usize, text(language).as_ptr());
            AppendMenuW(menu, MF_SEPARATOR, 0, null());
            let folder = match &self.preferences.backup_folder {
                Some(path) => format!(
                    "{} ({})",
                    self.tr(["Thư mục sao lưu…", "Backup folder…", "백업 폴더…"]),
                    path.display()
                ),
                None => self
                    .tr([
                        "Chọn thư mục sao lưu…",
                        "Choose backup folder…",
                        "백업 폴더 선택…",
                    ])
                    .into(),
            };
            AppendMenuW(
                menu,
                MF_STRING,
                BACKUP_FOLDER as usize,
                text(folder).as_ptr(),
            );
            if self.preferences.backup_folder.is_some() {
                let clear = self.tr([
                    "Chỉ lưu trong thư mục dữ liệu",
                    "Keep backups in the data folder only",
                    "데이터 폴더에만 백업",
                ]);
                AppendMenuW(
                    menu,
                    MF_STRING,
                    BACKUP_FOLDER_CLEAR as usize,
                    text(clear).as_ptr(),
                );
            }
            let data = self.tr(["Mở thư mục dữ liệu", "Open data folder", "데이터 폴더 열기"]);
            AppendMenuW(menu, MF_STRING, DATA as usize, text(data).as_ptr());
            AppendMenuW(menu, MF_SEPARATOR, 0, null());
            let ready = self
                .run
                .as_ref()
                .is_some_and(|run| run.status.borrow().ready);
            let rotate = self.tr(["Tạo mã mời mới…", "New invite code…", "새 초대 코드…"]);
            AppendMenuW(
                menu,
                MF_STRING | if ready { 0 } else { MF_GRAYED },
                ROTATE as usize,
                text(rotate).as_ptr(),
            );
            let mut rect: RECT = std::mem::zeroed();
            GetWindowRect(self.settings_button, &mut rect);
            TrackPopupMenu(
                menu,
                TPM_LEFTALIGN | TPM_TOPALIGN,
                rect.left,
                rect.bottom,
                0,
                hwnd,
                null(),
            );
            DestroyMenu(menu);
        }
    }

    unsafe fn tray_menu(&mut self, hwnd: HWND) {
        unsafe {
            let menu = CreatePopupMenu();
            let ready = self
                .run
                .as_ref()
                .is_some_and(|run| run.status.borrow().ready);
            let open = self.tr(["Mở cửa sổ", "Open window", "창 열기"]);
            let copy = self.tr(["Sao chép mã mời", "Copy invite", "초대 코드 복사"]);
            let exit = self.tr(["Dừng & Thoát", "Stop & Exit", "중지 후 종료"]);
            AppendMenuW(menu, MF_STRING, TRAY_OPEN as usize, text(open).as_ptr());
            AppendMenuW(
                menu,
                MF_STRING | if ready { 0 } else { MF_GRAYED },
                TRAY_COPY as usize,
                text(copy).as_ptr(),
            );
            AppendMenuW(menu, MF_SEPARATOR, 0, null());
            AppendMenuW(menu, MF_STRING, TRAY_EXIT as usize, text(exit).as_ptr());
            SetMenuDefaultItem(menu, TRAY_OPEN as u32, 0);
            let mut point = POINT { x: 0, y: 0 };
            GetCursorPos(&mut point);
            // Required so the menu closes when the user clicks elsewhere.
            SetForegroundWindow(hwnd);
            TrackPopupMenu(
                menu,
                TPM_RIGHTBUTTON | TPM_BOTTOMALIGN,
                point.x,
                point.y,
                0,
                hwnd,
                null(),
            );
            PostMessageW(hwnd, WM_NULL, 0, 0);
            DestroyMenu(menu);
        }
    }

    unsafe fn hide_to_tray(&mut self, hwnd: HWND) {
        unsafe {
            if !self.tray {
                let hint = (!self.preferences.tray_hint_shown)
                    .then(|| (self.tr(TRAY_TITLE), self.tr(TRAY_BODY)));
                self.tray = tray(hwnd, NIM_ADD, self.icons.1, hint);
                if self.tray {
                    self.tray_tip.clear();
                }
                if self.tray && hint.is_some() {
                    self.preferences.tray_hint_shown = true;
                    self.save_preferences();
                }
            }
            ShowWindow(hwnd, if self.tray { SW_HIDE } else { SW_MINIMIZE });
        }
    }

    unsafe fn restore(&mut self, hwnd: HWND) {
        unsafe {
            ShowWindow(hwnd, SW_SHOW);
            ShowWindow(hwnd, SW_RESTORE);
            SetForegroundWindow(hwnd);
            if self.tray {
                tray(hwnd, NIM_DELETE, null_mut(), None);
                self.tray = false;
                self.tray_tip.clear();
            }
        }
    }

    unsafe fn open(&self, hwnd: HWND, target: &Path) {
        unsafe {
            ShellExecuteW(
                hwnd,
                text("open").as_ptr(),
                text(target.as_os_str()).as_ptr(),
                null(),
                null(),
                SW_SHOWNORMAL,
            );
        }
    }

    unsafe fn command(&mut self, hwnd: HWND, id: i32) {
        unsafe {
            match id {
                PRIMARY => {
                    if !self.stop()
                        && let Err(error) = self.start(hwnd)
                    {
                        self.failed_phase = Phase::Idle;
                        self.error = error.to_string();
                        self.log(&format!("error: {}", self.error));
                    }
                }
                COPY | TRAY_COPY => self.copy_invite(hwnd),
                BACKUP => {
                    if let Some(run) = &self.run {
                        let folder = self.preferences.backup_folder.clone();
                        let _ = run.send.try_send(Action::Backup(folder));
                    }
                }
                SETTINGS => self.settings_menu(hwnd),
                OPEN_LOG => {
                    let log = self.log_path();
                    let target = if log.is_file() {
                        log
                    } else {
                        self.state.clone()
                    };
                    self.open(hwnd, &target);
                }
                DATA => self.open(hwnd, &self.state.clone()),
                BACKUP_FOLDER => {
                    let title = self.tr([
                        "Chọn thư mục nhận bản sao lưu",
                        "Choose a folder for backup copies",
                        "백업 사본을 저장할 폴더 선택",
                    ]);
                    if let Some(path) =
                        pick_folder(hwnd, title, self.preferences.backup_folder.as_deref())
                    {
                        self.preferences.backup_folder = Some(path);
                        self.save_preferences();
                    }
                }
                BACKUP_FOLDER_CLEAR => {
                    self.preferences.backup_folder = None;
                    self.save_preferences();
                }
                ROTATE => {
                    let question = self.tr([
                        "Mã mời cũ sẽ hết hiệu lực và người đang chơi bị ngắt kết nối. Tạo mã mời mới?",
                        "The old invite stops working and current players are disconnected. Create a new invite?",
                        "기존 초대 코드는 무효가 되고 플레이어 연결이 끊깁니다. 새 초대 코드를 만들까요?",
                    ]);
                    if MessageBoxW(
                        hwnd,
                        text(question).as_ptr(),
                        text("BoomServer").as_ptr(),
                        MB_YESNO | MB_DEFBUTTON2 | MB_ICONQUESTION,
                    ) == IDYES
                        && let Some(run) = &self.run
                    {
                        let _ = run.send.try_send(Action::Rotate);
                        self.log("new invite requested");
                    }
                }
                TRAY_OPEN => self.restore(hwnd),
                TRAY_EXIT => {
                    if self.stop() {
                        self.exit_after_stop = true;
                    } else {
                        DestroyWindow(hwnd);
                    }
                }
                id if (LANGUAGE..LANGUAGE + 3).contains(&id) => {
                    self.lang = (id - LANGUAGE) as usize;
                    self.preferences.language = Some(LANGUAGES[self.lang].into());
                    self.save_preferences();
                }
                _ => {}
            }
            self.refresh(hwnd);
        }
    }
}

unsafe fn app_icon(instance: HINSTANCE, size: i32) -> HICON {
    unsafe {
        // MAKEINTRESOURCEW(IDI_APP) from boom-server.rc.
        let icon = LoadImageW(
            instance,
            IDI_APP as usize as PCWSTR,
            IMAGE_ICON,
            size,
            size,
            LR_DEFAULTCOLOR,
        );
        if icon.is_null() {
            LoadIconW(null_mut(), IDI_APPLICATION)
        } else {
            icon as HICON
        }
    }
}

unsafe fn read(hwnd: HWND) -> OsString {
    unsafe {
        let mut data = vec![0; (GetWindowTextLengthW(hwnd) + 1) as usize];
        let size = data.len() as i32;
        let count = GetWindowTextW(hwnd, data.as_mut_ptr(), size);
        OsString::from_wide(&data[..count as usize])
    }
}

unsafe fn system_netsh() -> Option<PathBuf> {
    let mut dir = vec![0; 32768];
    let len = unsafe { GetSystemDirectoryW(dir.as_mut_ptr(), dir.len() as u32) } as usize;
    (len > 0 && len < dir.len())
        .then(|| PathBuf::from(OsString::from_wide(&dir[..len])).join("netsh.exe"))
}

// Minimal COM vtables for the Vista+ folder picker (windows-sys has no COM
// interfaces). Slots this program does not call are kept as plain pointers.
#[repr(C)]
struct UnknownVtbl {
    query_interface: usize,
    add_ref: usize,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
}

#[repr(C)]
struct FileDialogVtbl {
    base: UnknownVtbl,
    show: unsafe extern "system" fn(*mut c_void, HWND) -> HRESULT,
    set_file_types: usize,
    set_file_type_index: usize,
    get_file_type_index: usize,
    advise: usize,
    unadvise: usize,
    set_options: unsafe extern "system" fn(*mut c_void, u32) -> HRESULT,
    get_options: unsafe extern "system" fn(*mut c_void, *mut u32) -> HRESULT,
    set_default_folder: usize,
    set_folder: unsafe extern "system" fn(*mut c_void, *mut c_void) -> HRESULT,
    get_folder: usize,
    get_current_selection: usize,
    set_file_name: usize,
    get_file_name: usize,
    set_title: unsafe extern "system" fn(*mut c_void, PCWSTR) -> HRESULT,
    set_ok_button_label: usize,
    set_file_name_label: usize,
    get_result: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
}

#[repr(C)]
struct ShellItemVtbl {
    base: UnknownVtbl,
    bind_to_handler: usize,
    get_parent: usize,
    get_display_name: unsafe extern "system" fn(*mut c_void, SIGDN, *mut PWSTR) -> HRESULT,
}

const IID_IFILE_OPEN_DIALOG: GUID = GUID::from_u128(0xd57c7288_d4ad_4768_be02_9d969532d960);
const IID_ISHELL_ITEM: GUID = GUID::from_u128(0x43826d1e_e718_42ee_bc55_a1e261c37bfe);

/// Releases one COM reference when dropped.
struct Com(*mut c_void);
impl Drop for Com {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let vtbl = *(self.0 as *const *const UnknownVtbl);
                ((*vtbl).release)(self.0);
            }
        }
    }
}

/// `IFileOpenDialog` with `FOS_PICKFOLDERS` (replaces `SHBrowseForFolderW`).
unsafe fn pick_folder(hwnd: HWND, title: &str, start: Option<&Path>) -> Option<PathBuf> {
    unsafe {
        let mut raw = null_mut();
        if CoCreateInstance(
            &FileOpenDialog,
            null_mut(),
            CLSCTX_INPROC_SERVER,
            &IID_IFILE_OPEN_DIALOG,
            &mut raw,
        ) < 0
            || raw.is_null()
        {
            return None;
        }
        let dialog = Com(raw);
        let vtbl = &**(dialog.0 as *const *const FileDialogVtbl);
        let mut options = 0;
        if (vtbl.get_options)(dialog.0, &mut options) < 0 {
            return None;
        }
        let options = options | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST;
        if (vtbl.set_options)(dialog.0, options) < 0 {
            return None;
        }
        let title = text(title);
        (vtbl.set_title)(dialog.0, title.as_ptr());
        if let Some(start) = start {
            let mut folder = null_mut();
            let path = text(start.as_os_str());
            if SHCreateItemFromParsingName(path.as_ptr(), null_mut(), &IID_ISHELL_ITEM, &mut folder)
                >= 0
                && !folder.is_null()
            {
                let folder = Com(folder);
                (vtbl.set_folder)(dialog.0, folder.0);
            }
        }
        // Cancel returns HRESULT_FROM_WIN32(ERROR_CANCELLED).
        if (vtbl.show)(dialog.0, hwnd) < 0 {
            return None;
        }
        let mut item = null_mut();
        if (vtbl.get_result)(dialog.0, &mut item) < 0 || item.is_null() {
            return None;
        }
        let item = Com(item);
        let item_vtbl = &**(item.0 as *const *const ShellItemVtbl);
        let mut name: PWSTR = null_mut();
        if (item_vtbl.get_display_name)(item.0, SIGDN_FILESYSPATH, &mut name) < 0 || name.is_null()
        {
            return None;
        }
        let mut len = 0;
        while *name.add(len) != 0 {
            len += 1;
        }
        let path = PathBuf::from(OsString::from_wide(std::slice::from_raw_parts(name, len)));
        CoTaskMemFree(name.cast());
        Some(path)
    }
}

unsafe fn clipboard(hwnd: HWND, code: &str) {
    unsafe {
        if OpenClipboard(hwnd) == 0 {
            return;
        }
        let code = text(code);
        let memory = GlobalAlloc(GMEM_MOVEABLE, code.len() * 2);
        if !memory.is_null() {
            let destination = GlobalLock(memory);
            if !destination.is_null() {
                std::ptr::copy_nonoverlapping(code.as_ptr(), destination.cast::<u16>(), code.len());
                GlobalUnlock(memory);
            }
            if destination.is_null()
                || EmptyClipboard() == 0
                || SetClipboardData(CF_UNICODETEXT, memory).is_null()
            {
                GlobalFree(memory);
            }
        }
        CloseClipboard();
    }
}

fn copy_wide(target: &mut [u16], value: &str) {
    let wide: Vec<u16> = value.encode_utf16().take(target.len() - 1).collect();
    target[..wide.len()].copy_from_slice(&wide);
    target[wide.len()] = 0;
}

unsafe fn tray(hwnd: HWND, operation: u32, icon: HICON, balloon: Option<(&str, &str)>) -> bool {
    unsafe {
        let mut data: NOTIFYICONDATAW = std::mem::zeroed();
        data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        data.hWnd = hwnd;
        data.uID = 1;
        data.uFlags = NIF_MESSAGE | NIF_TIP;
        data.uCallbackMessage = TRAY;
        if !icon.is_null() {
            data.uFlags |= NIF_ICON;
            data.hIcon = icon;
        }
        copy_wide(&mut data.szTip, "BoomServer");
        if let Some((title, body)) = balloon {
            data.uFlags |= NIF_INFO;
            data.dwInfoFlags = NIIF_INFO;
            copy_wide(&mut data.szInfoTitle, title);
            copy_wide(&mut data.szInfo, body);
        }
        Shell_NotifyIconW(operation, &data) != 0
    }
}

unsafe fn tray_tip(hwnd: HWND, tip: &str) {
    unsafe {
        let mut data: NOTIFYICONDATAW = std::mem::zeroed();
        data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        data.hWnd = hwnd;
        data.uID = 1;
        data.uFlags = NIF_TIP;
        copy_wide(&mut data.szTip, tip);
        Shell_NotifyIconW(NIM_MODIFY, &data);
    }
}

unsafe extern "system" fn window(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => 0,
            WM_CTLCOLORSTATIC if l as HWND == DOT_HWND.get() => {
                let hdc = w as HDC;
                SetTextColor(hdc, DOT.get());
                SetBkMode(hdc, TRANSPARENT as i32);
                GetSysColorBrush(COLOR_BTNFACE) as LRESULT
            }
            WM_COMMAND | WM_TIMER | WM_CLOSE | TRAY | WM_DPICHANGED => {
                APP.with(|app| {
                    // Modal Windows dialogs and menus can dispatch a timer while an action is in progress.
                    let Ok(mut app) = app.try_borrow_mut() else {
                        return;
                    };
                    let Some(app) = app.as_mut() else { return };
                    match msg {
                        WM_COMMAND => app.command(hwnd, (w & 0xffff) as i32),
                        WM_DPICHANGED => {
                            app.dpi_changed(hwnd, (w & 0xffff) as u32, l as *const RECT)
                        }
                        WM_TIMER => {
                            app.refresh(hwnd);
                            if app.exit_after_stop && app.run.is_none() {
                                DestroyWindow(hwnd);
                            }
                        }
                        WM_CLOSE if app.run.as_ref().is_some_and(|run| run.stopping) => {
                            // Stop then Close should exit when draining finishes, even
                            // if Close arrives before the worker's completion message.
                            app.exit_after_stop = true;
                        }
                        WM_CLOSE if app.run.is_some() => app.hide_to_tray(hwnd),
                        WM_CLOSE => {
                            DestroyWindow(hwnd);
                        }
                        TRAY => match (l & 0xffff) as u32 {
                            WM_LBUTTONUP | WM_LBUTTONDBLCLK => app.restore(hwnd),
                            WM_RBUTTONUP | WM_CONTEXTMENU => app.tray_menu(hwnd),
                            _ => {}
                        },
                        _ => {}
                    }
                });
                0
            }
            WM_DESTROY => {
                tray(hwnd, NIM_DELETE, null_mut(), None);
                APP.with(|app| {
                    if let Ok(mut app) = app.try_borrow_mut()
                        && let Some(app) = app.as_mut()
                    {
                        if let Some(fonts) = app.fonts.take() {
                            fonts.delete();
                        }
                        for icon in [app.icons.0, app.icons.1] {
                            if !icon.is_null() {
                                DestroyIcon(icon);
                            }
                        }
                        app.icons = (null_mut(), null_mut());
                    }
                });
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}

pub fn main() {
    unsafe {
        let class = text("BoomServerS1");
        let existing = FindWindowW(class.as_ptr(), null());
        if !existing.is_null() {
            PostMessageW(existing, TRAY, 0, WM_LBUTTONUP as isize);
            return;
        }
        let _ = CoInitializeEx(null_mut(), COINIT_APARTMENTTHREADED as u32);
        let app = match App::new() {
            Ok(app) => app,
            Err(error) => {
                MessageBoxW(
                    null_mut(),
                    text(error.to_string()).as_ptr(),
                    text("BoomServer").as_ptr(),
                    MB_ICONERROR,
                );
                return;
            }
        };
        let instance = GetModuleHandleW(null());
        let info = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(window),
            hInstance: instance,
            lpszClassName: class.as_ptr(),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hIcon: app_icon(instance, GetSystemMetrics(SM_CXICON)),
            hIconSm: app_icon(instance, GetSystemMetrics(SM_CXSMICON)),
            hbrBackground: (COLOR_BTNFACE as usize + 1) as _,
            ..std::mem::zeroed()
        };
        RegisterClassExW(&info);
        let hwnd = CreateWindowExW(
            STYLE_EX,
            class.as_ptr(),
            text("BoomServer").as_ptr(),
            STYLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CLIENT.0 + 16,
            CLIENT.1 + 39,
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        if hwnd.is_null() {
            return;
        }
        APP.with(|slot| {
            let mut app = app;
            app.build(hwnd);
            *slot.borrow_mut() = Some(app);
        });
        SetTimer(hwnd, 1, 300, None);
        ShowWindow(hwnd, SW_SHOW);
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            if IsDialogMessageW(hwnd, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}
