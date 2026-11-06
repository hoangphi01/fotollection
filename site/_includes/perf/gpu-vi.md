### Mười thanh chỉnh với giá của một

Mọi thanh chỉnh màu (độ sáng, tương phản, ấm lạnh, bão hoà, phai màu, phủ màu vùng tối và vùng sáng, đen trắng) được gộp vào một bảng 33 × 33 × 33 màu, 35.937 điểm cho biết mỗi màu đi về đâu. Áp bảng lên ảnh là tìm ô của từng điểm ảnh trong bảng rồi nội suy giữa bốn góc của ô đó (nội suy tứ diện), nên mười thanh chỉnh tốn bằng một. Dựng bảng khi mọi thanh đều được kéo mất khoảng 2,6 ms; áp lên bản xem trước 1.600 px dưới 2,2 ms; lên ảnh xuất 24 MP khoảng 24 ms.

### Vì sao kéo thanh vẫn thấy chậm, và card đồ hoạ đã thay đổi gì

Đo từng bước, một nhịp kéo tốn khoảng 26 ms, trong đó phần màu chỉ chiếm 2 ms: phần còn lại là mã hoá bản xem trước thành JPEG (7 đến 11 ms) và cửa sổ giải mã lại (9 đến 15 ms). Card đồ hoạ bỏ được cả hai. Khi bắt đầu kéo, bản xem trước chưa chỉnh và một mặt nạ cho biết đâu là ảnh (không phải dấu ngày, khung hay viền) được gửi lên card đồ hoạ một lần; ở mỗi nhịp chỉ bảng mới được gửi theo, và một chương trình WebGL2 nhỏ nội suy nó, đúng như phần lõi làm: 1 ms mỗi nhịp trong WebKit của macOS, 2 ms trong Chromium. Khi bạn thả tay, bản dựng chính xác của lõi thay vào. Hai bản đã được so trên một look mạnh với mọi thanh đều được kéo: chúng khác nhau trung bình 0,13 theo ΔE2000 và 0,77 ở phân vị 99, thấp hơn rất xa mức 1 mà mắt thấy được khi đặt cạnh nhau.

Không cần công tắc nào. Ở lần kéo đầu tiên, Fotollection tự kiểm tra: nếu không có WebGL2, hoặc bộ dựng hình là phần mềm, hoặc một nhịp thử mất từ 8 ms trở lên, nó giữ đường bộ xử lý cho cả phiên. Bộ dựng hình phần mềm mất 16,5 ms mỗi nhịp khi thử, nên ở đó ở lại với bộ xử lý là đúng.

### Xuất ảnh trên mọi nhân

Việc giải mã và mã hoá một ảnh mỗi việc chạy trên một nhân, nên xuất ảnh nhanh khi nhiều ảnh được xử lý cùng lúc. Bao nhiêu ảnh thì tuỳ vào RAM: với ảnh 24 đến 26 MP, 2 ảnh cùng lúc cho 4,9 ảnh mỗi giây, 4 ảnh cho 9,4, 6 ảnh cho 11,5 và 8 ảnh cho 12,5, còn RAM tăng từ 0,49 lên 1,21 GB. Fotollection chọn con số từ hạn mức RAM 1,6 GB và cỡ của ảnh lớn nhất, và dừng ở 6, vì trên ảnh của một chuyến đi thật, 8 ảnh cùng lúc không nhanh hơn mà tốn thêm 0,2 GB. Xuất ảnh không bao giờ dùng card đồ hoạ: ảnh in phải giống hệt nhau trên mọi máy, và các bài kiểm tra so bản xuất từng byte dựa vào điều đó.
