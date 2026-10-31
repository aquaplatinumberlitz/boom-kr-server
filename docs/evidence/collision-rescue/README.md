# Collision/rescue và giới hạn report21 — CP036

Baseline CP035/[PR43](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/43) merge
`1e78035f60ad9de1329f6f0d983e5e5ac1519dc3`. CP036 snapshot trước tích hợp; PR nhánh
`kre/collision-runtime` giữ trạng thái merge. Review độc lập [waived](../checkpoints/review-waiver.json),
Codex tự kiểm. [Contract duy nhất](../../PROTOCOL.md#collision-rescue), [next/gate](../../RESEARCH.md#q-023).
[AUD011](../checkpoints/audits/AUD011.json) định kỳ qua CP036; audit kế CP040. Không gate mới đóng.

Raw/script/disasm/fixture/log/report ở `local/research/codex/collision-r1/`.
Bảy controller tuần tự,lease trong namespace user/net/mount/PID,chỉlo/no outbound route/files-only NSS/stub.
A/B prefix/wineserver/display/game-dir riêng. Không thêm binary patch/ép game state; copy kế thừa
CP020 import/API/AC/guard và initialglobalreset, không vanilla. Sourcebinary/pack/DB giữ nguyên.

| Artifact | SHA256 |
|---|---|
| Runtime A/B CA-r2-stage-social.exe | `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef` |
| Static CA-r2-login-logout.exe | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` |
| modO raw/base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |

RVA=VA−400000;modO khôngPE-remap.8 vùng code khớp4artifact; getter/writer/receiver/switch/spawn edges.

| Run / UTC31/10/2026 | Kết quả/giới hạn |
|---|---|
| enemy-first / 2026-10-31T04:06:14Z | Enemy chạm:17→19 sau2,322s; labelsubtypeGDBsai, captuređúng. |
| enemy-no-touch / 2026-10-31T04:09:40Z | Enemy giữkhoảngcách:17→19 sau6,180s; field/nativecontrol. |
| friendly-first / 2026-10-31T04:12:41Z | Mộtđội/twoactor:21 trước17,không18/19; failedrescuecontrol. |
| enemy-ack21 / 2026-10-31T04:16:35Z | Own-only170/21đọc6/6 nhưng49report21tiếp; khôngACK/resultproof. |
| friendly-three / 2026-10-31T04:20:00Z | Baactor/hai client thật:actual18,khôngforward→BgiữA56;153offCclearprofile. |
| rescue-forward / 2026-10-31T04:24:14Z | Forward18→170/18đọc7/7,B56→26,A26;153offCgiới hạn. |
| rescue-kept / 2026-10-31T04:28:39Z | Giữ153onC/profile;rescueA/B26vàAinputRight→Bthấyvịtrímới. |

Recipe argv ở `<run>-recipe.json`;mọi rawfile/size/SHA ở `<run>-hashes.json`. Tái lập bằng label/prefix mới,
không ghi đè sealedrun hoặc tranh runtimelease:

```bash
python3 local/research/codex/collision-r1/run_probe.py new-rescue CA-r2-stage-social.exe 165 wine_probe_collision_rescue_kept.py local/research/codex/collision-r1/collision-kept.gdb
python3 local/research/codex/collision-r1/seal_run.py new-rescue
python3 local/research/codex/collision-r1/derive_roles.py new-rescue
```

`run.json`:actual118/127reports/CRC/origin/seq,touchinputs,broadcast159/158/170; result175/180 tắt.
`derived/`:nativeframer mọi actual auth/channel C2S với CRCbad/partial.
`native-packets.json`:fulltyped118/159/158/commonprefix127/170,blob17/18/19 +checksum giữnative;
heap tổng hợp. `verify_start.py`:native121 fields/blobchecksum cho inner100/126 vàouter129/155.
Không dùng encoderS2C làm oracle hoặc emulationworld thay runtime.

`actor-fields.json`:actual secure+28/state,+4C/+58tọađộ,checksum vànative52A350 trêncapture,
sourcehash vàmốc trước/sauv148. Probe commonwriter là trước localrescue effect;
`RESCUE18_LOCAL_AFTER_V148` và`RESCUE170_AFTER_V148` xác nhận26; no-forwardcontrolB56.
`rescue-kept` bật Cslot153 kháccontrols153offC; fieldread/input/resumedmovement không thay acceptance3client.
`event-dispatch18-21.json`/`actor-vtable.json` khóa switch/vmethod; code-match8ranges khôngtoàncallgraph.

Ảnh đã đọc qua enemy-first6,friendly-first4,rescue-control6,final-controls6,kept-resume4;
một số ảnh lặp ở composite, không dùng sốảnh như chất lượng gate. Final no-forwardAthoát/Bbubble;
forwardfinalcảhaiAthoát; kept-resumeAđiRight/BthấyAđổi vịtrí. Ảnh immediateafter-touch có thểcòn
animationHELP trướcfullcallback; không dùngriêngảnh tức thời phủ nhận snapshot/ảnhtrễ.
`measurement-notes.json` giữ failedone-team,153offCsideeffect,misalignedread/stacklabelcorrection.
VN `boom-re/docs/oc_e-game.md` §rescue/enemy-pop chỉ hướng phépđo; wireKRđọc/đo riêng.
Cache/Capstone/Unicorn/WineGDB dùng tiếp; không lặp cold REA IDA timeoutCP024.

**11631 kiểm cơ học pass**,7sealedruns,1032nativepacket cases,1710actualsnapshot/nativechecks;8ranges/fourartifacts match.
Hash mọi metadata/script/artifact/rawindex pin trong [CP036manifest](../checkpoints/manifest.json).
Checker chỉ kiểm declaredhash/counter/link/timeline, không suy semantics hoặc đóng gate.

Giới hạn:hai client thật, C tổng hợp/peerzeroendpoints; khôngthirdclient/đảorescuer/auxkhác/
3actorno-touch/enemycontrol/UDPablation/validator/duplicate/stale/generation/result21/disconnect/toànreset.
Fullauth/cache/codec/reconnect/vanilla/relay/NAT/LAN/Internet/K5/K6a giữ mở. Rank/result175 không đo trong nhóm này.
