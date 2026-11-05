### Vì sao ảnh in qua trình sửa ảnh web trông nông màu

Phép đo xuất 20 ảnh máy ảnh và 5 bản Display P3 (dải màu rộng của điện thoại và máy Mac đời mới) qua đúng đường xuất thật của Fotollection, rồi đọc cả hai tệp bằng một bộ giải mã độc lập, chuyển sang CIELAB qua hồ sơ màu của chúng và so từng điểm ảnh với ảnh gốc. Một bản xuất kiểu trình sửa ảnh web thông thường cũng được so y như vậy: bỏ hồ sơ màu, thu xuống 2.000 px rồi phóng to lại để in, lưu ở chất lượng 80 với màu giảm một nửa (4:2:0).

| Trung vị | Fotollection, sRGB | Trình sửa web | Fotollection, Display P3 | Trình sửa web |
|---|---|---|---|---|
| ΔE2000 (sai lệch màu, dưới 1 là mắt không thấy) | 0,19 | 1,39 | 0,31 | 2,15 |
| PSNR | 52,6 dB | 37,9 dB | 50,8 dB | 37,0 dB |
| Độ bão hoà giữ lại | 100 % | 99,9 % | 99,9 % | 86,8 % |
| Chi tiết nhỏ giữ lại | 99,8 % | 13 % | 100 % | 18 % |

Hai nguyên nhân giải thích vẻ nông màu. Thu về 2.000 px rồi phóng to lại cho tiệm in làm mất 87 % chi tiết nhỏ, thứ làm ảnh in trông sắc nét. Và bỏ hồ sơ màu khiến ảnh Display P3 bị đọc như sRGB, mất 13 % độ bão hoà. Fotollection giữ hồ sơ màu (25 trên 25 tệp), xuất ở độ phân giải đầy đủ của ảnh với đủ màu (4:4:4) ở chất lượng 95, và những sai số lớn hơn chỉ nằm ở cạnh sắc, nơi JPEG nào cũng vất vả nhất. Còn chờ: ảnh HEIC thật từ iPhone, và ảnh in đo bằng máy đo quang phổ.
