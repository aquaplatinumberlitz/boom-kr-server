# CP101 — Closed recipient trong cached selector batch

KRE-064/[issue162](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/162), baseline
CP100/[PR161](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/161)
`8c69429acc27f847cc699f1dc6efa83d9484082a`. Codex solo/tự kiểm;
[review waived](../checkpoints/review-waiver.json), CP101 chờ PR. Contract/limits duy nhất ở
[PROTOCOL](../../PROTOCOL.md#selector-closed-readiness), gap ở [RESEARCH](../../RESEARCH.md#gap-diagnostic-service),
thứ tự ở [PLAN](../../PLAN.md#current-order). AUD030qua100/next104. Không nâng gate.

Raw/model/source/testlog `local/research/codex/selector-closed-readiness-r1/`.
`reuse-question.json` pin currentcontract/CP085cleanup/sendfailures/CP099gatederrno9 + servercode/tests,
VNsession/KRdocs scopedsearch; không có newnativebody cần RE. Tái dùng testfixture/socketpair/attribution/
partialsuccess/ENOSPC. Không port opcode/offset/authority VN; khoảng thiếu chỉ cachedbatch lifetime.

`reproduction.json` actualselect() trả1,2, execute1→2: Afd4 nhậnEOF, cleanupcloseA/B và ghi đúng B EPIPE107;
key2 vẫn dispatch với fd−1/closedtrue, recv raiseerrno9. `reproduce.py`/`reproduction.txt` giữ trước sửa;
`tests-before.txt`2fail directrecv/errno9. Đây là currentservice Linuxsocketpair model, không nativeKRrun
hoặc proof exactstack của oldCP099connection8.

`postfix-batch.json`/`.txt` cùng recipe/keyorder/lifetime sau fix trảno-op ở closedB, 1send_failure đúngrecipient,
không thêm semanticadmission. `server-tests-final.txt`129tests gồm5negative/positivebatchcases mới;
actualrunloop/save/finally kiểm không ghi spuriouserror. Markedclosedno-op/reverseorder/idempotency,
liveBbytes khôngdrop khiAEOF và unmarkedbadfd vẫnraise. ExistingfileENOSPC/secondsendpartialsuccess
vẫnpass. Source/hash report trongmanifest; không broadcatchEBADF hoặc sửa kết quả/fuse/fixtures.

Không newWine run: production delta chỉ guard cho alreadyclosed event; livepath có explicitcontrol.
CP100native2cycles giữ scope lịch sử, không dùng làm runtime của CP101 hoặc causalfix olderror.
Fullauth/codec/fullvalidator/authority/allreset/vanilla/LANInternet/fullmatch/K5/K6a vẫn mở.
Bước pendingbomb/resultintegration chỉ ởPLAN, không thêm plan theo phiên.
