# CP095 — Đối chứng hai bom qua thùng thật

KRE-058/[issue150](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/150), baseline
CP094/[PR149](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/149)
`5e580a4d879bbd7821d0759757891b36412ca4b6`. Codex solo/tự kiểm;
[review waived](../checkpoints/review-waiver.json). CP095 đã merge [PR151](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/151) tại `1c18aa2`; review waived, Codex tự kiểm.
Contract/giới hạn ở [PROTOCOL](../../PROTOCOL.md#opposite-box-control),
câu hỏi ở [RESEARCH](../../RESEARCH.md#gap-blast-block-chain), thứ tự ở [PLAN](../../PLAN.md#current-order).

Raw gốc: `local/research/codex/opposite-box-r1/`; repeat: `local/research/codex/opposite-box-repeat-r1/`;
aligned: `local/research/codex/opposite-box-aligned-r1/`;
stable: `local/research/codex/opposite-box-stable-r1/`.
`reuse-question.json` pin17sources: current map/flags/terrain/nativecallback/cache121/spawn,
KR flame-chain/data, VN game, l10n/archive. Dùng lại getter/framer/typed121 readers và actual map;
không RE lại codec/mapformat/entireclient hoặc port numeric VN. `preparation.json`/fixture metadata
pin đổi spawn1→4 duy nhất cùng matching trusted startSHA. Production CP094 không đổi.

| Run | Kết quả | run.json SHA256 |
|---|---|---|
| blocked | Partial controller A-later-escape; bốn actual118/159, ba158/view; blocked callback/boxclear đo được, later root chưa gửi158. | `a2528803ae9b600a510d14cb2219e9238d8e64e621005f75dcbfa3345513f61a` |
| freshpair | Partial trước cặp chính: B-behind-box miss sau vị trí Y119 ở hàng2; một native118/159/158. | `ab248498a228018e06b6a34cc1d84d56baf4bb2bc4dd2aef90ec5d3827efea6b` |
| aligned | Partial B-behind-box: một mẫu Y163 rồi horizontal thấy Y187/X360, chỉ opening root. | `d17354b58cea4d6bf378b5b0993f3d2e61c410f69e884ff0d44b97e54d72be58` |
| stable | 194,715s, strictpass49relations, actual5entities/4callbacks/view/18flagsnaps; blocked rồi freshpairchain. | `f299a5eee15e8002a765a7ce265e3e60d13a003a0adb4f395331a2b00c1aed2d` |

`<label>-recipe.json` argv; `<label>-hashes.json` pin original final raw. Một controller/lease,
private user/net/mount/PID namespace, lo-only/no routes/files-only NSS. Input bàn phím thường,
read-only actor feedback; finite195flags/native159/158lookup/process/cursor. Không teleport/terrainforce/
newnativebody/gamecall/sourcebinarypackDB edit. Prefix riêng, runtime cùng clientSHA đã pin CP094.

Run blocked: `control-verification.json` pass40relations cho partial boundary,14snapshots,
2758copiednativegetters/2730arraychecks riêng; planned `verify.py`/`verification.txt` vẫn fail original
controllercompletion và không dùng để nhận toàn ca. `verifier-schema-correction.json` ghi ack ID nằm
trong wire.fields11/end_line, không phải top-level id/line. Raw và expected geometry giữ nguyên.
Repeat preparation pin lỗi gốc và đổi lịch: B3 owncallback rồi cặp A4/B5 mới, không gọi đó là giữ B3 sống.
Repeat `freshpair` có6snapshots/1182copiedgetters/1170arraychecks, chỉ opening root; không có cặp chính,
control verifier fail và giữ nguyên. B return-top feedbackY155, bước Left sau đó thấyY119/X360,
không tới3,7; cơ chế timing chính xác chưa chứng minh. `aligned/preparation.json` pin run này và chỉ
thêm đọc neutral sau release/pulse45ms để căn hàng3/cột7. Aligned chỉ6snapshots/1182getters/1170arraychecks, controller và strictverifier vẫn fail.
Stable preparation pin aligned error; ba distinctfreshsamplecùngtọađộ sau250msneutral trước alignment,
OSpulse45ms giữ. Đưa B tới ô mới trướcA4fuse, chỉ chờ1s rồi SpaceB; actual118/nativecallback xác nhận.
Không force actor. A có thể trúng lửa sau A4; phép đo không lấy sống/chết hoặc result làm acceptance.

`stable/control-verification.json` pass49relations; năm actual159/view full39/nonzero, bốn158/view
full13; callbacks processed1/2/3/4,5. B3owncallback trước cặp mới, không phải originalB3 còn sống tới A4.
18snapshotflags/3546copiedgetters/3510arraychecks; all195flags/nativeprocessed match unchangedservice.
`source-scope-verification.json` thêm6checks ở blocked/stable cho lo-only/no-route/lease/filesNSS/
clientSHA/rawgrid/object1932classword2/profilepins, không coi tên/icon là behavioralproof.
Own158 của5 không quan sát trước chủ động dừng, không fullcycle/reset claim. Tổng2689sealedrawfiles
cả bốn run được kiểm bytes/SHA; mỗi run hash/counter riêng, không cộng làm proof độc lập.

Tái dùng113server/4kernel/18checkpointtests tại CP094 vì production/checker không đổi;
kiểm mới gồm parse/framer/nativepacket/getter/control, docs/catalog/artifacts/localhistory/diff.
Audit AUD027 qua CP092, periodic kế CP096. Giới hạn/gap chỉ ở contract/RESEARCH; không đóng gate.
