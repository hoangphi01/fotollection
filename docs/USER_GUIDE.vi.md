# Hướng dẫn sử dụng — Fotollection

> **Status:** Living · **Last reviewed:** 2026-10-30
>
> Bản tiếng Việt của [USER_GUIDE.md](USER_GUIDE.md), cũng là trang hướng dẫn trên website ([fotollection.org/vi/manual](https://fotollection.org/vi/manual/), [ADR-0031](adr/0031-manual-on-the-website-from-the-user-guide.md)). Giữ đúng các chương, mốc chương (`<!-- chapter: id -->`, ở dòng trước tiêu đề chương) và ảnh chụp màn hình (`<!-- shot: tên | mô tả -->`) như bản tiếng Anh, dùng đúng nhãn của app trong `ui/src/locales/vi.json`; `scripts/check-docs.sh` kiểm tra hai bản khớp nhau.

<!-- chapter: start -->

## 1. Bắt đầu

Fotollection làm việc trên các thư mục ảnh bạn đã có. Nó đọc ảnh ngay tại chỗ, không bao giờ di chuyển, đổi tên hay sửa một tệp nào ([FR-001](PRD.md#fr-001--import-photos)).

1. **Mở một thư mục:** nhấp **Mở thư mục** (biểu tượng thư mục cạnh *Thư viện* ở thanh bên), hoặc thả một thư mục vào bất kỳ đâu trên cửa sổ. Các thư mục con cũng được mở theo. App đọc JPEG, PNG, HEIC và ảnh RAW (chương 14).
2. **Thanh bên** liệt kê, dưới *Thư viện*:
   - **Tất cả ảnh**: mọi ảnh của mọi thư mục;
   - **Giỏ in** và **Để riêng**: hai khu vực giữ ảnh (chương 4);
   - **từng thư mục**, kèm số ảnh. Nhấp một thư mục để chỉ xem ảnh của nó. Ctrl/⌘-nhấp, hoặc nút **Chọn thư mục**, để xem nhiều thư mục cùng lúc. Dấu **×** hiện trên thư mục sẽ gỡ nó khỏi thư viện; tệp vẫn còn nguyên trên máy.
3. **Làm mới thư mục** (hai mũi tên xoay vòng) tìm ảnh mới được thêm vào các thư mục này; các chỉnh sửa được giữ nguyên.
4. **Gallery** nằm ở giữa. Thanh công cụ phía trên có *Ảnh* / *Nhóm* và *Tìm ảnh đẹp nhất* (chương 3), số ảnh, *Chọn* (chương 4) và các bộ lọc (chương 2).
5. **Bảng Thiết lập in** bên phải có, từ trên xuống: *Ngôn ngữ*, *Khổ in / xuất ảnh* (chương 5), *Dấu ngày* (chương 8), *Khung thông tin* (chương 9), *Đồng hồ máy ảnh* (chương 11) và *Địa điểm* (chương 10) khi thư viện đã có, rồi phần xuất ảnh (chương 12) và số phiên bản (chương 17).

<!-- shot: grid | Thư viện ảnh, đã chấm sao và chọn ảnh -->

**Ngôn ngữ và giao diện:** chọn *English* hoặc *Tiếng Việt* trong thẻ **Ngôn ngữ** ở đầu bảng. Ba nút ở cuối thanh bên đổi giao diện: **Theo hệ thống**, **Sáng** hoặc **Tối** ([FR-020](PRD.md#fr-020--localization), [FR-052](PRD.md#fr-052--appearance-theme-title-bar-and-hints)).

<!-- chapter: cull -->

## 2. Sao, cờ và bộ lọc

Đánh dấu ảnh trong lúc xem, rồi chỉ hiện những tấm bạn muốn ([FR-022](PRD.md#fr-022--culling-rate-flag-filter)). Sao và cờ không bao giờ sửa tệp ảnh; chúng được lưu trong dự án.

1. Chọn một ảnh trong gallery, hoặc mở nó trong bản xem trước.
2. Nhấn **1**–**5** để chấm sao, **0** để xoá sao.
3. Nhấn **P** để chọn, **X** để loại, **U** để bỏ cờ. Ảnh bị loại vẫn ở trong gallery, mờ đi, cho đến khi bạn lọc nó ra; không gì bị xoá.
4. Trong bản xem trước, thẻ **Sao & cờ** làm việc đó bằng chuột: nhấp một ngôi sao (nhấp lần nữa để xoá), **Chọn (P)** hoặc **Loại (X)**.
5. Lọc gallery bằng hai danh sách ở bên phải thanh công cụ:
   - cờ: **Tất cả**, **Đã chọn**, **Chưa gắn cờ** hoặc **Đã loại**;
   - sao: **Mọi số sao**, hoặc **≥ 1** đến **≥ 5**.

   Bộ lọc đang bật có viền màu chủ đề, và số ảnh cho biết đang hiện bao nhiêu trên tổng số. Khi không ảnh nào khớp, **Hiện tất cả ảnh** bỏ các bộ lọc.

<!-- shot: manual-filters | Bộ lọc cờ đang mở phía trên gallery: Tất cả, Đã chọn, Chưa gắn cờ, Đã loại -->

Những gì bộ lọc hiện cũng là những gì *Xuất ảnh* ghi ra (chương 12). Sao và cờ loại từ Lightroom và các app khác được đọc tự động (§10.1).

<!-- chapter: best-shot -->

## 3. Ảnh đẹp nhất

Các loạt ảnh chụp liên tiếp và ảnh gần giống nhau được gom nhóm, và tấm đẹp nhất mỗi nhóm được gợi ý kèm lý do ([FR-041](PRD.md#fr-041--best-shot-suggestions)). Không ảnh nào bị loại khi bạn chưa chọn, và không ảnh nào bị xoá.

1. Nhấp **Tìm ảnh đẹp nhất** trên thanh công cụ. Màn hình chuyển sang **Nhóm** trong lúc ảnh được phân tích ngay trên máy này; **Dừng** để ngừng sớm. Ảnh đã phân tích sẽ không phải phân tích lại.
2. Mỗi nhóm xếp các ảnh thành một hàng. Tấm được gợi ý mang nhãn **Đẹp nhất**, và nhóm ghi **Đẹp nhất trong n** kèm lý do: *Nét hơn n %*, *Ít rung tay hơn*, *Mắt mở*, *Phơi sáng tốt hơn*, *Ít vùng cháy sáng hơn*, *Ít nhiễu hơn*. Khi các ảnh gần như ngang điểm, nhóm ghi **Khó phân định**.
3. **So sánh** hiện các ảnh trong nhóm thật lớn, cạnh nhau, mỗi tiêu chí một thanh (*Độ nét*, *Mắt*, *Độ vững*, *Phơi sáng*, *Cháy sáng*, *Nhiễu*), kèm số khuôn mặt nhắm mắt. **Giữ ảnh này** giữ tấm bạn thích.
4. **Giữ ảnh tốt nhất** giữ tấm được gợi ý: nó được gắn cờ chọn, các tấm còn lại trong nhóm bị loại. Với nhóm khó phân định, nút này mở *So sánh* để bạn tự chọn. **Giữ ảnh tốt nhất ở n nhóm rõ ràng**, phía trên các nhóm, làm việc đó cho mọi nhóm không khó phân định.
5. Quay về **Ảnh**, chỉ hiện các ảnh **Đã chọn** (chương 2). **Cmd/Ctrl+Z** hoàn tác được mọi thao tác ở đây.

<!-- shot: manual-groups | Một nhóm bốn tấm ảnh chú mèo: tấm nét nhất được gợi ý là đẹp nhất, nét hơn 63 % -->

<!-- shot: groups | So sánh: bốn tấm ảnh chú mèo đặt cạnh nhau, mỗi tiêu chí một thanh -->

<!-- chapter: choose -->

## 4. Chọn ảnh để in

Một chuyến đi cho ra hàng trăm tấm ảnh, nhưng chỉ vài tấm được đem đi in. Fotollection giữ lựa chọn đó trong hai **khu vực**, bên cạnh gallery ([FR-050](PRD.md#fr-050--print-basket)):

| Khu vực | Chứa gì | Ở đâu |
|---------|---------|-------|
| **Giỏ in** | Những tấm bạn sẽ in | Thanh bên, dưới *Tất cả ảnh*, kèm số lượng |
| **Để riêng** | Những tấm bạn tạm cất đi. Chúng rời khỏi gallery nhưng **không bao giờ bị xoá hay di chuyển trên ổ đĩa** | Thanh bên, dưới *Giỏ in*, kèm số lượng |

Mỗi tấm ảnh chỉ ở một nơi: trong gallery, trong giỏ in, hoặc để riêng. Ảnh trong giỏ vẫn hiện trong gallery, với một biểu tượng giỏ nhỏ ở góc trên bên phải; ảnh để riêng không hiện trong *Tất cả ảnh* hay trong các thư mục cho đến khi bạn đưa chúng trở lại. Sao và cờ chọn/loại là chuyện riêng: ảnh được chọn không nhất thiết phải in, và giỏ in không phải là một cờ.

Cả hai khu vực được lưu trong dự án (`.fotollection`) và trong phiên làm việc, nên vẫn còn sau khi bạn tắt rồi mở lại app. Mọi thao tác đều hoàn tác được bằng **Cmd/Ctrl+Z**.

### 4.1 Cách A: quyết từng tấm trong bản xem trước

1. Mở một tấm: nhấp đúp vào nó, hoặc chọn nó rồi nhấn **Enter** (hoặc **Space**).
2. Xem ảnh với các thiết lập in đã áp.
3. Nhấn **↑** để cho vào **giỏ in**: tấm tiếp theo mở ra ngay.
4. Nhấn **↓** để **để riêng**: ảnh rời khỏi gallery, tấm tiếp theo thế chỗ.
5. Nhấn **→** để đi tiếp mà chưa quyết, **←** để quay lại.
6. Lặp lại đến hết chuyến đi. **Esc** đóng bản xem trước.

<!-- shot: swipe | Một tấm ảnh đã vào giỏ in, cùng các phím để lướt tiếp -->

Nút **Giỏ in** trên ảnh (cạnh *Cắt ảnh*, các nút xoay và *Xuất ảnh*) làm giống phím ↑, và hiện *Đã trong giỏ* khi ảnh đã nằm trong giỏ; nhấp lần nữa để lấy ảnh ra. Ảnh đã để riêng thì hiện **Trả về gallery**.

Khi bạn mở bản xem trước từ *Giỏ in* hoặc *Để riêng*, ↑ và ↓ chuyển ảnh sang khu vực còn lại; ảnh ở nguyên chỗ cũ thì chuyển sang tấm tiếp theo.

### 4.2 Cách B: tích ảnh ngay trong gallery (chế độ Chọn)

1. Nhấp **Chọn** trên thanh công cụ, phía trên các ảnh. Nút đổi thành **Xong**, và một thanh hiện ra dưới thanh công cụ.
2. Nhấp vào ảnh để tích: một vòng tròn có dấu tích hiện ở góc dưới bên phải, và ảnh được viền màu chủ đề. Nhấp lần nữa để bỏ tích.
   - **Shift+nhấp** tích mọi ảnh từ tấm vừa tích đến tấm này.
   - **Cmd/Ctrl+A** hoặc **Chọn tất cả** tích mọi ảnh đang hiện.
   - **Enter** tích hoặc bỏ tích ảnh đang chọn.
3. Trên thanh, chọn việc cần làm với các ảnh đã tích:
   - **Giỏ in**: cho chúng vào giỏ.
   - **Để riêng**: đưa chúng ra khỏi gallery.
   - **Trả về gallery**: chỉ có trong *Giỏ in* và *Để riêng*; đưa chúng trở lại.
   - **Bỏ chọn**: bỏ tích tất cả.
4. Nhấp **Xong**, hoặc nhấn **Esc**, để rời chế độ Chọn. Dấu tích chỉ là tạm thời: điều quyết định là mỗi ảnh đang ở khu vực nào.

<!-- shot: select | Chế độ Chọn: ảnh đã tích, giỏ in và chồng ảnh để riêng -->

**Nhiều thư mục:** dấu tích đi theo bạn từ tab thư mục này sang tab khác ([FR-056](PRD.md#fr-056--folders-as-tabs)). Tích vài tấm ở một thư mục, nhấp sang thư mục khác ở thanh bên rồi tích tiếp; thanh chọn hiện tổng số và số ảnh ở các thư mục khác, và *Giỏ in* chuyển tất cả cùng lúc.

<!-- shot: tabs | Ba thư mục là ba tab: ảnh đã tích ở hai thư mục, và nút đóng trên một tab -->

Trong chế độ Chọn, nhấp đúp không mở bản xem trước, và **phím Space làm việc khác** (§4.3).

### 4.3 Xem nhanh: xem mà không cần mở (chỉ trong chế độ Chọn)

Trong chế độ Chọn, **giữ Space** trên một tấm để xem nó thật lớn, đúng như khi in, với khổ in, dấu ngày và khung thông tin của gallery ([FR-059](PRD.md#fr-059--quick-preview)). **Thả Space** để đóng, giống Quick Look trên Mac.

- Nó hiện tấm ảnh dưới con trỏ chuột; nếu con trỏ không nằm trên ảnh nào, nó hiện tấm bạn tích gần nhất.
- Ảnh nhỏ hiện ngay, rồi bản dựng đầy đủ thay vào một lát sau.
- Trong lúc giữ Space: **← →** xem các tấm bên cạnh, **Enter** tích hoặc bỏ tích tấm đang hiện, **Esc** đóng.
- Ở đây không chỉnh sửa được gì. Muốn đổi khung cắt, dấu ngày hay khung thông tin của một tấm, hãy mở bản xem trước thường (nhấp đúp, hoặc **Enter** khi không ở chế độ Chọn).

Ngoài chế độ Chọn, Space vẫn mở bản xem trước thường.

### 4.4 Xem lại và xuất ảnh

- Nhấp **Giỏ in** hoặc **Để riêng** ở thanh bên để chỉ xem những ảnh đó. Đổi ý bằng ↑ ↓ trong bản xem trước, hoặc bằng chế độ Chọn và **Trả về gallery**.
- Nhấp **Tất cả ảnh**, hoặc một thư mục, để về gallery.
- **Xuất ảnh theo đúng chỗ bạn đang xem** (chương 12). Từ *Giỏ in*, *Xuất ảnh* ghi đúng những tấm trong giỏ, với ước tính dung lượng ngay trên nút ([FR-053](PRD.md#fr-053--export-for-social-media)). Từ *Tất cả ảnh* hay một thư mục, ảnh để riêng không bao giờ được xuất.

<!-- shot: estimate | Ô ước tính dung lượng ngay trên nút xuất ảnh, khi xuất cho Facebook -->

<!-- chapter: size -->

## 5. Khổ in và độ nét

Chọn khổ bạn sẽ in, Fotollection cắt mọi ảnh theo khổ đó và cho biết từng tấm có đủ nét không ([FR-026](PRD.md#fr-026--resolution-warning), [FR-043](PRD.md#fr-043--custom-print-sizes), [FR-045](PRD.md#fr-045--original-aspect-ratio)).

1. Trong thẻ **Khổ in / xuất ảnh**, giữ lựa chọn **In ảnh** (với **Mạng xã hội**, xem chương 12).
2. Mở danh sách khổ:
   - **Tiêu chuẩn (CEWE / EU)**: các khổ của máy in ảnh tại dm và Rossmann, như *10 × 15 cm · 2:3*;
   - **Gốc**: tỉ lệ gốc của ảnh, không cắt;
   - **Tùy chỉnh**: các khổ bạn tự thêm.

   Mỗi khổ có một huy hiệu cho ảnh đang chọn trong gallery: **Nét** (từ 300 DPI), **Tạm được** (150–299 DPI, ổn khi cầm xem) hoặc **Sẽ vỡ** (dưới 150 DPI). Rê chuột lên huy hiệu để xem số DPI.
3. Dưới danh sách, một dòng cho biết ảnh đang chọn in nét tới cỡ nào, và vẫn chấp nhận được tới cỡ nào.
4. Mọi ảnh được cắt theo tỉ lệ của khổ, **mặc định ở giữa**. Chọn phần được giữ cho từng ảnh trong bản xem trước (chương 6).

<!-- shot: manual-size | Danh sách khổ in, mỗi khổ một huy hiệu độ nét -->

**Khổ của riêng bạn:** chọn **Khổ tùy chỉnh…**, gõ tên (ví dụ *Instax wide*), chiều rộng, chiều cao và đơn vị (in hoặc cm), rồi **Lưu khổ**. Mỗi cạnh phải từ 2 đến 100 cm. Nút thùng rác cạnh danh sách xoá khổ tùy chỉnh đang chọn.

<!-- chapter: crop -->

## 6. Cắt và cân ảnh

Mỗi ảnh được cắt theo tỉ lệ của khổ in, ở giữa. Chọn phần được giữ, cân lại đường chân trời hay xoay ảnh, từng tấm một ([FR-006](PRD.md#fr-006--crop-to-print-ratio), [FR-044](PRD.md#fr-044--per-photo-overrides)).

1. **Nhanh:** trong bản xem trước, **kéo ảnh** để dịch nó trong khung in.
2. **Chính xác:** nhấp **Cắt ảnh** phía trên ảnh, hoặc nhấn **C**. Trong màn cắt ảnh:
   - **kéo** để dịch vùng cắt, **nhấp đúp** để đưa về giữa;
   - **phóng** bằng thanh trượt, chụm hai ngón, con lăn chuột hoặc **+** / **−**, tới 8×;
   - **Cân ngang** cân đường chân trời, ±15°: bằng thanh trượt, hoặc **← →** mỗi lần 1° (**⌥ ← →** mỗi lần 0,1°). Nhấp đúp vào thanh trượt để về 0°;
   - các nhãn cho biết cỡ in và số DPI, đổi sang cam hoặc đỏ khi phóng quá tay làm thiếu điểm ảnh.
3. **Xong** (hoặc **Enter**) giữ thay đổi, **Hủy** (hoặc **Esc**) để nguyên như cũ, **Đặt lại** đưa vùng cắt về giữa và góc về 0°.
4. **Xoay** ảnh một phần tư bằng **Xoay trái** và **Xoay phải** phía trên ảnh, hoặc **[** và **]**, ngay trong gallery cũng được.

<!-- shot: crop | Cân lại đường chân trời trong màn cắt ảnh -->

Trong thẻ khổ của bản xem trước, dòng *Cắt ảnh* ghi **Ở giữa** kèm nút **Chỉnh**, hoặc **Tùy chỉnh · phóng n×** kèm nút **Đặt lại**. Khi khung thông tin ở chế độ *Giữ trọn ảnh* (chương 9), cả tấm ảnh được in nên không có gì để cắt.

<!-- chapter: zoom -->

## 7. Xem kỹ: phóng to

Trong bản xem trước, **các nút phóng to** ở góc dưới bên phải ảnh hiện mức phóng giữa **−** và **+**; nút cuối cùng đưa ảnh về vừa khung ([FR-057](PRD.md#fr-057--zoom-in-the-preview)). Bàn phím và trackpad cũng làm được như vậy:

- **Chụm hai ngón** trên trackpad, hoặc **Cmd/Ctrl +** và **Cmd/Ctrl −**, phóng quanh con trỏ, tới 800 %.
- **Nhấp đúp** vào ảnh để chuyển giữa vừa khung và 200 %.
- Khi đã phóng to, **kéo** ảnh, hoặc cuộn bằng hai ngón, để di chuyển trong ảnh.
- **Cmd/Ctrl 0** hoặc nút vừa khung đưa về toàn bộ ảnh.

Bản xem trước ban đầu được hiện ở cỡ nhẹ để máy nào cũng vẽ nhanh. Khi bạn phóng vượt quá cỡ đó, một vòng xoay nhỏ hiện cạnh mức phóng, và một lát sau khi bạn ngừng di chuyển, phần trên màn hình được thay bằng điểm ảnh **độ phân giải đầy đủ**, đúng như trong tệp sẽ xuất. Chỉ phần đang thấy được tải, nên kể cả ảnh 32 MP vẫn nhẹ.

<!-- chapter: stamp -->

## 8. Dấu ngày

Ngày ở góc ảnh in, như máy ảnh phim, lấy theo lúc chụp ([FR-004](PRD.md#fr-004--date-stamp), [FR-033](PRD.md#fr-033--retro-film-date-stamp-and-stamp-presets)). Mặc định đang bật.

1. Trong thẻ **Dấu ngày**, công tắc bật hoặc tắt dấu cho mọi ảnh.
2. Chọn:
   - **định dạng**: `2026-07-14`, `14.07.2026` hoặc `'26 07 14`, kiểu máy ảnh phim;
   - **góc**: *Góc dưới phải*, *Góc dưới trái*, *Góc trên phải* hoặc *Góc trên trái*;
   - **màu**: **Trắng** hoặc **Cam retro**.
3. **Địa điểm cùng dòng ngày** thêm tên thị trấn nơi chụp, lấy từ GPS của ảnh, không cần mạng: dạng **Thành phố**, **+ mã nước** hoặc **+ tên nước**. Ảnh không có GPS có thể đặt nơi chụp bằng tay (chương 10).
4. **Dòng chữ dưới ngày**: một dòng ngắn chạy ngang ảnh, tối đa 80 ký tự, in bằng chữ hoa không dấu (ví dụ *KY NIEM BUDAPEST*).

<!-- shot: stamp | Dấu ngày kèm tên nơi chụp từ GPS của ảnh -->

**Riêng một tấm:** trong bản xem trước, thẻ **Dấu ngày** có đủ các điều khiển trên cho riêng ảnh đó; ảnh được đánh dấu **Riêng ảnh này**. **Áp dụng cho tất cả** biến dấu của ảnh này thành dấu cho mọi ảnh. Trong bảng bên, **Đặt lại n ảnh có dấu riêng** cho mọi ảnh theo lại bảng bên. Cả hai đều hoàn tác được.

Ngày in ra là ngày của máy ảnh, đã chỉnh theo đồng hồ máy ảnh (chương 11) hoặc đặt bằng tay (chương 10).

<!-- chapter: frame -->

## 9. Khung thông tin "Shot on"

Một dải trắng hoặc đen dưới ảnh, với logo và thông số máy ảnh, như ảnh chia sẻ từ điện thoại ([FR-040](PRD.md#fr-040--exif-info-frame), [FR-065](PRD.md#fr-065--border-in-crop-to-fill)). Mặc định đang tắt, và giống nhau cho mọi ảnh.

1. Bật thẻ **Khung thông tin**.
2. Chọn cách đặt khung vào ảnh in:
   - **Cắt cho đầy**: ảnh in giữ đúng khổ, dải thông tin nằm bên trong; kéo ảnh trong bản xem trước để chọn phần được giữ. **Tạo viền** thêm viền 3 mm theo màu khung ở cạnh trên và hai bên, vẫn nằm trong khổ in, như ảnh lấy liền;
   - **Giữ trọn ảnh**: cả tấm ảnh đúng khổ in, đặt trong khung như ảnh instax với viền 5 mm ở ba cạnh và dải thông tin ở dưới, nên ảnh in ra lớn hơn khổ đã chọn. Một dòng ghi chú cho biết lớn bao nhiêu, để bạn chọn giấy phù hợp ở tiệm.
3. Chọn **Trắng** hoặc **Đen**, định dạng ngày, và phông chữ: *Inter (có sẵn)*, hoặc **Nhập OTF/TTF…** để dùng phông của bạn ([FR-042](PRD.md#fr-042--custom-fonts)).
4. Trong **Hiển thị**, tích những gì dải thông tin ghi: *Logo*, *Máy ảnh*, *Ống kính*, *Tiêu cự*, *Khẩu độ*, *Tốc độ màn trập*, *ISO*, *Photo Style*, *Ngày*, *Địa điểm*.
5. Khi hiện logo, chọn **Màu logo** (*Cổ điển*, *Màu gốc* hoặc *Đơn sắc*) và **Cỡ logo** (nhấp đúp vào thanh trượt để về 100 %).

<!-- shot: frame | Khung “Shot on” với logo của hãng máy ảnh -->

<!-- shot: manual-frame | Thẻ Khung thông tin: cách đặt khung, viền, màu, phông và các thông tin hiển thị -->

**Thông tin máy ảnh:** trong bản xem trước, dưới thẻ khung, sửa những gì dải thông tin ghi cho máy ảnh này: **Tên máy ảnh**, **Ống kính** (cho ống kính manual mà máy không nhận ra), **Tiêu cự (mm)**, **Khẩu độ F** và **Photo Style**, cùng **Logo**: logo hãng có sẵn, **Chữ**, **Ảnh** (một tệp PNG của bạn) hoặc **Không**. Ô để trống sẽ dùng EXIF của ảnh. Các thông tin này được lưu cho mọi ảnh của máy đó.

<!-- chapter: dates-places -->

## 10. Ngày và nơi chụp

**Đặt bằng tay** ([FR-032](PRD.md#fr-032--manual-date-and-place)), cho ảnh in đã scan, máy ảnh không có đồng hồ, hay ảnh không có GPS:

1. **Một tấm:** trong bản xem trước, nhấp **Đặt ngày…** trong *Ngày chụp* hoặc **Đặt nơi chụp…** trong *Vị trí*.
2. **Nhiều tấm:** trong chế độ Chọn, tích chúng rồi nhấp **Ngày và nơi…** trên thanh chọn.
3. Gõ ngày giờ, và gõ tên thị trấn hay thành phố để chọn (dấu và khoảng trắng không quan trọng: `ha noi` tìm ra Hà Nội). Chỉ điền phần muốn đổi, rồi **Áp dụng**. Các nút mũi tên đưa về giờ của máy ảnh và GPS của chính ảnh.

<!-- shot: manual-place-picker | Đặt nơi chụp cho một tấm: gõ ba chữ là tìm ra thành phố -->

Ảnh dịch chuyển trong lưới, và dấu ngày, khung thông tin, thư mục theo ngày cùng ngày của bản xuất đều đổi theo. Mọi thứ đều offline, mỗi thay đổi là một bước Hoàn tác, và tệp ảnh không bao giờ bị sửa.

**Mượn nơi chụp từ điện thoại** ([FR-031](PRD.md#fr-031--borrow-gps-from-a-phone-or-gpx-track)): máy ảnh không có GPS (như Lumix GX9) có thể lấy nơi chụp của ảnh điện thoại chụp cùng cảnh, cách vài phút. Hãy chỉnh đồng hồ máy ảnh trước nếu nó lệch (chương 11). Trong thẻ **Địa điểm** của bảng cài đặt, nhấp **Tìm vị trí**, chọn khoảng cách thời gian (5, 10 hoặc 30 phút) rồi **Tìm vị trí**. Cặp chắc chắn được điền ngay; các cặp còn lại hiện cạnh ảnh điện thoại, đã tích sẵn, bạn bỏ tích cặp sai rồi xác nhận. Cặp chắc chắn cũng được điền sau mỗi lần phân tích ảnh đẹp nhất. Nơi đã mượn hiện một ghim có dấu tích trên ảnh nhỏ và chữ *Mượn* trong bản xem trước; nó không bao giờ thay nơi bạn đã đặt hay GPS của chính ảnh, và Hoàn tác lấy lại được.

**Đổi tên nơi chụp** ([FR-005](PRD.md#fr-005--location-stamp-offline)): thẻ **Địa điểm** trong bảng cài đặt liệt kê mọi nơi có trên ảnh của bạn. Gõ tên bạn muốn in, như `Prague` thay cho `Praha 1` hay `Nhà bà ngoại`, và mọi dấu ngày, khung thông tin có nơi đó đổi theo; nút mũi tên đưa về tên gốc.

<!-- shot: manual-places | Thẻ Địa điểm trong cài đặt: mọi nơi có trên ảnh, mỗi nơi một tên để in -->

### 10.1 Sao từ Lightroom và các app khác

Khi thư mục bạn mở đã được lọc trong Lightroom, Bridge, Capture One hay darktable, số sao và cờ loại chúng lưu trong tệp XMP đi kèm (`IMG_0416.xmp` hay `P1000123.RW2.xmp` cạnh ảnh) được đọc cùng ảnh ([FR-058](PRD.md#fr-058--lightroom-compatibility-through-xmp)). Thanh bên cho biết bao nhiêu tệp. Sao và cờ của bạn trong dự án Fotollection thắng tệp XMP, và tệp XMP không bao giờ bị sửa. Lightroom Classic chỉ ghi tệp XMP cho ảnh RAW, và chỉ khi bật *Automatically write changes into XMP* (hoặc sau *Metadata → Save Metadata to File*).

<!-- chapter: clocks -->

## 11. Đồng hồ máy ảnh

Máy ảnh đặt sai giờ, hoặc vẫn để múi giờ ở nhà, sẽ xếp ảnh sai thứ tự so với ảnh điện thoại, và đóng dấu sai ngày ([FR-030](PRD.md#fr-030--camera-clock-offset)).

1. **Theo một ảnh (dễ nhất):** mở một ảnh của máy đó, và trong **Ngày chụp** nhấp **Chỉnh giờ máy này theo một ảnh…**. Chọn ảnh điện thoại (hoặc máy khác) chụp cùng lúc, rồi **Chỉnh đồng hồ**.
2. **Bằng tay:** trong bảng cài đặt, **Đồng hồ máy ảnh** liệt kê mọi máy ảnh trong thư viện. Gõ lượng cần cộng thêm, như `+1:02:00` hay `−0:45`, rồi nhấn Enter. Nút mũi tên đặt lại.

Mọi ảnh của máy đó dịch chuyển cùng lúc, kể cả ảnh thêm vào sau: thứ tự trong lưới, nhóm ảnh đẹp nhất, thư mục theo ngày, dấu ngày và ngày của bản xuất. Bản xem trước vẫn hiện giờ mà đồng hồ máy ghi, và tệp ảnh không bao giờ bị sửa. Hai máy cùng đời giữ đồng hồ riêng nếu máy ảnh có ghi số seri. Phần chỉnh được lưu trong dự án.

<!-- chapter: export -->

## 12. Xuất ảnh

Xuất ảnh ghi ra các tệp mới để đem đi in hoặc chia sẻ; ảnh gốc của bạn không bao giờ bị sửa ([FR-008](PRD.md#fr-008--original-quality-export), [FR-009](PRD.md#fr-009--export-by-date), [FR-010](PRD.md#fr-010--metadata-privacy-on-export), [FR-053](PRD.md#fr-053--export-for-social-media)).

1. **Xuất những gì** gallery đang hiện: thư mục hay các thư mục đang chọn, theo bộ lọc (chương 2), không bao giờ gồm ảnh để riêng. Từ màn **Giỏ in**, xuất đúng những tấm trong giỏ (chương 4).
2. **Để in hay để chia sẻ:** ở đầu thẻ *Khổ in / xuất ảnh*, chọn **In ảnh** (độ phân giải đầy đủ, với khung cắt, dấu ngày và khung thông tin của từng ảnh) hoặc **Mạng xã hội**, rồi chọn cỡ chia sẻ:
   - **Giữ cỡ gốc, tệp nhẹ hơn**: giữ nguyên điểm ảnh, tệp còn khoảng một nửa hoặc ít hơn;
   - **Facebook · 2048 px** hoặc **Instagram · 1080 px**: đúng cỡ mỗi trang giữ lại, nên ảnh không bị thu nhỏ thêm;
   - **Cỡ tùy chỉnh**: cạnh dài 640–6000 px.

   Tệp chia sẻ ở hệ màu sRGB, nên trông giống nhau trên mọi màn hình.
3. **Lưu vào đâu:** nhấp **Chọn nơi lưu…** ở cuối bảng bên, rồi chọn một thư mục.
4. **Bao nhiêu thư mục:** khi đang hiện nhiều thư mục, chọn **Chung** (gộp vào cùng các thư mục theo ngày) hoặc **Riêng** (mỗi thư mục một thư mục con mang tên nó).
5. Phía trên nút, phần **ước tính** cho biết các tệp sẽ nặng bao nhiêu. Nhấp **Xuất n ảnh**. Một thanh hiện tiến độ, và **Hủy** dừng lại: các ảnh đã xuất xong vẫn giữ, tệp đang ghi dở bị xoá. Ảnh nào không xuất được sẽ bị bỏ qua, và phần tổng kết cho biết bao nhiêu ảnh lỗi.

<!-- shot: estimate | Ô ước tính dung lượng ngay trên nút xuất ảnh, khi xuất cho Facebook -->

Tệp được xếp vào mỗi ngày một thư mục, đặt tên theo ngày (`2026-07-14`), và `undated` cho ảnh không có ngày; tệp chia sẻ nằm trong thư mục `Share`. Không tệp nào ghi đè tệp khác: bản thứ hai mang thêm `_1`, `_2`. Thông số máy ảnh được giữ, còn **vị trí GPS luôn bị xoá**, để ảnh chia sẻ không làm lộ nơi bạn ở.

**Một tấm:** trong bản xem trước, nút **Xuất ảnh** phía trên ảnh dẫn tới nút **Xuất ảnh này**, cùng nơi lưu.

<!-- chapter: projects -->

## 13. Dự án, tự lưu và hoàn tác

Mọi việc bạn làm (sao, cờ, giỏ in, khung cắt, dấu ngày, ngày và nơi chụp, đồng hồ máy ảnh) được tự lưu ngay khi làm, và trở lại khi bạn mở app lần sau ([FR-023](PRD.md#fr-023--projects-autosave-and-undo)). Một dự án giữ một phần việc dưới một cái tên, để mở lại sau hoặc trên máy khác.

1. Khối **Dự án** ở đầu thanh bên hiện tên dự án; một chấm tròn nghĩa là *Đã sửa*. Nhấp đúp vào tên để đổi tên.
2. Bốn nút của nó: **Dự án mới**, **Mở dự án…**, **Lưu** và **Lưu thành…**. Trên Mac, các lệnh này cũng có trong menu *Tệp*, và trên phím **Cmd/Ctrl+N**, **Cmd/Ctrl+O**, **Cmd/Ctrl+S** và **Shift+Cmd/Ctrl+S**.
3. Dự án là một tệp `.fotollection`. Nó chứa các chỉnh sửa và thiết lập, cùng phông chữ và logo đang dùng, nhưng không chứa ảnh: nó trỏ tới các thư mục ảnh, nên một dự án được chép cùng với ảnh sẽ mở y hệt trên máy khác. Khi thư mục đã bị chuyển hoặc đổi tên, **Không tìm thấy n thư mục** cho bạn **Tìm…** từng thư mục, hoặc **Để sau**.
4. Rời một dự án còn thay đổi chưa lưu, app hỏi **Lưu thay đổi của "…"?**: **Lưu**, **Không lưu** hoặc **Hủy**.

<!-- shot: manual-project | Khối Dự án: tên dự án, và các nút để tạo, mở và lưu dự án -->

**Hoàn tác** thay đổi vừa làm bằng **Cmd/Ctrl+Z**, và **làm lại** bằng **Shift+Cmd/Ctrl+Z** (hoặc **Cmd/Ctrl+Y**), tới 200 bước. Trên Mac, hai lệnh này cũng có trong menu *Sửa*.

<!-- chapter: raw -->

## 14. Ảnh RAW

Fotollection mở ảnh RAW của hầu hết máy ảnh như chúng vốn có, không cần đổi sang DNG: RW2 (Panasonic), CR2 và CR3 (Canon), NEF (Nikon), ARW (Sony), RAF (Fujifilm), ORF (Olympus, OM System) và DNG ([FR-016](PRD.md#fr-016--raw-input)).

- Mở thư mục như thường. Ảnh nhỏ và gợi ý ảnh đẹp nhất hiện nhanh như với JPEG, vì chúng dùng bản xem trước nhỏ mà máy ảnh nào cũng lưu sẵn trong tệp RAW.
- Bản xem trước để in sẽ xử lý toàn bộ ảnh ở lần mở đầu, mất khoảng một giây; sau đó đổi thiết lập là thấy ngay. Khi xuất, mỗi ảnh RAW được xử lý theo cùng cách.
- Màu sắc và độ nét là cách xử lý trung tính của hệ điều hành, không phải của một trình sửa RAW: Fotollection chuẩn bị ảnh để in, không thay thế Lightroom.
- **Trên Windows**, hãy cài *Raw Image Extension* miễn phí của Microsoft từ Microsoft Store trước. Không có nó, tệp RAW bị bỏ qua và dòng gợi ý sẽ nói rõ. Một trình đọc RAW có sẵn trong Fotollection, không cần extension, đang được làm ([STATUS.md](STATUS.md)).

<!-- chapter: open-with -->

## 15. Mở ảnh từ Finder hoặc Explorer

Fotollection có thể là app mở ảnh khi bạn nhấp đúp ([FR-054](PRD.md#fr-054--open-photos-with-fotollection)). Nó không bao giờ tự đặt mình làm mặc định: bạn chọn.

- **Một lần:** nhấp phải vào ảnh → **Mở bằng** → **Fotollection** (macOS), hoặc **Mở bằng** → **Fotollection** (Windows).
- **Luôn luôn:** trên Mac, chọn ảnh, nhấn Cmd+I, chọn Fotollection trong *Mở bằng* rồi nhấp *Thay đổi tất cả*. Trên Windows, chọn **Mở bằng** → **Chọn ứng dụng khác**, chọn Fotollection và tích *Luôn dùng ứng dụng này*.

Ảnh mở ngay trong bản xem trước. Thư mục của nó thành một tab ở thanh bên, chỉ có ảnh của đúng thư mục đó (không gồm thư mục con), nên **← →** đi qua cả thư mục. Mọi thứ hoạt động như ở bất kỳ tab nào: sao, giỏ in, khung cắt, xuất ảnh. Xong thì nhấp **×** trên tab; ảnh vẫn nằm trên ổ đĩa của bạn. Mở một ảnh khác khi Fotollection đang chạy sẽ đưa cửa sổ lên trước với ảnh đó.

<!-- chapter: camera -->

## 16. Chụp với máy ảnh cắm vào máy tính

Khi máy ảnh cắm cáp, Fotollection hiện những gì máy thấy và chụp ảnh ([FR-062](PRD.md#fr-062--live-view-from-a-camera), [FR-063](PRD.md#fr-063--tethered-capture-over-usb)). Hiện tính năng này chạy trên Windows, với Sony a6400. Các máy Sony cùng thời có thể chạy được, nhưng chưa được kiểm tra.

1. **Trên máy ảnh:** *MENU → Setup → USB Connection → PC Remote*. Cắm cáp dữ liệu và bật máy. Đóng Imaging Edge Remote, vì mỗi lúc chỉ một app điều khiển được máy ảnh.
2. **Nhấp nút máy ảnh** cạnh *Mở thư mục* ở thanh bên. Màn hình chụp mở ra.
3. **Màn hình chụp:**
   - **Hình trực tiếp.** Nếu đã chọn khổ in, khung của khổ đó được vẽ đè lên, nên bạn thấy phần còn lại sau *Cắt cho đầy*.
   - **Thanh thiết lập** dưới hình trực tiếp hiện ISO, tốc độ màn trập, khẩu độ và định dạng tệp.
   - **Bảng bên** tăng giảm ISO và tốc độ bằng − / +, và chuyển giữa JPEG và RAW (máy có thể giữ định dạng cũ thêm một tấm). Khẩu độ chỉnh trên máy hoặc ống kính.
   - **Chụp**, hoặc **Space**, chụp ảnh. Máy lấy nét trước, nên mỗi tấm mất khoảng một giây. Nếu không lấy nét được, màn hình sẽ báo. Mỗi lần một tấm: nhấn tiếp (hoặc giữ Space) khi tấm trước chưa xong thì không có tác dụng.
   - **Các tấm đã chụp** xếp hàng dưới hình trực tiếp, mới nhất trước. Nhấp vào một tấm để mở trong bản xem trước.
4. **Đóng màn hình** (Esc). Bạn ở trong tab của phiên chụp, với mọi tấm đã chụp. Một thông báo phía trên lưới cho hai lựa chọn:
   - **Lưu tất cả…**: chọn một thư mục. Ảnh được chuyển vào đó, không ghi đè gì, và khung cắt, dấu ngày, số sao vẫn giữ. Nếu thư mục đã có tệp (như Pictures), ảnh vào một thư mục mới mang tên phiên chụp bên trong nó.
   - **Xóa ảnh**.

   Khi bạn chưa chọn, ảnh nằm trong dữ liệu riêng của Fotollection. Đóng tab sẽ hỏi trước, **thoát app** cũng vậy: *Lưu tất cả…*, *Xóa ảnh* hoặc *Hủy*. Nếu Fotollection lỡ đóng mà không hỏi (bị lỗi), phiên chụp trở lại thành một tab ở lần mở sau.

   Rút cáp hoặc tắt máy ảnh cũng đóng màn hình và hiện tab của phiên chụp.

**Ảnh RAW** được trình đọc RAW có sẵn của Fotollection đọc, nên hiện trong tab như JPEG ([ADR-0029](adr/0029-raw-through-rawler-then-our-own-decoder.md)); trên Windows, việc này đang được kiểm tra trên máy không có Raw Image Extension của Microsoft.

<!-- chapter: updates -->

## 17. Cập nhật lên phiên bản mới

Fotollection không bao giờ tự lên mạng ([FR-021](PRD.md#fr-021--manual-updates), [FR-061](PRD.md#fr-061--check-for-updates)).

1. Nhấp vào số phiên bản ở cuối bảng cài đặt, rồi **Kiểm tra cập nhật**. Đây là lúc duy nhất app lên mạng.
2. Nếu có bản mới hơn, nhấp **Tải bộ cài**: trình duyệt sẽ tải đúng bộ cài cho máy của bạn.
3. Cài đè lên bản đang dùng. Cài đặt, hồ sơ máy ảnh và dự án của bạn vẫn giữ nguyên.
   - **Mac:** mở tệp đĩa và kéo Fotollection vào Applications, rồi chọn **Thay thế**.
   - **Windows:** chạy tệp `-setup.exe`; giữ lựa chọn đầu tiên, *Gỡ cài đặt trước khi cài*. Nếu bạn đã cài bằng `.msi`, hãy cập nhật bằng `.msi`: trộn hai loại sẽ để lại hai bản.

Cùng ô đó hiện **cache**: ảnh nhỏ và kết quả phân tích ảnh đẹp nhất mà Fotollection giữ để mở thư mục nhanh, kèm dung lượng so với giới hạn 2 GB. Vượt giới hạn, các mục cũ nhất tự bị xoá. **Xoá cache** làm trống nó bất kỳ lúc nào; không mất gì của bạn, và ảnh nhỏ sẽ được tạo lại khi bạn duyệt.

<!-- chapter: shortcuts -->

## 18. Phím tắt

| Ở đâu | Phím | Việc |
|-------|------|------|
| Gallery | **← → ↑ ↓** | Di chuyển vùng chọn |
| Gallery | **Enter**, **Space**, nhấp đúp | Mở bản xem trước |
| Gallery, xem trước | **0**–**5** | Số sao |
| Gallery, xem trước | **P** / **X** / **U** | Chọn / loại / bỏ cờ |
| Gallery, xem trước | **[** / **]** | Xoay một phần tư sang trái / phải |
| Xem trước | **← →** | Tấm trước / tấm sau |
| Xem trước | **↑** | Cho vào giỏ in, sang tấm sau |
| Xem trước | **↓** | Để riêng, sang tấm sau |
| Xem trước | **C** | Mở màn cắt ảnh (chương 6) |
| Xem trước | **Cmd/Ctrl + / −**, chụm trên trackpad, các nút phóng | Phóng to / thu nhỏ (§7) |
| Xem trước | **Cmd/Ctrl 0**, nút vừa khung | Về vừa khung |
| Xem trước | nhấp đúp vào ảnh | Chuyển giữa vừa khung và 200 % |
| Xem trước | kéo ảnh (khi chưa phóng) | Chọn phần được in (*Cắt cho đầy*) |
| Xem trước | **Esc** | Rời màn cắt ảnh, rồi đóng |
| Màn cắt ảnh | **← →**, **⌥ ← →** | Cân ảnh từng độ, từng 0,1 độ |
| Màn cắt ảnh | **↑ ↓** (kèm **Shift**, xa hơn) | Dịch vùng cắt lên hoặc xuống |
| Màn cắt ảnh | **+** / **−** | Phóng to / thu nhỏ |
| Màn cắt ảnh | **0** | Đưa vùng cắt về giữa và góc về 0° |
| Màn cắt ảnh | **Enter** | Xong |
| Màn hình chụp | **Space** | Chụp một tấm |
| Màn hình chụp | **Esc** | Đóng và hiện tab của phiên chụp |
| Chế độ Chọn | nhấp, **Shift**+nhấp | Tích một tấm, tích cả dải |
| Chế độ Chọn | **Cmd/Ctrl+A** | Tích mọi ảnh đang hiện |
| Chế độ Chọn | **Enter** | Tích ảnh đang chọn |
| Chế độ Chọn | giữ **Space** | Xem nhanh; **← →** tấm sau, **Enter** tích, thả ra để đóng |
| Chế độ Chọn | **Esc** | Rời chế độ Chọn |
| Mọi nơi | **Cmd/Ctrl+Z**, **Shift+Cmd/Ctrl+Z** hoặc **Cmd/Ctrl+Y** | Hoàn tác, làm lại (chương 13) |
| Mọi nơi | **Cmd/Ctrl+N**, **Cmd/Ctrl+O**, **Cmd/Ctrl+S**, **Shift+Cmd/Ctrl+S** | Dự án mới, mở, lưu, lưu thành (chương 13) |

Ô nhập hay thanh trượt đang được chọn giữ các phím mũi tên cho riêng nó: nhấp vào ảnh để trả chúng về việc duyệt ảnh.
