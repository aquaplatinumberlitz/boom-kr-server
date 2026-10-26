# Tool của server KR

`check_docs.py`: kiểm các link Markdown nội bộ, không gọi network.
`artifacts.py verify`: kiểm SHA/size các đầu vào RE theo manifest đã ghi; chỉ đọc source.

Harness/probe/bootstrap và codec mới sẽ đặt tại đây hoặc `server/`; không ghi vào tool repo VN/KR cũ.
Raw output, bản copy client/DB và capture đặt trong `local/`, không commit.
