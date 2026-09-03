# UDF Resimcisi

Bir fotoğrafı veya ekran görüntüsünü, **tam çözünürlükle** bir `.udf` belgesine koyar.
UYAP Doküman Editörü'nün (UDE) resmi belgeye alırken uyguladığı küçültmeyi tamamen atlar.

![Uygulama simgesi](src-tauri/icons/128x128.png)

## Sorun ve ölçüm

UDE resmi *dosyaya yazarken* kalite kaybetmiyor; **resmi belgeye alırken bitmap'i yeniden
örnekliyor**. Aynı 3000×2000 px görselle, aynı belgede, aynı makinede ölçüldü:

| Yol | Belgeye gömülen bitmap | Gömülü bayt |
|---|---|---|
| **UDF Resimcisi** (UDE'nin kendi kaydetme turundan geçtikten *sonra*) | **3000 × 2000 px** | 98.201 |
| UDE "Ekle → Resim" (pencerede **Kayıpsız** seçili) | **524 × 349 px** | 14.903 |

Yani UDE resmi tam olarak punto görüntüleme boyutuna, **≈72 DPI**'a indiriyor: doğrusal 5,7 kat,
piksel sayısında 33 kat kayıp. Taranmış bir dilekçenin ince metni bu ölçekte okunamaz hâle gelir.

Ekleme penceresindeki "Kayıplı / Kayıpsız" seçeneği bu sonucu değiştirmiyor — o seçenek yalnızca
sıkıştırma biçimini etkiliyor, çözünürlüğü değil.

Bu uygulama `.udf` dosyasını kendisi yazdığı için o küçültme kodu hiç çalışmaz. Bitmap ne ise o
gömülür; sayfaya sığdırma yalnızca `width` / `height` **punto** öznitelikleriyle yapılır.

Bütün ölçümlerin ayrıntısı: [DOGRULAMA.md](DOGRULAMA.md).

## Ne yapar

1. Resimleri pencereye sürükleyin (veya "Dosya seç" / "Panodaki resmi al").
2. `.udf` belgesi 1 saniyenin altında üretilir ve çıktı klasörüne kaydedilir.
3. Seçtiğiniz kipe göre:
   - **Kip A — Panoya kopyala** (varsayılan): belge UDE'de açılır, içeriği panoya alınır, size
     "Ctrl+V yapın" denir. Panonun gerçekten değiştiği `GetClipboardSequenceNumber` ile
     doğrulanır; doğrulanamazsa size dürüstçe "Ctrl+A → Ctrl+C yapın" kılavuzu gösterilir.
     **Sessiz başarısızlık yoktur.**
   - **Kip B — UDF'yi aç**: belge kaydedilir ve açılır, kopyalama yapılmaz.

Üretim hızlıdır: 3000×2000 PNG için 66 ms, 12 MP fotoğraf için 172 ms (süreç başlatma, dosya
okuma ve yazma dâhil).

Desteklenen girdiler: PNG, JPEG, WEBP, BMP, TIFF, GIF (ilk kare).
PNG ve JPEG girdilerinde **tek bir bayt bile değiştirilmez** — yeniden sıkıştırma yoktur.
Telefon fotoğraflarında EXIF `Orientation` okunur; döndürme gerekiyorsa kayıpsız PNG'ye çevrilir.

## Seçenekler

| Ayar | Varsayılan |
|---|---|
| Kip | A (Panoya kopyala) |
| Otomasyon (Ctrl+A / Ctrl+C gönder) | Açık |
| Her resim ayrı sayfada | Açık |
| Görüntüleme boyutu | Sayfaya sığdır |
| Çıktı klasörü | `Belgelerim\UDF Resimcisi` |
| 9 MB üstü uyarısı | Açık |

## 10 MB sınırı

UYAP'a yüklenebilen UDF üst sınırı ≈ **10 MB**. Uygulama çözünürlüğe kendiliğinden dokunmaz;
belge 9 MB'ı aşarsa uyarır ve tek tıkla 300 DPI'lık (≈2200 px uzun kenar) küçük bir sürüm üretmeyi
teklif eder. Kararı siz verirsiniz.

Ölçülmüş büyüklükler:

| Girdi | Üretilen `.udf` |
|---|---|
| 12 MP telefon fotoğrafı (4000×3000 JPEG, 3,5 MB) | **3,5 MB** |
| 3000×2000 taranmış belge (PNG, 106 KB) | 190 KB |
| 2600×2000 sıkışmayan görsel (PNG, 14,9 MB) | 15,1 MB → uyarı → küçültülünce 5,5 MB |

Base64 kodlaması veriyi %33 şişirir ama `.udf` bir ZIP arşivi olduğu için sıkıştırma bunu geri
alır: dosya boyutu pratikte kaynak görselin boyutuna çok yakın çıkar.

Kabaca ölçek: A4 kullanılabilir genişlik 7,28 inç → 300 DPI ≈ 2200 px uzun kenar. UDE'nin kendi
hâli ≈ 72 DPI.

## Kurulum

`Releases` sayfasındaki `.msi` veya `.exe` kurulum dosyasını indirin.

Kod imzalama sertifikası olmadığı için Windows SmartScreen bir uyarı gösterebilir:
**"Daha fazla bilgi" → "Yine de çalıştır"**.

Kip A'nın çalışması için UYAP Doküman Editörü'nün kurulu olması gerekir. Kip B ve belge üretimi
UDE olmadan da çalışır.

## Kaynaktan derleme

Gerekenler: Rust (MSVC), Visual Studio Build Tools (C++), Node.js, WebView2 (Windows 10/11'de kurulu).

```bash
npm install
npm run tauri build
```

Çıktılar `src-tauri/target/release/bundle/` altında (MSI + NSIS), taşınabilir exe ise
`src-tauri/target/release/UDF Resimcisi.exe`.

Testler:

```bash
cd src-tauri && cargo test
```

## Gizlilik

Tamamen yerel çalışır. Ağ bağlantısı kurmaz, hiçbir veri gönderilmez, telemetri yoktur.

## Yasal not

Bu uygulama UYAP veya HAVELSAN ile ilişkili değildir. Yalnızca birlikte çalışabilirlik amacıyla
UDF dosyası üretir; UDE'ye ait hiçbir kod içermez.

## Lisans

MIT — bkz. [LICENSE](LICENSE).

`tools/macospasterich/UdeXml.java` ve `UdeDoc.java` bu projeye ait değildir:
[saidsurucu/ude-win-x64](https://github.com/saidsurucu/ude-win-x64) projesinden alınan referans
UDF serializer'dır (MIT, © 2026 Said Surucu). Rust portunun doğruluğu bu iki dosyayla bayt-bayt
karşılaştırılarak kanıtlandığı için depoda tutuluyorlar. Ayrıntı ve lisans metni:
[tools/macospasterich/KAYNAK.md](tools/macospasterich/KAYNAK.md).
