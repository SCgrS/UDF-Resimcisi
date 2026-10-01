//! Uygulamanın ürettiği belgelerin listesi ve kaldırırken yapılan belge temizliği.
//!
//! Kaldırıcıdaki "Uygulama verilerini sil" kutusu işaretlenince uygulamanın ürettiği belgeler de
//! silinir (1.3.0). 1.7.2'ye kadar kaldırıcı bunu kaydetme klasöründeki **bütün** `.udf`
//! dosyalarını silerek yapıyordu: klasör Masaüstü ya da bir dava klasörüyse kullanıcının UDE'de
//! yazdığı belgeler de gidiyordu. Artık uygulama ürettiği her belgeyi yoluyla, boyutuyla ve son
//! değişme zamanıyla bu listeye yazar. Kaldırıcı uygulamayı `--kaldirma-temizligi` ile çalıştırır
//! (`nsis/hooks.nsh`) ve yalnızca listede olup o günden beri değişmemiş belgeler silinir. UDE'de
//! açılıp üzerine kaydedilmiş belge, listede olmayan her dosya ve kullanıcının seçtiği klasör
//! yerinde kalır; klasör yalnızca uygulamanın kendi klasörüyse ve boş kaldıysa kaldırılır.
//!
//! Liste yazılamazsa ya da okunamazsa belge silinmez: her hata "dokunma" yönüne düşer.
//!
//! Dosya: `%APPDATA%\UDF Resimcisi\uretilenler.json` (ayar klasörü; kaldırıcı ardından onu da siler).

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Kaldırıcının uygulamayı çağırdığı komut satırı anahtarı.
pub const KALDIRMA_ANAHTARI: &str = "--kaldirma-temizligi";

/// Uygulamanın varsayılan kaydetme klasörünün adı. Yalnızca bu adı taşıyan klasör, boş kalırsa
/// kaldırılır; kullanıcının seçtiği başka bir klasör ("Belgelerim"in kendisi bile olabilir) asla.
const UYGULAMA_KLASORU: &str = "UDF Resimcisi";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct UretilenBelge {
    yol: String,
    boyut: u64,
    /// Son değişme zamanı: UNIX başlangıcından bu yana nanosaniye.
    degisme: u64,
}

/// Temizliğin sonucu (sınamalar için; kaldırıcı sonuca bakmaz).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Temizlik {
    pub silinen: usize,
    pub korunan: usize,
}

fn liste_dosyasi() -> PathBuf {
    crate::settings::ayar_klasoru().join("uretilenler.json")
}

/// Uygulamanın yeni ürettiği belgeyi listeye ekler. Hata olursa belge listeye girmez; kaldırıcı
/// da ona dokunmaz.
pub fn kaydet(belge: &Path) -> Result<()> {
    ekle(&liste_dosyasi(), belge)
}

/// Kaldırıcının çağırdığı temizlik: listedeki, o günden beri değişmemiş belgeleri siler.
pub fn temizle() -> Temizlik {
    temizle_listeden(&liste_dosyasi())
}

/// Dosyanın boyutu ve son değişme zamanı.
fn izi(yol: &Path) -> Option<(u64, u64)> {
    let m = std::fs::metadata(yol).ok()?;
    if !m.is_file() {
        return None;
    }
    let degisme = m.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_nanos();
    Some((m.len(), u64::try_from(degisme).ok()?))
}

fn oku(liste: &Path) -> Vec<UretilenBelge> {
    std::fs::read_to_string(liste)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn ekle(liste: &Path, belge: &Path) -> Result<()> {
    let (boyut, degisme) =
        izi(belge).ok_or_else(|| anyhow::anyhow!("Belgenin bilgisi okunamadı: {}", belge.display()))?;
    let yol = belge.to_string_lossy().to_string();

    // Artık diskte olmayan belgeler listeden düşer; liste her üretimde sınırsız büyümesin.
    let mut belgeler: Vec<UretilenBelge> = oku(liste)
        .into_iter()
        .filter(|b| b.yol != yol && Path::new(&b.yol).is_file())
        .collect();
    belgeler.push(UretilenBelge { yol, boyut, degisme });

    if let Some(klasor) = liste.parent() {
        std::fs::create_dir_all(klasor)?;
    }
    // Önce geçici dosyaya yazılıp yerine taşınır: yarım yazılmış liste kalmasın.
    let gecici = liste.with_extension("json.yeni");
    std::fs::write(&gecici, serde_json::to_string_pretty(&belgeler)?)?;
    std::fs::rename(&gecici, liste)?;
    Ok(())
}

fn temizle_listeden(liste: &Path) -> Temizlik {
    let mut sonuc = Temizlik::default();
    let mut klasorler: Vec<PathBuf> = Vec::new();

    for b in oku(liste) {
        let yol = Path::new(&b.yol);
        let udf_mi = yol
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("udf"));
        let degismemis = izi(yol) == Some((b.boyut, b.degisme));
        if udf_mi && degismemis && std::fs::remove_file(yol).is_ok() {
            sonuc.silinen += 1;
            if let Some(k) = yol.parent() {
                if !klasorler.iter().any(|x| x == k) {
                    klasorler.push(k.to_path_buf());
                }
            }
        } else if yol.exists() {
            sonuc.korunan += 1;
        }
    }

    for k in klasorler {
        let uygulamanin = k
            .file_name()
            .is_some_and(|a| a.to_string_lossy().eq_ignore_ascii_case(UYGULAMA_KLASORU));
        if uygulamanin {
            // Boş değilse hata verir ve klasör yerinde kalır.
            let _ = std::fs::remove_dir(&k);
        }
    }
    sonuc
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Her sınama kendi geçici klasöründe çalışır.
    fn kok(ad: &str) -> PathBuf {
        let k = std::env::temp_dir().join(format!("udfres-uretilen-{ad}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&k);
        std::fs::create_dir_all(&k).unwrap();
        k
    }

    fn yaz(yol: &Path, icerik: &[u8]) {
        std::fs::create_dir_all(yol.parent().unwrap()).unwrap();
        std::fs::write(yol, icerik).unwrap();
    }

    #[test]
    fn yalnizca_uretilen_ve_degismemis_belge_silinir() {
        let k = kok("temel");
        let liste = k.join("ayar").join("uretilenler.json");
        let klasor = k.join("Masaüstü");

        let uretilen = klasor.join("resim.udf");
        yaz(&uretilen, b"uygulamanin belgesi");
        ekle(&liste, &uretilen).unwrap();

        // Kullanıcının UDE'de yazdığı, aynı klasördeki belge: listede yok.
        let dilekce = klasor.join("dilekce.udf");
        yaz(&dilekce, b"kullanicinin belgesi");

        let sonuc = temizle_listeden(&liste);
        assert_eq!(sonuc, Temizlik { silinen: 1, korunan: 0 });
        assert!(!uretilen.exists(), "uygulamanın ürettiği belge silinmeli");
        assert!(dilekce.exists(), "listede olmayan .udf'ye dokunulmamalı");
        assert!(klasor.exists(), "kullanıcının klasörü kaldırılmamalı");
        std::fs::remove_dir_all(&k).ok();
    }

    #[test]
    fn udede_uzerine_kaydedilen_belge_korunur() {
        let k = kok("degisen");
        let liste = k.join("uretilenler.json");
        let belge = k.join("UDF Resimcisi").join("resim.udf");
        yaz(&belge, b"uretildigi hali");
        ekle(&liste, &belge).unwrap();

        // Kullanıcı belgeyi UDE'de açıp üzerine dilekçesini yazdı.
        yaz(&belge, b"kullanicinin yazdigi dilekce, daha uzun");

        let sonuc = temizle_listeden(&liste);
        assert_eq!(sonuc, Temizlik { silinen: 0, korunan: 1 });
        assert!(belge.exists());
        std::fs::remove_dir_all(&k).ok();
    }

    #[test]
    fn ayni_boyutta_degisen_belge_de_korunur() {
        let k = kok("ayniboyut");
        let liste = k.join("uretilenler.json");
        let belge = k.join("resim.udf");
        yaz(&belge, b"AAAA");
        ekle(&liste, &belge).unwrap();

        // Boyut aynı, içerik ve değişme zamanı farklı.
        let dosya = std::fs::OpenOptions::new().write(true).open(&belge).unwrap();
        dosya
            .set_modified(std::time::SystemTime::now() + std::time::Duration::from_secs(5))
            .unwrap();
        drop(dosya);

        assert_eq!(temizle_listeden(&liste).silinen, 0);
        assert!(belge.exists());
        std::fs::remove_dir_all(&k).ok();
    }

    #[test]
    fn uygulama_klasoru_bosalinca_kalkar_baskasi_kalmaz() {
        let k = kok("klasor");
        let liste = k.join("uretilenler.json");
        let kendi = k.join("UDF Resimcisi").join("a.udf");
        let secilen = k.join("Davalar").join("b.udf");
        yaz(&kendi, b"a");
        yaz(&secilen, b"b");
        ekle(&liste, &kendi).unwrap();
        ekle(&liste, &secilen).unwrap();

        assert_eq!(temizle_listeden(&liste).silinen, 2);
        assert!(!k.join("UDF Resimcisi").exists(), "boşalan kendi klasörü kaldırılır");
        assert!(k.join("Davalar").exists(), "kullanıcının seçtiği klasör boş kalsa da durur");
        std::fs::remove_dir_all(&k).ok();
    }

    #[test]
    fn uygulama_klasorunde_baska_dosya_varsa_klasor_kalir() {
        let k = kok("doluklasor");
        let liste = k.join("uretilenler.json");
        let belge = k.join("UDF Resimcisi").join("a.udf");
        let not = k.join("UDF Resimcisi").join("not.txt");
        yaz(&belge, b"a");
        yaz(&not, b"kullanicinin notu");
        ekle(&liste, &belge).unwrap();

        temizle_listeden(&liste);
        assert!(!belge.exists());
        assert!(not.exists());
        std::fs::remove_dir_all(&k).ok();
    }

    #[test]
    fn liste_yoksa_ya_da_bozuksa_hicbir_sey_silinmez() {
        let k = kok("bozuk");
        let belge = k.join("UDF Resimcisi").join("a.udf");
        yaz(&belge, b"a");

        assert_eq!(temizle_listeden(&k.join("yok.json")), Temizlik::default());
        let bozuk = k.join("bozuk.json");
        yaz(&bozuk, b"{ bu json degil");
        assert_eq!(temizle_listeden(&bozuk), Temizlik::default());
        assert!(belge.exists());
        std::fs::remove_dir_all(&k).ok();
    }

    #[test]
    fn listede_udf_olmayan_yol_silinmez() {
        // Liste elle değiştirilmiş olsa bile .udf olmayan dosyaya dokunulmaz.
        let k = kok("udfdegil");
        let liste = k.join("uretilenler.json");
        let dosya = k.join("onemli.docx");
        yaz(&dosya, b"x");
        let (boyut, degisme) = izi(&dosya).unwrap();
        let belgeler = vec![UretilenBelge {
            yol: dosya.to_string_lossy().to_string(),
            boyut,
            degisme,
        }];
        yaz(&liste, serde_json::to_string(&belgeler).unwrap().as_bytes());

        assert_eq!(temizle_listeden(&liste).silinen, 0);
        assert!(dosya.exists());
        std::fs::remove_dir_all(&k).ok();
    }

    #[test]
    fn diskte_olmayan_belgeler_listeden_duser() {
        let k = kok("budama");
        let liste = k.join("uretilenler.json");
        let a = k.join("a.udf");
        let b = k.join("b.udf");
        yaz(&a, b"a");
        yaz(&b, b"b");
        ekle(&liste, &a).unwrap();
        std::fs::remove_file(&a).unwrap();
        ekle(&liste, &b).unwrap();

        let belgeler = oku(&liste);
        assert_eq!(belgeler.len(), 1);
        assert_eq!(belgeler[0].yol, b.to_string_lossy());
        std::fs::remove_dir_all(&k).ok();
    }

    #[test]
    fn turkce_karakterli_yol() {
        let k = kok("turkce");
        let liste = k.join("uretilenler.json");
        let belge = k.join("Kullanıcı Ğüşİöç").join("Belgeler").join("İcra dosyası ğüşöç.udf");
        yaz(&belge, b"a");
        ekle(&liste, &belge).unwrap();
        assert_eq!(temizle_listeden(&liste).silinen, 1);
        assert!(!belge.exists());
        std::fs::remove_dir_all(&k).ok();
    }
}
