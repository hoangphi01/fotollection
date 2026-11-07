### Trước đây: bấm vào ảnh nào thì giải mã ảnh đó

Tới bản v0.10.0, bản xem trước chỉ được dựng khi bạn cần: đọc tệp, giải mã ảnh (với RAW là develop), xoay cho thẳng, thu về 2.048 px và chuyển sang sRGB, rồi cắt, chỉnh màu, đóng dấu ngày và nén cho màn hình. Với một tấm ảnh 20 MP từ Lumix GX9, việc đó mất 274 đến 278 ms cho ảnh JPEG và 440 đến 531 ms cho ảnh RAW, lần nào mở một ảnh không nằm trong tám ảnh vừa xem cũng vậy. Đủ nhanh cho một tấm; chưa đủ để lướt cả chuyến đi.

### Bản xem trước chuẩn, như Lightroom giữ

Từ bản v0.11.0, việc người dùng phải chờ được làm trước khi họ cần. Ngay khi ảnh nhỏ của một thư mục đã xong, Fotollection dựng bản 2.048 px của từng ảnh ở chế độ nền, mỗi luồng một ảnh, và giữ trên đĩa cạnh ảnh nhỏ, dưới dạng JPEG khoảng 0,66 MB (cả chuyến đi khoảng 280 MB). Bấm vào ảnh nào, gần hay xa, trong lần mở này hay lần sau, đều bắt đầu từ bản đó: 27 ms, JPEG hay RAW như nhau. Phần còn lại là việc riêng của bản xem trước (cắt, thu nhỏ, chỉnh màu, đóng dấu ngày và nén), cũng 27 ms như khi lấy từ bộ nhớ. Bản xem trước dựng từ bản trên đĩa chỉ lệch trung bình 1,35 mức trên 255 so với bản dựng mới, quá nhỏ để mắt thấy.{% include fn.html keys="fn_lightroom" %}

### Dùng bộ nhớ cho hiệu quả, không phải dùng càng ít càng tốt

Một ứng dụng xem ảnh chỉ giữ tám ảnh trong bộ nhớ trên chiếc laptop 16 GB là lãng phí chiếc laptop đó. Giờ Fotollection dành cho bản xem trước một phần tư bộ nhớ của máy (trên máy Mac 16 GB, khoảng 190 ảnh 2.048 px, 20 ảnh RAW đã develop để kéo thanh trượt, và 600 MB cho phóng to), tính theo byte, phần cũ nhất được trả lại trước. Khi một ảnh đang hiện, hai ảnh kế tiếp theo hướng bạn đang đi và một ảnh phía sau cũng được nạp sẵn, nên phím mũi tên không bao giờ phải chờ. Mức trần 2 GB vẫn giữ cho lúc xuất ảnh.

### Ảnh trên màn hình được làm trước

Mỗi lúc chỉ develop một ảnh RAW, để cả loạt ảnh RAW vừa trong bộ nhớ. Trước đây thứ tự trong hàng chờ là may rủi: một bản xem trước đang dựng, phần ước tính dung lượng xuất hay một lần xuất ảnh có thể bắt ảnh bạn vừa bấm phải chờ. Giờ mỗi lần develop đều biết ai đang chờ nó, và việc chạy nền nhường chỗ mỗi khi ảnh trên màn hình cần. Cũng nhờ lần làm này mà hai lỗi treo hiếm gặp đã được gỡ: một lỗi mà mọi luồng cùng chờ một lần develop không còn luồng nào để chạy, và một lỗi mà màn hình chờ một tấm ảnh nó chưa từng xin.

| Phiên bản | JPEG trung vị / p95 | RAW trung vị / p95 | Cách làm |
|---|---|---|---|
| v0.9.0 | 274 / 286 ms | 531 / 550 ms | Giải mã khi bấm; RAW qua macOS |
| v0.10.0 | 278 / 293 ms | 440 / 471 ms | Giải mã khi bấm; RAW qua bộ develop của Fotollection |
| **v0.11.0** | **27 / 32 ms** | **27 / 30 ms** | Từ bản xem trước chuẩn trên đĩa, hoặc từ bộ nhớ |

Đo trên chuyến đi Praha của tác giả (396 JPEG và 26 RAW, 20 MP) bằng Apple M3 với `measure_browse`: v0.9.0 dựng từ chính tag của nó, v0.10.0 đo trên mã nguồn ngay trước thay đổi này.
