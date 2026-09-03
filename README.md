# UDF Resimcisi

Bir fotoğrafı veya ekran görüntüsünü, **tam çözünürlükle** bir `.udf` belgesine koyar.
UYAP Doküman Editörü'nün (UDE) resmi belgeye alırken uyguladığı küçültmeyi tamamen atlar.

![Uygulama simgesi](src-tauri/icons/128x128.png)

## İndir

### ➡️ [Windows için indir — UDF-Resimcisi-kurulum.exe](https://github.com/SCgrS/udf-resimcisi/releases/latest/download/UDF-Resimcisi-kurulum.exe)

Bağlantıya tıklayınca dosya doğrudan inmeye başlar. İnen dosyaya çift tıklayın, kurulum biter;
yönetici izni istemez, Başlat menüsüne kısayol koyar.

Windows **"Bilgisayarınız korundu"** uyarısı gösterirse: **Daha fazla bilgi → Yine de çalıştır.**
Sebebi kod imzalama sertifikası olmaması.

Kurulum istemiyorsanız:
[taşınabilir sürüm](https://github.com/SCgrS/udf-resimcisi/releases/latest/download/UDF-Resimcisi-tasinabilir.exe)
(indir, çift tıkla, çalışır) ·
[MSI](https://github.com/SCgrS/udf-resimcisi/releases/latest/download/UDF-Resimcisi-x64.msi)
(kurumsal dağıtım için) ·
[tüm sürümler](https://github.com/SCgrS/udf-resimcisi/releases)

Windows 10/11 x64. **UYAP Doküman Editörü kurulu olmasa da çalışır**: belge her hâlükârda
üretilip kaydedilir. Editör kuruluysa belge üretildikten sonra ayrıca açılır.

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

Bu uygulama `.udf` dosyasını kendisi yazdığı için o küçültme kodu hiç çalışmaz.

Bütün ölçümlerin ayrıntısı: [DOGRULAMA.md](DOGRULAMA.md).

## Nasıl kullanılır

1. Resimleri pencereye sürükleyin veya **Dosya seç**'e basın. Panoda bir resim varsa
   **Ctrl+V** ile ya da pencerenin herhangi bir yerine sağ tıklayıp **Yapıştır** ile de
   ekleyebilirsiniz.
2. **UDF'de aç** düğmesine basın.

Belge kaydetme klasörüne yazılır ve (editör kuruluysa) açılır. Düğmenin altında, resimler
eklendikçe güncellenen bir **üretilecek dosya boyutu** yazar; onun altında da kaç resmin belgeye gireceği.

Desteklenen girdiler: PNG, JPEG, WEBP, BMP, TIFF, GIF (ilk kare).
PNG ve JPEG girdilerinde **tek bir bayt bile değiştirilmez** — yeniden sıkıştırma yoktur.
Telefon fotoğraflarında EXIF `Orientation` okunur; döndürme gerekiyorsa kayıpsız PNG'ye çevrilir.

Resim varsayılan olarak **tam çözünürlükle** gömülür; sayfaya sığması yalnızca punto cinsinden
görüntüleme ölçüsüyle sağlanır, bitmap'e dokunulmaz.

### Kalite

Düğmenin üstündeki listeden dosyayı küçültebilirsiniz. **Bu bir ölçek ayarı değildir:** resim
her basamakta sayfada aynı yeri kaplar, yalnızca içindeki piksel yoğunluğu — yani ayrıntı —
azalır.

| Seçenek | Belgeye giren bitmap (3000 × 2000 px girdi için) |
| --- | --- |
| **Orijinal Boyut** (varsayılan) | 3000 × 2000 px — baytlara hiç dokunulmaz |
| **Büyük Boyut** | 1574 × 1049 px |
| **Orta Boyut** | 1050 × 700 px |
| **Küçük Boyut** | 526 × 350 px — UDE'nin kendi "Ekle → Resim" çıktısıyla (524 × 349) eş değer |

Sayfaya zaten sığan küçük resimler hiçbir basamakta değiştirilmez: büyütmek kaliteyi artırmaz,
yalnızca dosyayı şişirir.

Aynı fotoğrafı birden çok kez ekleyebilirsiniz; belgeye eklediğiniz sırayla girer.

## Ayarlar

Sağ üstteki çark düğmesinden:

* **Tema** — Sistem / Açık / Koyu
* **Her resim ayrı sayfada** — kapalıyken (varsayılan) resimler arasına bir satır boşluk
  konur, açıkken her resim yeni sayfaya geçer
* **Kaydetme klasörü** — varsayılan `Belgelerim\UDF Resimcisi`
* **Güncellemeleri denetle** — yeni sürüm varsa tek tıkla indirip kurar
* UYAP evrak yükleme sınırının **10 MB** olduğu hatırlatması
* Sürüm ve geliştirici bilgisi

## Dosya boyutu

UYAP'a yüklenebilen evrak üst sınırı ≈ **10 MB**. Uygulama çözünürlüğe kendiliğinden dokunmaz;
düğmenin altındaki **üretilecek dosya boyutu** satırını izleyerek kararı siz verirsiniz. Belge
sınırı aşıyorsa düğmenin üstündeki kalite listesinden bir basamak inin.

Ölçülmüş büyüklükler:

| Girdi | Üretilen `.udf` |
|---|---|
| 12 MP telefon fotoğrafı (4000×3000 JPEG, 3,5 MB) | **3,5 MB** |
| 3000×2000 taranmış belge (PNG, 106 KB) | 190 KB |

Base64 kodlaması veriyi %33 şişirir ama `.udf` bir ZIP arşivi olduğu için sıkıştırma bunu geri
alır: dosya boyutu pratikte kaynak görselin boyutuna çok yakın çıkar.

## Kaldırma

Denetim Masası → Program Kaldır → **UDF Resimcisi**. Kaldırıcıdaki **Uygulama verilerini sil**
kutusunu işaretlerseniz ayar dosyası (`%APPDATA%\UDF Resimcisi`) ve kaydetme klasöründeki
`.udf` belgeleri de silinir. Kaydetme klasöründe uygulamanın üretmediği başka dosyalar varsa
onlara dokunulmaz ve klasör yerinde kalır.

## Kaynaktan derleme

Gerekenler: Rust (MSVC), Visual Studio Build Tools (C++), Node.js, WebView2 (Windows 10/11'de kurulu).

```bash
npm install
npm run tauri build
```

Çıktılar `src-tauri/target/release/bundle/` altında (MSI + NSIS), taşınabilir exe ise
`src-tauri/target/release/udf-resimcisi.exe`.

Testler:

```bash
cd src-tauri && cargo test
```

## Gizlilik

Belge üretimi tamamen yereldir; resimleriniz hiçbir yere gönderilmez. Uygulamanın tek ağ
bağlantısı, **siz "Güncellemeleri denetle" düğmesine bastığınızda** GitHub'a yaptığı sürüm
sorgusudur. Kendiliğinden hiçbir bağlantı kurulmaz, telemetri yoktur.

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
