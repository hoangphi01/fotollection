# Hướng dẫn sử dụng — Fotollection

> **Status:** Living · **Last reviewed:** 2026-11-04
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
5. **Bảng Thiết lập in** bên phải có, từ trên xuống: *Khổ in / xuất ảnh* (chương 5), *Dấu ngày* (chương 8), *Khung thông tin* (chương 9), *Đồng hồ máy ảnh* (chương 11) và *Địa điểm* (chương 10) khi thư viện đã có, rồi phần xuất ảnh (chương 12) và số phiên bản (chương 17).

<!-- shot: grid | Thư viện ảnh, đã chấm sao và chọn ảnh -->

**Ba chế độ:** các nút ở đầu thanh bên chuyển giữa **Đơn giản** (từng ảnh một, ↑ giữ, ↓ bỏ, rồi xuất: chương 18), **Ghép ảnh** (hai ảnh trên một tấm in: chương 19, hoặc bưu thiếp: chương 20) và **Pro**, phần app mà hướng dẫn này mô tả. Ảnh, giỏ in và thiết lập của bạn giống nhau ở mọi chế độ, và chế độ được nhớ trên máy này. **Cmd/Ctrl+1**, **2** và **3** chuyển giữa chúng, kể cả khi đang xem trước (bản xem trước sẽ đóng; trừ lúc đang cắt ảnh) ([FR-048](PRD.md#fr-048--simple-collage-and-professional-modes)).

**Ngôn ngữ và giao diện:** bấm số phiên bản ở chân bảng bất kỳ, rồi chọn *English* hoặc *Tiếng Việt* ở mục **Ngôn ngữ**; ở chế độ nào cũng giống nhau. Ba nút ở cuối thanh bên đổi giao diện: **Theo hệ thống**, **Sáng** hoặc **Tối** ([FR-020](PRD.md#fr-020--localization), [FR-052](PRD.md#fr-052--appearance-theme-title-bar-and-hints)).

<!-- chapter: cull -->

## 2. Sao, cờ và bộ lọc

Đánh dấu ảnh trong lúc xem, rồi chỉ hiện những tấm bạn muốn ([FR-022](PRD.md#fr-022--culling-rate-flag-filter)). Sao và cờ không bao giờ sửa tệp ảnh; chúng được lưu trong dự án.

1. Chọn một ảnh trong gallery, hoặc mở nó trong bản xem trước.
2. Nhấn **1**–**5** để chấm sao, **0** để xoá sao.
3. Nhấn **P** để chọn, **X** để loại, **U** để bỏ cờ. Ảnh bị loại vẫn ở trong gallery, mờ đi, cho đến khi bạn lọc nó ra; không gì bị xoá.
4. Trong bản xem trước, thẻ **Sao & cờ** làm việc đó bằng chuột: nhấp một ngôi sao (nhấp lần nữa để xoá), **Chọn (P)** hoặc **Loại (X)**.
5. Lọc gallery bằng nút **Bộ lọc** ở bên phải thanh công cụ. Nút mở ra ba lựa chọn:
   - cờ: **Tất cả**, **Đã chọn**, **Chưa gắn cờ**, **Đã loại** hoặc **File lạ**;
   - sao: **Mọi số sao**, hoặc **≥ 1** đến **≥ 5**;
   - loại tệp: **Mọi tệp**, **Chỉ RAW** hoặc **Không RAW**. Ảnh RAW có badge **RAW** màu chủ đề trong gallery.

   Khi có bộ lọc đang bật, nút có viền màu chủ đề và ghi số bộ lọc ("2 bộ lọc"), còn số ảnh cho biết đang hiện bao nhiêu trên tổng số. **Bỏ mọi bộ lọc** ngay trong đó, hoặc **Hiện tất cả ảnh** khi không ảnh nào khớp, đưa mọi ảnh trở lại.

   **File lạ** ([FR-071](PRD.md#fr-071--image-facts)) là những tệp nằm ngoài đường đi thông thường, có dấu **ⓘ** trong gallery: JPEG CMYK hoặc 16-bit, ảnh Adobe RGB, RAW X-Trans của Fujifilm, loại DNG mà app chưa tự develop. Thẻ **Ảnh này** trong bản xem trước cho biết mỗi tệp là gì, ngay dưới kích thước: ví dụ *JPEG · 4:2:0 · Baseline · 8-bit · Display P3 · đọc bằng zune-jpeg* hoặc *ARW · Bayer RGGB · 14-bit · đọc bằng ImageIO*. RAW không phải DNG hiện cảm biến sau khi đã mở trong bản xem trước. Hãy gửi kèm dòng này khi báo lỗi.

<!-- shot: manual-filters | Bộ lọc đang mở phía trên gallery: cờ, sao và loại tệp -->

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

**Xem nhanh ảnh:** thẻ đầu tiên bên phải, **Ảnh này**, cho biết bạn đang xem gì ([FR-078](PRD.md#fr-078--the-photo-at-a-glance-in-the-preview)): máy ảnh và ống kính (theo tên bạn đặt cho khung thông tin, chương 9), thông số phơi sáng đúng như khung in ra (*20mm · F1.7 · 1/60s · ISO800*), chụp ở đâu và khi nào, kích thước theo điểm ảnh, megapixel và megabyte, và tệp đó là gì (chương 2). Bấm vào tiêu đề để thu gọn thẻ, bấm lần nữa để mở; thẻ giữ nguyên như bạn để cho những ảnh tiếp theo. Các thẻ bên dưới dùng để sửa ngày và nơi chụp.

<!-- shot: manual-summary | Thẻ Ảnh này ở đầu bản xem trước: máy và ống kính, thông số, nơi và ngày chụp, kích thước -->

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
   - **Xuất ảnh**: khi đang có ảnh được tích, nút xuất ở chân bảng ghi *Xuất 3 ảnh đã chọn* và chỉ xuất những ảnh đó; bỏ tích hết thì nút lại xuất cả khung đang xem.
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
- **Xuất ảnh theo đúng chỗ bạn đang xem** (chương 12), trừ khi đang có ảnh được tích: khi đó chỉ những ảnh đó được xuất (§4.2). Từ *Giỏ in*, *Xuất ảnh* ghi đúng những tấm trong giỏ, với ước tính dung lượng ngay trên nút ([FR-053](PRD.md#fr-053--export-for-social-media)). Từ *Tất cả ảnh* hay một thư mục, ảnh để riêng không bao giờ được xuất.

<!-- shot: manual-share-fill | Mạng xã hội khi bật Giữ trọn ảnh trong khung: Vuông 1:1 và nền trắng -->

<!-- shot: estimate | Ô ước tính dung lượng ngay trên nút xuất ảnh, khi xuất cho Facebook -->

<!-- chapter: size -->

## 5. Khổ in và độ nét

Chọn khổ bạn sẽ in, Fotollection cắt mọi ảnh theo khổ đó và cho biết từng tấm có đủ nét không ([FR-026](PRD.md#fr-026--resolution-warning), [FR-043](PRD.md#fr-043--custom-print-sizes), [FR-045](PRD.md#fr-045--original-aspect-ratio), [FR-067](PRD.md#fr-067--3-mm-border-on-every-print)).

1. Trong thẻ **Khổ in / xuất ảnh**, giữ lựa chọn **In ảnh** (với **Mạng xã hội**, xem chương 12).
2. Mở danh sách khổ:
   - **Tiêu chuẩn (CEWE / EU)**: các khổ của máy in ảnh tại dm và Rossmann, như *10 × 15 cm · 2:3*;
   - **Gốc**: tỉ lệ gốc của ảnh, không cắt;
   - **Tùy chỉnh**: các khổ bạn tự thêm.

   Mỗi khổ có một huy hiệu cho ảnh đang chọn trong gallery: **Nét** (từ 300 DPI), **Tạm được** (150–299 DPI, ổn khi cầm xem) hoặc **Sẽ vỡ** (dưới 150 DPI). Rê chuột lên huy hiệu để xem số DPI.
3. Dưới danh sách, một dòng cho biết ảnh đang chọn in nét tới cỡ nào, và vẫn chấp nhận được tới cỡ nào.
4. Mọi ảnh được cắt theo tỉ lệ của khổ, **mặc định ở giữa**. Chọn phần được giữ cho từng ảnh trong bản xem trước (chương 6).
5. **Viền 3 mm** để một viền rộng 3 mm ở mọi cạnh, **Trắng** hoặc **Đen** tuỳ bạn chọn dưới công tắc, lấn vào ảnh: bản in vẫn đúng khổ của máy in tại tiệm. Với khổ **Gốc**, không mất pixel nào: viền bao quanh cả tấm ảnh, cạnh ngắn vẫn 10 cm và cạnh dài thêm 6 mm. Ảnh xuất cho mạng xã hội không bao giờ có viền. Khi bật khung thông tin, viền theo màu khung (chương 9).

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

- **Vuốt hai ngón lên hoặc xuống** (hoặc lăn chuột), **chụm hai ngón** trên trackpad, hoặc bấm **Cmd/Ctrl +** và **Cmd/Ctrl −**: ảnh phóng quanh con trỏ, tới 800 %. Vuốt lên là phóng to.
- **Nhấp đúp** vào ảnh để chuyển giữa vừa khung và 200 %.
- Khi đã phóng to, **kéo** ảnh, hoặc vuốt ngang bằng hai ngón, để di chuyển trong ảnh.
- **Cmd/Ctrl 0** hoặc nút vừa khung đưa về toàn bộ ảnh.

Bản xem trước ban đầu được hiện ở cỡ nhẹ để máy nào cũng vẽ nhanh. Khi bạn phóng vượt quá cỡ đó, một vòng xoay nhỏ hiện cạnh mức phóng, và một lát sau khi bạn ngừng di chuyển, phần trên màn hình được thay bằng điểm ảnh **độ phân giải đầy đủ**, đúng như trong tệp sẽ xuất. Chỉ phần đang thấy được tải, nên kể cả ảnh 32 MP vẫn nhẹ.

<!-- chapter: stamp -->

## 8. Dấu ngày

Ngày ở góc ảnh in, như máy ảnh phim, lấy theo lúc chụp ([FR-004](PRD.md#fr-004--date-stamp), [FR-033](PRD.md#fr-033--retro-film-date-stamp-and-stamp-presets)). Mặc định đang bật.

1. Trong thẻ **Dấu ngày**, công tắc bật hoặc tắt dấu cho mọi ảnh. Thẻ này như nhau ở chế độ Đơn giản, Ghép ảnh và Chuyên nghiệp, đủ mọi lựa chọn dưới đây.
2. Chọn:
   - **định dạng**: `2026-07-14`, `14.07.2026` hoặc `'26 07 14`, kiểu máy ảnh phim;
   - **góc**: *Góc dưới phải*, *Góc dưới trái*, *Góc trên phải* hoặc *Góc trên trái*;
   - **màu**: **Trắng** hoặc **Cam retro**;
   - **cỡ**: **Nhỏ**, **Vừa** hoặc **To**, như nhau ở mọi chế độ. Cỡ chữ theo khổ in: khổ lớn thì chữ lớn hơn, và trên tấm ghép mỗi dấu ngày bằng nửa dấu của cả tấm, vì hai ảnh dùng chung tấm.
3. **Địa điểm cùng dòng ngày** thêm tên thị trấn nơi chụp, lấy từ GPS của ảnh, không cần mạng: dạng **Thành phố**, **+ mã nước** hoặc **+ tên nước**. Ảnh không có GPS có thể đặt nơi chụp bằng tay (chương 10).
4. **Dòng chữ dưới ngày**: một dòng ngắn chạy ngang ảnh, tối đa 80 ký tự, in bằng chữ hoa không dấu (ví dụ *KY NIEM BUDAPEST*).

<!-- shot: stamp | Dấu ngày kèm tên nơi chụp từ GPS của ảnh -->

**Riêng một tấm:** trong bản xem trước, thẻ **Dấu ngày** có đủ các điều khiển trên cho riêng ảnh đó; ảnh được đánh dấu **Riêng ảnh này**. **Áp dụng cho tất cả** biến dấu của ảnh này thành dấu cho mọi ảnh. Trong bảng bên, **Đặt lại n ảnh có dấu riêng** cho mọi ảnh theo lại bảng bên. Cả hai đều hoàn tác được.

Ngày in ra là ngày của máy ảnh, đã chỉnh theo đồng hồ máy ảnh (chương 11) hoặc đặt bằng tay (chương 10).

<!-- chapter: frame -->

## 9. Khung thông tin "Shot on"

Một dải trắng hoặc đen dưới ảnh, với logo và thông số máy ảnh, như ảnh chia sẻ từ điện thoại ([FR-040](PRD.md#fr-040--exif-info-frame), [FR-065](PRD.md#fr-065--border-in-crop-to-fill), [FR-067](PRD.md#fr-067--3-mm-border-on-every-print)). Mặc định đang tắt, và giống nhau cho mọi ảnh.

1. Bật thẻ **Khung thông tin**.
2. Chọn cách đặt khung vào ảnh in:
   - **Cắt cho đầy**: ảnh in giữ đúng khổ, dải thông tin nằm bên trong; kéo ảnh trong bản xem trước để chọn phần được giữ. Khi bật **Viền 3 mm** (trong thẻ khổ in, chương 5), viền 3 mm theo màu khung chạy ở cạnh trên và hai bên, vẫn nằm trong khổ in, như ảnh lấy liền;
   - **Giữ trọn ảnh**: cả tấm ảnh đúng khổ in, đặt trong khung như ảnh instax với viền 5 mm ở ba cạnh và dải thông tin ở dưới, nên ảnh in ra lớn hơn khổ đã chọn. Một dòng ghi chú cho biết lớn bao nhiêu, để bạn chọn giấy phù hợp ở tiệm. Khi bật **Viền 3 mm**, cả tấm ảnh được thu vào trong khổ đã chọn, và ảnh in không to ra.
3. Chọn **Trắng** hoặc **Đen**, định dạng ngày, và phông chữ: *Inter (có sẵn)*, hoặc **Nhập OTF/TTF…** để dùng phông của bạn ([FR-042](PRD.md#fr-042--custom-fonts)).
4. Trong **Hiển thị**, tích những gì dải thông tin ghi: *Logo*, *Máy ảnh*, *Ống kính*, *Tiêu cự*, *Khẩu độ*, *Tốc độ màn trập*, *ISO*, *Photo Style*, *Ngày*, *Địa điểm*.
5. Khi hiện logo, chọn **Màu logo** (*Cổ điển*, *Màu gốc* hoặc *Đơn sắc*) và **Cỡ logo** (nhấp đúp vào thanh trượt để về 100 %).

<!-- shot: frame | Khung “Shot on” với logo của hãng máy ảnh -->

<!-- shot: manual-frame | Thẻ Khung thông tin: cách đặt khung, màu, phông và các thông tin hiển thị -->

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

   **Giữ trọn ảnh trong khung**, ngay dưới cỡ chia sẻ, giữ nguyên vẹn mọi ảnh khi đăng lên một lưới ảnh cùng khuôn ([FR-075](PRD.md#fr-075--fit-a-shared-photo-whole-into-a-ratio)): chọn tỷ lệ (**Vuông · 1:1**, **Dọc · 4:5**, **3:4** hoặc **2:3**, **Ngang · 3:2** hoặc **16:9**, **Story · 9:16**, hoặc **Tỷ lệ tùy chỉnh…** với hai con số của nó), rồi chọn nền: **Trắng**, **Đen** hoặc **Tùy chọn**, với bảng chọn màu và mã màu. Mỗi ảnh nằm giữa khung và không bị cắt mất phần nào; trên Instagram khung rộng 1080 px. Ảnh in không dùng tính năng này: chúng vẫn theo khổ in.
3. **Lưu vào đâu:** nhấp **Chọn nơi lưu…** ở cuối bảng bên, rồi chọn một thư mục. Ngay phía trên, chọn **Một thư mục** (mọi tệp vào thẳng thư mục đó) hoặc **Theo ngày** (mỗi ngày một thư mục, ví dụ `2026-07-14`); app nhớ lựa chọn này trên máy.
4. **Bao nhiêu thư mục:** khi đang hiện nhiều thư mục, chọn **Chung** (gộp chung) hoặc **Riêng** (mỗi thư mục một thư mục con mang tên nó).
5. Phía trên nút, phần **ước tính** cho biết các tệp sẽ nặng bao nhiêu. Nhấp **Xuất n ảnh**. Một thanh hiện tiến độ, và **Hủy** dừng lại: các ảnh đã xuất xong vẫn giữ, tệp đang ghi dở bị xoá. Ảnh nào không xuất được sẽ bị bỏ qua, và phần tổng kết cho biết bao nhiêu ảnh lỗi.

<!-- shot: estimate | Ô ước tính dung lượng ngay trên nút xuất ảnh, khi xuất cho Facebook -->

Với **Theo ngày**, tệp được xếp vào mỗi ngày một thư mục, đặt tên theo ngày (`2026-07-14`), và `undated` cho ảnh không có ngày. Tệp chia sẻ luôn nằm trong thư mục `Share`. Không tệp nào ghi đè tệp khác: bản thứ hai mang thêm `_1`, `_2`. Thông số máy ảnh được giữ, còn **vị trí GPS luôn bị xoá**, để ảnh chia sẻ không làm lộ nơi bạn ở.

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
- Ảnh được xử lý bằng bộ xử lý của riêng Fotollection cho mọi máy ảnh có cảm biến Bayer (trên máy Mac, tệp Panasonic và Olympus do macOS xử lý để được sửa méo ống kính; Fujifilm X-Trans do trình đọc có sẵn xử lý). Fotollection chuẩn bị ảnh để in: không thay thế Lightroom, nhưng ảnh RAW chỉnh được sâu hơn ảnh JPEG.
- **Chỉnh RAW sâu hơn.** Với ảnh RAW, *Sáng*, *Vùng sáng*, *Vùng tối*, *Ấm / lạnh* và *Tint* tác động trực tiếp lên dữ liệu gốc của máy ảnh trước khi ảnh được hoàn thiện, như trong một trình sửa RAW: kéo vùng sáng xuống lấy lại chi tiết gần trắng, kéo vùng tối lên ít bị vằn hơn. (Trên máy Mac, ảnh RAW Panasonic và Olympus do macOS xử lý nên còn ít dư sáng hơn.)
- **Màu RAW.** Ảnh RAW mở ra ở chế độ **Trung tính**: màu của cảm biến với độ sáng của máy và một đường cong nhẹ, điểm xuất phát dịu để bạn tự chỉnh màu. Trong các thẻ màu của ảnh (bản xem trước, hoặc ảnh được chọn trên một tấm ghép), thẻ *Màu RAW* đổi ảnh sang **Như JPEG của máy**, với độ tương phản và màu như trên màn hình máy ảnh; *Áp dụng cho mọi ảnh RAW* làm việc đó cho mọi ảnh RAW. Ảnh nhỏ trong lưới luôn là bản xem trước của máy. Project lưu bằng phiên bản cũ hơn vẫn giữ màu của máy cho ảnh RAW của nó.
- **Trên Linux**, RAW được mở bằng trình đọc có sẵn trong Fotollection, không cần cài thêm gì.
- **Trên Windows**, RAW cũng được mở bằng trình đọc có sẵn trong Fotollection, không cần cài thêm gì ([ADR-0029](adr/0029-raw-through-rawler-then-our-own-decoder.md)). Nếu đã cài *Raw Image Extension* của Microsoft, codec của hệ điều hành là phương án dự phòng cuối cùng cho tệp mà trình đọc có sẵn không mở được.

<!-- chapter: open-with -->

## 15. Mở ảnh từ Finder, Explorer hoặc trình quản lý tệp

Fotollection có thể là app mở ảnh khi bạn nhấp đúp ([FR-054](PRD.md#fr-054--open-photos-with-fotollection)). Nó không bao giờ tự đặt mình làm mặc định: bạn chọn.

- **Một lần:** nhấp phải vào ảnh → **Mở bằng** → **Fotollection** (macOS), hoặc **Mở bằng** → **Fotollection** (Windows). Trên Linux, khi đã cài bản `.deb`: nhấp phải → **Open With** → **Fotollection** trong Files, Dolphin hoặc Nemo; bản AppImage không có trong danh sách này, trừ khi một công cụ như AppImageLauncher đăng ký nó.
- **Luôn luôn:** trên Mac, chọn ảnh, nhấn Cmd+I, chọn Fotollection trong *Mở bằng* rồi nhấp *Thay đổi tất cả*. Trên Windows, chọn **Mở bằng** → **Chọn ứng dụng khác**, chọn Fotollection và tích *Luôn dùng ứng dụng này*.

Ảnh mở ngay trong bản xem trước. Thư mục của nó thành một tab ở thanh bên, chỉ có ảnh của đúng thư mục đó (không gồm thư mục con), nên **← →** đi qua cả thư mục. Mọi thứ hoạt động như ở bất kỳ tab nào: sao, giỏ in, khung cắt, xuất ảnh. Xong thì nhấp **×** trên tab; ảnh vẫn nằm trên ổ đĩa của bạn. Mở một ảnh khác khi Fotollection đang chạy sẽ đưa cửa sổ lên trước với ảnh đó.

<!-- chapter: camera -->

## 16. Chụp với máy ảnh cắm vào máy tính

Khi máy ảnh cắm cáp, Fotollection hiện những gì máy thấy và chụp ảnh ([FR-062](PRD.md#fr-062--live-view-from-a-camera), [FR-063](PRD.md#fr-063--tethered-capture-over-usb)). Hiện tính năng này chạy trên Windows, với Sony a6400; bản Linux không có chụp qua máy ảnh ([ADR-0034](adr/0034-linux-lean.md)). Các máy Sony cùng thời có thể chạy được, nhưng chưa được kiểm tra.

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
   - **Linux AppImage:** nút này tải AppImage mới; cho phép chạy (`chmod +x`) rồi dùng nó thay cho tệp cũ.
   - **Linux `.deb`:** nút này mở trang phát hành; tải `.deb` mới rồi chạy `sudo apt install ./Fotollection_*_amd64.deb` (`_arm64.deb` trên ARM).

Cùng ô đó có mục **Ngôn ngữ** (chương 1) và hiện **cache**: ảnh nhỏ, bản xem trước và kết quả phân tích ảnh đẹp nhất mà Fotollection giữ để mở thư mục và ảnh nhanh, kèm dung lượng so với giới hạn 5 GB. Vượt giới hạn, các mục cũ nhất tự bị xoá. **Xoá cache** làm trống nó bất kỳ lúc nào; không mất gì của bạn, và ảnh nhỏ sẽ được tạo lại khi bạn duyệt.

**Bản xem trước tức thì.** Sau khi có ảnh nhỏ của một thư mục, Fotollection chuẩn bị sẵn bản xem trước của mọi ảnh ở chế độ nền (*Đang chuẩn bị bản xem trước* trên thanh của gallery) và giữ trong cache, nên mở ảnh nào cũng hiện ngay, kể cả ảnh RAW, và → ← hiện ảnh kế tiếp không phải chờ. Việc này dùng tối đa một phần tư bộ nhớ máy, và giới hạn cache là 5 GB (trước đây 2 GB).

**Gặp lỗi, hay có ý tưởng?** Bấm số phiên bản ở chân bảng, rồi nút đỏ **Báo lỗi / góp ý**: nút này mở trang báo lỗi và góp ý của dự án trên GitHub, trong trình duyệt ([FR-069](PRD.md#fr-069--report-a-problem)). Hãy ghi bạn đã làm gì và thấy gì, rồi bấm **Sao chép thông tin chẩn đoán** và dán vào: phần này có phiên bản, hệ điều hành và các mã lỗi gần nhất.

**Khi gặp mã lỗi.** Mỗi thông báo lỗi kết thúc bằng một mã, ví dụ `ERR-IMG-0006`. Mấy chữ đầu cho biết loại lỗi: `IMG` do chính file ảnh, `FS` một file hay thư mục không đọc hoặc ghi được, `EXT` hệ điều hành thiếu codec, `DRV` kết nối với máy ảnh, `CFG` thiết lập, font hay logo, `EXP` khi xuất ảnh, `NET` khi *Kiểm tra cập nhật*, `RUS` lỗi của chính ứng dụng. Nút × trên thông báo lỗi để ẩn nó; thông báo chỉ hiện lại nếu lần thử sau lại gặp đúng lỗi đó. Mỗi mã được giải thích, kèm cách xử lý, trong [ERROR_CONTROL.md](ERROR_CONTROL.md). Ứng dụng giữ một nhật ký nhỏ các lỗi này trên máy (không có ảnh, địa điểm hay tên file) chỉ để *Sao chép thông tin chẩn đoán* dùng; ứng dụng không gửi gì đi.

<!-- chapter: simple -->

## 18. Chế độ Đơn giản

*Mới trong v0.9.0.* Đường ngắn nhất từ một thư mục tới ảnh in: xem mỗi ảnh một lần, giữ hoặc bỏ, rồi xuất những ảnh đã giữ ([FR-048](PRD.md#fr-048--simple-collage-and-professional-modes), [FR-050](PRD.md#fr-050--print-basket)).

1. Nhấp **Đơn giản** ở đầu thanh bên, và mở một thư mục nếu thư viện còn trống.
2. Ảnh đầu tiên hiện ở giữa, **đúng như khi in**, với khổ in, dấu ngày và viền của bạn.
3. Nhấn **↑** hoặc nhấp **Giữ**: ảnh vào giỏ in, và ảnh kế tiếp hiện ra.
4. Nhấn **↓** hoặc nhấp **Bỏ**: ảnh được để riêng, không bao giờ bị xoá, và ảnh kế tiếp hiện ra.
5. **← →** đi lùi, đi tới mà không quyết định. **Cmd/Ctrl+Z** hoàn tác lần giữ hoặc bỏ vừa rồi. Các phím lọc ảnh của bản xem trước cũng dùng được: **0**–**5** chấm sao, **P** / **X** / **U** gắn cờ, **[** / **]** xoay một phần tư.
6. Thanh phía trên ảnh cho biết bạn đang ở ảnh nào, đã giữ và đã bỏ bao nhiêu.
7. Ở bảng bên phải, chọn **In ảnh** hoặc **Mạng xã hội**, **khổ in**, **viền 3 mm** và **dấu ngày** (định dạng, góc, màu, cỡ, địa điểm và dòng chữ riêng, như ở Chuyên nghiệp), rồi **Chọn nơi lưu…** và **Xuất**: app lưu những ảnh bạn đã giữ.

<!-- shot: simple | Chế độ Đơn giản: ảnh đúng như khi in, nút Giữ và Bỏ, và Xuất những ảnh đã giữ -->

Ảnh đã giữ ở chế độ Đơn giản cũng nằm trong giỏ in ở Pro, nên bạn có thể sang Pro bất kỳ lúc nào để cắt hay thêm khung cho một ảnh, rồi quay lại.

<!-- chapter: collage -->

## 19. Chế độ Ghép ảnh

*Mới trong v0.9.0.* Hai ảnh trên một tấm in, như máy ảnh film half-frame chụp hai khung trên một ô phim: một tấm 10 × 15 chứa hai ảnh 10 × 7,5, nên in cả bộ chỉ tốn một nửa ([FR-035](PRD.md#fr-035--multi-photo-layouts)).

1. Nhấp **Ghép ảnh** ở đầu thanh bên. Chọn ảnh để ghép ở thanh bên, thường là **Giỏ in**.
2. Các ảnh được ghép theo thứ tự, 1 + 2, 3 + 4…, và mỗi tấm hiện đúng như khi in. Ảnh lẻ cuối cùng để nửa kia trắng.
3. **Khổ tấm in**: khổ in, ví dụ 10 × 15; khổ Gốc không có ở đây, vì tấm ghép cần một khổ cố định. Tấm được chia đôi theo cạnh dài và xoay theo ảnh đầu tiên: hai ảnh ngang thì trên dưới, hai ảnh dọc thì cạnh nhau. Nút **xoay** trên mỗi tấm, hoặc phím **R**, xoay tấm sang hướng kia. **Trắng** hoặc **Đen** dưới *Viền 3 mm* tô cả phần giấy quanh các ảnh lẫn viền.
4. **Hai ảnh mỗi tấm**:
   - **Lấp đầy mỗi nửa**: không lề trắng. Mỗi ảnh phủ kín nửa tấm (7,5 × 10 với tấm 10 × 15) và hai ảnh giáp cạnh nhau; cắt từng ảnh (bước 7) để chọn phần được giữ, bằng cách kéo và phóng to. Các thiết lập bên dưới, cả đường cắt, khi đó được ẩn.
   - **Tỷ lệ ảnh**: tỷ lệ gốc của từng ảnh (**Ảnh**), tỷ lệ của tấm (**Tấm**, 2:3 với 10 × 15), **4:3** hoặc **1:1**. Phần còn lại của nửa tấm để trắng.
   - **Cỡ ảnh**: app gợi ý cỡ lớn nhất còn chừa 5 mm trắng quanh ảnh, ví dụ 6 × 9 cm cho ảnh 2:3 trong nửa tấm 7,5 × 10. Bạn có thể gõ cỡ khác; khi **khoá** tỷ lệ, nhập một cạnh thì cạnh kia tự tính. **Dùng gợi ý** để quay lại.
   - **Đường cắt**: một nét mảnh giữa hai ảnh, để cắt tấm in làm đôi. Tắt đi thì không có nét cắt, và hai ảnh vẫn giữ khoảng trắng giữa chúng.
   - Dòng ghi chú bên dưới cho biết cỡ ảnh, phần còn lại trong **viền 3 mm** (chương 5) và độ nét.
5. **Đổi thứ tự:** kéo một ảnh thả lên ảnh khác để đổi chỗ, hoặc thả vào nửa tấm trống để đưa nó xuống cuối. **← →** chọn ảnh, **⌥/Alt ← →** dời ảnh đi một chỗ. Đổi thứ tự thì mỗi tấm quay về hướng hợp với ảnh đầu của nó, và ảnh thêm vào sau sẽ nằm ở cuối.
   - **Tự xoay** (nút đũa phép trên các tấm), khi in ra để cắt đôi: mọi tấm xoay thành ngang và mỗi ảnh xoay cho đứng, nên không còn ảnh đứng cạnh ảnh nằm. **Về chiều gốc** đưa mọi tấm và ảnh về chiều của nó. Mỗi nút là một bước hoàn tác. Nút **ⓘ** cạnh số tấm liệt kê các phím tắt.
6. **Chọn ảnh để in**, như trong gallery (chương 4): **Chọn** để tích cả tấm bằng một cú nhấp, cả hai ảnh cùng lúc, viền chung một khung (**Cmd/Ctrl+A** chọn tất cả, **Enter** tích tấm đang chọn), rồi thanh công cụ đưa ảnh của các tấm đó vào **Giỏ in**, để riêng hoặc đưa về gallery. Khi đang có tấm được tích, nút xuất ghi *Xuất 2 tấm đã chọn* và chỉ xuất những tấm đó. Ảnh trong giỏ có biểu tượng giỏ. Chọn **Giỏ in** ở thanh bên để chỉ ghép những ảnh đó.
7. **Xem trước:** nhấp đúp vào một tấm, hoặc nhấn **Enter** hay **Space**, để mở tấm cỡ lớn trong cửa sổ xem trước, như mở ảnh từ gallery. Ở đó:
   - tấm mở ra không chọn ảnh nào, đúng như khi in. Nhấp vào một ảnh (hoặc **1**, **2**) để chọn, nhấp lần nữa (hoặc **Esc**) để bỏ chọn;
   - **← →** sang các tấm khác; khi đã chọn ảnh, **↑** cho ảnh vào giỏ in và **↓** để riêng (hoặc nút giỏ cạnh ảnh), các nút phóng to hoặc **Cmd/Ctrl + / −** để phóng;
   - **Cắt ảnh** (hoặc **C**, hoặc nút cắt cạnh mỗi ảnh) cắt và cân ngang ảnh đang chọn cho vị trí của nó trên tấm; việc này không đổi bản in của ảnh, và Đặt lại đưa vùng cắt về giữa;
   - **Xoay tấm** (**R**) xoay tấm sang hướng kia;
   - **Xoay ảnh** (**⇧R**) xoay ảnh đang chọn 90° trong nửa của nó, chỉ trên tấm này, còn tấm giữ nguyên (**Xoay tấm** mới xoay cả tấm): khi một ảnh ngang chung tấm với một ảnh dọc, ảnh ngang sẽ lấp đầy nửa của nó thay vì nằm nhỏ ở giữa. Bản in riêng của ảnh không đổi;
   - bảng bên phải liệt kê hai ảnh, có cùng thẻ *Hai ảnh mỗi tấm* và *Dấu ngày*, và **Xuất tấm này** lưu riêng tấm đó. Ở *Dòng chữ dưới ngày*, tắt **Giống nhau cho cả 2 ảnh** để gõ dòng riêng cho từng ảnh; phần còn lại của dấu ngày vẫn như nhau cho cả hai;
8. **Xuất** ở bảng bên phải lưu mỗi tấm một tệp, đặt tên theo hai ảnh của nó (ví dụ `P1000123+P1000124.jpg`), trong thư mục ngày của ảnh đầu khi xuất theo ngày.

<!-- shot: collage | Chế độ Ghép ảnh: các tấm đúng như khi in, cùng tỷ lệ ảnh, cỡ gợi ý và đường cắt -->

<!-- shot: manual-collage-preview | Bản xem trước một tấm: tấm cỡ lớn, hai ảnh của nó, thẻ Hai ảnh mỗi tấm và Xuất tấm này -->

Mỗi thay đổi là một bước hoàn tác. Dấu ngày được in trên từng ảnh, kèm địa điểm và dòng chữ riêng nếu bạn chọn; khung thông tin không dùng trên tấm ghép. Chế độ Ghép ảnh cũng làm được bưu thiếp: chương 20.

<!-- chapter: postcards -->

## 20. Bưu thiếp

*Mới trong v0.9.4.* Một tấm ảnh in ở kiosk thành bưu thiếp gửi được qua bưu điện: một ảnh ở một nửa, mặt sau bưu thiếp ở nửa kia ([FR-073](PRD.md#fr-073--postcard-sheets)).

1. Trong chế độ Ghép ảnh, bật công tắc **Bưu thiếp** ở thanh phía trên các tấm, cạnh nút **Chọn** (tắt để quay về hai ảnh mỗi tấm). Mỗi ảnh giờ thành một tấm riêng: 50 ảnh là 25 tấm hai ảnh, hoặc 50 bưu thiếp. Thẻ **Bưu thiếp** hiện ra trong bảng bên phải.
2. **Chiều cầm**: **Ngang** đặt ảnh bên trái, bưu thiếp bên phải; **Dọc** đặt bưu thiếp ở trên, ảnh ở dưới. Một **ô dán tem** nét đứt nằm ở góc trên bên phải, nhỏ hơn phần lớn các con tem nên tem dán lên sẽ che kín nó, và cả dải phía trên để trống cho thêm tem hoặc nhãn cước dài in tại bưu điện. Các **dòng kẻ** nằm cùng nhau ở phía dưới: **3 dòng** hoặc **4 dòng** cho lời nhắn, rồi 3 dòng cho địa chỉ; bưu thiếp ngang xếp trên dưới, bưu thiếp dọc chia hai cột (lời nhắn bên trái, địa chỉ bên phải).
3. Nửa ảnh vẫn như trước: tỷ lệ, cỡ ảnh, *Lấp đầy mỗi nửa*, viền 3 mm, dấu ngày, **Cắt ảnh** và **Xoay ảnh** trong bản xem trước. Bưu thiếp không có đường cắt và không xoay tấm; nút Tự xoay được ẩn.
4. **Chữ trên bưu thiếp**, thẻ ngay bên dưới, hoặc để trống các dòng để viết tay. Chọn một bưu thiếp (bấm vào nó, hoặc mở bản xem trước) rồi gõ **Lời nhắn** (tối đa 400 ký tự, trên các dòng phía trên) và **Địa chỉ** (tối đa 200 ký tự; trên 3 dòng của nó, hoặc 4 dòng nhỏ hơn trong cùng khoảng đó khi địa chỉ dài). Các nút **B**, *I*, U cạnh mỗi phần làm phần đó đậm, nghiêng hoặc gạch chân trên mọi bưu thiếp. Cũng thẻ đó chỉnh cách mọi bưu thiếp in chữ: **Font**, font máy đánh chữ *Xanh Mono*, font viết tay *Patrick Hand* hoặc font của bạn (**Nhập OTF/TTF…**, như chương 9), có đủ chữ của mọi tiếng châu Âu viết bằng chữ Latin (chữ Hy Lạp hay Cyrillic lấy từ font kia của app), nên địa chỉ dán vào in đúng như gõ; **Cỡ chữ** nhỏ, vừa hoặc lớn; **Màu chữ**, **Đen** (thành trắng trên giấy đen), **Xanh**, **Nâu** hoặc **Đỏ**; và **Dòng ghi công (tuỳ chọn)**, một dòng chữ nhỏ ở chân mọi bưu thiếp, ví dụ *Ảnh © Anna Schmidt*. Phần không vừa các dòng của nó sẽ không được in, đúng như bản xem trước, và một dấu **!** đỏ hiện trên bưu thiếp ở lưới, ở bản xem trước và ở phần bị cắt: hãy rút ngắn, hoặc chọn cỡ chữ nhỏ hơn.
5. **Xuất ảnh** ghi mỗi bưu thiếp thành một tệp, đặt tên theo ảnh của nó (ví dụ `P1000123-postcard.jpg`).

**Một địa chỉ cho cả bộ** ([FR-077](PRD.md#fr-077--one-address-for-many-postcards)). Gõ địa chỉ ở một tấm; ngay bên dưới, **Dùng cho N bưu thiếp chưa có địa chỉ** điền nó vào mọi tấm đang hiện chưa có địa chỉ, trong một bước mà **Cmd/Ctrl+Z** hoàn tác được. **Hoặc cho cả N tấm khác** thay luôn địa chỉ của chúng. Khi đang tick bưu thiếp ở chế độ chọn, địa chỉ chỉ vào các tấm đã tick. Ở tấm chưa có địa chỉ, **Địa chỉ gần đây** liệt kê các địa chỉ vừa gõ trên máy này: bấm một mục để điền vào tấm, hoặc × để quên nó. Chúng chỉ nằm trên máy này, không bao giờ vào file dự án.

<!-- shot: manual-postcard-apply | Thẻ Chữ trên bưu thiếp: địa chỉ đã gõ, và Dùng cho N bưu thiếp chưa có địa chỉ bên dưới -->

<!-- shot: manual-postcard-recent | Một tấm chưa có địa chỉ: Địa chỉ gần đây để chọn -->

**Viết tay.** Mặt ảnh in là một lớp phủ mà nhiều loại bút không ăn. Hãy dùng bút dạ dầu nét mảnh (Sharpie Ultra Fine, edding 400, Stabilo OHPen S), thử ở ô tem trước và để khô khoảng một phút; bút bi hay bị đứt nét, bút gel và bút kim dễ lem. Chữ gõ thì không gặp các vấn đề này.

**Gửi đi.** Ảnh 10 × 15 thực ra là 15,2 × 10,2 cm: Deutsche Post nhận bưu thiếp từ 14 × 9 đến 23,5 × 12,5 cm và 150–500 g/m² (giấy ảnh khoảng 240), quy định của Magyar Posta ghi từ 9 × 14 đến 12 × 23,5 cm (nên hỏi lại tại quầy). Česká pošta chỉ nhận bưu thiếp dài tối đa 14,8 cm, nên ở Séc nó được tính như thư. Bưu điện có thể in mã vạch dọc mép dưới, mực này dễ bị chùi trên ảnh bóng; khi không chắc, hãy hỏi tại quầy.

**Giấy bóng và bưu điện.** Dấu bưu điện đóng lên chính con tem thì không sao, vì tem là giấy; nhưng phần dấu lan ra lớp phủ bóng có thể nhoè trước khi mực khô, và máy phân loại có thể làm xước mặt ảnh. Nếu cửa hàng cho chọn, hãy in bưu thiếp trên giấy **mờ (matt)**; dùng tem tự dính, vì tem phải liếm nước có thể không bám trên lớp phủ; hoặc cho vào phong bì và gửi như thư. Hãy gửi thử một tấm trước khi gửi cả bộ.

<!-- shot: manual-postcard | Chế độ Ghép ảnh khi bật Bưu thiếp: tàu điện Praha về đêm, mỗi tấm in có một ảnh tràn nửa tấm và mặt sau bưu thiếp với ô tem và dòng kẻ -->

<!-- shot: postcard-night | Bản xem trước một bưu thiếp: tàu điện đêm, lời nhắn và địa chỉ Bảo tàng Quốc gia đã gõ, thẻ Chữ trên bưu thiếp -->

<!-- chapter: colour -->

## 21. Màu: Cơ bản và Công cụ

Hai thẻ chỉnh màu ảnh, nằm dưới **Khổ in / xuất**, chỉ có trong bản xem trước: mở một ảnh (hoặc một tấm ghép) để chỉnh màu. Chúng chỉ đổi riêng ảnh đó, như dấu ngày, và **Dùng look này cho mọi ảnh** biến look của ảnh đó thành look chung.

- **Cơ bản** hiện bốn thanh: **Sáng**, **Tương phản**, **Ấm / lạnh** và **Bão hoà**. **Thêm** mở thêm **Vùng sáng**, **Vùng tối**, **Tint** và **Rực màu**. Rê chuột lên một thanh để đọc nó làm gì; nhấp đúp để đưa về 0. Bấm hơi lệch lên trên hay xuống dưới thanh vẫn kéo được, và nhấp vào tên thanh thì phím ← → chỉnh nó (mỗi lần 1, giữ Shift thì 10).
- **Công cụ** gập sẵn; nhấp vào tiêu đề để mở. **Fade** nâng điểm đen, **Nhuộm vùng tối** và **Nhuộm vùng sáng** phủ một màu lên phần tối và phần sáng (có **Cân bằng** giữa hai vùng), còn **Trắng đen** chuyển ảnh sang xám qua một kính lọc: lọc đỏ làm trời xanh tối nhất, lọc xanh làm lá sáng lên.
- Thẻ đang gập vẫn báo có bao nhiêu thanh đã chỉnh ("Công cụ · đã chỉnh 2"); nút ↺ trên tiêu đề đặt lại cả thẻ.
- Trong bản xem trước, giữ nút **Trước** (hoặc phím **\\**) để xem ảnh khi chưa có look.
- Dấu ngày, khung thông tin và viền không bao giờ bị nhuộm màu, và ảnh chưa chỉnh gì in ra y như trước.
- **Trong chế độ Ghép ảnh**, mở bản xem trước của tấm ghép rồi nhấp vào một ảnh trên tấm để chỉnh màu ảnh đó; hai thẻ chỉ hiện khi có ảnh đang được chọn. Nhấp ra ngoài ảnh, hoặc nhấn **Esc**, để bỏ chọn.

<!-- chapter: shortcuts -->

## 22. Phím tắt

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
| Chế độ Đơn giản | **↑** / **↓** | Giữ (giỏ in) / bỏ (để riêng), sang tấm sau |
| Chế độ Đơn giản | **← →** | Ảnh trước / sau, không quyết định |
| Chế độ Ghép ảnh | **← →**, **↑ ↓** | Chọn ảnh, tấm ở trên hoặc dưới |
| Chế độ Ghép ảnh | **⌥/Alt ← →** | Dời ảnh đang chọn đi một chỗ |
| Chế độ Ghép ảnh | **Enter**, **Space**, nhấp đúp | Mở bản xem trước của tấm |
| Chế độ Ghép ảnh | **R** | Xoay tấm |
| Chế độ Ghép ảnh, xem trước tấm | **⇧R** | Xoay ảnh đang chọn trong nửa của nó |
| Xem trước tấm | **← →** / **1**, **2** | Tấm khác / chọn ảnh |
| Xem trước tấm | **C** / **R** | Cắt ảnh đang chọn / xoay tấm |
| Xem trước tấm | **↑** / **↓** | Cho ảnh đang chọn vào giỏ in / để riêng |
| Xem trước | **C** | Mở màn cắt ảnh (chương 6) |
| Xem trước | Giữ **\\** | Trước / Sau: ảnh khi chưa có look (§21) |
| Xem trước | **Cmd/Ctrl + / −**, vuốt hai ngón, chụm trên trackpad, các nút phóng | Phóng to / thu nhỏ (§7) |
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
| Chế độ Chọn (Pro) | giữ **Space** | Xem nhanh; **← →** tấm sau, **Enter** tích, thả ra để đóng |
| Chế độ Chọn | **Esc** | Rời chế độ Chọn |
| Mọi nơi | **Cmd/Ctrl+Z**, **Shift+Cmd/Ctrl+Z** hoặc **Cmd/Ctrl+Y** | Hoàn tác, làm lại (chương 13) |
| Mọi nơi | **Cmd/Ctrl+1**, **2**, **3** | Đơn giản, Ghép ảnh, Pro (chương 1) |
| Mọi nơi | **Cmd/Ctrl+N**, **Cmd/Ctrl+O**, **Cmd/Ctrl+S**, **Shift+Cmd/Ctrl+S** | Dự án mới, mở, lưu, lưu thành (chương 13) |

Ô nhập hay thanh trượt đang được chọn giữ các phím mũi tên cho riêng nó: nhấp vào ảnh để trả chúng về việc duyệt ảnh.
