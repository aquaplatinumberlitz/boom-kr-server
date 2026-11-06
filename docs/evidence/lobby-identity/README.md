# Identity và consumer panel sảnh — CP182

Owner Codex, tự kiểm; [review waived](../checkpoints/review-waiver.json).
Baseline CP181/PR253 merged `5a154e5`. Contract duy nhất [PROTOCOL](../../PROTOCOL.md#local-name-model),
câu hỏi [RESEARCH](../../RESEARCH.md#gap-lobby-identity-followup), thứ tự [PLAN](../../PLAN.md#current-order).
K3 giữ mở; checkpointcomplete không nghiệm thu UI hoặc fullauth.

## Recipe và bằng chứng

Raw `local/research/codex/k3-identity-cache-r1/`; hai runidentity và bốn runpanel có recipeJSON,
preparation pin input, runJSON, client/service logs, ảnh và raw-seal sau cleanup. Metadata cũ
[in-progress](in-progress.json) giữ nguyên snapshot hai runidentity trước khi nối UI, không là trạng thái mới.
Danh sách raw/size/SHA ở manifest bằng chứng CP182, không sao chép raw vào docs.

Copy Wine `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`,
analysisinput `cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b`;
base400000, RVA=VA−400000. Không sửa sourcebinary/DB/pack, không gọi nativegamefunction hoặc ép
record/guard. Kế thừa copydiagnostic/bootstrapglobalreset đã có; không nguyên bảnWindows.
Productionservice/47fixtureprofile không đổi; mỗi run dùng user/net/mount/PID namespace riêng,
loopbackonly/noexternalroute/NSS files-only, prefix/Xvfb riêng, source/config frozen trongrun.
Game/prefix copy kế thừa cache; không gọi là coldcache. Rollback dùng recipeidentityr2 khôngpanelprobe.
Cleanup ownednamespace/process/prefix theo harness trước khi chuyển phép thử, raw/hash lưu ngay.

| Run | Kết quả / giới hạn |
|---|---|
| identity-r1 | SetterID/name vàactual6; FinishBreakpointname không tin cậy, giữ candidate |
| identity-r2 | ExactgetterreturnPC/name/ID vàactual6nativeparse; không UIproof |
| panel-r1 | ActualRoomUIctor; SquareLeftselectedcallbacks khônghit trong60s, không suy classabsence |
| panel-r2 | Read-onlyready snapshot62RTTI candidates; actualembeddedCUserListUIWnd được xác định |
| panel-r3 | C++exception/SIGABRT trướcready, guard chưa đo; không gọi harnessexit0 là thành công |
| panel-r4 | Hoãn probe tớiready, hai actualrenderguard0 và record544byte/nameempty; không causalfixclaim |

VN `lobby.md` giúp phân biệt UserMap list/ownpanel với authmanager; ownlevel/exp phíaVN cũng chưa
kiểm nên không có contract hoàn chỉnh để port. CP058/059/060/181 auth/cache/nativecodec vàCP016
UIdelegation được dùng lại, tránh đo lại auth56field/CRCrepresentation/186getterxrefs.
16UI/storebody mới được so byte với runtime; badSP/None decompile và privateDBloadfailure giữ
raw, dùng assembly/privateDBcopy khác, không sửa cấu hình MCP. `D7A260` chỉ levelchange consumer.

`python3 local/research/codex/k3-identity-cache-r1/verify_panels.py` đã chạy **777 kiểm**:6rawseals,
inputpins/isolation/ownedcleanup/nonoverlap,16nativebodies và guard/SSOraw. Script ghi output một lần;
không chạy lại để ghi đè report. Checker chứng minh hash/khai báo, không ngữ nghĩa/gate.
Các kiểm repo bắt buộc và exactheadPR bổ sung trong bàn giaoGit. Audit AUD054 qua180, kếCP184.

Bước tiếp: producerrecord→dispatch→store→render theo PLAN; fullcache/identityauthority/Windows/LAN/
Internet/8client/fullmatch không được khôi phục bởi checkpoint này.
