## I. Vạch xuất phát {#start}

Một ngày, tôi nhận ra mình đã sống được hai mươi năm. Trong chiếc điện thoại có hàng chục nghìn tấm ảnh, và tôi gần như chưa bao giờ mở lại chúng. Những gì của năm, sáu năm trước bắt đầu nhạt đi trong trí nhớ: một buổi chiều, một khuôn mặt, tên một con phố. Ảnh vẫn còn đó, nhưng chúng nằm im trong một thư mục mà tôi chẳng bao giờ ghé tới.

Rồi tôi sang châu Âu học. Mùa đông ở đây dài và tối sớm, nên có nhiều khoảng lặng hơn tôi tưởng. Trong những khoảng lặng ấy tôi ngồi lướt lại những năm tháng cũ, và thấy một nỗi buồn man mác: những kỷ niệm đẹp, những câu chuyện còn dở dang, tất cả nằm sau một tấm kính. Ông bà tôi có những cuốn album dày, mỗi tấm ảnh có góc dán, có dòng chữ viết tay ở mặt sau. Thế hệ của tôi thì chụp nhiều hơn bất kỳ thế hệ nào trước đó, nhưng lại giữ được ít hơn. Tôi nghĩ: cứ bắt đầu từ đó đã. Tôi muốn làm một thứ gì đó để những kỷ niệm ấy trở lại thành vật cầm được trên tay.

Tôi coi việc này như một cuộc chạy marathon. Không ai chạy hết bốn mươi hai cây số trong một hơi, nên tôi chỉ hỏi mình bước đầu tiên là gì. Vào tháng 9 năm 2026, bước đầu tiên là một bản kế hoạch cho một ứng dụng web tên "PhotoOrder". Vấn đề tôi viết ra khi ấy vẫn còn nguyên trong tài liệu sản phẩm của dự án: người ta chụp hàng nghìn tấm mà in chẳng được mấy; ảnh in ra không ghi ngày, không ghi nơi, vài năm sau chẳng ai nhớ; tiệm in tự cắt ảnh về 2:3, 3:4 hay vuông, có khi cắt mất cả đầu người và đường chân trời; còn các công cụ online thì đòi tải ảnh lên, trong khi dữ liệu GPS có thể chỉ thẳng tới nhà mình.

## II. Những bước chạy đầu tiên {#first-steps}

Commit đầu tiên của dự án vào chiều ngày 23 tháng 10 năm 2026, chỉ có ba tài liệu: kế hoạch, bảo mật và tiến độ, chưa có dòng code nào. Ba mươi lăm phút sau, dự án đã có tên mới, Fotollection, và một hướng đi khác hẳn. Tôi bỏ ứng dụng web để làm một ứng dụng desktop chạy hoàn toàn offline, viết lõi bằng Rust. Lý do được ghi ngay trong quyết định kiến trúc đầu tiên: người dùng là những người chụp ảnh nghiêm túc, nên hiệu năng và chất lượng bản in phải đứng trước. Một trình duyệt bị giới hạn bộ nhớ mỗi tab và không cho tôi nắm chắc cách ảnh được mã hoá.

Cùng ngày hôm đó, vài nguyên tắc được đặt xuống và đến giờ vẫn giữ nguyên. Tên địa điểm được tra ngay trên máy, từ một bộ dữ liệu GeoNames đóng gói sẵn, chứ không gửi toạ độ của bạn cho một máy chủ nào cả. Mã nguồn theo giấy phép GPL-3.0, để không ai có thể biến nó thành một bản đóng. Bộ cài thì chưa được ký số, vì đây là một dự án miễn phí, không có ngân sách cho những chứng chỉ trả phí hằng năm. Và tài liệu luôn đi cùng code trong cùng một commit, một thói quen sau này giữ cho cả dự án không bị lạc.

Tối hôm đó, lát cắt đầu tiên chạy được từ đầu đến cuối: nhập ảnh, xem trước, đóng dấu ngày, xuất JPEG. Rồi tôi làm một việc mà mọi người chạy nên làm sớm: đo. Tôi đặt bản Rust cạnh một bản chạy trong trình duyệt, cùng 95 tấm ảnh 16 MP trên cùng một chiếc máy. Kết quả làm tôi bất ngờ: xuất ảnh gần như ngang nhau, 18,0 ảnh mỗi giây so với 17,9. Giả thuyết ban đầu của tôi, rằng trình duyệt sẽ chậm, đã sai, và tài liệu tối ưu ghi lại đúng chữ đó. Lý do thật để làm một ứng dụng native nằm ở chỗ khác: ảnh in được mã hoá ở 4:4:4 thay vì 4:2:0, bộ nhớ có giới hạn kể cả với năm nghìn tấm, hồ sơ màu được giữ lại, và tệp được ghi thẳng ra đĩa. Ngay hôm đầu, tôi đã học được rằng con số đo được đáng tin hơn điều mình tin.

## III. Lần đầu vấp ngã {#first-fall}

Commit sửa lỗi đầu tiên không liên quan gì đến ảnh: các tệp test trên Windows không chạy được vì thiếu một manifest của hệ thống. Một vấp ngã nhỏ, nhưng là lời nhắc rằng mỗi hệ điều hành là một mặt đường khác nhau.

Hai ngày sau, những cú vấp bắt đầu dạy tôi nhiều hơn. Có lần một dấu đỏ trên CI nằm im qua ba lần push mà tôi không thấy, chỉ vì một danh sách bị đánh số "7." hai lần trong tài liệu. Từ đó, việc kiểm tra định dạng tài liệu chạy ngay trong script kiểm tra mà tôi luôn chạy trước khi commit. Có lần mở một tấm ảnh bằng ứng dụng khi ứng dụng đang tắt thì chẳng có gì xảy ra: macOS đã giao tệp tới trước khi ứng dụng kịp dựng xong trạng thái của nó. Và khung "Shot on" đầu tiên in ra dòng "NIKON CORPORATION NIKON D750", vì trường Make trong EXIF là tên một công ty chứ không phải tên một thương hiệu.

Sau những lần như thế, tôi mở một tệp tên LESSONS.md. Mỗi lần gỡ lỗi mất hơn một lần nhìn đều được ghi vào đó: thấy gì, vì sao, sửa thế nào, và từ nay làm gì. Tệp ấy giờ là thứ tôi đọc đầu tiên mỗi khi có gì đó hỏng. Chạy đường dài không phải là không bao giờ ngã, mà là ngã ở đâu thì nhớ chỗ đó.

## IV. Bản in không được nhạt {#prints}

Có một điều tôi để ý từ lâu: ảnh in qua các trình sửa ảnh trên web thường trông nhạt, màu bệt và loang, thiếu chiều sâu so với ảnh gốc. Ngày 24 tháng 10, tôi đem nỗi nghi ngờ đó ra đo từng điểm ảnh. So với một bản xuất kiểu trình sửa ảnh web thông thường, Fotollection lệch màu ΔE2000 trung bình 0,19 thay vì 1,39, giữ 100 % chi tiết mịn thay vì 13 %, giữ 99,9 % độ bão hoà của ảnh Display P3 thay vì 86,8 %, và giữ hồ sơ màu ICC ở cả 20 trên 20 tấm. Nguyên nhân không có gì bí ẩn: hồ sơ màu bị bỏ đi, ảnh bị thu nhỏ rồi phóng to lại, màu bị nén một nửa. Chỉ cần không làm những việc đó là bản in giữ được ảnh.

Những ngày ấy, đích đến cụ thể nhất là máy in tự phục vụ ở các cửa hàng dm và Rossmann. Tôi đã thêm đủ mọi khổ in trong bảng giá dịch vụ ảnh CEWE ở đó vào ứng dụng. Website ra đời cùng lúc, bằng tiếng Anh và tiếng Việt, mở đầu bằng một câu hỏi tôi vẫn thấy đúng: chụp hàng nghìn ảnh, chỉ xem lại khi ngồi trên máy bay? Ngày 25 tháng 10, trang chuyển về fotollection.org. Cũng ngày hôm ấy có bảy bản phát hành, từ v0.3.0 đến v0.7.0: giỏ in, trình xem có phóng to, rồi ảnh RAW. Đó là những cây số chạy nhanh nhất, khi đôi chân còn khoẻ và con đường còn mới.

## V. Lời từ chối bên đường {#refusal}

Ngày 26 tháng 10, tôi nộp đơn xin ký số miễn phí cho bản cài Windows qua SignPath Foundation. Lời từ chối đến ngay trong ngày: họ tìm những dấu hiệu tin cậy từ bên ngoài như số sao, số fork, người đóng góp, bài viết độc lập, và dự án của tôi chưa có đủ. Họ nói tôi có thể nộp lại sau. Lời từ chối ấy không làm tôi dừng lại. Nó chỉ cho tôi thấy rõ quãng đường còn phía trước. Đến giờ bản cài vẫn chưa được ký, và mỗi bản phát hành đều kèm mã SHA-256 để ai cũng kiểm tra được tệp mình tải về.

Cùng ngày, tôi phạm một lỗi mà tôi vẫn nhắc mình: website và ghi chú phát hành v0.7.0 viết rằng ảnh RAW mở được mà không cần cài gì thêm. Trên Windows, điều đó không đúng. Chính tôi phát hiện ra chứ không phải một bài test nào cả. Bài học được ghi lại thành quy tắc: không bao giờ viết "hỗ trợ" khi mới thử trên một hệ thống. Và cũng từ đó, tôi quyết định ảnh RAW phải đọc được bằng code nằm ngay trong Fotollection, không phụ thuộc vào thứ người dùng phải tự cài.

## VI. Tự đóng đôi giày của mình {#shoes}

Một người chạy đường dài rồi sẽ muốn có đôi giày vừa chân mình. Ngày 27 tháng 10, Fotollection có bộ giải mã RAW của riêng nó, bắt đầu từ DNG. Tôi muốn màu đẹp ngang những phần mềm lớn mà không bắt ai phải cài thêm gì. Trên một tệp DNG của Ricoh GR III, bộ giải mã mới xử lý xong trong 355 ms, dùng 526 MB bộ nhớ, và lệch màu ΔE2000 2,67 so với ảnh JPEG chính máy ảnh tạo ra. Thư viện tôi dùng trước đó cần 640 ms, 1,33 GB và lệch 4,08.

Con số 2,67 có một câu chuyện riêng. Khi bản DNG của tôi còn lệch khoảng một đơn vị, tôi tưởng lỗi nằm ở màu. Vẽ bản đồ sai số ra mới thấy phần giữa ảnh sáng hơn rìa: đó là độ tối góc của ống kính, không phải màu. Bài học ghi lại khi ấy: vẽ bản đồ lỗi theo từng vùng trước khi thêm tham số. Cùng tuần, bản v0.8.0 ra đời với cái tên "Places and Lightroom": sửa đồng hồ máy ảnh bị lệch, đặt ngày và nơi chụp bằng tay, và đọc số sao từ Lightroom. Rồi có hai ngày gần như im lặng, ngày 28 và 29, quãng nghỉ duy nhất trong cả cuộc chạy.

## VII. Tìm được nhịp chạy {#rhythm}

Đến cuối tháng 10, ứng dụng đã làm được rất nhiều thứ. Nhưng càng nhiều tính năng thì tôi càng thấy nó khó hiểu với người mới. Ngày 31 tháng 10, tôi tách nó thành ba chế độ. **Đơn giản** là đường ngắn nhất từ một thư mục tới những tấm ảnh in: xem từng tấm, giữ hay bỏ, rồi xuất. **Ghép ảnh** đặt hai ảnh lên một tấm in, như chiếc máy film half-frame từng chụp hai khung trên một ô phim, nên in cả bộ chỉ tốn một nửa. **Pro** là tất cả những gì đã xây trước đó.

Đó là lúc Fotollection tìm được nhịp chạy của mình. Nó không cố làm một trình quản lý ảnh đa năng, mà là một con đường ngắn và rõ từ điện thoại hay thẻ nhớ tới tờ giấy ảnh trên tay. Bản v0.9.0 "Three modes" ra ngày 2 tháng 11. Những lỗi của giai đoạn này cũng thành quy tắc: một ô nhập làm cả bảng nhảy lên khi gõ đã sinh ra nguyên tắc "không gì nhấp nháy, không gì nhảy". Một tấm ghép xuất xong trong 214 ms, nhanh hơn xuất riêng hai tấm ảnh của nó (633 ms). Và lưới các tấm ghép được vẽ lại từ thumbnail, từ 4.088 ms còn 29 ms.

## VIII. Những người cổ vũ đầu tiên và những tấm ảnh xanh {#cheering}

Ngày 1 tháng 11 là ngày đầu tiên những người dùng thật lên tiếng, và họ cho tôi thấy điều mà không bài test nào chỉ ra. Một người dùng bản v0.8.0 mở thư mục của họ và thấy nửa số ảnh hiện thành những ô màu xanh có sọc ngang mảnh. Tôi nghi bộ giải mã RAW trước, nhưng những tệp ấy là JPEG. Bộ giải mã JPEG nhanh mà tôi dùng đã đọc sai một kiểu cấu trúc tệp hiếm gặp, và vẫn báo là thành công. Bản v0.8.1 "Green photos fixed" đặt một bộ giải mã tham chiếu phía sau bộ giải mã nhanh. Từ đó có một quy tắc tôi coi là cốt lõi: một bộ giải mã nhanh chỉ được tin trên những cấu trúc mà một bài test đã chứng minh.

Cùng ngày, một góp ý khác: tick ba tấm ảnh mà nút xuất lại ghi "Xuất 12 ảnh". Bản v0.8.3 sửa đúng điều đó, và thêm một nút báo lỗi màu đỏ để người dùng có thể nói với tôi nhiều hơn. Cũng hôm ấy, Fotollection có bản cho Linux. Tôi không có máy Linux nào, nhưng Linux có ít ứng dụng ảnh tốt, nên đó là nơi Fotollection có thể trở thành lựa chọn hiển nhiên. Những người cổ vũ đầu tiên không đứng ở vạch đích. Họ đứng dọc đường, chỉ ra chỗ mình đang chạy sai.

## IX. Giữ sức cho đường dài {#long-run}

Đường dài đòi hỏi những thứ không ai nhìn thấy. Ứng dụng chạy offline và không gửi gì đi, nên khi có lỗi, thứ duy nhất người dùng có thể đọc cho tôi là một đoạn mã. Bản v0.9.1 cho mọi lỗi một mã riêng theo dạng `ERR-IMG-0006`, cùng một nhật ký lỗi nằm ngay trên máy, không chứa ảnh, nơi chốn hay tên tệp nào. Nút "Sao chép thông tin chẩn đoán" giúp người dùng dán nó vào một báo cáo.

Có những lỗi khiến tôi phải dừng lại thở. Phóng to một tấm RAW từng làm mọi bản xem trước đứng im năm phút. Không phải vì chậm, mà vì một khoá bị giữ trong lúc nhóm luồng xử lý chờ chính nó. Từ đó có quy tắc: không bao giờ giữ khoá trong lúc công việc song song đang chạy. Ngày 2 tháng 11, tôi viết lại định hướng cho phần lõi bằng một câu ngắn mà tôi muốn giữ đến cuối: phần mềm đơn giản, nhưng không bao giờ cẩu thả.

## X. Chạy qua Praha {#prague}

Ngày 3 tháng 11, tôi muốn có một thứ để gửi cho người ở xa: những tấm bưu thiếp in từ chính ảnh của mình. Một nửa tấm in là ảnh, nửa kia là mặt sau bưu thiếp với ô dán tem, dòng kẻ và chữ gõ như máy đánh chữ, xếp theo đúng bản phác thảo của tôi. Bản v0.9.4 đưa bưu thiếp vào chế độ Ghép ảnh. Sau đó là một ô tem nhỏ hơn để tem dán lên che kín, một dòng địa chỉ thứ tư khi cần, và một dấu đỏ khi chữ không vừa.

Rồi tôi đem ứng dụng vào một cuộc thử thật: nguyên một thư mục chép ra từ chiếc Lumix GX9 sau một cuối tuần ở Praha, 396 ảnh JPEG 20 MP và 26 tệp RAW, tổng cộng 4,7 GB. Chuyến đi thật làm vỡ hai mục tiêu mà những bộ ảnh thử chưa từng chạm tới: xuất chỉ được 7,6 ảnh mỗi giây, và bộ nhớ lên tới 2,14 GB, vượt mức trần 2 GB. Tôi đo lại từng bước. Hoá ra bộ mã hoá giữ toàn bộ dữ liệu của mỗi tấm ảnh chỉ để làm tệp nhỏ hơn chưa tới 1 %. Một nửa số bộ nhớ "còn lại" thì lại là của chính chương trình đo. Bản v0.9.7 xuất những tấm JPEG ấy ở 10,1 ảnh mỗi giây, và cả lượt chạy không vượt quá 1,31 GB. Cũng trong những ngày ấy, một người dùng góp ý rằng xuất ảnh theo từng ngày làm họ phải đi tìm: từ bản v0.9.6, mặc định là mọi ảnh vào một thư mục, đúng như một chiếc USB mang ra tiệm in cần.

Những tấm ảnh tàu điện đêm ở Praha giờ nằm trên trang chủ, thành một tấm bưu thiếp đề địa chỉ Bảo tàng Quốc gia, với dòng "Ahoj Praha!". Kỷ niệm của một cuối tuần đã thành một thứ có thể dán tem và gửi đi.

## XI. Còn bao xa nữa {#how-far}

Mười ba ngày, hơn ba trăm bảy mươi commit, hơn hai mươi bản phát hành. Tính theo cuộc marathon thì tôi mới qua những cây số đầu. Phía trước là bản v0.10.0 "Core", với bộ xử lý ảnh RAW của riêng Fotollection, rồi v0.11.0 "Looks", với những màu ảnh từ tệp LUT. Xa hơn nữa là phiên bản 1.0.

Tôi hay tự hỏi mình có thể đưa Fotollection đi xa đến đâu. Liệu một ngày nó có thể giống những dự án mã nguồn mở mà cả cộng đồng biết đến, những repo có hàng chục nghìn người dùng tin tưởng mỗi ngày? Tôi chưa biết. Điều tôi biết là mỗi tấm ảnh được in ra là một kỷ niệm không còn nằm sau tấm kính, như những cuốn album của ông bà ngày trước. Nếu bạn đọc tới đây, cảm ơn bạn đã chạy cùng tôi một đoạn. Con đường vẫn còn dài, và tôi vẫn đang chạy.
