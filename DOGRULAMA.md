# Ölçülmüş doğrulama sonuçları

Ortam: Windows 10 Pro 19045, **UYAP Doküman Editörü (Uyap Kelime İşlemci) 5.4.20**,
kurulum `C:\Uyap\Uyap Kelime Islemci\`. Ölçümler `tools/macospasterich/ProtoMain.java`
(referans serializer `UdeXml.java` ile birebir) ve gerçek UDE penceresi üzerinden yapıldı.

Test görseli: `testdata/test3000x2000.png` — 3000×2000 px, 1 px dama tahtası, 1 px çizgiler,
8–48 pt metin. Yeniden örnekleme yapılırsa görünür şekilde bozulur.

## §6.1 Turnusol testi — GEÇTİ

`format_id="1.8"`, tek girişli ZIP (`content.xml`), CDATA `U+FFFC` + `\n`.
UDE dosyayı hatasız açtı; durum çubuğu **"Belge Sürüm : 1.8(güncel)"**.

## §6.2 Tam çözünürlük kanıtı — GEÇTİ

Aynı `.udf` içinde iki resim, aynı kaynak görselden:

| Yol | Gömülü bitmap | Gömülü bayt | `width`/`height` |
|---|---|---|---|
| **Bizim ürettiğimiz** (UDE'de açılıp UDE'nin kendisi tarafından kaydedildi) | **3000 × 2000 px** | 98.201 | 524.4 / 349.6 |
| **UDE "Ekle → Resim"** (dialogda **Kayıpsız** seçili) | **524 × 349 px** | 14.903 | 524.0 / 349.0 |

UDE 5.4.20 resmi tam olarak punto görüntüleme boyutuna indiriyor (1 px = 1 pt ⇒ **≈72 DPI**).
Bu, promptun beklediği ~600×790'dan **daha yıkıcı**: doğrusal 5,7×, piksel sayısında 33× kayıp.
Bizim yolumuz UDE'nin kendi kaydetme turundan geçtikten sonra bile piksellerin tamamını koruyor.

Not: UDE 5.4.20'nin ekleme penceresinde "Kayıplı / Kayıpsız" seçeneği var. **Kayıpsız seçili
olsa bile küçültme yapılıyor** — o seçenek yalnızca sıkıştırma biçimini etkiliyor, çözünürlüğü değil.

## §6.3 Pano zinciri kanıtı (Kip A) — GEÇTİ

Belge UDE'de açıldı → tuvale tıklandı → `Ctrl+A` → `Ctrl+C`
(`GetClipboardSequenceNumber` 231 → 238, yani pano gerçekten değişti) →
`Ctrl+N` ile yeni belge → `Ctrl+V` → diske kaydedildi (`testdata/pano-yapistir.udf`).

Yapıştırılan belgedeki bitmap: **3000 × 2000 px, 98.201 bayt — kaynakla bayt-bayt aynı.**
Kip A güvenli.

## §6.4 JPEG kabul testi — GEÇTİ

`imageData` içine PNG yerine JPEG baytları konuldu (`testdata/jpegtest.udf`, 3000×2000 JPEG,
450.139 bayt). UDE dosyayı hatasız açtı ve resmi doğru gösterdi.
Sonuç: **JPEG doğrudan gömülebilir**; büyük fotoğraflarda ara dosya küçük kalır.

## Rust portunun doğruluğu — bayt-bayt karşılaştırma

Aynı 3000×2000 PNG için:

* Java referansı (`UdeXml.serialize`) → `content.xml`, 145.615 bayt
* Rust portu (`udf::serialize`) → `content.xml`, 145.615 bayt
* `cmp` farkı: **yok** — çıktı bayt-bayt aynı.

Bu, portun otorite kabul edilen uygulamayla birebir örtüştüğünün en güçlü kanıtı.
Ayrıca 38 birim test (`cargo test`) ve `cargo clippy --all-targets -- -D warnings` temiz.

## Uygulamayla uçtan uca ölçümler

Hepsi gerçek uygulama penceresi kullanılarak, gerçek UDE ile yapıldı.

| Senaryo | Sonuç |
|---|---|
| Panodaki 3000×2000 ekran görüntüsü → Kip A | `.udf` üretildi (168.637 bayt), UDE açtı, otomasyon `Ctrl+A`/`Ctrl+C` gönderdi, pano sıra numarası değişti, arayüz **"Kaliteli resim panoda"** dedi |
| Yukarıdaki pano içeriği → yeni UDE belgesine `Ctrl+V` → kaydet | Gömülü bitmap **3000 × 2000 px** (bkz. `testdata/uygulama-pano-sonuc.udf`) |
| Dosyadan JPEG + PNG (ikisi de 3000×2000), ayrı sayfa, Kip B | 2 resim, 1 `<page-break>`, offset'ler 0 ve 4; UDE "Sayfa : 1 / 2" gösterdi |
| Aynı belgede gömülü baytlar ↔ kaynak dosyalar | **`cmp` farkı yok** — JPEG de PNG de bit-bit korunmuş |
| 12 MP fotoğraf (4000×3000 JPEG, 3.702.117 bayt) | `.udf` = **3.721.281 bayt (3,5 MB)**. base64'ün %33 şişmesini ZIP sıkıştırması geri alıyor; 10 MB sınırının çok altında |
| Sıkışmayan 14,9 MB PNG (2600×2000 gürültü) | `.udf` = 15,1 MB → **9 MB uyarısı çıktı**, küçültme teklifi göründü |
| Uyarıdaki "300 DPI'lık küçük sürüm oluştur" | 2200×1692 px JPEG, `.udf` = 5,5 MB; dosya adı çakışması `buyuk-gurultu (2).udf` olarak çözüldü |
| EXIF `Orientation = 6` taşıyan JPEG | Piksel verisi döndürüldü, en/boy takas edildi, kayıpsız PNG olarak gömüldü (birim test) |
| Üretim süresi (süreç başlatma + okuma + yazma dâhil, sürüm derlemesi) | 3000×2000 PNG: **66 ms** · 12 MP JPEG: **172 ms** |
| Explorer'dan pencereye **gerçek sürükle-bırak** (`SendInput` ile OLE sürüklemesi) | Dosya listeye düştü, ölçüsü ve "orijinal baytlar korunuyor" işareti doğru göründü |
| Ayarların kalıcılığı | Uygulama kapatılıp yeniden açıldığında kip/klasör/seçenekler korunuyor |

## Biçim hakkında ek bulgular (kodda karşılığı var)

1. **UDE base64'ü MIME satır sonlarıyla yazar** (76 karakterde bir `\n`). Üretirken satırsız
   yazmak sorun değil — UDE okuyor — ama UDE'nin ürettiği dosyayı okuyan araç MIME çözücü
   kullanmalı.
2. UDE kendi kaydettiğinde resim paragrafına ek olarak `<content startOffset="1" length="1" />`
   koyuyor (paragraf sonu `\n` için). Bizim bunu yazmamamız kabul ediliyor.
3. UDE `bottomMargin="14.170000000000032"` gibi kayan nokta gürültüsü yazıyor; bizim
   `%.2f` biçimimiz sorunsuz okunuyor.
4. `.udf` dosya ilişkisi kayıt defterinde:
   `"C:\Uyap\Uyap Kelime Islemci\Uyap Doküman Editörü.exe" "getNewWPInstance" "EDITOR_TYPE_DOCUMENT" "%1" "%~s1"`
   — promptun §5.1'deki uyarısı doğrulandı: doğrudan çalıştırırken `getNewWPInstance` jetonu şart.
