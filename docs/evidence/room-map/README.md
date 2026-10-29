# CP019 — map metadata và request start thực tế

Contract duy nhất: [PROTOCOL §map](../../PROTOCOL.md#room-map). Trạng thái/gate và thứ tự công việc:
[PLAN](../../PLAN.md). Câu hỏi tiếp: [RESEARCH](../../RESEARCH.md#q-023).
Metadata/SHA từng file trong [CP019 manifest](../checkpoints/manifest.json); raw không commit.
Baseline `8cd42ce8e909e73a5dc5e9ce613c04084f05f14b` (CP018/PR26 đã merge).
Codex solo, tự kiểm; review độc lập **waived**, không có approval độc lập. Tích hợp theo PR của nhánh
`kre/room-map-runtime`; record là snapshot trước tích hợp. Audit định kỳ kế CP020.

## Target, provenance và recipe

Runtime dùng nguyên copy CP018 `CA-r2-room-keystate.exe`, SHA256
`94ee93dc7978a1ab22f25ec877e29ddcab254ca761f2eac22534421e2dd628e8`;
CP019 không thêm patch. Các thay đổi chẩn đoán/API candidate kế thừa có provenance ở
[evidence CP018](../room-player/README.md); không coi tương đương vanilla Windows.
Static reader dùng copy `CA-r2-login-logout.exe`, SHA256
`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
các cạnh code đang đọc chung với runtime copy, khác import bindings đã ghi CP018.
Native oracle `boomkr-re/out/modO.bin`, base400000, SHA256
`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`.
VA trong raw, RVA=VA−400000. REA native provider chưa dùng được theo mốc trước;
phép đo dùng GDB/Capstone/Unicorn hiện có, không sửa cấu hình/DB nguồn.

Raw root `local/research/codex/room-map-r1/`; harness ở `local/research/codex/`.
Mỗi run có `*-recipe.json` argv đầy đủ, `run.json` PID/timeline/interface/routes/peer/hash,
`client.log`, `debug-commands.txt`, CALog, ảnh và `*-hashes.json` phủ raw. Tám run tuần tự,
namespace user/net/mount/PID chỉ lo, không route outbound, NSS files-only, peer loopback.
Prefix riêng mỗi run; game-dir/cache chung **không chuẩn hóa**. Snapshot cache riêng giữ hash;
không sửa file client/DB nguồn hoặc liên hệ Nexon thật.

Ví dụ tái tạo phép thử đã lưu (tạo run ID mới để không ghi đè raw CP019):

```bash
python3 local/research/codex/room-map-r1/run_probe.py NEW-RUN CA-r2-room-keystate.exe 32 wine_probe_map_enabled238.py local/research/codex/room-map-r1/map-enabled.gdb
```

Phải đổi output/prefix và bảo toàn raw cũ trước khi chạy. Harness mới nối stub/86/106/110/team của
CP015–018; `wine_probe_map_ack117.py`, `wine_probe_map_table238.py`,
`wine_probe_map_enabled238.py` chỉ bổ sung ack hoặc table/config mới.
Fixture là **emu_choice**. Native parser, runtime cursor/callback và outgoing capture là oracle KR;
encoder tự sinh không chứng minh schema/server policy.

## Phép thử gom trong checkpoint

| Run | Kết quả và giới hạn |
|---|---|
| map-control | Roster hai đội kế thừa; map capacity0, modal callerDCC37A. |
| map-ack117 | Ack đọc8/8, errors0, pending0 nhưng dynamic-count0; ack không tạo table. |
| map-list | Capacity0; probe init bị lặp nên không dump list. Giữ lỗi phép đo. |
| table238 | GDB mất `$prepare`, lỗi predicate tạiDDF330; chưa gửi table/ack. Không gọi lỗi client. |
| table238-fixed | Sửa probe; checksum native/table đọc67/67, count1; UI nhãn/capacity4, start CantDecideMap. Dump local list1151 record. |
| map-filter | Players2/teams2/capacity4; E00540/E06F80 trả0. |
| filter-branches | Mode0/default predicate0; các predicate sau chưa chạy. |
| map-enabled | Chỉ inner[52]0→1 và checksum/config; predicate1, selectable1, start-check0, send1/pending3; capture104/109byte. |

`map-enabled` vẫn stage12; stub chưa phản hồi start, không có loading/Battle/hai client.
Ảnh cuối `map-enabled/screen-16.png` và `screen-18.png`; đối chứng
`table238-fixed/screen-15.png`. Chữ Village01 là nhãn fixture; code0 trong snapshot KR là
Antarctic01/capacity4, không dùng nhãn làm bằng chứng nạp map/geometry.

## Kiểm, sửa giả thuyết và raw

`verification.json`: **2033 kiểm cơ học / 8 run pass**. Hash/isolation/pinnedcopy/harness,
frame capture replay native CRC/partial negatives; 18 ca framing fixture; native full config241,
blob/checksum/17 getters row238 và native compressed blob/tail của capture104.
`verify.py` tổng hợp; `replay-run.py`, `verify-map*.py`, `verify-config241*.py`,
`verify-start104.py` giữ implementation riêng. Check không tự xác nhận ngữ nghĩa hoặc đóng gate.
Allocator/free được mô hình hóa; checksum405E70/inflate/reader giữ native.
Oracle104 dùng arena1MiB, allocation≤64KiB (inflater cần11520/32768byte); context265byte vẫn opaque.

`measurement-corrections.json` giữ đầy đủ lỗi và thay đổi so với CP018:
giả thuyết Adler trên tag/header bị bác bỏ bằng native checksum inner-only;
config241 raw blob cũ thiếu selector không được dùng làm proof full schema;
probe13ADD10 không phủ direct-send13ADD60; dump /tmp mất sau namespace và range disasm lệch
instruction được giữ dưới dạng navigation failure. Không sửa raw/checkpoint đã complete.

`cached-map-before238.ssd` là cache kế thừa: parser đọc1090row/CRC khác expected và gửi528;
cache synthetic ghi lại rồi reload cũng khác checksum. Chưa giải persistence/cold/warm/reconnect.
`playmap-list.bin`59852byte (1151×52), SHA256
`49727b4d68bca96e7a71ea842c9fe03fe4fd92f032e8510bbcade62b2b796714`;
summary chỉ giải SSO inline, không theo heap string dài.
`captured-start104.bin` lấy socket4/frame28/seq124, wire109/payload71; native CRC/blob inflate265
và tail đọc71/71. SHA capture, fixtures, scripts/logs/ảnh và snapshot nằm trong manifest.

Nguồn VN `vn-map-reuse.json` khóa hash tài liệu lobby/packets tham chiếu: chỉ dẫn câu hỏi
config/table/ack/start/load. Không áp VN opcode, row128 hoặc map index954 lên KR.
Các field còn unknown và phép thử kế chỉ giữ ở RESEARCH, contract ở PROTOCOL.
