# Old159 khi serial local dùng lại — CP043

Baseline CP042/[PR50](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/50) merge
`ed47c9723d5951bb521fe99b96ec6f928685b29d`. CP043 snapshot trước tích hợp trên nhánh
`kre/late-bomb-confirmation`; PR của nhánh giữ trạng thái tích hợp.
Codex tự kiểm, review độc lập [waived](../checkpoints/review-waiver.json).
[Contract duy nhất](../../PROTOCOL.md#late-bomb-confirm), [bước tiếp/gate](../../RESEARCH.md#q-023).
AUD013/CP040 gần nhất; audit định kỳ kế CP044, không gate mới đóng.

Raw/recipe/harness/GDB/fixtures/log/capture/native reports/ảnh:
`local/research/codex/late159-r1/`. Ba controller tuần tự với lease trong namespace
user/net/mount/PID, loopback/no outbound/files-only NSS/local stub. A/B prefix/wineserver/display/
game-dir riêng. Source binary/pack/DB giữ nguyên; không thêm patch/ép actor hoặc bind ID.
Copy kế thừa CP020 import/API/AC/guard và initial global reset, không vanilla Windows.

| Artifact | SHA256 |
|---|---|
| Runtime A/B CA-r2-stage-social.exe | `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef` |
| Static CA-r2-login-logout.exe | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` |
| modO raw memory/base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |

RVA=VA−400000; modO raw, PEcopies mapped theo section. `code-match.json`20vùng/4artifact,
kể cả localpool AEF700 và159/158/result; byte match không proof runtime.
Đọc cache CP024/025 `bomb-position-r1/AEF700.asm`/`round-go-r1/E5E8F0.asm`, kiểm bằng runtime;
không lặp cold REA/IDA timeout hoặc quét toàn client. Nativeframer/getter qua Unicorn có modeled
allocation/receiver; S2Cfixture không oracle server gốc.

| Run / UTC31/10/2026 | Kết quả | run.json SHA256 |
|---|---|---|
| no-old159 / 09:25:05 |Hai vòng đúng ID1/2, hai175/180/phòng. | `98a2821e2946cf3eaea5bc8d4f9186625a62fca50721841e171e53be44ffa450` |
| old159-first / 09:30:53 |A bindID1 vòng2/currentID2entity0; B currentID2 nổ, A không; chỉ result/return vòng1. | `da783617bae4fc95fdec42510162002821da47096018efca59b117a760ae4e07` |
| fenced-old159 / 09:35:44 |Oldjob drop/stale_round; currentID2 bind/nổ cả hai, hai175/180/phòng. | `4b8f0bc51797ca9665f3cd15c6652c69974c6de4e65ca14b5351a732227a6288` |

Mỗi run cùng8connections/IDs1,2/slots0,1/session1, hai actual95/104/121full129/99/clock;
582 rỗng, một actual118 A mỗi vòng/owner0/serial0. ID wire do stub cấp1→2, fuse6s/outcome
Alose/Bwin là emulator choices. Control/old chỉ khác việc chèn159 cũ; timer/current159 giữ nguyên.
Actual118 vòng2 là trigger; old159 được reencode seq receiver hiện tại ngay trước159 hiện tại
trên cùng socket. Không gửi159 trước actual118 hoặc replay rawframe vượt CRC/seq.

Ở `old159-first`, cả ba159 trên mỗi client đều39/39/errors0, không CRC/parser failure.
A vòng2 oldconfirm so serial0=0 rồi bindID1 vào entity local mới; currentconfirm có24 candidate
serialgetter FFFFFFFF/incoming0, entity0. Không gán FFFFFFFF là toàn pending table hoặc mọi mode.
B tạo hai pointer entity khác nhau cho oldID1 và currentID2; không cần localbind breakpoint hit remote.
Current158/ID2 full13: A không tới E1449C/kind0, B lookup đúng current159entity vàkind0/cleanup.
Không actualA17/19 vòng2; rank/return chỉ một lần vòng1. Đây là intentional disruption/partial
gameplay, không “mọi run pass hai trận”. Không giả report19/rebind/ép result để che mất đồng bộ.

Ảnh oldrun ngay sau timer B có bubble cho A nhưng A không; ảnh muộn thay đổi animation.
Không có secure snapshot đúng mốc bubble này, nên không gọi đó là enum remote trap/death tồn tại mãi.
Final A/B vẫn scene trận, không result vòng2. 22ảnh ghép đã đọc: control8/old6/fenced8;
immediate send snapshot có thể trước callback. Runtime entity/ID/lookup và actualreports là nguồn riêng.

`fenced-old159` import [RoundFence](../../../server/round_fence.py) sẵn có, không đổi code module;
`server-round_fence.py` archive exactbytes, runtime sourceSHA. Begin trên actualStart, register ticket
cho actual118 hiện tại; kiểm lifetime ticket gốc trước159, timer consume cùng ticket trước158,
finish khi policy chọn rank. Oldticket vòng1 được kiểm tại generation2 →stale_round/broadcast rỗng;
current159 ID2 vẫn bind/nổ, actual17/19/statepeer1 trước175,175outer302/inner292 và180full3/phòng
hai vòng. Không consume ticket timer chỉ để ACK; kiểm lifetime không tự chặn duplicate159 cùng vòng.
Owner/lock serialization vẫn là caller responsibility; full requestvalidator/UDP/already-sent ngoài scope.

`parse_runtime.py` ghép159 theo thứ tự TCP/send/callback từng receiver; lưu entity/binds/serialcompare,
158lookup/kind0/cleanup và countsresult. `parse_fields.py` giữ securestate/coords/stat/checksum/
native52A350/sourcehash ởclock/rank; các fieldclock hai vòng khớp trong cả ba run không làm oldrun
đạt consistency. `derived/` replay mọi actualC2S/CRCbad/partial; `native-packets.json` typed/blob/checksum
cho selected159/158/127/170/175/180/582. Seq S2C riêng liên tục, oldconfirm có currentseq.

**6819 kiểm cơ học pass**,3sealedruns gồm1intentionaldisruption,471nativepacketcases/
768actualsecure getterchecks,20ranges/4artifact.12policy regressiontests pass, sourceartifacts5/5.
Số kiểm không là tiến độ gameplay, checker không suy semantics/gate. Script/report/index/hash/raw
được pin trong [CP043manifest](../checkpoints/manifest.json). Rawfile/size/SHA cuối run tại
`<run>-hashes.json`, argv chính xác ở `<run>-recipe.json`.

Tái lập bằng label/prefix mới, controller tuần tự, không đè sealedrun:

```bash
python3 local/research/codex/late159-r1/run_probe.py new-control CA-r2-stage-social.exe 230 wine_probe_late159_control.py local/research/codex/late159-r1/late.gdb
python3 local/research/codex/late159-r1/seal_run.py new-control
python3 local/research/codex/late159-r1/run_probe.py new-old159 CA-r2-stage-social.exe 230 wine_probe_late159_old.py local/research/codex/late159-r1/late.gdb
python3 local/research/codex/late159-r1/run_probe.py new-fenced CA-r2-stage-social.exe 230 wine_probe_late159_guarded.py local/research/codex/late159-r1/late.gdb
```

Giới hạn: chưa169/variant/context khác/serialwrap/IDallocator/roundcancel/timer abort/concurrent,
gói đã phát hoặc UDP/peerqueue/fullreset/fullauth/cache/codec/vanilla/accountreconnect/NAT/relay.
Không fullmatch/K5/K6a/LAN/Internet. CP041uniqueID chống old158 không bao phủ old159 khi serial
trùng; CP043 bổ sung ca riêng, giữ nguyên raw/checkpoint lịch sử.
