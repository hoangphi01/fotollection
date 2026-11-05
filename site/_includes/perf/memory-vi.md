### Vì sao RAM không tăng theo thư mục

Fotollection chỉ giữ vài byte cho mỗi ảnh trong thư viện (13 MB cho 5.035 ảnh), ghi ảnh nhỏ vào bộ nhớ đệm trên ổ đĩa thay vì giữ trong RAM, chỉ giữ một số bản xem trước có giới hạn, và chỉ xử lý cùng lúc số ảnh cỡ lớn vừa với hạn mức. Lưới trên màn hình là lưới ảo: chỉ những ô bạn đang thấy mới tồn tại. Để kiểm tra, cùng 95 ảnh được liên kết 53 lần vào một thư mục 5.035 tệp:

| | 95 ảnh | 5.035 ảnh |
|---|---|---|
| Nhập và đọc EXIF | 5 ms, 7 MB | 383 ms, 13 MB |
| Ảnh nhỏ 400 px vào bộ nhớ đệm | 47,8 ảnh/giây | 46,4 ảnh/giây |
| Xuất, 6 ảnh cùng lúc | 18,0 ảnh/giây | 15,4 ảnh/giây, 0 lỗi |
| RAM dùng nhiều nhất cả lượt | 1,35 GB | 1,45 GB |

Số ảnh gấp 53 lần chỉ tốn thêm 0,1 GB. Lượt xuất dài chậm hơn một chút mỗi ảnh chỉ vì nó chạy hơn năm phút, và bộ xử lý của máy xách tay chậm lại khi chạy nặng liên tục.
