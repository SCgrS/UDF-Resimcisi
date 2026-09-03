# Doğrulama araçları

Bunlar uygulamanın parçası değil; `DOGRULAMA.md`'deki ölçümleri üreten yardımcılar.

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
bitmap'i ayrı dosyaya da yazar.

`BuyukGorsel.java` ölçüm için gerçekçi büyük girdiler üretir (12 MP fotoğraf benzeri JPEG,
sıkışmayan gürültü PNG'si). `IconGen.java` uygulama simgesinin kaynağını çizer.

## `uiauto.ps1` / `capture.ps1`

`ude.rs`'teki Windows otomasyonunun prototipleri; pencere bulma, `AttachThreadInput` ile öne
getirme, `SendInput` ile tıklama/tuş gönderme ve `GetClipboardSequenceNumber` ile pano
doğrulaması burada önce denendi, sonra Rust'a taşındı.

`capture.ps1 -Print` arka plandaki pencereyi `PrintWindow` ile yakalar — üstte başka pencere
varken bile arayüzü denetlemeye yarar.

## `gizle-dene.ps1`

1.1'deki **görünmeden kopyalama**nın deney dosyası. UDE'yi açar, hem açılış görselini
(`JavaSplash`) hem belge penceresini (`SunAwtFrame`) yerinde saydamlaştırır, `Ctrl+A`/`Ctrl+C`
gönderir, panonun değiştiğini doğrular ve pencereyi kapatır. Zaman damgalarıyla birlikte
çalıştığı için "kaç ms'de ne oluyor" ölçümü buradan çıktı.

Önemli: pencereyi **taşımak** yerine saydamlaştırmanın sebebi, taşımanın UDE'nin
`~/.uki/tercihler.xml` dosyasındaki `win_posx`/`win_posy` değerini kalıcı bozması. Ayrıntı
`DOGRULAMA.md` içinde.
