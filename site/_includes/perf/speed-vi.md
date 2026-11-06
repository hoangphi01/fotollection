### Khi bạn mở một thư mục

Một thư mục ảnh không bị đọc hết. Với mỗi tệp, Fotollection chỉ đọc dữ liệu EXIF (ngày, máy ảnh, nơi chụp) và ảnh thu nhỏ mà máy ảnh nào cũng lưu sẵn trong tệp, ngoài ra không gì khác: 1.000 ảnh mất 0,12 giây, trong khi ngân sách đặt ra trước khi viết mã là 1 giây. Sau đó các ảnh nhỏ 400 px nét hơn được làm ở chế độ nền, trên hai luồng để Fotollection vẫn nhanh, và lưu vào bộ nhớ đệm trên ổ đĩa, nên mở lại thư mục là có ngay. Với 5.035 ảnh, việc nhập mất 383 ms và ảnh nhỏ có sẵn mất 236 ms.

### 17 ms của một lần xem trước đi đâu

Đổi một thiết lập thì bản xem trước được dựng lại từ một bản sao 2.048 px của ảnh giữ trong RAM, không bao giờ từ tệp đầy đủ. Trong 16,7 ms đó, cắt khung và thu nhỏ bằng bộ lọc Lanczos mất 9,7 ms, dấu ngày dưới 0,1 ms, và mã hoá bản xem trước thành JPEG cho màn hình mất 7,0 ms. Bộ mã hoá đó được chỉnh theo phép đo: với thiết lập của bản xuất (đủ độ phân giải màu và bảng mã tối ưu) nó mất 16,2 ms; màn hình không cần những thứ ấy, nên bản xem trước dùng màu giảm một nửa và bảng mã thường, còn bản xuất vẫn giữ đủ màu (4:4:4). Một bộ lọc thu nhỏ rẻ hơn đã được thử và hoá ra chậm hơn trên bộ xử lý này (12,5 ms), nên Lanczos được giữ lại.

### Mã chạy trực tiếp so với trình duyệt

Cùng một việc (giải mã, cắt về 2:3, dấu ngày, JPEG chất lượng 95) được chạy trên cùng 95 ảnh, trong Fotollection và trong Chromium với mỗi nhân một worker. Tốc độ xuất ngang nhau, 18,0 so với 17,9 ảnh mỗi giây, nghĩa là phỏng đoán ban đầu của chúng tôi đã sai: trình duyệt không chậm. Lý do để viết mã chạy trực tiếp nằm ở chỗ khác: Fotollection ghi tệp 4:4:4 với đủ độ phân giải màu, còn bộ mã hoá của trình duyệt giảm màu một nửa; nó giữ hồ sơ màu của từng ảnh, ghi tệp thẳng xuống ổ đĩa, và dùng ít hơn một phần tư RAM (1,35 GB so với 1,78 GB). Từ đó tới nay bộ mã hoá ảnh in còn nhanh hơn nữa: 23,5 ảnh mỗi giây ở 16 MP.
