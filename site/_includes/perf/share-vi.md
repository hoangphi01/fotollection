### Facebook làm gì với một bức ảnh, và chúng tôi làm gì trước

Facebook thu mọi ảnh về tối đa 2.048 px, nén lại ở khoảng chất lượng 71 với màu giảm một nửa, và bỏ qua hồ sơ màu. Kiểu xuất Mạng xã hội làm những việc đó cho đúng trước khi tải lên, để Facebook không còn gì phải thu nhỏ: ảnh được chuyển sang sRGB qua hồ sơ màu của nó, thu nhỏ bằng bộ lọc Lanczos trong ánh sáng tuyến tính, làm nét nhẹ, và lưu ở chất lượng 88. Mỗi thiết lập đều được chọn qua phép đo: làm nét mạnh hơn (0,35 thay vì 0,2) cho điểm thấp hơn, còn chất lượng 84 hay 80 mất nhiều hơn phần tiết kiệm được.

| Trung vị | Giữ cỡ gốc | Facebook, 2.048 px | Instagram, 1.080 px |
|---|---|---|---|
| Dung lượng (ảnh máy ảnh 3,08 MB) | 1,42 MB | 0,51 MB | 0,24 MB |
| SSIM so với bản thu nhỏ lý tưởng (1 là giống hệt) | 0,981 | 0,979 | 0,982 |
| ΔE2000 | 0,96 | 1,27 | 1,58 |

Sau khi Facebook tự nén lại, kết quả vẫn giống ảnh hơn cách làm thông thường (SSIM 0,965 so với 0,961), và ảnh Display P3 giữ được 99,6 % màu thay vì 86 %, vì hồ sơ màu của nó được đổi thành màu sRGB thật chứ không bị bỏ đi. Các kiểu thu nhỏ vẫn trên ΔE 1 vì màu giảm một nửa, điều mà nền tảng nào cũng làm; *Giữ cỡ gốc* giữ nguyên từng điểm ảnh với tệp chỉ còn khoảng một nửa, dưới ΔE 1.
