# Tool của server KR

`check_docs.py`: kiểm file đích của link Markdown nội bộ, không gọi network; **không kiểm fragment/anchor**.
`artifacts.py verify`: kiểm SHA/size đầu vào RE theo manifest **host-local**; chỉ đọc source.
Máy khác dùng `--manifest /path/to/local-artifacts.json`; tool không tải hoặc tự tạo binary nguồn.

`verify_primitives.py`: chạy byte helper KR gốc trong Unicorn, kiểm output, guard byte và bounded
return. Chỉ nhận image unpack có SHA đã khóa; không chạy client/bootstrap hoặc network. Cần dependency
RE tùy chọn trong `requirements-re.txt`; kết quả mới phải có output path riêng, không ghi đè raw cũ.

```bash
python3 tools/verify_primitives.py \
  --binary local/research/dsh2/kr-full.exe \
  --out local/research/codex/primitives-run2.json
```

Phạm vi bằng chứng: [writer primitive](../docs/evidence/primitives/README.md). Tool không xác nhận
header/checksum/crypto/seq hoặc mode selection của packet/socket đang chạy.

`build_wine_copy.py`: dựng đúng recipe KR copy đã đo, khóa SHA3 đầu vào và output, ghi before/after
từng patch vào report local. --recipe bootstrap-r1 là mặc định, SHA output giữ nguyên;
--recipe post-bootstrap-r2 là profile riêng đi tới cửa sổ/kết nối thứ hai theo
[evidence R2](../docs/evidence/wine-r2/README.md). Cần `pefile==2024.8.26`; candidate metadata là artifact host-local.
`wine_probe.py`: chạy bounded trong namespace riêng, fresh CALog/window/screenshot/Winsock và accept
TCP loopback mặc định silent; `--bootstrap-reply` gửi fixture local một lần trên connection đầu tiên. Runtime paths chỉ nhận copy dưới `local/`; xem
[recipe Wine](../docs/evidence/wine/README.md) để biết lệnh chạy và giới hạn.

Harness/probe/bootstrap và codec mới đặt tại đây hoặc `server/`; không ghi vào tool repo VN/KR cũ.
Raw output, bản copy client/DB và capture đặt trong `local/`, không commit.

`verify_bootstrap.py`: native length/copy/receive/reader với modO SHA khóa, allocator tổng hợp,
254 ca; không thực thi full ProcessPatch hoặc xác nhận hello game. Lệnh và giới hạn ở
[bootstrap wire](../docs/evidence/bootstrap-wire/README.md).

check_checkpoints.py: kiểm manifest/counter append-only theo Git,required fields,docs-sync links,
evidence hash,closed-task/active-next-step và cadence audit;dùng lại check_docs.py.
Trên host chạy --require-local --require-history;clone thiếu raw báo UNVERIFIED khi không flag.
Pending review hiện WARNING,không thành semantic approval. [Quy trình/giới hạn](../docs/evidence/checkpoints/README.md).
Tests: python3 tools/test_checkpoints.py.
Audit có thể ghi một `AGENTS.md` ở đầu trước chuỗi liên tục README→PLAN→PROTOCOL→RESEARCH;
checker vẫn từ chối thiếu/đảo docs lõi hoặc prefix khác. Đây là tương thích metadata AUD057,
giữ nguyên audit/checkpoint complete và không tự xác nhận nội dung đã đọc.

`feature_catalog.py --generate`: sinh [coverage/cây/thẻ cơ chế](../docs/COVERAGE.md) từ
[catalog ID ổn định](../docs/features.json). Không flag: kiểm bản sinh đã đồng bộ, schema/trạng thái,
file/anchor, dependency/parent DAG và ID/key so HEAD. RESEARCH giữ nguồn câu hỏi/phép đo để trích;
PLAN giữ thứ tự. Check này cũng chạy trong `check_docs.py`; kiểm Markdown thường vẫn không kiểm fragment.
External URL không fetch. Tool không suy nghĩa/kết luận, xác nhận runtime hoặc tự đóng gate.
Negative tests: `python3 tools/test_feature_catalog.py`.

`export_probe_profile.py --source-root local/research/codex --out <local/profile.json>`: chỉ lập index path/size/SHA/sequence của
fixture RE hiện hữu cho [service chẩn đoán](../server/README.md); không nhập binary/asset vào Git.
Profile dùng chính xác bytes/seq đã đo, không là schema auth/cache tổng quát hoặc oracle tự sinh.

CP048: checkpoint checker phân biệt docs hiện hành với snapshot cũ. Khi `.md` repository đã đổi,
chỉ chấp nhận bytes lịch sử khớp size/SHA chính xác từ Git và báo revision; không bỏ qua mismatch.
Code/raw/local/binary vẫn so đúng file hiện tại. Negative test kiểm cả tìm đúng bytes cũ và từ chối
hash không tồn tại; không sửa lịch sử checkpoint để cập nhật canonical docs.

`terrain_shadow.py`: facade offline cho purekernel `server/terrain_model.py`, model candidate cho profile map tường/box/floor rõ ràng và simple
bombpower≤2. Không service/socket/report; dùng API `TerrainShadow`/`Bomb` để so retained native
callbacks/flag snapshots trước khi tích hợp policy. MapSHA là metadata, caller phải kiểm actualbytes.
[Contract/giới hạn](../docs/PROTOCOL.md#terrain-shadow),
[phép so/source](../docs/evidence/terrain-shadow/README.md). Constraints:
`python3 tools/test_terrain_shadow.py`; tests/model output không thay native oracle hoặc đóng gate.


`export_probe_profile.py --host-back-rejoin`: opt-in5fixtureCP035 cho normalAhostBack/Bpromotion/Aguest
rejoin/ReadyA; chỉ thêm index path/size/SHA/sequence vàpolicyname, không phân phối binary hoặc tựchứngnhận
hành vi. [Contract/currentlimits](../docs/PROTOCOL.md#host-normal-back). Kiểm `python3 tools/test_export_host_back.py`.


`check_issue_links.py`: read-only GitHub ↔ currentcatalog/PLAN/taskdeclarations, dùng
[datedsnapshot/review](../docs/evidence/issue-reconciliation/README.md) mặc định; `--live` chỉ đọcGitHub,
`--capture PATH` tạo snapshot mới cóUTC/caplimit. Check open/closed/backlinks/URL/PRexistence/PLANactive
markers vàclosedtask-active-step; không tự đọc hiểu acceptance, đóngtask/gate hoặc sửacompletedhistory.
`check_docs.py` gọi offlinecheck; snapshotcũwarning>24h, output luôn nêuas-of. Kiểmnegative:
`python3 tools/test_issue_links.py`. Thứ tự công việc vẫn chỉ ởPLAN, issue là nguồn workstatus.


`patch_wine_compare.py --input local/.../CA-r2-stage-social.exe --out local/.../CA-compare.exe --report local/.../compare.json`:
tạo copy riêng từ đúngSHAe9, sửa duy nhất reconstructedimport16EA1CC sanglstrcmpiA, outputSHA4b781f…;
chỉ paths dưới local,khôngoverwrite. [Native/control/giới hạn](../docs/PROTOCOL.md#own-profile-result-compare).
Không đổi recipes R1/R2 hoặc binary nguồn; không là general IAT repair.


<a id="portable-bundle"></a>
## Gói service/fixture để chuyển thư mục

Dùng exporter/preset hiện có; output và fixture nguồn phải ở `local/` của checkout khi tạo gói:

```bash
python3 tools/export_probe_profile.py --source-root local/research/codex \
  --preset two-client-basic --own-profile-291 --bundle --out local/lan-bundle
python3 -I local/lan-bundle/run.py --verify-only
```

Sau khi chuyển **nguyên thư mục** sang host/thư mục dự án khác, chạy `python3 -I /path/to/bundle/run.py`
với các option service hiện có: `--out /fresh/run`, `--port ...`, `--bind-ip`, `--channel-ip/--channel-port`,
`--peer-ip A=... --peer-ip B=...`. Không truyền `--profile`; profile đã cố định trong gói. `--out` runtime
nên là absolutepath vì launcher đặt cwd về rootgói. Không ghi đè thư mục export; tạo gói mới khi đổi profile.
Chỉ Python stdlib, không cần PYTHONPATH hay RE dependencies để chạy service. Gói local chứa wirefixtures
đã pin, **không commit gói/fixture binary**. Manifest kiểm drift/size/SHA, không chữ ký hoặc ngữ nghĩa.

[Contract/giới hạn runtime](../docs/PROTOCOL.md#portable-probe-bundle) và [PLAN](../docs/PLAN.md#current-order)
giữ nghiệm thu/recipe. Operator phải thiết lập mạng cô lập trước runtime; bundle không tự đổi mạng/hosts
hoặc liên hệ endpoint. Startup hiện tuần tự A tới channelready rồi mới B; hai namespace không hai máy LAN.
Kiểm packaging: `python3 -m unittest discover -s tools -p 'test_probe_bundle.py'` (real TCP/relocation,
source/destination drift, symlinkescape, overwrite và profileoverride controls).

<a id="incoming-admission"></a>
## Incoming admission A/B theo IP nguồn (opt-in)

Dùng hai host thử có IP TCP nguồn riêng; quảng bá UDP qua NAT vẫn cấu hình độc lập bằng `--peer-ip` và
`--proxy-a/--proxy-b` port đã phục hồi. [Contract/giới hạn](../docs/PROTOCOL.md#operator-incoming-binding),
[phép thử tiếp](../docs/RESEARCH.md#gap-lobby-identity-followup).

```bash
python3 -I /path/to/new-bundle/run.py --out local/run --port 3858   --bind-ip 10.77.0.1 --admission-ip A=10.78.1.2 --admission-ip B=10.78.2.3
```

Thiếu tùy chọn giữ legacy serial/globalordinal. Nếu bật phải đúng A/B, distinctIPv4; unknownsource và
kết nối thứ5 của role bị reject. Count không reset sau EOF, không kết hợp replacement_slot; không
accountauth/resume/nhiều client chung sourceIP. Gói cũ immutable không có tùy chọn mới: export bundle
mới từ source đã kiểm; không sửa file hoặc bundle.json đang chạy. Caller vẫn phải cô lập mạng theo recipe.

<a id="four-session-diagnostic"></a>
## Service bốn session chẩn đoán

[Scope/native/giới hạn CP200](../docs/PROTOCOL.md#four-session-native). Dùng profile47fixture đã export/pin
của hai-client basic; service4 kiểm branch zero-profile/Start121 trước listen và tạo body4riêng trong runtime.
Đây là entrypoint riêng, không đổi launcher/bundle hai người. Phải chạy trong mạng offline cô lập theo recipe.

```bash
python3 -m server.four_player_service --profile /absolute/local/profile.json --out local/run4 \
  --bind-ip 10.77.0.1 --channel-ip 10.77.0.1 --channel-port 3858 --port 3858 \
  --source-ip A=10.77.0.2 --source-ip B=10.77.0.3 --source-ip C=10.77.0.4 --source-ip D=10.77.0.5
```

Bootstrap cần listen đúng các port trong recipe Wine; ví dụ trên chỉ minh họa channel. Identity/sourceIP
là operatorpolicy, bốn transport/role/lifetime; không accountauth/sameNATIP/retry/resume. Mốc này chỉ
room/load/input/peer, reject gameplay/bom/event/result/reset; chưa server4trận hoàn chỉnh hoặc bundle4portable.
Tests: `python3 -m unittest server.test_four_player_service`; nguồn chuẩn thứ tự tiếp ở PLAN.

CP201 thêm opt-in `--bomb-profile /absolute/local/four-bomb.json` cho entrypoint4, cấu hình rõ:

```json
{"mode":"four_current_baseline","capacities":{"A":1,"B":2,"C":1,"D":1},"powers":{"A":1,"B":1,"C":1,"D":1},"fuse_seconds":6}
```

[Baseline/giới hạn](../docs/PROTOCOL.md#four-owner-bombs) chỉ profile/char đã đo; capacity/fuse là policy emulator.
Không khai giữ CP200reject118; bật có159/158/reservation/fence, vẫn chưa life/result/reset2v2/fullmatch.
Tests `python3 -m unittest server.test_four_bomb_policy`; không lấy generatedwire làm nativeoracle.

CP202 entrypoint4 hỗ trợ `--battle-outcome` cùng `--bomb-profile`: bounded17/19/21→3peer, report-derived2v2/
rank4/180/reset/Ready2Start2. [Contract/native2cycles/giới hạn](../docs/PROTOCOL.md#four-two-team-cycles).
Không là item/terrain/rescue/needle4playerintegration hoặc hardened/publicservice; rank/economy/settle làdesign.
`python3 -m unittest server.test_four_battle_policy` kiểm policy; native evidence riêng trong CP202.

<a id="four-player-bundle"></a>
### Gói bốn client KR — CP203

```bash
python3 tools/export_probe_profile.py --source-root local/research/codex --out local/four-player-bundle --preset four-client-battle --bundle
python3 -I local/four-player-bundle/run.py --verify-only
```

Preset này dùng đúng47alias đãpin/43fixturefile của profile4 và policy capacity1/2/1/1,power1,fuse6.
Gói64file chứaentry4/profile/policy/source, có thể chuyển thư mục; không chứa client/assets/DB và không tự dựng networkisolation.
Launcher gắn sẵn `--battle-outcome`/profile/bombprofile; không truyền overrideprofile/policy. Với topology đã kiểm:

```bash
kr_four_ports=()
for kr_four_port in {3808..3998..10}; do kr_four_ports+=(--port "$kr_four_port"); done
python3 -I /absolute/four-player-bundle/run.py --out /absolute/run-four --bind-ip 10.77.0.1 --channel-ip 10.77.0.1 --channel-port 3858 --source-ip A=10.77.0.2 --source-ip B=10.77.0.3 --source-ip C=10.77.0.4 --source-ip D=10.77.0.5 "${kr_four_ports[@]}"
```

Chạy cùng [recipe cô lập/source/raw](../docs/evidence/peer-endpoints/README.md#cp203), copyWine4b và riêngprefix/IP/netns;
sourceclient giữ nguyên. [Scope/authority/knownstartupfailure](../docs/PROTOCOL.md#four-portable-guards) phân biệt native2v2
đã kiểm với unknownhardening/Internet/K7 và4item/terrain/rescue/needle chưa tíchhợp. Gói hai người hiện hữu giữ entrypoint cũ.

CP204 thêm [recipient-safe transport và giới hạn](../docs/PROTOCOL.md#four-transport-bounds): bound65536wire,
pertransport lifetime1MiBinput/4096chunks,2MiBattemptedoutput/4096sendcalls, chunk/framebucketburst128+32/s;
Battle reset không xóa budgets. Chạmquota đóngconnection/fence room; cần processlifetime mới để cấp lại quota.
CP205 thêm opt-in `--abort-room` vào lệnh bundle bốn người: cần preset đã pin và `--battle-outcome`
(bundle đã cố định). [Contract/giới hạn](../docs/PROTOCOL.md#four-member-abort-room): guest/hostEOF khi loaded
→unranked180/setup585/107/110/ba survivorRoom; khôngrankgiả/late158/Start3người/reconnect.
CP206 gói mới67pins và tự ghi `run-metadata.json`, `service-state.json`, `health.json` vào mỗi `--out` mới.
Healthreader phải match snapshotsequence/hash của state; runmetadata đã tồn tại sẽ từ chối để giữ lịch sử.
Thêm `--identity-store /absolute/identities.json` nếu muốn pin khai báo4identity giữa processrestart; parent phải có sẵn,
existingbytes phải đúng canonicalpin, không accountauth/resume. [Contract/cách đọc/giới hạn](../docs/PROTOCOL.md#four-health-identity).
Tại CP204 gói65pins; launcher từ chối serverPython còn hiện diện mà không cópin trướcimportserver. Khôngsignature,
khôngqueuednonblocking/backpressurebenchmark.335server/16bundle/214nonreader+247guard vànative4positive cónguồn;
Tại CP204 EOF/sendfailure chưa đưa ba nativepeer vềphòng; CP205 opt-in đã kiểm guest/hostEOF,
native sendfailure/quota còn theo gap/PLAN. AUD060qua204/kế208.


<a id="eight-session-preview"></a>
### Gói tám session preview

`python3 tools/export_probe_profile.py --preset eight-client-start --source-root local/research/codex --out local/eight-bundle --bundle`

Chạy `python3 -I local/eight-bundle/run.py --verify-only` trước. Entrypoint tám người nhận
`--source-ip A=IPv4` đến `H=IPv4` riêng, cùng `--out`, `--bind-ip`, `--port` như gói bốn người.
Chỉ phòng/load/peer; không có flag bom/result8. [Contract/bằng chứng/giới hạn](../docs/PROTOCOL.md#eight-session-preview).


EightPlayerService có opt-in `--bomb-profile` cho **đúng** measured`eight_current_baseline` của
[CP209](../docs/PROTOCOL.md#eight-mesh-owner-bombs); default preset/bundle CP208 không tự bật policy đó.
Profile khác bị từ chối; không có flag outcome/reset8. Gói đã xuất không tự cập nhật theo source hiện tại.
