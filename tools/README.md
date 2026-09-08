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
