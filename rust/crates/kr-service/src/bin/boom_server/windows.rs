use kr_service::self_host::{self, Action, Settings, Status};
use std::{
    cell::RefCell,
    ffi::OsString,
    io,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
    ptr::{null, null_mut},
};
use tokio::sync::{mpsc, watch};
use windows_sys::Win32::{
    Foundation::*,
    Globalization::GetUserDefaultUILanguage,
    Graphics::Gdi::{DEFAULT_GUI_FONT, GetStockObject},
    System::{
        Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoTaskMemFree},
        DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
        LibraryLoader::GetModuleHandleW,
        Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock},
        SystemInformation::GetSystemDirectoryW,
    },
    UI::{Input::KeyboardAndMouse::EnableWindow, Shell::*, WindowsAndMessaging::*},
};

const SETUP: i32 = 100;
const BROWSE: i32 = 101;
const COPY: i32 = 102;
const BACKUP: i32 = 104;
const DATA: i32 = 105;
const ROTATE: i32 = 106;
const TRAY: u32 = WM_APP + 1;
const CF_UNICODETEXT: u32 = 13;

type Text = [&'static str; 3];
const READY: Text = ["Đang chạy", "Running", "실행 중"];
const WAIT: Text = [
    "Đang chuẩn bị máy chủ…",
    "Starting the server…",
    "서버를 시작하는 중…",
];
const STOPPED: Text = ["Đã dừng", "Stopped", "중지됨"];
const STOPPING: Text = ["Đang dừng…", "Stopping…", "중지하는 중…"];
const ERROR: Text = [
    "Không thể chạy máy chủ",
    "Could not run the server",
    "서버를 실행할 수 없습니다",
];

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

struct App {
    root: PathBuf,
    state: PathBuf,
    settings: Option<Settings>,
    lang: usize,
    run: Option<Run>,
    preparation_error: String,
    tray: bool,
    name: HWND,
    game: HWND,
    status: HWND,
    counts: HWND,
    invite: HWND,
    primary: HWND,
    copy: HWND,
    rotate: HWND,
    backup: HWND,
    last_code: String,
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

unsafe fn set(hwnd: HWND, message: impl AsRef<std::ffi::OsStr>) {
    unsafe {
        SetWindowTextW(hwnd, text(message).as_ptr());
    }
}

fn detected(root: &Path) -> Option<PathBuf> {
    let contains = |path: &Path| {
        ["FxMap.idx", "FxContents.idx", "fx.idx"]
            .iter()
            .all(|name| path.join(name).is_file())
    };
    let parent = root.parent()?;
    [
        root.to_owned(),
        root.join("game"),
        parent.to_owned(),
        parent.join("game"),
    ]
    .into_iter()
    .find(|path| contains(path))
    .or_else(|| {
        std::fs::read_dir(parent)
            .ok()?
            .take(64)
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| contains(path))
    })
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
        let lang = match unsafe { GetUserDefaultUILanguage() } & 0x03ff {
            0x2a => 0,
            0x12 => 2,
            _ => 1,
        };
        Ok(Self {
            root,
            state,
            settings,
            lang,
            run: None,
            preparation_error: String::new(),
            tray: false,
            name: null_mut(),
            game: null_mut(),
            status: null_mut(),
            counts: null_mut(),
            invite: null_mut(),
            primary: null_mut(),
            copy: null_mut(),
            rotate: null_mut(),
            backup: null_mut(),
            last_code: String::new(),
        })
    }

    fn tr(&self, x: Text) -> &'static str {
        x[self.lang]
    }

    unsafe fn build(&mut self, hwnd: HWND) {
        const EDIT: u32 = WS_BORDER | WS_TABSTOP | ES_AUTOHSCROLL as u32;
        unsafe {
            child(
                hwnd,
                "STATIC",
                self.tr(["Tên máy chủ", "Server name", "서버 이름"]),
                (16, 16, 110, 20),
                0,
                0,
            );
            self.name = child(
                hwnd,
                "EDIT",
                self.settings
                    .as_ref()
                    .map(|s| s.name.as_str())
                    .unwrap_or("Boom Server"),
                (128, 14, 282, 23),
                0,
                EDIT,
            );
            child(
                hwnd,
                "STATIC",
                self.tr(["Thư mục game", "Game folder", "게임 폴더"]),
                (16, 48, 110, 20),
                0,
                0,
            );
            self.game = child(
                hwnd,
                "EDIT",
                "",
                (128, 45, 282, 23),
                0,
                EDIT | ES_READONLY as u32,
            );
            if let Some(game) = self
                .settings
                .as_ref()
                .map(|s| s.game.clone())
                .or_else(|| detected(&self.root))
            {
                set(self.game, game.as_os_str());
            }
            child(
                hwnd,
                "BUTTON",
                self.tr(["Chọn…", "Browse…", "찾아보기…"]),
                (420, 44, 91, 24),
                BROWSE,
                WS_TABSTOP,
            );
            self.status = child(hwnd, "STATIC", self.tr(STOPPED), (16, 85, 500, 40), 0, 0);
            self.counts = child(hwnd, "STATIC", "", (16, 126, 500, 22), 0, 0);
            child(
                hwnd,
                "STATIC",
                self.tr(["Mã mời S1", "S1 invite", "S1 초대 코드"]),
                (16, 157, 110, 24),
                0,
                0,
            );
            self.invite = child(
                hwnd,
                "EDIT",
                "",
                (128, 154, 282, 23),
                0,
                EDIT | ES_READONLY as u32,
            );
            self.copy = child(
                hwnd,
                "BUTTON",
                self.tr(["Sao chép", "Copy", "복사"]),
                (420, 153, 91, 24),
                COPY,
                WS_TABSTOP,
            );
            self.primary = child(
                hwnd,
                "BUTTON",
                self.tr(["Chạy", "Start", "시작"]),
                (16, 193, 88, 27),
                SETUP,
                WS_TABSTOP,
            );
            self.backup = child(
                hwnd,
                "BUTTON",
                self.tr(["Sao lưu ngay", "Back up now", "지금 백업"]),
                (112, 193, 108, 27),
                BACKUP,
                WS_TABSTOP,
            );
            child(
                hwnd,
                "BUTTON",
                self.tr(["Mở thư mục dữ liệu", "Open data folder", "데이터 폴더 열기"]),
                (228, 193, 138, 27),
                DATA,
                WS_TABSTOP,
            );
            self.rotate = child(
                hwnd,
                "BUTTON",
                self.tr(["Tạo lại mã mời", "Reset invite", "초대 코드 재설정"]),
                (374, 193, 137, 27),
                ROTATE,
                WS_TABSTOP,
            );
            self.refresh(hwnd);
            if self.settings.is_some() {
                PostMessageW(hwnd, WM_COMMAND, SETUP as usize, 0);
            }
        }
    }

    unsafe fn start(&mut self, hwnd: HWND) -> io::Result<()> {
        let settings = Settings {
            name: unsafe { read(self.name) }
                .to_string_lossy()
                .trim()
                .to_owned(),
            game: PathBuf::from(unsafe { read(self.game) }),
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
        self.last_code.clear();
        self.preparation_error.clear();
        self.run = Some(Run {
            send,
            status,
            done,
            stopping: false,
        });
        unsafe {
            self.refresh(hwnd);
        }
        Ok(())
    }

    unsafe fn firewall(&self, hwnd: HWND, port: u16) {
        let marker = self.state.join("firewall-asked");
        if marker.exists() {
            return;
        }
        let yes = unsafe {
            MessageBoxW(hwnd, text(self.tr([
                "Cho phép máy khác kết nối trực tiếp qua UDP trên mạng Riêng tư? Windows sẽ xin quyền quản trị. Bạn vẫn có thể dùng máy chủ nếu chọn Không.",
                "Allow direct UDP connections on Private networks? Windows will ask for administrator permission. The server can still run if you choose No.",
                "개인 네트워크에서 직접 UDP 연결을 허용할까요? Windows가 관리자 권한을 요청합니다. 아니요를 선택해도 서버를 실행할 수 있습니다.",
            ])).as_ptr(), text("BoomServer").as_ptr(), MB_YESNO | MB_ICONQUESTION | MB_DEFBUTTON2) == IDYES
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

    unsafe fn refresh(&mut self, hwnd: HWND) {
        if let Some(run) = &self.run {
            match run.done.try_recv() {
                Ok(result) => {
                    self.preparation_error = result.err().unwrap_or_default();
                    self.run = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.preparation_error = self.tr(ERROR).into();
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
        if status.invite != self.last_code {
            self.last_code.clone_from(&status.invite);
            unsafe {
                set(self.invite, &self.last_code);
            }
        }
        let summary = if !self.preparation_error.is_empty() {
            format!("{}: {}", self.tr(ERROR), self.preparation_error)
        } else if self.run.as_ref().is_some_and(|run| run.stopping) {
            self.tr(STOPPING).into()
        } else if !status.ready {
            self.tr(if self.run.is_some() { WAIT } else { STOPPED })
                .into()
        } else {
            let path = if !status.relay_expected {
                self.tr(["Mạng cục bộ", "Local network", "로컬 네트워크"])
            } else if status.relay_online {
                self.tr(["Relay đã kết nối", "Relay connected", "릴레이 연결됨"])
            } else {
                self.tr([
                    "Đang kết nối relay…",
                    "Connecting to relay…",
                    "릴레이 연결 중…",
                ])
            };
            let message = if status.message.is_empty() {
                String::new()
            } else {
                format!("\r\n{}", status.message)
            };
            format!("{} · {path}{message}", self.tr(READY))
        };
        unsafe {
            set(self.status, summary);
            set(
                self.counts,
                format!(
                    "{}: {}     {}: {}",
                    self.tr(["Đang chơi", "Online", "접속 중"]),
                    status.players,
                    self.tr(["Phòng", "Rooms", "방"]),
                    status.rooms
                ),
            );
            set(
                self.primary,
                self.tr(if self.run.is_some() {
                    ["Dừng", "Stop", "중지"]
                } else {
                    ["Chạy", "Start", "시작"]
                }),
            );
            EnableWindow(self.copy, i32::from(status.ready));
            EnableWindow(self.rotate, i32::from(status.ready));
            EnableWindow(self.backup, i32::from(status.ready));
            EnableWindow(
                self.primary,
                i32::from(!self.run.as_ref().is_some_and(|run| run.stopping)),
            );
            EnableWindow(self.name, i32::from(self.run.is_none()));
            EnableWindow(GetDlgItem(hwnd, BROWSE), i32::from(self.run.is_none()));
        }
    }

    unsafe fn command(&mut self, hwnd: HWND, id: i32) {
        unsafe {
            match id {
                SETUP => if let Some(run) = &mut self.run {
                    if run.send.try_send(Action::Stop).is_ok() { run.stopping = true; }
                } else if let Err(error) = self.start(hwnd) {
                    self.preparation_error = error.to_string();
                },
                BROWSE => if let Some(path) = pick_folder(hwnd, self.tr(["Chọn thư mục game KR 00967", "Select the KR 00967 game folder", "KR 00967 게임 폴더를 선택하세요"])) {
                    set(self.game, path.as_os_str());
                },
                COPY => if let Some(run) = &self.run {
                    let status = run.status.borrow();
                    if status.ready { clipboard(hwnd, &status.invite); }
                },
                BACKUP => if let Some(run) = &self.run { let _ = run.send.try_send(Action::Backup); },
                DATA => { ShellExecuteW(hwnd, text("open").as_ptr(), text(self.state.as_os_str()).as_ptr(), null(), null(), SW_SHOWNORMAL); },
                ROTATE => {
                    if MessageBoxW(hwnd, text(self.tr([
                        "Mã cũ sẽ hết hiệu lực và người đang chơi sẽ ngắt kết nối. Tiếp tục?",
                        "The old code will stop working and current players will disconnect. Continue?",
                        "기존 코드는 무효가 되고 플레이어 연결이 끊깁니다. 계속할까요?",
                    ])).as_ptr(), text("BoomServer").as_ptr(), MB_YESNO | MB_DEFBUTTON2 | MB_ICONQUESTION) == IDYES
                        && let Some(run) = &self.run
                    {
                        let _ = run.send.try_send(Action::Rotate);
                    }
                }
                _ => {}
            }
            self.refresh(hwnd);
        }
    }
}

unsafe fn child(
    hwnd: HWND,
    class: &str,
    label: &str,
    rect: (i32, i32, i32, i32),
    id: i32,
    style: u32,
) -> HWND {
    unsafe {
        let (x, y, w, h) = rect;
        let handle = CreateWindowExW(
            0,
            text(class).as_ptr(),
            text(label).as_ptr(),
            WS_CHILD | WS_VISIBLE | style,
            x,
            y,
            w,
            h,
            hwnd,
            id as isize as HMENU,
            GetModuleHandleW(null()),
            null(),
        );
        SendMessageW(
            handle,
            WM_SETFONT,
            GetStockObject(DEFAULT_GUI_FONT) as usize,
            1,
        );
        handle
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

unsafe fn pick_folder(hwnd: HWND, title: &str) -> Option<PathBuf> {
    unsafe {
        let title = text(title);
        let mut display_name = [0; 260];
        let info = BROWSEINFOW {
            hwndOwner: hwnd,
            lpszTitle: title.as_ptr(),
            pszDisplayName: display_name.as_mut_ptr(),
            ulFlags: BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE,
            ..std::mem::zeroed()
        };
        let item = SHBrowseForFolderW(&info);
        if item.is_null() {
            return None;
        }
        let mut path = vec![0; 32_768];
        let ok = SHGetPathFromIDListEx(item, path.as_mut_ptr(), path.len() as u32, 0) != 0;
        CoTaskMemFree(item.cast());
        if !ok {
            return None;
        }
        let end = path.iter().position(|unit| *unit == 0)?;
        Some(PathBuf::from(OsString::from_wide(&path[..end])))
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

unsafe fn tray(hwnd: HWND, operation: u32) -> bool {
    unsafe {
        let mut data: NOTIFYICONDATAW = std::mem::zeroed();
        data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        data.hWnd = hwnd;
        data.uID = 1;
        data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        data.uCallbackMessage = TRAY;
        data.hIcon = LoadIconW(null_mut(), IDI_APPLICATION);
        let label = text("BoomServer");
        data.szTip[..label.len()].copy_from_slice(&label);
        Shell_NotifyIconW(operation, &data) != 0
    }
}

unsafe extern "system" fn window(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => 0,
            WM_COMMAND | WM_TIMER | WM_CLOSE | TRAY => {
                APP.with(|app| {
                    // Modal Windows dialogs can dispatch a timer while an action is in progress.
                    let Ok(mut app) = app.try_borrow_mut() else {
                        return;
                    };
                    let Some(app) = app.as_mut() else { return };
                    match msg {
                        WM_COMMAND => app.command(hwnd, (w & 0xffff) as i32),
                        WM_TIMER => app.refresh(hwnd),
                        WM_CLOSE if app.run.is_some() => {
                            if !app.tray {
                                app.tray = tray(hwnd, NIM_ADD);
                            }
                            if app.tray {
                                ShowWindow(hwnd, SW_HIDE);
                            } else {
                                ShowWindow(hwnd, SW_MINIMIZE);
                            }
                        }
                        WM_CLOSE => {
                            DestroyWindow(hwnd);
                        }
                        TRAY if matches!(l as u32, WM_LBUTTONUP | WM_RBUTTONUP) => {
                            ShowWindow(hwnd, SW_RESTORE);
                            SetForegroundWindow(hwnd);
                            tray(hwnd, NIM_DELETE);
                            app.tray = false;
                        }
                        _ => {}
                    }
                });
                0
            }
            WM_DESTROY => {
                tray(hwnd, NIM_DELETE);
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
        let info = WNDCLASSW {
            lpfnWndProc: Some(window),
            hInstance: instance,
            lpszClassName: class.as_ptr(),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hIcon: LoadIconW(null_mut(), IDI_APPLICATION),
            hbrBackground: (windows_sys::Win32::Graphics::Gdi::COLOR_BTNFACE as usize + 1) as _,
            ..std::mem::zeroed()
        };
        RegisterClassW(&info);
        let hwnd = CreateWindowExW(
            WS_EX_CONTROLPARENT,
            class.as_ptr(),
            text("BoomServer").as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            542,
            277,
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
