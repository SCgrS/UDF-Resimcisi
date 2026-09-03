# Bu klasördeki üçüncü taraf kaynak

`UdeXml.java` ve `UdeDoc.java` bu projeye ait değildir. Bunlar
[saidsurucu/ude-win-x64](https://github.com/saidsurucu/ude-win-x64) projesindeki
(`scripts/pasterich/macospasterich/`) **referans UDF serializer**'ın olduğu gibi kopyasıdır.

* Telif: © 2026 Said Surucu
* Lisans: MIT — tam metin `LICENSE-ude-win-x64` dosyasında.

Neden burada duruyorlar: UDF Resimcisi'nin Rust serializer'ı (`src-tauri/src/udf/serialize.rs`)
bu Java uygulamasının portudur. Doğruluğu, aynı girdi için iki uygulamanın ürettiği
`content.xml`'i **bayt bayt karşılaştırarak** kanıtlandı (bkz. `DOGRULAMA.md`). Karşılaştırmanın
tekrar edilebilmesi için referans kaynağın da depoda bulunması gerekiyor.

`ProtoMain.java`, `BuyukGorsel.java` ve `IconGen.java` bu projeye aittir (MIT, © 2026 Çağrı Şahin);
yalnızca yukarıdaki iki dosya dışarıdan gelmedir.
