## 1. Cảm biến: đếm ánh sáng {#wiki-sensor}

Cảm biến của máy ảnh là một lưới gồm hàng triệu ô nhỏ, gọi là điểm cảm quang. Trong lúc phơi sáng, mỗi ô gom ánh sáng chiếu vào nó và biến thành điện tích; cuối cùng một bộ chuyển đổi đổi điện tích đó thành một con số. Ánh sáng gấp đôi thì con số gấp đôi: cảm biến **tuyến tính**, khác với mắt người.

Có hai cách đọc lưới ra:

- **CCD** (charge-coupled device) chuyển điện tích của từng ô, lần lượt từng hàng, tới một bộ khuếch đại duy nhất ở mép chip. Đây là cảm biến của những máy ảnh số đầu tiên: ảnh sạch, nhưng chậm và tốn điện.
- **CMOS** cho mỗi ô một bộ khuếch đại riêng và đọc nhiều ô cùng lúc. Nhanh hơn, ít tốn điện hơn, rẻ hơn, nên gần như mọi máy ảnh và điện thoại ngày nay đều dùng. Phần lớn đọc lần lượt từng hàng, vì thế vật chuyển động nhanh có thể bị nghiêng khi chụp bằng màn trập điện tử (*rolling shutter*).

Các con số này có nhiều bậc hơn JPEG: máy ảnh lưu 12 hoặc 14 bit cho mỗi ô (4.096 hoặc 16.384 mức), còn JPEG chỉ có 8 bit (256 mức). Và không có ánh sáng cũng không phải là số 0: cảm biến nào cũng cộng thêm một **mức đen** nhỏ. Ví dụ trong tệp mẫu của Nikon D750, mức đen là 600, còn giá trị sáng nhất cảm biến giữ được, tức **mức trắng**, là 15.520. Việc đầu tiên của mọi bộ xử lý RAW là đưa hai con số đó về 0 và 1.

## 2. Màu: một tấm lọc trên mỗi điểm cảm quang {#wiki-cfa}

Một điểm cảm quang chỉ đo được lượng ánh sáng tới, không biết màu. Để thấy màu, cảm biến được phủ một **ma trận lọc màu**: trên mỗi điểm có một tấm lọc nhỏ màu đỏ, xanh lá hoặc xanh dương, nên mỗi điểm chỉ đếm một phần của quang phổ.

- **Bayer**, do Bryce Bayer ở Kodak đăng ký bằng sáng chế năm 1976, lặp lại một ô vuông 2 × 2: một đỏ, hai xanh lá, một xanh dương. Một nửa số điểm là xanh lá vì mắt người cảm nhận chi tiết chủ yếu qua phần xanh lá của ánh sáng. Gần như mọi máy ảnh đều dùng kiểu này.
- **X-Trans**, mẫu của Fujifilm từ năm 2012, lặp lại một ô 6 × 6, hàng nào và cột nào cũng có đủ đỏ, xanh lá, xanh dương. Sự không đều đặn đó làm ảnh ít bị vân moiré hơn, đổi lại việc dựng ảnh khó hơn.
- **Các kiểu khác:** cảm biến đơn sắc không có tấm lọc nào (dòng Monochrom của Leica); Foveon của Sigma xếp ba lớp chồng theo chiều sâu; nhiều điện thoại gộp bốn điểm cùng màu thành một nhóm (*Quad Bayer*).

Vậy một cảm biến Bayer 24 MP ghi 24 triệu con số, mỗi số chỉ của một màu. Mỗi điểm ảnh thiếu hai trong ba màu, và hai màu đó phải được tính ra.

## 3. Trong tệp RAW có gì {#wiki-raw}

JPEG là một bức ảnh đã hoàn thiện. Tệp RAW là các con số của cảm biến, giữ lại từ trước khi có bất kỳ quyết định nào, cùng những thứ cần để đưa ra các quyết định đó:

- chính các con số, mỗi điểm cảm quang một số, và điểm đó nằm dưới tấm lọc màu nào;
- mức đen và mức trắng;
- **vùng hoạt động** (những điểm có nhận ánh sáng; vài điểm ở mép bị che để đo mức đen) và **vùng cắt mặc định** của máy;
- **cân bằng trắng** máy đã chọn lúc chụp;
- các **ma trận màu**, cho biết đỏ, xanh lá, xanh dương của cảm biến này ứng với những màu ta thấy như thế nào;
- một **bản xem trước**, tức ảnh JPEG do chính máy ảnh làm, cùng dữ liệu EXIF.

Mỗi hãng có định dạng riêng, phần lớn dựa trên TIFF và một phần không công bố: CR2 và CR3 (Canon), NEF (Nikon), ARW (Sony), RW2 (Panasonic), ORF (Olympus), RAF (Fujifilm). **DNG** là định dạng Adobe công bố công khai (phiên bản 1.7), được Ricoh, Leica, Pentax và nhiều điện thoại dùng.

Fotollection đọc DNG bằng bộ đọc của riêng mình, viết theo bản đặc tả. Các định dạng khác được giải nén bằng rawler, một thư viện LGPL có sẵn trong app. Ảnh thu nhỏ không bao giờ cần dữ liệu cảm biến: chúng lấy từ bản xem trước trong tệp, mất 9 đến 32 ms.

## 4. Demosaic: tính ra những màu còn thiếu {#wiki-demosaic}

Dựng lại hai màu còn thiếu của mỗi điểm ảnh từ các điểm xung quanh gọi là **demosaic** (khử khảm). Đây là bước quyết định nhiều nhất việc ảnh RAW nét và sạch đến đâu.

- **Bilinear** lấy trung bình các điểm gần nhất cùng màu. Đơn giản và nhanh, nhưng mềm, và để lại viền màu dọc mọi cạnh sắc.
- **Malvar–He–Cutler** (2004) bắt đầu từ trung bình đó rồi hiệu chỉnh bằng chi tiết của chính màu ở điểm ấy. Nét hơn mà vẫn nhanh. Fotollection đang dùng cách này.
- **PPG** (patterned pixel grouping) trước hết xem ảnh ít thay đổi nhất theo hướng nào, rồi nội suy theo hướng đó. rawler dùng cách này.
- **RCD** (ratio corrected demosaicing, của Luis Sanz Rodríguez) làm việc trên tỉ lệ giữa các màu và nổi tiếng ít lỗi trên chi tiết nhỏ; darktable và RawTherapee đều có.
- **Markesteijn**, theo tên Frank Markesteijn, là cách dành riêng cho X-Trans, chạy một hoặc ba lượt.

Khi một cách đoán sai, lỗi hiện ra theo những kiểu quen thuộc:

- **zipper (răng cưa):** một hàng điểm sáng tối xen kẽ dọc theo cạnh;
- **mê cung:** những vân nhỏ như giun trên vùng phẳng, khi hai điểm xanh lá trong một ô Bayer đọc không hoàn toàn giống nhau;
- **màu giả:** chấm hoặc viền màu trên chi tiết vốn màu xám, như vải, tóc hay hàng rào ở xa;
- **moiré:** những gợn sóng màu trên hoa văn lặp lại rất nhỏ.

**Cách chúng tôi chấm điểm.** Mắt người dễ bị đánh lừa, nên mọi cách đều được chấm trên những ảnh đã biết đáp án. Máy tính vẽ một zone plate (các vòng tròn mỗi lúc một mảnh hơn), một hình sao Siemens, các dải màu đậm và các dải chuyển màu mượt, biến chúng thành dữ liệu cảm biến, demosaic, rồi so kết quả với ảnh gốc:

| Ảnh chuẩn (768 px) | Bilinear | Malvar (đang dùng) |
|---|---|---|
| Zone plate: PSNR, càng cao càng tốt | 13,0 dB | 17,3 dB |
| Zone plate: màu giả, càng thấp càng tốt | 65,9 | 43,0 |
| Sao Siemens: PSNR | 21,3 dB | 25,2 dB |
| Dải màu đậm: sai số dọc theo cạnh, càng thấp càng tốt | 11,6 | 21,6 |

Malvar tốt hơn hẳn trên chi tiết xám nhỏ, nhưng kém hơn ở cạnh giữa hai màu đậm, vì phép hiệu chỉnh của nó giả định các màu thay đổi cùng nhau. Đó là khoảng cách mà RCD và một bước chống màu giả phải lấp, đo trên đúng những ảnh chuẩn này (sắp có trong v0.11.0).

## 5. Từ màu của máy ảnh tới màu bạn thấy {#wiki-colour}

Đỏ, xanh lá, xanh dương của máy ảnh không phải đỏ, xanh lá, xanh dương của màn hình: mỗi cảm biến có tấm lọc với thuốc nhuộm riêng. Các ma trận màu trong tệp RAW chuyển màu của máy ảnh sang **CIE XYZ**, mô hình chuẩn về cách mắt người thấy màu (1931), rồi từ đó sang **sRGB**, không gian màu của phần lớn màn hình và các tiệm in ảnh.

- **Cân bằng trắng** đi trước: nhân thêm cho kênh đỏ và kênh xanh dương để vật màu trắng trong cảnh ra đúng màu trung tính. Máy ảnh ghi lại mức nó đã chọn.
- Một tệp DNG thường có **hai ma trận**: một đo dưới đèn sợi đốt ấm, một dưới ánh sáng ban ngày. Ma trận đúng cho tấm ảnh của bạn nằm ở giữa: bộ xử lý suy ra ánh sáng lúc chụp từ cân bằng trắng rồi pha hai ma trận lại. Fotollection làm y như vậy với các định dạng khác, từ các ma trận rawler biết cho từng dòng máy.
- Mọi việc trên diễn ra trong **ánh sáng tuyến tính**, theo đúng thang của cảm biến, trước mọi đường cong: pha trộn và thu phóng màu ở đó mới cho kết quả đúng.

**Màu riêng của máy ảnh.** Ảnh JPEG của máy ảnh không trung tính: mỗi hãng thêm độ tương phản, màu sắc và tông da của riêng mình. Hiện nay Fotollection khớp kết quả xử lý theo bản xem trước nằm trong tệp RAW, để ảnh RAW trông giống những gì máy ảnh hiện trên màn hình. Sắp có trong v0.11.0: màu trung tính làm mặc định, cùng look có sẵn **Như JPEG của máy**, chỉ cách một cú nhấp.

## 6. Tông màu: từ con số thành bức ảnh {#wiki-tone}

Các con số tuyến tính trông tối và nhạt trên màn hình, vì mắt người cảm nhận độ sáng theo tỉ lệ chứ không theo lượng. Một **đường cong tông** biến thang của cảm biến thành bức ảnh: nâng vùng tối, giữ vùng sáng không bị cháy ngay, và cho ảnh độ tương phản. Sau đó tệp được lưu theo đường cong sRGB (cái gọi là "gamma"), dành nhiều trong 256 mức của JPEG hơn cho vùng tối, nơi mắt người nhạy nhất.

Có hai thứ máy ảnh sửa trong JPEG mà dữ liệu RAW không có:

- **Tối góc:** ống kính nào cũng tối hơn ở các góc. Fotollection đo độ tối góc từ chính bản xem trước của máy: khớp thêm một hệ số theo bán kính đưa kết quả DNG của chúng tôi từ ΔE 3,8 xuống 2,67 so với JPEG của máy, mọi vùng của ảnh đều lệch trong khoảng ±0,5 L\*.
- **Méo hình:** độ phình hoặc thắt nhẹ của ống kính, mà máy Micro Four Thirds (Panasonic, Olympus) dựa vào máy ảnh để sửa. Trên máy Mac, các tệp đó được macOS xử lý, và macOS có sửa méo; việc tự sửa méo trong bộ xử lý của chúng tôi đã có trong kế hoạch (FR-085).

## 7. Fotollection xử lý một ảnh RAW thế nào {#wiki-fotollection}

Từng bước, cho một lần xử lý đầy đủ (bản xem trước để in, phóng to và xuất ảnh):

1. **Nhận biết** định dạng từ những byte đầu tiên của tệp, không dựa vào tên tệp.
2. **Đọc** các con số của cảm biến: DNG bằng bộ đọc của riêng mình, các định dạng khác do rawler giải nén. Tệp quá lớn hoặc hỏng bị từ chối trước khi giải mã bất kỳ điểm ảnh nào.
3. **Mức:** đưa đen về 0 và trắng về 1, theo từng vị trí của tấm lọc màu.
4. **Demosaic** (hiện là Malvar; RCD và Markesteijn sắp có trong v0.11.0).
5. **Màu:** cân bằng trắng và các ma trận của máy ảnh, trong ánh sáng tuyến tính.
6. **Tông và màu riêng**, khớp theo bản xem trước của máy, kèm độ tối góc.
7. Từ đây ảnh RAW giống mọi ảnh khác: cắt khung, dấu ngày, khung viền, cỡ in và hồ sơ màu, xuất ở độ phân giải đầy đủ.

Vài nguyên tắc luôn giữ. Mỗi lần chỉ xử lý một ảnh, trên mọi nhân của bộ xử lý, và không bao giờ trên luồng của cửa sổ, nên app vẫn luôn phản hồi. Xuất ảnh luôn chạy trên bộ xử lý, để ảnh in ra giống hệt nhau trên mọi máy; card đồ hoạ chỉ tăng tốc những gì bạn thấy khi chỉnh. Và mã nguồn là của chúng tôi, viết theo các bản đặc tả và bài báo đã công bố, không chép từ dự án khác, rồi được kiểm tra: trên các ảnh chuẩn tổng hợp ở trên, trên chín tệp RAW từ các dòng máy khác nhau (CC0, từ raw.pixls.us), và bằng cách cố tình cắt cụt, làm hỏng tệp để tệp lỗi chỉ báo lỗi, không bao giờ làm app sập.

**Sắp có trong v0.11.0:** bộ xử lý của chúng tôi đảm nhận mọi máy ảnh có cảm biến Bayer, rawler chỉ còn giải nén tệp. Đo trên chín tệp mẫu (xử lý đầy đủ, trung vị của 3 lần):

| Máy ảnh | Của chúng tôi | rawler | macOS |
|---|---|---|---|
| Sony A7 III | 211 ms | 241 ms | 488 ms |
| Canon R10 | 293 ms | 360 ms | 1.647 ms |
| Nikon D750 | 340 ms | 402 ms | 412 ms |
| Canon 5D Mark IV | 462 ms | 565 ms | 418 ms |
| Ricoh GR III | 333 ms | 650 ms | — |

Tiếp theo là Fujifilm X-Trans với Markesteijn, rồi xử lý trên card đồ hoạ cho bản xem trước, lấy kết quả của bộ xử lý làm chuẩn.
