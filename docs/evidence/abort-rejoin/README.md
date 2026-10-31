# Phục hồi sau EOF, client mới và chủ mới Start — CP040

Baseline CP039/[PR47](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/47) đã merge
`e19bc9e9e255d160efc90a68474cab742f360781`. CP040 là snapshot trước tích hợp trên nhánh
`kre/abort-rejoin-runtime`; PR của nhánh giữ trạng thái tích hợp hiện tại.
Codex solo/tự kiểm; review độc lập [waived](../checkpoints/review-waiver.json).
[Contract duy nhất](../../PROTOCOL.md#abort-rejoin), [câu hỏi/bước tiếp](../../RESEARCH.md#q-023),
[AUD013](../checkpoints/audits/AUD013.json) định kỳ CP040, audit kế CP044. Không gate mới đóng.

Raw/recipe/harness/GDB/fixtures/log/capture/native report/ảnh:
`local/research/codex/abort-rejoin-r1/`. Năm controller tuần tự, lease trong namespace
user/net/mount/PID, chỉ loopback/no outbound/files-only NSS/local stub. Mỗi client prefix,
wineserver, display và game-dir riêng. Binary/pack/DB nguồn bất biến; không thêm patch hoặc ép
actor/state. Copy kế thừa CP020 import/API/AC/guard và initial global reset; không vanilla Windows.

| Artifact | SHA256 |
|---|---|
| Runtime A/B/C CA-r2-stage-social.exe | `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef` |
| Static CA-r2-login-logout.exe | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` |
| modO raw memory/base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |

RVA=VA−400000; PE copy mapped theo section, modO đọc raw memory. `code-match.json` kiểm14vùng/5artifact
kể cả C; byte equality không là proof thực thi. Dùng cạnh cached và WineGDB/Unicorn,
không lặp cold REA/IDA timeout CP024 hoặc quét toàn client.

| Run / UTC31/10/2026 | Kết quả | run.json SHA256 |
|---|---|---|
| replacement-first / 07:35:37 | Collector NameError trước TCP; không protocol/client negative. | `f697a8ae08635e233b18e2997768295252b6819fda1ec95267d123628397d9a4` |
| replacement-fixed / 07:36:47 | C ID3 Join/Ready; B chief; một lần bấm chưa104 hết320s, chưa Start. | `612422400b84dbbbea0febbe261db51bd200a5435b8d58759d05e798d8ff769a` |
| replacement-retry / 07:43:36 | Một retry rồi actual B104 →121/99/clock/movement B/C. | `d8a37567ff7378ccb0d60a0e47c2f207282e33e70d2e61d541b9889fa7dedd8a` |
| guest-EOF-control / 07:49:38 | BchannelEOF; A vẫn trận/di chuyển, Bghost; không result/cleanup. | `d5f886bcc1a9a3927f14fd84aed23d2aecdacc5b2010081b171093590ec30c2c` |
| guest-EOF-policy / 07:53:55 | EOF B →175/180 A →room107 B/110 A; A vẫn chief. | `8129c8df47684253ce31c0421d761fea17e2cdcc7570283ac8b10dc07c13999a` |

`prepare.py` tạo fixture C106 slot0/ID3 bằng sửa identity trên profile unready đã kiểm;
C nhận own106seq58, Bchief106seq64; B nhận dynamic106 theo seq hiện tại103.
Tên C vẫn KRLocal, khác identity A dù tên giống. Cauth dùng prefix ID3 đã kiểm CP037,
không full authentication/account reconnect. 106 native chỉ kiểm **prefix5byte slot/ID**,
không ghi full profile parser pass. Runtime own-ID/slot/own-match/ảnh và Ready/Start là nguồn riêng.

`replacement-retry` dùng cùng fixture/code với fixed, thêm tối đa3 retry cách6s;
thực tế một retry rồi có104. Không quy nguyên nhân lần bấm đầu cho focus/timing/protocol rejection.
Harness nhận actual104 từ B mới phát582/121 cho B/C, archive/reset marker B,
tra actual99 cho endpoint153 mới; A đã EOF không thuộc peer set vòng2.
`derived/` replay toàn actual C2S theo từng role/socket, CRCbad/partial;
`native-packets.json` giữ field/blob/checksum cho actual82/95 và selected107/110/175/180/582/106.
Fixture S2C được stub chọn; native parser không là oracle server gốc.

Guest policy chỉ kill Bprefix, chờ EOF đúng channelB rồi chọn Awin/Blose sau12s,
180 sau5s, room107/110 sau3s. Không giả report19; secure actorB state0 quanh rank/return.
Metadata cũ `host_disconnect` và ảnh `B-after-EOF-input` là nhãn kế thừa; role B bị kill,
role A nhận input/quan sát. 107 đo base6 rồi intentional rewind/final1; Achief1 không promotion sai.
`actor-fields.json` giữ actual secure state/coordinates/stat, checksum/native52A350/sourcehash.

Ảnh đã đọc thủ công: fixed2, retry9, guestcontrol3, guestpolicy4 (18ảnh ghép).
Ảnh ngay lúc gửi rank/cleanup có thể chưa qua callback; ghép trace với ảnh trễ cuối run.
Retry trước/sau input cho thấy cả hai view cùng vị trí mới của B/C. Control guestEOF còn Bghost;
policy cuối chỉ A trong phòng. Không dùng UI/clock/inputreturn0 riêng lẻ làm proof gameplay.

Recipe chính xác tại `<run>-recipe.json`; mọi rawfile/size/hash tại `<run>-hashes.json`.
`verify.py`: **7224 kiểm cơ học pass**, 5sealedruns gồm1failedcollector/1incompleteStart,
159nativepacketcases/212actualsecure snapshot/checksum/getter,14ranges/5artifact.
Không cộng số kiểm thành tiến độ gameplay; checker không tự đọc nghĩa hoặc đóng gate.
Các script/report/index/raw được pin trong [CP040manifest](../checkpoints/manifest.json).

Tái lập bằng label/prefix mới; không đè sealedrun, không chạy controller đồng thời:

```bash
python3 local/research/codex/abort-rejoin-r1/run_probe.py new-replacement CA-r2-stage-social.exe 310 wine_probe_abort_replacement_retry.py local/research/codex/abort-rejoin-r1/recovery.gdb
python3 local/research/codex/abort-rejoin-r1/seal_run.py new-replacement
python3 local/research/codex/abort-rejoin-r1/run_probe.py new-guestEOF CA-r2-stage-social.exe 175 wine_probe_round_guest_eof_policy.py local/research/codex/abort-rejoin-r1/recovery.gdb
python3 local/research/codex/abort-rejoin-r1/seal_run.py new-guestEOF
```

Giới hạn: fresh C không A reconnect/session resume; B/C mới tới movement, chưa bom/death/result.
Forfeit/outcome/timing/profile/slot reuse/session là emu_choice/design; không authority gốc.
Chưa full auth/cache/codec/vanilla, ID allocator/stale/generation/allentity/timer/peerqueue reset,
hai client cùng mất, NAT/relay/LAN/Internet hoặc fullmatch/K5/K6a.
Kết quả CP037–039/raw cũ giữ nguyên như lịch sử; AUD013 đối chiếu các bổ sung và unknown hiện hành.
