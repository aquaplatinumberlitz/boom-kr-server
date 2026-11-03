# CP078 — Một vòng Battle sau rejoin; vòng hai còn thiếu

Baseline CP077/PR115 merge `a3ca2ab064280b0aaf863223c5401d5e094c1b68`;
KRE-041/[issue116](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/116), **acceptance hai vòng partial**.
Codex solo/tự kiểm, [review waived](../checkpoints/review-waiver.json), tích hợp theo [PR117](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/117).
[Contract/giới hạn](../../PROTOCOL.md#rejoin-battle-cycles), [câu hỏi](../../RESEARCH.md#gap-diagnostic-service),
[thứ tự duy nhất](../../PLAN.md#current-order). Raw/script/seal: `local/research/codex/rejoin-battle-cycles-r1/`.

Nguồn trước nhánh: CP077 actualBack86/rejoin82/87/585/107/roster/map/Ready/Start, CP069 basicroute/
effect/reset và CP075 currenteffects/partialbudget. `reuse-question.json` pin KR/VN items/oc_e-game/
session/boom-rs/l10n/archive theo câu hỏi. Tái dùng writer/consumer/body/clock/nativegetter/framer;
không nativebody mới/patch/force state, không port opcode/layoutVN hoặc quét toànclient.

| Run / UTC | Kết quả có giới hạn | run.json SHA256 |
|---|---|---|
|rejoin-battle-budget / 2026-11-03T11:46:08Z|Same-run guestBack/rejoin→R1Roller17/Bomb27,2bom power1,17/19/peerstateA1/175/180/refresh. KhôngB95/A104 saurefresh, không vòng2.|`6684e01e22f5778cabf7357f2a8c0e55548e189f9281efd42ec84c84b6f01105`|
|rejoin-render / 2026-11-03T11:56:44Z|Chưa tới Ready2/render followup: controller100s từJoin dừng1,5065s sauactual99cảhai, trước153/clock; không protocolfailureproof.|`4268d897ba58833dc92d6e6f1406387f2cd25db35c98cec4afcef1049efcd9e4`|

**Confirmed hẹp first:** cùng8TCP/IDs1,2/slots0,1, giữseq sauBBack/87/585/roster. Actual12417/27,
165 đúng2lần mỗi view; native speed5→6/capacity1→2, powergiữ1; haiactual118ID1,2/159/158,
actual17/19→B170/stateA1 trước175full302/180full3/refresh. Fire28 không nhặt, không hiddenpointerjoin
mới/probeabsent. Đây là **một vòng sau rejoin**, không fullmatch/two-cyclepass/toànBattle/wholeReset.
`first-cycle-integration.json`/`first-cycle-verification.json` đối chiếu native/capture/field joins.

**Khoảng trống cụ thể:** actualC2S585 saureturn ởA(seq346) vàB(seq304), cảu32body0; service giữ
player_response_sent cũ và không gửi106 mới. Không cóactualB95/A104 saurefresh, dù OSReadyreturn0.
NativeSLOT110 ởphòng đọc đủ/error0/ownchiefA1/B0. Chưa chứng minh thiếu106 gây khôngReady; chỉ là
input/caller/consumer cần đối chiếu theo contract room-player, không tự sửa dựaopcode. Ảnh trướcReady2
đen, không là nativehang hoặc rootcauseproof.

Hai phép UI phụ giữ nguyên log: nsenter explicitgid bị setgroupsdeny trướcUI; bản preservecredentials
chọnnhầmDISPLAY0 của Wineboot/serverprocess mangprefixB, khác actualcontrollerstatesB_env/DISPLAY1.
Ảnh raw`B/round2-ui-feedback-focused.png` thực tế làAhost/Start, **loại khỏi Bproof**. Click đó khôngphát
104 hoặc95, không suy serviceđãrejectStart. Orchestratorresume trongfinally, Wine/service/GDB tiếp tục;
không nativefunction/stateforcing, không controller/client/lease thứhai. `ui-feedback-*.json` và
`round2-render-followup.json` ghi correction/provenance, không lấytênfile làm ownership.

Followup chỉ thêm controllerstatesA/Bfocus/chờ12s/render trướcReady2/Start2, service/profile/route giữ.
Nhưng observerbanđầu100s từJoin dừng trướcphase153: last99→stop1,506506725s, trong khi policy
197→196→153 cần≥2s; native121cảhai1lần, clock/rank/return0. Đây là observationbound, khôngnative/
serverprotocolfailure, và **chưa đo điều kiện Ready2 mới**. No unchangedretry sauhai boundedattempt;
phần còn thiếu chuyển targeted585/Readyconsumer, không tănghold hoặc tìm lại tấtcảwriters.

Service/profile41rawfixture/sourcebinary/pack/DB giữ CP077. RuntimeSHA
`e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`; static
`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`; modO raw/base400000
`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`, RVA=VA−400000.
Privateuser/net/mount/PID/chỉlo/no route/NSSfiles-only/lease đơn; prefix/display riêng. Không semantic
replayTCP/UDP, fixture/newpatch/forcing; fuse/drop/rank/host vẫn design. Controller/serviceerrors[]
cảhai, helperfailures riêng ởtrên; EOFfollowup sau controlledstop, không đổ lỗi client.

Plannedtwo-cycleassert vẫnfail ởfirst-planned-acceptance.txt, verifierplanned giữ. Lỗi preflightchecker
banđầu đọc hidden-summary khôngdùng/probeabsent đã sửa riêng, không bỏtwo-cycleassert; archived
first-analysis/ rồi followup-analysis/ giữ report/counters/sourcehash trước táisửdụng tênproducer.
Boundedpass first:3013joins/2802integration,945nativeframer/174typedpacket/6native153/246secure;
followup:941consistency,354nativeframer/66typedpacket. Input trùng không cộng thànhproof độc lập.
Nativecallbacks/state/raw khác syntheticreceiver/heap/getter models emulation. Ảnh sequential cótime/
vịtríkhác, không khẳng định fullmovement/frameperfectsync từảnh.

Reuse83server/18checkpoint tests CP077/sourcecode unchanged;7catalog/docs/artifacts5/5/localhistory/
diff kiểm bàn giao. AUD023/CP076, periodicCP080. Chưa gate/fullmatch/K5/K6a/Windowsvanilla/LAN/
Internet/fullauth/codec/validator/reset/activeleave/hostlivepeer/accountresume/currenthaiownerbom.
CP075hai-cycle guardpartial vẫn riêng; CP078 không replayguard để thay lịch sử đó.

Recipe `<label>-recipe.json`, rawseal `<label>-hashes.json`. Firstcontroller _rejoin_battle_cycles.py,
followup _rejoin_battle_render.py;550s budget, rank/return/refreshpolicy không rútngắn. Attempt
`rejoin-battle` argparsercap360 từchối trướcruntime, giữlog/recipe; privatecap600 đểchophép550,
label/prefix mới. Phép kiểmplanned/partial và correction đều cóhash trongmanifest. Không overwrite
raw/label hoặc chạy chồnglease. Bước tạiCP078 (lịch sử): đọcnguồn585→106/Ready và đo phầnthiếu.

[CP079counterexample/Readytracegiánđoạn](../return585-ready/README.md) khôngchứngminh cần106mới; CP078cause giữunknown, việc hiện hành chỉPLAN.
