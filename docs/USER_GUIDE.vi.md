# Hướng dẫn sử dụng — Fotollection

> **Status:** Living · **Last reviewed:** 2026-10-30
>
> Bản tiếng Việt của [USER_GUIDE.md](USER_GUIDE.md), cũng là trang hướng dẫn trên website ([fotollection.org/vi/manual](https://fotollection.org/vi/manual/), [ADR-0031](adr/0031-manual-on-the-website-from-the-user-guide.md)). Giữ đúng các chương, mốc chương (`<!-- chapter: id -->`, ở dòng trước tiêu đề chương) và ảnh chụp màn hình (`<!-- shot: tên | mô tả -->`) như bản tiếng Anh, dùng đúng nhãn của app trong `ui/src/locales/vi.json`; `scripts/check-docs.sh` kiểm tra hai bản khớp nhau.

<!-- chapter: choose -->

## 1. Chọn ảnh để in

Một chuyến đi cho ra hàng trăm tấm ảnh, nhưng chỉ vài tấm được đem đi in. Fotollection giữ lựa chọn đó trong hai **khu vực**, bên cạnh gallery ([FR-050](PRD.md#fr-050--print-basket)):

| Khu vực | Chứa gì | Ở đâu |
|---------|---------|-------|
| **Giỏ in** | Những tấm bạn sẽ in | Thanh bên, dưới *Tất cả ảnh*, kèm số lượng |
| **Để riêng** | Những tấm bạn tạm cất đi. Chúng rời khỏi gallery nhưng **không bao giờ bị xoá hay di chuyển trên ổ đĩa** | Thanh bên, dưới *Giỏ in*, kèm số lượng |

Mỗi tấm ảnh chỉ ở một nơi: trong gallery, trong giỏ in, hoặc để riêng. Ảnh trong giỏ vẫn hiện trong gallery, với một biểu tượng giỏ nhỏ ở góc trên bên phải; ảnh để riêng không hiện trong *Tất cả ảnh* hay trong các thư mục cho đến khi bạn đưa chúng trở lại. Sao và cờ chọn/loại là chuyện riêng: ảnh được chọn không nhất thiết phải in, và giỏ in không phải là một cờ.

<!-- shot: grid | Thư viện ảnh, đã chấm sao và chọn ảnh -->

Cả hai khu vực được lưu trong dự án (`.fotollection`) và trong phiên làm việc, nên vẫn còn sau khi bạn tắt rồi mở lại app. Mọi thao tác đều hoàn tác được bằng **Cmd/Ctrl+Z**.

### 1.1 Cách A: quyết từng tấm trong bản xem trước

1. Mở một tấm: nhấp đúp vào nó, hoặc chọn nó rồi nhấn **Enter** (hoặc **Space**).
2. Xem ảnh với các thiết lập in đã áp.
3. Nhấn **↑** để cho vào **giỏ in**: tấm tiếp theo mở ra ngay.
4. Nhấn **↓** để **để riêng**: ảnh rời khỏi gallery, tấm tiếp theo thế chỗ.
5. Nhấn **→** để đi tiếp mà chưa quyết, **←** để quay lại.
6. Lặp lại đến hết chuyến đi. **Esc** đóng bản xem trước.

<!-- shot: swipe | Một tấm ảnh đã vào giỏ in, cùng các phím để lướt tiếp -->

Nút **Giỏ in** trên ảnh (cạnh *Cắt ảnh*, các nút xoay và *Xuất ảnh*) làm giống phím ↑, và hiện *Đã trong giỏ* khi ảnh đã nằm trong giỏ; nhấp lần nữa để lấy ảnh ra. Ảnh đã để riêng thì hiện **Trả về gallery**.

Khi bạn mở bản xem trước từ *Giỏ in* hoặc *Để riêng*, ↑ và ↓ chuyển ảnh sang khu vực còn lại; ảnh ở nguyên chỗ cũ thì chuyển sang tấm tiếp theo.

### 1.2 Cách B: tích ảnh ngay trong gallery (chế độ Chọn)

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

Trong chế độ Chọn, nhấp đúp không mở bản xem trước, và **phím Space làm việc khác** (§1.3).

### 1.3 Xem nhanh: xem mà không cần mở (chỉ trong chế độ Chọn)

Trong chế độ Chọn, **giữ Space** trên một tấm để xem nó thật lớn, đúng như khi in, với khổ in, dấu ngày và khung thông tin của gallery ([FR-059](PRD.md#fr-059--quick-preview)). **Thả Space** để đóng, giống Quick Look trên Mac.

- Nó hiện tấm ảnh dưới con trỏ chuột; nếu con trỏ không nằm trên ảnh nào, nó hiện tấm bạn tích gần nhất.
- Ảnh nhỏ hiện ngay, rồi bản dựng đầy đủ thay vào một lát sau.
- Trong lúc giữ Space: **← →** xem các tấm bên cạnh, **Enter** tích hoặc bỏ tích tấm đang hiện, **Esc** đóng.
- Ở đây không chỉnh sửa được gì. Muốn đổi khung cắt, dấu ngày hay khung thông tin của một tấm, hãy mở bản xem trước thường (nhấp đúp, hoặc **Enter** khi không ở chế độ Chọn).

Ngoài chế độ Chọn, Space vẫn mở bản xem trước thường.

### 1.4 Xem lại và xuất ảnh

- Nhấp **Giỏ in** hoặc **Để riêng** ở thanh bên để chỉ xem những ảnh đó. Đổi ý bằng ↑ ↓ trong bản xem trước, hoặc bằng chế độ Chọn và **Trả về gallery**.
- Nhấp **Tất cả ảnh**, hoặc một thư mục, để về gallery.
- **Xuất ảnh theo đúng chỗ bạn đang xem.** Từ *Giỏ in*, *Xuất ảnh* ghi đúng những tấm trong giỏ, với ước tính dung lượng ngay trên nút ([FR-053](PRD.md#fr-053--export-for-social-media)). Từ *Tất cả ảnh* hay một thư mục, ảnh để riêng không bao giờ được xuất.

<!-- shot: estimate | Ô ước tính dung lượng ngay trên nút xuất ảnh, khi xuất cho Facebook -->

<!-- chapter: zoom -->

## 2. Xem kỹ: phóng to

Trong bản xem trước, **các nút phóng to** ở góc dưới bên phải ảnh hiện mức phóng giữa **−** và **+**; nút cuối cùng đưa ảnh về vừa khung ([FR-057](PRD.md#fr-057--zoom-in-the-preview)). Bàn phím và trackpad cũng làm được như vậy:

- **Chụm hai ngón** trên trackpad, hoặc **Cmd/Ctrl +** và **Cmd/Ctrl −**, phóng quanh con trỏ, tới 800 %.
- **Nhấp đúp** vào ảnh để chuyển giữa vừa khung và 200 %.
- Khi đã phóng to, **kéo** ảnh, hoặc cuộn bằng hai ngón, để di chuyển trong ảnh.
- **Cmd/Ctrl 0** hoặc nút vừa khung đưa về toàn bộ ảnh.

Bản xem trước ban đầu được hiện ở cỡ nhẹ để máy nào cũng vẽ nhanh. Khi bạn phóng vượt quá cỡ đó, một vòng xoay nhỏ hiện cạnh mức phóng, và một lát sau khi bạn ngừng di chuyển, phần trên màn hình được thay bằng điểm ảnh **độ phân giải đầy đủ**, đúng như trong tệp sẽ xuất. Chỉ phần đang thấy được tải, nên kể cả ảnh 32 MP vẫn nhẹ.

<!-- chapter: open-with -->

## 3. Mở ảnh từ Finder hoặc Explorer

Fotollection có thể là app mở ảnh khi bạn nhấp đúp ([FR-054](PRD.md#fr-054--open-photos-with-fotollection)). Nó không bao giờ tự đặt mình làm mặc định: bạn chọn.

- **Một lần:** nhấp phải vào ảnh → **Mở bằng** → **Fotollection** (macOS), hoặc **Mở bằng** → **Fotollection** (Windows).
- **Luôn luôn:** trên Mac, chọn ảnh, nhấn Cmd+I, chọn Fotollection trong *Mở bằng* rồi nhấp *Thay đổi tất cả*. Trên Windows, chọn **Mở bằng** → **Chọn ứng dụng khác**, chọn Fotollection và tích *Luôn dùng ứng dụng này*.

Ảnh mở ngay trong bản xem trước. Thư mục của nó thành một tab ở thanh bên, chỉ có ảnh của đúng thư mục đó (không gồm thư mục con), nên **← →** đi qua cả thư mục. Mọi thứ hoạt động như ở bất kỳ tab nào: sao, giỏ in, khung cắt, xuất ảnh. Xong thì nhấp **×** trên tab; ảnh vẫn nằm trên ổ đĩa của bạn. Mở một ảnh khác khi Fotollection đang chạy sẽ đưa cửa sổ lên trước với ảnh đó.

<!-- chapter: raw -->

## 4. Ảnh RAW

Fotollection mở ảnh RAW của hầu hết máy ảnh như chúng vốn có, không cần đổi sang DNG: RW2 (Panasonic), CR2 và CR3 (Canon), NEF (Nikon), ARW (Sony), RAF (Fujifilm), ORF (Olympus, OM System) và DNG ([FR-016](PRD.md#fr-016--raw-input)).

- Mở thư mục như thường. Ảnh nhỏ và gợi ý ảnh đẹp nhất hiện nhanh như với JPEG, vì chúng dùng bản xem trước nhỏ mà máy ảnh nào cũng lưu sẵn trong tệp RAW.
- Bản xem trước để in sẽ xử lý toàn bộ ảnh ở lần mở đầu, mất khoảng một giây; sau đó đổi thiết lập là thấy ngay. Khi xuất, mỗi ảnh RAW được xử lý theo cùng cách.
- Màu sắc và độ nét là cách xử lý trung tính của hệ điều hành, không phải của một trình sửa RAW: Fotollection chuẩn bị ảnh để in, không thay thế Lightroom.
- **Trên Windows**, hãy cài *Raw Image Extension* miễn phí của Microsoft từ Microsoft Store trước. Không có nó, tệp RAW bị bỏ qua và dòng gợi ý sẽ nói rõ. Một trình đọc RAW có sẵn trong Fotollection, không cần extension, đang được làm ([STATUS.md](STATUS.md)).

<!-- chapter: updates -->

## 5. Cập nhật lên phiên bản mới

Fotollection không bao giờ tự lên mạng ([FR-021](PRD.md#fr-021--manual-updates), [FR-061](PRD.md#fr-061--check-for-updates)).

1. Nhấp vào số phiên bản ở cuối bảng cài đặt, rồi **Kiểm tra cập nhật**. Đây là lúc duy nhất app lên mạng.
2. Nếu có bản mới hơn, nhấp **Tải bộ cài**: trình duyệt sẽ tải đúng bộ cài cho máy của bạn.
3. Cài đè lên bản đang dùng. Cài đặt, hồ sơ máy ảnh và dự án của bạn vẫn giữ nguyên.
   - **Mac:** mở tệp đĩa và kéo Fotollection vào Applications, rồi chọn **Thay thế**.
   - **Windows:** chạy tệp `-setup.exe`; giữ lựa chọn đầu tiên, *Gỡ cài đặt trước khi cài*. Nếu bạn đã cài bằng `.msi`, hãy cập nhật bằng `.msi`: trộn hai loại sẽ để lại hai bản.

Cùng ô đó hiện **cache**: ảnh nhỏ và kết quả phân tích ảnh đẹp nhất mà Fotollection giữ để mở thư mục nhanh, kèm dung lượng so với giới hạn 2 GB. Vượt giới hạn, các mục cũ nhất tự bị xoá. **Xoá cache** làm trống nó bất kỳ lúc nào; không mất gì của bạn, và ảnh nhỏ sẽ được tạo lại khi bạn duyệt.

<!-- chapter: clocks -->

## 6. Đồng hồ máy ảnh

Máy ảnh đặt sai giờ, hoặc vẫn để múi giờ ở nhà, sẽ xếp ảnh sai thứ tự so với ảnh điện thoại, và đóng dấu sai ngày ([FR-030](PRD.md#fr-030--camera-clock-offset)).

1. **Theo một ảnh (dễ nhất):** mở một ảnh của máy đó, và trong **Ngày chụp** nhấp **Chỉnh giờ máy này theo một ảnh…**. Chọn ảnh điện thoại (hoặc máy khác) chụp cùng lúc, rồi **Chỉnh đồng hồ**.
2. **Bằng tay:** trong bảng cài đặt, **Đồng hồ máy ảnh** liệt kê mọi máy ảnh trong thư viện. Gõ lượng cần cộng thêm, như `+1:02:00` hay `−0:45`, rồi nhấn Enter. Nút mũi tên đặt lại.

Mọi ảnh của máy đó dịch chuyển cùng lúc, kể cả ảnh thêm vào sau: thứ tự trong lưới, nhóm ảnh đẹp nhất, thư mục theo ngày, dấu ngày và ngày của bản xuất. Bản xem trước vẫn hiện giờ mà đồng hồ máy ghi, và tệp ảnh không bao giờ bị sửa. Hai máy cùng đời giữ đồng hồ riêng nếu máy ảnh có ghi số seri. Phần chỉnh được lưu trong dự án.

<!-- chapter: dates-places -->

## 7. Ngày và nơi chụp

**Đặt bằng tay** ([FR-032](PRD.md#fr-032--manual-date-and-place)), cho ảnh in đã scan, máy ảnh không có đồng hồ, hay ảnh không có GPS:

1. **Một tấm:** trong bản xem trước, nhấp **Đặt ngày…** trong *Ngày chụp* hoặc **Đặt nơi chụp…** trong *Vị trí*.
2. **Nhiều tấm:** trong chế độ Chọn, tích chúng rồi nhấp **Ngày và nơi…** trên thanh chọn.
3. Gõ ngày giờ, và gõ tên thị trấn hay thành phố để chọn (dấu và khoảng trắng không quan trọng: `ha noi` tìm ra Hà Nội). Chỉ điền phần muốn đổi, rồi **Áp dụng**. Các nút mũi tên đưa về giờ của máy ảnh và GPS của chính ảnh.

<!-- shot: manual-place-picker | Đặt nơi chụp cho một tấm: gõ ba chữ là tìm ra thành phố -->

Ảnh dịch chuyển trong lưới, và dấu ngày, khung thông tin, thư mục theo ngày cùng ngày của bản xuất đều đổi theo. Mọi thứ đều offline, mỗi thay đổi là một bước Hoàn tác, và tệp ảnh không bao giờ bị sửa.

**Mượn nơi chụp từ điện thoại** ([FR-031](PRD.md#fr-031--borrow-gps-from-a-phone-or-gpx-track)): máy ảnh không có GPS (như Lumix GX9) có thể lấy nơi chụp của ảnh điện thoại chụp cùng cảnh, cách vài phút. Hãy chỉnh đồng hồ máy ảnh trước nếu nó lệch (chương 6). Trong thẻ **Địa điểm** của bảng cài đặt, nhấp **Tìm vị trí**, chọn khoảng cách thời gian (5, 10 hoặc 30 phút) rồi **Tìm vị trí**. Cặp chắc chắn được điền ngay; các cặp còn lại hiện cạnh ảnh điện thoại, đã tích sẵn, bạn bỏ tích cặp sai rồi xác nhận. Cặp chắc chắn cũng được điền sau mỗi lần phân tích ảnh đẹp nhất. Nơi đã mượn hiện một ghim có dấu tích trên ảnh nhỏ và chữ *Mượn* trong bản xem trước; nó không bao giờ thay nơi bạn đã đặt hay GPS của chính ảnh, và Hoàn tác lấy lại được.

**Đổi tên nơi chụp** ([FR-005](PRD.md#fr-005--location-stamp-offline)): thẻ **Địa điểm** trong bảng cài đặt liệt kê mọi nơi có trên ảnh của bạn. Gõ tên bạn muốn in, như `Prague` thay cho `Praha 1` hay `Nhà bà ngoại`, và mọi dấu ngày, khung thông tin có nơi đó đổi theo; nút mũi tên đưa về tên gốc.

<!-- shot: manual-places | Thẻ Địa điểm trong cài đặt: mọi nơi có trên ảnh, mỗi nơi một tên để in -->

### 7.1 Sao từ Lightroom và các app khác

Khi thư mục bạn mở đã được lọc trong Lightroom, Bridge, Capture One hay darktable, số sao và cờ loại chúng lưu trong tệp XMP đi kèm (`IMG_0416.xmp` hay `P1000123.RW2.xmp` cạnh ảnh) được đọc cùng ảnh ([FR-058](PRD.md#fr-058--lightroom-compatibility-through-xmp)). Thanh bên cho biết bao nhiêu tệp. Sao và cờ của bạn trong dự án Fotollection thắng tệp XMP, và tệp XMP không bao giờ bị sửa. Lightroom Classic chỉ ghi tệp XMP cho ảnh RAW, và chỉ khi bật *Automatically write changes into XMP* (hoặc sau *Metadata → Save Metadata to File*).

<!-- chapter: camera -->

## 8. Chụp với máy ảnh cắm vào máy tính

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

<!-- chapter: shortcuts -->

## 9. Phím tắt

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
| Xem trước | **C** | Mở màn cắt ảnh |
| Xem trước | **Cmd/Ctrl + / −**, chụm trên trackpad, các nút phóng | Phóng to / thu nhỏ (§2) |
| Xem trước | **Cmd/Ctrl 0**, nút vừa khung | Về vừa khung |
| Xem trước | nhấp đúp vào ảnh | Chuyển giữa vừa khung và 200 % |
| Xem trước | kéo ảnh (khi chưa phóng) | Chọn phần được in (*Cắt cho đầy*) |
| Xem trước | **Esc** | Rời màn cắt ảnh, rồi đóng |
| Màn cắt ảnh | **← →** | Cân ảnh từng độ một |
| Màn hình chụp | **Space** | Chụp một tấm |
| Màn hình chụp | **Esc** | Đóng và hiện tab của phiên chụp |
| Chế độ Chọn | nhấp, **Shift**+nhấp | Tích một tấm, tích cả dải |
| Chế độ Chọn | **Cmd/Ctrl+A** | Tích mọi ảnh đang hiện |
| Chế độ Chọn | **Enter** | Tích ảnh đang chọn |
| Chế độ Chọn | giữ **Space** | Xem nhanh; **← →** tấm sau, **Enter** tích, thả ra để đóng |
| Chế độ Chọn | **Esc** | Rời chế độ Chọn |
| Mọi nơi | **Cmd/Ctrl+Z**, **Shift+Cmd/Ctrl+Z** | Hoàn tác, làm lại |
| Mọi nơi | **Cmd/Ctrl+S**, **Shift+Cmd/Ctrl+S**, **Cmd/Ctrl+O** | Lưu, Lưu thành, Mở dự án |

Ô nhập hay thanh trượt đang được chọn giữ các phím mũi tên cho riêng nó: nhấp vào ảnh để trả chúng về việc duyệt ảnh.
