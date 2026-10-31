# Movement cũ: khóa153, handler và endpoint — CP047

Baseline CP046/[PR54](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/54) merge
`f85cda4146dd48a5a476400c9851bb8ca1c2f8c4`; nhánh `kre/udp-freshness`, KRE-005b/issue14.
Codex solo/tự kiểm; independent review waived. [Contract duy nhất](../../PROTOCOL.md#udp-generation),
[bước tiếp](../../RESEARCH.md#q-023). Raw/binary/ASM/helper/log/ảnh: `local/research/codex/udp-freshness-r1/`.

**Nguồn trước nhánh:** PROTOCOL/RESEARCH/CP045–046/evidence/actor-apply/cached125DEF0,13DE880;
boomkr-re kr-data189; boom-re oc_e-game475–486; boom-rs VN_LEGACY_SERVER_REFERENCE29;
l10n migration và archive KNOWLEDGE75. `reuse-question.json` giữ sourcehash/build/locator,
static/runtime/policy, phần reuse/phải kiểm KR và phép đo tối thiểu. Nguồn VN chứng minh P2P/endpoint
exchange, không KR key guard. Dùng lại fullproxy/isolation/CRC/queue/getter, không tìm lại các đường đó.

Runtime125DF30 phát hiện targetE66680/vt176A470 cả A/B; search cache trước body mới thấy CP025 đã có
`bomb-serial-r1/E66680.asm`, E83900 (reset24bytesFF),14E57B0 (peercreateprefix) và CP026native_all.py
(decoder153). Dùng lại các nguồn này, chỉ thêm runtime guard probes và phép thử đúng khoảng trống.
`E66680-prefix.asm` local là boundedconfirmation, không thêm một cơ chế độc lập hoặc quét wholeclient.
Bỏ việc tìm “gate input” E8E960false đã giảiCP046; không suy return callback thành accept/reject.

Runtimecopy A/B SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
staticcopy `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO raw/base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`.
PEcopies mapped, RVA=VA−400000. Sourcebinary/pack/DB bất biến; không thêm patch/ép actor/gate.
Controller tuần tự/lease/user-net-mount-PIDnamespace/chỉlo/nooutbound/files-onlyNSS;
A/B có prefix/display/wineserver/game-dir riêng. Diagnosticcopy kế thừaCP020, không vanillaWindows.

Probe kế thừa CP04624message/client/vòng2, thêm target125DF30; bản freshness.gdb thêmE667BE key,
E66868 tailcompare/E66884drop/E66889store/E668ADapplycall và13DE949endpointresolve.
Round/sample/thread guards giữ pairing. Endpointresolve có thể hit lần hai khi remainingbytes0;
chỉ exactfullwirematch xác nhận endpoint của replay, không gộp mẫu0byte thành datagram.
Secure wrapper/data actor+28/+4C/+58 giải checksum riêng rồi getter native52A350.
Getter oracle dùng giới hạn CP046:19D88D4starts0/1A7E438reset0perread, bỏ periodiccallbacks/rekey/storage
trong emulation; không đổi bất kỳ state/counter của client runtime.

| Run / UTC31/10/2026 | Recipe/phạm vi | run.json SHA256 |
|---|---|---|
| handler-stale / 15:06:50 |Cùng key/ports; thêm targetprobe, hai bom/17/19/statepeer/result/return.| `e5f819e43c826f83681bda4755dbbb1b132b81f23c9a3d1521a9c37e7d0543bf` |
| key-rotate / 15:11:40 |Chỉ second153u32 đổi12345→12346 ởR2/cảA-B; oldraw/ports giữ nguyên; hai result/return.| `adbf470826bc16b8c72935da99e4fe177332ccb660ee48a1b102f58544edf08f` |
| rotate-old / 15:16:53 |153ports mớiR2/key12345; exactold directtoclient từ retainedR1socket; hai result/return.| `4efe0a022a90b2908d9cf5aaa5c0354563aa8c2b61313613862df7f05d614495` |
| rotate-new / 15:21:41 |Cùng rotation/key12345, replay từ R2source; hai result/return.| `80041caf5f796d3c3af1282237dbc1e45c79125c6ec820a5f0d97c9e37fb594d` |

`key-rotate` thay đúng encodedtypedfield bodyoffset15..18 (stream11byte+firstu32); không GDBghi key.
Decoder native14EF300/14EB410/E8A340/typedgetters dùng lại CP026:97/97/errors0, key12345/12346 đúng,
26slot bật0/1, remoteport khớp actual153 theo vai/vòng. C heap/allocation là mô hình riêng;
CRCbad/partial là negativecases, không serverauthorityoracle.

`key-rotate` oldwire `10010100003b00016fc8009f00000043e3`, SHA
`12f19edc8462717b528f36ece3a4d5606bd4b128c78aeb3551628c44547b8291`:
R1B9442→proxy43086→A9525. R2/sample4 queue60771CE8 count26→45/drain19→0;
output13byte `00003b00016fc8009f00000043`, E66680 key59≠56. Cùngactor633B04B0 cóstate15/x120/y159
trước/sau; không tail/applycall. Gói mới key56 match/apply. Oldpacket không bị proxy hoặc CRC loại.

`rotate-old` rawold/tail69/key59 gửi trực tiếp từ cổng cũ60880 sau advertisedBport mới46041.
13DE949 resolve peer5FB758B8 khác0, exactold tới queue/drain/game/E66680; R2/sample4 tail69/stored86
vẫn store/apply. Không suy object sống liên tục chỉ từ pointer; kết luận dựa exactwire/branch/securefields.
Relay thử bỏ traffic tới obsoletegenerationdestinations, nhưng replay đi thẳng tới actualclient từ socket
cũ giữ lại, vượt bộ lọc proxy đó. Vì vậy client nhận/apply không phải artefact “sendlog chưa tới client”.
Rotations/sources/oldsocketretention/discarded_old_dest được ghi rõ; không productionNAT/relay.

`native_guard_branches.py`:10tailcases +6keycases chạy code native trên context tổng hợp,
dừng tại branch/store trước sideeffect. Có biên lùi9/10/wrap và key+256 collision; không là runtime
mọi biên, auth hoặc proof toàn UDPgrammar. `code-match.json`6ranges/4artifacts.
`replay-effects.json` ghép exacthash→nativeendpoint→queue/drain→output/target/branches→securefield;
`runtime-summary`/`downstream-summary` giữ rawline/sourcehash, không suy ngữ nghĩa bằng checker.

[UdpRoundKeys](../../../server/udp_round_keys.py)/[kiểm policy](../../../server/test_udp_round_keys.py)
là `design/emu_choice`, chưa runtime tích hợp: cấp một u32/mỗi acceptedStart, dùng cùng khóa cho cả
phòng, không lặp low8 trong256lần rồi báo lỗi. Giữ instance qua finish/abort; không tự release/reset.
Lifetime peer mới và roster/ID thay do caller xử lý/kiểm riêng; không tạo instance mới trên traffic cũ
rồi tự suy an toàn. Runtime hiện chọn12345/12346 trực tiếp qua153, không gọi module mới.
Các biênu32 trong unit test chỉ là kiểm allocator, không bằng chứng clientKR nhận mọi key.

Tái lập tuần tự với label/prefix mới, không đè sealedrun; mỗi run giữ recipe argv/harnessSHA:

```bash
python3 local/research/codex/udp-freshness-r1/run_probe.py new-key CA-r2-stage-social.exe 260 wine_probe_udp_key_rotate.py local/research/codex/udp-freshness-r1/freshness.gdb
python3 local/research/codex/udp-freshness-r1/seal_run.py new-key
python3 local/research/codex/udp-freshness-r1/run_probe.py new-old-source CA-r2-stage-social.exe 260 wine_probe_udp_rotate_old.py local/research/codex/udp-freshness-r1/freshness.gdb
python3 local/research/codex/udp-freshness-r1/run_probe.py new-new-source CA-r2-stage-social.exe 260 wine_probe_udp_rotate_new.py local/research/codex/udp-freshness-r1/freshness.gdb
```

Không fullvalidator/wholegrammar/productiongenerationguard hoặc fullmatch/K5/K6a/LAN/Internet.
AUD014/CP044 giữ nguyên; audit định kỳ kếCP048. CP047 hoàn tất record/docs/tự kiểm, tích hợp qua PR nhánh.

`rotate-new` exactold từ sourceR2port47725 resolvepeer645F8B50, R2/sample4 tail70/stored86,
key59match/store/apply/actor x120→200/y159 giữ nguyên; nativegetter/checksum khớp. Có hai result/return.
Mọi run giữ cùng8connections/IDs1,2, slots0,1, samechosenmap; không có GDBfailure/collectortraceback.
Các oldwirehash/hex/sourceport/SHA/run/path riêng trong `replay-effects.json`; không ghép các run
khác nhau thành một ca trận hoàn chỉnh, không suy nhân quả từ ảnh/tutorial.

Kiểm cuối: **13650** quan hệ cơ học/4sealedcontrollers,678native packet cases/1766secure snapshots;
48native153cases (16actualvalid vàCRCbad/partial),372nativeUDPCRCrows/259unique;
16nativeguardbranchcases/6ranges4artifacts;16policytests(4newallocator)/artifacts5/5 pass.
`visual-index.json`16ảnh đã đọc qua4composite: mỗi runA/B ngay sau probevòng2 và finalroom.
Ảnh không thay nativebranch/securefieldproof. Rawfile/size/hash trong `<run>-hashes.json` được verify;
checkpoint pin helper/source/report/recipe/metadata, binary/ASM chỉ ởlocal.
