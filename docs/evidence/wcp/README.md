# WCP — phản hồi trên kết nối thứ hai

Đối chiếu ngày27/10/2026, CP001. Đây là thí nghiệm riêng sau [Wine R2](../wine-r2/README.md),
chưa nằm trong harness đã merge. Snapshot review/integration ở [metadata](verification.json).
Contract và mức khẳng định chỉ giữ tại [PROTOCOL](../../PROTOCOL.md#wcp);
bước kiểm tiếp chỉ giữ ở [RESEARCH](../../RESEARCH.md#q-023).

## Recipe và run

Binary R2 SHA7b3b435b5d8e110185359107376b736615ceabe9452cea8154d02c8628f44c29.
Private local/research/codex/wine_probe_wcp.py SHA8c61e88f537f6dcfc995b64835f394923c517d7edd2b891b04eafecf249fc860
gửi fixture cho hai accept đầu, thay vì chỉ accept đầu như tool đã merge; không sửa binary.
Script có path local riêng; không coi là tool phân phối hoặc service.

Fixture15 byte69000c00000003c700000000000000,
SHA75be47b3067e2b1e9d2cf99b5a7eb7f949a074482409fb3eafc1a553cec53cd1.
Chọn sau oracle native, không phải capture server gốc. Opaque/zero/empty là emu_choice;
u16 thứ hai không được chứng minh là version WCP chỉ vì fixture có967.

| Run trong local/research/wcp-probe/cold-1/ | Quan sát | Giới hạn |
|---|---|---|
| run,2026-10-27T00:16:39Z,20s | Hai accept3858/3888,hai reply15 byte,zero C→S; CALog InitBeforePatch/ProcessPatch | Plain;Login Failed từ sample01,zero C→S20s;absence timeout riêng nó không chứng minh consumer |
| debug,2026-10-27T06:02:03Z,15s | Break13AD782EAX1 sau poll và13AD79CEAX1 sau consumer; WSAAsyncSelect socket208,hWnd40044,msg400,mask21 status0;zero C→S | Debugger,reuse game/prefix; không cold start hoặc codec nhiều frame |
| debug cuối | Login Failed; CALog mới [XRpt] MW_SEP(10) | Khác WCP timeout run silent cũ; guard thực gây lỗi chưa đo |

Caller runtime:1255C73→1255A69→1234EAF→123235D→122E913→125407F.
Oracle native14EF300→13AEFE0 dùng input tổng hợp, chỉ thay allocator14F5790:
return1,cursor12,reader_error0,socket+40/+44/+48=[2,3,0].
+48=0 không chứng minh đó là storage của seed vì input u32 và nền socket đều0.
Debug đọc globals2/3,chưa đọc trực tiếp socket fields sau hello.
Không chạy lại client trong đợt đồng bộ docs; chỉ đọc raw và kiểm hash.

## Lịch sử, review và phần còn thiếu

R2 đã review/merge PR10 tại2dbbf32; các run silent cũ vẫn đúng cho recipe đó.
Probe hai-reply và report Claude WCP r1/r2 chưa có review độc lập riêng tại CP001;
Snapshot khi ghi CP001 là pending;review về sau xem [manifest](../checkpoints/manifest.json). Không coi thành gate K2/login.
Report có erratum giữ hash cũ; contract mới trích correction, không sửa raw lịch sử.
Zero C→S không chứng minh toàn client chắc chắn cần gói server tiếp; còn guard sau connect chưa đo.
Nguồn báo cáo/oracle/debug commands/run/log/digest ở [verification.json](verification.json).

Audit CP001 không nhập suy luận thời gian dialog từ offset chéo run (raw audit F6) thành fact; cần timestamp/caller guard trực tiếp.
Report WCP r2 còn sai suy luận1234E70return1⇒WCPaccept;correction ở [PROTOCOL](../../PROTOCOL.md#wcp).
