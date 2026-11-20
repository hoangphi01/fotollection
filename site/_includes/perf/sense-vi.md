### Thử tải, không phải một ngày bình thường

Phần lớn mọi người lọc từng chuyến đi một, 2.000 đến 3.000 tấm. Để xem Fotollection có chạy ổn định xa hơn thế không, RollSense được giao 10.044 tấm cùng lúc trên một laptop Windows bình thường đời 2020 (Lenovo ThinkPad X1, Intel i7-10610U, 15 W, 16 GB): JPEG từ máy ảnh, ảnh điện thoại, ảnh chụp màn hình và file RAW của Canon, Sony. Nó đọc 9.639 tấm trong 70 phút trên mọi nhân, bộ xử lý bận suốt, bộ nhớ trong khoảng 400 đến 850 MB và không tăng dần.

### Thời gian đi đâu

Mỗi tấm được giải mã một lần, thu nhỏ rồi đưa cho mô hình nhận diện nội dung. Riêng mô hình mất 128 ms mỗi tấm trên laptop này, cả bước mất 431 ms, nên khoảng 70 % thời gian là đọc và giải mã chính tấm ảnh, phần lớn là file JPEG và RAW độ phân giải đầy đủ. Card đồ hoạ gần như không dùng, cao nhất 1,5 %: tất cả chạy bằng Rust trên bộ xử lý.

### Thêm luồng, và giới hạn của laptop

Hai luồng mất 614 ms mỗi tấm, bốn luồng 498 ms, tám luồng 438 ms. Laptop mỏng chia 15 W cho các nhân, nên khi cả tám luồng cùng chạy thì mỗi nhân chậm lại (trung bình 1,8 GHz, lúc nghỉ 3,4 GHz), và bốn trong tám luồng dùng chung nhân với bốn luồng còn lại.

| | Laptop này (đo) |
|---|---|
| Một chuyến đi 2.500 tấm | 18 phút |
| Cả kho ảnh điện thoại 15.000 tấm | 108 phút |
| Một tấm đọc lại từ cache | 1,3 ms |

Đo ngày 20/11/2026 bằng `sort_pass_bench`, ghi số liệu mỗi giây.
