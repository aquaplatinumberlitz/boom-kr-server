# Server tương thích KR

Code server mới thuộc thư mục này. Harness RE trong `local/` đã chạy các chu kỳ hai client
Wine theo [contract](../docs/PROTOCOL.md); CP048 thêm service fixture riêng theo [phạm vi](../docs/PROTOCOL.md#probe-service), chưa nghiệm thu toàn codec/hello.

`round_fence.py` là policy emulator cho vòng đời **job local**: ticket gắn instance chủ phòng,
generation vòng và lần cấp ID; chỉ job hiện tại được consume một lần. Caller phải giữ ticket gốc,
gọi `begin()` khi Start được chấp nhận, `finish()` khi kết thúc/hủy vòng và serialize mọi thao tác
cùng việc phát packet dưới owner/lock phòng. Timer còn giữ mỗi `wire_id` không được đổi sang ticket
mới khi chạy. `register()` không tự chọn ID; nó từ chối ID đang active hoặc ngoài1..65535.

Đây là design của server, không thêm field generation vào wire KR. Nó không kiểm request từ client,
deadline, chain đã xử lý tại client, UDP, packet đã gửi hoặc allocator wrap/quarantine. Các phạm vi đó
vẫn cần kiểm riêng; không gọi module này là toàn validator hoặc đủ đóng K5/K6.

CP043 đo [159 cũ khi serial local trùng](../docs/PROTOCOL.md#late-bomb-confirm): caller cũng phải kiểm
ticket gốc trước phát159. `rejection()` chỉ kiểm lifetime; duplicate159 trong cùng generation cần
quy tắc riêng, không consume ticket timer chỉ để gửi ACK.

CP044 kiểm [timer còn chờ khi host EOF](../docs/PROTOCOL.md#abort-timer): caller gọi `finish()` ngay
khi quan sát abort dưới owner/lock; job cũ tới hạn bị loại `round_closed`. Module không tự theo dõi
socket/roster hoặc hủy timer trong scheduler; mỗi đường gửi vẫn phải kiểm ticket gốc.

Kiểm policy: `python3 -m unittest server.test_round_fence`.

`item_claims.py` giữ catalog item hiện tại, kiểm slot gắn socket/context/index/hàng/cột/kind,
chọn một claim đầu cho mỗi index và chỉ consume xác nhận local một lần. `begin/finish` làm
ticket cũ hết hiệu lực, kể cả khi dùng lại index. Caller serialize claim/consume/phát165 cùng owner phòng.
Đây là policy thử; chưa tự kiểm distance, thời gian, session key, actor state hoặc UDP. Nếu request cũ
có mọi field trùng catalog hiện tại và wire không có epoch đã xác nhận, module không phân biệt được nó;
không gọi đây là toàn validator chống stale124.

Kiểm cả hai policy: `python3 -m unittest server.test_round_fence server.test_item_claims`.

`udp_round_keys.py` là policy cấp khóa cho [nhánh movement đã đo CP047](../docs/PROTOCOL.md#udp-generation).
Owner phòng serialize `next_key()` một lần mỗi Start được chấp nhận, dùng cùng giá trị ở u32 thứ hai
của153 cho mọi client. Giữ instance khi kết thúc/hủy vòng; module không có reset/release tự động.
Sau256 giá trị byte thấp khác nhau, allocator báo lỗi; không quay vòng rồi giả định gói cũ đã hết hạn.
Chỉ tạo instance mới khi caller đã thiết lập lifetime peer mới an toàn; việc đóng transport/loại queue
để bảo đảm điều đó còn cần kiểm riêng. Roster/ID thay đổi cũng phải kiểm identity riêng.

Runtime CP047 kiểm12345→12346, stable IDs1/2 và một gói movement cũ. Module CP047 chỉ qua unit test; CP048 đã ghép vào service fixture theo phạm vi bên dưới; các key khác/biênu32 là kiểm policy, không thêm bằng chứng runtime KR.
Không gọi đây là xác thực, toàn UDP validator hoặc bảo vệ mọi loại packet. Fixture RE CP047 chọn hai
giá trị trực tiếp qua153; runtime đó không tự chứng minh caller đã tích hợp module này.

Kiểm policy: `python3 -m unittest server.test_udp_round_keys`.

Lộ trình và điều kiện mở probe server: [PLAN](../docs/PLAN.md).

## Service chẩn đoán CP048

`probe_service.py` chạy tiến trình riêng, sở hữu TCP/frame/seq, round owner và timer; harness chỉ
Wine/UI/isolation/UDP đo lường. `probe_codec.py` là profile hẹp đã so với actual frame/native KR;
`probe_room.py` ghép key/fence/claims theo owner serialize. Fixture profile chỉ trỏ path/size/hash,
không chép tài nguyên vào Git. Chi tiết native/partial-run/recipe ở [evidence](../docs/evidence/probe-service/README.md).

```bash
python3 tools/export_probe_profile.py --source-root local/research/codex --out local/new-profile.json
python3 -m server.probe_service --profile local/new-profile.json --out local/new-service-run --port 3808
python3 -m unittest discover -s server -p 'test_*.py'
```

Service chỉ bind loopback. Khi chạy Wine phải dùng namespace/lease/files-only NSS của recipe đã đo;
loopback binding riêng nó không cô lập outbound client. Cần đủ các port của fixture/harness, không
chỉ3808 cho chuỗi runtime. Profile CP051 cho identity A/B rồi C thay A theo accept-order (4connection/vai),
synthetic auth/map/roster/rank và stable slots. Không là public server hoặc login/auth hoàn chỉnh.
Service chưa quản membership tổng quát/account reconnect, full124validator/timepolicy/actorstate,
relay/NAT/backpressure hay gate K5/K6a. CP050 bổ sung [EOF recovery hẹp](../docs/PROTOCOL.md#service-eof); mô tả chỉ finish là snapshot CP049.
Runtime r2 có hai chu kỳ; r3 chỉ một result, không dùng để tuyên bố tính tái lập mọi input.


## Các bổ sung theo checkpoint

Các đoạn dưới ghi phạm vi tại checkpoint được nêu. “Tiếp” trong đoạn lịch sử không là việc đang
hoạt động; tra [PLAN](../docs/PLAN.md#current-order), [contract hiện hành](../docs/PROTOCOL.md#authority)
và [coverage](../docs/COVERAGE.md). Raw và profile cũ giữ hash, không tự migrate.

CP049: `probe_service.py` so u32 k của124 với khóa153 hiện hành trước claim, khác thì không phát165
và không chiếm item. Đây là policy server ở caller, không đổi ItemClaims thành toàn validator.
[Contract/giới hạn](../docs/PROTOCOL.md#pickup-round-key), [hai run native effect và recipe](../docs/evidence/stale124/README.md).
Tests kiểm old→current→duplicate, origin/catalog vẫn áp dụng và key+256 cùng low8 vẫn bị policyu32 loại.
Tại CP049 không xác thực sender hoặc kiểm t/distance/actorstate; CP055/056 bổ sung nguồn t/reportguard
ở các đoạn dưới. Body chèn tại dispatcher chưa proof transport replay.


CP050: channel close vào một đường idempotent, loại closedpeer khỏi sends/setup, finish vòng ngay
để ticket bom cũ không phát158. Sau forfeit/return mới107/110 cho survivor. Delay12/5/3s, rank,
slot/ID/promote là policy fixture; [native evidence](../docs/evidence/service-eof/README.md) kiểm cả host/guest.
Tại CP050 profile cần34fixture, export lại bằng tool ra path mới; profile32 cũ fail rõ trước listen.
33server tests tại CP050 gồm5EOF/profile regressions. Snapshot CP050 chỉA/B/cap8/StartA;
CP051 thay giới hạn C mới/BStart theo phần dưới; account reconnect/idle-EOF/setup/sendfailure race vẫn mở. Một wrapper giữ99 10s đã kiểm wait-both hẹp,
không chứng minh mọi timeout hoặc full loading gate.


CP051: [C mới thay A/B Start](../docs/PROTOCOL.md#service-replacement) đã tích hợp và có một vòng
B/C bom/death/result/return trên Wine copy; run đầu partial giữ riêng. C có ID3/slot0, B giữ ID2/slot1/chief;
không giả C là A reconnect. Profile cần **41fixture**, export ra path mới; profile34 thiếu bảy aliasC
fail trước listen. Max12connection theo thứ tựA-B-C, chỉ hai live members; admission chờ cleanup A.
36server tests có3replacement regressions cho Ready/host/roster/key/seq/ticket và report21 đúng peer.
Auth/map/rank/roster vẫn fixture, chưa login server công khai, membership hoặc fullvalidator/reset.


CP052: [guest setup EOF](../docs/PROTOCOL.md#setup-eof) cleanup theo Join/create acceptance,
không đợi map. Control bản cũ còn slot ma; bản sửa native107 xóa ID2→0, A giữ chủ, không false phase.
39server tests có3case thêm cho before-map/before-own106/lobby-only/duplicate và Start với closedpeer.
Tại CP052 host EOF/survivor chưa map còn gap; CP053 bên dưới đã đo deferred cleanup hẹp,
không membership tổng quát.


CP053: [host setup EOF](../docs/PROTOCOL.md#host-setup) chọn host từ member đã Join và queue
cleanup cho recipient chưa map, flush107/110 sau setup; closed recipient bỏ queue. Native B ID1→0,
chief0→1 ở control/fix;42server tests. C trước trận đầu còn bị refresh guard, chưa account reconnect.


CP054: [C trước round1](../docs/PROTOCOL.md#prestart-replacement) được nhận khi B idle/mapped/chief,
A đã đóng và107/110 gửi xong/không pending; post-round path giữ, active/lobby/setup bị chặn.
Rejection có reason/time, không assign role.45server tests; native một B/C playedround sau setupEOF.
Không account reconnect/general membership/fullauth/fullvalidator/K6a hoặc LAN.

CP055: service log `client_tick` của124; [nguồn native/giới hạn](../docs/PROTOCOL.md#pickup-tick).
Không thêm time threshold; hai Wine run dùng source CP054 trước đổi logging, actual-capture bench
kiểm log mới. Chưa original-server time/actorstate/distance validator.

CP056: [guard theo report](../docs/PROTOCOL.md#pickup-report-state) chặn124 sau17/19 trong cùng
generation trước claim/consume; Start xóa marker.50tests cùng native control/fix/actualpickup vòng2.
Policy chỉ fixture Roller hiện hành; rescue/selfescape chưa hỗ trợ, phải nối1/18 trước mở kim/cứu.
Không originalserver stateauthority/auth/distance/TCP replay; không dùng marker làm enum native.

CP058: exporter41fixture chuyển auth_A/B/C sang [profile count1 mới](../docs/PROTOCOL.md#auth-profile).
Tại CP058 A/B callback137/137/errors0/actual6 tên riêng đã đo; C135 mới native/emulation.
CP060 bên dưới bổ sung C runtime, không sửa phạm vi phép đo cũ.
Profile JSON cũ giữ hash/path cũ, phải export sang file mới để dùng fixture mới. Service/codec không đổi;
values0/emptylist vẫn design, không publicauth/account/cache/vanilla/gateproof.

CP059: exporter đổi ready241/table238 sang [cặp compressed/checksumkhớpcache](../docs/PROTOCOL.md#dynamic-cache-representation).
A/B missing→nativewrite→freshprefixwarm/69bytes/loader1/no528;39alias khác giữ nguyên, oldprofileJSONs giữ.
Không fullcache/reconnect/auth/profile/gate. Tiếp runtimeC với bộfixture mới, không đo lại raw/nénCRC.

CP060: [Ccurrentprofile](../docs/PROTOCOL.md#current-profile-c) đã135/errors0/actual6KRNew/ID3 và1B/Ccycle,
không newcode/protocol. Một Breset cuối controller sau roomrefresh được giữ, không errorfreerun/generalrecovery.
Tiếp2vòngA/B/currentprofile, không Caccountresume/fullmembership/gate hoặc toànauth/cache.

CP063 thêm [admission report127](../docs/PROTOCOL.md#report127-freshness): cursor tick/bodySHA riêng
trên kết nối, giữ qua Start; old/exactduplicate không sửa marker/death/phát170 hoặc góp21 vào result.
Policy design hẹp, không là authentication hoặc toàn validator. Regression57case; control/fix Wine dùng
dispatcher replay, actualTCP replay admission vẫn mở. Recipe/giới hạn ở link contract/evidence.

CP065: exporter mặc định [spawn riêng](../docs/PROTOCOL.md#distinct-spawn) theo exactfixtureCP024;
export ra path mới, profileJSON cũ giữ nguyên hash. `--spawn-layout overlap` dành ca contact/race RE.
Native currentA/B2cycles đã đo index0/1→point→actor/reset; B/C/accountresume/generalroster chưa kiểm
với default này. Metadata chỉ khai báo fixture, không thêm fieldwire hoặc randomspawnpolicy.

CP067 thêm [guardC2S118](../docs/PROTOCOL.md#c2s118-admission): fullfield0key và serial/bodySHA
riêng boundowner/round trước cấpID/timer;Start xóa serialmap, fullkeymismatch/duplicate/conflict log
không159. Policydesign hẹp, không auth/cap/distance/wholevalidator;actualtransportreplay chưa kiểm.
64server tests;control/fix Wine/nativeentity tách với dispatcherinjection trong evidence.

CP068: `--item-layout hidden` xuất profileknownlist2/FixedBox/Bomb+Fire;defaultfloor giữ.
`hidden_items` catalog riêng merged vào claimcatalog,124 chờknowncell142 saucurrent158đãphát.
Duplicate/report/reset kiểm hẹp, **report-deriveddesign**, không collision/auth hoặc chốngold142sau
blast. [Nativeintegration/partial/limits](../docs/PROTOCOL.md#current-hidden-items);72server tests.

CP069: `--item-layout battle` chọn một [profile cảfloor+hidden](../docs/PROTOCOL.md#basic-battle)
đãđo baeffects/reset qua hai vòngcùngrun;floor/hidden/defaultfloor giữ. Exportpathmới/profilehash,
không tựmigrate hoặc coi basicdiagnosticprofile là wholematch/K6a/Internet/authauthority.


CP076 thêm [Back86/emptyroomrecreate cùngchannel](../docs/PROTOCOL.md#back86-membership), chỉ A sauBEOF/
resultcleanup. Archive membershipmarkers, giữ codecseq/identity/roundallocator/jobs; new81/setup dùng
currentseq và không guestBfixture đã ngắt. Export mới khai báo sequencecreate/player_A/map, mọi raw
fixture giữ hash; profile cũ thiếu metadata bị logreject trước mutation. 79server tests gồm7membership/
realFixtureProfile regressions. Chưa guestleave/hostlivepeer/activeleave/generalmembership/accountresume.
CP075 [actualTCPselectedreplay](../docs/PROTOCOL.md#current-tcp-replay) bổ sung các giới hạn transport
lịch sử CP049/063/067 ở trên; planned hai-cycle result vẫnpartial. Thứ tự chỉ theo PLAN.


CP077 mở thêm [guest B idle Back/rejoin với A cònmapped](../docs/PROTOCOL.md#guest-back-rejoin).
107/110 xóaB/giữAhost, archiveBmembership/channellive, refresh241/91, new82/87/585/setup currentseq,
profile hai phía cập nhật lại trướcReady/Start.83server tests; cùngTCP/IDslot/nativeclock được đo, chưa
Battle/result/return saurejoin. Export metadata knownseq cho roomlist_B/join/player_B/guest_A/guest_B,
41rawfixtures giữhash; oldprofile thiếu metadata bị logreject trước mutation. Hostlivepeer/activeleave/
accountresume vẫn ngoài scope. Thứ tự chỉPLAN, không gọi priorharnessCP035 là productionproof.

CP083: optional profile `rank_all_dead` cho policy A/B cùng accepted19, grace8s từ deathcuối, rank2/2. Profile thiếu alias explicitpending; không invent winner/fake21. [Contract/limits](../docs/PROTOCOL.md#all-dead-result), [runtime](../docs/evidence/all-dead-result/README.md). Export thêm `--all-dead-rank PATH` khi có rawfixture đã pin dưới source-root; không tự đổi profile cũ.

CP085: `deliver_departure` ghi recipient/errno/op/successprefix và close khi send107/110 gặp expectednetworkfailure; không coi unsentpacket là đã gửi. 91server tests; [scope/runtime/model](../docs/PROTOCOL.md#tcp-guard-cycles). Chưa general send/broadcast/backpressure recovery.

CP087: optional metadata `bomb_capacity` khai `initial`/`maximum` theo role cho reservationguard trướcID/job; absent giữ profilecũ. AcceptedBomb+1/clamp/Startreset, jobretire/fence là releaseboundary. [Config/source/limits](../docs/PROTOCOL.md#capacity-reservations), [runtime](../docs/evidence/capacity-reservations/README.md). Nativeearlychainrelease chưa có trongpolicy, không tựgán baseC hoặcfullvalidator.


CP089 thêm optional `capacity_chain` cùng `bomb_capacity`: pin raw map/start SHA, explicit clear-cells/
maximum_power để release quota theo direct first-hit sau root158. Giữ own jobs/deadlines/fence, không
release theo report; oldsecondary không mở chain giả lên newbomb. Absent giữ CP087 policy.
[Contract/config/giới hạn](../docs/PROTOCOL.md#chain-capacity-release),
[recipe/control/partial/hash](../docs/evidence/chain-capacity-release/README.md). 105server tests;
chưa generic map loader/recursive chain/fullvalidator hoặc tựmigrate profile cũ.


CP094 optional `bomb_terrain` cùng capacityA/B, mutuallyexclusive `capacity_chain`: fixed15×13 seed/
maxpower≤2/source-mapdigest+seedhash/map-startfixturepins. [Contract/config/limits](../docs/PROTOCOL.md#terrain-policy).
Knownvariant/floor/nonoverlap/currentepoch/quotaguards trướcID/job, worldcommit sau159/158success;
keepsecondaryjobs/fence/reset, no142worldoracle. Failedbroadcast khôngterraincommit nhưngkhôngrollback
existingID/serial/fence/jobs.113tests/newnative2cycles14entities,12callbacks/52flagsnapshots. Profileabsent
unchanged; no automaticexport/migrate/genericmap/auth/fullvalidator. [Evidence](../docs/evidence/terrain-policy/README.md).


CP107: optional `host_back_rejoin: {"mode":"idle_A_to_B"}` tích hợp [host Back/promotedrejoin](../docs/PROTOCOL.md#host-normal-back)
đã biết từ CP035. Export profile với `--host-back-rejoin` để thêm5alias/hash/sequence; legacy không đổi.
NormalA86 vớiBidle/mapped→107/110promoteB, Arefresh/rejoinsameTCP làmkhách, AReady/BStart→clock đã đo.
ReturnguestAalias có modeltest; gameplay/result/reset hai vòng saupromotion chưa đo ởCP107.
Kiểm `python3 -m unittest server.test_host_back_rejoin`; không originalauthority/fullmembership/gateproof.

Report17/19 chỉ nhận serialized envelope variant đãđo trướccursor/marker/death/forward; xem [contract và giới hạn](../docs/PROTOCOL.md#death-envelope-admission). Malformedpacket theo exceptionclose/abort hiện hành; semanticfirstseen/location/inventory chưa làauthority. Không coi guard này làhỗtrợmọi mode/report.

`needle_escape: {"mode":"needle_145_report_pair"}` chỉopt-inmộtkim145/two-liveAB; credit/same-trapreport1→0/peer1701/commit/reset có [contract](../docs/PROTOCOL.md#needle-service). Không bậtrescue18/activeitemkhác hoặc giả inventoryauth.


`rescue_contact: {"mode":"friendly_AB_synthetic2_aux1"}` là legacy option name của fixed-profile
CP115; CP116 hỗ trợ hai variant đã đo A/aux1 và B/aux0. Profilehash/106 trướcStart/153 giữ slot2
và admission/send-success-commit có [contract/giới hạn](../docs/PROTOCOL.md#rescue-reverse).
Hai client thật cộng actor đối địch tổng hợp, không generalteam/roster/auth; defaultoff.
[Control/positive/provenance](../docs/evidence/rescue-service/README.md#cp116), 190tests tại CP116.


CP135: profile có thể bật `loading_timeout: {"mode":"close_missing_init99","seconds":30}`.
Default off; số giây nguyên1–300 do emulator chọn. Deadline pin generation/channel sau121,
current dispatched barrier thắng expiry trong owner tick; hết hạn đóng riêng thành viên chưainit99,
vô hiệu hóa fence và tái dùng close/forfeit/return/cleanup hiện hành. Không giả99/death/cancelpacket.
[Contract/giới hạn và runtime control/fixed](../docs/PROTOCOL.md#loading-timeout);
`python3 -m unittest server.test_loading_timeout` kiểm local policy, không là original-server oracle.
[CP136 positive normal/reset](../docs/PROTOCOL.md#loading-timeout-positive) đã qua trên same-profile all-dead;
tổ hợp normal movement/survivor/current-trap/result-drain theo PLAN vẫn cần cùng-run proof.
