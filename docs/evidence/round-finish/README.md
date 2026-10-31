# Kết quả hai vòng và host EOF — CP039

Baseline [CP038/PR46](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/46) merge
`1fb2c667d94d72ddcef1526445df37d1e49c1d1b`. CP039 snapshot trước tích hợp;
PR nhánh `kre/round-result-runtime` là nguồn trạng thái tích hợp.
Codex solo/tự kiểm; review độc lập [waived](../checkpoints/review-waiver.json).
[Contract](../../PROTOCOL.md#round-finish), [thứ tự tiếp/gate](../../RESEARCH.md#q-023).
AUD012 giữ nguyên; audit định kỳ kế CP040. Không gate mới đóng.

Raw/harness/GDB/fixture/capture/log/native reports/ảnh ở `local/research/codex/round-finish-r1/`.
Ba controller tuần tự với lease trong namespace user/net/mount/PID, chỉlo/no outbound/files-only NSS/stub.
Mỗi client prefix/wineserver/display/game-dir riêng. Chỉ bản copy chẩn đoán Wine; không thêm binarypatch/ép actorstate;
kế thừa CP020 import/API/AC/guard và initialglobalreset, không vanilla. Nguồn binary/pack/DB giữ nguyên.

| Artifact | SHA256 |
|---|---|
| Runtime A/B CA-r2-stage-social.exe | `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef` |
| Static CA-r2-login-logout.exe | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` |
| modO memory/base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |

RVA=VA−400000; modO đọc raw memory, không PE-remap. `code-match.json`14 vùng/4artifact;
rank175/return180/parser/report21/selfescape/actor getter đã có cạnh từ các checkpoint trước;
không quét lại client hoặc cold REA/IDA timeout. Cache/Capstone/Unicorn/WineGDB dùng đúng phạm vi.

`finish-first`: actual19 và21 trước175,180/roomrefresh/actual95/104/121/99/153 vòng2;
vòng1 có một kim/hai bom, vòng2 itemlist rỗng/một bom. Outcome Alose/Bwin, delay1758s sau19,
return5s/refresh3s, spawn/fixture/fuse6s/allocatorID là **emu_choice/design**.
Không coi việc thấy21 là bằng chứng server gốc dùng cùng điều kiện chấm kết quả.
`finish-first-visual.png` đã đọc8ảnh: cùng LOSEKRLocal/PERFECTKRGuest với ownhighlight,
phòng sau cả hai180. Aownavatar ở phòng là silhouette trắng trong khi B thấyAđỏ;
nguyên nhân cosmetic/profile/animation còn unknown, không giấu là toàn reset sạch.

`host-EOF-control`: sau153/clock hai phía, chỉ kill wineserver thuộc prefixA;
stub ghi channelEOF thật, không gửi107/175/180 hoặc report19 giả. B còn trong trận và di chuyển Right;
A vẫn hiện trênB trong cửa sổ đo, không tự về phòng. `host-EOF-control-visual.png`3ảnh đã đọc.
`host-EOF-policy`: chỉ sau EOF/socketerror đúng Achannel, chờ12s→175 choB/5s→180 choB/3s→
107xóaA và110promoteB ở phòng. Đây là policy bỏ cuộc emulator, không broadcast death giả
hoặc thử107trên CGameStage rồi suy nó được xử lý. Ađãkill không là peer nhận; Bcallback/slot/host đọc thật.

Recipe argv ở `<run>-recipe.json`, mọi rawfile/size/SHA pin trong `<run>-hashes.json`.
`derived/` native replay mọi actualC2S/CRCbad/partial; `native-packets.json` giữ nativeframer/getters/
serializedblob/checksum cho actual95/118/124/127 và reply159/158/165/170/175/180/582/107/110.
S2Cfixture là input đã chọn, không oracle server gốc. `actor-fields.json` giữ secure+28/+4C/+58
và stat+4A4/+50C/+4BC/+524/checksum/native52A350, sourcehash/context/round.
So state/coordinates/stat ở clock hai vòng chỉ là reset các field đã đo, không toàn heap/entity/timer/UDP.
`verify.py` đã sửa filter snapshot theo run khi kiểm nhiều run cùngfile; raw không đổi.

Tái lập dùng label/prefix mới, không đè sealedrun hoặc tranh lease:

```bash
python3 local/research/codex/round-finish-r1/run_probe.py new-finish CA-r2-stage-social.exe 260 wine_probe_round_finish.py local/research/codex/round-finish-r1/finish.gdb
python3 local/research/codex/round-finish-r1/seal_run.py new-finish
python3 local/research/codex/round-finish-r1/run_probe.py new-EOF CA-r2-stage-social.exe 175 wine_probe_round_host_eof_policy.py local/research/codex/round-finish-r1/finish.gdb
python3 local/research/codex/round-finish-r1/seal_run.py new-EOF
```

Giới hạn: report21 semantics/validator/ACK chưa phục hồi đầy đủ; không có21 sau175 trong ca này
không chứng minh mọi mode/timing hoặc packet đó là ACK. Actual127/sub34 sau175 được ghi neutral,
chưa đặt tên ACK/resultconfirm. Chưa guestEOF/rejoin sau abort/promotedhostStart/timeout/NAT/relay/
full auth/cache/codec/vanilla/allocator/stale/generation/allreset, không nghiệm thu fullmatch/K5/K6a/LAN/Internet.
CP028/032/038 là raw lịch sử riêng, không ghép thành ca pass tự động.

| Run / UTC31/10 | Kết quả | run.json SHA256 |
|---|---|---|
| finish-first / 2026-10-31T07:01:29Z | Hai vòng/statepeer1trước175;7B21mỗivòng/0sau175;returnphònghai lần. | `f6fdada7838df04ffcba1dffb01d925eff96e0c779e2178fb4a54ad16af53f77` |
| host-EOF-control / 2026-10-31T07:06:21Z | HostAEOF/Bcòntrận/di chuyển,Aghost;khôngpolicyreply. | `a30489b18a1e85796f0187a9ec8a3a0aac6fb663161a8bcc9cf35f04793757ab` |
| host-EOF-policy / 2026-10-31T07:09:49Z | Forfeit175/180B→room107A/110B;Bchủmới,mộtmìnhởphòng. | `7d4807c4bb5e444036d6b28b801201989d4868494d696dc81b6105e6420db2a5` |

**5023 kiểm cơ học pass**,3sealedruns,255nativepacket cases/436actualsecure snapshot/nativegetterchecks;14ranges/4artifact.
Hashrawindex/script/native reports/ảnh pin trong [CP039manifest](../checkpoints/manifest.json). Check không tự suy semantics hoặc đóng gate.
