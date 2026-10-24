<!-- Bản tiếng Việt của README.md. Sửa README.md thì sửa file này trong cùng commit (docs/STYLE_RULE.md §6). -->
<div align="center">

<img src="docs/images/logo.png" alt="Logo Fotollection" width="140" />

# Fotollection

by [@hoangphi01](https://github.com/hoangphi01)

[EN](README.md) · **Tiếng Việt**

**Chuẩn bị ảnh để in: nhanh, riêng tư, chạy offline trên macOS và Windows.**

In ngày và nơi chụp lên ảnh, crop đúng khổ in thật, rồi gửi đi tiệm in, mà không có tấm ảnh nào rời khỏi máy của bạn.

[![CI](https://github.com/hoangphi01/fotollection/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/hoangphi01/fotollection/actions/workflows/ci.yml)
[![Docs](https://github.com/hoangphi01/fotollection/actions/workflows/docs.yml/badge.svg?branch=main)](https://github.com/hoangphi01/fotollection/actions/workflows/docs.yml)
[![License: GPL-3.0-only](https://img.shields.io/badge/license-GPL--3.0--only-blue.svg)](LICENSE)
[![Status: pre-alpha](https://img.shields.io/badge/status-pre--alpha-orange.svg)](docs/STATUS.md)
[![Latest release](https://img.shields.io/github/v/release/hoangphi01/fotollection?include_prereleases&sort=semver&label=release)](https://github.com/hoangphi01/fotollection/releases)

[![Platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Windows-lightgrey.svg)](docs/RELEASE.md)
[![Rust 1.99](https://img.shields.io/badge/rust-1.99-b7410e.svg?logo=rust)](rust-toolchain.toml)
[![Tauri 2](https://img.shields.io/badge/Tauri-2-24c8db.svg?logo=tauri&logoColor=white)](https://v2.tauri.app/)
[![React 19](https://img.shields.io/badge/React-19-61dafb.svg?logo=react&logoColor=black)](https://react.dev/)
[![shadcn/ui](https://img.shields.io/badge/ui-shadcn%2Fui-000000.svg?logo=shadcnui&logoColor=white)](https://ui.shadcn.com/)
[![Offline · zero telemetry](https://img.shields.io/badge/privacy-offline%20%C2%B7%20zero%20telemetry-2ea44f.svg)](docs/SECURITY.md#5-privacy)

[![Stars](https://img.shields.io/github/stars/hoangphi01/fotollection?style=social)](https://github.com/hoangphi01/fotollection/stargazers)
[![Last commit](https://img.shields.io/github/last-commit/hoangphi01/fotollection.svg)](https://github.com/hoangphi01/fotollection/commits/main)
[![Issues](https://img.shields.io/github/issues/hoangphi01/fotollection.svg)](https://github.com/hoangphi01/fotollection/issues)
[![Repo size](https://img.shields.io/github/repo-size/hoangphi01/fotollection.svg)](https://github.com/hoangphi01/fotollection)

</div>

---

> [!NOTE]
> **Fotollection đang trong giai đoạn phát triển sớm.** App đã chạy trọn vẹn với **JPEG, PNG và HEIC**: nhập ảnh → lọc ảnh bằng sao và cờ → xem trước với crop, dấu ngày và khung thông tin → export. Dự án được lưu thành một file `.fotollection`, mở trên máy khác vẫn y như cũ. Tên địa điểm offline và gợi ý ảnh tốt nhất đang được làm. Theo dõi tiến độ ở [docs/STATUS.md](docs/STATUS.md) và [docs/ROADMAP.md](docs/ROADMAP.md) (tiếng Anh).

<p align="center">
  <img src="docs/images/screenshot-light.png" alt="Cửa sổ chính của Fotollection (giao diện sáng)" width="49%" />
  <img src="docs/images/screenshot-dark.png" alt="Cửa sổ chính của Fotollection (giao diện tối)" width="49%" />
</p>

## Vì sao có Fotollection?

Bạn chụp 800 tấm trong một chuyến đi và muốn in ra hai mươi tấm. Hiện nay việc đó rất mất công:

- **Ảnh in không ghi khi nào, ở đâu.** Vài năm sau chẳng ai còn nhớ.
- **Tiệm in tự crop ảnh** về 2:3, 3:4 hay 1:1, và có thể cắt mất đầu người.
- **Công cụ online bắt tải ảnh lên.** Dữ liệu GPS để lộ nơi bạn sống.

Fotollection làm tất cả những việc đó **ngay trên máy bạn**, với tốc độ của Rust native.

## Tính năng

| | Tính năng | Giai đoạn |
|---|---------|-------|
| 📅 | **Dấu ngày và nơi chụp** lấy từ EXIF, dùng được font OTF/TTF của riêng bạn. Địa điểm tra offline, không gọi API nào | MVP |
| ✂️ | **Crop hàng loạt** theo 8 khổ in châu Âu (9×13 … 20×30, vuông, dải), khổ của riêng bạn (ví dụ 4 × 6 in) hoặc giữ **tỉ lệ gốc** của ảnh, chỉnh được khung crop từng ảnh | MVP |
| ⭐ | **Lọc ảnh**: chấm sao, cờ chọn/loại và bộ lọc; export đúng tập ảnh đang lọc | MVP |
| ✨ | **Gợi ý ảnh đẹp nhất**: gom các ảnh chụp liên tiếp và gần giống nhau, gợi ý ảnh đẹp nhất mỗi nhóm (nét nhất, mắt mở, không rung) kèm lý do. Chạy hoàn toàn offline | MVP |
| 🏷️ | **Khung thông tin "Shot on …"** với máy ảnh, ống kính và thông số phơi sáng, kiểu LUMIX Lab, đúng tỉ lệ khổ in | MVP |
| 🖼️ | **Giữ nguyên chất lượng**: độ phân giải đầy đủ, giữ ICC profile, crop không mất chất lượng khi có thể | MVP |
| 📱 | Đầu vào **JPEG, PNG và HEIC** (iPhone) | MVP |
| 🔒 | **Xóa GPS khi export** theo mặc định. Không tài khoản, không cloud, không telemetry | MVP |
| 🌐 | Giao diện **tiếng Anh và tiếng Việt** | MVP |
| 📦 | **File dự án như `.docx`**: lưu lại, mở trên máy khác với đúng font, logo và thiết lập, chỉ lại thư mục ảnh đã bị chuyển | MVP |
| 🧳 | **Gom ảnh theo chuyến đi**, crop thông minh theo khuôn mặt, phát hiện ảnh trùng | 2 |
| 🎞️ | Đầu vào RAW, xem trước photobook, mã QR ở mặt sau ảnh in | 3 |

Đặc tả đầy đủ: [docs/PRD.md](docs/PRD.md) (tiếng Anh).

## Hiệu năng

Fotollection dành cho người chụp ảnh nghiêm túc và nhiếp ảnh gia, nên tốc độ là yêu cầu bắt buộc, không phải thứ có thì tốt. **Mọi con số dưới đây đều đo thật, không phải ước tính**, trên Apple M3 (8 nhân, 16 GB) với ảnh JPEG 16 MP thật, ngày 23/10/2026. Mỗi biểu đồ có một mốc tham chiếu để so: ngân sách hiệu năng, trần bộ nhớ, hoặc trình duyệt. Kết quả đầy đủ: [docs/RESULT.md](docs/RESULT.md) (tiếng Anh).

### So với ngân sách

![Mở thư mục: 0,12 giây cho 1.000 ảnh, ngân sách dưới 1 giây. Preview: 18 ms, ngân sách dưới 50 ms. Export: 18,0 ảnh/s, ngân sách tối thiểu 8. RAM đỉnh: 1,45 GB, ngân sách tối đa 2 GB.](docs/images/bench-budgets-vi.svg)

| Việc | Ngân sách | Đo được | |
|------|--------|----------|---|
| Mở thư mục (import + EXIF + thumbnail đầu tiên) | < 1 giây cho 1.000 ảnh | **0,12 giây** cho 1.000 ảnh (5.035 ảnh trong 0,62 giây) | ✓ |
| Cập nhật preview khi đổi thiết lập | < 50 ms (p95) | **18 ms** p95, 17 ms p50 | ✓ |
| Tốc độ export | ≥ 8 ảnh/s với ảnh 24 MP | **18,0 ảnh/s** với ảnh 16 MP | ✓ |
| RAM đỉnh | ≤ 2 GB với batch bất kỳ | **1,45 GB** với 5.035 ảnh | ✓ |
| Phân tích ảnh đẹp nhất, mỗi ảnh (24/10/2026) | ≤ 20 ms, ≤ 40 ms khi có mặt người | **17 ms**, 28 ms khi có mặt người | ✓ |

### Batch lớn

Bộ nhớ không được tăng theo số ảnh. Batch lớn gấp 53 lần chỉ tốn thêm 0,1 GB ở mức đỉnh:

![RAM đỉnh: 1,35 GB với 95 ảnh và 1,45 GB với 5.035 ảnh, so với trần 2 GB.](docs/images/bench-memory-vi.svg)

| Số ảnh | RAM đỉnh | Export | Thư viện trong RAM |
|-------:|------------:|-------:|------------------:|
| 95 | 1,35 GB | 18,0 ảnh/s | — |
| 5.035 | 1,45 GB | 15,4 ảnh/s | 13 MB |

Lần export 5.035 ảnh chạy hơn 5 phút, và M3 tự giảm xung khi tải nặng kéo dài, nên tốc độ thấp hơn.

### So với trình duyệt

Phương án thay thế dễ thấy nhất là một web app làm đúng việc đó trong trình duyệt (`createImageBitmap` + `OffscreenCanvas`, mỗi nhân CPU một Web Worker). Cả hai chạy trên cùng 95 ảnh, cùng một máy:

![Export cả lô: Fotollection 18,0 ảnh/s, trình duyệt 17,9. Preview đã cache: Fotollection 17,4 ms, trình duyệt 18,5 ms. RAM đỉnh: Fotollection 1,35 GB, trình duyệt 1,78 GB.](docs/images/bench-vi.svg)

| | Fotollection (Rust) | Trình duyệt (Chromium 156) | |
|---|---:|---:|---|
| Export cả lô | 18,0 ảnh/s | 17,9 ảnh/s | Gần như bằng nhau |
| Preview đã cache, 1600 px (p50) | 17,4 ms | 18,5 ms | Fotollection nhanh hơn |
| RAM đỉnh | 1,35 GB | 1,78 GB | Fotollection thấp hơn 24 % |

Về tốc độ export thô, trình duyệt nhanh ngang ngửa vì Chromium dùng libjpeg-turbo có SIMD. Giờ Fotollection đã preview nhanh hơn. Tuy vậy, tốc độ không phải lý do chính để làm app native:

- Trình duyệt chỉ mã hóa JPEG được ở chế độ màu 4:2:0, tức một nửa độ phân giải màu. Fotollection export 4:4:4, gấp đôi dữ liệu màu, trong cùng thời gian.
- Fotollection dùng ít hơn 24 % bộ nhớ, và việc export có giới hạn bộ nhớ, như bài thử batch lớn cho thấy. Một tab trình duyệt thì có trần cứng.
- Fotollection ghi file ra đĩa với rename nguyên tử và không ghi EXIF, nên GPS không bao giờ rời khỏi máy bạn. Trình duyệt chỉ tạo blob trong bộ nhớ và không ghi file nào.
- ICC profile, RAW 16-bit ở Phase 3, và HEIC nằm ngoài khả năng của trình duyệt.

### Thời gian preview đi đâu

![Phân tích preview: crop và resize 9,7 ms, dấu ngày dưới 0,1 ms, mã hóa JPEG 7,0 ms, tổng 16,7 ms, so với Chromium 18,5 ms và ngân sách 50 ms.](docs/images/bench-preview-vi.svg)

| Bước | Thời gian | Tỉ lệ |
|------|-----:|------:|
| Crop + resize | 9,7 ms | 58 % |
| Dấu ngày | < 0,1 ms | ≈ 0 % |
| Mã hóa JPEG | 7,0 ms | 42 % |
| **Tổng** | **16,7 ms** | ngân sách 50 ms · Chromium 18,5 ms |

Trước đây bộ mã hóa là nút thắt: 16,2 ms, chiếm 62 % của 26 ms. Preview chỉ để xem trên màn hình, nên giờ được mã hóa ở chế độ màu 4:2:0 với bảng Huffman thường. Phóng to 3× vẫn nhìn giống hệt. Ảnh export vẫn giữ 4:4:4 và bảng Huffman tối ưu. Chỉ một thay đổi đó đã đưa preview từ 26 ms xuống 17 ms.

### Gợi ý ảnh đẹp nhất chính xác đến đâu?

Với tính năng gợi ý nên giữ ảnh nào, nhanh thôi chưa đủ, nên gợi ý ảnh đẹp nhất còn được chấm điểm độ chính xác. 95 ảnh thật (người, đồ ăn, phố, cảnh đêm, hoàng hôn) được gán nhãn bằng tay: 22 nhóm ảnh chụp cùng một khoảnh khắc, và ở 6 nhóm có ghi rõ ảnh nên giữ. `eval_bestshot` so các nhóm và lựa chọn của app với các nhãn đó.

![Số nhóm tìm được trên 22 nhóm gán nhãn: dHash 3, thêm chữ ký bố cục và màu 13, thêm tính zoom từ tiêu cự 15, thêm so độ nét công bằng và mặt bị che 15. Lựa chọn chấp nhận được: 2/2, 3/4, 5/6, 6/6. Không lần nào gom sai.](docs/images/bench-bestshot-vi.svg)

| Bước | Nhóm tìm được (trên 22) | Cặp tìm được | Nhóm sai | Lựa chọn chấp nhận được |
|------|-----:|-----:|-----:|-----:|
| dHash (trước) | 3 | 9 % | 0 | 2/2 |
| + chữ ký bố cục và màu | 13 | 67 % | 0 | 3/4 |
| + tính zoom từ tiêu cự | 15 | 73 % | 0 | 5/6 |
| + độ nét công bằng, mặt bị che | **15** | **73 %** | **0** | **6/6** |

- **dHash** (mã băm 64 bit theo bậc độ sáng) bị lật bit lung tung ở vùng phẳng như bầu trời. Hai ảnh khách sạn gần như y hệt bị coi là khác nhau chẳng kém hai cảnh không liên quan. Nó được thay bằng chữ ký bố cục 32 × 24, so bằng tương quan có cho phép xê dịch và zoom nhẹ, cộng với histogram màu để tách các cảnh khác nhau nhưng bố cục giống nhau.
- **Chụp lại gần hơn là chuyện thường**, ví dụ zoom 5 → 9,9 mm khi chụp nhà thờ ban đêm. Zoom không còn tách nhóm nữa: tiêu cự trong EXIF cho biết ảnh sau gần hơn bao nhiêu lần.
- **So sánh công bằng.** Trước đây ảnh có khuôn mặt quay đi bị so độ nét cả ảnh với độ nét vùng mắt của ảnh kia. Giờ mọi ảnh trong nhóm được đo cùng một cách, và ảnh có khuôn mặt bị quay đi hoặc che khuất bị trừ điểm.
- **Vẫn còn sót:** 7 nhóm có tư thế hoặc khung hình đổi nhiều, ví dụ chụp lại pizza từ phía bên kia hay selfie bị lộn ngược. Các trường hợp đó cần model hiểu nội dung ảnh, là bước tiếp theo. 2 nhóm "thừa" app tạo ra là các cặp đã được gán nhãn mơ hồ, không phải lỗi.

Phân tích vẫn nằm trong ngân sách: 17,4 ms mỗi ảnh, 27,6 ms khi có mặt người ([NFR-014](docs/PRD.md#nfr-014--best-shot-analysis-speed)). Nhãn chỉ chứa tên file vì ảnh là ảnh riêng tư: hãy gán nhãn thư mục của bạn theo cùng định dạng (`bench/bestshot/truth-103_09.json`) để chấm điểm.

Đo lại tất cả bằng `bench/run.sh <thư-mục-ảnh>` và `bench/batch.sh <thư-mục-ảnh>`; chấm điểm gợi ý ảnh đẹp nhất bằng `cargo run --release -p fotollection-core --example eval_bestshot -- <thư-mục-ảnh> <nhãn.json>`. Phương pháp đo ở [docs/OPTIMIZATION.md §2](docs/OPTIMIZATION.md#2-why-native-rust-instead-of-a-web-app), và các ngân sách mà mọi thay đổi phải tuân theo ở [§3](docs/OPTIMIZATION.md#3-budgets) (tiếng Anh).

## Tải về

Hiện chưa có bản phát hành nào. Khi có, chúng sẽ nằm ở trang [Releases](https://github.com/hoangphi01/fotollection/releases).

> [!IMPORTANT]
> Các bản build hiện **chưa được ký số** ([ADR-0009](docs/adr/0009-unsigned-distribution.md)). Lần mở đầu tiên:
>
> - **macOS:** System Settings → Privacy & Security → **Open Anyway**
> - **Windows:** SmartScreen → **More info** → **Run anyway**
>
> Kiểm tra từng file tải về với `SHA256SUMS.txt` trên trang phát hành.

## Build từ mã nguồn

```sh
git clone https://github.com/hoangphi01/fotollection.git
cd fotollection
scripts/bootstrap.sh --install     # macOS / Linux (Windows: scripts\bootstrap.ps1 -Install)
npm run dev                        # chạy app
```

Mọi phiên bản công cụ đều được ghim: `rust-toolchain.toml`, `.node-version`, `Cargo.lock` và `package-lock.json`. Nhờ vậy một bản clone mới build ra giống hệt nhau trên mọi máy. Xem [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) (tiếng Anh) để biết yêu cầu, các lệnh và cách dọn dung lượng build (`scripts/clean.sh`).

## Công nghệ

| Lớp | Công nghệ |
|-------|------------|
| Lõi (Rust) | Codec thuần Rust: zune-jpeg, jpeg-encoder, png ([ADR-0012](docs/adr/0012-pure-rust-codecs-first.md)); HEIC qua hệ điều hành (ImageIO / WIC, [ADR-0017](docs/adr/0017-heic-through-os-codecs.md)); fast_image_resize, ab_glyph, rayon. Dự kiến: lcms2 cho màu, GeoNames cho địa điểm offline |
| Vỏ app | [Tauri 2](https://v2.tauri.app/): dùng webview của hệ điều hành, không kèm Chromium |
| Giao diện | React 19, TypeScript, Tailwind CSS 4, [shadcn/ui](https://ui.shadcn.com/), [Lucide](https://lucide.dev/) |
| IPC có kiểu | [tauri-specta](https://github.com/specta-rs/tauri-specta): binding TypeScript sinh ra từ Rust |

Thiết kế: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Các quyết định: [docs/adr/](docs/adr/README.md) (tiếng Anh).

## Tài liệu

Toàn bộ tài liệu nằm trong [`docs/`](docs/README.md), viết bằng tiếng Anh, và thay đổi cùng với code ([ADR-0006](docs/adr/0006-docs-as-code.md)).

| Tài liệu | Nội dung |
|-----|---------------|
| [PRD](docs/PRD.md) | Yêu cầu (`FR-xxx`, `NFR-xxx`) |
| [Architecture](docs/ARCHITECTURE.md) | Crate, pipeline, hợp đồng IPC |
| [Results](docs/RESULT.md) | Số đo tốc độ và bộ nhớ mới nhất |
| [Optimization](docs/OPTIMIZATION.md) | Ngân sách, phương pháp đo, nhật ký benchmark |
| [File formats](docs/EXTENSION.md) | `.fotollection`, `.fotopreset` và mọi loại file khác |
| [Style rules](docs/STYLE_RULE.md) | Quy ước Rust, TypeScript, UI, git và tài liệu |
| [Security](docs/SECURITY.md) | Mô hình mối đe dọa, quyền riêng tư, chuỗi cung ứng |
| [Roadmap](docs/ROADMAP.md) / [Status](docs/STATUS.md) | Các giai đoạn và tiến độ |

## Đóng góp

Rất hoan nghênh đóng góp. Vui lòng đọc [CONTRIBUTING.md](CONTRIBUTING.md) (tiếng Anh) trước:

- Mỗi thay đổi đều gắn với một mã yêu cầu.
- Tài liệu được cập nhật trong cùng PR.
- **Đóng góp từ bên ngoài cần ký CLA** ([ADR-0008](docs/adr/0008-license-gpl3-with-relicensing-option.md)).

Báo lỗi bảo mật qua [GitHub security advisory](https://github.com/hoangphi01/fotollection/security/advisories/new) riêng tư, không mở issue công khai.

## Giấy phép

[GPL-3.0-only](LICENSE) © hoangphi01.

### Ghi nhận bên thứ ba

- Mã hóa JPEG dùng [`jpeg-encoder`](https://github.com/vstroebel/jpeg-encoder), một phần dựa trên công trình của Independent JPEG Group.
- Dấu ngày dùng font [Inter](https://rsms.me/inter/), theo giấy phép SIL Open Font License 1.1 (`crates/core/assets/fonts/Inter-OFL.txt`).
- Dữ liệu địa lý từ [GeoNames](https://www.geonames.org/) (CC BY 4.0) sẽ được đóng gói để tra địa điểm offline.
- Gợi ý ảnh tốt nhất đóng gói model phát hiện khuôn mặt [YuNet](https://github.com/opencv/opencv_zoo/tree/main/models/face_detection_yunet) (MIT, © 2020 Shiqi Yu) và model điểm mốc MediaPipe FaceMesh V2 của Google (Apache License 2.0), đã chuyển sang ONNX. License và checksum: `crates/core/assets/models/`.
