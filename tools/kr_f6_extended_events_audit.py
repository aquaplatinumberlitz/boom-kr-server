"""Sinh đặc tả F6 mở rộng từ tài liệu và metadata KR 00967 trong repo."""

import argparse
import hashlib
import ipaddress
import json
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
INPUTS = {
    "classification": "docs/content/classification.json",
    "catalog": "docs/evidence/content/f6-events-catalog.json",
    "catalog_notes": "docs/content/f6-events-catalog.md",
    "policy_notes": "docs/content/f6-events-policy.md",
    "roadmap_f6_row": "docs/PLAN.md",
}
MARKDOWN = "docs/content/f6-extended-events-spec.md"
EVIDENCE = "docs/evidence/content/f6-extended-events-spec.json"
BUILD = "KR00967"
EXCLUDED = "PCRoomBenefit"
SHARED = "Web nhúng: Dwarf/libcef"

# Mỗi giả thuyết mở rộng có một hàng xếp loại làm điểm xuất phát.
LOTTERY = ("system", "Bốc thăm và đậu may mắn (뽀끼뽀끼, 럭키빈)")
BOOKS = ("system", "Chợ sách phép (마법거래소)")
MINIGAMES = ("system", "Mini-game sự kiện (빙고, 주사위, 퀴즈)")
ALCHEMY = ("system", "Hợp thành và dừng thời gian (합성, 타임스톱)")
PET_CARE = ("system", "Chăm sóc pet (애정도, 봉인, 듀얼펫)")
DRAW_TYPES = ("system", "Các kiểu bốc thăm (알쏭달쏭 쾅쾅, 마법구슬, 선물상자)")
ATTENDANCE = ("system", "Điểm danh và quà quay lại (출석, 귀환 스탬프)")
MISSION = ("system", "Nhiệm vụ")
FISHING = ("system", "Câu cá")

# Quota và khóa đều là thiết kế; chúng không mô tả luật KR. Khóa bên dưới
# bổ sung vào prefix (system_id, account_id, season_id) chung.
POLICY_RULES = {
    "CAPass": ("Giới hạn cập nhật tiến độ và số mốc nhận theo mùa.", "ID kết quả đã chốt cho tiến độ; ID nhánh và mốc cho quà."),
    "Câu cá": ("Giới hạn lượt câu và lượt đổi cá theo ngày.", "ID lượt câu hoặc giao dịch đổi do server cấp."),
    "DIYBomb": ("Giới hạn lượt ghi thay đổi theo ngày.", "ID thao tác server và phiên bản vật phẩm đích."),
    "DTT": ("Giới hạn lượt đổi theo ngày và lượng hàng theo mùa.", "ID giao dịch đổi do server cấp."),
    "Exchange": ("Giới hạn lượt đổi mỗi ngày và lượng đổi theo công thức.", "ID giao dịch đổi do server cấp."),
    "GrowSprout": ("Giới hạn lượt tăng tiến độ và nhận quà theo ngày.", "ID nguồn tiến độ hoặc ID mốc nhận quà."),
    "HoneyButterChip": ("Giới hạn lượt tham gia và cấp quà theo ngày.", "ID lượt tham gia do server cấp."),
    "LineFriends": ("Giới hạn số lần nhận theo mùa.", "ID đợt phát và mục quà."),
    "LuckyCard": ("Giới hạn lần lật theo ngày và tổng quà theo mùa.", "ID thẻ hoặc lượt lật; giữ kết quả đã ghi."),
    "MagicMarket": ("Giới hạn lượt chế tạo theo ngày và theo công thức.", "ID đơn chế tạo; gắn phiên bản công thức."),
    "MineExplore": ("Giới hạn lượt tham gia và nhận quà theo ngày.", "ID lượt do server cấp và ID mốc tương ứng."),
    "MonWars": ("Giới hạn đóng góp và nhận quà theo ngày.", "ID nguồn đóng góp hoặc mốc nhận quà."),
    "Nhiệm vụ": ("Giới hạn lần nhận theo chu kỳ nhiệm vụ.", "ID nhiệm vụ, chu kỳ và mốc; ghi nguồn tiến độ một lần."),
    "Partners": ("Giới hạn lần nhận theo chiến dịch.", "ID chiến dịch và mục quà."),
    "Pet": ("Giới hạn số lần đổi trạng thái mỗi pet theo ngày.", "ID pet và phiên bản trạng thái trước thao tác."),
    "RetroWriggle": ("Giới hạn lượt tham gia và nhận quà theo ngày.", "ID lượt hoặc mốc do server cấp."),
    "SelectionBox": ("Giới hạn lượt mở theo ngày và một lần mỗi hộp.", "ID thực thể hộp; không đổi lựa chọn sau khi chốt."),
    "Sự kiện TakeCareOfEvent": ("Giới hạn đóng góp theo ngày và số mốc nhận theo mùa.", "ID nguồn đóng góp hoặc mốc nhận quà."),
    "Sự kiện theo tiền tố Event": ("Giới hạn tham gia theo ngày và quà theo từng sự kiện.", "ID sự kiện và ID lượt tham gia hoặc mốc quà."),
}

# ID nội bộ của đặc tả, không phải tên cấu hình F6 hiện có hay ID trên wire.
# Một phân hệ có thể chi tiết hóa hệ thống in_scope mà không thêm lại nó vào policy.
EXTENDED = (
    ("pet-care", "Chăm sóc pet cơ bản", PET_CARE,
     "Giới hạn lượt chăm sóc theo pet và ngày.",
     "ID pet và ID lần chăm sóc; không trừ vật phẩm lần hai.",
     "Dùng một lượt chăm sóc, mở lại UI và xác nhận trạng thái."),
    ("pet-intimacy", "Độ thân thiết pet", PET_CARE,
     "Giới hạn điểm tăng theo pet và ngày; chặn tăng quá trần.",
     "ID pet và sự kiện sinh điểm; cùng nguồn chỉ cộng một lần.",
     "So sánh trước và sau một nguồn điểm hợp lệ và một nguồn bị từ chối."),
    ("pet-sealing", "Niêm phong và mở niêm phong pet", PET_CARE,
     "Giới hạn lượt niêm phong và mở theo pet và ngày.",
     "ID pet, phiên bản và hướng chuyển; giữ một chủ sở hữu hợp lệ.",
     "Niêm phong rồi mở; thử mở lại trạng thái đã mở."),
    ("pet-upgrade", "Nâng bậc pet", PET_CARE,
     "Giới hạn lượt nâng theo pet và ngày; chỉ cho phép cặp bậc đã khai báo.",
     "ID pet, phiên bản và bậc đích; nguyên liệu và bậc cập nhật trong một giao dịch.",
     "Quan sát một lần nâng và một lần thiếu nguyên liệu."),
    ("dual-pet", "Trang bị hai pet", PET_CARE,
     "Giới hạn đổi trang bị theo ngày; tối đa hai ô trong thiết kế hai pet.",
     "ID thao tác và phiên bản bộ trang bị; cùng pet không chiếm hai ô.",
     "Trang bị hai pet, thử trùng hoặc vượt ô, rồi đăng nhập lại."),
    ("pet-auto-use", "Pet tự dùng Khiên hoặc Kim", PET_CARE,
     "Giới hạn theo pet và trận; tuân thủ RoomRules và tồn kho.",
     "ID trận, sự kiện kích hoạt và pet; tối đa một lần dùng cho cùng sự kiện.",
     "Quan sát một kích hoạt hợp lệ và một tình huống bị RoomRules từ chối."),
    ("attendance", "Điểm danh", ATTENDANCE,
     "Tối đa một dấu điểm danh theo ngày UTC và lần nhận theo từng mốc mùa.",
     "Ngày UTC hoặc ID mốc; đăng nhập lại không cấp lại.",
     "Kiểm hai lần đăng nhập cùng ngày và qua mốc đổi ngày UTC."),
    ("comeback", "Tem và quà quay lại", ATTENDANCE,
     "Giới hạn dấu theo đợt đủ điều kiện và số mốc nhận theo chiến dịch.",
     "ID đợt đủ điều kiện và mốc; đăng nhập đồng thời không tạo đợt thứ hai.",
     "Đối chiếu tài khoản đủ và không đủ điều kiện qua một lần đăng nhập lại."),
    ("bingo", "Bingo sự kiện", MINIGAMES,
     "Giới hạn lượt đánh dấu theo ngày và quà theo bảng.",
     "ID bảng, ô hoặc hàng thưởng; mỗi ô và mốc thưởng chỉ ghi một lần.",
     "Đánh dấu một ô, hoàn tất một mốc và thử nhận mốc lại."),
    ("dice", "Xúc xắc sự kiện", MINIGAMES,
     "Giới hạn lượt gieo theo ngày và tiến độ theo bảng.",
     "ID lượt gieo; phát lại giữ nguyên mặt và vị trí.",
     "Gieo một lượt rồi thử gửi lại cùng lượt."),
    ("quiz", "Câu đố sự kiện", MINIGAMES,
     "Giới hạn số lần nộp mỗi phiên câu hỏi và tổng lượt theo ngày.",
     "ID phiên câu hỏi và lượt nộp; điểm và quà chỉ ghi một lần.",
     "Nộp một câu đúng, một câu sai hoặc quá hạn và quan sát phản hồi."),
    ("daily-quest", "Tiến độ và nhận quà nhiệm vụ ngày", MISSION,
     "Giới hạn đóng góp hợp lệ theo ngày UTC và một quà mỗi nhiệm vụ trong chu kỳ.",
     "ID nguồn tiến độ hoặc ID nhiệm vụ và ngày nhận; không cộng lại khi phát lại kết quả trận.",
     "Hoàn thành một điều kiện, nhận một quà và kiểm biên đổi ngày."),
    ("fishing-loop", "Lượt câu và đổi cá", FISHING,
     "Giới hạn lượt câu và lượt đổi theo ngày; kiểm tồn kho trước khi đổi.",
     "ID lượt câu hoặc đơn đổi; không bốc lại kết quả hoặc đổi lại cùng cá đã tiêu.",
     "Câu một lượt, kiểm túi đồ và đối chiếu một lượt đổi cá."),
    ("capass-shop-map", "Map CAPassShopMap", ("map", "CAPassShopMap"),
     "Giới hạn chuyển cảnh theo phiên và một lần vào đang mở.",
     "ID phiên và lượt chuyển; tải lại không tự nhận quà CAPass.",
     "Thử vào, mở UI, rời và quay lại mà không nhận thưởng."),
    ("daily-mission-shop-map", "Map DailyMissionShop", ("map", "DailyMissionShop"),
     "Giới hạn chuyển cảnh theo phiên và một lần vào đang mở.",
     "ID phiên và lượt chuyển; tải lại không đặt lại tiến độ nhiệm vụ.",
     "Thử vào, mở UI nhiệm vụ và rời trong khi giữ nguyên tiến độ."),
    ("fishing-test-map", "Map LobbySquareFishingTest", ("map", "LobbySquareFishingTest"),
     "Giới hạn chuyển cảnh theo phiên; vào map không tiêu lượt câu.",
     "ID phiên và lượt chuyển; không sinh cá hay thưởng từ lần vào lặp.",
     "Xác định client có vào được không và có mở màn câu cá hay không."),
    ("magic-market-map", "Map MagicMarketmap", ("map", "MagicMarketmap"),
     "Giới hạn chuyển cảnh theo phiên và một lần vào đang mở.",
     "ID phiên và lượt chuyển; vào lại không tạo thêm đơn chế tạo.",
     "Thử vào, mở UI, hủy trước khi giao dịch và rời map."),
    ("selection-box-map", "Map SelectionBox", ("map", "SelectionBox"),
     "Giới hạn chuyển cảnh theo phiên và một lần vào đang mở.",
     "ID phiên và lượt chuyển; không mở hộp khi tải lại map.",
     "Thử chuyển cảnh và kiểm map có thực sự mở UI chọn hộp hay không."),
    ("lucky-card-reveal", "Luồng lật thẻ LuckyCard", ("system", "LuckyCard"),
     "Giới hạn lật theo ngày; tối đa một lần mỗi thẻ và có trần quà theo mùa.",
     "ID thẻ; lưu lượt lật và kết quả trước khi phản hồi.",
     "Lật thẻ với vật phẩm thử miễn phí rồi thử lật lại."),
    ("magic-market-craft", "Luồng chế tạo MagicMarket", ("system", "MagicMarket"),
     "Giới hạn chế tạo theo công thức và ngày; kiểm lượng nguyên liệu tối đa.",
     "ID đơn và phiên bản công thức; tiêu nguyên liệu và cấp đồ trong cùng giao dịch.",
     "Xem trước, chế tạo một món, thử thiếu nguyên liệu và đối chiếu túi đồ."),
    ("magic-book-market", "Chợ sách phép", BOOKS,
     "Giới hạn tin đang mở và lượt đăng, mua, hủy theo ngày.",
     "ID tin và phiên bản; giao dịch chỉ cho một người mua, không vừa bán vừa trả đồ.",
     "Hai client thử đăng, mua hoặc hủy; đối chiếu quyền sở hữu ở hai phía."),
    ("bean-lottery", "Bốc thăm và đổi đậu may mắn", LOTTERY,
     "Giới hạn lượt bốc, lần đổi và tổng quà; chỉ dùng chìa khóa và đậu miễn phí không quy đổi.",
     "ID lượt bốc hoặc đơn đổi; lưu kết quả bốc cùng lần tiêu chìa khóa.",
     "Thử một lượt bốc và một lượt đổi bằng vật phẩm thử miễn phí."),
    ("seasonal-draws", "Đập trứng, ngọc phép và hộp quà", DRAW_TYPES,
     "Giới hạn theo loại lượt mở và ngày, cùng trần quà cho cả mùa.",
     "ID thực thể hoặc lượt mở và loại; phát lại nhận cùng kết quả đã lưu.",
     "Kiểm riêng UI của từng loại trước khi thử một lượt mở miễn phí của loại đó."),
    ("item-alchemy", "Hợp thành vật phẩm", ALCHEMY,
     "Giới hạn số lần hợp thành theo ngày và theo công thức.",
     "ID giao dịch và phiên bản các vật phẩm nguồn; không tái sử dụng nguyên liệu đã tiêu.",
     "Quan sát xem trước, một lần hợp thành và một lần dùng nguyên liệu không hợp lệ."),
    ("time-stop", "Dừng thời gian vật phẩm", ALCHEMY,
     "Giới hạn số lần đổi hạn theo ngày và số lượt mỗi vật phẩm.",
     "ID vật phẩm và phiên bản hạn dùng; phát lại không gia hạn thêm.",
     "Đo hạn trước và sau thao tác, kể cả trường hợp đã đổi hạn."),
)

CONSTRAINTS = {
    "no-real-money-monetization": "Không nạp tiền, bán lượt, thanh toán bằng tiền thật, rút tiền hay giao dịch có giá trị ngoài game. "
    "Trao đổi có giá cố định chỉ dùng tài sản trong game không quy đổi được.",
    "no-gambling-for-money": "Bốc thăm chỉ dùng lượt miễn phí hoặc vật phẩm không mua, chuyển nhượng hay quy đổi được. Quà ngẫu nhiên cũng không được quy đổi, đem bán hoặc dùng làm cược.",
}
DEFAULTS = {
    "enabled": False,
    "start_utc": None,
    "end_utc": None,
    "rate_caps": {"per_account_per_hour": 0, "global_per_minute": 0},
}
INVARIANTS = (
    "Mỗi phân hệ cần opt-in riêng, lịch UTC [start_utc, end_utc), giới hạn dương theo tài khoản/giờ (tối đa 60) "
    "và toàn server/phút (tối đa 600), "
    "và quota nghiệp vụ ở bảng dưới. Thiếu cấu hình, giới hạn bằng 0, chưa đạt gate, ngoài lịch hoặc không kiểm được quota thì từ chối trước khi đổi trạng thái. "
    "Bật hệ thống cha không bật phân hệ hay map.",
    "Server lấy account, mùa, chu kỳ và điều kiện đủ từ trạng thái có thẩm quyền; không dùng đồng hồ hoặc ID tài khoản do client khai. "
    "Các ngưỡng dương ngoài ràng buộc nêu rõ chưa được chọn và không lấy từ KR gốc.",
    "Khóa chống lặp có phần chung (system_id, account_id, season_id) và phần riêng trong bảng. ID thao tác/lượt phải do server cấp hoặc "
    "gắn với một sự kiện có thẩm quyền, không cấp lượt mới khi gửi lại cùng hành động. Cùng khóa và dữ liệu trả kết quả đã lưu, "
    "kể cả kết quả ngẫu nhiên; cùng khóa nhưng dữ liệu khác thì từ chối.",
    "Kiểm quyền, quota nghiệp vụ, ghi khóa, trừ đầu vào, cấp quà và đổi trạng thái bền vững phải nguyên tử; "
    "lần gửi lại không trừ quota nghiệp vụ hay cấp quà lần nữa. Giới hạn lưu lượng vẫn áp dụng cho lần gửi lại. "
    "Chuyển cảnh chỉ khóa theo phiên hiện tại và không ghi thưởng.",
)
WINE_GATE = (
    "Cô lập outbound, chỉ cho phép Wine/client kết nối server và capture cục bộ. Dùng bản sao KR 00967; ghi hash client, bản build/hash server, "
    "phiên bản Wine và mọi sai khác của bản sao. Không kết nối Nexon; nếu phải sửa bản sao thì ghi phép đối chứng, giới hạn và cách hoàn nguyên.",
    "Với từng opcode định bật, lưu capture từ thao tác thật trong client, chiều, độ dài và trường đã đối chiếu với bộ đọc/ghi KR "
    "(VA/RVA khi có); gắn ID run và hash capture. Đối chiếu một thao tác thành công với UI/trạng thái client trước và sau; chỉ có cửa sổ, "
    "kết nối TCP, log server hay gói tự sinh chưa đủ.",
    "Chạy đối chứng tắt và ngoài lịch, một trường hợp sai điều kiện hoặc chạm quota, và kiểm biên lịch/quota. "
    "Với thao tác ghi, phát lại, thử đồng thời và thử lại sau đăng nhập lại hoặc khởi động lại server. "
    "Xác nhận không trừ hoặc cấp lần hai, không đổi kết quả đã chốt và không ghi gì khi bị từ chối.",
    "Chỉ xét đúng opcode và ngữ cảnh đã đo. Nếu client không phát hoặc xử lý được gói tương ứng thì giữ tắt và ghi unknown. "
    "Quy tắc gốc và trạng thái còn hoạt động cũng giữ unknown khi chưa có bằng chứng; giá, thưởng và tỷ lệ cục bộ "
    "nếu có phải ghi là design. Lưu raw ở local/ và chỉ commit metadata cùng công thức tái lập.",
)
EXCLUSION = ("Không có đường opt-in, opcode, phần thưởng hay ngoại lệ trên LANBoom/self-hosted. "
             "Từ chối khóa cấu hình này kể cả khi enabled: false; không chạy probe để mở mục này.")

PRIVATE = re.compile(
    r"/(?:home|Users|root|var/lib|mnt|srv|tmp)/|\b[A-Z]:\\+Users\\+|\bocid" r"1\.|"
    r"-----BEGIN [A-Z0-9 ]*PRIVATE" r" KEY-----|"
    r"\b(?:gh[pousr]_[A-Za-z0-9]{16,}|github_pat_[A-Za-z0-9_]{16,}|"
    r"(?:AKIA|ASIA)[A-Z0-9]{16}|sk-[A-Za-z0-9_-]{16,}|"
    r"eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+)\b|"
    r"""\b(?:api[_-]?key|access[_-]?token|token|client[_-]?secret|password|secret)["']?\s*[:=]\s*[^\s,}]+""",
    re.IGNORECASE,
)
IP_LITERAL = re.compile(
    r"(?<![\w.])(?:[0-9]{1,3}\.){3}[0-9]{1,3}(?!\w|\.[0-9])"
    r"|(?<![\w:])(?=[0-9A-Fa-f:.]*:[0-9A-Fa-f:.]*:)[0-9A-Fa-f:.]+(?![\w:])"
)


class AuditError(ValueError):
    """Lỗi kiểm tra có thông báo không lộ dữ liệu đầu vào."""


def sha256(raw):
    return hashlib.sha256(raw).hexdigest()


def json_line(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False)


def repo_file(root, relative):
    path = root / relative
    for candidate in (path, *path.parents):
        if candidate == root:
            break
        if candidate.is_symlink():
            raise AuditError(f"{relative}: cần tệp thường trong worktree")
    if path.exists() and (not path.is_file() or path.stat().st_nlink != 1):
        raise AuditError(f"{relative}: cần tệp thường trong worktree")
    return path


def reject_private(body, label):
    if PRIVATE.search(body):
        raise AuditError(f"{label}: có đường dẫn máy hoặc dữ liệu riêng")
    for match in IP_LITERAL.finditer(body):
        try:
            address = ipaddress.ip_address(match[0].rstrip("."))
        except ValueError:
            continue
        if isinstance(address, ipaddress.IPv6Address):
            address = address.ipv4_mapped or address
        if not address.is_global or address.is_multicast:
            raise AuditError(f"{label}: có địa chỉ IP không công khai")


def read_inputs(root):
    data = {key: repo_file(root, relative).read_bytes() for key, relative in INPUTS.items()}
    # Chỉ băm dòng F6 ở đúng anchor; thay đổi ở dashboard không làm đặc tả cũ đi.
    anchor = b'<a id="full-game-roadmap"></a>'
    tail = data["roadmap_f6_row"].partition(anchor)[2]
    sections = re.split(rb"^#{1,3} ", tail, maxsplit=2, flags=re.M)
    rows = re.findall(rb"^\| F6 [^\r\n]*", sections[1] if len(sections) > 1 else b"", re.M)
    if len(rows) != 1:
        raise AuditError("PLAN cần đúng một dòng F6 trong mục full-game-roadmap")
    data["roadmap_f6_row"] = rows[0] + b"\n"
    return data


def unique_index(rows, key):
    result = {}
    for row in rows:
        identity = key(row)
        if identity in result:
            raise AuditError("Có hàng trùng trong dữ liệu F6")
        result[identity] = row
    return result


def grade(row):
    return {key: row[key] for key in ("kind", "name", "wave", "classification", "status")}


def build_spec(data):
    classification = json.loads(data["classification"])
    catalog = json.loads(data["catalog"])
    client_hash = classification["client_sha256"]
    if (classification["schema"] != "kre-content-classification-v1" or classification["source"] != "kr_static"
            or catalog["schema"] != "kre.f6-events-catalog.v1"
            or classification["build"] != BUILD or catalog["build"] != BUILD
            or not re.fullmatch(r"[0-9a-f]{64}", client_hash)
            or catalog["static_metadata"]["client_exe_sha256"] != client_hash):
        raise AuditError("Schema, build hoặc SHA-256 client của nguồn F6 không khớp")
    payload = {key: value for key, value in catalog.items() if key not in {"catalog_sha256", "input_sha256"}}
    if sha256((json_line(payload) + "\n").encode()) != catalog["catalog_sha256"]:
        raise AuditError("Hash nội dung catalog F6 không khớp")

    rows = unique_index(
        (row for row in classification["rows"] if row["wave"] in {"F6", "F6?", "F4/F6"}
         or row["name"].casefold() == EXCLUDED.casefold()),
        lambda row: (row["kind"], row["name"]),
    )
    catalog_rows = unique_index(catalog["systems"], lambda row: row["system"])
    scoped = {name for name, row in catalog_rows.items() if row["scope"] == "in_scope"}
    excluded = {name for name, row in catalog_rows.items() if row["scope"] == "excluded_policy"}
    if scoped != POLICY_RULES.keys() or catalog["summary"]["in_scope"] != len(scoped):
        raise AuditError("Tập in_scope của catalog và đặc tả không khớp")
    exclusion_grade = rows.get(("system", EXCLUDED), {})
    if (excluded != {EXCLUDED} or catalog["summary"]["excluded_policy"] != 1
            or {key for key in rows if key[1].casefold() == EXCLUDED.casefold()} != {("system", EXCLUDED)}
            or exclusion_grade.get("wave") != "ngoài phạm vi" or exclusion_grade.get("status") != "design"
            or exclusion_grade.get("classification", "") is not None
            or exclusion_grade.get("roadmap_only", False) is not False):
        raise AuditError("PCRoomBenefit phải là mục loại trừ duy nhất")

    selected = {key for key, row in rows.items() if row["wave"] in {"F6", "F6?"}}
    shared = ("system", SHARED)
    if (rows.get(shared, {}).get("wave") != "F4/F6" or rows[shared].get("roadmap_only") is not True
            or {key for key, row in rows.items() if row["wave"] == "F4/F6"} != {shared}):
        raise AuditError("Danh sách phụ thuộc chung F4/F6 đã đổi")
    policy_keys = {("system", name) for name in scoped}
    map_keys = {key for key in selected if key[0] == "map"}
    roadmap_keys = selected - policy_keys - map_keys
    if (not policy_keys <= selected
            or any(rows[key].get("classification") != "C" or rows[key].get("status") != "candidate"
                   for key in selected | {shared})
            or any(rows[key]["wave"] != "F6" or rows[key].get("roadmap_only", False) is not False
                   for key in policy_keys)
            or any(key[0] != "system" or rows[key].get("roadmap_only") is not True for key in roadmap_keys)):
        raise AuditError("Các hàng F6 không tách được thành hệ thống policy, roadmap_only và họ map")
    for key in policy_keys:
        catalog_grade = catalog_rows[key[1]]["classification"]
        if any(rows[key][field] != catalog_grade[field] for field in ("wave", "status", "classification")):
            raise AuditError("Xếp loại hệ thống in_scope đã lệch catalog F6")

    expected_sources = roadmap_keys | map_keys | {MISSION, FISHING, ("system", "LuckyCard"), ("system", "MagicMarket")}
    if {entry[2] for entry in EXTENDED} != expected_sources or len({entry[0] for entry in EXTENDED}) != len(EXTENDED):
        raise AuditError("Các phân hệ mở rộng thiếu hàng xếp loại, có hàng dư hoặc trùng ID")
    if any(EXCLUDED.casefold() in field.casefold() or field in scoped
           for identifier, name, *_ in EXTENDED for field in (identifier, name)):
        raise AuditError("Tên phân hệ mở rộng trùng mục loại trừ hoặc hệ thống policy")

    maps = []
    for key in sorted(map_keys):
        row = rows[key]
        counts = {field: row[field] for field in ("map_count", "script_count", "script_unknown")}
        if (row["wave"] != "F6?" or row["category"] != "lobby" or row["base"] != row["name"]
                or any(type(number) is not int for number in counts.values())
                or not 0 <= counts["script_unknown"] <= counts["script_count"] <= counts["map_count"]
                or not counts["map_count"]):
            raise AuditError("Metadata họ map F6? không hợp lệ hoặc đã đổi đợt")
        maps.append({**grade(row), **counts})

    policy = [
        {**grade(rows[("system", name)]), "scope": "in_scope", "enabled": False,
         "quota": POLICY_RULES[name][0], "idempotency": POLICY_RULES[name][1]}
        for name in sorted(scoped)
    ]
    extended = [
        {"id": identifier, "name": name, "scope": "roadmap_only", "enabled": False,
         "classification_ref": {"kind": ref[0], "name": ref[1]}, "quota": quota,
         "idempotency": replay, "wine_probe": probe,
         **({"policy_context": "Pet"} if ref == PET_CARE else
            {"policy_context": ref[1]} if ref in policy_keys else {})}
        for identifier, name, ref, quota, replay, probe in EXTENDED
    ]
    return {
        "schema": "kre.f6-extended-events-spec.v1",
        "build": BUILD,
        "client_sha256": client_hash,
        "source": "design",
        "status": "design",
        "limits": {"kr_wire": "unknown", "original_server_prices_rewards_and_rates": "unknown",
                   "active_in_build": "unknown", "wine_acceptance": "not_run"},
        "sources": {key: {"path": path, "sha256": sha256(data[key]),
                          **({"locator": "full-game-roadmap: dòng F6, thêm LF cuối"} if key == "roadmap_f6_row"
                             else {})} for key, path in INPUTS.items()},
        "summary": {"in_scope": len(policy), "roadmap_classification_systems": len(roadmap_keys),
                    "map_families": len(maps), "maps": sum(row["map_count"] for row in maps),
                    "roadmap_only": len(extended), "excluded_policy": 1},
        "defaults": DEFAULTS,
        "constraints": CONSTRAINTS,
        "invariants": INVARIANTS,
        "wine_gate": WINE_GATE,
        "in_scope": policy,
        "roadmap_only": extended,
        "roadmap_classification": [{**grade(rows[key]), "roadmap_only": True} for key in sorted(roadmap_keys)],
        "map_families": maps,
        "shared_dependencies": [{**grade(rows[shared]), "roadmap_only": True, "enables_f6": False}],
        "excluded_policy": [{**grade(exclusion_grade), "scope": "excluded_policy", "enabled": False,
                             "enableable": False, "rule": EXCLUSION}],
    }


def render_json(spec):
    # Một hàng mỗi dòng để review thay đổi scope và quota.
    fields = []
    for key, value in sorted(spec.items()):
        if isinstance(value, list):
            rows = ",\n".join("    " + json_line(row) for row in value)
            fields.append(f'  "{key}": [\n{rows}\n  ]')
        else:
            fields.append(f'  "{key}": {json_line(value)}')
    return "{\n" + ",\n".join(fields) + "\n}\n"


def render_markdown(spec):
    count = spec["summary"]
    lines = [
        "# Kiểm kê và đặc tả sự kiện theo mùa, chăm sóc pet F6 mở rộng", "",
        f"Build `{BUILD}`: **{count['in_scope']} hệ thống `in_scope`**, **{count['roadmap_only']} phân hệ "
        f"`roadmap_only`** và **{count['excluded_policy']} mục `excluded_policy`**. Các phân hệ là lát cắt công việc: "
        f"chúng chi tiết hóa {count['roadmap_classification_systems']} nhóm bổ sung và một số hệ thống sẵn có, "
        f"đồng thời bao phủ {count['map_families']} họ `F6?` ({count['maps']} map). "
        "Con số khoảng 25 trong PLAN là ước lượng, không phải số hàng mới của bảng xếp loại.", "",
        "Nguồn: [xếp loại](classification.json), [catalog F6-0](f6-events-catalog.md), "
        "[policy F6-1](f6-events-policy.md), [policy Rust](../../rust/crates/kr-service/src/f6_policy.rs) "
        "và [lộ trình F6](../PLAN.md#full-game-roadmap). "
        "[JSON](../evidence/content/f6-extended-events-spec.json) giữ tên hàng gốc, locator và SHA-256 đầu vào. "
        "Catalog F6-0 giữ snapshot xếp loại trước khi PR #713 mở rộng lộ trình; đối chiếu tập hệ thống theo từng hàng. "
        "Năm mục `related_candidate` của catalog vẫn thuộc F2, F3 hoặc F4.", "",
        "Tên tài nguyên và số map là metadata tĩnh (`kr_static`); các nhóm lộ trình và liên hệ theo tên là `candidate`. "
        "Quota, khóa chống lặp và gate bên dưới là `design`. Chưa chạy Wine cho đặc tả này. "
        "Opcode, layout, giá, thưởng, tỷ lệ gốc và trạng thái hoạt động trên KR 00967 đều là `unknown`.", "",
        "## Mặc định và bất biến áp dụng chung", "",
        "Mặc định `enabled: false`, `start_utc: null`, `end_utc: null` và hai `rate_caps` đều bằng 0. "
        "JSON này là đặc tả, không phải profile hợp lệ để nạp vào Rust.", "",
        *[f"- {rule}" for rule in spec["invariants"]],
        *[f"- `{key}`: {rule}" for key, rule in spec["constraints"].items()], "",
        "## Các hệ thống trong policy hiện tại", "",
        "Các tên dưới đây là đúng tập `SYSTEM_NAMES` của Rust, được đối chiếu trong test. "
        "Rust kiểm opt-in và lịch (bao gồm cả hai mốc); tài liệu policy mô tả khoảng loại mốc cuối. "
        "Khoảng nửa mở ở đặc tả này là đề xuất cho phần mở rộng. Việc thực thi các quota và khóa riêng bên dưới thuộc handler tương lai.", "",
        "| Hệ thống `in_scope` | Quota nghiệp vụ đề xuất | Phần riêng của khóa chống lặp |",
        "|---|---|---|",
    ]
    for row in spec["in_scope"]:
        lines.append(f"| {row['name']} | {row['quota']} | {row['idempotency']} |")

    lines += ["", "## Phân hệ mở rộng", "",
              "Mỗi dòng có ID đặc tả riêng và chưa có khóa cấu hình trong policy Rust. "
              "`Dwarf/libcef` là phụ thuộc chung F4/F6, không bật tính năng theo mùa. "
              "`LobbyPet` giữ nhãn `F5?` và không được tính vào F6.", "",
              "| Phân hệ `roadmap_only` | Hàng xếp loại | Quota nghiệp vụ đề xuất | Phần riêng của khóa chống lặp | Probe Wine riêng |",
              "|---|---|---|---|---|"]
    for row in spec["roadmap_only"]:
        ref = row["classification_ref"]
        lines.append(f"| {row['name']} | `{ref['kind']}`: {ref['name']} | {row['quota']} | "
                     f"{row['idempotency']} | {row['wine_probe']} |")

    lines += ["", "| Họ map `F6?` | Map | Script chưa rõ / tổng |", "|---|---:|---:|"]
    for row in spec["map_families"]:
        lines.append(f"| `{row['name']}` | {row['map_count']} | {row['script_unknown']}/{row['script_count']} |")
    lines += [
        "", "## Loại trừ bắt buộc", "",
        f"`{EXCLUDED}` (ưu đãi phòng máy Nexon) là `excluded_policy`. {EXCLUSION}",
        "", "## Gate Wine trước khi mở opcode", "",
        *[f"{index}. {step}" for index, step in enumerate(spec["wine_gate"], 1)],
        "", "## Tái lập offline", "", "```bash",
        "python3 tools/kr_f6_extended_events_audit.py --write",
        "python3 tools/kr_f6_extended_events_audit.py --check", "```", "",
        "Công cụ chỉ đọc các đầu vào tài liệu và JSON đã lưu trong repo; `--check` so khớp byte của cả hai đầu ra. "
        "SHA-256 băm byte gốc, ngoại trừ PLAN chỉ băm dòng F6 và một LF cuối.", "",
    ]
    return "\n".join(lines)


def audit(root, *, write):
    spec = build_spec(read_inputs(root))
    outputs = {MARKDOWN: render_markdown(spec), EVIDENCE: render_json(spec)}
    paths = {}
    for relative, body in outputs.items():
        reject_private(body, relative)
        paths[relative] = repo_file(root, relative)
    stale = []
    for relative, body in outputs.items():
        path = paths[relative]
        expected = body.encode("utf-8")
        if write:
            path.write_bytes(expected)
        elif not path.is_file() or path.read_bytes() != expected:
            stale.append(relative)
    if stale:
        raise AuditError(f"Chưa đồng bộ: {', '.join(stale)}; chạy lại với --write")
    return spec["summary"]


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="sinh Markdown và JSON")
    mode.add_argument("--check", action="store_true", help="đối chiếu đầu ra mà không ghi")
    args = parser.parse_args(argv)
    try:
        count = audit(ROOT, write=args.write)
    except AuditError as error:
        print(f"F6 mở rộng: {error}", file=sys.stderr)
        return 1
    except (OSError, ValueError, KeyError, TypeError, AttributeError):
        print("F6 mở rộng: đầu vào sai định dạng hoặc không thể đọc/ghi tệp", file=sys.stderr)
        return 1
    print(f"F6 mở rộng {'--write' if args.write else '--check'} OK: "
          f"{count['in_scope']} in_scope, {count['roadmap_only']} roadmap_only "
          f"({count['map_families']} họ map), {count['excluded_policy']} excluded_policy.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
