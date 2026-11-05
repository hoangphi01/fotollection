## I. Vạch xuất phát {#start}

Một ngày, tôi nhận ra mình đã sống được hai mươi năm. Trong chiếc điện thoại có hàng chục nghìn tấm ảnh, và tôi gần như chưa bao giờ mở lại chúng. Những gì của năm, sáu năm trước bắt đầu nhạt đi trong trí nhớ: một buổi chiều, một khuôn mặt, tên một con phố. Ảnh vẫn còn đó, nhưng chúng nằm im trong một thư mục mà tôi chẳng bao giờ ghé tới.

Rồi tôi sang châu Âu học. Mùa đông ở đây dài và tối sớm, nên có nhiều khoảng lặng hơn tôi tưởng. Trong những khoảng lặng ấy tôi ngồi lướt lại những năm tháng cũ, và thấy một nỗi buồn man mác: những kỷ niệm đẹp, những câu chuyện còn dở dang, tất cả nằm sau một tấm kính. Ông bà tôi có những cuốn album dày, mỗi tấm ảnh có góc dán, có dòng chữ viết tay ở mặt sau. Thế hệ của tôi thì chụp nhiều hơn bất kỳ thế hệ nào trước đó, nhưng lại giữ được ít hơn. Tôi nghĩ: cứ bắt đầu từ đó đã. Tôi muốn làm một thứ gì đó để những kỷ niệm ấy trở lại thành vật cầm được trên tay.

Tôi coi việc này như một cuộc chạy marathon. Không ai chạy hết bốn mươi hai cây số trong một hơi, nên tôi chỉ hỏi mình bước đầu tiên là gì. Vào tháng 9 năm 2026, bước đầu tiên là bản kế hoạch cho một trang web tên "PhotoOrder". Những điều tôi viết ra khi ấy đến giờ vẫn còn nguyên: người ta chụp hàng nghìn tấm mà in chẳng được mấy; ảnh in ra không ghi ngày, không ghi nơi, vài năm sau chẳng ai nhớ; tiệm in tự cắt ảnh về 2:3, 3:4 hay vuông, có khi cắt mất cả đầu người và đường chân trời; còn các công cụ trực tuyến thì đòi tải ảnh lên, trong khi vị trí ghi trong ảnh có thể chỉ thẳng tới nhà mình.

## II. Những bước chạy đầu tiên {#first-steps}

Bản vá đầu tiên của dự án ra đời vào chiều ngày 23 tháng 10 năm 2026, và trong đó chỉ có ba trang giấy: một bản kế hoạch, một ghi chú về an toàn và một trang theo dõi tiến độ, chưa có dòng mã nào. Ba mươi lăm phút sau, dự án đã có tên mới, Fotollection, và một hướng đi khác hẳn. Tôi bỏ trang web để làm một ứng dụng chạy thẳng trên máy tính, hoàn toàn không cần mạng, với phần lõi viết bằng ngôn ngữ Rust. Lý do được ghi ngay trong quyết định đầu tiên: người dùng là những người chụp ảnh nghiêm túc, nên tốc độ và chất lượng bản in phải đứng trước. Một trình duyệt thì giới hạn bộ nhớ cho mỗi trang, và không cho tôi nắm chắc cách một tấm ảnh được nén.

Cùng ngày hôm đó, vài nguyên tắc được đặt xuống và đến giờ vẫn giữ nguyên. Tên địa điểm được tra ngay trên máy của bạn, từ một bộ dữ liệu địa danh đóng gói sẵn, chứ không gửi toạ độ của bạn cho bất kỳ máy chủ nào. Mã nguồn được mở theo giấy phép GPL, để không ai có thể biến nó thành một bản đóng kín. Bộ cài thì chưa có chữ ký số, vì đây là một dự án miễn phí, không có tiền cho những chứng chỉ phải trả hằng năm. Và mỗi khi mã thay đổi, phần ghi chép về nó thay đổi theo ngay trong cùng một bản vá, một thói quen về sau giữ cho cả dự án không bị lạc đường.

Tối hôm đó, lát cắt đầu tiên chạy được từ đầu đến cuối: nhập ảnh, xem trước, đóng dấu ngày, xuất ảnh. Rồi tôi làm việc mà người chạy nào cũng nên làm sớm: bấm giờ. Tôi đặt bản viết bằng Rust cạnh một bản chạy trong trình duyệt, cùng 95 tấm ảnh 16 megapixel trên cùng một chiếc máy. Kết quả làm tôi bất ngờ: tốc độ xuất ảnh gần như ngang nhau, 18,0 ảnh mỗi giây so với 17,9. Điều tôi đoán ban đầu, rằng trình duyệt sẽ chậm, đã sai, và tôi ghi lại đúng chữ "sai" ấy. Lý do thật để làm một ứng dụng riêng nằm ở chỗ khác: ảnh in giữ trọn màu của từng điểm ảnh thay vì nén màu đi một nửa, bộ nhớ có giới hạn kể cả với năm nghìn tấm, hồ sơ màu của ảnh được giữ lại, và tệp được ghi thẳng ra ổ đĩa. Ngay hôm đầu tiên, tôi đã học được rằng con số đo được đáng tin hơn điều mình tin.

## III. Lần đầu vấp ngã {#first-fall}

Bản vá lỗi đầu tiên chẳng liên quan gì đến ảnh: trên Windows, các chương trình kiểm thử không chạy được vì thiếu một tệp khai báo của hệ thống. Một cú vấp nhỏ, nhưng là lời nhắc rằng mỗi hệ điều hành là một mặt đường khác nhau.

Hai ngày sau, những cú vấp bắt đầu dạy tôi nhiều hơn. Có lần một dấu đỏ của hệ thống kiểm tra tự động nằm im qua ba lần tôi đẩy mã lên mà không ai thấy, chỉ vì một danh sách trong phần ghi chép bị đánh số "7." hai lần. Từ đó, việc soát lỗi trình bày chạy luôn trong bước kiểm tra tôi làm trước mỗi bản vá. Có lần mở một tấm ảnh bằng ứng dụng khi ứng dụng đang tắt thì chẳng có gì xảy ra: macOS đã trao tấm ảnh tới trước khi ứng dụng kịp sẵn sàng. Và khung "Shot on" đầu tiên in ra dòng "NIKON CORPORATION NIKON D750", vì tên hãng mà máy ảnh ghi vào ảnh là tên một công ty, không phải tên một thương hiệu.

Sau những lần như thế, tôi bắt đầu ghi lại những bài học. Mỗi lỗi phải nhìn hơn một lần mới tìm ra đều được chép vào đó: đã thấy gì, vì sao, sửa thế nào, và từ nay phải làm gì. Đó là thứ tôi đọc đầu tiên mỗi khi có gì hỏng. Chạy đường dài không phải là không bao giờ ngã, mà là ngã ở đâu thì nhớ chỗ đó.

## IV. Bản in không được nhạt {#prints}

Có một điều tôi để ý từ lâu: ảnh in qua các trình sửa ảnh trên mạng thường trông nhạt, màu bệt và loang, thiếu chiều sâu so với ảnh gốc. Ngày 24 tháng 10, tôi đem nỗi nghi ngờ đó ra đo từng điểm ảnh. So với một bản xuất kiểu trình sửa ảnh trên mạng, Fotollection lệch màu trung bình 0,19 thay vì 1,39 (dưới 1 thì mắt người không còn thấy khác), giữ 100 % chi tiết nhỏ thay vì 13 %, giữ 99,9 % độ rực của ảnh chụp bằng điện thoại thay vì 86,8 %, và giữ được hồ sơ màu ở cả 20 trên 20 tấm. Nguyên nhân chẳng có gì bí ẩn: hồ sơ màu bị bỏ đi, ảnh bị thu nhỏ rồi phóng to lại, màu bị nén đi một nửa. Chỉ cần đừng làm những việc đó, bản in sẽ giữ được tấm ảnh.

Những ngày ấy, đích đến cụ thể nhất là những chiếc máy in tự phục vụ ở các cửa hàng dm và Rossmann, và mọi khổ in trong bảng giá dịch vụ ảnh ở đó đều được đưa vào ứng dụng. Trang web ra đời cùng lúc, bằng tiếng Anh và tiếng Việt, mở đầu bằng một câu hỏi tôi vẫn thấy đúng: chụp hàng nghìn ảnh, chỉ xem lại khi ngồi trên máy bay? Ngày 25 tháng 10, trang chuyển về địa chỉ fotollection.org, và cũng ngày hôm ấy có bảy bản phát hành liền nhau, từ v0.3.0 đến v0.7.0: giỏ in, khung xem có phóng to, rồi ảnh RAW. Đó là những cây số chạy nhanh nhất, khi đôi chân còn khoẻ và con đường còn mới.

## V. Lời từ chối bên đường {#refusal}

Ngày 26 tháng 10, tôi gửi đơn xin chữ ký số miễn phí cho bộ cài Windows tới một tổ chức chuyên hỗ trợ việc này cho các dự án mã nguồn mở. Lời từ chối đến ngay trong ngày: họ tìm những dấu hiệu tin cậy từ bên ngoài, như số người theo dõi, người đóng góp, những bài viết độc lập, và dự án của tôi chưa có đủ. Họ nói tôi có thể gửi lại sau. Lời từ chối ấy không làm tôi dừng lại; nó chỉ cho tôi thấy rõ quãng đường còn ở phía trước. Đến giờ bộ cài vẫn chưa có chữ ký, và mỗi bản phát hành đều kèm một mã kiểm tra để ai cũng xác minh được tệp mình tải về là đúng.

Cùng ngày, tôi mắc một lỗi mà tôi vẫn tự nhắc mình: trang web và ghi chú của bản v0.7.0 viết rằng ảnh RAW mở được mà không cần cài gì thêm. Trên Windows, điều đó không đúng, và chính tôi phát hiện ra chứ không phải một bài kiểm thử nào. Bài học thành một quy tắc: không bao giờ viết "hỗ trợ" khi mới thử trên một hệ điều hành. Và cũng từ đó, tôi quyết định ảnh RAW phải được đọc bằng mã nằm ngay trong Fotollection, không phụ thuộc vào thứ người dùng phải tự cài.

## VI. Tự đóng đôi giày của mình {#shoes}

Một người chạy đường dài rồi sẽ muốn có đôi giày vừa chân mình. Ngày 27 tháng 10, Fotollection có bộ đọc ảnh RAW của riêng nó, bắt đầu từ định dạng DNG. Tôi muốn màu đẹp ngang những phần mềm lớn mà không bắt ai phải cài thêm gì. Với một tệp DNG chụp từ chiếc Ricoh GR III, bộ đọc mới xử lý xong trong 355 phần nghìn giây, dùng 526 MB bộ nhớ, và màu chỉ lệch 2,67 so với tấm ảnh JPEG chính máy ảnh tạo ra. Bộ đọc tôi dùng trước đó cần 640 phần nghìn giây, 1,33 GB bộ nhớ và lệch 4,08.

Con số 2,67 có một câu chuyện riêng. Khi bản DNG của tôi còn lệch thêm khoảng một đơn vị, tôi đinh ninh lỗi nằm ở màu. Vẽ bản đồ sai lệch ra mới thấy phần giữa ảnh sáng hơn rìa: đó là độ tối ở góc của ống kính, không phải màu. Bài học ghi lại khi ấy: vẽ bản đồ lỗi theo từng vùng trước khi thêm điều chỉnh. Cùng tuần, bản v0.8.0 ra đời với tên "Places and Lightroom": sửa đồng hồ bị lệch của máy ảnh, đặt ngày và nơi chụp bằng tay, đọc số sao đã chấm trong Lightroom. Rồi có hai ngày gần như im lặng, ngày 28 và 29, quãng nghỉ duy nhất trong cả cuộc chạy.

## VII. Tìm được nhịp chạy {#rhythm}

Đến cuối tháng 10, ứng dụng đã làm được rất nhiều thứ. Nhưng càng nhiều tính năng thì nó càng khó hiểu với người mới. Ngày 31 tháng 10, tôi tách nó thành ba chế độ. **Đơn giản** là đường ngắn nhất từ một thư mục tới những tấm ảnh in: xem từng tấm, giữ hay bỏ, rồi xuất. **Ghép ảnh** đặt hai ảnh lên một tấm in, như chiếc máy ảnh phim half-frame từng chụp hai khung trên một ô phim, nên in cả bộ chỉ tốn một nửa. **Pro** là tất cả những gì đã xây trước đó.

Đó là lúc Fotollection tìm được nhịp chạy của mình. Nó không cố làm một phần mềm quản lý ảnh đủ thứ việc, mà là một con đường ngắn và rõ ràng, từ chiếc điện thoại hay thẻ nhớ tới tờ giấy ảnh trên tay. Bản v0.9.0 "Three modes" ra ngày 2 tháng 11. Những lỗi của giai đoạn này cũng thành quy tắc: một ô nhập khiến cả bảng nhảy lên khi gõ đã sinh ra nguyên tắc "không gì nhấp nháy, không gì nhảy". Một tấm ghép xuất xong trong 214 phần nghìn giây, nhanh hơn xuất riêng hai tấm ảnh của nó (633 phần nghìn giây), và lưới các tấm ghép được vẽ lại từ ảnh thu nhỏ chỉ trong 29 phần nghìn giây thay vì hơn bốn giây.

## VIII. Những người cổ vũ đầu tiên và những tấm ảnh xanh {#cheering}

Ngày 1 tháng 11 là ngày đầu tiên những người dùng thật lên tiếng, và họ cho tôi thấy điều mà không bài kiểm thử nào chỉ ra. Một người dùng bản v0.8.0 mở thư mục của họ và thấy nửa số ảnh hiện thành những ô màu xanh có sọc ngang mảnh. Tôi nghi bộ đọc ảnh RAW trước, nhưng những tệp ấy là JPEG. Bộ đọc JPEG nhanh mà tôi dùng đã đọc sai một kiểu tệp hiếm gặp, và vẫn báo là thành công. Bản v0.8.1 "Green photos fixed" đặt thêm một bộ đọc chuẩn phía sau bộ đọc nhanh, và từ đó có một quy tắc tôi coi là cốt lõi: một bộ đọc nhanh chỉ được tin ở những kiểu tệp mà một bài kiểm thử đã chứng minh.

Cùng ngày, một góp ý khác: đánh dấu chọn ba tấm ảnh mà nút xuất lại ghi "Xuất 12 ảnh". Bản v0.8.3 sửa đúng điều đó, và thêm một nút báo lỗi màu đỏ để người dùng có thể nói với tôi nhiều hơn. Cũng hôm ấy, Fotollection có bản cho Linux. Tôi không có chiếc máy Linux nào, nhưng Linux có ít ứng dụng ảnh tốt, nên đó là nơi Fotollection có thể trở thành lựa chọn hiển nhiên. Những người cổ vũ đầu tiên không đứng ở vạch đích; họ đứng dọc đường, chỉ ra chỗ mình đang chạy sai.

## IX. Giữ sức cho đường dài {#long-run}

Đường dài đòi hỏi những thứ không ai nhìn thấy. Ứng dụng chạy không cần mạng và không gửi gì đi, nên khi có lỗi, thứ duy nhất người dùng có thể đọc cho tôi là một đoạn mã. Từ bản v0.9.1, mỗi lỗi có một mã riêng, kiểu `ERR-IMG-0006`, cùng một sổ ghi lỗi nằm ngay trên máy, không chứa ảnh, nơi chốn hay tên tệp nào, và một nút để chép nó vào lời báo lỗi.

Có những lỗi khiến tôi phải dừng lại thở. Phóng to một tấm ảnh RAW từng làm mọi khung xem trước đứng im năm phút, không phải vì chậm, mà vì một phần của chương trình giữ chặt một thứ trong lúc chính nó đang chờ thứ ấy được trả lại. Quy tắc từ đó: không bao giờ giữ chặt gì trong lúc nhiều việc đang chạy song song. Ngày 2 tháng 11, tôi viết lại hướng đi cho phần lõi bằng một câu ngắn mà tôi muốn giữ tới cuối: phần mềm giản dị, nhưng không bao giờ cẩu thả.

## X. Chạy qua Praha {#prague}

Ngày 3 tháng 11, tôi muốn có một thứ để gửi cho người ở xa: những tấm bưu thiếp in từ chính ảnh của mình. Một nửa tấm in là ảnh, nửa kia là mặt sau bưu thiếp, có ô dán tem, có dòng kẻ và chữ gõ như máy đánh chữ, xếp theo đúng bản phác thảo của tôi. Bản v0.9.4 đưa bưu thiếp vào chế độ Ghép ảnh; sau đó là một ô tem nhỏ hơn để tem dán lên che kín, một dòng địa chỉ thứ tư khi cần, và một dấu đỏ khi chữ không vừa.

Rồi tôi đem ứng dụng vào một lần chạy thật: nguyên một thư mục chép ra từ chiếc Lumix GX9 sau một cuối tuần ở Praha, 396 ảnh JPEG 20 megapixel và 26 ảnh RAW, tổng cộng 4,7 GB. Chuyến đi thật làm vỡ hai mục tiêu mà những bộ ảnh thử chưa từng chạm tới: xuất ảnh chỉ được 7,6 tấm mỗi giây, và bộ nhớ lên tới 2,14 GB, vượt mức trần 2 GB. Tôi đo lại từng bước. Hoá ra phần nén ảnh giữ toàn bộ dữ liệu của mỗi tấm chỉ để tệp nhỏ đi chưa tới 1 %, và một nửa số bộ nhớ "còn lại" lại thuộc về chính chương trình đo. Bản v0.9.7 xuất những tấm ảnh ấy ở 10,1 tấm mỗi giây, và cả lần chạy không vượt quá 1,31 GB. Cũng trong những ngày ấy, một người dùng góp ý rằng xuất ảnh theo từng ngày làm họ phải đi tìm; từ bản v0.9.6, mặc định là mọi ảnh vào một thư mục, đúng như một chiếc USB mang ra tiệm in cần.

Những tấm ảnh tàu điện đêm ở Praha giờ nằm trên trang chủ, thành một tấm bưu thiếp đề địa chỉ Bảo tàng Quốc gia, với dòng "Ahoj Praha!". Kỷ niệm của một cuối tuần đã thành một thứ có thể dán tem và gửi đi.

## XI. Còn bao xa nữa {#how-far}

Mười ba ngày, hơn ba trăm bảy mươi bản vá, hơn hai mươi bản phát hành. Tính theo cuộc marathon thì tôi mới qua những cây số đầu tiên. Phía trước là bản v0.11.0, một cửa sổ cho mọi cài đặt, rồi v0.12.0 "Looks", với những tông màu của riêng mỗi người, và xa hơn nữa là phiên bản 1.0.

Tôi hay tự hỏi mình có thể đưa Fotollection đi xa đến đâu. Liệu một ngày nó có thể giống những dự án mã nguồn mở mà cả cộng đồng đều biết, những kho mã có hàng chục nghìn người tin dùng mỗi ngày? Tôi chưa biết. Điều tôi biết là mỗi tấm ảnh được in ra là một kỷ niệm không còn nằm sau tấm kính, như những cuốn album của ông bà ngày trước. Nếu bạn đã đọc tới đây, cảm ơn bạn đã chạy cùng tôi một đoạn. Con đường vẫn còn dài, và tôi vẫn đang chạy.
