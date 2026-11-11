# Boom KR Server

Server giả lập cho **the lastest client Crazy Arcade KR build00967 before end of service**, dựa trên kiến thức từ client
VN VBNB_116. 
Chú ý:
1- dự án không cung cấp client game
2- dự án chứa lượng lớn text và kiến thức RE được viết bằng agents, ko phải con người, nếu đọc không hiểu thì hãy hỏi AI giải thích. 
<img width="1024" height="1230" alt="fixed-contact" src="https://github.com/user-attachments/assets/739b9237-b44a-47f0-87a3-97ccd3c9f669" />

**Thư mục làm việc hiện tại trên host:** `/home/admin/boom/repos/boom-kr-server`.
Docs, tools, code và evidence mới đều ở đây. Không cần mở worktree của repo Rust cũ để làm project này.

```text
boom-kr-server/
  AGENTS.md                 quy tắc của project này
  docs/
    PLAN.md                 lộ trình K0–K7, task và gate
    features.json           chỉ mục cơ chế/ID/dependency/trạng thái có nguồn
    COVERAGE.md             bảng/cây/thẻ tra cứu sinh từ catalog
    PROTOCOL.md             contract KR, ma trận reuse VN→KR và authority
    RESEARCH.md             câu hỏi mở, owner và điều kiện đóng
    REFERENCES.md           nguồn RE cũ, provenance và bản đồ migration
    evidence/               metadata/hash; không chứa binary hoặc decompile
  tools/                    harness, probe và kiểm tra artifact/docs
  rust/                     kr-wire + kr-service; lát cắt Rust từng bước
  server/                   Python serving/RE/reference vẫn giữ
  local/                    raw/copy client/DB/capture; git-ignored
```

**Đích bàn giao giai đoạn 1 (chốt 11/11):** dịch vụ nội bộ bền vững, người chơi qua Internet thật, khoảng 50 người đồng
thời, chế độ thường với nhiều map và item phổ biến. Dài hạn vẫn hướng tới gameplay đầy đủ.
[Đích, việc còn lại và thứ tự](docs/PLAN.md#delivery-target).

## Tra nhanh và trạng thái hiện tại

**CP263: R1c đã tách player identity khỏi luật cấp slot và dùng chung template 106.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-r1c-player-slots), [bằng chứng](docs/evidence/rust-service/r1c-player-slots.json).
Mặc định legacy giữ byte; `first_vacant` cấp slot độc lập, người tạo là host. Runtime không chọn fixture theo alias identity.
94 test Rust, 305 ca TCP và 3.189 frame phòng cũ khớp baseline CP262; 30 fixture 106 không có chênh lệch ngoài trường đã biết.
Ca TCP mới kiểm player 2 tạo phòng, player 1 vào slot 1, chuyển host/vào lại và năm guard trước bind.
[PR337](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/337) đã merge R1b; tiếp R1d theo PLAN.
Chưa chứng nhận native binary mới, chưa general profile/full M5 hoặc shared-NAT; **N1 vẫn mở** điều kiện uplink.

**CP262: R1b đã chuyển luật phòng sang chính sách có tên, giữ byte của ba profile hiện có.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-r1b-policies), [bằng chứng](docs/evidence/rust-service/r1b-policies.json).
`ResultPolicy`, `DeparturePolicy`, `BombPolicy`, `LifePolicy`, `TeamPolicy` thay các cờ nội bộ; fixture `rank` chỉ cấp bytes.
90 test Rust/fmt/clippy/locked build,298ca TCP mỗi binary và3.189frame phòng khớp byte/hash với baseline.
JSON cũ giữ nguyên; chưa đổi identity/slot/registry/transport của R1c–g hoặc nghiệm thu native lại binary mới.
Tiếp R1c; **N1 vẫn mở** vì chưa xác nhận môi trường di động/ISP-CGNAT, R1f vẫn phải sửa ca chung NAT.

**CP261: hai KR thật đã qua account → phòng/basic → hai kết quả đối nghịch/reset → Start3 trên đường Internet đã đo.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-internet-observation), [bằng chứng](docs/evidence/rust-service/internet-observation.json).
Native login field51 điều khiển ngưỡng chờ channel: fixture0 ghi đè cache thành0 gây timeout; control30000 vào sảnh ở cảhai.
Control là policy server được khai rõ, clientcopy4b giữ nguyên. Runtime ARM/source giữ CP260; frontendTTL120 là config đã hỗ trợ.
752TCP/106reply/91raw,78nativeUDP ingress/77helper-apply/78join các capture,2.286copiedgetters,cleanupđạt.
Cổng A tự báo99 khác cổng sau NAT; B thực gửi về nguồn quan sát được, không chỉ cổng153. Ca chungNAT: A giữ sảnh8s,
B đúng frontend/session nhưng bị binding theoIP/metadata từ chối; R1f còn phải sửa.
**N1 chưa đóng:** chưa có nguồn xác nhận uplink di động/CGNAT của ISP như môi trường tối thiểu trong PLAN; không suy từ IP.
Kết quả chỉ chứng nhận đường đã đo; chưa chứng nhận mọi NAT/ISP hoặc sharedNATplay. K6c/K7scopegiữ; detailsVPS chỉlocal.

**CP260: tiền kiểm Internet đã thông; native account qua NAT nhận auth thành công, N1 vẫn mở.**
[Contract và giới hạn](docs/PROTOCOL.md#rust-controlled-network), [bằng chứng](docs/evidence/rust-service/controlled-network.json).
Rust có chế độ mạng controlled với allowlist và mapping metadata login tường minh; quyền account/session vẫn theo transport thật.
90 test Rust, hai bộ 59 ca account TCP, 9 guard mạng và 5 guard metadata đạt. Hai KR/Wine nhận auth result0 qua mạng thật,
nhưng lỗi chờ channel trước own291/phòng. R7 xác nhận modal timeout/đóng client xảy ra trước HTTP JSON rỗng;
lỗi HTTP là nhánh phát sinh sau đó, chưa phải nguyên nhân mất channel. Guard cũ cũng chưa giúp vào sảnh.
Chưa có 99/153, UDP gameplay, hai kết quả/reset hoặc ca chung NAT; chưa chọn direct/học endpoint/relay.
Thông tin hạ tầng/raw giữ riêng trong `local/`. **Internet thật chưa kiểm ở mức game KR**; K6c/K7 WAN giả lập giữ nguyên.

**CP259: đã đồng bộ main PR329/PR330 và chuyển ưu tiên sang N1 theo chỉ đạo mới.**
[Quyết định/giữ WIP](docs/evidence/owner-directives/pr329-integration-order.json), [kiểm sau gộp](docs/evidence/rust-service/pr329-integration.json).
Fmt, clippy, 86 test Rust, locked build và 59 ca account TCP đạt; 143 frame phòng khớp hash với CP258.
Bằng chứng native CP258 vẫn gắn binary đã đo, chưa tự chứng nhận binary sau `RoomRules`.
WIP projector được giữ trong snapshot/stash, chưa chạy native hoặc merge; thứ tự ở [PLAN](docs/PLAN.md#current-order).


### Mốc trước — phạm vi CP258 vẫn giữ nguyên


**CP258: account → phòng cố định → hai trận basic/kết quả/reset → Start3 đã chạy trên hai KR thật.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-account-basic), [raw/hash](docs/evidence/rust-service/README.md#cp258).
Account sở hữu đúng ID/profile/channel/slot; revoke/expiry/disconnect khi bom chờ đưa survivor về phòng, không nổ muộn.
Manualfresh B dùng process/backend/frontend mới vào lại slot và trận mới; UDP nguyên bản cũ bị native key-drop, currentmovement áp dụng.
5run/280raw/375exactTCP/cleanup0;84Rust/59account-basicTCP/16lobbyregression. AffectedB revoke/expiry có lỗi phiên/Winefatal,
**chưa graceful recovery; M5 tổng thể vẫn mở** (generalprofile/nativeDBqueuefault). M6/M7/M8 riêng, không dependency lát cắt này.
Audit nhóm AUD076; không lịch4CP. Internet thật chưa kiểm; VPS preflight riêng chưa là nativeInternetE2E.

### Các mốc trước

Đã chuyển sang [docs/HISTORY.md](docs/HISTORY.md). Hiện trạng và việc kế tiếp ở [Bảng điều hành](docs/PLAN.md#dashboard).

## Bắt đầu

Đọc [AGENTS](AGENTS.md), [PLAN](docs/PLAN.md), rồi [PROTOCOL](docs/PROTOCOL.md) và
[RESEARCH](docs/RESEARCH.md) theo task.

```bash
python3 tools/feature_catalog.py
python3 tools/check_docs.py
python3 tools/check_checkpoints.py --require-local --require-history
python3 tools/artifacts.py verify
```

`artifacts.py verify` dùng manifest **host-local** trong `docs/evidence/artifacts.json`; client/DB
không nằm trong Git nên clone sạch trên máy khác chưa thể chạy lệnh này. Khi chuyển máy, dùng
`--manifest /path/to/local-artifacts.json` với đường dẫn nguồn và hash đã xác nhận tại máy đó.

Client nguồn giữ ở `/home/admin/boom/clients/kr-00967`; không sửa hoặc commit. Artifact và database
đã unpack ở repo RE cũ chỉ là đầu vào tham chiếu; mỗi phép thử dùng bản copy riêng trong `local/`.
Nguồn/commit và giới hạn bằng chứng: [REFERENCES](docs/REFERENCES.md).
