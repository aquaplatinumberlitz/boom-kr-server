# Boom KR Server

Server giả lập cho **the lastest client Crazy Arcade KR build00967 before end of service**, dựa trên kiến thức từ client
VN VBNB_116. 
Chú ý:
1- dự án không cung cấp client game
2- dự án chứa lượng lớn text và kiến thức RE được viết bằng agents, ko phải con người, nếu đọc không hiểu thì hãy hỏi AI giải thích. 
<img width="1024" height="1230" alt="fixed-contact" src="https://github.com/user-attachments/assets/739b9237-b44a-47f0-87a3-97ccd3c9f669" />

**Thư mục làm việc hiện tại trên host:** `/home/admin/boom/repos/boom-kr-server`.
Docs, tools, code và evidence mới đều ở đây. Không cần mở worktree của repo Rust cũ để làm project này.

```text
boom-kr-server/
  AGENTS.md                 quy tắc của project này
  docs/
    PLAN.md                 lộ trình K0–K7, task và gate
    features.json           chỉ mục cơ chế/ID/dependency/trạng thái có nguồn
    COVERAGE.md             bảng/cây/thẻ tra cứu sinh từ catalog
    PROTOCOL.md             contract KR, ma trận reuse VN→KR và authority
    RESEARCH.md             câu hỏi mở, owner và điều kiện đóng
    REFERENCES.md           nguồn RE cũ, provenance và bản đồ migration
    evidence/               metadata/hash; không chứa binary hoặc decompile
  tools/                    harness, probe và kiểm tra artifact/docs
  server/                   code server tương thích KR
  local/                    raw/copy client/DB/capture; git-ignored
```

## Tra nhanh và trạng thái hiện tại

[Coverage và cây chức năng](docs/COVERAGE.md) trả lời: cơ chế nào hiện diện, mức RE/implementation/
runtime nào có nguồn, còn thiếu gì và phép kiểm nào giải quyết. Mỗi ID dẫn thẳng contract/giới hạn,
evidence/hash và câu hỏi; [PLAN](docs/PLAN.md#current-order) giữ duy nhất thứ tự công việc.
Đây là scope đường bootstrap → login → sảnh → phòng → Battle → kết quả/reset, không kiểm kê toàn client.
Tra checkpoint: `python3 tools/checkpoints.py latest`, `show CPxxx`, `evidence CPxxx`
(summary/page hoặc `--verify --require-local`); [kho v2/migration](docs/evidence/checkpoints/README.md#kho-checkpoint-v2-và-cách-đọc-chọn-lọc)
giữ record cũ nguyên object. Không cần đọc manifest lớn.

Hai copy KR Wine đã chạy login/sảnh/phòng/Battle và các chu kỳ kết quả/reset có bằng chứng;
**K5 basic local đạt CP188; K6a local đạt CP189. K6b LANgiảlập đạt CP191; Internet còn mở**. [Movement/terrain hai vòng](docs/PROTOCOL.md#current-movement-cycles),
[kim/reset](docs/PROTOCOL.md#needle-round-reset), [cứu hai chiều và hai kết quả](docs/PROTOCOL.md#rescue-two-results),
[deep chain/reset](docs/PROTOCOL.md#recursive-chain-cycles) giữ phạm vi và giới hạn tại nguồn chuẩn.

Các mốc dưới ghi trạng thái **tại checkpoint nêu tên**; next-step hiện hành chỉ ở PLAN.

**K0 và bước tĩnh K2a đạt CP178; K1 đạt riêng recipe Wine R2 offline tại CP179**, đã merge
[PR250](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/250)/`2f2b87f` và
[PR251](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/251)/`2e6dae4`.
[Phạm vi nghiệm thu](docs/PLAN.md#k1-acceptance) không bao gồm bootstrap Windows nguyên bản hoặc ba coldstart e9.

**CP181: K2 đạt cho profile codec/hello offline đã đo**, sau đối chiếu thủ công từng tiêu chí
giữ nguyên. Native retry trong cùng process: result1 hợp lệ → command1002 → login socket mới →
auth137/137/errors0 → channel socket mới; hello/mode/seq reset được đo trên từng connection.
[Nghiệm thu/giới hạn](docs/PLAN.md#k2-acceptance), [contract](docs/PROTOCOL.md#login-retry-codec),
[run/hash](docs/evidence/live-codec-stream/README.md#cp181). CP180 split/coalesced/CRCcontrol đã merge
[PR252](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/252) tại`fe242cc`.

**K6c Internet/K7 vẫn mở; K6b LANgiảlập đạtCP191 theoowner. K3 local CP186, K4 local CP187, K5 basic CP188 và K6a local CP189 đã đạt riêng scope**,
[nghiệm thu/giới hạn](docs/PLAN.md#k3-acceptance). Fullgrammar/unknownheader/prefix/mode và
accountauthority chưa được khôi phục đầy đủ; K2 không nghiệm thu các phần đó. Thứ tự tiếp ở
[PLAN](docs/PLAN.md#current-order): nhánh mạng/admission đã có CP192–198; bốn session vào Battle tại CP200, tiếp gameplay2v2 theo PLAN, giữ Internet gate mở.
CP181 đã merge [PR253](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/253) tại `5a154e5`.
**CP184: identity291 đã tích hợp opt-in vào service.** Hai run A đã hiện tên ở sảnh rồi tạo phòng,
nhận roster/map; native framer nhận sequence43…73 liên tục, next76. [Contract/giới hạn](docs/PROTOCOL.md#own-profile-integration),
[run/hash](docs/evidence/lobby-identity/README.md#cp184). Đây là một client thật với guest tổng hợp,
không thay bằng chứng hai client hoặc nghiệm thu trận. CP183 đã merge [PR255](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/255)
tại`5c1f3fb`; CP184 đã merge [PR256](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/256) tại`1f8018a`. Tại CP184 K3 còn mở; [CP186 đã đối chiếu nghiệm thu](docs/PLAN.md#k3-acceptance) cho profile hẹp.

**CP185: cache dynamic-map đã có đường request528→reply238 trong service opt-in.**
Đối chứng thiếu file, native ghi lại và warm không yêu cầu528 có [evidence/giới hạn](docs/PROTOCOL.md#lobby-cache-route).
Chưa cold toàn client; CP186 bổ sung cold route kênh và version/identity/negative cho nghiệm thu K3 hẹp. [Wiki KR do chủ dự án tin cậy](docs/REFERENCES.md#owner-trusted-namu-wiki)
được dùng cho luật/stat/char/map/mode trước khi mở RE gameplay; [bảng dùng lại](docs/PROTOCOL.md#trusted-wiki-reuse) giữ phần phải đối chiếu00967.

**CP186: K3 đạt trong scope một A/một kênh normal offline.** Bảy run và tám ca wire kiểm cache route,
version lỗi, native identity và guard request6; [contract/giới hạn](docs/PROTOCOL.md#k3-version-identity-controls).
Không fullaccountauth, không caption kênh có tên hoặc hai client291 đã nghiệm thu.
**CP187: K4 đạt trong scope phòng/load/start hai actualAB.** Đã đo identity291 cả hai, hostBack/
rejoin, một kết quả/reset và Start2; [nghiệm thu/giới hạn](docs/PLAN.md#k4-acceptance).
Copy riêng sửa importABI đã qua lỗi kết quả; [control/recipe](docs/PROTOCOL.md#own-profile-result-compare).
Run CP187 vẫn là lịch sử partial ở input vòng2; [CP188](docs/PROTOCOL.md#current-basic-cycles-291) đã có hai outcome mới, không sửa kết quả run cũ.
**CP188: K5 basic local đạt** theo [tiêu chí giữ nguyên](docs/PLAN.md#k5-acceptance).
CurrentAB291 có hai normal outcome/reset, hai chiều peer và hai owner bom; run riêng đã đo pending
B-host EOF/A cleanup/fence. [Contract/giới hạn](docs/PROTOCOL.md#current-basic-cycles-291),
[recipe/hash và partial giữ nguyên](docs/evidence/peer-endpoints/README.md#cp188).
Tại CP188 còn currentbasiccontest; CP189 bổ sung bên dưới, không thay kết quả lịch sử.
CP187 đã merge [PR259](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/259) tại `e57e6283`.
**CP189: K6a basic local đạt** sau currentcontesteditem/two-normalcycles/reset/nativepeer/life
và đối chiếu CP188Bowner/EOF riêng; [scope/giới hạn](docs/PLAN.md#k6a-acceptance),
[recipe/raw/hash](docs/evidence/peer-endpoints/README.md#cp189). Hai claim thật trên production,
mộtwinner/effect/view; baseline stat riêng A5/B4. Không physicalLAN/Internet/bốn–tám actualclient.
CP188 đã merge [PR260](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/260) tại`2ea548bb`.
**CP190: service/47fixture đã đóng thành gói chuyển thư mục**, kiểm bằng Python-I/realTCP và hai KR
Wine copy trên service đã relocation. [Cách dùng](tools/README.md#portable-bundle),
[nguồn/giới hạn](docs/PROTOCOL.md#portable-probe-bundle). Tại CP190 còn yêu cầu2máyvậtlý;
CP191owner thay bằng LANgiảlập, startup vẫn tuần tự.
CP189 đã merge [PR261](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/261) tại`1559ad57`.
**CP191: K6b LANgiảlập đạt theo chỉ đạo chủ dự án**, dùng hai actualKR trong netns/IP/NIC riêng;
[decision/giới hạn](docs/PLAN.md#k6b-acceptance). Không còn blocker phải có hai máy vật lý.
BpostwinRollerreset4→5→4 đã xác nhận qua hai normaloutcomes/Start2/3 trênportablebundle;
[nguồn](docs/PROTOCOL.md#emulated-lan-acceptance). K6cInternet/K7 vẫn mở.
CP190 đã merge [PR262](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/262) tại`b7e83f46`.
**CP192: actualKR Wine đã qua hai subnet/router** với hai normalcycles/reset/Start3 và endpoint/nativeUDP
có capture; [contract/giới hạn](docs/PROTOCOL.md#routed-Wine-network). Chưa NAT/relay/Internetgate.
[Owner chấp nhận Wine thay Windows](docs/evidence/owner-directives/wine-accepted.json), không cần máy Windows.
CP191 đã merge [PR263](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/263) tại`56a08dd6`.
AUD057/058 periodic qua192/196 là lịch sử; AUD059 qua200, audit kế **CP204**. [Review waived](docs/evidence/checkpoints/review-waiver.json).

**CP193: đã đo đối chứng NAT trên hai KR Wine thật.** Endpoint riêng làm consumer trả0; khai báo tuple
đã ánh xạ + DNAT khôi phục handler, key và movement hai chiều. [Contract/giới hạn](docs/PROTOCOL.md#nat-endpoint-control),
[run/hash](docs/evidence/peer-endpoints/README.md#cp193). Tại CP193 chưa hai chu kỳ NAT; [CP194](docs/PROTOCOL.md#nat-two-cycles) bổ sung, K6c Internet vẫn mở.
CP192 đã merge [PR264](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/264) tại`1a6daad0c17d718bc86574162bf2946bca1ed800`.
CP193 đã merge [PR265](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/265) tại`8acf57c81a727892c04e735c28a20a5319d70fd9`; review waived; tại CP193 audit kếCP196, hiện theo counter manifest.

**CP194: hai KR Wine đã hoàn thành hai vòng Battle bình thường/reset/Ready2/Start3 qua mapping NAT tĩnh.**
[Phạm vi và giới hạn](docs/PROTOCOL.md#nat-two-cycles), [raw/hash](docs/evidence/peer-endpoints/README.md#cp194).
CP194 đã merge [PR266](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/266) tại`9f4ebf53d29d22092b0971c7fb34e07dbcb2cce1`; chưa nghiệm thu Internet/K6c.

**CP195: hai vòng KR qua NAT vẫn hoàn tất/reset/Start3 với UDP delay40±5ms/loss5% đã kiểm counter.**
[Phạm vi và giới hạn](docs/PROTOCOL.md#udp-netem-cycles), [raw/hash](docs/evidence/peer-endpoints/README.md#cp195).
CP195 đã merge [PR267](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/267) tại`fe329b0b4da66aa4138c0e1a242da6ab3033d4a2`; TCP baseline, chưa Internet gate.

**CP196: pending B-host EOF qua NAT/UDP impairment đã kiểm A về phòng/ID clear/chief và fence bom chờ.**
[Contract/giới hạn](docs/PROTOCOL.md#nat-netem-host-eof), [run/hash](docs/evidence/peer-endpoints/README.md#cp196).
Run r1 partial do Python probe assertion giữ nguyên; r2 bounded probe pass. AUD058 qua196; audit kế **CP200**.
CP196 đã merge [PR268](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/268) tại`e842347b7cac073c6822b2251096747710abc170`; K6c Internet/K7 còn mở.

**CP197: incoming admission A/B theo IP nguồn riêng đã có opt-in**, realTCP xen kẽ0wrong/8 và native
startup tuần tự vẫn qua hai trận/reset/Start3 trên bundle mới. [Contract/giới hạn](docs/PROTOCOL.md#operator-incoming-binding),
[recipe/hash](docs/evidence/peer-endpoints/README.md#cp197). CP197 đã merge [PR269](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/269) tại`4c3f6ec246b3e0fcec017592aa6bd34f832ba6b9`. Tại CP197 native startup xen kẽ còn thiếu; [CP198](docs/PROTOCOL.md#native-interleaved-startup) đã bổ sung. Chưa bốn client/K7.

**CP198: hai KR native startup xen kẽ A1,A2,B1,B2,A3,A4,B3,B4**, đúng role/stage/identity, đã hoàn tất
hai normal cycles/reset/Start3 trên gói admission hiện hành. [Scope/giới hạn](docs/PROTOCOL.md#native-interleaved-startup),
[raw/hash](docs/evidence/peer-endpoints/README.md#cp198). K6c Internet/K7/bốn client vẫn mở; audit kế CP200.

**CP199: copy KR hiện hành đã đọc Start121/peer153 bốn slot và khởi tạo bốn actor** ở cả hai view.
[Phạm vi](docs/PROTOCOL.md#four-slot-native-consumers), [raw/hash](docs/evidence/peer-endpoints/README.md#cp199):
hai actual client + hai actor tổng hợp, A/B peer còn hoạt động. Chưa bốn actual client, kết quả/reset2v2 hoặc K7.
Blocker chuyển sang service bốn session/identity/roster/Ready/barrier/mesh; thứ tự chỉ giữ ở PLAN.

**CP200: bốn KR Wine thật đã login/sảnh/phòng/load vào Battle**, đúng bốn identity/ownslot; run ít probe
quan sát movement từ từng client ở ba client còn lại. [Phạm vi/controls](docs/PROTOCOL.md#four-session-native),
[recipe/hash](docs/evidence/peer-endpoints/README.md#cp200). Service bốn session riêng; chưa bom/kết quả/reset2v2,
Start2 hoặc đóng K7. CP199 đã merge [PR271](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/271)/`ae329187`.
AUD059 quaCP200; audit định kỳ kế **CP204**. Run lỗi/header/probe/metadata giữ raw riêng, không sửa thành pass.

**CP201: bốn owner A/B/C/D đã đặt bom và nhận confirm/nổ ở cả bốn KR Wine view.**
[Contract/baseline/giới hạn](docs/PROTOCOL.md#four-owner-bombs), [raw/hash](docs/evidence/peer-endpoints/README.md#cp201).
Capacity đã đo1/2/1/1, power1; native15939/39/entity và15813/13/errors0 đủ bốn bom/bốn view. Chưa kết quả/reset2v2.
CP200 đã merge [PR272](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/272)/`cd958c95`; audit kế CP204.

Đích đầu: hai client thật chơi hai trận liên tiếp; mở bốn client và hardening sau đó.

## Bắt đầu

Đọc [AGENTS](AGENTS.md), [PLAN](docs/PLAN.md), rồi [PROTOCOL](docs/PROTOCOL.md) và
[RESEARCH](docs/RESEARCH.md) theo task.

```bash
python3 tools/feature_catalog.py
python3 tools/check_docs.py
python3 tools/check_checkpoints.py --require-local --require-history
python3 tools/artifacts.py verify
```

`artifacts.py verify` dùng manifest **host-local** trong `docs/evidence/artifacts.json`; client/DB
không nằm trong Git nên clone sạch trên máy khác chưa thể chạy lệnh này. Khi chuyển máy, dùng
`--manifest /path/to/local-artifacts.json` với đường dẫn nguồn và hash đã xác nhận tại máy đó.

Client nguồn giữ ở `/home/admin/boom/clients/kr-00967`; không sửa hoặc commit. Artifact và database
đã unpack ở repo RE cũ chỉ là đầu vào tham chiếu; mỗi phép thử dùng bản copy riêng trong `local/`.
Nguồn/commit và giới hạn bằng chứng: [REFERENCES](docs/REFERENCES.md).
