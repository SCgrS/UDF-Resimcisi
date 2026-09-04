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
seçmiş olabilir.

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
