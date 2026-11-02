# CP072 — EOF trước clock: forfeit/return đã thấy, cleanup còn thiếu

Baseline CP071/[PR103](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/103) merge
`d95c5ba12c49fe10488c5afbdc00809ba719fd86`; KRE-035/[issue104](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/104).
Codex solo/tự kiểm, [review waived](../checkpoints/review-waiver.json); tích hợp theo [PR105](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/105).
[Contract/giới hạn](../../PROTOCOL.md#preclock-eof), [gap](../../RESEARCH.md#gap-disconnect-replacement),
[thứ tự](../../PLAN.md#current-order). Raw/script/log/capture/seal: `local/research/codex/preclock-eof-r1/`.

**Partial:** native175/180 trước clock đã kiểm, **chưa native107/IDclear**. Không gọi toàn cleanup,
forfeit thành death, hay sender phát gói thành client đã đọc. Không sửa service policy khi chưa có
native failure ở boundary đó. Ba run gom một checkpoint; hai startup failures không tính vào phép EOF.

`reuse-question.json` pin CP050 playedEOF/delayed99 và CP052–054 setup/departure, native175/180/
107/110/IDclear/chief và current service; KR cũ/VN lobby/session/oc_e-game, boom-rs reference/l10n/
archive. Tái dùng packet/consumer/probes/fixture, tránh RE lại clock/auth/cache/nativebody hoặc mọi
lifecycle race. Profile CP069basic41 giữ nguyên, production close/finish/forfeit/cleanup không đổi.
Private `hold99.py` chỉ giữ dispatch actualB99≤30s và discard sau realEOF; không networkloss hoặc
native server policy. Native codec/raw-frame bên ngoài dispatcher vẫn chạy thật.

Runtime A/B SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO raw/base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`.
RVA=VA−400000. Source binary/pack/DB giữ; không patch mới hoặc gọi/ép hàm/state game.
Mỗi controller tuần tự/private user/net/mount/PID namespace/chỉlo/no route/files-only NSS/mộtlease;
privategamesCP059/display/wineserver riêng. Prefix CP062 cho run đầu, CP069warmcopy cho hai run sau;
registry digest khác đã ghi, **không chứng minh prefix gây/sửa lỗi NMCO**.

| Run / UTC02/11 | Kết quả | run.json SHA256 |
|---|---|---|
|guest-preclock /17:55:29|Bstartupabort/noStart/không held99|`791159a3543c89fe9da8173679a11fb835281ad11061dc0e5b8ac3010124b002`|
|guest-preclock-warm /18:01:18|A/B Start121/A99/B99held→realBEOF; A175/180, chưa107native|`8980acfa4d824423cfdef9eefc55cc47f2fd90cfc5877f688c5d56002a66c95f`|
|guest-preclock-observed /18:05:16|Astartupabort/noStart, B chưa spawn; observer không chạy|`c15e6a82aada3d0863dfcca4c6ce02b64f72be54a4f7cb03b6db41ae52cf7b3d`|

`guest-preclock-warm`: actualB99 seq118 được giữ3188,541ms tới discard sauEOF; actualA99 đã dispatch,
B99 chưa dispatch. Hai native121 đọc129/129/errors0; không197/196/153 ở wire, không clock cảhai.
Controller kill đúng prefixB sauhold+2s/A99; room finish/abort theo production policy, không bom,
pickup hoặc17/19/21. Survivor A nhận175outer302/inner292/errors0,180full3/errors0 và edge stage12.
Service sau đó phát107/110 nhưng trace trước teardown chưa có107/slot1ID2→0/chief sauclear.
Dừng cũ lấy service refresh+3s; A reset tại1602701507858344ns **sau** stop1602701233789626ns.
Giữ service_errors connection4/ECONNRESET, không báo run error-free hoặc quy reset cho preclockEOF.
Không có trace đủ để quy mất107 cho tốc độ debugger/stop/engine; missing observation vẫn unknown.

`guest-preclock-observed` đổi observer: dừng khi native107/IDclear/chief thực, tối đa30s sau cleanup,
bỏ autoUI sauEOF; không đổi service/client/probe/profile. A lại SIGABRT trướcStart, nên không đo
được điều kiện mới. Hai startupfailure cùng UI `nmcogame.dll SetLocaleAndRegion Error(2000:0)`,
SIGABRT kernelbase7BC0A4A8, stack1526052→96E300→96E400→9769B3…; ảnh và log được đọc/pin.
Lỗi cùng tên đã có tại CP064, không quy cho wrapper/observer/prefix/EOF từ tương quan. `error` absent/
launcherreturn0 không là native startup thành công. Không thử thêm runtime không đổi điều kiện.

`verification.json`: 2146 quan hệ/hash/seals/source, 549 nativeframer,
36 nativepacket cases, 24 secure snapshots. Native175/180/held99 parse tách với107/110
chỉ wire/model đã parse. Heap/receiver/gettercounter model chỉ emulation, không game forcing.
Code không đổi, dùng baseline72server/18checkpoint tests; catalog7/docs/artifacts5/5/localhistory/diff
kiểm lúc bàn giao. **AUD022/CP072**, periodic kếCP076; audit đọc docs và đối chiếu raw CP069–072,
sửa authority/header/RESEARCHprogress lặp, historical nextsteps và scope incidentalRoller CP070.

Recipe/hash: `<label>-recipe.json`, `<label>-hashes.json`; run_probe.py với label/prefix mới
CA-r2-stage-social.exe300/wine_probe_preclock_eof.py hoặc _warm.py hoặc _observed.py/preclock.gdb.
Current startupblocker chưa sửa nên không tự lặp recipe. Seal sau completion, parse_fields/lifecycle,
derive_roles/native_packets/verify với từng run; roleB không tồn tại ở run3 phải giữ absence, không
bịa emptyB log/capture. `verify_positive.py` giữ yêu cầu cleanup dự kiến; partialverifier không tự đóng.

Giới hạn: no gameclock/gameplay/fullreset/accountresume/timeout/loadfail/hostvariant/fullmatch/K4/
K5/K6a/LANInternet/vanilla/auth/originalforfeit. Native slotclear còn mở. Ưu tiên kế tiếp là caller/
input/result của NMCO locale đang cản runtime: reuse CP006–020/064, chỉ trace branch96A15A/96E300/
9769B3 và API liên quan trên copy. Chưa kết luận lỗi do identity/cache/region/prefix; giữ positive và
negative runtime để chọn phép kiểm nhỏ, không retry proxy/chain hoặc ép success khi chưa biết nhánh.

Attribution cập nhật ở [CP073](../../PROTOCOL.md#launcher-error-attribution): cùng popup đã tái hiện
trong auxiliary/no gameTCP; stack96A15A là HTTPcontents, chưa locale caller. Không dùng nhãn
startupNMCO tại CP072 làm causalroot main. Raw/error/nativecleanupunknown giữ; thứ tự hiện hành ở PLAN.
