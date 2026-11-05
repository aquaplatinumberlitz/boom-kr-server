# CP107 — Normal host Back/chuyển chủ/rejoin trong service hiện hành

KRE-070/[issue174](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/174), baseline CP106/
[PR173](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/173)
`f67a160d20e9cb0d4b964da7a132aa009b129102`. Codex tự kiểm, [review waived](../checkpoints/review-waiver.json),
CP107 đã merge [PR175](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/175) tại `ac6562e`. [Contract/giới hạn](../../PROTOCOL.md#host-normal-back),
[câu hỏi](../../RESEARCH.md#gap-room-membership), [thứ tự](../../PLAN.md#current-order).
AUD032 qua CP104; audit định kỳ kế **CP108**, không reset chu kỳ.

Raw `local/research/codex/host-normal-back-r1/`; exactargv ở `<label>-recipe.json`, size/SHA ở
`<label>-hashes.json`/`sealed-verification.json`:1.566 file across3runs, không sửa sau seal.
Controller copies `wine_probe_host_normal_back_{control,fixed}.py` dưới local/research/codex.

| Run | Kết quả | SHA256 run.json |
|---|---|---|
|control|A SIGABRT trước vào phòng/khôngBack; startupfail riêng, không dùng làm controlhostleave.|`9d6305af77fe046703ccd6b74a0f55379372799d9db423697cdd7ad58088695a`|
|control-r2|NormalA86empty/seq139 bị currentservice reject unsupportedvariant; không107 trong15s trướccontrollercompletion.|`a97f75b886eb40184045f8b18215d0fa6d97ee5f52ecd7fb2376050285c66aef`|
|fixed|168,144543118s; normalA86→B107clear/chief1→AnormalLobby→A82/585/guestchief0→A95/B104→12199/clockcảhai;0errors.|`e9c80c455212efcc17000e0ca82315ceb1ef8e22862aee43165369443854827f`|

`reuse-question.json`18pins: CP034/035 đã có exactKRcontract, CP053cleanup, CP076/077service/sequence,
CP106currentcopy/profile/proxy. `rejoin-r1/prepare_promoted.py` và năm fixture CP035 dùng trực tiếp;
VN lobby §0/§18 chỉ đối chiếu normal-stage/session vs membership/host không cố địnhslot0, không
port opcode/layout/offset. KRdata/boom-rs/l10n/archive là context, không thêm contract. Không đọc
thân hàm native mới hoặc tìm lại87/107/110/auth/cache/clock. Nativecopy SHAe9bb4766…/static660447…/
modOrawa08950…/base400000/RVA=VA−400000 giữ nguyên; sourcebinary/pack/DB không sửa.

Mỗi run user/net/mount/PID riêng, lo-only/no routes/filesNSS/mộtlease/controller; A/Bprefix/display
riêng. Giữ recipe bootstrap/globalguardreset đã đo; không thêm actor forcing/gamecall/patch.
Control đầu dùng bộ probe lịch sử quá rộng và gặp SIGABRT; followup dùng CP106gọn+known107/chiefprobes,
không khẳng định observer là nguyên nhân startup. Cảcontrol-r2/fixed cùng bộ probe/routeback.

Implementation **design/opt-in** `host_back_rejoin.mode=idle_A_to_B`, defaultlegacy giữ nguyên.
Exporter `--host-back-rejoin` chỉ thêm5alias pin đúng CP035bytes/seq, profile hiện hành42fixture cũ
không đổi/47total. Preflight missingalias/sequence trước owner/membershipmutation; chỉhostAidle còn
Bmapped/live, không activeleave/accountresume/generalroom. A membership archive nhưng channel/
identity/seq/keyallocator/job history giữ. Không gửi107 cho leaver; Bnhận107slot0/id1/flag0 rồi110chief22.
Arefresh dùng roomlist_B vì phòngB còn tồn tại; chỉA82 vào đúng sameBconnection, khôngA81 tạo đè.
Known87slot0/session1/own106Aguest0/110guest0/B106chief22, broadcastA106choB; A95ready4/B104Start.
Refreshsaukếtquả có aliasguestA thayC khi Ađãrejoin; chỉ modeltest, chưa runtimepostresult ởCP107.

`host-runtime-verification.json`:42boundedchecks/control/fix, sameTCPfront86/admissionSHA/body38,
native107base6/rewind0/final1/errors0/ID1→0, Bchief1/Aownmatch1/chief0, native87full7,
actualA82/585/95/B104/currentseq/native121full129/99/153/clockcảhai.72typednativeframer/model cases
và perroleframer negatives tách khỏi nativeUI. `fixed-contact.png` đã đọc6ảnh lobby/promotedroom/
clock2views; `visual-review.json` giữ nhận xét/hash, không screenshot-only gameplay proof.

`runtime-check.txt` negativewholecapture107 fail giữ nguyên: teardownAEOF phátB107 **183,815997ms
sau** controllercompletion. `verify_host_runtime_scoped.py` kiểm boundary15s trước completion,
không gán teardown107 cho Back86; `analysis-boundary-correction.json` giải thích. Raw không sửa.
157server tests (8new), exporterhost1/all-dead1 pass; profile47actualfixtures/prelaunch41pins/roleB
kiểm. Control source/finalsource snapshots riêng, không đổi code đang chạy giữa run.

CP106/PR173 đã merge; issue166/172 đóng theo qualification hẹp, không gate. CP107 chỉ tới clock,
chưa bom/item/result/reset/hai trận sau đổi chủ. Bước tạiCP107 (lịch sử) là reuse CP106route/observers/currentpolicy,
đổi roleinputReadyA/StartB và ghép hai Battlecycles; auditCP108. Fullauth/codec/authority/validator/
activeleave/accountresume/generalroom/fullreset/vanilla/fullmatch/K5/K6a/LANInternet còn mở.
