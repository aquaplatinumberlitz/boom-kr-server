# CP097 — Current terrain, TCP guards và hai native cycles

KRE-060/[issue154](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/154), baseline
CP096/[PR153](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/153)
`fb3a9a257bb8f8b5b0cbb46054f6c7d031d7198f`. Codex solo/tự kiểm,
[review waived](../checkpoints/review-waiver.json), CP097 chờ PR. Contract/limits ở
[PROTOCOL](../../PROTOCOL.md#terrain-tcp-guard-cycles), gap ở [RESEARCH](../../RESEARCH.md#gap-diagnostic-service),
thứ tự ở [PLAN](../../PLAN.md#current-order). Không production change.

Raw dưới `local/research/codex/{terrain-tcp-cycles-r1,terrain-tcp-peer-r1,terrain-tcp-stable-r1}/`.
`reuse-question.json` pin currentCP094/085/096code/profile/route/flags/nativeoracles, oldKRdata/flamechain
và VNgame/session; repeat pin thêm actualfirstmissingreportrole. VNsessionfixed12345 không là KR epoch.
Tái dùng toàn map/codec/getter/body-proxy, không mở nativebody mới. `proxy-selection.diff` chỉ selector:
A118/124 và B127/17,19, exactcapturedbody/currentouterseq/S2Cunchanged. Nativefront khác proxyupstream.

| Run | Boundary | run.json SHA256 |
|---|---|---|
| cycles |299,222s, currentterrain2cycles positive/14entities/12callbacks/view/52snaps nhưng A-onlyproxy chỉ7replays, thiếu17/19; targetpartial. | `854a5cabe52fa2cfd91fd12a652e40a37a5eb0d7e7f7c8562ffc7402665e5e4d` |
| peer |184,660s, samebody17/19roleB đo được, nhưng inputLeftmiss sauUpovershootY119,6entities vòng1; targetpartial. | `25920cf276aa6427296d81d766ddbb7fa282ba8e73dab1a4b5d2169dee70e73e` |
| stable |306,328s,12replays/14nativeentities/11callbacks/view/48snaps/hairesultreset, targetguard+terrain pass với actualfence boundary. | `2a4fd20b406698c98ef8beb61f772a3516a00c2e19225450863c9780e77d1edf` |

`<label>-recipe.json` exactargv; `<label>-hashes.json` pin originalfinalraw. Tổng5007sealedrawfiles
được kiểm size/SHA. Một controller/lease/privateuser-net-mount-PID namespace, chỉlo/noIPv4routes/
files-onlyNSS. NormalOSinput/read-onlyfeedback; sourcebinary/pack/DB không sửa. RuntimecopySHA và
12sourcechecks ở `source-verification.json`; production/profile bytes giữCP094. Newproxy selectors
được kiểm actualfront/upstream/nativepacket/reject chứ không lấy oldCP085bench làm chứng minh bản đổi.

Firstplanned12replaycheck fail và151boundedchecks chỉ xác nhận7observedreplays ởcycles; peer16checks
xác nhận4samebody nhưng controllerfail giữ nguyên. `reports-launch.txt` và bad-rootanalysis source giữ,
correction exactroot replacement chỉ sửa đường dẫn parser, không runtime.
Stablepreparation metadata firstattempt đọc reuseJSON chưa chép sau khi code/profile/GDB/prefix đã tạo;
shell tiếp launch. `preparation.json` ghi rõ reconstructpostlaunch, không giảpreseal. OriginalharnessSHA/
policy_source/profile khớp; giới hạn provenance vẫn nêu rõ.

Final `runtime-verification.json`128relations/48native195flags, `transport-verification.json`219joins
(8same4old/actualoriginbody/seq/SHA/reject/noextraID165170/acceptednativefront), `effects-verification.json`
43checks (BombFire1→2/Start2baseline1/pre175A0B1/rankstorage1,2/full175302);9.456copiednativegetter/
9.360arraychecks và1194frontframer/486native-reportcases là emulation khác nativecallback/runtime.
Không có native170entrycounter. Hai typed175/180/roomrefresh cảhai view không tự nghiệm thu fullmatch.

`verify_transport_planned12.py`/`transport-planned12-failure.txt` giữ expected12callbacks fail;
`callback-count-correction.json` nhận đúng11actual158/view/48snapshots, exactunsent7/13/14round_closed/
no158/noterraincommit, không sửa raw hay bịa3callback. IDs7/13/14 chưa nativeprocess trướcdispose;
không gọi14entities là14lầnnổ hoặcallprocessed. Hai extras mỗi vòng vẫn trước oldsecondaryownjob;
geometryactual118 là nguồn, khôngép nominalcells.

Docs/catalog7/artifacts/localhistory/diff;113server/4kernel/18checkpointtests tái dùngCP094 vì code
production/checker không đổi. AUD028quaCP096, periodic CP100. Nativeorder/generalmap/fullauth/codec/
location/distance/forgedfreshbody/fullvalidator/allreset/vanilla/LANInternet/K5/K6a/fullmatch còn mở.
