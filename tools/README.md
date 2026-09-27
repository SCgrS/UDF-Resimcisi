# Doğrulama araçları

Bunlar uygulamanın parçası değil; `DOGRULAMA.md` ve README'deki ölçümleri üreten yardımcılar.

## `../src-tauri/examples/olcum.rs` — boyut ölçümü

README'deki tabloların kaynağı. Bir resmi her kalite basamağında indirger; belgeye girecek
bitmap'in ölçüsünü, biçimini, `.udf` dosyasının boyutunu ve UDE'ye yapıştırıldığında
(PNG'ye çevrildiğinde) kaplayacağı yeri yazar.

```bash
cd src-tauri
cargo run --release --example olcum -- ../testdata/telefon-12mp.jpg
# Her basamağın .udf'sini ve bitmap'ini de yaz (UDE'de açıp gözle bakmak için):
cargo run --release --example olcum -- ../testdata/dilekce-tarama.jpg --yaz ../testdata/olcum
```

## `bellek-olcumu.ps1` / `cdp.mjs` — bellek ölçümü

`DOGRULAMA.md` "1.7.1 sürümü — bellek" tablolarının kaynağı. Uygulamayı kullanıcının ekranına
dokunmadan açar ve Görev Yöneticisi'nin **Bellek** sütununu (özel çalışma kümesi) uygulama
süreci ile altındaki bütün WebView2 süreçleri için toplar: boşta, 3 resimle (İdeal ve Orijinal),
3'ü dikey 9 fotoğrafla, liste temizlenince ve simge durumunda.

```powershell
tools\bellek-olcumu.ps1 -Derle                      # gizli pencereli derleme + tam ölçüm
tools\bellek-olcumu.ps1 -Exe <exe> -Etiket eski     # hazır bir gizli pencereli derlemeyi ölç
tools\bellek-olcumu.ps1 -Exe <exe> -Kisa -Argumanlar "<WebView2 ayarları>"   # ayar denemesi
```

- Pencere ekran dışı `x`/`y` ile açılamıyor: tao, hiçbir ekrana düşmeyen konumu yok sayıp
  pencereyi Windows'un varsayılan konumunda (sol üstte, görünür) açıyor.
  Bu yüzden `-Derle`, `tauri.conf.json`'daki pencere ayarını gizli ve odaksız (`visible`,
  `focus`: false) derler; betik pencereyi önce ekran dışına taşır, sonra odak vermeden gösterir.
- Resimler WebView2 hata ayıklama kapısından (`cdp.mjs`, Node 22+) sayfanın kendi işlevleriyle
  eklenir. `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` uygulamanın kendi WebView2 ayarlarının yerine
  geçtiği için betik aynı ayarları (`additionalBrowserArgs`) kapının yanına kendisi ekler.
- "Dikey" fotoğraf, `testdata/telefon-12mp.jpg`'ye EXIF `Orientation = 6` eklenerek üretilir.
  Çıktılar `%TEMP%\udf-resimcisi-bellek` altına yazılır.

## `macospasterich/`

`UdeXml.java` ve `UdeDoc.java`, `ude-win-x64` projesindeki **referans UDF serializer**'ın
kopyasıdır. Rust portu (`src-tauri/src/udf/`) bunlarla karşılaştırılarak doğrulandı: aynı girdi
için üretilen `content.xml` bayt-bayt aynı çıkıyor.

```bash
JDK=".../jdk-17/bin"
$JDK/javac -encoding UTF-8 -d out macospasterich/*.java

# Test görseli üret (1 px dama tahtası + ince çizgiler + 8-48 pt metin)
$JDK/java -cp out macospasterich.ProtoMain gen test.png 3000 2000

# Referans serializer ile .udf üret
$JDK/java -cp out macospasterich.ProtoMain udf test.png cikti.udf

# Bir .udf içindeki resimlerin GERÇEK piksel boyutunu ölç
$JDK/java -cp out macospasterich.ProtoMain read cikti.udf
```

`ProtoMain read` en çok işe yarayan komut: UDE'nin kaydettiği dosyalarda base64 MIME satır
sonlarıyla yazıldığı için normal base64 çözücüler patlar; bu araç doğru çözer ve gömülü
bitmap'i ayrı dosyaya da yazar. JDK yoksa aynı işi PowerShell ile yapmak da kolay:
`[Convert]::FromBase64String` boşlukları zaten yok sayar; `content.xml`'i ZIP'ten çıkarıp
`imageData="…"` değerlerini çözmek yeter (1.6 ölçümleri böyle yapıldı).

`BuyukGorsel.java` ölçüm için gerçekçi büyük girdiler üretir (12 MP fotoğraf benzeri JPEG,
sıkışmayan gürültü PNG'si). `IconGen.java` uygulama simgesinin kaynağını çizer.

## `uiauto.ps1` / `capture.ps1`

Arayüzü ve UDE'yi denetlemek için kullanılan pencere yardımcıları: pencere bulma, öne getirme,
tıklama/tuş gönderme, pencere taşıma. Uygulamanın kendisi bunları kullanmaz; yalnızca elle
doğrulama turlarında işe yarar. (1.1'de var olan arka planda pano kopyalama özelliği 1.2'de
kaldırıldı; bu betiklerdeki pano ve saydamlaştırma yardımcıları o dönemden kalmadır.)

`capture.ps1 -Print` arka plandaki pencereyi `PrintWindow` ile yakalar — üstte başka pencere
varken bile arayüzü denetlemeye yarar.

UDE yapıştırma sınaması (1.6) da aynı yaklaşımla yapıldı: belgeyi dosya ilişkisiyle aç,
pencereyi başlığından bul, `Ctrl+A` / `Ctrl+C` / `Ctrl+N` / `Ctrl+V` / `Ctrl+S` gönder,
dosya seçicisine tam yolu yaz. Pano sıra numarası (`GetClipboardSequenceNumber`) her adımda
denetlendi.
