# Item, stat, pickup và kim

<a id="g1-scoped-drops"></a>
### Bảng rơi server theo phạm vi G1

Theo [quyết định chủ dự án](../evidence/owner-directives/g1-item-scope-cut.json), Rust giữ Bomb/Fire/Roller,
MaxPower, kim/cứu, Shoe và Punch. [Dữ liệu server](../../rust/data/g1-item-drops.json) giữ mọi row của sáu map,
kể cả row trùng, với trọng số 0 cho mọi tên ngoài tập giữ. Đường fixture cũ dùng bảng đã pin khi load. Khi cài từ pack, Rust kiểm ID của bảy item ở ItemAttr và áp
cùng chính sách lên hàng rơi đọc từ map; bộ đối chứng cũ giữ nguyên. [Metadata nguồn/hash](../evidence/rust-service/g1-item-scope-data.json)
ghi số row và phạm vi kiểm. Trọng số không là phần trăm hoặc bằng chứng RNG server gốc.

Các item cố định và hidden list2 từ fixture được kiểm riêng trước bind: chỉ chấp nhận kind 0–5 hoặc 145,
sau đó kiểm policy MaxPower/Shoe/Punch/kim. Bảng rơi không tự cấp quyền bật hành động chưa nghiệm thu,
không sinh lại item của nhóm bị tắt hoặc mở thêm nhóm item ngoài G1.

**F1-a (design):** catalog thử riêng chỉ dùng Bomb, Fire, Roller và MaxPower. List1 của 582 lấy các
placement có giá trị 100 ở ô trống hợp lệ, ngoài spawn; placement khác vẫn giữ trong profile nguồn. List2 có tối
đa một drop ẩn mỗi map, chọn ô thùng và loại item bằng hash payload map cùng trọng số dương của nhóm hỗ trợ.
Kết quả lặp lại mỗi ván. Policy có mục đích đối chứng terrain/pickup, chưa phục hồi RNG hoặc mật độ rơi gốc;
không bật item còn thiếu. [Bằng chứng native](../evidence/rust-service/f1-pack-map-adapters.json) giữ từng ca
placement và drop đã quan sát. Đường cài G1 giữ floor hiện hành.

[Mục lục](../PROTOCOL.md) · [Câu hỏi đang mở](../RESEARCH.md) · [Thứ tự việc](../PLAN.md#dashboard).

<a id="item-families"></a>
### Nhóm item và kiến thức tái sử dụng — CP045

Đối chiếu theo câu hỏi Battle cơ bản; bảng dưới không là kiểm kê mọi cơ chế trong client.
Nguồn/hash/locator và bốn câu hỏi tái sử dụng tại [evidence](../evidence/item-families/README.md).
ItemAttr KR335/VN269 đã parse từ CP037:269 tên chung,267 hàng giống group/field160/five-tail/weight;
hai hàng khác là KR247/VN213 `ItemCpGold`, KR248/VN214 `ItemCpPocket`. Cùng field không chứng minh
cùng hiệu ứng. `boom-re-l10n/decomp/item_kinds.json` là cùng bảng VN269, ba số−1 khác biểu diễn
unsignedFFFFFFFF; không coi là ba cơ chế mới hoặc một nguồn runtime độc lập.

`boom-rs` R169–185 bổ sung **VN static/runtime confirmed** cho Roller: selector3, tăng1/kẹp max,
base/max/current, movement amount và đường cong đo trên actor cụ thể. Dùng lại kiến thức và cách kiểm;
offset/ID/consumer/wire KR vẫn phải đối chiếu. `oc_e-game.md:454` ghi unknown cũ; dòng463 đã có
Bomb/Fire runtime, evidence R169–185 mới hơn đã có Roller. Không đọc unknown lịch sử thành hiện trạng.
`archive/KNOWLEDGE.md` và bundle là bản đồ nguồn; câu tóm tắt33px/s/cấp hoặc fuse2,5s không thay
giới hạn capture gốc. `boom-rs` wire/authority/scope V1 là design khác project này.

| Nhóm / ví dụ KR↔VN | Đã biết từ VN | KR static/emulation | KR runtime / phần còn thiếu cho demo dùng nhóm đó |
|---|---|---|---|
| Stat tích lũy, group1: Bomb0/Fire1/Roller2 |Bomb/Fire effect và nhiều bom; R169–185 Roller+1 logical point/kẹp max/reset/consumer theo fixture VN |1192700/group1 gọi1191840 cho **bốn** selector đầu; fifth-tail không phải selector thứ5. Effect1→vf4,2→vf8,3→vf14; CP031 increment/clamp; Roller B48350 current iface+5C/max+50 |Bomb/Fire đã [đo stat/peer](#item-effects), [reset hẹp](#hidden-items). CP045 Roller5→6/max7 cả hai view, đầu vòng2 về5 rồi nhặt lên6; không chuyển rate VN thành px/s KR. Không tiêu hao kiểu Ctrl; item nền bị xóa sau165. |
| Ability/debuff cùng group1: Shoe3/Punch4/MaxPower5, Hollow6, Skeleton7/8, MoonWalk9, Superman10, GreenDevil11 |Tên/selector và render đã có; tránh suy Shoe là tăng speed, Roller mới là nhánh tăng point đã đo |Selector4/5/6 dùng các virtual khác ở1191840; không RE lại124/165 cho từng bản ghi |CP045 MaxPower1→7/max7 và118/159power7, reset đầu vòng2 về1. Ném/đá, immunity, thời hạn/debuff/mode cần consumer/case riêng trước dùng. Tên/icon hoặc số0 trong table không chứng minh vô hạn. |
| Mount/biến hình group2: Airship40↔34/Owl41↔35/Turtle42↔36… |Table và tên/variant có sẵn |Group2 là nhánh khác11928A2; không áp increment group1 |Chưa runtime KR trong đợt này; exclude khỏi fixture Battle tối thiểu theo policy thử, không gọi client không hỗ trợ. |
| Dùng chủ động group23: Needle145↔117 và12 hàng KR cùng group |Kim Ctrl sau trap, dùng cạn; R169–185 không là bằng chứng kim |1190C20 amount>0 rồi nhánhgroup23/vf1C; variant cùng group là ứng viên tái dùng đường |[Kim145](life-results.md#needle-selfescape)/[lần hai](life-results.md#needle-repeat): pickup124/165→Ctrl×1→actual127/1 rồi0(kind145), peer170/1 đưa56→26; cạn không thoát lần hai. Inventory validator, multi-charge/variant/round-reset toàn túi chưa đo. |
| Chủ động khác: Dart143↔115(g21), Oxygen144↔116(g22), Potion147↔119(g25), Spring148↔120(g26), Drill149↔121(g27), RemoteBomb150↔122(g28) |Table và đường Ctrl/nhận xét cũ; không runtime từng loại |Group/field160 chỉ phân loại dữ liệu; không gọi field160 luôn là số lần dùng |Điều kiện, effect, tiêu hao/report0/confirm, peer và reset chưa được kiểm theo từng nhóm; dùng Needle làm phương pháp, không copy kết quả. |
| Có thời hạn/variant: Shield151↔123(g29,raw2000/1000), Sensor152↔124(g30,raw30000/5000), SJNeedle161↔133(g46) |Shield được ghi2000ms là locator/hypothesis; SJ/needle variants khác group |Raw tail giống không đủ xác nhận đơn vị, duration hoặc cùng nhánh145; cần consumer timer/group |Chưa runtime KR duration/expiration/consume/peer/reset; không gán max/giây từ tên/field. |
| Mode/event/reward: HPNeedle17(g1/effect27), GoldFlag19(effect99), Lucci67↔50(g99), HCHint300↔266(g1000), RPG… |Catalog cũ cho phép tránh nhầm HPNeedle17 với Needle145 |Cùng tên “Needle” không cùng group/effect; R153 loader/group table khác biệt với effectconsumer |Không đưa vào ca Battle cơ bản hiện hành; mode guard/serverreward/variants còn unknown, không RE mọi row trước K6a. |

Nhóm pickup dùng đường [582/124/165](#item-pickup) đã đo; đối chiếu KR indexu32, không dùng VN indexu16.
Confirm broadcast có CRC/cursor đúng chưa đủ chứng minh effect; đo securefield/native consumer/peer và
đầu vòng kế. Placement/drop/first-arrival/fuse của stub là `emu_choice/design`, weight không là phần trăm.
Nguồn R150 cũ gọi E9E660 là item path đã bị [CP030 sửa](#item-pickup): đó là bom kind0;
R153 loader/table vẫn dùng được, không lặp tìm pickup/stat từ nhánh nổ.

**Delta KR runtime CP045, hai run riêng `roller-rounds`/`maxpower-rounds`:** mỗi run hai client thật,
hai Start/clock trên cùng8connections/IDs1,2/slots0,1/session1. Mỗi vòng một item nền index17/hàng4/
cột3, A Down tới actual124, ItemClaims/current165 phát cả hai, không chèn gói cũ hoặc sửa state.
Sau nhặt A Space thật, bom nổ→A17/19→170 peerstate1 trước175/180/phòng hai vòng. Placement,
catalog/fuse6s/rank/first-arrival là policy emulator, không luật server gốc.

- Roller2: actual165full12 →1191840 effect3/0/0/0, receiver actor+58C/vt17760BC →B48350.
  Secure **actor+5E8/current5→6**, **+5DC/max7→7** trên cả A/B; capacity/Fire/max và stat slot1
  được chụp giữ nguyên. Native B48350 dùng636620 tăng1 rồi so current với max, vượt thì54B610 chép max;
  năm ca emulation giữ machinecode tăng/kẹp, chỉ model local gateAD5BA0=false. Runtime tại max chưa đo.
  Cặp field/consumer tương ứng VN R173 nhưng không copy offset658/64C hay định lượng px/s từ VN.
- MaxPower5: effect6/0/0/0 →vfC/B472A0. Secure **actor+4BC/power1→7** và **+524/max7→7**
  cả hai; capacity/movement được chụp giữ nguyên. Actual118.field10=7,159.field8=7 ở hai vòng;
  không suy capacity/speed cũng lên max hoặc mọi feature branch từ tên MaxPower.
- Clock vòng2 reset current Roller về5 và power về1 trước pickup mới; sau nhặt lặp cùng delta.
  Hai run có hai175outer302/inner292 và180full3 mỗi client. Đây là reset của field đã đo, không
  toàn túi/entity/UDP hoặc kiểm mọi item. Không có input Ctrl/127sub0 trong hai ca này; chưa đo TTL/
  expiration trong cùng vòng. Không gán “vĩnh viễn” từ việc giữ qua các mốc hoặc table tail0.

<a id="f1b-next-items"></a>
### F1-b: Đối chiếu Lucci, Dart, Scanner, Bond và Trap

[Kiểm kê map](../CONTENT.md#item-được-map-dùng) cho biết item nào xuất hiện trong bảng rơi hoặc được đặt sẵn.
[CP045](../evidence/item-families/README.md), `boomkr-re/docs/kr-data.md` §6 và §12, và `boom-re/docs/items.md`
là nguồn đối chiếu. Parse lại hai bản trích `boomkr-re/data/kr/{KR,VN}_ItemAttr.bin` bằng
`tools/content_census.py::parse_item_attr`: cả 11 dòng dưới đây khớp tên, sprite, group, trọng số, `+160`
và tail, nhưng khác kind. `monster_drop.json` trong cùng thư mục chỉ dùng năm tên Lucci này từ Bronze tới
PocketSilver trong bảng thưởng monster; bảng đó không mô tả pickup Battle. Các JSON KR khác không cho thêm
nhánh consumer cho năm nhóm. Trọng số là giá trị thô của ItemAttr, chưa có luật rơi Battle gốc.

- `kr_static`, KR 00967: SHA-256 ItemAttr `a9b55540c85c21de108e8720b4148b2030a231d907f3709dcb06f7ef728fcdba`.
  IDA đọc bản sao riêng của DB ảnh giải nén `kr-full.exe`, SHA-256 input
  `cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b`. Địa chỉ dưới đây là VA của ảnh;
  `RVA = VA − 0x400000`. DB trước khi mở có SHA-256 `20f4b3bf608a46236b5a9cfc0843a7aecc2471810a5112185b098ed7d07ed462`.
- `vn_static`, VBNB_116: SHA-256 ItemAttr `a98883ce35f3a509719db637f662f2ec90da64dd3c1faa0c9f7eb22c0de7dd1d`.
  `boom-re/docs/items.md` dẫn loader VN VA `0x89F3DE`; chỉ dùng để đối chiếu bảng, không áp VA hoặc kind VN lên KR.

Các hướng consumer trong bảng là `candidate`. Ở đường pickup đã dùng làm mỏ neo tại CP031, `1190A40` có
nhánh nhận item nếu `B81B40` trả khác 0, trước khi tới dispatch pickup `1192700`; chưa đo giá trị getter hoặc
inventory của từng kind tại pickup. Ở đường dùng, `B4D710` gọi `1190C20` tại `B4D999`, truyền interface
actor `+0x590`. Dispatcher kiểm kind hợp lệ, amount dương và gọi guard ảo theo group trước khi chọn nhánh.
RTTI cho `CGOBomber` gốc đặt interface này tại vtable `1744F30`; bảng ghi đích ở vtable đó, còn đích thực
của actor đang chạy hoặc lớp con vẫn cần kiểm native.

| Item, kind KR ↔ VN và group | ItemAttr thô (weight; `+160`; tail) | Nhánh KR và hướng consumer (`candidate`) | Phép đo native tối thiểu và phần `unknown` |
|---|---|---|---|
| LucciBronze, Silver, Gold, PocketBronze, PocketSilver, PocketGold: 67–72 ↔ 50–55; PocketRandom: 77 ↔ 60. Group 99. | 67–72: weight lần lượt 800, 1100, 250, 30, 30, 15; `+160=0`; tail `[n,0,0,0,1]` với `n=10,20,50,100,200,500` theo thứ tự. 77: `0; 0; [2000,-1,0,0,1]`. | Pickup `1192700` → `1192B40` → `B4A230`, có đường hiển thị theo tọa độ và đường gọi getter/setter `13E4930/13E4950` theo chỉ số actor có guard. Nhánh dùng `1190C20` trả 0 tại `1190D41` cho group 99. | Đặt lần lượt từng kind 67–72 và 77. Đọc đầu vào `B4A230`, chênh lệch trường được getter/setter truy cập và hiển thị trên hai client; lặp 77 ở vòng sạch để phân biệt giá trị cố định với biến thiên. Cách chọn giá trị 77, giới hạn, kết quả trận và quyền cộng tiền bền vững còn chưa rõ. |
| ItemDart: 143 ↔ 115. Group 21. | `0; 3; [0,0,0,0,0]`. | Nhánh dùng `1190F62` gọi slot `+0x14` (đích gốc `B4A780`) không truyền tail; nếu slot trả khác 0, gọi helper `D1CD10` với ID `0xA2`. | Sau pickup, bấm Ctrl hướng về B và chạy đối chứng quay ra hướng trống. Ghi guard, số lượng trước/sau, đích ảo, vật thể hoặc thay đổi trên B và frame thực tế nếu có. Điều kiện ngắm/trúng, số lần tiêu hao và report/peer còn chưa rõ. |
| ItemScanner: 146 ↔ 118. Group 24. | `0; 1; [0,0,0,0,0]`. | Nhánh dùng `1191003` gọi slot `+0x20` (đích gốc `B4B350`) không truyền kind hoặc tail. | Dùng Ctrl cạnh một thùng có item ẩn, so với thùng cùng loại không có item. Ghi nhánh, số lượng, thay đổi hiển thị trước/sau trên cả hai client và frame thực tế nếu có. Đối tượng được quét, phạm vi, thời hạn và mức đồng bộ còn chưa rõ. |
| ItemBond: 224 ↔ 191. Group 11. | `0; 3; [1,0,0,1,0]`. | Nhánh dùng `1190D4F` gọi slot `+0x08` (đích gốc `B4C1F0`) với `(1,false,false,true,0)`: tail đầu là selector thô, ba tail kế đổi thành bool, đối số cuối là hằng 0. | Dùng Ctrl tại một ô trống, theo dõi ô và cho B đi qua; lấy lượt B qua cùng ô không dùng item làm đối chứng. Chụp đối số, số lượng, thay đổi trên ô/actor và frame nếu có. Nghĩa của cờ, điều kiện hợp lệ, vòng đời và hiệu ứng lên B còn chưa rõ. |
| ItemTrap: 225 ↔ 192. Group 11. | `0; 2; [2,1,1,0,0]`. | Cùng nhánh `1190D4F` và slot `+0x08`, nhưng đối số là `(2,true,true,false,0)`. Dùng chung slot không chứng minh dùng chung hiệu ứng. | Dùng Ctrl với cùng ô và tuyến đi của B như Bond, lần này với kind 225. So thời điểm đặt, B bước vào ô và rời ô; ghi số lượng, trạng thái trên hai client và frame nếu có. Điều kiện kích hoạt, hiệu ứng, thời hạn và quyền áp dụng còn chưa rõ. |

**Native p93 — `kr_runtime`, xác nhận một phần.** Chạy hai lượt, mỗi lượt có hai bản copy client KR
00967 trong Wine `10.0-20260425 (CachyOS)`. SHA-256 của `CA.exe` chẩn đoán A/B là
`5fcb696d0c36c3b5e161c6a63bf03588f62aedf602eeea7b61941b07e9d40b09`, cùng bản CP394.
Fixture cục bộ (`emu_choice`) dùng binary Rust riêng từ `ba349d8` và floor `S2C582` đặt trên ô hợp lệ.
Nó mở các kind thử cho bước kiểm claim theo round key, slot, tọa độ, index và kind hiện có. Các report
ngoài tập life đã biết chỉ được quan sát. Không sửa pack exporter hay mở kind mới cho server chuẩn.

- Lượt 1 hai client vào phòng nhưng còn cùng đội. Sau Start, client hiện hộp thoại và helper nhập liệu dừng;
  không có `S2C582`, `C2S124` hoặc `S2C165`. Lượt 2 chọn nhân vật và đội riêng thành công, chạy Forest24
  rồi CAhero119. Có 14 claim hợp lệ (gồm hai pickup Fire đối chứng); tất cả 27 confirm nhìn thấy đều khớp
  claim. Người nhặt nhận confirm cho cả 14 claim. Peer nhận 13; lúc B nhặt Scanner, phiên A đã ngắt.
- **Lucci 67–72/77 — `confirmed` cho pickup.** A đi qua bảy ô Forest24, phát `124` và cả hai client nhận
  `165` cho cả bảy kind; ảnh cho thấy các vật phẩm rời sàn. Trong các mẫu trước pickup, sau hai đồng đầu,
  sau cả tuyến và đầu ván kế, tám giá trị secure tại context Lucci `[0x19D8A94]+0x11C+12*i` đều bằng 0
  trên A và B. Không bắt trực tiếp đối số hoặc lần gọi `B4A230/13E4950`; mẫu rời rạc này chưa xác định
  số Lucci cộng, giá trị ngẫu nhiên của 77, hoặc quyền cộng tiền. Kind 77 chỉ được nhặt một lần.
- **Dart 143 — `confirmed` cho pickup.** A phát `124`, hai client nhận `165`. HUD A hiện x03 và trường
  secure chọn item của actor (`+0x1C4C`) đọc được 143 trên hai góc nhìn. Sau hai input `VK_CONTROL`,
  HUD A vẫn x03; không thấy `127` hoặc `170` trong cửa sổ ngay sau input. Một lần quay ra khoảng trống;
  lần thử hướng về B không đặt hai actor cùng trục. Lượt này không kiểm chứng được input đã tới nhánh
  dùng Dart, va chạm hoặc hiệu ứng lên B.
- **Bond 224 và Trap 225 — `confirmed` cho pickup.** B phát `124` cho hai ô Forest24
  `(row,col)=(6,13)/(6,11)`, hai client nhận `165`. HUD B hiện Bond x03 rồi Trap x02.
  Các lần gửi `VK_CONTROL` sau pickup không đổi số trên HUD, không có `127` hoặc `170` trong
  cửa sổ input, và chưa quan sát được vật đặt. Tuyến di chuyển đã đi qua các ô pickup, chưa có ca
  B đi qua ô đã đặt và ca đối chứng cùng ô. Bên sử dụng, hiệu ứng lên người đi qua và vòng đời còn
  `unknown`. Đầu ván CAhero119, HUD Ctrl trống và hai actor đọc được item đang chọn là `-1`
  trên cả hai góc nhìn; đây là reset của item đang chọn qua hai map đã đo.
- **Scanner 146 — `confirmed` cho pickup và report tiêu hao.** A nhặt ở `(8,0)`, B ở `(10,1)`;
  HUD người nhặt từ x01 sang trống sau `Control_L`. Fixture đặt Fire ẩn ở `(9,2)`; A thử phía lối
  không có item ẩn liền kề, B thử từ lối phía dưới thùng có Fire. Trong khoảng 25 ms sau mỗi input,
  client phát `C2S127/sub0` dài 22 byte. Theo độ rộng `[1,1,2,2,4,4,1,2,1,4]`, A gửi
  `[0,0,0,359,8101,2,1,146,0,0]`, B gửi `[0,1,52,439,8145,2,1,146,0,0]`. Đối chứng
  `Control_L` khi HUD trống không phát sub0 trong cửa sổ đo. Guard life hiện chỉ nhận đuôi sub0
  của Kim kind 145, nên đóng lần lượt hai phiên Scanner. Không có `S2C170/sub0`; `170/sub19`
  tới B sau report A là thông báo loại A khỏi trận, không phải confirm Scanner. Ca so sánh hiển
  thị bị gián đoạn. Phạm vi, việc hiện item ẩn, đồng bộ và reset sau ván Scanner vẫn chưa xác nhận.

Biên nhận và cách tái lập: `local/runs/f1b-next-items-wine/{receipt,recipe}.json`. Wire channel kiểm
checksum và thứ tự claim/confirm nằm trong `run-2/wire.jsonl` dưới cùng thư mục; UDP thô và cửa sổ
input theo đồng hồ monotonic nằm cạnh đó. Không gán UDP chưa giải mã cho hành động Ctrl. Ảnh đối chiếu
chính trong `run-2/`: `lucci-first-A.png`, `lucci-dart-A.png`, `dart-empty-A.png`,
`bond-picked-r1-B.png`, `trap-ctrl-r1-B.png`, `r2-arena-A.png` và
`scanner-{no-hidden-before,no-hidden-after}-A.png`, `scanner-{hidden-before,hidden-after}-B.png`.
Blocker tiếp theo là cho fixture giữ phiên khi quan sát Scanner sub0, rồi đo lại Dart/Bond/Trap
với input `Control_L` đã quan sát được trên Scanner và với đối chứng vị trí đã ghim.

**Rust opt-in — `emu_choice/design`.** Với phòng người chơi được quản lý, đặt `fixed_room.item_policy`
thành `report_derived_f1b_extended` để mở pickup các kind trên. Mỗi lần nhặt hợp lệ cộng một lượt cho từng kind
Lucci và một charge Scanner, ba charge Dart hoặc Bond, hoặc hai charge Trap. Tồn kho theo thành viên
và vòng, được làm sạch khi kết thúc vòng và khởi tạo lại ở Start121. Scanner chỉ tiêu charge khi
actor hiện hành còn sống, không mắc bóng, đúng slot và tick đi sau pickup Scanner của vòng đó.
Tick là guard của emulator; sub0 chưa có khóa vòng đã xác nhận. Server không phát `170/sub0`,
cộng tiền Lucci hoặc áp hiệu ứng Scanner, Dart, Bond và Trap. Phần này mới có kiểm thử Rust.

<a id="f1d-vehicles"></a>
### F1-d: Đối chiếu phương tiện cưỡi và biến hình

[Kiểm kê nội dung](../CONTENT.md#item) và [CP045](#item-families) cho phạm vi group 2. Đọc
`FxContents/Attr/ItemAttr` từ bản sao cặp pack KR 00967 có SHA-256 khớp `PACK_SHA256` của
`tools/kr_pack_export.py`, rồi dùng `tools/content_census.py::parse_item_attr`. Dữ liệu trùng từng byte
với `boomkr-re/data/kr/KR_ItemAttr.bin`; nguồn `boomkr-re/docs/kr-data.md` §12 và các JSON cùng thư mục
không cho thêm consumer phương tiện. `boom-re/docs/items.md` và bản trích VN cho tên và hàng đối chiếu.

- `kr_static`, KR 00967: SHA-256 ItemAttr `a9b55540c85c21de108e8720b4148b2030a231d907f3709dcb06f7ef728fcdba`.
  IDA batch đọc bản sao DB ảnh giải nén `kr-full.exe`, SHA-256 input
  `cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b`, SHA-256 DB trước khi mở
  `6be15a1fccea0837ee95c50893345e9f13dc217b0fa570975b5c284f0d547fd1`. Địa chỉ KR là VA, `RVA = VA − 0x400000`.
- `vn_static`, VBNB_116: SHA-256 ItemAttr `a98883ce35f3a509719db637f662f2ec90da64dd3c1faa0c9f7eb22c0de7dd1d`.
  Loader VN tại VA `0x89F3DE` theo `boom-re/docs/items.md`; chỉ dùng bảng VN để đối chiếu, không dùng kind hoặc VA đó ở KR.

| Item | Kind KR ↔ VN | Group | ItemAttr thô: weight; `+160`; tail | Dòng `Ship/*` cùng seq: tám số thô | Consumer KR (`candidate`) |
|---|---|---:|---|---|---|
| ItemAirship | 40 ↔ 34 | 2 | `150; 0; [1,0,0,0,0]` | `Airship: [2,10,0,1,2,0,0,0]` | `11928A2`, selector 1. |
| ItemOwl | 41 ↔ 35 | 2 | `350; 0; [2,0,0,0,0]` | `Owl: [1,5,0,0,0,0,0,0]` | `11928A2`, selector 2. |
| ItemTurtle0 | 42 ↔ 36 | 2 | `340; 0; [3,0,0,0,0]` | `Turtle0: [1,2,0,0,0,0,0,0]` | `11928A2`, selector 3. |
| ItemTurtle1 | 43 ↔ 37 | 2 | `60; 0; [4,0,0,0,0]` | `Turtle1: [1,9,0,0,0,0,0,0]` | `11928A2`, selector 4. |
| ItemAirshipBroken | 44 ↔ 38 | 2 | `70; 0; [5,0,0,0,0]` | `AirshipBroken: [2,2,0,1,2,0,0,0]` | `11928A2`, selector 5. |
| ItemUFO | `unknown` ↔ `unknown` | — | Không có dòng tên này trong hai bảng. | Không có mục `Ship/*` tên UFO. | Chưa có kind, selector hay consumer; chưa xác lập tên khác. |

Năm hàng có cùng sprite, thuộc tính ItemAttr và tám số Ship trong hai build. Trong bản sao `FxMap` đã đối chiếu
hash, `Map/Tomb12` có trọng số dương cho Airship, Owl và hai Turtle; `Map/Village10` có 200 cho AirshipBroken
và Owl. Các trọng số trong pack không cho biết luật rơi của server gốc hay tốc độ phương tiện.

- **Nhặt và lên (`kr_static/candidate`):** Tái dùng [đường 124/165 đã đo cho item nền](#item-pickup) làm mỏ neo.
  Dispatch `1192700` rẽ tại `11928A2`, kiểm cast `IEatObject → CGOBomber` và lấy tail đầu qua `64DB00(0)`.
  Nếu guard `actor+0x584/vf+0x94` trả 0, lời gọi tại `1192957` qua `vf+0x17C` dẫn tới `B479E0`
  ở cả `CGOBomber`, `CGOGameBomber` và `CGOCAHeroGameBomber`. Khi guard khác 0, `B48000` và
  `1370710` quyết định có gọi `actor+0x590/vf+0x0C` tại `1192920` hay không; đích tĩnh `B4C7A0`
  nối qua `B4C9D0` tới cùng bộ khởi tạo Ship. Điều kiện thay phương tiện đang dùng còn `unknown`.
- **Di chuyển và tháo (`kr_static/candidate`):** `B479E0` gọi `B978B0` trên `CGOShip` tại actor `+0x804`
  (`B47A60`). Bộ khởi tạo lấy dòng Ship qua `1197900(selector)` và gọi các getter `B98640/B98660/B98680`.
  Sau đó `B479E0` dùng giá trị đọc từ actor `+0x2998` khi gọi `vf+0x264` tại `B47ABF`.
  Slot liền sau nhánh lên (`vf+0x180`) trỏ tới `B73680` ở hai lớp GameBomber trên; `B736A4` gọi
  `B47B80`. Hàm này đọc giá trị tại actor `+0x29A4`, đổi dấu trước khi truyền vào helper, gọi lại `vf+0x264`, rồi gọi
  `CGOShip/vf+0x60` tại `B47D45`. Đây là điểm đo cho đổi di chuyển và tháo; tốc độ, va chạm, trigger
  trúng bom, thao tác tự xuống và caller reset đầu vòng vẫn `unknown`.
- **Wire trạng thái (`kr_static/candidate`):** Switch KR có case S→C `381` tại `E5D930`, gọi
  `E68BB0 → E84CD0`; `E84E2F` gọi cùng lookup `1197900`. Chưa biết schema, hiệu ứng của gói
  hay liên hệ với pickup, tháo và reset. So thời điểm với [127/170 đã đo cho trap/death](life-results.md#peer-events)
  và [UDP movement](udp-peer.md#peer-duo); chưa có subtype hoặc frame trạng thái nào được đo cho phương tiện.

Phép đo tối thiểu: dùng hai client KR cô lập, mỗi ca một kind có trong bảng, cùng nhân vật và đoạn đường
đối chứng không có item. Ghi claim `124` thực và chỉ xác nhận `165` nếu claim khớp; đo cả hai view trước/sau
callback, selector, trạng thái Ship, vị trí theo thời gian và frame thực tế. Cho B đặt một bom để A trúng tia
khi đang cưỡi rồi đi lại cùng đoạn; ghi caller tháo, trạng thái và frame, so với ca trúng tia khi đi bộ.
Nếu thấy thao tác xuống, thử riêng khi không trúng tia. Qua kết quả, vào
[đầu vòng kế](load-phase.md#round-go) và đo lại trạng thái Ship cùng tốc độ trước khi nhặt thêm, kể cả ca
kết thúc vòng khi vẫn cưỡi. Chưa có lượt native F1-d, nên các bước này chưa xác lập hiệu ứng hay quyền server.

<a id="shoe-punch-source"></a>
### Shoe/Punch: pickup, hành động và giới hạn đã đo

[Metadata và giới hạn](../evidence/item-families/shoe-punch.json) bổ sung nguồn KR cho hai item có trọng số dương
trong Tomb12. Shoe 3/effect 4 gọi `B48670`, ghi secure field `actor+5F4`; Punch 4/effect 5 gọi `B48710`, ghi
`actor+60C`. Hai setter so giá trị hiện hành trước khi ghi; điều này chưa thay ca âm runtime. Interface offset được
đối chiếu bằng RTTI và byte của các body với cả hai client đã chạy. Hai field này tách khỏi Roller và MaxPower.

Python reference đã phát current 165 sau actual 124: trong cả hai view, Shoe và Punch lần lượt đổi field tương ứng
`0→1`, giữ sáu stat và actor khác. Đầu vòng hai, hai field trở về 0 rồi nhận lại item; có hai kết quả/về phòng.
Đó là reset giữa hai vòng đã quan sát, chưa có Start thứ ba và không chứng minh lifetime trong mọi trường hợp.
Lượt Shoe riêng trước đó chỉ có một vòng, không được tính thành nghiệm thu hai vòng.

Ở ca Space lần hai tại bom hiện hành, nhánh Punch `B76730` ghi pending record qua `E2CC10`; `E2D310` gọi
`E2E910` để phát actual **119**, rồi clear pending flag. Schema đã đo dài **13 byte**, với widths
`u8,u8,u8,u16,u8,u8,u8,u8,u16,u16`. Hai mẫu có field thứ tư khớp wire ID bom và hai field kế khớp cell
của actual 118. Chưa đặt tên đầy đủ cho mọi field, chưa đặt đơn vị cho giá trị cuối hoặc suy luật thời hạn.
`E2CC10` có decompiler stack warning và tail không liên tiếp; chỉ dùng assembly, byte binding đầy đủ và actual
packet để xác nhận phần trên. Không dùng chữ ký C suy ra như một API đã phục hồi chính xác.

Shoe có nhánh `E6F510→E6F8F0→E6F9C0` dẫn tới cùng pending-record path khi đủ điều kiện;
[ca dùng reference selector 5](#rust-bomb-motion-codec) đã có hai vòng trong phạm vi ghi bên dưới. Reader 160 đọc 8 byte và gọi các nhánh entity/counter; chưa có bằng chứng nó
là reply cho 119 hoặc là gói dịch chuyển bom. Hai lượt không gửi ACK cho hành động này.

**Shoe/Punch có các lát Rust opt-in giới hạn** ở [contract bên dưới](#rust-bomb-motion-codec).
Pickup/reset và phát request riêng lẻ chưa đủ để mở toàn bộ hành động: mỗi lát cần consumer, lifecycle bom, peer,
ca không đủ điều kiện và guard membership/vòng hiện hành. Không coi các run Python RE là bằng chứng account,
Internet hoặc vận hành trên Rust.

<a id="rust-bomb-motion-codec"></a>
### Rust 119/162 và lát Shoe/Punch có giới hạn

[Metadata nguồn và codec](../evidence/item-families/bomb-motion-codec.json) bổ sung reader 162 `E5EFC0`:
body **9 byte**, widths `u8,u8,u8,u16,u8,u8,u8,u8`. Reader kiểm context, tìm actor và wire ID của bom;
selector `≤3` gọi virtual `+94` của `CGOGameBomb`, nhánh còn lại gọi `+90`. Không suy mọi selector đã hỗ trợ
trên server từ việc tìm được các nhánh này.

Python reference trả candidate 162 từ prefix của actual 119. Cả hai view đọc `9/9`, errors 0 và gọi
`B37A00` trên cùng wire ID. Secure root `+1AC/+1B8` đổi từ cell nguồn sang cell đích; nhánh này cũng đặt
secure `CC=0`. Ở các lượt chỉ gửi 162 rồi 158 loại 0, bom đã hạ cánh trước 158 nhưng vẫn active tại cleanup;
một lượt còn thấy cùng bom active ở update hơn hai giây sau đó, không có actual 118 tiếp. Vì vậy tăng fuse
không phải cách sửa được bằng chứng đó hỗ trợ. Callback pool `AEF700` quan sát được thuộc nhận bom 159,
trả đúng pointer trước 162; không gọi nó là callback thu hồi. Guard đầu của nhánh pool thường là
`CC == 0 || E4 != 0`, rồi mới kiểm serial và điều kiện nhánh. `B378F0` ghi secure `E4`, không phải ACK.

[Policy reference đã đo](../evidence/item-families/bomb-retirement.json) gửi **158 loại 1 ngắn**, widths
`u8,u16,u8` với count 1, wire ID hiện hành, loại 1, dài **4 byte**; ngay sau đó mới gửi 158 loại 0 dài
13 byte để nổ tại deadline 6 giây đang dùng. Nguồn `E14360` cho thấy loại 1 căn lại pixel/root, đăng ký
footprint khi phù hợp và gọi `B37880(-1)`, đặt lại `CC=1`; nó không phải gói nổ. Hai view đọc đủ cả hai
body, chọn root đã dịch chuyển và ghi active về 0 trước cleanup. Sau hành động có actual 118 mới với ID
khác, rồi kết quả/về phòng; phép lặp có hai vòng hoàn tất và reset cờ pickup ở đầu vòng sau.

Đây là **emu_choice/design đã kiểm trên reference**, không phải thứ tự server gốc được phục hồi. Phạm vi
của lượt retirement reference này là Punch C0, selector 3, một hành động mỗi vòng và bom đã hạ cánh trước
deadline. Shoe selector 5 có phép đo riêng bên dưới; các selector khác, hành động sát deadline/khi đang bay,
lặp hoặc nhắm cell tùy ý và ca không đủ điều kiện vẫn cần bằng chứng tương ứng.
Các lượt lỗi wineboot, collector, recipe và giới hạn probe được giữ trong metadata, không tính là ca âm gameplay.

Rust `kr-wire` có `BombAction119::decode` cho đúng 13 byte, giữ hai `u16` cuối bằng `opaque_tail`, và
`BombMotion162::{decode,encode}` cho đúng 9 byte. Các trường gồm context, slot, selector, wire ID, cell nguồn
và cell đích; codec không cấp quyền cho các giá trị này. Regression dùng hai request thực có ID/cell/tail khác nhau
và body 162 đã qua native reader, kiểm cả gói thiếu/thừa và nhầm độ dài. Không dùng roundtrip tự sinh làm oracle.

[Lát Rust Punch đã nghiệm thu](../evidence/rust-service/g1-punch.json) có handler 119 và policy opt-in
`report_derived_punch`. Policy nền `report_derived_basic` vẫn từ chối floor 4; policy Punch này không mở Shoe 3. Chỉ nhận
actor thuộc membership/slot/vòng đang chạy, đã nhặt Punch và chưa có report trap/death. Bom phải thuộc người
thao tác, có wire ID hiện hành chưa tái sử dụng, source cell khớp ledger; selector 3 chỉ đi từ hàng nguồn sang
hàng nguồn trừ 1, cùng cột. Cell đích phải trống theo terrain và ngoài cell hủy bom của map. Một bom chỉ được
chuyển một lần; server từ chối khi còn dưới 3 giây hoặc đã bắt đầu ghi blast cho bất kỳ recipient nào. Đây là
các giới hạn **design** của lát đầu, không phải toàn bộ luật KR hoặc bảo đảm cho mọi độ trễ/chuyển động.

Admission chạy trên bản clone của room: ledger và terrain cùng đổi root, chỉ publish khi đủ chỗ trong hàng đợi
của mọi peer. Quota, owner, allocation và deadline 6 giây cũ giữ nguyên. Trước lần ghi blast đầu tiên, server
đóng admission motion; với bom đã chuyển, mỗi recipient nhận 158 loại 1 rồi loại 0 trong cùng batch. Quota và
terrain clearance chỉ commit sau khi mọi recipient hiện hành ghi xong. Trong player round, motion đã chấp nhận
thuộc allocation/vòng hiện hành và các recipient còn lại, giống 159/158: nếu actor rời trước khi peer nhận 162,
peer vẫn cần nhận motion đó để root client khớp ledger/terrain. Membership đã thu hồi không được phát lệnh mới;
allocation đã kết thúc, recipient/visit mới hoặc vòng mới không nhận batch cũ. Các profile hủy toàn trận giữ
subject guard. [Correction nhánh rời phòng](../evidence/rust-service/g1-punch-departure.json) có kiểm hồi quy
server; chưa có fault injection native cho cửa sổ queued162/EOF. Không thêm epoch/key vào 119, không lấy tail
opaque làm quyền hạn và không gửi 160 như ACK.

Native Wine trên Rust dùng hai account backend, companion xác thực và LAN cô lập: account A có nhân vật C1
đã lưu, peer B là C0. Floor chỉ có Punch 4, không có Shoe. Mỗi vòng Space trước pickup tạo bom thường nhưng
không có 119; sau pickup actual 119 selector 3 được nhận, cả hai reader 162 đọc đủ, root chuyển `(6,3)→(5,3)`,
bom được thu hồi trước cleanup rồi có actual 118 mới. Hai vòng có kết quả/về phòng, cờ pickup reset và hai
journal DB thưởng 0 đúng identity/nhân vật. Một lượt riêng phát selector 1 do hướng nhập khác bị từ chối;
không chuẩn hóa nó thành selector 3. Test server còn kiểm sai transport/slot/owner, thiếu item, death, vòng/visit
cũ, hàng đợi đầy, cell bị chặn, deadline và đóng admission trước blast. Các kiểm này không thay bằng chứng
native cho Shoe, selector khác, hành động lặp/sát deadline/khi đang bay hoặc mọi nhân vật/map/roster.

[Reference Punch hướng xuống](../evidence/item-families/punch-down-action.json) đã có hai vòng C0:
actual 119 selector 1, source `(6,3)` và target `(7,3)`, cả hai reader 162 đọc `9/9`, errors 0. Bom hạ cánh,
short 158 rebind rồi active về 0 trước cleanup; có actual 118 mới, hai kết quả/về phòng và reset hai cờ pickup.
Luồng chunk TCP được ghép lại và kiểm CRC độc lập; 712 snapshot field khớp native getter. Reference nhặt cả
Shoe/Punch, chỉ phát hành động Punch này. Short 158 trước blast tại deadline 6 giây vẫn là policy reference,
không phải thứ tự server gốc. Các lượt sai geometry/căn phím hoặc chỉ một chuyển động giữ nguyên là partial/lỗi.

[Lát Rust Punch hướng xuống](../evidence/rust-service/g1-punch-down.json) đã qua native hai account trên
Wine/LAN cô lập. Policy opt-in `report_derived_punch` và `report_derived_shoe_punch` nhận thêm selector 1,
source row cộng 1, cùng cột; selector 3 vẫn đi một hàng phía trên. Giữ toàn bộ guard owner, membership/vòng,
item/life, deadline, terrain và queue đã nêu. Biên hàng đích được kiểm riêng để không ra ngoài map. Reservation
cho Shoe mới giữ cả ô nguồn cũ của Punch đang active ở cả hai chiều dọc đến khi allocation hoàn tất.

Actor A là C1 đã lưu, peer B là C0; floor chỉ có Punch 4. Mỗi vòng Space trước pickup chỉ tạo bom thường,
không phát 119. Sau pickup có exact 119/162 `(6,3)→(7,3)`, reader `9/9`, errors 0, landing trước rebind và
active 0 trước cleanup trong cả hai view. Có actual 118 mới, hai kết quả/về phòng/reset và hai journal đúng
account/player/character, thưởng 0; rank rows nhận qua TCP khớp outcome DB. Test server giữ các ca sai quyền,
queue rollback, biên/terrain bị chặn, quota đến mọi receipt và source reservation. Các ca lỗi/partial của
reference không thay nghiệm thu này. Lát ngang có bằng chứng riêng bên dưới; bom người khác, motion
lặp/khi đang chạy/sát deadline và mọi nhân vật/map/roster vẫn chưa mở. Không suy toàn bộ luật server gốc hoặc G1.

[Reference Punch ngang](../evidence/item-families/punch-horizontal-action.json) có hai vòng actual 119
selector 0, source `(6,3)` và target `(6,5)`. Cả hai view đọc đủ 162, landing/rebind/active 0 trước cleanup,
có bom tiếp, hai kết quả/về phòng/reset; 712 field khớp native getter. Đây là endpoint đã đo, không chứng minh
đường bay chỉ dài hai ô hoặc không wrap. Reference nhặt cả hai cờ Shoe/Punch, không thay acceptance Rust.

[Lát Rust ngang](../evidence/rust-service/g1-punch-horizontal.json) nhận selector 0 cùng hàng, target column
bằng source column cộng 2, trong biên map. Policy opt-in giữ các guard owner/member/vòng/item/life, deadline,
clone/queue, quota/fuse và short 158→blast. Vì chưa đo toàn đường bay, server reserve **cả hàng** đến khi mọi
recipient hoàn tất retirement. Admission từ chối allocation/path khác giao hàng, placement mới, vertical Punch
và Shoe giao hàng đó; mọi ô hủy bom trên hàng đều chặn admission, kể cả ô ngoài đoạn giữa hai endpoint.
Đây là policy bảo thủ của server; không khẳng định luật KR chỉ dùng hai ô hoặc nguyên nhân chọn target.

Rust có hai account, actor C1 đã lưu và peer C0, floor chỉ Punch 4: trước pickup chỉ có bom thường/no119;
hai actual 119/162 `(6,3)→(6,5)` được nhận trong cả hai view, landing trước rebind rồi active 0 trước cleanup,
có bom tiếp, hai kết quả/về phòng/reset và journal thưởng 0 đúng account/player/character. 216 snapshot khớp
native getter; rank TCP khớp outcome DB. Lượt này hoàn tất gameplay nhưng launcher **exit 1** vì unmount busy
sau khi đã export cùng byte; B gdb còn giữ output fd tại chẩn đoán. Sau namespace exit, không còn process
hoặc mount thuộc lượt đó; 70 file export đã được phục hồi với size/SHA khớp và kiểm ngữ nghĩa độc lập.
Giữ nguyên lỗi/receipt/archive, không gọi unmount gốc thành công hoặc log crash-durable. Test bổ sung kiểm
cancellation ngoài endpoint; production binary giữ nguyên. Hướng selector 2, Shoe các hướng/khoảng cách khác,
motion lặp/inflight/late và mọi map/roster/nhân vật còn theo acceptance G1.

[Reference selector 2](../evidence/item-families/punch-selector2-action.json) có hai vòng actual 119
`(6,4)→(6,3)`, landing/rebind/active 0 trước cleanup, bom tiếp và hai kết quả/về phòng/reset trong cả hai view;
712 field khớp native getter và wire/CRC kiểm độc lập. Không áp đối xứng +2 cột của selector 0: selector 2 ở ca
này có endpoint trừ một cột. Nhánh [#462](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/462) có guard/test
Rust cho cùng hàng, đích trừ một cột và cùng reservation cả hàng. [Ma trận Punch theo nhóm](../evidence/rust-service/g1-punch-matrix.json)
đã ghi native Rust selector 2: hai account, floor chỉ Punch, peer, retirement, reset và hai kết quả journal thưởng 0;
216 getter native khớp và cleanup hoàn tất sạch. Các lớp khoảng cách/biên còn mở; không suy toàn đường bay
hoặc đóng nhóm Punch từ bốn endpoint đã đo.

**Punch hiện hành theo ma trận nhóm.** [Bằng chứng nhóm](../evidence/rust-service/g1-punch-matrix.json)
thay giới hạn endpoint cố định của các lát lịch sử trên: selector 0/2 nhận endpoint trong cùng hàng,
selector 1/3 trong cùng cột, ở trong map. `B38A80` quét và wrap; endpoint có thể trùng nguồn sau vòng quét.
Rust giữ nguyên source/target của actual 119 và exact prefix 9 byte ở 162, không chuẩn hóa target hoặc dùng
hai byte tail làm quyền hạn. Đây là policy **report-derived design**, không là vật lý hoặc luật server KR gốc.
Toàn hàng/cột của Punch được giữ đến mọi retirement receipt; mọi cancellation cell trên trục đều chặn motion,
kể cả ngoài hai endpoint. Owner, transport/member/vòng, item/life, deadline, clone/queue, quota và retirement
vẫn giữ fences. Test hồi quy kiểm endpoint/biên, foreign/current/old membership, source/path, queue rollback,
quarantine và chuyển vòng; các control đó không được gọi là native fault injection.

Hai batch Rust mới dùng cùng harness tham số hóa: mỗi batch bốn vòng, đủ selector 0–3 và cả hai view.
Batch đầu có endpoint delta +5 cột, +3 hàng, −1 cột và −3 hàng. Batch biên có nguồn (6,5)→(6,10),
(9,4)→(9,4), (6,3)→(6,3), (3,4)→(3,4); delta 0 không có nghĩa là bom không bay.
Mỗi vòng có bom thường/no119 trước pickup, pickup Punch duy nhất, landing trước rebind/cleanup,
kết quả/về phòng và cờ reset; mỗi batch có 432 snapshot khớp getter native, journal đúng account và thưởng 0.
Hai batch mới exit 0 và cleanup sạch. Selector 2 hai vòng trước đó được gộp vào cùng nhóm; các lượt lỗi vẫn giữ
raw và giới hạn riêng. Không suy mọi roster, nhân vật, queued EOF, late/inflight hoặc Internet từ ma trận này.

[Hành động Shoe trên reference](../evidence/item-families/shoe-action.json) dùng floor chỉ có Shoe 3, C0,
bom của người thao tác và va chạm tự nhiên từ bên dưới. Actual 119 có selector **5**, source `(6,3)` và target
`(3,3)`; prefix 9 byte được trả bằng candidate 162. Hai view đọc đủ, gọi `B37910`, đổi root, vào state 3 rồi
dừng ở state 4 trước short 158 loại 1 và gói nổ loại 0. Bom được chọn ở root mới, active về 0 trước cleanup;
sau đó có actual 118 mới và kết quả/về phòng trong hai vòng. Native getter trên snapshot xác nhận Shoe về 0
ở clock rồi lên 1 khi pickup, Punch giữ 0. Đây là reference với identity diagnostic, chưa là acceptance Rust.

Nguồn `E6F9C0` gọi `EAB1C0` rồi queue selector bằng direction cộng 4. `EAB1C0` khởi đích ở cell nguồn,
lặp hữu hạn qua `E9BFA0`/`E9D830` và kiểm occupancy; ở ca đo, đối số giới hạn là 13, không phải vô hạn.
Probe tại đúng `ret` của hàm trả 1 và target `(3,3)`, trước vật cản hàng 2. `B3A3B0` cập nhật pixel, kiểm
vật cản trung gian và đổi state 3→4 khi tới đích. Chưa phục hồi tốc độ hoặc toàn bộ luật va chạm. Không tự
mở các hướng khác, bom người khác, hành động lặp/sát deadline/khi đang chạy, hoặc đường có actor/vật cản.

Lượt control đầu chỉ có một hành động/kết quả và clock vòng hai; nó được giữ là partial. Mẫu EAX từ
`FinishBreakpoint` của lượt đó không chứng minh giá trị trả về boolean. Control mới chờ placement hiện hành,
đổi nhịp nhập và đặt breakpoint ở lệnh `ret` đã có trong body được đối chiếu byte; không sửa raw cũ hoặc
khẳng định nguyên nhân của lượt thiếu hành động. Reference không tự cấp quyền bật Shoe; lát Rust với guard
path/terrain/placement, queue và membership/vòng được nghiệm thu riêng dưới đây.

[Lát Rust Shoe selector 5](../evidence/rust-service/g1-shoe.json) dùng policy mới `report_derived_shoe_punch`.
Policy này giữ lát Punch selector 3 và mở riêng Shoe 3; hai preset `report_derived_basic`/`report_derived_punch`
giữ phạm vi cũ. Shoe là cờ theo membership/vòng, về 0 ở begin, không phải inventory bền vững. Chỉ nhận actor
đã nhặt item, chưa trap/death, slot/transport hiện hành, bom của chính actor và wire ID chưa tái sử dụng.
Geometry ban đầu là ba hàng phía trên cùng cột; đường từ nguồn tới đích phải trống, có vật cản ngay sau đích,
không có bom khác, ray barrier hoặc cell hủy bom. Đây là giới hạn **design**, không phải toàn luật đá bom KR.

Ledger giữ reservation cho toàn hành lang Shoe, gồm ô nguồn, đến khi mọi recipient hiện hành hoàn thành
158 hoặc rời allocation. Placement mới hoặc motion giao đường bị từ chối. Khi kiểm một Shoe mới, nguồn và
đích của Punch còn active cũng là vùng có thể đang chuyển động; không chỉ kiểm root mới. Thời gian giữ
reservation là policy bảo thủ vì không có wire báo dừng đã xác nhận, không phải thời gian KR được đo.
Clone, reserve mọi queue, deadline tối thiểu 3 giây, owner/quota/fuse 6 giây và chuỗi short158→blast giữ như
lát Punch. Motion đã nhận còn phục vụ survivor sau actor departure; lệnh từ membership cũ và batch của
allocation/vòng/recipient cũ vẫn bị chặn. Test server kiểm rollback, corridor, deadline và cửa sổ rời phòng;
chưa có native fault injection cho những race này.

Native Wine/LAN cô lập trên Rust có hai account: A là C1 đã lưu, B là C0, floor chỉ có Shoe 3. Mỗi vòng
contact trước pickup không có 119; sau pickup actual119 selector5 chuyển `(6,3)→(3,3)` trong cả hai view,
state 3→4 trước rebind/blast và active 0 trước cleanup. Có bom tiếp, hai kết quả/về phòng, reset cờ, hai journal
thưởng 0 đúng account/player/character. Ca chỉ phát đích `(2,3)` đã bị từ chối theo geometry; không chuẩn hóa
request đó thành đích hỗ trợ. Ca khác dừng khi cả hai report chết được giữ partial, không tính thành nghiệm thu.
Các hướng/khoảng cách khác, foreign bomb, motion lặp/sát deadline/khi đang chạy, mọi nhân vật/map/roster vẫn mở.

[Correction đồng bộ tài liệu](../evidence/checkpoints/shoe-docs-sync-correction.json) giữ lỗi kiểm link/catalog của CP329; không sửa record hoặc raw và không thay nghiệm thu runtime.

**Shoe hiện hành theo ma trận nhóm.** [Bằng chứng nhóm](../evidence/rust-service/g1-shoe-matrix.json)
thay giới hạn chỉ up-three của lát lịch sử trên. Native contact phát selector **4/trái, 5/lên, 6/phải, 7/xuống**;
Rust kiểm cùng trục, đúng dấu, khác nguồn, trong map và toàn corridor không bị terrain/allocation khác chặn.
Nguồn và corridor giữ reservation đến mọi retirement receipt, kể cả khi actor đã rời nhưng allocation còn thuộc
vòng hiện hành. Giữ các fences item/life/transport/member/vòng/deadline/quota/queue, không nhận report cũ cho
membership hoặc vòng kế. Không buộc một wall nằm ngay sau target: target do native báo sau kiểm collision/occupancy,
policy này không khôi phục toàn vật lý hay authority server gốc.

Ma trận tám vòng trên Rust có đủ bốn hướng, hai peer/kết quả/reset và các endpoint dọc dài 2/3/6 ô, ngang một ô.
Hai vòng bổ sung ngang có `(6,3)→(6,5)` và `(6,5)→(6,3)`, sau bom thường trước pickup mở ô tiếp cận.
Mỗi vòng có ca trước pickup/no119, Shoe duy nhất, exact119/162, landing/state4/rebind/cleanup và journal đúng
hai account C1/C0, thưởng 0. Probe EAB1C0 đọc giới hạn lời gọi **13 dọc, 15 ngang** trong các ca này; không gọi
đó là stat nhân vật. Snapshot protected field kiểm bằng getter native riêng, không lấy Rust log làm oracle.
Raw/matrix gốc giữ nguyên: nhãn selector ngang của batch8 bị đảo và count8 kế thừa ở batch2 có correction
riêng gắn hash, kiểm phím/source/target/reader; không biến deviation của input thành input nguyên bản đạt.
Các run Factory07 không cho thêm acceptance: hai cấu hình bị từ chối trước ready, một route gặp tường đã giữ
lỗi. Không suy mọi ô/roster/nhân vật, native queued EOF/late/inflight hoặc Internet từ ma trận này.

**Correction nguồn phụ:** `B486B0` nhận interface `actor+584`, cộng `70` tới wrapper `actor+5F4`, đúng bằng
interface `actor+58C` cộng `68` của setter. Câu `+5F4+8` trong ghi chép phụ trước là lỗi tính offset; sidecar correction
mới giữ nguyên ghi chép cũ. Contract, byte binding và snapshot runtime trước đã dùng `+5F4` đúng.

<a id="rust-maxpower"></a>
### Rust MaxPower: max theo nhân vật và vòng hiện hành

[Bằng chứng Rust MaxPower](../evidence/rust-service/g1-maxpower.json) tái dùng contract CP045 của kind 5,
effect 6 và endpoint `B472A0`; ba vùng dispatcher/endpoint đã đối chiếu byte với diagnostic copy hiện tại.
Không cần mở lại body hoặc wire pickup đã đo. Rust nhận kind 5 trong catalog 582 và policy `ReportDerivedBasic`:
claim hợp lệ đặt current power bằng `maximum_power` của nhân vật, giữ nguyên capacity, speed và mọi max.
Không hardcode power 7 cho mọi nhân vật. Initial map và từng map choice từ chối floor/hidden MaxPower trước
bind nếu policy stat này tắt. Kind chưa đo vẫn bị từ chối, các guard identity/membership/round và hàng đợi giữ nguyên.

Native hai account trên Factory07 có A chọn C0, B chọn C1. Mỗi vòng cả hai nhặt MaxPower rồi Fire và MaxPower
thứ hai khi đã đạt trần. Native 165 đọc đủ 12 byte, tìm đúng item và xác nhận trong hai view: power 1→7 ở C0,
1→9 ở C1; Fire/MaxPower ở trần được tiêu thụ nhưng không tăng thêm. Capacity/speed và stat actor khác giữ nguyên.
Actual C2S118 và broadcast 159 mang power 7/9; terrain phản ánh đúng cờ trước/sau sáu bom, có hai rank/return
và Start thứ ba. Clock mới khôi phục power 1, sáu stat, spawn và thùng. Các root được chọn có static/thùng chặn gần,
nên không suy đã nghiệm thu mọi ray dài 7/9 hoặc mọi hướng/map từ power trong packet.

Index/vị trí item hiện là **design**; journal join đúng account/player từ launcher, nhân vật, đội và kết quả,
thưởng 0. Tests kiểm dead actor không được claim, member/round cũ, duplicate, full peer queue không tiêu thụ
item hoặc tăng stat, power guard và reset. Ca âm native là không tăng thêm power ở cap; chưa có ca native
claim khi dead/trapped. Không suy TTL hoặc “vĩnh viễn” từ việc effect giữ qua các mốc của hai vòng.

Run giữ exit 1 vì output tmpfs unmount busy sau khi export được so đủ size/SHA; DB/host bind đã cleanup và
namespace hết PID. Xác minh từ locator export, không gọi output unmount thành công hoặc nghiệm thu M8.
Frozen verifier giả định locator chuẩn và bản sửa dùng sai vị trí wire ID 159 đều giữ nguyên; bản kiểm tiếp
dùng field 11 theo contract cũ, không đổi raw hoặc wire. Hai lần kiểm config âm dừng sớm vì DB path/quyền parent
không hợp lệ được giữ riêng; ca với DB copy và parent private đúng mới tới guard MaxPower. Build Linux, locked
tests, fmt, clippy và snapshot source/binary có nguồn riêng, không thay native acceptance.

<a id="item-pickup"></a>
### Item trên nền và xác nhận nhặt — CP030

**kr_static/kr_runtime confirmed trong phạm vi copy Wine**; [recipe/control/hash](../evidence/item-pickup/README.md).
Giữ cùng binary chẩn đoán SHAe9bb47… và profile ExVillage01/278, hai identity/slot riêng, không patch thêm.
VN hướng dẫn cách thử layout trước start và report→confirm; opcode, index32bit và reader KR dưới đây
được phục hồi riêng. Không áp VN553/117/158 hoặc index16bit sang KR.

**Layout trước load:** prepare dispatcher `DC51F0→DC5378`, **S→C582**, đọc serialized blob rồi
`D824D0`. Guard `5725B0` thực trả1; `11ED590` đọc từng record. Inner sau giải blob gồm:

```text
u16LE n1; n1 × {u32LE index, u8 row, u8 column, u32LE kind}
u16LE n2; n2 × cùng record
u16LE n3; n3 × cùng record
```

Tại CP030 chỉ list1 là item trên nền đã quan sát; list2/3 đều rỗng trong các run đó.
CP032 kiểm tiếp [list2 giấu trong gạch](#hidden-items); list3 vẫn chưa có ca nonempty.
Mẫu hai item tiêu26byte inner/36byte payload; mẫu một item tiêu16/26byte, errors0 cả hai.
Framer/blob/getters và record reader native kiểm độc lập; runtime guard/list install/map render là
nguồn riêng. Tag raw0 trong recipe đã được reader chấp nhận, không phải capture server gốc.
Item record dùng hàng/cột, khác point spawn map có quy ước riêng; không chuyển nhãn tọa độ giữa hai kiểu.
Kind0 vẽ Bomb, kind1 vẽ Fire ở profile này; icon không chứng minh đã cộng số bom/tầm nổ.

**Request/confirm:** actual **C→S124** từ `E2E390` (stack thực `E2E5EC`) có19byte:

```text
u8 ctx, u8 slot, u8 row, u8 column, u32 index, u16 kind, u32 t, u32 k, bool flag
```

Đây là typed outer fields qua primitive KR, không phải raw LE của inner blob. Writer lấy record+0,
slot từ argument, record+0C/+10/+08/+14/+1C/+20/+24 tương ứng. Nguồn `t` đã phục hồi ở
[CP055](#pickup-tick); `k` có [guard hẹp CP049](#pickup-round-key), không là auth. Ca CP030 ctx0, flag0, k12345 do profile peer đã cấp.
Native replay actual frame theo seq/socket xác nhận CRC, độ dài và giá trị; không dựng request từ encoder.

**S→C165**: `E5D570→E63260(packet,0)`, payload12byte:

```text
u8 ok, u8 ctx, u8 slot, u8 row, u8 column, u32 index, u16 kind, bool flag
```

`E2C660` xóa pending theo ctx/index trước kiểm ok/context. Native reject/context-mismatch chỉ kiểm
header/nhánh từ chối với cache/world model; positive effect lấy từ runtime. `ok!=0` và context khớp
mới đi tiếp lookup actor theo slot; interface actor+588 gọi `F73D30`, sau đó tìm item, kiểm tọa độ/kind
trước phần xử lý tiếp. Cạnh tĩnh `F73D30→B52840→E14DA0` tại CP030 mới là mỏ neo; CP031 đã nối
và đo [stat Bomb/Fire cùng giới hạn](#item-effects).
Không gửi nguyên124 làm reply: thêm ok, bỏ haiu32 t/k, mã hóa lại165 theo seq của từng receiver.

**Control và can thiệp:** `near-control` có A gửi124/index17/row4/col3/kind0 nhưng không165;
item hiện lại sau khi rời ô. `pickup-ack` giữ input/placement A đó; B được đặt Fire tại ô hàng8/cột3
thay vị trí gần gạch ở control. Cả B rồi A gửi124 thật; mỗi xác nhận165 phát tới cả hai, đọc12/12/errors0,
context0 khớp, đúng actor/item/tọa độ/kind; pending local được xóa và ảnh cuối không còn hai item.
Đây là can thiệp có đối chứng cho A; B đổi placement nên không tuyên bố một biến duy nhất cho cả run.

**Một ca tranh index:** `pickup-race` đặt Bomb/index19 ở hàng6/cột3, input Down(A)/Up(B) đồng thời.
Cả hai gửi124 cho cùng index trước ACK; stub giữ request đến trước, chờ2s rồi chỉ phát một positive165
slot0/index19 tới A/B. Request B ghi bị từ chối do đã giữ index. Cả hai có một callback165 đầy đủ,
cùng slot thắng, xóa pending của chính mình và không còn item ở ảnh cuối. Đã kiểm một ca/two requests;
chưa đảo thứ tự winner, duplicate replay, stale/round generation, stat chỉ cộng một lần hoặc mọi race.

| Vai trò cho hành vi đã dùng | Bằng chứng/giới hạn |
|---|---|
| Creator | Stub chọn layout/index/kind; client tự phát124 khi chạm item. Không phục hồi thuật toán drop server gốc. |
| Simulator | Client dựng item, giữ pending và xử lý165/interface. Tại CP030 chưa đo stat/cap; kết quả Bomb/Fire mới ở [CP031](#item-effects). |
| Validator | Harness kiểm CRC, slot theo origin và allowlist index/tọa độ/kind; giữ một index cho request đầu. Đây là design hẹp, thiếu kiểm room/round/generation/timing đầy đủ. |
| Broadcaster | Stub mã hóa165 riêng từng socket và gửi cả origin lẫn peer; hai consumer đã quan sát. |
| Peer-editable state | Report124 chỉ được chấp nhận theo slot gắn socket trong harness. Toàn grammar/quyền UDP và chống sửa state ngoài đường này còn unknown. |

Placement/first-arrival/delay2s là **emu_choice/design**, không luật server gốc. Stat và duplicate sau
xóa được kiểm tiếp hẹp ở [CP031](#item-effects); item ẩn/phá block/drop, variants, malformed/stale/
reset/disconnect và full auth/cache/codec còn mở. Không ghép riêng mốc này với CP029 thành nghiệm thu
full match/K5/K6a hoặc LAN/Internet.

<a id="current-pickup-race"></a>
### Tranh index thật trên serializedservice — CP062

**kr_runtime/confirmed hẹp**, [recipe/hash/giới hạn](../evidence/current-pickup-race/README.md).
Tái dùng CP030race, CP061profile41/floor17/Roller/overlapspawn/currentkey và ItemClaims; không newwire/
nativebody/patch/servicecode. NormalUI A/B cùngDown0.30s qua barrier, không mintreport bằng encoder.
Wrapperlocal chỉ giữ **delivery165 trong2s**, productionclaim/consume/key/reportguard/winnerlogic nguyên.
Đây là canthiệp đo để cả hai nativewriter có thời gian gửi, không latency/arbitrationpolicy servergốc.

Cùng8connections/2cycles: mỗi vòng **hai actualTCP124** index17/hàng4/cột3/kind2/currentkey,
loser đến trước165delivery khoảng1.92s/v1 và1.96s/v2. Production first-arrivalclaim thắng:

| Vòng/key | Request tới trước / sau | Native165 cùngslot trênA/B | SecureRoller trước→sau ở cả2view |
|---|---|---|---|
|1/12345|A / B|một165/receiver,slot0|A5→6, B4→4|
|2/12346|B / A|một165/receiver,slot1|A5→5, B4→5|

Không có165cho requestloser; cùngpayload positive165 gửi origin/peer một lần, callback12/errors0.
Winner đảo tự nhiên theo actualarrival, không forcewinner. Clockv2/statbefore165 trởbaseline A5/B4;
claim reusedindex17 còn nhận được trước khi reserve/consume mới. Hai bom/stateA1/peerstate trước175,
2rank/return/room; không auth/fullvalidator/whole reset hoặc K6a/LAN/Internet từ sốcallback.

Production claim được consume **trước** queue165; wrapper flush sau2s chỉ nếu room/generation hiện hành,
discard khácvòng nhưng ca discard chưa runtime riêng. Không thay firstarrival bằng công bằng/thờiđiểm
va chạm tuyệt đối; normalUIbarrier không chứng minh physics cùngtick. CP030oldharness và CP062current
service là hai scope có liên kết, không nâng mọi duplicate/stale/transport/mode biến thể. Khoảng trống report127 của CP062 được tiếp ở [CP063](udp-peer.md#report127-freshness); thứ tự hiện hành chỉ ở [PLAN](../PLAN.md#current-order).

<a id="distinct-spawn-race"></a>
### Tranh item thật giữa hai spawn riêng — CP066

**kr_runtime/confirmed hẹp**, [recipe/raw/hash](../evidence/distinct-spawn-race/README.md).
Tái dùng CP030 midpoint/Down-Up1s, CP062 claim/deliveryhold/nativeeffect, CP065 currentprofile/
mapspawn0/1. Không thêm body native/patch/productionservicecode. Chỉ datafixture582 và catalog
row4→6, giữ index17/col3/kind2; native11ED590 kiểm fullinner16/1floorrecord/2list rỗng, không dropalgorithm.

NormalUI ADown/BUp1s qua barrier, không mintrequest; wrapper giữ outgoingpositive1652s nhưCP062,
productionclaim/consume/currentkey/reportguard không đổi. Cùngrun8connections,2Start/clock và native
spawn/reset A120159/B120399 cảhai view, hai key12345→12346. Mỗi vòng có **hai actualTCP124** cùng
index17/row6/col3/kind2/currentkey trước delivery165. Firstarrival claim thắng một lần, loser không165;
secureRoller chỉwinner+1 cảhai view, vòng2reset baseline A5/B4 trước nhặt mới. Winner/order cụ thể ở
bảng evidence, không force đảo winner hoặc gọi đồng bộphím là cùng physicstick/fairnessservergốc.

Sau165, normalUIBRight/Aspace cho hai bom/17/19→B170/B21/stateA1bothbefore175/hairesult/return/room.
Đây là same-run integration của ca tranhitem/distinctspawn/two-cycle đã đo, không đủ fullmovement/
transportcausality/actorauth/distancevalidator/fullreset/K6a/LAN/Internet. Delay2s là canthiệp đo;
không suy productionlatency hoặc TCP replay admission. Wrapperdiscardqueued165khácgeneration chưa
runtime riêng, policytiming/fuse/rank/placement/authfixture vẫn design.

Tại CP066, C→S118 admission còn thiếu; [CP067](bombs.md#c2s118-admission) đã đo control/fix, không coi
oldS→C159/158 hoặc k124 là proofrequest auth. Thứ tự hiện hành chỉ ở PLAN.

<a id="pickup-round-key"></a>
### Guard124 theo khóa đã cấp — CP049

[Schema124/165 và consumer](#item-pickup) giữ nguyên. [Recipe/raw/hash/controls](../evidence/stale124/README.md)
tái dùng CP030/042/047/048 và nguồn VN, không phục hồi lại receiver hoặc đổi layout/opcode theo VN.

`probe_service.receive_channel` kiểm **u32 k (field7)** bằng khóa đang cấp ở u32 thứ hai153 **trước
ItemClaims.claim/consume**. Khác khóa: ghi `round_key_mismatch`, không phát165/không chiếm claim.
Khớp khóa: tiếp origin/context/catalog/first-winner/one-consume hiện có. So sánh cảu32 là lựa chọn
`design`, không sao chép quy tắc low8 của [movement](udp-peer.md#udp-generation) hoặc gọi k là auth.

**kr_runtime/confirmed hẹp trên Wine copy:** control service CP048 nhận lại body actual124 vòng1
(k12345) qua dispatcher ở vòng2/key12346 rồi phát165 trước input mới; native secure Roller5→6
cả A/B, không actual124 vòng2. Service có guard loại body cũ; sau input **actual TCP124/k12346**
vẫn phát165, callback12/12/errors0 và Roller5→6 cả hai. Clock vòng2 về5; hai run đều có hai
bom/17/19/peerstate1 trước175/180/phòng. Điều này chứng minh consequence của dispatch/165 và guard
hẹp; body chèn bypass C2S transport, **chưa kiểm TCP admission của replay**. Không giả nó là request
mới do native writer phát. Hai proxy setup dừng trước phép đo được giữ riêng, nguyên nhân unknown.

Không còn coi “service chưa kiểm k” là trạng thái hiện hành. [Nguồn t](#pickup-tick) đã giải ở CP055;
policy dùng t, distance/actorstate, request giả đã
đổi k hiện tại, mode/variant, key lifetime/roster/otherUDP và fullvalidator vẫn mở. Equality không xác
thực sender; fuse/drop/rank/auth/profile còn fixture. Không gate K5/K6a/LAN/Internet mới đóng.


<a id="pickup-tick"></a>
### Nguồn và lifetime của t trong124 — CP055

**kr_static/kr_runtime confirmed trong scope Wine copy**, [recipe/hash/giới hạn](../evidence/pickup-time/README.md).
Tái dùng VN time/session producer làm hướng truy nguồn và cached KR writer, không chuyển địa chỉ/đơn vị VN.
`t` là **bộ đếm cập nhật logic riêng của CMainSystem**, chốt khi tạo pending record; không là đồng hồ
server hoặc số mili giây tính từ đầu trận. Physical timebase/đơn vị của nguồn thời gian vẫn unknown.

Native `E2C450` gọi `E2F2D0→60FDE0(global19D9920)→12580B0(+100)→123DD10(+10)→65A340→65A5D0`;
protected member **CMainSystem+110**. RTTI cùng hash runtime: vtable179234C, COL18076F4,
type descriptor19D089C, `.?AVCMainSystem@@`. `12580B0` đã có trong cache bootstrap cũ; CP055 nối nó
với124. Getter khi globalnull trả0. Constructor `E36F20` ghi u32t vào record+1C, k vào+20;
writer `E2E390/E2E583` đọc lại; actual TCP124 giữ cùng giá trị. Temporary stack record được copy vào
vector heap, không đòi pointer giống nhau. Gọi collision lặp không nhất thiết tạo thêm record.

Setter stack runtime phân biệt tăng thực (`D54743←123DCAB←1256B3B`) với periodic getter rekey
(`65A734`, giữ giá trị). Native `123DC20`: unsigned delta chia30, số bước xử lý kẹp15,
flag timer+20 có thể kẹp1; counter timer+10 cộng số bước đã xử lý, reference timer+4 tiến30×quotient
**chưa kẹp**.17 ca emulation kiểm arithmetic/wrap với time source/protected storage/callback model;
không chứng minh full scheduler hoặc30ms/33Hz. Counter/rekey globals chỉ được model/reset trong
emulation, không sửa runtime. Pointer timebase vào Wine kernelbase chưa khớp exact export.

| Run (hai vòng liên tiếp) | actual A124 t/k vòng1 | vòng2 |
|---|---|---|
| clock-trace |2215/12345|4365/12346|
| clock-late (input đầu4→12s) |2558/12345|4698/12346|

Trong từng run/role, cùng CMainSystem tiếp tục qua Start vòng2; A/B có counter khác nhau tại
Start/clock tương ứng. Chỉ chứng minh lifetime qua hai vòng trong cùng process, chưa logout/reconnect
hoặc mọi reset.45 snapshot native getter,7 constructor cases,4 chuỗi init→record→writer→TCP;
context của snapshot là tag **trước đó**, không đếm tag context lặp thành writer mới.

Service chỉ log `client_tick` cùng request; không thêm ngưỡng thời gian/tự nhận auth hay chống replay.
Hai run dùng source CP054 lưu `runtime-source/` **trước** thay đổi logging; dispatcher bench với
bốn actual frame kiểm code mới, broadcast sockets stubbed, không gọi bench là runtime mới.
[Guard k](#pickup-round-key) giữ nguyên. Original-server validator/time policy, distance/actorstate,
forged-current-key và TCP replay admission còn mở; bước kiểm ở [RESEARCH](../RESEARCH.md#gap-floor-pickup).

<a id="pickup-report-state"></a>
### Guard124 theo report nhốt/chết của fixture — CP056

[Recipe/hash/control/giới hạn](../evidence/pickup-state/README.md); nguồn `kr_runtime/confirmed` cho
consequence trên Wine copy, **`design`** cho policy service. Tái dùng124/165,17/19→170, securestate/
Roller và [t/key](#pickup-tick); không RE lại login/item IDs/clock hoặc mang layout VN sang KR.

**Native tĩnh có mục tiêu:** caller `B77300` (subobject actor+584) tìm floor item tại ô actor rồi
kiểm `B49FE0` (protected actor+618) trước `B7B440→E2C450`. `B7B440` entry `B323D0` đọc **item+1C**,
`B8C970` kiểm item properties; vf390 trên vtable actor1775B40 là `55FE90` trảtrue.
Không gọi các guard này là enum sống/chết của actor. Runtime alive có +618=0; chưa nối field này với
nhốt/chết hoặc toàn eligibility upstream. `B40F30` tại mainvf108(-1) xét các khoảng27..30/39..42;
chưa chứng minh toàn update/state rule. Static bytes8body chính khớp runtimecopy; cạnh vf108 bổ sung
được pin riêng. Candidate khác EE5A30/1124C40 không mở rộng thành contract mọi class/mode.

**Control `state-control2`:** giữ item17/Roller ở4,3; thêm spare18/19 ở8,7/8,8, ngoài đường input/bom.
ActualA124 nhặt17→165→Roller5→6. Sau **actual127/17**, securestateA56: chèn request từ body CP055
nhưng **đổi index/hàng/cột và k hiện tại**, giữ t2215, tại production dispatcher. Service CP055 nhận
index18/phát165; native callback12/12/errors0 và Roller6→7 **cả A/B**, lặp cả hai vòng.
Sau actual19/state1, index19 cũng được nhận/phát165; Roller đã max7 nên7→7, **không** có proof tăng stat
ở actor chết. Không coi đây là gói mới do native writer phát hoặc replay qua TCP; request chèn cách xa
actor cũng chưa tách riêng distance khỏi eligibility. Chỉ chứng minh service dựa key/catalog vẫn nhận nó.

**Policy mới:** sau report17 hợp lệ trong layout fixture hoặc19 từ slot khớp connection, service ghi
`pickup_ineligible` cùng generation/report SHA/sequence. Sau [guard k](#pickup-round-key), trước claim/
consume,124 từ connection đó bị `actor_report_ineligible`, không165/không chiếm item. Marker không phụ
thuộc có peer để forward; Start kế xóa nó. Đây là report-derived/conservative fixture policy, không
state authority/auth hoặc full original validator. Fixture Roller hiện chưa hỗ trợ rescue/self-escape;
không áp marker chỉ xóa ở Start sang service kim/cứu mà chưa nối report1/18 và kiểm transition.

**`state-guard`:** cùng bốn injected frame/SHA/keys12345/12346, cả bốn bị loại; không callback165 cho
spare18/19. Input vòng1 không tạo actual124 (nguyên nhân chưa gán cho guard); vòng2 có **actualTCP124**
index17/key12346, native165 full12 và Roller5→6 cả A/B, chứng minh reset marker trong scope này.
Hai run đều2Start/clock/175outer302/180full3, stateA1 trước175 cả hai view. Không hai trận nghiệm thu.

Wrapper measurementv1 bị outer127 ghi đè provenance label của injection; final verifier nối **exact
injection SHA/fields/trigger**,2 actual124 ở control/1 ở fixed vòng2; raw không sửa. V1 source đã archive;
v2 chỉ sửa setdefault label, chưa runtime.5regression + actual-capture dispatcher bench kiểm peer còn
claim được item bị A từ chối, keyguard trước actor và Start/reset. Không dùng số test đóng gate.
Distance/auth/transport/malformed/stale127 hoặc report giả, rescue/self-escape/general eligibility vẫn mở.

<a id="item-effects"></a>
### Bomb/Fire: stat, giới hạn và nguồn power — CP031

**kr_runtime/kr_static confirmed trong phạm vi diagnostic copy Wine**;
[control/native/snapshot/hash và audit](../evidence/item-effects/README.md). CP030 mới kiểm layout/pickup;
CP031 theo cạnh đang chạy, không thay binary hoặc ép stat bằng debugger.

**Đường thực thi:**165→`F73D30→B52840→E14DA0→1190A40`; custom-effect getter trả0,
đi `1192700`. Group1 chọn `11927AE`, gọi `1191840` bốn effect number. Bomb/kind0 có effect1,
Fire/kind1 có effect2; ba số còn lại là0 ở hai loại này. Receiver runtime là actor+58C,
vtable17760BC. Effect1→v4/B47180→context actor+4A0/vA4/**AF06F0**;
effect2→v8/B47210→cùng context/vA0/**AF0690**. Không lấy interface+588 của165 làm receiver stat.

| Stat secure của actor | Offset wrapper | Phạm vi đã đo |
|---|---|---|
| Capacity hiện tại / max |+4A4 / +50C |Slot0 ban đầu1/10, sau Bomb2/10; slot1 giữ2/7 |
| Fire/power hiện tại / max |+4BC / +524 |Slot1 ban đầu1/9, sau Fire2/9; slot0 giữ1/7 |

Hai client chụp cùng giá trị trước/sau từng165 và lúc clock bắt đầu; checksum giải độc lập rồi gọi
native `529C60→52A350` trên wrapper/data đã capture. Chỉ profile hai slot hiện tại; chưa nối tên
asset/character tới mọi base/max. Đây là phạm vi CP031; Roller/MaxPower được bổ sung tại
[CP045](#item-families), reset stat hẹp ở [CP032](#hidden-items). Shoe và toàn reset vẫn chưa đo.

**Phép tăng/kẹp native:** AF06F0 dùng context+4/+6C; AF0690 dùng+1C/+84.
`636620` đọc secure current rồi cộng1/ghi bằng52A860; sau đó so signed current với max.
Nếu vượt, `54B610` chép max về current và trả0; trong giới hạn trả1. Mười ca emulation chạy code
increment/getter/setter/clamp/RNG thật với context/box storage tổng hợp, gồm tại max và current>max.
Counters/callback định kỳ của instrumentation được khởi tạo0; chưa mô phỏng rollover các counters.
Đây là **emulation** của nhánh kẹp; runtime mới kiểm pickup dưới max, chưa ca nhặt dư tại max.

**Control hành vi:** cùng layout hai item và ba lần bấm Space xen Right cho mỗi slot, fuse thử30s
để mọi input kết thúc trước158 đầu. Không165: A có một118, B có hai; có165: A có hai, B vẫn hai.
Mọi bom có actual118/native replay và159 trả entity ở cả hai. Không lấy việc thiếu request thứ ba
riêng lẻ làm proof trần; snapshot stat và nhánh kẹp là các bằng chứng riêng. Tại B, field10 của118
tăng1→2 sau Fire. Không suy hai actor có cùng capacity ban đầu từ cùng recipe.

**165 gửi lặp:** sau mỗi positive165 khoảng1s, gửi lại cùng payload bằng seq receiver mới liên tục.
Cả hai vẫn đọc12/12/errors0 nhưng stat không tăng lần hai; mỗi loại chỉ đi increment một lần.
`E14DA0` tra item trước effect; trong nhánh item không còn, trả sớm. Ca đo giới hạn hai index đã bị
xóa trong cùng round, không chứng minh mọi stale/round-generation/reuse index đều an toàn.

**Correction power118→159:** mapping cũ dùng field6 của118 như power chỉ vì cả field6 vàfield10
đều1 trước pickup. Fire phân biệt: field6 vẫn1, **field10/u8 tại payload+16, nguồn record+18**, lên2.
Writer `E2E1D1` serialize record+18; không dùng record+30 (nguồn field6) thay nó.
Harness `capacity-power` đổi nguồn **118.field10→159.field8/u8 tại payload+18**;
owner/serial/hàng/cột/ID giữ cách ghép đã kiểm. Native đọc packet độc lập, và `E9C58C` trên cả hai
client đọc power2 cho bom B tại6/3,6/4; bom A tại3/3,3/4 giữ1. Trước correction chỉ có proof wire
đang lấy nhầm field, chưa probe calc của peer để gán toàn bộ ảnh hưởng runtime cũ.
Raw/checkpoint/harness CP024–030 và các run mapping cũ giữ nguyên làm lịch sử/control; không sửa hash cũ.

Emulator vẫn chọn placement/index, fuse30s, broadcast, timing/duplicate và outcome. Nhánh tăng stat
và clamp là code client đã chạy, không tự phục hồi authority server gốc. Item ẩn/phá block/reveal,
runtime nhặt tại max, speed, validator/disconnect/full auth/cache/codec còn mở;
reset stat/item được kiểm tiếp hẹp ở [CP032](#hidden-items), chưa toàn lifecycle round;
**chưa full match/K5/K6a/LAN/Internet**.

<a id="hidden-items"></a>
### Gạch, item ẩn và khởi tạo lại vòng — CP032

**kr_static/kr_runtime confirmed trên diagnostic copy Wine**, không patch thêm hoặc ép stat/state.
[Recipe, controls, native, snapshot và hash](../evidence/hidden-items/README.md).
VN chỉ hướng dẫn thử list2 trước start và theo dõi report vỡ gạch; schema/consumer KR được kiểm riêng.

**List2 của582:** sau giải blob có counts `[0,2,0]`, hai record
`index27/row2/col3/kind0` và `index28/row3/col2/kind1`;26byte inner được đọc đủ.
`D824D0→1245700` lưu list2 ở context+3E8; getter `1245780` được gọi từ `EA46E2`, rồi
`EA46F8→EB56E0` duyệt list2 trước list1. Với hai ô trên ExVillage01/code278, lookup đầu
`E9FCF0` trả tile, hai lookup thay thế trả0. `B9C8D0` qua guard `11902B0`, ghi kind vào
tile+B0 và index vào tile+AC. Cả A/B đo đúng tọa độ/index/kind, vtable1747724;
RTTI static khớp `CGOTileFixedBox`. Không suy mọi loại block/map đều chấp nhận list2.

**Control và reveal:** cùng input A đặt bom3/3 rồi rời ô, quay lại hai ô gạch; B giữ ở spawn.
Control không phát158: bom/gạch còn nguyên, không124/142/165 hoặc hit hàm tăng stat đã probe trong cửa sổ đo.
Ca có158/kind0 sau fuse thử6s: hai gạch biến mất và hai item hiện trên cả hai view;
A thực gửi124/index27 rồi28, stub xác nhận165 tới cả hai. Snapshot secure/native xác nhận
capacity và Fire của slot0 lần lượt1→2 ở cả hai; các max và stat slot1 đã đo giữ nguyên.
Không cần chuyển tiếp report142 để hai view reveal trong recipe này; chưa có phép tắt UDP riêng.

**Đường phá/reveal có mục tiêu:** `CGOTileFixedBox/B9CEC0` có nhánh giảm secure durability,
ẩn tile và lấy object nền `E9F360`; `118FBF0(index)`/`B8CC50(kind)` kiểm trước cạnh
`B9D0B4→B8B5B0(game,index,kind,...)`. Runtime đo hai destination vtable1746A4C/RTTI `CGOItem`;
pointer từng index khớp đúng item tìm thấy sau đó trong165 ở cả hai client.
Không dùng entry `E9DFC0` của đường đặt item nền làm proof reveal: probe entry đó không hit trong ca list2.

**C→S142**, writer `E72720`, construct tại `E727FF`, serialize `E7281B/28/35/41`:
`u8 row,u8 column,u8 selectedSlot,u32 aux`;7byte payload, khác VN135/u16 đuôi.
Hai client đều report cùng ô2/3 và3/2 nhưng selectedSlot lần lượt0/1, không phải luôn bomb owner0.
Hàm chọn slot bằng guard/local-slot registry; aux được caller `B9D11C` truyền−1 trong ca này.
Native framer/getters kiểm actual frame theo seq/socket; chưa generalize aux hoặc validator/server reply.

**Vòng kế:** run `hidden-rounds` có hai121/clock/layout trên cùng tám connection; hai tile/index
được gắn lại, cùng hai124/165 mỗi vòng. Snapshot đầu vòng2 ở cả hai đã trở lại capacity1/Fire1
cho slot0; sau nhặt lại lên2/2. Serial bomA lặp0/28 ở mỗi vòng, ID stub chọn1–4;
power request/broadcast là1 trước Fire,2 sau Fire. Đây là reset của các field/đối tượng đã đo,
không phải chứng minh toàn entity/timer/peer/seq không rò.
Run này chỉ có175/180 vòng1: harness kế thừa policy chờ B reported19 vòng2 trong khi input điều khiển A;
actual A17/19 vòng2 vẫn capture. Run sửa policy phải được đối chiếu riêng, không gọi ca này hai vòng hoàn tất.
`hidden-rounds-fixed` chỉ đổi điều kiện origin của policy thành A/A: hai reported17/19 của A dẫn tới
170 gửi B,175/180 mỗi client hai lần, cùng tám connection/seq liên tục và ảnh kết quả/phòng vòng2.
Hai lần layout/reveal/nhặt, stat reset và serial0/28/power1→2 vẫn khớp. Không coi sửa policy stub
là sửa bug client hoặc chứng minh mọi reset/validator đã đạt.

Placement/index/fuse6s, giữ một ACK/index/round, broadcast, rank và timing vẫn **emu_choice/design**.
Chưa phục hồi thuật toán random drop/server authority; list3, block variants, chain/rescue, runtime tại max/
speed, stale/generation/disconnect/full auth/cache/codec và K5/K6a/LAN/Internet còn mở.

<a id="current-hidden-items"></a>
### Item ẩn trong currentservice — CP068

[Recipe/hash/partial/retry/audit](../evidence/current-hidden-items/README.md). **kr_runtime/confirmed**
cho integration Winecopy, **design** cho catalog/reportgate. Tái dùng CP032exact582/counts0/2/0/
FixedBox/revealconsumer/142/124165/nativeeffects/knownUI; không thêm thân hàm native/patch hoặc RE lại
receiver/IDs/clock. Currentprofile giữ distinctspawn/cleanauth/cache/key+serial/reportguards.

`hidden_items` là catalog optional riêng với floor`items`, merged vào Room claimcatalog; duplicateindex
fail.124 kiểmcurrentkey rồi hiddeneligibility trướcclaim;142 có7byte đã phục hồi, kiểmboundselectedslot,
auxFFFFFFFF và knowncell. Sau ít nhấtmộtcurrent158 đã gửi, report mởindex tươngứng, duplicateidempotent,
Startreset. Không forward142, không servercollision/replayauth; old/forged142saucurrent158 vẫn ngoài
policy, currentblast bất kỳ không chứng minh ô báo đãphá. Floorprofilekhônghidden giữbehavior.

Run đầu2cyclesreveal/result chỉ1pickup27vòng2, khôngfulleffects. Retry normalUIroute có haiactual27/28
mỗi vòng,142 từhairole trướcpickup. Nativebothviews FixedBox(tile,index,kind)→revealcommit CGOItem→
165lookupsamepointer; capacity/FireA1/1→2/1→2/2, reset1/1vòng2;4actual118/159158/power1→2/serial0,28,
Astate1bothbefore175/hairesult/return/refresh/errors0. Không dùng cursor/ảnh hoặc runpartial thayproof.
Nativeframer/blob/11ED590/gettermodel tách khỏi runtime; counts0/2/0/inner26 kiểm, chưa mọilist2/mode.

Exporter `--item-layout hidden` giữ exactCP032fixture27/28 vàcatalog;defaultfloor giữ,41alias chỉfloor
khác. Đây là deterministicfixtureplacement, không thuật toán drop gốc, không tựmigrate profilecũ.
72serverregressions kiểmguardnegative/reset/forgerylimit; actualcapturebench khôngruntime mới.
Chưafullvalidator/reset/auth/K5/K6a/LAN/Internet. Nghiệm thu profile dùnghànhvi nào vẫn theo PLAN/gates.

<a id="item-generation"></a>
### Xác nhận item cũ khi tái dùng index qua vòng — CP042

**kr_runtime/kr_static confirmed hẹp trên diagnostic copy Wine**;
[recipe/control/hash](../evidence/item-generation/README.md). Không thêm patch hoặc ép actor/stat.
Theo [item165](#item-pickup)/[stat Bomb](#item-effects), đặt một Bomb/kind0 tại hàng4/cột3
mỗi vòng, index17 vòng1; vòng2 đổi18 hoặc giữ17. Hai client A/B giữ8connections/IDs1,2/slots0,1,
hai actual95/104/121/99/clock. Mỗi vòng một bom A rồi17/19,170 đồng bộ peerstate1 trước175/180/phòng.
Placement/index/outcome/fuse6s là **emu_choice/design**.

Sau clock vòng2, trước input/actual124 mới, stub mã hóa lại payload165 xác nhận vòng1 bằng seq
hiện tại của từng receiver. Payload12byte là `ok1,ctx0,slot0,row4,col3,index17,kind0,flag0`.
Đây là semantic replay từ stub, không replay nguyên frame vượt CRC/seq và không request client giả.

| Ca | Quan sát ở cả hai client |
|---|---|
| `unique-index`: vòng2 index18 |165 cũ đọc12/12/errors0, item17 lookup0, capacityA giữ1→1, không increment. Sau A Down có actual124/index18;165 hiện tại làm capacity1→2. |
| `reused-index`: vòng2 index17 |165 cũ tìm item17 hiện tại, chạy effect/increment, capacityA1→2 trước input nhặt. Toàn capture không có124 vòng2; chỉ có124 vòng1. |
| `guarded-fixed`: vòng2 index17, claim policy local |Ticket xác nhận cũ bị loại `stale_round`, không phát165. Sau input có actual124/index17,165 hiện tại đọc12/12 và capacityA1→2 cả hai; hai vòng vẫn kết quả/về phòng. |

Stat là secure actor+4A4 trước/sau từng callback, checksum và native52A350 kiểm độc lập;
max/power/capacityB đã đo giữ nguyên. Clock vòng2 reset capacityA về1 trước replay.
Không dùng riêng full cursor, icon biến mất hoặc log stub làm bằng chứng cộng stat.

**Cạnh đã kiểm:** E63260 đọc packet, E63331→E2C660 thử xóa pending theo ctx/index,
sau đó mới kiểm ok/context và actor. Với variant đã đo, việc không có124 mới không chặn effect
khi item hiện tại khớp. Interface→F73D30/B52840/E14DA0 tìm item bằng index; E636D9 kiểm pointer,
E63712…E6374F kiểm hàng/cột và E637F9 qua nhánh match. Probe effect có thể xảy ra trước nhãn
FOUND/MATCHED của outer handler do interface gọi lồng; không dùng thứ tự tên probe thay call graph.
Chưa suy mọi variant đều bỏ qua pending hoặc mọi tọa độ/flag/kind sẽ được chấp nhận.
Lượt `guarded-index` trước bản fixed lỗi collector `claimed_items` trước165, không phải client/protocol
từ chối; giữ raw và harness lỗi như lịch sử tại evidence, không gộp thành ca runtime thành công.

**Policy emulator:** [ItemClaims](../../server/item_claims.py) kiểm catalog hiện tại cùng slot gắn socket,
context/index/hàng/cột/kind, giữ một winner và ticket xác nhận local gắn owner/generation.
`begin/finish` vô hiệu hóa ticket cũ; `consume` chỉ một lần dưới owner/lock trước phát165.
Đây là design cho job/xác nhận local chưa gửi, không field epoch phục hồi từ wire.
Request124 cũ trùng toàn bộ catalog hiện tại vẫn không thể được module phân biệt chỉ bằng các field đó;
distance/tick/session/actor-state, pending159, timer abort và UDP/peerqueue còn cần kiểm riêng.
Không gọi module này là toàn validator hoặc nghiệm thu K5/K6a/LAN/Internet.
[Bước tiếp](../RESEARCH.md#q-023).

<a id="needle-service"></a>
### Kim145 trong current service: credit và report-pair opt-in — CP113

[Metadata/raw/control/hash](../evidence/needle-service/README.md); [gap](../RESEARCH.md#gap-needle).
**design/implementation** trênmodeltrước, **kr_runtime/confirmed hẹp** sau currentcopyrun; không
servergốc/inventoryauth. CP037038nguồnnative táidùng trực tiếp, khôngnativebody/codec mới hoặc portVN117.

Profile có `needle_escape: {"mode":"needle_145_report_pair"}` mới bật. Chỉ đúngmột catalogkind145
(kểcảhiddenlist), Start đòihai live mappedA/B. Khôngoption giữdefault, không tựbật theo tênitem.
Accepted124 đúngcurrentclaim/key/catalog/boundmember, hai165sendxong mớicredit1 cho boundconnection/
currentgeneration. Dedupeclaim cũ ngăncreditlặp. Credit không làaccountinventory hoặc locationvalidation.

Actual1 chỉcommonprefix14; actual0 đúng22 với typedtail widths1/2/1/4=`1,145,0,0`.
Sau freshnessread-only, admission đòicurrenttrap17/noaccepted19/credit1 và đúngliveboundmember.
1 chỉstagecandidate cógeneration/trapSHA/report1SHA/seq/prefix6/tick; khôngconsume hoặcforward.
0 phảikhớpcandidate trap/slot/XY/logicaltick; haioldcontrolsCP037038cùngtick nhưng rulepairing là
designvariant, khôngguaranteemọi mode. Actor_field5→6 không đòikhớp. Service gửi170/sub1 exactprefix6
của1tớipeer, rồi mớicommitcursor0/credit1→0/clear17/pending. No170/sub0 guessed.

Thiếukim/sai_shape/cặp/tick/XY/crossconnection/dead khôngforward/unblock/consume hoặcthaycursor.
Accepted17/19mới hủypending; accepted19history khôngbị17sauđóche. Start/acceptedleave/EOF xóastate
kim của member (survivor trongroundabort khôngadmitkhiinactive); connectionreportcursor giữCP063.
Sendfailure chưa consume/clear/cursorcommit, nhưngrealserviceexception vẫnclose/abort; directmodelretry
không lànetworkrecoveryproof. Rescue18 vàreport21 không tựclearinventory/pickupmarker.

Runtime199,834777498s/SHA6074f0ea… trêncopyWine/profile47+CP037needlefloor/index41/AB/BchiefAguest:
actual124/165→17→1→0/tick3220/slot0XY120309, report1frontseq274/service280 vàreport0front277/service283
khácnhauvìTCPproxy replay; only170/sub1Bseq112/full6. A local56→26/Bpeer56→26/reader6/6/lỗi0;
actualnative afterRightA/B XY200309, ôkimtrống vàsecondCtrlkhông1/0 mới. Ledger source/design logs1credit/
1consume, defaultoff/onactualoldpairmodels36joins/175server tests. Nativegetters162+12/typed108models
khácactualcallbacks. Noresult/secondStart/depletedsecondtrap trênnewservice trongrun này.

Giới hạn lịch sử CP113: run này chưa có Start2/depleted. Phần đó được đo tiếp ở
[CP114](#needle-round-reset), rescue A/aux1 ở[CP115](life-results.md#rescue-service); multi-charge vẫn unknown.
Fullinventory/forgedfirstseen/location/authority/auth/fullmatch/K5/K6a/vanilla/LANInternet vẫn mở.


<a id="needle-round-reset"></a>
### Kim cạn và freshcredit sau Start2 trên current service — CP114

**kr_runtime/confirmed hẹp**, [metadata/raw/hash](../evidence/needle-service/README.md#cp114).
Không code/profile/fuse/client/probe mới: giữ CP113option và floor145, reuse CP038depletedcontrol/
CP110BhostreturnedReadyAStartB. Chỉ normalOSinput thêmSpace/emptyCtrl và knownStart2.

Run339,135766083s/SHA9356d26d11f03676d1ccdc388ccb06f317f2d0b4c9d381c2a3e82893616825db:
R1actual124145→17→1→0/credit1→0/peer1701/nativeA,B56→26. SauRight/Space thứhai A nhận17,
Ctrlrỗng không1/0 mới, rồi **actual19** trước chosenrank175/return180/585/110/fullnativeerrors0.
Khôngsuytimer/gamephysics law từkhoảngchờCtrl hoặc giả19do service tạo.

CùngTCP/IDslot/Bchief/Aguest, actualAReady95/BStart104→R2native121129/clock/currentkey12346.
Old124bodykey12345 replay bịloại trướccredit/165. Fresh124index41/kind145/key12346 đượcclaim/lạihai165;
A nativeCtrl×1, report17→1→0 mới/cặpđúngXYtick, ledgercredit1→0 lần2 vàpeer1701/nativeA,B56→26.
Duplicate124hai vòng khôngcredit thêm. Native4lầnescape-after26/hairole×hairound, localpairsource/nativegetters
/cursor/hashvàảnh tách ledger; modelStartclear không tựlàoracleclient.

Chỉ **một result/return hoàn tất**, R2 dừngactive sauescape. HaiStart/hai lần dùngkim khác hai trậnđãkếtthúc;
không nâng K5/K6a/fullmatch/resetmọibiếnthể. Baactualbomb/3complete158/view; reportsequenceR1
17,1,0,17,19 và R2 17,1,0. 2.082joins gồm1.977rawhash, copiednativefields408/typed264models riêng.
Nguồn CP113oneescape giữnguyên; oldCP038depletedvà CP114currentreset khácscope rõ.

Bước kế tại thời điểm CP114 là rescue18/profile; đã được giải trong scope fixed A/aux1 ở
[CP115](life-results.md#rescue-service), chiều B/aux0 ở[CP116](life-results.md#rescue-reverse). Thứ tự hiện hành chỉ ở PLAN.


<a id="direct-item-feedback"></a>
### Fire xác nhận trước input kế và bom hai owner trên directservice — CP150

**`kr_runtime/confirmed` hẹp**, [recipe/control/raw/hash](../evidence/peer-endpoints/README.md#cp150).
Reuse CP082freshfeedback/CP106native165/currentstats/CP149allTCPdirect và map/item KR đã biết;
không tìm lại nativebody/opcode/codec. Productioncode/profile/timeout30s/actorbinary giữ nguyên.
Chỉ controllercopy kiểm currentactual124index17/kind1 +native165 ởcảhai view trước khi đặt bom;
nếu chưa có, dùng boundednormalOSfeedback, không fake124/165/gamefunction/writeactor.
Giữ20pulse/.045s/3sneutral và5sdeliverywait; stopkhi nativepickupđãcó, không tăng hạn đểpass.

Hai run được giữ riêng. R1partial199.741s có actualFire124/165/effect trước extraalignment,
nhưng đòi tọa độ tâmY279/tolerance4 rồi20pulse dao động259..319 gâyalignmiss; chưaSpace/kếtquả.
Đây là thất bại điều kiện kết thúc phép đo, khônglàFirefail. R2newcopy/freeze364.479s/exit0/
1348raw hoàn tất: cảhai vòng A nhặtBomb16+Fire17,165cursor12/errors0 ởhai view,
copygettercapacity/power`1/1→2/1→2/2`; clockvòng2 vềbaseline. R1Fiređãcótrướchelper,
R2alignment dừngtrên actual124+hai native165; không đòi đứng đúngpixel sau khi đãnhặt.
Không suy đã giải nguyênnhân CP149R1miss hoặc hiệu ứng timing/proxy/packetloss.

Vòng1nativeA118/159owner0/ID1/3,3/power2; vòng2nativeB118/159owner1/ID2/9,3/power1
(BkhôngnhặtFire). Native158/process/195flags ởhai view khớp power/geometrythực; currentA17→19/
B21 vòng1, B17→19/A21 vòng2. Native175/180 decodedrank2,1→1,2, resultreset hai lần,
finalReady95/native110/noStart3. **664vethPCAPrecords**,4peerwindows/currentkey/target/currentprogress,
tấtcả8TCPdirectservicepeerA.2/B.3/noTCPproxy/UDPproxy. Render4ảnh sau reset/Ready đãxem.

3537lifecycle/1490wire/499peer/70pickup/16inputgate/25Ready/55directTCP joins;
models1668actor/324peergetter/2364flaggetter+2340array/294typedpacket/297report/12native153/
42UDPCRC táchruntime. Hai loadingbarrier27.376/27.838s<30; khôngcoldstartstabilityclaim.
Currentcredit/claims/result vẫn emulatorpolicy/authoritylimits, không luật server gốc/toàninventory.
CP150 khôngxóa CP149partial/CP150R1failure, chưafullmatch/K6a/realLAN/Internet/8clientsthật.
[PLAN](../PLAN.md#current-order) tiếp acceptance còn thiếu cho hai client trực tiếp, ưu tiên actualcontested
pickup theoCP030/062 với claimlogicgiữ và boundedmeasurementdelivery, không RElại 124/165.


<a id="direct-contested-pickup"></a>
### Hai nativeclaim cùngFire17 trên distinctspawn/directroute — CP151 partial

**`kr_runtime/confirmed` R1**, [recipe/raw/partial](../evidence/peer-endpoints/README.md#cp151).
Reuse CP030/062actualtwo124 vàpositive165delay2s, CP150currentdistinctspawn/profile/claims/directroute;
không mang overlapspawn/Roller củaCP062 sangKRcurrentprofile. Wrapper`runtime_hold.py` sao nguyên
CP062: chỉ delaypositive165, productionclaim/key/consume nguyên, provenance là **design measurement**.
NormalOSfreshfeedback approachA199/B359 rồi DownA/UpB0.30s và release; khôngfake124/forcewinner.

R1/key12345 cóhai actual124 cùngindex17/kind1/cell6,3: A tới trướcđượcnhận, B bị
`claim_not_current_or_already_consumed`. Hold2.032375181s; Brequest tới trướcdelivery1.692042763s.
Mỗi receiverchỉmộtFire165/full12/errors0/slot0, nativegetterA power1→2/Bpower1giữ,
capacityA/B2giữ;45contestjoins. R1cóhai peerwindows/A118ID1power2/158/195/currentA17→19/B21/
native175rank2,1/180/110reset và Start2. Đây **không** là luật fairness/arbitrationserver gốc.

**Blocker:** R2cảhai nhận121full129 vàinit2actor, nhưngkhôngactual99 trênTCPchanneltrướchạn30s;
serviceđóngcảhai vìloading_timeout. KhôngnativeclockR2/contestR2/endingR2/finalReady2.
Serviceghi forfeit rank/return/refresh R2 với **broadcastrỗng**, khôngphải kếtquảnative lần2.
Harness exit0/`virtual_two_returns_native_observer_timeout` chỉnói cleanupxong; không2cyclespass.
Nguyênnhân thiếu99 chưa xácđịnh; khôngquy delay165/networkloss/clientdeadlock từcorrelation.

Run353.595s/1204raw/407vethrecords;905partialjoins/298peerjoins/2R1windows;
models1176actor/192peergetter/1182flaggetter+1170array/171typedpacket/174report/6native153/22UDPCRC
riêng. Initialverifier thiếucontext165 nên trùng snapshot; sửajoinđúngcallback/context, raw giữ.
Initialpartialchecker giảR2khôngrank bịbácbỏ bởi emptyforfeit; raw/model/policy/nativereceipt phânbiệt.
Source/config/binary khôngđổi giữarun;30sdeadlinegiữ, capture/process/namespacecleanup/hashđầyđủ.

[PLAN](../PLAN.md#current-order) tiếp trace nativewriter99 đãbiết`E7F2A0`/guardstage+6410 và
actualsend`13ADD10` với monotonic timestamp từ121 tới deadline, khôngraise timeout/force99/clock.
CP150hai cycles/BombFire/ABowner vẫnnguồnriêng; CP151R1khôngbackfillR2/gate/fullmatch/LAN/8clients.


<a id="current-basic-hidden"></a>
### Basicprofile ghép trên currentmode/Bhost: native reveal và ba hiệu ứng — CP169

**`kr_runtime/confirmed` hẹp cho copyprofile thử; placement/reportgate/controller là `design`.**
[Raw/partials/hash](../evidence/peer-endpoints/README.md#cp169), [gap](../RESEARCH.md#gap-floor-pickup).
Reuse CP032/068/069 exact582/list2/FixedBox/reveal142/124165/effectfamily, CP095actualbox/currentterrain,
CP165boundednativefeedback vàCP166/168Manner/title/Bhost. VNitems/oc_e-game chỉworkflow/family đãpin;
không portopcode/ID/offset/enum hoặc RE lạiconsumer/ItemAttr/thân hàm native. Sourcebuild e9bb4766…/
static660447…/modOa08950…/base400000 giữ, khôngsourcebinary/pack/DBchange.

Copyprofile ghép từCP168current47pin: **46alias giữexact**, chỉfloor582 thayexactCP069counts1/2/0/
inner36; floorRollerindex17,row4col3,kind2; hiddenBomb27,row2col3,kind0/Fire28,row3col2,kind1.
items/hidden_items/item_layoutcatalogtươngứng, mode/title/terrain/network/capacity/guards khôngđổi.
Nativeframer/blob/11ED590 valid/CRC/partial3cases vàconstructorpass trướcWine; list2gắn vàoactualFixedBox,
khôngtạo tường mới. Committedpreset/default vẫnfloor16/17; không gọi profilecopy làpresetđãmigrate.

R1knownCP069OSroute chỉactual17+27; R2thêmfeedbackmissing28nhưngoldrouteđãnhặt28/bỏ27 nênchỉ17+28,
khôngchạyfallback. Cảhai view của từngrun đãnativeinstallFixedBox(vt1747724)/reveal27+28→CGOItem
(vt1746A4C), actual142bothroles; các effect cónguồnriêng, khônggộphai partial thành3effects.
R3freshcopy7connections/Aroom/Blogin3-nochannel4 tới330sbound/nohiddenruntime; exit0/khôngerror
khôngacceptance, không gánstartupfailurechohidden/membership hoặcbypassauth. R4freshprefixcùnglogic
retry/R3nguồn giữ; nonewnativebody/handlerfix. R2generatorindentfallback sửa **trướclaunch**/refreeze,
candidatecontrollercũgiữlocal, khôngsource/configchangegiữarun.

CùngR4 192.983476401s/exit0: actualMannercreate/Bhostpromotion/Arejoin/ReadyAStartB/12199clock1;
normalSpace/158 pháactual2Box→native27/28reveal/4actual142→actualA12417,27,28/currentkey12345→
native165full12/errors0/3callbacks/view/lookup27,28 đúngpointerreveal. Nativegetterbothviews chỉA tăng
Roller5→6/capacity1→2/Fire1→2, Bcapacity2/power1/Roller4 giữ; đây là familyeffect đãRE, khôngmớiluậtitem.
Missing28 dùngpeer_alignread-onlyArowY159/X80,tolerance12/20attempt/3freshneutral/45ms, giữ3rare165+
lookupE636D9/statprobes, exactrestore4viewphaseacks. Only2alignmentgroupshit; missing27fallbackkhônghit
trongR4, không giảmọinhánhcontroller đãkiểm.

176integrationjoins/122effectjoins/60capturednativeframer/gettermodels; pointer/cell/typedreportSHA và
securegetters là bằngchứng, ảnhfinal2view chỉrender. 721raw/109frozen/namespacecleanupsealed.
Chỉepoch1/oneblast/3effects, **chưa normaloutcome/reset/epoch2** trênprofileghép. Originalrandomdrop/
distance/auth/hiddenreportfreshness/validator giữgiớihạnCP068; khônglấy142 làmworldoracle.
Next[PLAN](../PLAN.md#current-order)ghépknownfinalSpace/result/reset/twoepochs, khôngRElạinativeconsumer.
K4dependencies/K5/K6a/fullmatch/Windows/LAN/Internet/8client giữmở; auditAUD050qua168/next172.


<a id="hidden-cell-admission-current"></a>
### Hidden142: ràng buộc ô đã phá trong bounded terrain — CP172

**Guard là `emu_choice/design`, native compatibility là `kr_runtime/confirmed` hẹp.**
[Recipe/raw/hash](../evidence/peer-endpoints/README.md#cp172). Source-first CP068 weak report policy,
CP069/170/171 basic/native195 và `terrain_blast.cleared`/commit có sẵn; không mới native body/opcode,
không suy hành vi từ asset/VNlayout. Trước CP172, known142 chỉ cần có anycurrent158, kể cả blast
không phá ô được báo; test cũ giữ giới hạn này cho profile không có terrain.

Khi profile có pinned terrain, sau slot/aux/knowncell/anycurrentblast guard, service đòi một bomb
đúng generation có `terrain_blast.generation` hiện tại, `cleared` chứa đúng ô, commit>0 trước report,
và receipt158 đúng toàn bộ roster. Metadata terrain chỉ được commit sau broadcast thành công;
pending/partial/oldgeneration/unrelated cell không mở eligibility. Report hợp lệ lưu `terrain_clearance`
source design/generation/bombid/commit; duplicate vẫn setdefault/idempotent,124/165 không đổi.
Không terrain giữ lower bound lịch sử, không nâng mọi profile thành validated world.

Năm test mới kiểm unrelated/pending→positive, two-peer duplicate/two cells/onepickup, reset/old
generation, partialbroadcast và roster/commit order;270server tests đạt. Đây là policy tests, không
native oracle hoặc runtime negative attack controls. 142 vẫn không có recovered roundkey:
**cùng old bytes sau legitimate currentcellclear không phân biệt được**; auth/distance/fullfreshness mở.

Fresh modifiedWine/privatehosts/directproduction cùng exact basic export CP171/47pins, chỉ delta
service guard; source/config frozen→cleanup/seal. Run406,400966485s/exit0/1438raw/158frozen. Hai
native cycles/keys12345→12346, bốn A bombs IDs1..4/power1,2,1,2, ba pickup/effect mỗi epoch trênhai
view, native stat/hiddeneligibility/195 reset và56→1/other0 before175 giữ. Ba barriers
6,459173952/6,455755669/6,374629005s trước30s; normalBStart3/key12347/native12199/hai clocks thêm
9,229294342s/render hai view. Không thirdbomb/outcome; cleanupEOF sau completion0,409834981s.

16actual142 gồm **8accepted hidden-cell reports** (4already-revealed other-view duplicates),
8outsidecatalog reports bị `unknown_hidden_cell`, không phải 16accepted hoặc injected negatives.
Accepted đều gắn rootbomb1/3 đã clear ô trong currentepoch/fullroster/commitbeforereport; không mở
eligibility phụ/không thêm165effect.52admissionjoins,1736first-two và419Start3/stream/observerchecks;
180capturednative packet/552securefields,life23/16getters;4adaptivealignmentgroups/8viewphaseACK/
rare165lookup enabled/exactrestore. Initial counting assumption16accepted được giữ và sửa về8+8;
raw/runtime/guard/expectedvalues không đổi để pass. Không causalclaim về số report/input timing.

AUD051 qua172 đối chiếu sources/raw và sửa stale current status/next-step; next auditCP176.
Không K5/K6a/fullmatch/dependency/Windows/LAN/Internet/8client. [PLAN](../PLAN.md#current-order) tiếp
B118 và peer windows hai chiều trên đúng basic run; oldfloor proof giữ riêng. Generalphysics/drop/
mode/inventory/auth và old142saulegitclear không được giải bởi guard này.


CP172 cũng giữ được pin source lịch sử CP166 khi service thay đổi: checker xác minh đúng source
Python tại commit lần đầu công bố record v2, gồm exact declaration/path/size/SHA và externalmanifest
nếu có. CP166/source `81b0c8d3…` khớp commit `09759d0`; không sửa complete record/evidence. Raw/
binary/local không dùng fallback; source chỉ có v1 hoặc không khớp khi công bố v2 vẫn lỗi.34checker
schema/history tests đạt; đây là versioned provenance, không native semantics/gate. [Quy tắc](../evidence/checkpoints/README.md).


<!-- protocol-topic-content-end -->
