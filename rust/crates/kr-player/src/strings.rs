//! All player-facing launcher text. Columns are Vietnamese, English and Korean.
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Language {
    #[default]
    Vi,
    En,
    Ko,
}

impl Language {
    pub const ALL: [Self; 3] = [Self::Vi, Self::En, Self::Ko];

    pub(crate) fn id(self) -> &'static str {
        ["vi", "en", "ko"][self as usize]
    }

    pub fn get(self, text: Copy) -> &'static str {
        COPY[text as usize][self as usize]
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(["Tiếng Việt", "English", "한국어"][*self as usize])
    }
}

// Keep the key and all of its translations together.
macro_rules! strings {
    ($($(#[$attr:meta])* $key:ident => $vi:literal, $en:literal, $ko:literal;)*) => {
        #[derive(Clone, Copy)]
        pub(crate) enum Copy { $($(#[$attr])* $key,)* }
        const COPY: &[[&str; 3]] = &[$($(#[$attr])* [$vi, $en, $ko],)*];
    };
}

strings! {
    Title => "BOOM PLAY", "BOOM PLAY", "BOOM PLAY";
    Settings => "Cài đặt", "Settings", "설정";
    Lang => "Ngôn ngữ", "Language", "언어";
    Mode => "Chế độ chơi", "Play mode", "플레이 모드";
    Servers => "Máy chủ", "Servers", "서버";
    BundleServer => "Máy chủ trong gói", "Bundled server", "패키지에 포함된 서버";
    NoServers => "Gói chưa cấu hình máy chủ.", "No server configured in this bundle.", "이 패키지에 서버가 설정되지 않았습니다.";
    AddServer => "Thêm máy chủ", "Add server", "서버 추가";
    Download => "Tải bản mới", "Download latest", "최신 버전 받기";
    ReleaseError => "Không thể mở liên kết tải.", "Could not open the download link.", "다운로드 링크를 열 수 없습니다.";
    PlayFriends => "Chơi với bạn bè", "Play with friends", "친구와 플레이";
    Soon => "Sắp có", "Coming soon", "출시 예정";
    Friends => "Tạo phòng và gửi mã cho bạn bè. Không cần tài khoản.", "Host a room and share the code. No account needed.", "방을 만들고 코드를 공유하세요. 계정이 필요 없습니다.";
    FriendsName => "Tên nhân vật", "Character name", "캐릭터 이름";
    FriendsNameBad => "Nhập tên hợp lệ (3–15 byte trong game).", "Enter a valid name (3–15 bytes in game).", "유효한 이름을 입력하세요(게임에서 3–15바이트).";
    FriendsHost => "Tạo phòng", "Host room", "방 만들기";
    FriendsJoin => "Vào phòng", "Join room", "방 참가";
    FriendsCode => "Mã phòng", "Room code", "방 코드";
    FriendsCodeBad => "Mã phòng không hợp lệ.", "The room code is invalid.", "방 코드가 올바르지 않습니다.";
    FriendsFind => "Tìm phòng LAN", "Find LAN rooms", "LAN 방 찾기";
    FriendsSearching => "Đang tìm phòng LAN…", "Searching for LAN rooms…", "LAN 방을 찾는 중…";
    FriendsNone => "Không thấy phòng trong mạng LAN.", "No rooms found on this LAN.", "LAN에서 방을 찾을 수 없습니다.";
    FriendsUnavailable => "Không vào được phòng. Kiểm tra mã và hỏi chủ phòng.", "Couldn't join. Check the code and ask the host.", "참가할 수 없습니다. 코드와 방 상태를 확인하세요.";
    FriendsVersion => "Khác phiên bản gói với chủ phòng.", "Your package version differs from the host's.", "방장과 패키지 버전이 다릅니다.";
    FriendsClosed => "Phòng đã đóng hoặc mất kết nối với chủ phòng.", "The room closed or the host disconnected.", "방이 닫혔거나 방장과의 연결이 끊겼습니다.";
    FriendsHostBad => "Không mở được máy chủ cục bộ.", "Couldn't start the local server.", "로컬 서버를 시작하지 못했습니다.";
    FriendsNoHost => "Gói này thiếu máy chủ cục bộ.", "This package has no local server.", "이 패키지에 로컬 서버가 없습니다.";
    ServerInvite => "Mã mời máy chủ", "Server invite", "서버 초대 코드";
    ServersIntro => "Vào máy chủ do cộng đồng dựng bằng mã mời. Có tài khoản và lưu tiến trình.", "Join community servers with an invite code. Accounts and saved progress.", "초대 코드로 커뮤니티 서버에 접속하세요. 계정을 사용하고 진행 상황을 저장합니다.";
    StartGuide => "Cách bắt đầu", "Getting started", "시작하기";
    ConnectionGuide => "Trợ giúp kết nối", "Connection help", "연결 도움말";
    VersionGuide => "Phiên bản & tệp", "Version & files", "버전 및 파일";
    Play => "CHƠI", "PLAY", "게임 시작";
    Ready => "Sẵn sàng", "Ready", "준비 완료";
    Checking => "Đang kiểm tra tệp…", "Checking game files…", "게임 파일 확인 중…";
    Launching => "Đang mở game…", "Starting the game…", "게임 시작 중…";
    Running => "Game đang chạy", "Game is running", "게임 실행 중";
    Error => "Có lỗi", "Something went wrong", "오류가 발생했습니다";
    ServerChecking => "Đang kiểm tra máy chủ", "Checking server", "서버 확인 중";
    ServerReady => "Máy chủ hoạt động", "Server online", "서버 정상";
    ServerOffline => "Không kết nối được", "Server unreachable", "서버에 연결할 수 없음";
    Retry => "Thử lại", "Retry", "다시 시도";
    Repair => "Sửa tệp", "Repair files", "파일 복구";
    FilesBad => "Không thể xác minh tệp game. Hãy kiểm tra hoặc sửa tệp từ gói cài đặt.", "Game files could not be verified. Check or repair them from the bundle.", "게임 파일을 확인할 수 없습니다. 설치 패키지에서 파일을 확인하거나 복구해 주세요.";
    StartBad => "Không thể mở game. Hãy thử lại.", "The game could not start. Please try again.", "게임을 시작할 수 없습니다. 다시 시도해 주세요.";
    WriteBad => "Không thể ghi tệp game. Hãy kiểm tra quyền ghi của thư mục.", "Game files could not be written. Check folder permissions.", "게임 파일에 쓸 수 없습니다. 폴더의 쓰기 권한을 확인해 주세요.";
    SessionEnded => "Phiên chơi đã kết thúc. Hãy mở game để đăng nhập lại.", "The game session ended. Open the game to sign in again.", "게임 세션이 종료되었습니다. 게임을 열고 다시 로그인해 주세요.";
    PrefBad => "Không thể lưu lựa chọn.", "Your choice could not be saved.", "선택 사항을 저장할 수 없습니다.";
    Launcher => "Launcher", "Launcher", "런처";
    Package => "Gói", "Bundle", "패키지";
    Help => "Hướng dẫn", "Help", "이용 안내";
    Close => "Đóng", "Close", "닫기";
    Back => "Quay lại", "Back", "뒤로";
    Create => "Tạo tài khoản", "Create account", "계정 만들기";
    Forgot => "Quên mật khẩu", "Forgot password", "비밀번호 찾기";
    Manage => "Quản lý tài khoản", "Manage account", "계정 관리";
    Change => "Đổi mật khẩu", "Change password", "비밀번호 변경";
    Delete => "Xoá tài khoản", "Delete account", "계정 삭제";
    DeleteConfirm => "Xác nhận xoá tài khoản", "Confirm account deletion", "계정 삭제 확인";
    DeleteHint => "Thao tác này xoá tài khoản. Nhập mật khẩu hiện tại để xác nhận.", "This deletes your account. Enter your current password to confirm.", "계정이 삭제됩니다. 확인하려면 현재 비밀번호를 입력해 주세요.";
    LoginId => "ID tài khoản", "Account ID", "계정 아이디";
    LoginHint => "ID: 3–32 chữ cái ASCII, chữ số hoặc dấu gạch dưới.", "ID: 3–32 ASCII letters, digits or underscores.", "아이디: 영문, 숫자 또는 밑줄 3~32자.";
    Name => "Tên hiển thị", "Display name", "표시 이름";
    NameHint => "Nhập tên hiển thị.", "Enter a display name.", "표시 이름을 입력해 주세요.";
    Password => "Mật khẩu", "Password", "비밀번호";
    PasswordHint => "Mật khẩu: 12–128 ký tự ASCII in được.", "Password: 12–128 printable ASCII characters.", "비밀번호: 인쇄 가능한 ASCII 문자 12~128자.";
    Current => "Mật khẩu hiện tại", "Current password", "현재 비밀번호";
    New => "Mật khẩu mới", "New password", "새 비밀번호";
    Code => "Mã khôi phục một lần", "One-time recovery code", "일회용 복구 코드";
    CodeHint => "Nhập mã khôi phục gồm 64 ký tự thập lục phân.", "Enter the 64-character hexadecimal recovery code.", "64자리 16진수 복구 코드를 입력해 주세요.";
    KeepCode => "Lưu mã ở nơi an toàn. Mã chỉ hiện một lần.", "Save this code somewhere safe. It is shown only once.", "코드를 안전한 곳에 보관하세요. 한 번만 표시됩니다.";
    Clipboard => "Sao chép mã", "Copy code", "코드 복사";
    Copied => "Đã sao chép mã.", "Code copied.", "코드를 복사했습니다.";
    AccountOk => "Thao tác đã hoàn tất.", "Account updated successfully.", "계정 요청이 완료되었습니다.";
    AccountBad => "Không thực hiện được. Kiểm tra thông tin và thử lại.", "Request failed. Check your details and try again.", "처리할 수 없습니다. 정보를 확인한 후 다시 시도해 주세요.";
    Sending => "Đang gửi yêu cầu…", "Sending request…", "요청을 보내는 중…";
    Folder => "Thư mục game", "Game folder", "게임 폴더";
    Open => "Mở", "Open", "열기";
    Logs => "Mở thư mục log", "Open log folder", "로그 폴더 열기";
    CheckRepair => "Kiểm tra & sửa tệp", "Check & repair files", "파일 확인 및 복구";
    #[cfg(windows)]
    Shortcut => "Tạo lối tắt", "Create shortcut", "바로가기 만들기";
    Unavailable => "Chưa có", "Unavailable", "사용 불가";
    FilesOk => "Tệp game đã được xác minh.", "Game files verified.", "게임 파일을 확인했습니다.";
    FilesRepaired => "Đã sửa tệp. Bản cài cũ được giữ cạnh thư mục game.", "Files repaired. The previous installation was saved beside the game folder.", "파일을 복구했습니다. 이전 설치는 게임 폴더 옆에 보관했습니다.";
    OpenBad => "Không thể mở thư mục hoặc tạo lối tắt.", "Could not open the folder or create the shortcut.", "폴더를 열거나 바로가기를 만들 수 없습니다.";
    #[cfg(windows)]
    ShortcutOk => "Đã tạo lối tắt trong thư mục gói.", "Shortcut created in the bundle folder.", "패키지 폴더에 바로가기를 만들었습니다.";
}
