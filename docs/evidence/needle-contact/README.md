# Kim và tranh chấp cứu/diệt — CP037

Baseline CP036/[PR44](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/44) merge
`7ab702f8ceee0cb60e38e2bf5bba15d505a4c465`. Snapshot trước tích hợp CP037;
PR nhánh `kre/needle-race-runtime` là nguồn trạng thái tích hợp.
Codex solo/tự kiểm; review độc lập [waived](../checkpoints/review-waiver.json).
Contract duy nhất: [kim](../../PROTOCOL.md#needle-selfescape),
[precedence](../../PROTOCOL.md#contact-precedence). [Bước tiếp/gate](../../RESEARCH.md#q-023).
AUD011/CP036 định kỳ đã hoàn tất; [AUD012](../checkpoints/audits/AUD012.json) sửa nhận định nháp CP037, không đổi lịch audit kế CP040. Không gate mới đóng.

Raw, script, fixture, disassembly, capture API và ảnh ở hai root ignored:
`local/research/codex/needle-r1/`, `local/research/codex/contact-race-r1/`.
Controller chạy tuần tự với lease trong namespace user/net/mount/PID, chỉlo/no outbound/files-only NSS.
Mỗi client có prefix/wineserver/display/game-dir riêng. Không thêm patch/ép state;
copy kế thừa CP020 import/API/AC/guard và initial global reset, không vanilla. Nguồn binary/pack/DB giữ nguyên.

| Artifact | SHA256 |
|---|---|
| Runtime A/B/C CA-r2-stage-social.exe | `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef` |
| Static CA-r2-login-logout.exe | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` |
| modO memory/base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |
| KR_ItemAttr.bin | `a9b55540c85c21de108e8720b4148b2030a231d907f3709dcb06f7ef728fcdba` |
| VN_ItemAttr.bin | `a98883ce35f3a509719db637f662f2ec90da64dd3c1faa0c9f7eb22c0de7dd1d` |

RVA=VA−400000; modO đọc raw memory, không PE-remap.
`needle-r1/code-match.json` kiểm10 vùng/4artifact; `contact-race-r1/code-match.json`
kiểm12 vùng/5artifact kể cả C. Equality code không tự chứng minh runtime/luật gốc.
VN `boom-re/docs/oc_e-game.md:457` là nguồn runtime lịch sử định hướng ca kim;
`items.json` giữ path/hash/offset/hàng KR/VN. Không chuyển opcode/kind bằng offset VN.

| Root / Run / UTC31/10 | Kết quả/giới hạn | run.json SHA256 |
|---|---|---|
| needle-r1/needle-first / 2026-10-31T04:40:05Z | Collector fail:KeyError41/role; không client-negative. | `9ff9e9df1605b519b477d06d1b4bc7096b0ed1cbbd19525a6dbdd2cc4f567fd4` |
| needle-r1/needle-fixed / 2026-10-31T04:44:48Z | No forward1 control:localthoát/peerbubble. | `e94efafeed2da5a95b732cd4472a087850e1c79727492cbd92cad2256a88d63d` |
| needle-r1/needle-forward / 2026-10-31T06:03:55Z | 170/1full6/peer26,movement/inventoryimageempty. | `0262fb0bdadfc4bd63c2b33e4445f29b0a389ffd22e29cfd941b622a729e4fb7` |
| contact-race-r1/race-first / 2026-10-31T05:59:38Z | Ccloser→19; concurrentkeys≠samecontact. | `62b757a0a36ae82a14c69e98b90498f4439ffaa06947d4d6c82ae8fd48f88294` |
| contact-race-r1/race-Bfirst / 2026-10-31T06:07:05Z | Cinputtrễ0,7s;Bvalid→18. | `57105a263ad8df0f1c6005f67b3db047175facebbd1bca6fc451f7eb66bdfa3b` |
| contact-race-r1/race-aligned / 2026-10-31T06:10:55Z | Probe hotgetter:clock nhưng khôngactual118/contact. | `ca213b112aeb00a0eb44751b5b978cbff57751b20aa562908f4dec350510824b` |
| contact-race-r1/race-aligned-limited / 2026-10-31T06:14:48Z | Getterselfdisable12:actual118/17/18;tick426Bvalid. | `48801ac52005cb0671536c245d6d8568b52d7781a7ab17c88037e1bcdd6ef7ae` |
| contact-race-r1/race-order / 2026-10-31T06:33:49Z | Cinputtrễ0,2s;visit0/1/2×4;tick389Cvalid→19. | `80d824e404c29b60d8cf5c22d81788aeab67585e6607d780ce782a5c871e7f0d` |

Recipe chính xác là `<run>-recipe.json`; `<run>-hashes.json` pin mọi rawfile/size/SHA.
`run.json` có actual CRC-checked118/124/127, seq, input time và S2C đã phát.
`derived/` lọc metadata theo role rồi replay **mọi C2S actual** qua nativeframer với CRCbad/partial.
`native-packets.json` giữ full typed fields/blob/checksum; S2C do stub tạo không là oracle server gốc.
`actor-fields.json` giữ secure state+28/coordinates+4C/+58 thực, checksum và native52A350,
source log/hash và mốc trước/sau effect. Parser mới map đủ page khi dữ liệu12byte đi qua biên page;
không sửa raw để làm parser pass.

Tái lập với label/prefix mới, không ghi đè sealedrun hay tranh lease:

```bash
python3 local/research/codex/needle-r1/run_probe.py new-needle CA-r2-stage-social.exe 165 wine_probe_needle_forward.py local/research/codex/needle-r1/needle-forward.gdb
python3 local/research/codex/needle-r1/seal_run.py new-needle
python3 local/research/codex/contact-race-r1/run_probe.py new-contact CA-r2-stage-social.exe 205 wine_probe_contact_flags_aligned.py local/research/codex/contact-race-r1/contact-detail-limited.gdb
python3 local/research/codex/contact-race-r1/seal_run.py new-contact
```

`native-selfescape.json`:5 ca native170/sub1 dispatcher/E86820, endpoints world/actor/effect model.
`native-decision.json`:6 ca native final branch E6EEA7–E6EF3E với flags/frame/vtable tổng hợp;
`native-predicate.json`:18 ca1269B20 với rectangle/getter/vf1F0 model;
`native-collection.json`:4 ca E6EAB3–E6EE66/predicate với cả hai candidate đủ điều kiện, đảo thứ tự danh sách.
Final branch cả hai cờ là counterfactual; collection thực thi native dừng ở chạm hợp lệ đầu, không gom cả hai để suy diệt luôn thắng.
Không gọi các ca này là runtime cùng chạm hoặc full world emulation.
Raw static rộng gần E7C4D0/E7C0D0 gồm hàm không liên quan; chỉ phân tích cạnh runtime
E7C7B0→E6E440 và predicate/dispatch/selfescape. Không lặp cold REA/IDA timeout CP024;
dùng cache/Capstone/Unicorn/WineGDB, không sửa DB nguồn.

`measurement-notes.json` giữ failed collector, probe không có118, candidate nguyên nhân,
điều kiện sửa probe và giới hạn. Root `error=None`, returncode input0, TCP hay clock không làm run pass gameplay.
Inventory trống/tiếp movement là phạm vi quan sát; server inventory/duplicate/stale, kim lần hai/
Ctrl cạnh tranh, validator/result21/disconnect/toàn reset/full auth/cache/codec/vanilla/relay/NAT/
LAN/Internet/K5/K6a còn mở. Raw CP036 và checkpoint đã complete giữ bất biến.

Kiểm cơ học: needle **3188** checks/3sealedruns(1failedcollector),57nativepacket/114actualsnapshot/5dispatchcases; contact **10669** checks/5sealedruns(1incomplete-no118),156nativepacket/612actualsnapshot/4collection/18predicate/6counterfactualbranchcases.
Không cộng số kiểm thành mức hoàn thành gameplay. Hai probe không đạt được giữ rõ outcome, không gọi mọi run pass.

Ảnh đã đọc: needle-first-visual4/needle-forward-visual6; race-first-visual6/race-Bfirst-visual6 và race-order-visual. Immediate images có thể còn HELP/animation giữa callback; state/cursor/ảnh trễ mới dùng đối chiếu. Hash chỉ mục raw, native reports, scripts và ảnh pin trong [CP037](../checkpoints/manifest.json).
