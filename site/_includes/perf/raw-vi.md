### Cảm biến thật sự ghi lại gì

Cảm biến của máy ảnh là một lưới gồm hàng triệu ô nhỏ, gọi là điểm cảm quang. Khi màn trập mở, mỗi ô gom ánh sáng chiếu tới và biến thành điện tích; khi màn trập đóng, một bộ chuyển đổi đổi từng điện tích thành một con số. Quan hệ này **tuyến tính**: ánh sáng gấp đôi thì con số gấp đôi. Mắt người thì không tuyến tính chút nào, ta cảm nhận độ sáng theo tỉ lệ, vì thế một tệp RAW trông tối và nhạt cho tới khi được xử lý.

Có hai loại cảm biến đọc lưới đó ra:

- **CCD** (charge-coupled device) chuyển điện tích của từng ô, lần lượt từng hàng, tới một bộ khuếch đại duy nhất ở mép chip. Đây là cảm biến của những máy ảnh số đầu tiên và của nhiều lưng máy medium format thời đầu: ảnh sạch, nhưng đọc chậm và tốn điện.
- **CMOS** cho mỗi ô một bộ khuếch đại riêng và đọc nhiều ô cùng lúc. Nhanh hơn, ít tốn điện hơn, rẻ hơn, nên gần như mọi máy ảnh và điện thoại ngày nay đều dùng. Phần lớn cảm biến CMOS đọc lần lượt từng hàng, vì thế vật chuyển động nhanh có thể bị nghiêng khi chụp bằng màn trập điện tử (*rolling shutter*), còn cảm biến xếp chồng (stacked) đọc nhanh hơn nhiều nên ít nghiêng hơn.

Các con số có nhiều bậc hơn JPEG rất nhiều. Máy ảnh lưu 12 hoặc 14 bit cho mỗi ô, tức 4.096 hoặc 16.384 mức, trong khi JPEG chỉ có 8 bit, 256 mức. Chính những mức dư ra đó cho phép kéo sáng vùng tối hay hạ bớt bầu trời quá sáng mà không bị vằn. Và không có ánh sáng cũng không phải là số 0: cảm biến nào cũng cộng thêm một khoảng nhỏ, gọi là **mức đen**, để nhiễu quanh màu đen không bị cắt mất. Trong tệp mẫu của Nikon D750, mức đen là 600, còn giá trị sáng nhất cảm biến giữ được, tức **mức trắng**, là 15.520. Việc đầu tiên của mọi bộ xử lý RAW là đưa hai con số đó về 0 và 1, cho từng vị trí của tấm lọc màu, vì có máy cho mỗi vị trí một mức đen riêng.

### Màu đến từ tấm lọc trên mỗi điểm cảm quang

Một điểm cảm quang chỉ đo được lượng ánh sáng tới, không bao giờ biết màu. Để thấy màu, cảm biến được phủ một **ma trận lọc màu**: trên mỗi điểm có một tấm lọc nhỏ màu đỏ, xanh lá hoặc xanh dương, nên mỗi điểm chỉ đếm một phần của quang phổ.

- **Bayer**, do Bryce Bayer ở Kodak đăng ký bằng sáng chế năm 1976, lặp lại một ô vuông 2 × 2: một đỏ, hai xanh lá, một xanh dương. Một nửa số điểm là xanh lá vì mắt người cảm nhận chi tiết chủ yếu ở phần giữa của quang phổ, nên màu xanh lá mang phần lớn độ nét. Gần như mọi máy ảnh dùng kiểu này, theo một trong bốn chiều (RGGB, BGGR, GRBG, GBRG) mà tệp phải ghi rõ.
- **X-Trans**, mẫu của Fujifilm từ năm 2012, lặp lại một ô 6 × 6, hàng nào và cột nào cũng có đủ đỏ, xanh lá, xanh dương. Cách sắp xếp không đều đó làm ảnh ít bị vân moiré màu trên chi tiết lặp lại nhỏ, nên Fujifilm có thể bỏ tấm lọc làm mờ mà nhiều cảm biến khác phải có, đổi lại việc dựng ảnh khó hơn.
- **Các kiểu khác:** cảm biến đơn sắc không có tấm lọc nào, nên điểm nào cũng thấy đủ chi tiết (dòng Monochrom của Leica); Foveon của Sigma xếp ba lớp hấp thụ màu ở những độ sâu khác nhau trong silicon; nhiều điện thoại gộp bốn điểm cùng màu (*Quad Bayer*) và đọc như một điểm khi thiếu sáng.

Vậy một cảm biến Bayer 24 MP ghi 24 triệu con số, mỗi số chỉ của một màu: mỗi điểm ảnh thiếu hai trong ba màu. Biến bức khảm đó thành bức ảnh là việc của demosaic, ở phần dưới.

### Trong tệp RAW có gì

JPEG là một bức ảnh đã hoàn thiện: máy ảnh đã quyết định mọi thứ. Tệp RAW giữ lại các con số của cảm biến từ trước những quyết định đó, cùng mọi thứ cần để quyết định sau:

- chính các con số, mỗi điểm cảm quang một số, và điểm đó nằm dưới tấm lọc màu nào;
- mức đen và mức trắng;
- **vùng hoạt động** (những điểm có nhận ánh sáng; vài hàng và cột ở mép được cố ý che lại để đo mức đen) và **vùng cắt mặc định** của máy;
- **cân bằng trắng** máy đã chọn lúc chụp;
- các **ma trận màu**, cho biết đỏ, xanh lá, xanh dương của riêng cảm biến này ứng với những màu người ta thấy ra sao;
- một **bản xem trước**, tức ảnh JPEG do chính máy ảnh làm, với màu và tông của nó, cùng dữ liệu EXIF.

Mỗi hãng có định dạng riêng, phần lớn dựa trên TIFF và một phần không công bố: CR2 và CR3 (Canon), NEF (Nikon), ARW (Sony), RW2 (Panasonic), ORF (Olympus), RAF (Fujifilm). **DNG** là định dạng Adobe công bố công khai (phiên bản 1.7), được Ricoh, Leica, Pentax và nhiều điện thoại dùng, và cũng là định dạng mà các quy tắc trên trang này dựa theo.

Fotollection đọc DNG bằng bộ đọc của riêng mình, viết theo bản đặc tả, còn các định dạng khác được giải nén bằng rawler, một thư viện LGPL có sẵn trong Fotollection. Ảnh nhỏ trong lưới không bao giờ đụng tới dữ liệu cảm biến: chúng lấy từ bản xem trước trong tệp, mất 9 đến 32 ms, vì thế một thư mục RAW mở nhanh như một thư mục JPEG.

### Demosaic: tính ra hai màu còn thiếu

Dựng lại hai màu còn thiếu của mỗi điểm ảnh từ các điểm xung quanh gọi là **demosaic** (khử khảm), và đây là bước quyết định nhiều nhất việc ảnh RAW nét và sạch đến đâu. Cách nào cũng gặp cùng một thế khó: lấy trung bình các điểm bên cạnh thì an toàn ở vùng phẳng, nhưng ở cạnh vật thì lại trộn hai bề mặt khác nhau, nên cái khó là biết những điểm nào thuộc về cùng một chỗ.

- **Bilinear** chỉ lấy trung bình các điểm gần nhất cùng màu. Đây là mức sàn để mọi cách khác so vào: mềm, và có viền màu dọc mọi cạnh sắc, vì đỏ, xanh lá, xanh dương được lấy trung bình ở những chỗ hơi khác nhau.
- **Malvar–He–Cutler** (2004) bắt đầu từ trung bình đó rồi hiệu chỉnh bằng chi tiết của chính màu mà điểm ấy đã thấy, qua các bộ lọc 5 × 5 cố định. Nét hơn mà vẫn rất nhanh. Fotollection dùng cách này cho tới v0.10.0.
- **PPG** (patterned pixel grouping) trước hết xem ảnh ít thay đổi nhất theo hướng nào, rồi chỉ nội suy theo hướng đó. rawler dùng cách này, và bản Windows, Linux của Fotollection cũng dùng nó cho phần lớn định dạng cho tới v0.10.0.
- **RCD** (ratio corrected demosaicing, của Luis Sanz Rodríguez) làm việc trên tỉ lệ giữa các màu thay vì hiệu số, nên đứng vững cả ở vùng sáng lẫn vùng tối, và nổi tiếng ít lỗi trên chi tiết nhỏ; darktable và RawTherapee đều có.
- **Markesteijn**, theo tên Frank Markesteijn, là cách dành riêng cho X-Trans, chạy một lượt, hoặc ba lượt chậm hơn mà sạch hơn.
- **Cách demosaic của Fotollection từ v0.10.0** lấy cách quyết định hướng của RCD, rồi ước lượng màu xanh lá từ từng phía, hiệu chỉnh theo chính màu của điểm đó (như Hamilton và Adams), và cân theo mức thay đổi của mỗi phía. Chúng tôi viết bước tỉ lệ của RCD trước, nhưng nó cho điểm kém hơn trên các ảnh chuẩn, nên biến thể thắng mới là thứ được dùng, dưới đúng tên của nó.

Khi đoán sai, lỗi hiện ra theo bốn kiểu quen thuộc:

- **zipper (răng cưa):** một hàng điểm sáng tối xen kẽ dọc theo cạnh sắc, khi cách tính lấy điểm từ cả hai phía của cạnh;
- **mê cung:** những vân nhỏ như giun trên vùng phẳng, khi hai điểm xanh lá trong một ô Bayer đọc không hoàn toàn giống nhau (chúng nằm trên hai hàng có hàng xóm khác nhau, và trên nhiều cảm biến lệch nhau một hai phần trăm);
- **màu giả:** chấm hoặc viền màu trên chi tiết vốn màu xám, như vải, tóc, tường gạch hay hàng rào ở xa;
- **moiré:** những gợn sóng màu trên hoa văn lặp lại rất nhỏ, nhỏ hơn mức tấm lọc màu lấy mẫu được.

**Cách chúng tôi chấm điểm.** Mắt người dễ bị một kết quả trông nét hơn đánh lừa, nên mọi cách đều được chấm trên những ảnh đã biết chính xác đáp án. Máy tính vẽ năm ảnh chuẩn 768 px: một zone plate (các vòng tròn mảnh dần về phía mép, tới mức chi tiết nhỏ nhất các điểm xanh lá còn giữ được), một hình sao Siemens 72 nan, các dải màu đậm trên nền xám theo ba hướng, các dải chuyển màu mượt, và một nền xám phẳng với hai điểm xanh lá lệch nhau 2 %. Mỗi ảnh được biến thành đúng những gì một cảm biến Bayer sẽ ghi, demosaic, rồi so với ảnh gốc. PSNR đo sai số chung theo decibel (càng cao càng tốt; cứ 6 dB thì sai số giảm một nửa); màu giả là lượng màu trung bình xuất hiện trên các ảnh xám; sai số ở cạnh là sai số trung bình trong phạm vi 2 px quanh cạnh.

| Ảnh chuẩn (768 px) | Bilinear | Malvar (tới v0.10.0) | Của mình từ v0.10.0 |
|---|---|---|---|
| Zone plate: PSNR, càng cao càng tốt | 13,0 dB | 17,3 dB | 16,5 dB |
| Zone plate: màu giả, càng thấp càng tốt | 65,9 | 43,0 | 18,7 |
| Sao Siemens: PSNR | 21,3 dB | 25,2 dB | 37,9 dB |
| Sao Siemens: màu giả | 9,4 | 8,4 | 0,16 |
| Dải màu đậm: sai số dọc theo cạnh, càng thấp càng tốt | 11,6 | 21,6 | 10,5 |
| Dải chuyển màu: PSNR | 78,9 dB | 77,5 dB | 78,4 dB |
| Nền xám lệch xanh lá: mê cung, càng thấp càng tốt | 0,38 | 0,38 | 0 |
| Vân mịn có điểm chết: còn lại trên 10.000, càng thấp càng tốt | 35,8 | 66,5 | 0 |

Malvar tốt hơn bilinear khoảng 4 dB trên chi tiết xám nhỏ, đúng như bài báo của nó hứa, nhưng kém hơn ở cạnh giữa hai màu đậm, vì phép hiệu chỉnh mượn chi tiết của một màu cho các màu kia và giả định chúng thay đổi cùng nhau, điều mà một biển báo đỏ trên bức tường xám không làm. Cả hai đều không xoá được vân mê cung, vốn không phải lỗi demosaic mà là chênh lệch giữa hai điểm xanh lá, phải cân lại từ trước. Từ v0.10.0, chuỗi của mình lấp những khoảng trống đó: điểm chết và chênh lệch xanh lá được sửa khi mỗi điểm còn giữ đúng thứ nó đo được, cách demosaic theo hướng dựng lại màu, và một phép trung vị nhỏ trên hiệu màu xoá màu giả còn sót. Trên các ảnh chuẩn, nó thắng ở mọi chỗ trừ PSNR chung của zone plate, thấp hơn Malvar 0,8 dB ở vùng chi tiết nhỏ nhất, nơi cách nào cũng bị răng cưa; ngay cả ở đó màu giả vẫn giảm một nửa. Trên chín tệp thật, nó đưa kết quả của mọi máy Bayer gần JPEG của máy bằng hoặc hơn trước, và tốn khoảng 180 ms cho ảnh 24 MP, so với 40 ms của Malvar: thêm vài phần mười giây mỗi ảnh, đổi lại ảnh in có chi tiết sạch hơn.

### Từ màu của máy ảnh tới màu bạn thấy

Đỏ, xanh lá, xanh dương của máy ảnh không phải của màn hình: tấm lọc của mỗi cảm biến dùng thuốc nhuộm riêng, và chúng chồng lên nhau khác với các tế bào nón trong mắt ta. Các ma trận màu trong tệp chuyển màu của máy ảnh sang **CIE XYZ**, mô hình chuẩn về cách mắt người thấy màu, định nghĩa năm 1931, rồi từ đó sang **sRGB**, không gian màu của phần lớn màn hình và các tiệm in ảnh.

- **Cân bằng trắng** đi trước: nhân thêm cho kênh đỏ và kênh xanh dương để vật màu trắng trong cảnh ra đúng màu trung tính. Máy ảnh ghi lại các hệ số nó đã chọn, và Fotollection áp chúng trước khi demosaic, để chênh lệch giữa các màu, thứ mà các cách tính ở trên dựa vào, luôn nhỏ.
- Một tệp DNG thường có **hai ma trận**: một đo dưới đèn sợi đốt ấm, một dưới ánh sáng ban ngày. Ma trận đúng cho tấm ảnh của bạn nằm ở giữa: bộ xử lý suy ra ánh sáng lúc chụp từ cân bằng trắng, dưới dạng nhiệt độ màu, rồi pha hai ma trận lại. Fotollection làm y như vậy với các định dạng khác, từ các ma trận rawler biết cho từng dòng máy.
- Mọi việc trên diễn ra trong **ánh sáng tuyến tính**, theo đúng thang của cảm biến, trước mọi đường cong. Pha trộn, nhân và thu phóng màu trong ánh sáng tuyến tính mới cho kết quả đúng về mặt vật lý; làm sau đường cong thì cạnh tối đi và màu bị lệch.

**Màu riêng của máy ảnh.** Ảnh JPEG của chính máy ảnh không trung tính: mỗi hãng thêm độ tương phản, màu sắc và tông da của riêng mình, và chất màu đó là một phần lý do người ta chọn một hãng. Từ v0.10.0, ảnh RAW mở ra với màu **trung tính**, trung thành với ma trận của cảm biến, giữ độ tối góc và độ sáng máy đã chọn; **Như JPEG của máy** chỉ cách một cú nhấp trong thẻ *Màu RAW* của ảnh RAW, cho một ảnh hoặc mọi ảnh RAW cùng lúc. (Trước v0.10.0, mọi ảnh RAW đều được khớp theo JPEG của máy; project lưu từ hồi đó vẫn giữ màu ấy.) Lựa chọn này khớp theo JPEG của máy bằng cả hai cách, một đường cong cùng một ma trận hoặc mỗi kênh một đường cong, rồi giữ cách nào gần hơn với từng tệp: với Nikon D750, chênh lệch giảm một nửa, từ 2,26 xuống 1,16 theo ΔE2000. Ảnh nhỏ trong lưới vẫn giữ bản xem trước của máy, như Lightroom.

### Tông màu: từ con số tuyến tính thành bức ảnh

Các con số tuyến tính trông tối và nhạt trên màn hình, vì ánh sáng gấp đôi, thứ cảm biến ghi thành con số gấp đôi, với ta chỉ là sáng hơn một bậc. Một **đường cong tông** biến thang của cảm biến thành bức ảnh: đặt vị trí màu xám giữa, nâng vùng tối, cho vùng sáng chuyển êm thay vì cháy ngay, và cho ảnh độ tương phản. Sau đó ảnh được lưu theo đường cong sRGB (cái gọi là "gamma"), dành nhiều trong 256 mức của JPEG hơn cho vùng tối, nơi mắt ta dễ thấy các bậc nhất.

Có hai việc máy ảnh sửa trong JPEG mà dữ liệu RAW không có, và cả hai đều quan trọng với ảnh in:

- **Tối góc:** ống kính nào cũng tối dần về phía các góc. Fotollection đo xem máy ảnh đã làm sáng các góc bao nhiêu, từ chính bản xem trước của máy, dưới dạng một hệ số theo bán kính. Lần đầu so kết quả DNG của chúng tôi với JPEG của máy, phần giữa sáng hơn khoảng 4 L\* và các góc tối hơn 1,5; hệ số theo bán kính đưa chênh lệch từ ΔE 3,8 xuống 2,67, mọi vùng của ảnh đều lệch trong khoảng ±0,5 L\*.
- **Méo hình:** độ phình hoặc thắt nhẹ của ống kính. Máy Micro Four Thirds (Panasonic, Olympus) được thiết kế để máy ảnh tự sửa, và tệp của chúng ghi lại cách sửa. Trên máy Mac, các tệp đó được macOS xử lý, và macOS áp dụng việc sửa méo; tự sửa méo trong bộ xử lý của chúng tôi đã có trong kế hoạch (FR-085).

### Fotollection xử lý một ảnh RAW, từng bước

Cho một lần xử lý đầy đủ, thứ đứng sau bản xem trước để in, phóng to và xuất ảnh:

1. **Nhận biết** định dạng từ những byte đầu tiên của tệp, không bao giờ dựa vào tên, để tệp bị đổi tên hay tệp RAW trông như TIFF thường vẫn được đọc đúng.
2. **Đọc** các con số của cảm biến: DNG bằng bộ đọc của riêng mình, các định dạng khác do rawler giải nén. Tệp quá lớn hoặc hỏng bị từ chối trước khi giải mã bất kỳ điểm ảnh nào, kèm một mã lỗi.
3. **Mức:** đưa đen về 0 và trắng về 1, cho từng vị trí của tấm lọc màu, tính từ góc của vùng hoạt động.
4. **Cân bằng trắng**, rồi **demosaic**: sửa điểm chết và cân xanh lá trước, demosaic theo hướng, trung vị chống màu giả (Markesteijn cho X-Trans có ở v0.15.0).
5. **Màu:** các ma trận của máy ảnh, pha theo ánh sáng lúc chụp, trong ánh sáng tuyến tính.
6. **Tông và màu riêng**, khớp theo bản xem trước của máy, kèm độ tối góc.
7. Từ đây ảnh RAW giống mọi ảnh khác: cắt khung, dấu ngày, khung viền, cỡ in và hồ sơ màu, xuất ở độ phân giải đầy đủ, 4:4:4, chất lượng 95.

Vài nguyên tắc luôn giữ. Mỗi lần chỉ xử lý một ảnh, chia ra mọi nhân của bộ xử lý, và không bao giờ trên luồng của cửa sổ, nên Fotollection vẫn phản hồi trong lúc một ảnh 30 MP đang được xử lý. Xuất ảnh luôn chạy trên bộ xử lý, để ảnh in ra giống hệt nhau trên mọi máy; card đồ hoạ chỉ tăng tốc những gì bạn thấy khi chỉnh. Và mã nguồn là của chúng tôi, viết theo các bản đặc tả và bài báo đã công bố, không chép từ dự án khác, rồi được kiểm tra: trên các ảnh chuẩn tổng hợp ở trên, trên chín tệp RAW từ chín dòng máy (CC0, từ raw.pixls.us), và bằng cách cố tình cắt cụt, làm hỏng tệp để tệp lỗi chỉ báo lỗi, không bao giờ làm Fotollection dừng đột ngột.

**Từ v0.10.0: bộ xử lý của riêng Fotollection cho mọi máy Bayer.** rawler chỉ còn giải nén tệp, phần còn lại là của Fotollection. Thời gian xử lý đầy đủ chín tệp mẫu, trung vị của 3 lần, trên cùng một máy: bộ xử lý của mình với cách demosaic sạch hơn, so với cách đơn giản hơn của rawler và với macOS:

| Máy ảnh (định dạng) | Của Fotollection | rawler | macOS |
|---|---|---|---|
| Panasonic GX9 (RW2, 20 MP) | 280 ms | 216 ms | 436 ms |
| Panasonic S5 (RW2, 24 MP) | 442 ms | 301 ms | 1.044 ms |
| Olympus E-M10 III (ORF, 16 MP) | 308 ms | 364 ms | 1.164 ms |
| Sony A7 III (ARW, 24 MP) | 372 ms | 241 ms | 488 ms |
| Canon R10 (CR3, 24 MP) | 494 ms | 360 ms | 1.647 ms |
| Ricoh GR III (DNG, 24 MP) | 523 ms | 650 ms | — |
| Nikon D750 (NEF, 24 MP) | 502 ms | 402 ms | 412 ms |
| Canon 5D Mark IV (CR2, 31 MP) | 691 ms | 565 ms | 418 ms |
| Fujifilm X-T3 (RAF, 26 MP, X-Trans) | tiếp theo | 524 ms | 1.339 ms |

Bộ xử lý của mình mất thêm một phần mười tới một phần tư giây so với rawler, cho cách demosaic sạch hơn và màu được khớp, và vẫn nhanh hơn macOS ở phần lớn máy; việc này chỉ diễn ra một lần mỗi ảnh, sau đó mỗi lần kéo thanh chỉ hoàn thiện lại một bản giữ sẵn trong 4 đến 11 ms. Trên máy Mac, tệp Panasonic và Olympus vẫn do macOS xử lý để được sửa méo ống kính, cho tới khi bộ xử lý của mình tự sửa méo. Các mảng xám của S5 ra trung tính hơn so với qua rawler (độ ám màu 1,42 so với 2,43, trong đó 0 là xám tuyệt đối). Tiếp theo là Fujifilm X-Trans với Markesteijn, rồi sau đó là xử lý trên card đồ hoạ cho bản xem trước, lấy kết quả của bộ xử lý làm chuẩn.
