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

## `ude-sinama.ps1` — UDE ile açma sınaması

`DOGRULAMA.md` "1.7.2 sürümü — UDE'yi bulma" sonuçlarının kaynağı. Her şey ayrı, görünmeyen bir
masaüstünde (`CreateDesktop`) olur: orada başlayan süreçlerin ve açtıkları UDE'nin pencereleri
kullanıcının ekranına düşmez, odak çalmaz. UDE'nin açtığı pencere başlıktaki belge adından,
UDE'ye giden komut satırı WMI'dan okunur.

```powershell
tools\ude-sinama.ps1 -Kip kayit                     # belge UDE'nin kaydıyla (ShellExecuteExW + SEE_MASK_CLASSKEY)
tools\ude-sinama.ps1 -Kip dogrudan                  # UDE'nin exe'si jetonlarla doğrudan
tools\ude-sinama.ps1 -Kip uygulama -Exe <exe>       # uygulamanın kendisi: açılış, "UDE'nin yerini göster", UDF'de aç
```

- UDE açıksa başlamaz: yeni belge kullanıcının açık UDE'sine giderdi.
- UDE'nin `%USERPROFILE%\.uki` ayar dosyaları (son açılanlar listesi dahil) ve uygulama kipinde
  uygulamanın ayar dosyası yedeklenip iş bitince geri konur; çalışma klasörü silinir.
- Uygulama kipinde sürüm denetimi ağa çıkmaz; sayfa `cdp.mjs` ile sürülür.
- 1.7.2 sınamaları paketli (MSIX) bir uygulamanın içinden çalıştırıldı. Oradan yapılan `HKCU`
  yazımları paketin sanal kayıt defterinde kalıyor (WMI ile gerçek kayıt defterinden okunarak
  denetlendi); sahadaki bozuk `.udf` ilişkisi gerçek kayıt defterine dokunmadan böyle
  canlandırıldı. Başka bir ortamda aynı yazım gerçek kayıt defterini değiştirir.
- Uygulama kipi `ayarlar.json`'u geri koyar ama üretilen belgelerin listesine
  (`uretilenler.json`) dokunmaz. 1.7.3'te betik `APPDATA` geçici bir klasöre çevrilerek
  çalıştırıldı; listeye o klasörde bakıldı.

## `kaldirma-sinama.ps1` — kaldırma kancası

`DOGRULAMA.md` "1.7.3 sürümü" tablosunun kaynağı. `src-tauri/nsis/hooks.nsh`'i Tauri'nin
kaldırıcısındaki sırayla (PREUNINSTALL, program exe'sinin silinmesi, POSTUNINSTALL) küçük bir
NSIS programına gömer ve uygulamanın gerçek exe'siyle çalıştırır. Üç durumu dener: kutu
işaretli, işaretsiz, güncelleme (`/UPDATE`). Gerçek ayar klasörüne, Belgelerim'e ve kayıt
defterine dokunmaz: kancadaki bu yollar sınama klasörüne çevrilir, çevrilmemiş bir yol kalırsa
betik durur; exe'ye `APPDATA` olarak sınama klasörü verilir.

```powershell
tools\kaldirma-sinama.ps1 -Exe src-tauri\target\release\udf-resimcisi.exe
git show v1.7.2:src-tauri/nsis/hooks.nsh > $env:TEMP\eski.nsh
tools\kaldirma-sinama.ps1 -Exe <herhangi bir exe> -Kanca $env:TEMP\eski.nsh   # 1.7.2'nin kancası
```

`makensis` Tauri'nin indirdiği NSIS'ten (`%LOCALAPPDATA%\tauri\NSIS`) alınır; bir kez
`npx tauri build` çalıştırılmış olmalı.

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
