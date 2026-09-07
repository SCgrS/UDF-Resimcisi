# UDF Resimcisi

Bir fotoğrafı ya da ekran görüntüsünü, çözünürlüğünü bozmadan `.udf` belgesine koyan Windows
uygulaması. UYAP'a evrak yükleyen avukat ve büro çalışanları için: taranmış bir dilekçeyi,
telefonla çekilmiş bir belgeyi ya da ekran görüntüsünü birkaç saniyede UYAP'ın kabul ettiği
belge biçimine çevirir.

UYAP Doküman Editörü (UDE) bir resmi **Ekle → Resim** yoluyla belgeye alırken onu sayfadaki
görüntüleme boyutuna, yaklaşık 72 DPI'a küçültür: 3000 × 2000 piksellik bir tarama belgeye
524 × 349 piksel olarak girer ve ince yazılar okunmaz olur. Ekleme penceresindeki **Kayıpsız**
seçeneği bunu değiştirmez. UDF Resimcisi `.udf` dosyasını kendisi yazdığı için o küçültme hiç
çalışmaz. Ölçümlerin tamamı [DOGRULAMA.md](DOGRULAMA.md) dosyasında.

> Bu uygulama UYAP veya HAVELSAN ile ilişkili değildir. Yalnızca birlikte çalışabilirlik için
> UDF dosyası üretir; UDE'ye ait hiçbir kod içermez.

## Ne yapar

- Sürüklenen, seçilen ya da panodan yapıştırılan resimleri tek bir `.udf` belgesine koyar.
- PNG ve JPEG girdilerinde **Orijinal Boyut** seçiliyken tek bir bayt bile değiştirilmez.
- Kalite listesi dosya boyutunu küçültür ama resmin sayfada kapladığı yeri değiştirmez.
- Telefon fotoğraflarındaki EXIF yön bilgisini okur; gerekiyorsa resmi kayıpsız PNG'ye
  çevirip düzeltir.
- Resimler eklendikçe üretilecek dosyanın boyutunu gerçekten hesaplayıp gösterir.
- Belgeyi kaydetme klasörüne yazar ve UDE kuruluysa açar; UDE yoksa yine kaydeder.
- Birden çok resim tek belgeye eklendiğinde araya boş satır ya da sayfa sonu koyar.

## Kurulum

### [⬇ UDF-Resimcisi-kurulum.exe indir](https://github.com/SCgrS/UDF-Resimcisi/releases/latest/download/UDF-Resimcisi-kurulum.exe)

Tek dosya, 2 MB. Windows 10 ve üzeri, 64 bit; yönetici hakkı gerekmez. UDE kurulu olmasa da
çalışır.

> Depo özel (private) olduğu için indirme bağlantısı yalnızca GitHub'da oturumu açık olan
> yetkili hesaplarda çalışır. Tüm sürümler: [Sürümler sayfası](https://github.com/SCgrS/UDF-Resimcisi/releases)

1. Yukarıdaki bağlantıya tıklayıp dosyayı indirin.
2. İndirilen `UDF-Resimcisi-kurulum.exe` dosyasına **çift tıklayın**.
3. Dosya imzalı olmadığı için Windows SmartScreen bir uyarı gösterebilir:
   **Daha fazla bilgi** yazısına tıklayın, sonra çıkan **Yine de çalıştır** düğmesine basın.
4. Kurulum birkaç saniye sürer; bitince Başlat menüsünden **UDF Resimcisi**'ni açın.

Kurulum sistem klasörlerine bir şey yazmaz:

- Uygulama `%LOCALAPPDATA%\UDF Resimcisi` klasörüne kopyalanır.
- Başlat menüsüne **UDF Resimcisi** kısayolu eklenir.
- Windows'un "Uygulamalar ve özellikler" listesine kaydedilir.

Kurulum istemiyorsanız aynı sürüm sayfasında iki seçenek daha var:
`UDF-Resimcisi-tasinabilir.exe` (5 MB; indirin, çift tıklayın, çalışır) ve
`UDF-Resimcisi-x64.msi` (3 MB; kurumsal dağıtım için).

### Güncelleme

Yeni sürümün `UDF-Resimcisi-kurulum.exe` dosyasını indirip çalıştırmanız yeterli; ayarlarınıza
dokunulmaz. **Ayarlar**'daki **Güncellemeleri denetle** düğmesi sürüm bilgisini GitHub'dan
sorar; depo özel olduğu sürece bu sorgu yanıt alamaz ve düğme "Sürüm bilgisi alınamadı" der.

## Kullanım

1. Resimleri pencereye sürükleyin ya da **Dosya seç**'e basın. Panoda bir resim varsa
   (örneğin `Win+Shift+S` ile alınmış ekran görüntüsü) **Ctrl+V** ile veya pencerede sağ
   tıklayıp **Yapıştır** ile ekleyin.
2. İsterseniz düğmenin üstündeki kalite listesinden bir basamak seçin.
3. **UDF'de aç**'a basın.

Belge kaydetme klasörüne yazılır; UDE kuruluysa hemen açılır, kurulu değilse "Belge
kaydedildi … açılamadı" yazar. Durum satırındaki **Klasörü aç** bağlantısı dosyayı Gezgin'de
seçili olarak gösterir.

- Desteklenen girdiler: PNG, JPEG, WEBP, BMP, TIFF, GIF (yalnızca ilk kare). PNG ve JPEG
  olduğu gibi gömülür; diğerleri kayıpsız PNG'ye çevrilir.
- Eklenen her resim listede piksel ölçüsü, sayfadaki boyutu (cm), biçimi ve dosya boyutuyla
  görünür; yanındaki **Kaldır** ile çıkarılır, **Tümünü kaldır** listeyi boşaltır.
- Aynı resim birden çok kez eklenebilir; belgeye eklediğiniz sırayla girer.
- Dosya adı ilk resmin adından türetilir (`tarama.png` → `tarama.udf`); panodan gelen
  resimlerde `resimler-YYYYAAGG-SSDD.udf` olur. Aynı ad varsa ` (2)`, ` (3)` eklenir.
- Kalite listesinin altındaki **Üretilecek dosya boyutu** satırı tahmin değildir: belge
  gerçekten kurulup ölçülür, her ekleme ve kalite değişiminde yenilenir.

### Kalite

Listedeki basamaklar dosyanın büyüklüğünü belirler. **Bu bir ölçek ayarı değildir:** resim her
basamakta sayfada aynı yeri kaplar, yalnızca içindeki piksel yoğunluğu (ayrıntı) azalır.

| Seçenek | Belgeye giren bitmap (3000 × 2000 px girdi için) |
| --- | --- |
| **Orijinal Boyut** | 3000 × 2000 px — baytlara hiç dokunulmaz |
| **Optimal Boyut** (varsayılan) | 1574 × 1049 px |
| **Orta Boyut** | 1050 × 700 px |
| **Küçük Boyut** | 526 × 350 px — UDE'nin kendi **Ekle → Resim** çıktısıyla (524 × 349) eş değer |

**Optimal Boyut** ekranda ve baskıda Orijinal Boyut'tan ayırt edilmez, dosya ise 10 kata kadar
küçülür. Tek bir pikselin bile korunması gerekiyorsa **Orijinal Boyut**'u seçin. Sayfaya zaten
sığan küçük resimler hiçbir basamakta değiştirilmez; büyütme yapılmaz.

UYAP'a yüklenebilen evrak üst sınırı yaklaşık **10 MB**'tır. Boyut satırı bu sınırı aşıyorsa
listeden bir basamak inin. Ölçülmüş örnek: 12 MP telefon fotoğrafı (4000 × 3000 JPEG, 3,5 MB)
Orijinal Boyut'ta 3,5 MB'lık `.udf` üretir; `.udf` bir ZIP arşivi olduğu için base64'ün
şişirdiği yer geri kazanılır.

## Ayarlar

Sağ üstteki çark düğmesi **Ayarlar** penceresini açar. Her değişiklik anında kaydedilir.

| Satır | Ne yapar |
| --- | --- |
| **Tema** | **Sistem** / **Açık** / **Koyu**. Sistem, Windows'un tema ayarını izler. |
| **Sayfa düzeni** → **Her resim ayrı sayfada** | Kapalıyken (varsayılan) resimler arasına bir boş satır konur; açıkken her resim yeni sayfaya geçer. |
| **Kaydetme klasörü** → **Değiştir** | Belgelerin yazıldığı klasör. Varsayılan `Belgelerim\UDF Resimcisi`. |
| **Güncelleme** → **Güncellemeleri denetle** | GitHub'dan son sürümü sorar; yeni sürüm bulursa **İndir ve kur** düğmesi çıkar. |

Pencerenin altında 10 MB hatırlatması, kalite basamakları hakkında kısa bir not, sürüm numarası
ve geliştirici bağlantısı bulunur.

## Verileriniz nerede duruyor?

Belge üretimi tamamen yereldir; resimleriniz hiçbir yere gönderilmez, telemetri yoktur.

- Ayarlar: `%APPDATA%\UDF Resimcisi\ayarlar.json`.
- Üretilen belgeler: seçtiğiniz kaydetme klasörü (varsayılan `Belgelerim\UDF Resimcisi`).
- Kayıt defteri: yalnızca `HKCU\Software\UDF Resimcisi\CiktiKlasoru` değeri; kaldırıcı
  hangi klasörü temizleyeceğini buradan öğrenir.
- Ağ: yalnızca siz **Güncellemeleri denetle**'ye bastığınızda `api.github.com`'a tek bir sürüm
  sorgusu; **İndir ve kur** derseniz kurulum dosyası `%TEMP%\UDF Resimcisi` altına indirilip
  çalıştırılır. Kendiliğinden hiçbir bağlantı kurulmaz.

## Kaldırma

**Ayarlar → Uygulamalar → Uygulamalar ve özellikler** listesinden **UDF Resimcisi**'ni seçip
**Kaldır**'a basın. Kaldırıcıdaki **Uygulama verilerini sil** kutusunu işaretlerseniz ayar
dosyası, kayıt defteri değeri ve kaydetme klasöründeki `.udf` belgeleri de silinir. Kaydetme
klasöründe uygulamanın üretmediği başka dosyalar varsa onlara dokunulmaz, klasör yerinde kalır.

Taşınabilir sürüm hiçbir şey kurmaz; `.exe` dosyasını silmeniz yeterlidir. Ayar dosyası ve
kayıt defteri değeri yukarıdaki yerlerde kalır.

## Bilinen sınırlar

- Yalnızca Windows. Belgeyi açmak için UDE gerekir; UDE yoksa belge yine üretilir.
- Depo özel olduğu sürece uygulama içi güncelleme denetimi çalışmaz; yeni sürümü elle indirin.
- Kod imzalama sertifikası olmadığından SmartScreen uyarısı her yeni sürümde çıkar.
- GIF'lerin yalnızca ilk karesi alınır. WEBP, BMP, TIFF ve GIF, UDE'nin tanımadığı biçimler
  olduğu için PNG'ye çevrilir; çevirme kayıpsızdır ama dosya boyutu değişebilir.
- Ayarlar penceresi açıkken **Ctrl+V** ve sağ tık menüsü çalışmaz.

### Kaynak koddan

Rust (MSVC), Visual Studio Build Tools (C++) ve Node.js gerekir. WebView2, Windows 10/11'de
zaten kuruludur.

```bash
npm install
npm run tauri build
```

Kurulum paketleri `src-tauri/target/release/bundle/` altına (MSI ve NSIS), taşınabilir sürüm
`src-tauri/target/release/udf-resimcisi.exe` olarak çıkar. Yardımcı ölçüm araçları
[tools/README.md](tools/README.md) dosyasında anlatılıyor.

## Teşekkür

Bu uygulama, aşağıdaki bağımsız geliştiricilerin emeği üzerine kurulu; her biri kendi
lisansıyla kullanıldı.

- **Tauri** ve eklentileri `tauri-plugin-dialog`, `tauri-plugin-opener` (Tauri Programme within
  The Commons Conservancy, MIT/Apache-2.0) — pencere, dosya ve klasör seçme diyalogları,
  Gezgin'de gösterme ve bağlantı açma.
- **image** (image-rs geliştiricileri, MIT/Apache-2.0) — resim biçimlerini çözme, küçültme,
  PNG/JPEG kodlama ve önizleme üretme.
- **kamadak-exif** (KAMADA Ken'ichi, BSD-2-Clause) — telefon fotoğraflarındaki EXIF yön
  bilgisini okuma.
- **arboard** (1Password, MIT/Apache-2.0) — panodaki resmi alma.
- **ureq** (Martin Algesten ve Jacob Hoffman-Andrews, MIT/Apache-2.0) — sürüm sorgusu ve
  kurulum dosyasını indirme.
- **zip** (zip-rs geliştiricileri, MIT) — `.udf` arşivini yazma.
- **base64** (Marshall Pierce ve katkıcılar, MIT/Apache-2.0) — resim baytlarını belgeye gömme.
- **serde**, **serde_json**, **anyhow**, **chrono** (David Tolnay, Erick Tryzelaar ve
  katkıcılar; MIT/Apache-2.0) — ayar dosyası, hata metinleri ve dosya adındaki tarih damgası.
- **ude-win-x64** (Said Surucu, MIT) — `tools/macospasterich/UdeXml.java` ve `UdeDoc.java`
  referans UDF yazıcısı; Rust portunun doğruluğu bu iki dosyayla bayt bayt karşılaştırılarak
  kanıtlandı. Ayrıntı: [tools/macospasterich/KAYNAK.md](tools/macospasterich/KAYNAK.md).

Lisans: MIT. Ayrıntılar için [LICENSE](LICENSE) dosyasına bakınız.
