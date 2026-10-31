# UDP vòng mới: ingress/queue/drain và dữ liệu cũ — CP045

Baseline CP044/PR52 merge `b947a9d62507a9aef9625cc2c4586198c53ddbf1`, nhánh `kre/item-families-udp`.
Codex solo/tự kiểm, independent review waived; [contract duy nhất](../../PROTOCOL.md#udp-generation),
[bước tiếp/gate](../../RESEARCH.md#q-023). Raw ở `local/research/codex/udp-generation-r1/`.

Trước nhánh UDP đã đối chiếu currentCP027/039–044/evidence/cache13DE6D0/actualUDPCrc;
boom-re oc_e-game475–486 và server/src/session.rs92/146; boom-rs VN_LEGACY_SERVER_REFERENCE và
ADR0009; boom-re-l10n docs migration; archiveKNOWLEDGE/retainedlogs pointers. Nguồn cũ giúp bỏ
việc tìm lại endpoint/proxyIP/CRC, không có source được đọc nào đã chứng minh KRstaleUDPvalidator.
ADR0009 là protocolRewrite/TCP với epochdesign, không KRnative. `reuse-udp-question.json` giữ hash,
điều đã biết/prooflevel/reuse/KRdelta/gap và phép đo tối thiểu trước body mới.

Runtime A/B SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static copy `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO raw/base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`.
RVA=VA−400000, PEcopies mapped. No newbinarypatch/actorforcing; sourcebinary/pack/DB immutable.
Controller tuần tự với lease/user-net-mount-PIDnamespace, onlylo/nooutbound/files-onlyNSS;
clientprefix/wineserver/display/game-dir riêng. Diagnosticcopy kế thừa CP020imports/API/AC/guard và
initialglobalreset, không vanilla Windows.

Read-only GDB13DE7AA sau successfulrecv trước vf1C ghi receiver/vtable/actualcallback/ip/port/time/raw;
`ingress-rounds`96sample(24/client/round), actualtarget13DE880/vt17A2BB0. Theo target mới đọc có mục tiêu
13DE880endpoint/length/CRC→14E7340→14E8280queue. Wrapper pass0 không thay thế fullgameepochvalidator.
Thêm14E83A0clearbody/14E8480drain và alignedcallsite14E5742; không gọi callsitewindow14E5710 là functionentry.
`code-match.json`10ranges/4artifact; byte equality không proof execution, runtimequeue/drain source riêng.

Proxy đổi endpoints153 sang ownedloopback sockets, forward originaldatagrambytes từ advertisedsource
đến actual99port. Một controller chỉ sở hữu hai proxies/clientcopies; không traffic game/auth ngoài local.
Record udp_proxy.packets giữ originalhex/hash/time/source/destination/round; exactoldreplay có metadata riêng.
Đây là thí nghiệm local, không productionNAT/relay hoặc chống giả mạo.

Các ca giữ riêng:

- ingress-rounds: hai vòng directpeer, endpoint/receivecb proof, chưa replay.
- proxy-control: một159/entity/158kind0 nhưng khôngA17/19/result; partialgoal, không nói proxyđã đủhai trận.
- centered-control: GDB conveniencevar chưa khởi tạo, `Invalid type combination in equality test`/KR_STOP
  tại14E85B7. ControllererrorNone không có nghĩa probehealthy; raw lỗi giữ nguyên.
- centered-fixed: khởi tạo allvars và giữA ởspawn/bỏDown trướcSpace; queue/drain, haiactual17/19,
  statepeer1 trước175, hai175/180/phòng. Không quy nguyên nhân partialrun chỉ bằng thayinput/probe.
- stale-round2: exactold B→A trước input vòng2 khớp actualqueue/drain, chưa đo gameconsumer.
- stale-game-guard: probe game mọi vòng/tần suất cao, chỉ vòng1/A17/no19/noR2/replay. Không lỗi GDB;
  overhead là giả thuyết, không kết luận nguyên nhân. Giữ partialgoal.
- stale-game-lite: chỉ bật probe game từ Start2/tắt sau64 nonempty message/client; cùng staleharness,
  exactold khớp queue/drain/gameconsumer, hai17/19/statepeer1/result/return. Không suy actorapply.

Queue samplelength15: count0→19, drain15→0; length3:0→7. Buffer chứa4bytelength+data,
caller14E5747; nhiều record có count≠0 được giữ. GDB queue/drain pairing có thread/receiver guards;
conveniencevars initialized0 ở bản fixed. Body14E83A0 clear0 chưa là proof nó chạy đúng boundary.
Consumer prefix13E0E00/reader14EB0D0 giải phần selector/slot/conditional4bytecompare; không toàn parser.
Effect của packet cũ lên actor và round identity vẫn chưa giải; [contract](../../PROTOCOL.md#udp-generation)
giữ kết luận duy nhất, không lấy ảnh spawn/tutorial làm proof teleport.

| Run / UTC31/10/2026 | run.json SHA256 |
|---|---|
| ingress-rounds / 13:00:36 | `a45b3860a629a12fdb8222b4da0edaba30efb11f276c7d48813c1d28cf8d0f60` |
| proxy-control / 13:05:04 | `e7431600e456ddd5d2c1443bb61336c5bc8be7c246adb0c920d6b59a989514ca` |
| centered-control / 13:09:37 | `1e9cd19e58bad62f6a702d728c480439433da3d52e10638eb3ecd0aebf7334d7` |
| centered-fixed / 13:12:00 | `ed6660046297077460774c0f7bc4739e3c592837e2dd4991dbd498079db3abe9` |
| stale-round2 / 13:18:45 | `d7ccab9f0799e8576ce159bcc44d387042a4ac16405a6561de2bcc53b92f3a75` |
| stale-game-guard / 13:24:40 | `cec937d0972b357aa344d04639f14efd3c56d0c6f982e4be62023ce07277e106` |
| stale-game-lite / 13:34:04 | `af0f7a9779fbc71b9aca8be7ab1802d202c89e886fe9b1c42faa3d8e4b261d16` |

`stale-round2` rawold17byte `10010100003b000132c8009f0000004bf7`, SHA
`bbd38001fff30521da221b5c67a5b2389e3d95abab160cb85e376d9a063f4058`:
actualB9265 → proxyB40196 → A9973, R2/sample10 queue61FE6E00 count26→45/drain57→38.
`stale-game-lite` rawold17byte `10010100003b00013ac8009f0000004866`, SHA
`dc261c4b21730298f962a78cc6cbe26fe64c2034c3828bd2feb6d9f7e8715246`:
actualB9900 → proxyB55775 → A9992, R2/sample6 queue5C303B80 count19→38/drain45→26;
actualgamebuffer khớp inner15byte, slot1/gate0/return13. Không compare/windowdrop/store trong ca này.
Cả hai oldhash vắng trong normalforwardR2; không chỉnh key/seq/payload/CRC, không thay TCP packet cũ.
`runtime-summary.json` giữ line/hash/pairing/replay→queue/drain/game; không dùng output emulator
hoặc sendlog làm bằng chứng native xử lý. Prefix game/timestamp fields chưa là full decoder.

Kiểm: `verify.py` **21349** quan hệ cơ học/7sealedcontrollers/795native packet cases/1280secure
snapshot pass; hai partial và một GDBfailed giữ nguyên. `native-udp.json`:482records/384unique,
CRCnative14A5860prefix=wireCRC/fullCRC0; không semanticoracle. `code-match.json`10ranges/4artifacts.
`visual-index.json`24ảnh đã đọc qua3composite: control/oldqueue/oldgame, mỗi runA/B trướcinputvòng1,
ngay sau probevòng2, resultvòng2, finalroom. Spawn/tutorial không chứng minh tọa độ thay đổi do oldpacket.
Runmetadata/filebytes/SHA ở `<run>-hashes.json`; helper/ASM/rawmetadata/derived được pin trong CP045.
Số kiểm không là phần trăm tiến độ hoặc nghiệm thu gate.

Tái lập tuần tự, label/prefix mới; không đè sealedrun:

```bash
python3 local/research/codex/udp-generation-r1/run_probe.py new-control CA-r2-stage-social.exe 235 wine_probe_udp_centered_control.py local/research/codex/udp-generation-r1/queue-fixed.gdb
python3 local/research/codex/udp-generation-r1/seal_run.py new-control
python3 local/research/codex/udp-generation-r1/run_probe.py new-stale CA-r2-stage-social.exe 235 wine_probe_udp_centered_stale.py local/research/codex/udp-generation-r1/game-round2-lite.gdb
python3 local/research/codex/udp-generation-r1/seal_run.py new-stale
```

Mỗi run giữ recipe argv/harnessSHA; read-only breakpoint probes, copy-only và isolation như trên.
Không K6a/fullmatch/LAN/Internet. CP045 record/tự kiểm hoàn tất, review độc lập waived;
CP044/PR52 là baseline đã merge, tích hợp CP045 qua PR nhánh. AUD014/CP044, audit định kỳ kếCP048.
