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

## Görünmeden kopyalama (1.1 sürümü) — ölçümler

1.0'da "Panoya kopyala" UDE penceresini kullanıcının gözü önünde açıyordu. 1.1'de bütün işlem
görünmez yapılıyor. Yol açan ölçümler:

| Soru | Ölçüm |
|---|---|
| Tuşların editöre ulaşması için tuvale tıklamak şart mı? | **Hayır.** Taze açılan belgede tuval zaten odaklı; `AttachThreadInput` + `SetForegroundWindow` + `SetFocus` sonrası `Ctrl+A`/`Ctrl+C` çalışıyor (pano sıra no 377 → 384). 1.0'daki tıklama gereksizmiş. |
| Pencere görünmezken tuş alır mı? | **Alır.** Ekran dışına taşınmış pencerede de (384 → 391), yerinde saydamlaştırılmış pencerede de (407 → 412) kopyalama çalışıyor. |
| UDE açılışında ekranda ne beliriyor? | İki pencere: `JavaSplash` sınıfı açılış görseli (**~140 ms**) ve `SunAwtFrame` sınıfı belge penceresi (**~2 sn**). İkisi de sınıfından tanınıp görünmez yapılabiliyor. |
| UDE zaten çalışırken yeni belge kaç saniyede açılıyor? | **2,0 sn** (soğuk açılış da bu makinede ~2 sn sürdü; önbelleği soğuk makinede daha uzun olabilir). |
| Görünmez kopyalamanın toplam süresi | **≈3,4 sn** (dosya yazma + UDE açılışı + kopyalama + kapatma). |
| Panodaki içerik gerçekten tam çözünürlük mü? | **Evet.** Görünmeden kopyalanan içerik yeni bir UDE belgesine yapıştırılıp kaydedildi: **3000 × 2000 px**, 105.925 bayt PNG. |

### Tuzak: pencereyi taşımak UDE'nin ayarını kalıcı bozuyor

İlk denemede pencere `-32000,-32000` konumuna taşınıyordu. UDE kapanırken pencere konumunu
`~/.uki/tercihler.xml` içindeki `win_posx` / `win_posy` alanlarına **kalıcı** yazıyor; sonuçta
kullanıcının kendi UDE açılışı da ekran dışında oluyordu (ölçülerek görüldü, dosya elle onarıldı).

Bu yüzden 1.1'de pencere **taşınmıyor**; yerinde saydamlaştırılıyor
(`WS_EX_LAYERED` + `SetLayeredWindowAttributes(alpha = 0)`). Geometri hiç değişmediği için
kayıtlı konum bozulmuyor — test sonrası `win_posx`/`win_posy` değerlerinin değişmediği doğrulandı.

### Tuzak: pencereyi tam yolla aramak tutmuyor

Doğru pencereyi bulmak için başlıktaki tam yolla eşleştirmek denendi ve **başarısız oldu**:
`TEMP` ortam değişkeni 8.3 kısa biçimde olabiliyor
(`C:\Users\ARAHIN~1\AppData\Local\Temp\...`) ama UDE pencere başlığında uzun biçimi gösteriyor
(`C:\Users\Çağrı Şahin\AppData\Local\Temp\...`). Sonuç: pencere bulunamıyor, kopyalama
"başarısız" diyor ve geride görünür bir UDE penceresi kalıyordu.

Çözüm: geçici belgeye **benzersiz** bir ad veriliyor
(`udfres-<yıl><ay><gün>-<saat><dakika><saniye>-<ms>.udf`) ve eşleştirme dosya adı üzerinden
yapılıyor. Ad benzersiz olduğu için kullanıcının açık olan kendi belgesiyle karışma riski de yok.

## 1.2 sürümünde kaldırılanlar

Panoya arka planda kopyalama özelliği (1.1) **tamamen kaldırıldı**: `ude.rs` yalnızca "belgeyi
aç" ve "UDE kurulu mu" işlevlerini tutuyor; pencere arama, saydamlaştırma, `SendInput`, pano
sıra numarası denetimi ve ilgili tüm kod silindi. Uygulama artık **UDE kurulu olmadan da**
çalışıyor: belge her hâlükârda üretilip diske yazılıyor, UDE varsa ayrıca açılıyor.

Yine kaldırılanlar: 9 MB uyarısı ve 300 DPI küçültme, görüntüleme boyutu seçenekleri
(artık her zaman tam çözünürlük + sayfaya sığdırma), kip seçimi ve otomasyon onay kutusu.

1.2'de doğrulananlar:

| Senaryo | Sonuç |
|---|---|
| Aynı resim iki kez ekleme | İkisi de listeye ve belgeye sırayla giriyor (birim test + uygulamada ölçüldü) |
| "Her resim ayrı sayfada" kapalı (yeni varsayılan) | Belgede `<page-break>` yok; resimler arasında tek bir boş paragraf ("enter") var: `startOffset="2" length="2"`. Resim offset'leri 0 ve 4. |
| Eski ayar dosyası | 1.x'ten gelen `ayri_sayfa: true` değeri, ayar sürümü taşımasıyla yeni varsayılana (kapalı) çekiliyor; kullanıcının 1.2'de kendi yaptığı seçim korunuyor |
| Güncelleme denetimi | Depo gizli olduğu için GitHub API 404 dönüyor ve arayüz bunu dürüstçe söylüyor ("depo gizliyse sürüm bilgisi dışarıya kapalıdır"). Depo herkese açıldığında çalışır. |

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

## 1.3 sürümü — kalite basamakları ve kaldırma temizliği

### Kaldırırken "Uygulama verilerini sil" gerçekten silmiyordu

Ölçüm (1.2.0, bu bilgisayarda kaldırıldı, kutu işaretli):

| Yer | Kaldırmadan sonra |
|---|---|
| `%LOCALAPPDATA%\UDF Resimcisi` (program) | silindi |
| `%APPDATA%\UDF Resimcisi\ayarlar.json` | **duruyor** |
| `Belgelerim\UDF Resimcisi\a.udf` | **duruyor** |

Sebep: Tauri'nin NSIS şablonu yalnızca `$APPDATA\<paket kimliği>` ve
`$LOCALAPPDATA\<paket kimliği>` klasörlerini siliyor (`com.cagrisahin.udfresimcisi`), bu
uygulamanın verileri ise ürün adıyla açılmış klasörlerde duruyor.

Çözüm: `src-tauri/nsis/hooks.nsh` içindeki `NSIS_HOOK_POSTUNINSTALL` kancası. Kaydetme
klasörünün güncel yolunu uygulama `HKCU\Software\UDF Resimcisi\CiktiKlasoru` değerine yazıyor
(`src-tauri/src/kayit.rs`); kaldırıcı oradan okuyor.

Ölçüm (1.3.0, kutu işaretli):

| Yer | Kaldırmadan sonra |
|---|---|
| `%LOCALAPPDATA%\UDF Resimcisi` | silindi |
| `%APPDATA%\UDF Resimcisi` | silindi |
| Kaydetme klasöründeki `deneme.udf` | silindi |
| Kaydetme klasöründeki `onemli.txt` (uygulamanın üretmediği dosya) | **korundu**, klasör de yerinde kaldı |
| `HKCU\Software\UDF Resimcisi` | silindi |

Kaydetme klasörü hiçbir zaman özyinelemeli silinmiyor: yalnızca `*.udf` siliniyor, klasör de
ancak boş kaldıysa kaldırılıyor. Kullanıcı kaydetme klasörü olarak "Belgelerim"in kendisini
seçmiş olabilir. (1.7.3'te değişti: `*.udf` kullanıcının kendi belgelerini de kapsıyordu;
bkz. "1.7.3 sürümü".)

Bir kez, uygulama kapatıldıktan ~1 sn sonra kaldırıcı çalıştırıldığında `%APPDATA%` klasörü
silinemedi (dosya tutamacı hâlâ açıktı). Kanca artık iki saniye bekleyip bir kez daha deniyor,
o da olmazsa işi `/REBOOTOK` ile yeniden başlatmaya bırakıyor.

### Kalite basamakları

Basamaklar **punto başına düşen piksel** olarak tanımlı. UDE 1 pikseli 1 punto saydığı için
"punto başına 1 piksel" tam olarak UDE'nin kendi `Ekle → Resim` çıktısına denk geliyor.

3000 × 2000 px girdi (sayfaya sığdırılmış görüntüleme ölçüsü 524,41 × 349,61 punto):

| Seçenek | Belgeye giren bitmap |
|---|---|
| Orijinal Boyut | 3000 × 2000 px (baytlar korunuyor) |
| Optimal Boyut (1.5 sürümünden beri varsayılan) | 1574 × 1049 px |
| Orta Boyut | 1050 × 700 px |
| Küçük Boyut | 526 × 350 px |
| UDE "Ekle → Resim" (Kayıpsız seçili) | 524 × 349 px |

Uygulamada ölçüldü (2600 × 2000 px pano görüntüsü, gürültü deseni):
Orijinal Boyut'ta belge 19,1 MB; Küçük Boyut'ta 60 KB. Üretilen belgedeki resim öğesi
`width="524.4" height="403.8"` — Orijinal Boyut'takiyle aynı yeri kaplıyor (18,5 × 14,2 cm).

Sayfaya zaten sığan resimler hiçbir basamakta değiştirilmiyor: hedef piksel ölçüsü görüntüleme
ölçüsünün üstünde tutuluyor, böylece resim küçülmüyor; büyütme de hiç yapılmıyor.

Kodlama: küçültülen resim hem PNG hem JPEG olarak kodlanıp küçük olanı seçiliyor. Alfa kanalı
gerçekten kullanılıyorsa (herhangi bir piksel saydamsa) JPEG hiç denenmiyor.

## 1.6 sürümü — yapıştırma ölçümü, İdeal Boyut, UDE zorunluluğu, kendini güncelleme

Ortam: Windows 11 Pro 26200, UDE 5.4.20 (`C:\Uyap\Uyap Kelime Islemci\`). Ölçüm aracı
`src-tauri/examples/olcum.rs` (`cargo run --release --example olcum -- <resim> [--yaz <klasör>]`).
Kalite basamağının adı bu sürümde **Optimal → İdeal** oldu; davranışı aynı (punto başına 3 px).

### UDE yapıştırılan her resmi PNG olarak yeniden kodluyor

Kullanıcı gözlemi: "İdeal'de 400 KB iniyor ama UDE'de kopyalayıp dilekçeye yapıştırınca
3,38 MB oluyor." Sınandı: uygulamanın ürettiği belge UDE'de açıldı → `Ctrl+A`, `Ctrl+C`
(pano sıra numarası değişti) → `Ctrl+N` → `Ctrl+V` → `Ctrl+S` ile kaydedildi; kaydedilen
dosyanın içindeki resim çözüldü.

| Kaynak belge | Gömülü resim | Yapıştırılıp kaydedilen belge | İçindeki resim |
|---|---|---|---|
| `telefon-12mp-ideal.udf` (307.302 B) | **JPEG** 1575 × 1181, 317.949 B | 1.692.811 B | **PNG** 1575 × 1181, 1.648.490 B |
| `app1-ideal.udf` (446.070 B) | PNG 1575 × 866, 465.686 B | 368.825 B | PNG 1575 × 866, 364.757 B (baytlar farklı: yeniden kodlanmış) |

Sonuç: JPEG de PNG de UDE tarafından **yeniden PNG olarak yazılıyor**; gömdüğümüz biçim yalnızca
`.udf` dosyasının boyutunu etkiliyor, dilekçeye giren miktarı **piksel sayısı** belirliyor.
Word'den çevrilen belgeyle aynı büyüklüğe çıkması da bundan: iki yolda da resim benzer piksel
sayısıyla PNG olarak gömülüyor.

### Yapıştırma boyutu kestirimi

UDE'nin PNG'sine hangi kodlama ayarı yaklaşıyor? `image` sandığıyla her ayar denendi
(UDE'nin yazdığına göre fark):

| Sıkıştırma | Filtre | telefon-ideal (RGB) | app1-ideal (RGBA) |
|---|---|---|---|
| Default | **NoFilter** | **+%2** | **+%2** |
| Default | Adaptive | −%11 | +%14 |
| Best | Adaptive | −%18 | +%14 |
| Fast | Adaptive | +%46 | +%28 |

UDE (Java ImageIO) satır filtresi kullanmıyor, varsayılan deflate düzeyiyle yazıyor.
`image_io::yapistirma_boyutu` bu ayarla kodlayıp ölçüyor; arayüzdeki boyut satırı
"dilekçeye yapıştırıldığında yaklaşık …" olarak bunu gösteriyor. Sonuç basamak başına
önbelleğe alınıyor.

### Yeni ölçümler (tüm basamaklar)

12 MP telefon fotoğrafı, 4000 × 3000 JPEG, 3,53 MB:

| Basamak | Bitmap | Biçim | Bitmap boyutu | `.udf` | Yapıştırınca ≈ |
|---|---|---|---|---|---|
| Orijinal | 4000 × 3000 | JPEG | 3,53 MB | 3,55 MB | 13,35 MB |
| İdeal | 1575 × 1181 | JPEG | 310 KB | 300 KB | 1,60 MB (ölçülen: 1,65 MB) |
| Orta | 1050 × 788 | JPEG | 76 KB | 68 KB | 544 KB |
| Küçük | 526 × 395 | JPEG | 15 KB | 13 KB | 108 KB |

Taranmış A4 dilekçe sayfası (sentetik: Times New Roman 12 pt, 300 DPI, sensör gürültüsü),
2480 × 3508 JPEG, 1,93 MB:

| Basamak | Bitmap | Biçim | Bitmap boyutu | `.udf` | Yapıştırınca ≈ |
|---|---|---|---|---|---|
| Orijinal | 2480 × 3508 | JPEG | 1,93 MB | 1,93 MB | 7,73 MB |
| İdeal | 1574 × 2227 | JPEG | 789 KB | 777 KB | 2,82 MB |
| Orta | 1050 × 1485 | JPEG | 259 KB | 250 KB | 992 KB |
| Küçük | 525 × 743 | JPEG | 64 KB | 60 KB | 192 KB |

Ekran görüntüsü (`app1.png`, 2576 × 1416 PNG, 333 KB): İdeal basamağı yeniden örnekleyince
PNG **büyüyordu** (455 KB > 333 KB; keskin kenarlar ara tonlara dönüşüp sıkışmayı bozuyor).
`kaliteye_indir` artık küçültme dosyayı büyütüyorsa orijinali koruyor: İdeal = Orijinal =
309 KB `.udf`. Orta 228 KB, Küçük 71 KB.

`docs/karsilastirma.png`: taranmış sayfanın aynı bölgesi (sol 25 mm, üst 118 mm, 78 × 30 mm)
üç basamaktan kesilip aynı ekran ölçüsüne getirildi. Küçük Boyut'ta harfler bulanık; İdeal ile
Orijinal ayırt edilmiyor.

### UDE zorunlu

"UDE olmadan da çalışır" davranışı kaldırıldı: `udfde_ac` önce `ude_kurulu_mu()` bakıyor, UDE
yoksa belge üretmeden hata döndürüyor; UDE açılamazsa (dosya yazılmış olsa da) hata dönüyor.
Arayüz açılışta UDE'yi bulamazsa durum satırına bunu yazıyor ve **UDF'de aç** düğmesini hiç
açmıyor. `UretimSonucu.ude_acildi` alanı silindi.

### Klasörü aç

Durum satırındaki bağlantı yerine büyük düğmenin altında her zaman etkin küçük bir düğme.
Son üretilen belge varsa onu Gezgin'de seçili gösteriyor (`revealItemInDir`), yoksa kaydetme
klasörünü açıyor (`klasoru_ac`: klasör yoksa önce oluşturuyor).

### Kendini güncelleme

- Açılışta (ayar açıksa, varsayılan açık) 2,5 sn sonra arka planda `guncelleme_denetle`;
  yeni sürüm varsa başlık altında şerit: **Güncelle** / **Daha sonra**.
- **Güncelle** ve Ayarlar'daki **Şimdi denetle ve güncelle**: indir → `%TEMP%\UDF Resimcisi\UDF-Resimcisi-kurulum.exe`
  → `/P /R` ile çalıştır (yalnızca ilerleme penceresi; bitince uygulamayı yeniden aç) →
  uygulama 1,5 sn sonra kendini kapatır. Tauri'nin NSIS şablonu `/P` kipinde çalışan
  uygulamayı kendisi de kapatıyor, `/R` ile yeniden başlatıyor.
- Sürüm adresi `UDF_RESIMCISI_SURUM_ADRESI` ortam değişkeniyle yerel bir sunucuya
  yönlendirilebiliyor; uçtan uca sınama bununla yapıldı (aşağıda).

Uçtan uca sınama (bu bilgisayarda kurulu 1.5.0 üzerinde): yerel bir Node sunucusu
`/latest.json` için `tag_name: v9.9.9` ve kurulum dosyası olarak 1.6.0'ın NSIS paketini verdi.
1.6.0'ın taşınabilir sürümü ortam değişkeniyle açıldı.

| Adım | Gözlem |
|---|---|
| Açılıştan 2,5 sn sonra | Sunucu günlüğü: `GET /latest.json` (User-Agent `UDF-Resimcisi`); pencerede şerit: "Yeni sürüm v9.9.9 hazır (kullandığınız: 1.6.0). Güncelle · Daha sonra" |
| **Güncelle** tıklandı | `GET /latest.json` + `GET /UDF-Resimcisi-kurulum.exe`; dosya `%TEMP%\UDF Resimcisi\UDF-Resimcisi-kurulum.exe` (2.047.637 B) olarak indi |
| ~5 sn sonra | Taşınabilir süreç kapanmış, kurucu bitmiş (soru sormadı), `%LOCALAPPDATA%\UDF Resimcisi\udf-resimcisi.exe` **1.5.0 → 1.6.0**, uygulama o yoldan yeniden açılmış (`/R`) |
| Yeniden açılan uygulama | Ortam değişkeni kurucudan miras kaldığı için sahte sunucuya yeniden sordu ve şeridi yine gösterdi — sınama ortamının yan etkisi, gerçek kullanımda değişken yok |

Ayarlar dosyası ve kayıt defteri değeri korunmuş; belge üretimi 1.6.0'da yeniden denendi.

### Arayüz doğrulaması (ekran görüntüleriyle)

Gerçek pencere `PrintWindow` ile yakalandı: yeni alt başlık; listede **İdeal Boyut**; boyut
satırı "Üretilecek dosya boyutu: 776 KB · Dilekçeye yapıştırıldığında yaklaşık 2.8 MB"
(2480 × 3508 tarama); **UDF'de aç** sonrası yeşil "Belge hazır (776 KB) ve UYAP Doküman
Editörü'nde açıldı." ve UDE'de belge; büyük düğmenin altında ortalı küçük **Klasörü aç**
(tıklanınca Gezgin `Belgelerim\UDF Resimcisi`'ni açtı); Ayarlar'da **Açılışta yeni sürümü
denetle** kutusu ve **Şimdi denetle ve güncelle** düğmesi, sürüm 1.6.0. Tuzak: UDE
penceresinin başlığında da "UDF Resimcisi" (klasör adı) geçtiği için pencereyi başlıktan
aramak yanlış pencereyi buluyor; tıklamalar pencere tanıtıcısıyla (hwnd) yapıldı.

## 1.7 sürümü — resim başına kalite, geliştirici bağlantısı

### Resim başına kalite

Geliştirme derlemesi (`npm run dev`) gerçek pencerede denendi. Girdiler: `test3000x2000.png`,
`telefon-12mp.jpg`, `dilekce-tarama.jpg`.

| Adım | Satırlardaki "Belgede" | Alttaki seçim | Boyut satırı |
|---|---|---|---|
| Üçü eklendi (varsayılan İdeal) | 106 KB · 310 KB · 789 KB | İdeal Boyut | 1.1 MB · yapıştırınca ≈ 4.5 MB |
| Telefon fotoğrafı satırdan **Orijinal** | 106 KB · **3.5 MB** · 789 KB; satırda yeşil "Orijinal baytlar korunuyor" | **Özel** (kendiliğinden) | 4.4 MB · ≈ 16.3 MB |
| Alttan **Küçük Boyut** | 16 KB · 15 KB · 64 KB; bütün satırlar Küçük, yeşil işaret kalktı | Küçük Boyut | 80 KB · ≈ 324 KB |
| 1. satır Orta, sonra yine Küçük | Orta'dayken alt seçim Özel; Küçük'e dönünce 16 KB | Küçük Boyut'a döndü | 80 KB · ≈ 324 KB |

"Özel" seçeneği alttaki açılır listede görünmüyor, elle seçilemiyor; yalnızca gösteriliyor.
Karışık basamakla belge kurulumu birim testle sınandı (`her_resim_kendi_kalitesiyle_hazirlanir`):
Orijinal satırın baytları aynen gömülüyor, Küçük < İdeal < Orijinal piksel; genel seçimden
sonra hepsi aynı çıktıyı veriyor. **UDF'de aç** bu sürümde UDE ile yeniden denenmedi; boyut
satırıyla aynı `belge_uret` yolunu kullanıyor.

### Ayarlardaki geliştirici bağlantısı

1.6.0'da tıklanınca hiçbir şey olmuyordu: izin dosyasında `opener:allow-open-url` vardı ama
kapsamı (izin verilen adresler) boştu; eklenti bu durumda her adresi reddeder. Kapsam
`https://x.com/*` ile sınırlandırıldı. Tıklanınca varsayılan tarayıcı açıldı, pencere başlığı
"… (@CgrShn) / X".

## 1.7.1 sürümü — bellek

Ortam: Windows 11 Pro 26200, WebView2 154.0.4258.37, 12 çekirdek, 32 GB. Ölçüm aracı
`tools/bellek-olcumu.ps1`: uygulama gizli pencereyle derlendi, ekran dışına taşınıp odak
verilmeden gösterildi; resimler WebView2 hata ayıklama kapısından sayfanın kendi işlevleriyle
(`yollariEkle`, alttaki kalite seçimi) eklendi. Değer: Görev Yöneticisi'nin **Bellek** sütunu (özel çalışma kümesi), uygulama süreci
ile altındaki bütün WebView2 süreçlerinin toplamı. Girdiler: `test3000x2000.png`,
`telefon-12mp.jpg`, `dilekce-tarama.jpg`; "dikey" fotoğraf, `telefon-12mp.jpg`'ye EXIF
`Orientation = 6` eklenerek üretildi (dikey çekilmiş telefon fotoğrafları böyle gelir).

| Durum | 1.7.0 | 1.7.1 |
|---|---|---|
| Boşta (açılıştan 10 sn sonra) | 78,8 MB | **69,5 MB** |
| 3 resim, İdeal | 92,7 MB | **78,1 MB** |
| 3 resim, Orijinal | 94,5 MB | **79,3 MB** |
| 9 resim (3'ü dikey telefon fotoğrafı), İdeal | 194,4 MB | **105,1 MB** |
| — bunun uygulama süreci | 105,3 MB | **34,2 MB** |
| Liste temizlendi | 93,2 MB | **74,6 MB** |
| Simge durumunda | 89,4 MB | **54,8 MB** |
| Uygulama sürecinin tepe noktası (işlenen bellek) | 331 MB | **192 MB** |

Boştaki dağılım (1.7.0 → 1.7.1): WebView2 tarayıcı süreci 30,0 → 30,2 MB, GPU süreci
21,1 → 8,3 MB, sayfa süreci 13,4 → 16,6 MB, ağ + depolama + çökme raporlayıcı 11,1 → 11,2 MB,
uygulama süreci 3,2 → 3,2 MB. Uygulamanın kendisi boşta 3 MB; gerisi WebView2.

### Ne değişti

1. **Dikey telefon fotoğrafının PNG'si bellekte tutulmuyor.** EXIF yönü düzeltilen 12 MP
   fotoğraf kayıpsız PNG'ye çevrildiğinde **26,9 MB** tutuyordu (girdi JPEG 3,5 MB) ve bu PNG
   liste boyunca bellekte duruyordu. Artık yeniden kodlanması gereken girdide (EXIF yönü ya da
   WEBP/BMP/TIFF/GIF) PNG girdiden büyükse PNG atılıyor; girdi ve yön saklanıyor, pikseller
   gerektiğinde girdiden yeniden çözülüyor. PNG kayıpsız olduğu için bu pikseller PNG'ninkilerle
   birebir aynı; PNG yalnızca **Orijinal Boyut** seçilince yeniden üretiliyor (bayt bayt aynı
   çıkıyor) ve başka basamağa geçilince bırakılıyor. BMP gibi PNG'si girdiden küçük olanlarda
   eskisi gibi PNG tutuluyor.
2. **Önizleme bir kez üretiliyor.** Liste her yenilendiğinde (ekleme, kaldırma, temizleme)
   bütün resimler baştan çözülüp 280 px PNG önizleme üretiliyordu. Artık resim yüklenirken bir
   kez, satırdaki 64 × 48'lik kutuya %300 ekran ölçeğine kadar yeten 192 × 144 boyutunda ve
   saydamlık yoksa JPEG olarak üretiliyor (PNG'nin onda biri kadar).
3. **Belge bellekte bütün olarak kurulmuyor.** Boyut satırı her değişiklikte belgeyi baştan
   kuruyordu: resim baytlarının kopyası, base64 metni (%33 büyük), bütün `content.xml` ve ZIP
   çıktısı aynı anda bellekteydi. Artık baytlar paylaşımlı (`Arc`), XML ve base64 sıkıştırıcıya
   parça parça akıyor; boyut satırı çıktıyı tutmadan yalnızca sayıyor (`udf::Olcer`),
   **UDF'de aç** doğrudan dosyaya yazıyor. Yapıştırma kestirimindeki PNG de artık yalnızca
   sayılıyor.
4. **WebView2 GPU'suz çiziyor** (`additionalBrowserArgs`: Tauri'nin varsayılanı +
   `--disable-gpu`). GPU süreci 21 → 8 MB. Denenen öteki ayarlar (boşta toplam; son iki satır
   1.7.1 derlemesinde, ilk üçü 1.7.0'da — boştayken iki sürümün uygulama süreci aynı):

   | Ayar | Toplam |
   |---|---|
   | Tauri varsayılanı | 78,8 MB |
   | `--disable-gpu` (**alındı**) | 66,7 MB |
   | `--in-process-gpu` | 71,4 MB |
   | `--disable-gpu --in-process-gpu` | 63,4 MB |
   | `--disable-gpu` + `NetworkServiceInProcess2` | 62,4 MB |

   Son ikisi GPU'yu ya da ağ hizmetini tarayıcı sürecinin içine alıyor: oradaki bir çökme bütün
   pencereyi boşaltır ve WebView2 kendi kendine güncellendiği için az kullanılan ayarlar
   ileride sessizce bozulabilir. 6-7 MB için alınmadı.
5. **Arka planda WebView2 belleği kısılıyor** (`src/bellek.rs`). Pencere simge durumuna
   küçültülünce hemen, odağı kaybedip 30 sn geri almazsa WebView2'ye
   `MemoryUsageTargetLevel = Low` deniyor; odak gelince `Normal`. Ölçümde sayfa süreci
   20,5 → 4,1 MB'a indi; tarayıcı süreci (~28 MB) bu ayarla kırpılmıyor. Odak olayları Tauri'de
   WebView2'nin `GotFocus`/`LostFocus` olaylarından geliyor: sayfa içine tıklamak odak kaybı
   sayılmıyor, pencere öne gelince wry odağı sayfaya taşıdığı için düzey normale dönüyor.
   Ölçülen yol simge durumu; "30 sn arka planda" yolu aynı çağrıyı yapıyor ama sınamak için
   kullanıcının odağını almak gerektiğinden ölçülmedi.

### Çıktı değişmedi

Aynı karşılaştırma aracı 1.7.0 ve 1.7.1 koduyla derlendi (sürüm derlemesi). 10 girdi
(PNG, 12 MP JPEG, taranmış JPEG, ekran görüntüsü PNG, EXIF 6'lı dikey fotoğraf, BMP, TIFF, GIF,
36 MB'lık BMP fotoğraf, 15 MB gürültü PNG) × 4 basamak için belgeye girecek baytlar, piksel
ölçüleri, yapıştırma kestirimleri, listede görünen bilgiler ve bütün resimlerden iki sayfa
düzeninde kurulan belgeler: **49 dosyanın SHA-256'sı aynı.**

Arayüz: 1.7.0 ve 1.7.1 pencereleri üç resimle `PrintWindow` ile yakalandı. Piksellerin %3,7'si
farklı; farklar yalnızca yazı ve köşe kenarlarındaki kenar yumuşatmasında (GPU'suz çizim, en
büyük fark 65/255) ve önizlemelerde. Gözle ayırt edilmiyor.

53 birim test ve `cargo clippy --all-targets -- -D warnings` temiz. Yeni testler: dikey
fotoğrafın PNG'si tutulmadan her basamakta PNG kaynaklı resimle bayt bayt aynı sonucu vermesi,
PNG'si küçük girdide PNG'nin tutulması, önizlemenin ölçüsü ve biçimi, parça parça base64'ün
tek seferlikle aynılığı, sayılan belge boyutunun üretilen dosyayla aynılığı, dosyaya akan
belgenin bellekte kurulanla aynılığı, Orijinal'den çıkınca önbelleğin bırakılması.

### Kalan

- 12 MP fotoğraf İdeal'e küçültülürken çözülmüş resim (36 MB) ile `image` sandığının Lanczos3
  ara belleği (~75 MB) birkaç yüz milisaniye birlikte duruyor; uygulama süreci bu anda
  ~130 MB'a çıkıyor. Başka bir yeniden örnekleyici gömülen baytları değiştireceği için
  dokunulmadı.
- Dikey fotoğrafın listede gösterilen "Orijinal" boyutu için PNG yüklenirken yine bir kez
  kodlanıyor; 9 resmin (3'ü dikey) eklenmesi 10,2 → 9,8 sn, belirgin değişmedi.

## 1.7.2 sürümü — UDE'yi bulma

Ortam: Windows 11 Pro 26200 (ofis), UDE 5.4.20 (`C:\Uyap\Uyap Kelime Islemci`, MSI kurulumu;
kurulu programlar listesinde adı "Uyap Kelime İşlemci").

### Sahadaki hata

Bir kullanıcı 1.7.1'de açılışta "UYAP Doküman Editörü bu bilgisayarda bulunamadı" hatası aldı;
UDE kurulu ve güncel. 1.7.1 UDE'yi yalnızca `HKCR\.udf` → ProgID → `shell\open\command` yolundan
arıyordu.

`HKCR`, kullanıcı (`HKCU\Software\Classes`) ve makine (`HKLM\Software\Classes`) kayıtlarının
birleşimi; aynı değer ikisinde de varsa kullanıcınınki geçer. Birleşme değer düzeyinde:
kullanıcı anahtarı varsayılan değer taşımıyorsa makinenin değeri görünüyor (bu bilgisayarda 12
uzantıda bakıldı: `.bat`, `.docx`, `.csv` …). Yani hatanın çıkması için kullanıcı kökündeki
`.udf` varsayılan değerinin *dolu* olması ve UDE'den başka bir yeri göstermesi gerekir.

Bu bilgisayarda tam olarak böyle bir kayıt bulundu (gerçek kayıt defteri, WMI ile okundu):

| Yer | Değer |
|---|---|
| `HKLM\Software\Classes\.udf` | `Adalet Bakanlığı.Uyap Kelime İşlemci` (UDE kurucusu) |
| `…\Adalet Bakanlığı.Uyap Kelime İşlemci\shell\open\command` | `"C:\Uyap\Uyap Kelime Islemci\Uyap Doküman Editörü.exe" "getNewWPInstance" "EDITOR_TYPE_DOCUMENT" "%1" "%~s1"` |
| `HKCU\Software\Classes\.udf` | `udf_auto_file` ("Birlikte aç" ile bir exe seçilince oluşan kayıt) |
| `…\udf_auto_file\shell\open\command` | `"C:\Users\…\Desktop\kopya.exe" "%1"`; dosya artık yok |
| `…\Explorer\FileExts\.udf\UserChoiceLatest\ProgId` | `Adalet Bakanlığı.Uyap Kelime İşlemci` |

Çift tıklama Windows'un `UserChoiceLatest` seçimine baktığı için UDE'yi açıyor; `HKCR\.udf` ise
silinmiş `kopya.exe`'yi gösteriyor. 1.7.1 exe'nin diskte olup olmadığına bakmadığı için bu
bilgisayarda şans eseri "UDE var" dedi. Kullanıcı kökündeki ProgID'nin açma komutu hiç yoksa
(örneğin kaldırılmış bir programdan kalmışsa) "UDE yok" der. Sahadaki hatanın en olası nedeni
bu; kullanıcının bilgisayarına bakılamadı.

### Yeni arama sırası (`src-tauri/src/ude.rs`)

1. ProgID kayıtları: kullanıcının seçimi (`UserChoiceLatest`, `UserChoice`), `.udf`'nin birleşik,
   makine ve kullanıcı kökündeki değerleri ve kurucunun bilinen ProgID adları; her biri üç kökte.
   Komut UDE'nin `getNewWPInstance` jetonunu taşımalı, exe diskte olmalı.
2. Kurulu programlar listesi: adı "Uyap Kelime …" olan kaydın `InstallLocation`'ı (64 ve 32 bit
   görünüm, HKCU).
3. `C:\Uyap\Uyap Kelime Islemci`.
4. Kullanıcının **UDE'nin yerini göster** ile seçtiği exe (ayarlarda `ude_yolu`; adı
   "uyap … doküman … .exe" değilse kabul edilmez). Düğme yalnızca UDE bulunamadığında görünür.

Açma: 1. yoldan bulunduysa belge o kaydın komutuyla açılıyor (`ShellExecuteExW` +
`SEE_MASK_CLASSKEY`); kullanıcının `.udf` için seçtiği varsayılan program başka bir şey olsa bile
UDE açılır. Öbür yollarda exe doğrudan `getNewWPInstance EDITOR_TYPE_DOCUMENT "<belge>"` ile
çalıştırılıyor. Kabuk yolu hata verirse doğrudan çalıştırma deneniyor.

### Gerçek UDE ile sınama

Kullanıcının ekranına dokunmamak için UDE ayrı, görünmeyen bir masaüstünde (`CreateDesktop`)
açıldı; pencere başlığı `EnumDesktopWindows` ile okundu, süreç komut satırı WMI'dan alındı.
Belgenin adı Türkçe harfli (`Dilekçe eki ğüşıöç ….udf`). UDE'nin `.uki` ayar dosyaları denemeden
önce yedeklenip sonra geri kondu.

| Yol | Sonuç |
|---|---|
| `ShellExecuteExW` + `SEE_MASK_CLASSKEY` (HKLM'deki UDE ProgID'si) | Belge 2,6 sn'de açıldı; başlık "Doküman Editörü v5.4.20 - Dilekçe eki ğüşıöç ….udf (…)" |
| Exe doğrudan, 3 argüman | Belge 2,1 sn'de açıldı |

Kabuğun UDE'ye verdiği komut satırı `… "EDITOR_TYPE_DOCUMENT" "C:\…\Dilekçe eki ğüşıöç ….udf"
"%~s1"`: `%~s1` açılmıyor, UDE'ye olduğu gibi gidiyor ve yok sayılıyor. Doğrudan çalıştırmada üç
argüman yeterli.

`RegGetValueW` + `RRF_RT_REG_SZ`, `REG_EXPAND_SZ` değerleri de açarak okuyor
(`batfile\shell\edit\command` ile denendi); komutun genişletilebilir yazılması sorun değil.

### Uygulamanın kendisiyle, sahadaki durumda (uçtan uca)

`tools/ude-sinama.ps1 -Kip uygulama`: uygulama görünmeyen masaüstünde açıldı, sayfa WebView2
hata ayıklama kapısından sürüldü (açılış durumu, "UDE'nin yerini göster" komutu, resim ekleyip
**UDF'de aç**). Sahadaki bozuk ilişki, `HKCU\Software\Classes\.udf` varsayılanı açma komutu
olmayan bir ProgID'yi gösterecek biçimde canlandırıldı. Sınama paketli (MSIX) bir uygulamanın
içinden çalıştı; oradan yapılan `HKCU` yazımı paketin sanal kayıt defterinde kaldı, gerçek kayıt
defteri değişmedi (WMI ile okundu). 1.7.2 olarak sınanan dosya, CI'ın taslak sürüme yüklediği
taşınabilir exe (SHA-256 GitHub'ın özetiyle aynı).

| Durum | 1.7.1 (kurulu olan) | 1.7.2 (taslak sürüm) |
|---|---|---|
| Canlandırılan bozuk ilişki: açılış | "UYAP Doküman Editörü bu bilgisayarda bulunamadı …" — sahadaki ekran görüntüsünün aynısı; **UDF'de aç** aynı hatayı veriyor | UDE bulundu, hata yok, "yerini göster" düğmesi gizli |
| Canlandırılan bozuk ilişki: **UDF'de aç** | — | "Belge hazır (8 KB) ve UYAP Doküman Editörü'nde açıldı."; UDE penceresi başlıkta `Ekran görüntüsü ….udf`, açılıştan 6,1 sn sonra |
| Bu bilgisayarın gerçek durumu (`kopya.exe`): **UDF'de aç** | Belge açıldı (şans eseri) | Belge açıldı, 5,8 sn |
| `ude_yerini_kaydet("C:\Windows\notepad.exe")` | — | Reddedildi: "Seçilen dosya UYAP Doküman Editörü değil …" |
| `ude_yerini_kaydet(<UDE exe>)`, ardından arayüzün ayar kaydı | — | Yol kaydedildi ve ayar kaydından sonra da yerinde |

1.7.2'de UDE'ye giden komut satırı kayıtlı komutun biçiminde
(`"…Uyap Doküman Editörü.exe" "getNewWPInstance" "EDITOR_TYPE_DOCUMENT" "<belge>" "%~s1"`): belge
UDE'nin kaydıyla, kabuk üzerinden açıldı. CI'da 1.7.2'nin bütün birim testleri ve
`cargo clippy --all-targets -- -D warnings` temiz.

Yayımlanınca `releases/latest/download/UDF-Resimcisi-kurulum.exe` indirildi (SHA-256 sürümdeki
özetle aynı) ve ofis bilgisayarına `/S` ile kuruldu: exe dosya sürümü ve gerçek `HKCU` kaldırma
kaydı 1.7.2.

### Kalan

- Kullanıcının bilgisayarına bakılamadı; neden, aynı hatayı veren en olası durum. Başka bir
  nedende de **UDE'nin yerini göster** çıkış yolu var.
- "UDE'nin yerini göster" düğmesinin görünür hâli ve dosya seçme penceresi elle denenmedi: bu
  bilgisayarda UDE her yoldan bulunduğu için düğme çıkmıyor. Arkasındaki komut yukarıda sınandı.
- UYAP'ın sitesindeki `.exe` kurucusu incelenmedi; bu bilgisayardaki kurulum MSI. Başka bir yere
  kuruyorsa UDE yine `.udf` kaydından, kurulu programlar listesinden ya da elle gösterilerek
  bulunur.

## 1.7.3 sürümü — kaldırıcı yalnızca uygulamanın belgelerini siler

### Hata

1.3.0'dan beri kaldırıcıda **Uygulama verilerini sil** işaretlenince kanca, kaydetme
klasöründeki **bütün** `*.udf` dosyalarını kalıcı olarak siliyordu (`Delete "$0\*.udf"`).
Hangisini uygulamanın ürettiğine bakılmıyordu. Kaydetme klasörü Masaüstü ya da bir dava
klasörüyse kullanıcının UDE'de yazdığı dilekçeler de gidiyordu. Uygulamanın ürettiği bir belgeyi
UDE'de açıp üzerine dilekçe yazan kullanıcının belgesi de aynı şekilde siliniyordu. README "uygulamanın
üretmediği dosyalara dokunulmaz" diyordu; bu `.udf` için doğru değildi. Kişisel veri ve
güvenlik denetiminde bulundu.

### Çözüm

- Uygulama ürettiği her belgeyi yolu, boyutu ve son değişme zamanıyla
  `%APPDATA%\UDF Resimcisi\uretilenler.json`'a yazar (`src-tauri/src/uretilenler.rs`); diskte
  artık olmayanlar listeden düşer.
- Kaldırıcı (`src-tauri/nsis/hooks.nsh`) program dosyalarını silmeden önce exe'yi kendi geçici
  klasörüne kopyalar, ardından `--kaldirma-temizligi` ile çalıştırır. Uygulama pencere açmadan
  yalnızca listede olup boyutu ve değişme zamanı kayıttakiyle aynı kalan `.udf` belgelerini siler.
- Kaydetme klasörü yalnızca adı `UDF Resimcisi` ise (varsayılan klasör) ve boş kaldıysa kaldırılır.
- Her hata "dokunma" yönüne düşer: liste yoksa ya da bozuksa, exe kopyalanamadıysa, belge
  değiştiyse silinmez.
- Kayıt defterine artık yazılmıyor (`kayit.rs` silindi); 1.7.2'ye kadar yazılan
  `HKCU\Software\UDF Resimcisi` anahtarını kaldırıcı yine siler.
- 1.7.3'ten önce üretilen belgeler listede olmadığı için kaldırıcı onlara dokunmaz.

### Sınama

Birim testleri (`uretilenler.rs`, 9 test): üretilen ve değişmemiş belge silinir; aynı klasördeki
listede olmayan `.udf` kalır; üzerine kaydedilen (boyutu ya da yalnızca değişme zamanı farklı)
belge kalır; kendi klasörü boşalınca kalkar, kullanıcının seçtiği klasör boş kalsa da durur;
liste yoksa ya da bozuksa hiçbir şey silinmez; listeye elle `.udf` olmayan yol yazılsa da
silinmez; Türkçe harfli yol. `cargo test` 71/71, `cargo clippy --all-targets -- -D warnings` temiz.

Kaldırma kancası (`tools/kaldirma-sinama.ps1`; kanca, Tauri şablonundaki sırayla küçük bir NSIS
programına gömüldü, 1.7.3'ün yerelde derlenen gerçek exe'siyle):

| Dosya / yer | 1.7.2, kutu işaretli | 1.7.3, kutu işaretli | 1.7.3, kutu işaretsiz | 1.7.3, güncelleme |
|---|---|---|---|---|
| Üretilen belge, kendi klasöründe | silindi | silindi | duruyor | duruyor |
| Üretilen belge, Masaüstü'nde | silindi | silindi | duruyor | duruyor |
| Kullanıcının dilekçesi (listede yok), Masaüstü'nde | **silindi** | duruyor | duruyor | duruyor |
| Üretilen, sonra üzerine kaydedilen | **silindi** | duruyor | duruyor | duruyor |
| `not.txt`, Masaüstü'nde | duruyor | duruyor | duruyor | duruyor |
| `Belgeler\UDF Resimcisi` (boşaldı) | kalktı | kalktı | duruyor | duruyor |
| Masaüstü klasörü | duruyor | duruyor | duruyor | duruyor |
| Ayar klasörü, kayıt defteri anahtarı | kalktı | kalktı | duruyor | duruyor |

İlk denemede sınama programının içindeki kurulum yolu Türkçe harfleri bozan bir kodlamayla
yazıldı; kanca exe'yi bulamadı ve hiçbir belgeye dokunmadı. Bu, "exe yoksa dokunma" yolunun
da çalıştığını gösterdi; sınama dosyası BOM'lu UTF-8'e çevrilince tablo yukarıdaki gibi çıktı.

Uçtan uca (`tools/ude-sinama.ps1 -Kip uygulama`, görünmeyen masaüstü, gerçek UDE 5.4.20, `APPDATA`
geçici klasöre çevrildi): **UDF'de aç** belgeyi üretti (8 KB) ve UDE'de açtı; liste o belgeyi
doğru boyutla yazdı. Belge UDE'de açılıp kapatılana dek 100 ms arayla izlendi: boyut ve değişme
zamanı listedeki kayıtla birebir aynı kaldı. UDE açarken belgeye yazmıyor, yani UDE'de açılmış
ama üzerine kaydedilmemiş belge kaldırılırken silinir.
